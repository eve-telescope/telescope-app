//! Laravel Reverb client (Pusher protocol 7) for intel network updates.

use std::time::Duration;

use futures::{SinkExt, StreamExt};
use log::{debug, info, warn};
use serde_json::{Value, json};
use tokio::sync::mpsc::UnboundedSender;
use tokio::task::JoinHandle;
use tokio::time::Instant;
use tokio_tungstenite::tungstenite::Message;

use crate::config;

const CLIENT_NAME: &str = "telescope-rust";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const PONG_TIMEOUT: Duration = Duration::from_secs(30);
const DEFAULT_ACTIVITY_TIMEOUT: Duration = Duration::from_secs(120);
const BACKOFF_MIN: Duration = Duration::from_secs(1);
const BACKOFF_MAX: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RealtimeEvent {
    /// An entry or annotation was created or updated; the network should be refetched.
    EntryChanged {
        network_id: i64,
    },
    EntryDeleted {
        network_id: i64,
        entry_id: i64,
    },
    ScanShared {
        network_id: i64,
        scan_id: i64,
    },
    /// Subscribed (also after a reconnect, so callers can refetch missed state).
    Connected,
    Disconnected,
}

/// Aborts the background connection task when dropped.
pub struct RealtimeHandle {
    task: JoinHandle<()>,
}

impl Drop for RealtimeHandle {
    fn drop(&mut self) {
        self.task.abort();
    }
}

/// Must be called from within a tokio runtime.
pub fn spawn(
    token: String,
    network_id: i64,
    events: UnboundedSender<RealtimeEvent>,
) -> RealtimeHandle {
    let task = tokio::spawn(run(token, network_id, events));
    RealtimeHandle { task }
}

pub fn channel_name(network_id: i64) -> String {
    format!("private-intel-network.{network_id}")
}

pub fn socket_url(key: &str, host: &str, port: u16, tls: bool) -> String {
    let scheme = if tls { "wss" } else { "ws" };
    format!(
        "{scheme}://{host}:{port}/app/{key}?protocol=7&client={CLIENT_NAME}&version={VERSION}&flash=false"
    )
}

#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    pub event: String,
    pub channel: Option<String>,
    /// `data` with string-encoded JSON already decoded.
    pub data: Value,
}

pub fn parse_frame(text: &str) -> Option<Frame> {
    let mut value: Value = serde_json::from_str(text).ok()?;
    let obj = value.as_object_mut()?;
    let event = obj.get("event")?.as_str()?.to_string();
    let channel = obj
        .get("channel")
        .and_then(Value::as_str)
        .map(str::to_string);
    let data = match obj.remove("data") {
        Some(Value::String(s)) => serde_json::from_str(&s).unwrap_or(Value::String(s)),
        Some(other) => other,
        None => Value::Null,
    };
    Some(Frame {
        event,
        channel,
        data,
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConnectionInfo {
    pub socket_id: String,
    pub activity_timeout: Duration,
}

pub fn parse_connection_established(frame: &Frame) -> Option<ConnectionInfo> {
    if frame.event != "pusher:connection_established" {
        return None;
    }
    let socket_id = frame.data.get("socket_id")?.as_str()?.to_string();
    let activity_timeout = frame
        .data
        .get("activity_timeout")
        .and_then(Value::as_u64)
        .filter(|&s| s > 0)
        .map(Duration::from_secs)
        .unwrap_or(DEFAULT_ACTIVITY_TIMEOUT);
    Some(ConnectionInfo {
        socket_id,
        activity_timeout,
    })
}

pub fn subscribe_message(channel: &str, auth: &str) -> String {
    json!({
        "event": "pusher:subscribe",
        "data": { "auth": auth, "channel": channel },
    })
    .to_string()
}

pub fn ping_message() -> String {
    json!({ "event": "pusher:ping", "data": {} }).to_string()
}

pub fn pong_message() -> String {
    json!({ "event": "pusher:pong", "data": {} }).to_string()
}

/// Body pusher-js sends to the auth endpoint.
pub fn auth_body(socket_id: &str, channel: &str) -> String {
    format!(
        "socket_id={}&channel_name={}",
        urlencoding::encode(socket_id),
        urlencoding::encode(channel)
    )
}

/// Laravel names events `App\Events\X` by default, or `.X`/`X` with `broadcastAs`,
/// so only the last segment is significant.
pub fn event_short_name(event: &str) -> &str {
    event.rsplit(['\\', '.']).next().unwrap_or(event)
}

fn id_field(data: &Value, key: &str) -> Option<i64> {
    match data.get(key)? {
        Value::Number(n) => n.as_i64(),
        Value::String(s) => s.parse().ok(),
        _ => None,
    }
}

pub fn map_event(frame: &Frame, network_id: i64) -> Option<RealtimeEvent> {
    if frame.event.starts_with("pusher:") || frame.event.starts_with("pusher_internal:") {
        return None;
    }
    if frame.channel.as_deref() != Some(channel_name(network_id).as_str()) {
        return None;
    }
    match event_short_name(&frame.event) {
        "IntelEntryCreated" | "IntelEntryUpdated" | "AnnotationCreated" | "AnnotationUpdated" => {
            Some(RealtimeEvent::EntryChanged { network_id })
        }
        "IntelEntryDeleted" | "AnnotationDeleted" => Some(RealtimeEvent::EntryDeleted {
            network_id,
            entry_id: id_field(&frame.data, "entry_id")?,
        }),
        "ScanShared" => Some(RealtimeEvent::ScanShared {
            network_id,
            scan_id: id_field(&frame.data, "scan_id")?,
        }),
        _ => None,
    }
}

/// Delay before reconnect attempt `attempt` (0-based): 1s, 2s, 4s, ... capped at 30s.
pub fn backoff(attempt: u32) -> Duration {
    BACKOFF_MIN
        .checked_mul(1u32.checked_shl(attempt).unwrap_or(u32::MAX))
        .unwrap_or(BACKOFF_MAX)
        .min(BACKOFF_MAX)
}

#[derive(Debug)]
enum SessionEnd {
    /// Event receiver is gone; stop for good.
    Closed,
    Failed {
        subscribed: bool,
        reason: String,
    },
}

async fn run(token: String, network_id: i64, events: UnboundedSender<RealtimeEvent>) {
    let mut attempt = 0u32;
    loop {
        match session(&token, network_id, &events).await {
            SessionEnd::Closed => return,
            SessionEnd::Failed { subscribed, reason } => {
                warn!("realtime connection lost: {reason}");
                if subscribed {
                    attempt = 0;
                    if events.send(RealtimeEvent::Disconnected).is_err() {
                        return;
                    }
                }
            }
        }
        let delay = backoff(attempt);
        attempt = attempt.saturating_add(1);
        debug!("realtime reconnecting in {delay:?}");
        tokio::time::sleep(delay).await;
    }
}

async fn authenticate(token: &str, socket_id: &str, channel: &str) -> Result<String, String> {
    let client = crate::api::create_client()?;
    let url = format!("{}/broadcasting/auth", config::api_base_url());
    let response = client
        .post(&url)
        .bearer_auth(token)
        .header(reqwest::header::ACCEPT, "application/json")
        .header(
            reqwest::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(auth_body(socket_id, channel))
        .send()
        .await
        .map_err(|e| format!("auth request failed: {e}"))?;
    let status = response.status();
    if !status.is_success() {
        return Err(format!("auth rejected with {status}"));
    }
    let body: Value = response
        .json()
        .await
        .map_err(|e| format!("invalid auth response: {e}"))?;
    body.get("auth")
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| "auth response missing `auth`".to_string())
}

async fn session(
    token: &str,
    network_id: i64,
    events: &UnboundedSender<RealtimeEvent>,
) -> SessionEnd {
    let failed = |subscribed: bool, reason: String| SessionEnd::Failed { subscribed, reason };

    let reverb = config::reverb();
    let url = socket_url(reverb.key, reverb.host, reverb.port, reverb.tls);
    let (mut ws, _) = match tokio_tungstenite::connect_async(url.as_str()).await {
        Ok(ok) => ok,
        Err(e) => return failed(false, format!("connect failed: {e}")),
    };
    let channel = channel_name(network_id);

    let mut subscribed = false;
    let mut activity_timeout = DEFAULT_ACTIVITY_TIMEOUT;
    let mut last_activity = Instant::now();
    let mut ping_sent: Option<Instant> = None;
    let mut ticker = tokio::time::interval(Duration::from_secs(1));

    loop {
        let message = tokio::select! {
            message = ws.next() => message,
            _ = ticker.tick() => {
                let now = Instant::now();
                if let Some(sent) = ping_sent {
                    if now.duration_since(sent) > PONG_TIMEOUT {
                        return failed(subscribed, "pong timeout".into());
                    }
                } else if now.duration_since(last_activity) > activity_timeout {
                    if let Err(e) = ws.send(Message::text(ping_message())).await {
                        return failed(subscribed, format!("send failed: {e}"));
                    }
                    ping_sent = Some(now);
                }
                continue;
            }
        };

        let text = match message {
            None => return failed(subscribed, "socket closed".into()),
            Some(Err(e)) => return failed(subscribed, format!("socket error: {e}")),
            Some(Ok(Message::Text(text))) => text,
            Some(Ok(Message::Close(frame))) => {
                return failed(subscribed, format!("closed by server: {frame:?}"));
            }
            Some(Ok(_)) => {
                last_activity = Instant::now();
                ping_sent = None;
                continue;
            }
        };
        last_activity = Instant::now();
        ping_sent = None;

        let Some(frame) = parse_frame(text.as_str()) else {
            debug!("realtime: ignoring unparseable frame");
            continue;
        };

        match frame.event.as_str() {
            "pusher:connection_established" => {
                let Some(info) = parse_connection_established(&frame) else {
                    return failed(false, "malformed connection_established".into());
                };
                activity_timeout = info.activity_timeout;
                let auth = match authenticate(token, &info.socket_id, &channel).await {
                    Ok(auth) => auth,
                    Err(e) => return failed(false, e),
                };
                if let Err(e) = ws
                    .send(Message::text(subscribe_message(&channel, &auth)))
                    .await
                {
                    return failed(false, format!("send failed: {e}"));
                }
            }
            "pusher:ping" => {
                if let Err(e) = ws.send(Message::text(pong_message())).await {
                    return failed(subscribed, format!("send failed: {e}"));
                }
            }
            "pusher:pong" => {}
            "pusher_internal:subscription_succeeded" => {
                if frame.channel.as_deref() == Some(channel.as_str()) {
                    info!("realtime subscribed to {channel}");
                    subscribed = true;
                    if events.send(RealtimeEvent::Connected).is_err() {
                        return SessionEnd::Closed;
                    }
                }
            }
            "pusher:error" => {
                return failed(subscribed, format!("server error: {}", frame.data));
            }
            _ => {
                if let Some(event) = map_event(&frame, network_id)
                    && events.send(event).is_err()
                {
                    return SessionEnd::Closed;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_connection_established_with_string_data() {
        let frame = parse_frame(
            r#"{"event":"pusher:connection_established","data":"{\"socket_id\":\"123.456\",\"activity_timeout\":30}"}"#,
        )
        .unwrap();
        let info = parse_connection_established(&frame).unwrap();
        assert_eq!(info.socket_id, "123.456");
        assert_eq!(info.activity_timeout, Duration::from_secs(30));
    }

    #[test]
    fn connection_established_defaults_activity_timeout() {
        let frame = parse_frame(
            r#"{"event":"pusher:connection_established","data":"{\"socket_id\":\"1.2\"}"}"#,
        )
        .unwrap();
        assert_eq!(
            parse_connection_established(&frame)
                .unwrap()
                .activity_timeout,
            DEFAULT_ACTIVITY_TIMEOUT
        );
    }

    #[test]
    fn parses_object_and_missing_data() {
        let frame = parse_frame(r#"{"event":"pusher:ping","data":{}}"#).unwrap();
        assert_eq!(frame.data, json!({}));
        let frame = parse_frame(r#"{"event":"pusher:ping"}"#).unwrap();
        assert_eq!(frame.data, Value::Null);
        assert!(parse_frame("not json").is_none());
        assert!(parse_frame(r#"{"data":{}}"#).is_none());
    }

    #[test]
    fn parses_error_frame() {
        let frame = parse_frame(
            r#"{"event":"pusher:error","data":{"code":4009,"message":"Connection is unauthorized"}}"#,
        )
        .unwrap();
        assert_eq!(frame.data["code"], 4009);
    }

    #[test]
    fn short_name_handles_all_forms() {
        assert_eq!(
            event_short_name(r"App\Events\IntelEntryCreated"),
            "IntelEntryCreated"
        );
        assert_eq!(
            event_short_name("App.Events.IntelEntryCreated"),
            "IntelEntryCreated"
        );
        assert_eq!(event_short_name(".ScanShared"), "ScanShared");
        assert_eq!(event_short_name("ScanShared"), "ScanShared");
    }

    fn event_frame(event: &str, data: &str, network_id: i64) -> Frame {
        let raw = json!({
            "event": event,
            "channel": channel_name(network_id),
            "data": data,
        });
        parse_frame(&raw.to_string()).unwrap()
    }

    #[test]
    fn maps_change_events() {
        for name in [
            r"App\Events\IntelEntryCreated",
            r"App\Events\IntelEntryUpdated",
            ".AnnotationCreated",
            "AnnotationUpdated",
        ] {
            let frame = event_frame(name, r#"{"entry_id":5}"#, 7);
            assert_eq!(
                map_event(&frame, 7),
                Some(RealtimeEvent::EntryChanged { network_id: 7 })
            );
        }
    }

    #[test]
    fn maps_delete_events() {
        let frame = event_frame(r"App\Events\IntelEntryDeleted", r#"{"entry_id":42}"#, 3);
        assert_eq!(
            map_event(&frame, 3),
            Some(RealtimeEvent::EntryDeleted {
                network_id: 3,
                entry_id: 42
            })
        );
        let frame = event_frame(".AnnotationDeleted", r#"{"entry_id":"43"}"#, 3);
        assert_eq!(
            map_event(&frame, 3),
            Some(RealtimeEvent::EntryDeleted {
                network_id: 3,
                entry_id: 43
            })
        );
        let frame = event_frame("IntelEntryDeleted", r#"{}"#, 3);
        assert_eq!(map_event(&frame, 3), None);
    }

    #[test]
    fn maps_scan_shared() {
        let frame = event_frame(r"App\Events\ScanShared", r#"{"scan_id":99}"#, 1);
        assert_eq!(
            map_event(&frame, 1),
            Some(RealtimeEvent::ScanShared {
                network_id: 1,
                scan_id: 99
            })
        );
    }

    #[test]
    fn ignores_other_channels_and_unknown_events() {
        let frame = event_frame("IntelEntryCreated", "{}", 2);
        assert_eq!(map_event(&frame, 1), None);
        let frame = event_frame("SomethingElse", "{}", 1);
        assert_eq!(map_event(&frame, 1), None);
        let frame = event_frame("pusher_internal:subscription_succeeded", "{}", 1);
        assert_eq!(map_event(&frame, 1), None);
    }

    #[test]
    fn builds_subscribe_message() {
        let msg: Value =
            serde_json::from_str(&subscribe_message("private-intel-network.1", "key:sig")).unwrap();
        assert_eq!(
            msg,
            json!({
                "event": "pusher:subscribe",
                "data": { "auth": "key:sig", "channel": "private-intel-network.1" },
            })
        );
    }

    #[test]
    fn builds_ping_and_pong() {
        let ping: Value = serde_json::from_str(&ping_message()).unwrap();
        assert_eq!(ping["event"], "pusher:ping");
        let pong: Value = serde_json::from_str(&pong_message()).unwrap();
        assert_eq!(pong["event"], "pusher:pong");
    }

    #[test]
    fn builds_auth_body() {
        assert_eq!(
            auth_body("123.456", "private-intel-network.7"),
            "socket_id=123.456&channel_name=private-intel-network.7"
        );
    }

    #[test]
    fn builds_socket_url() {
        assert_eq!(
            socket_url("abc", "ws.example.com", 443, true),
            format!(
                "wss://ws.example.com:443/app/abc?protocol=7&client=telescope-rust&version={VERSION}&flash=false"
            )
        );
        assert!(socket_url("abc", "localhost", 8080, false).starts_with("ws://localhost:8080/"));
    }

    #[test]
    fn backoff_doubles_and_caps() {
        assert_eq!(backoff(0), Duration::from_secs(1));
        assert_eq!(backoff(1), Duration::from_secs(2));
        assert_eq!(backoff(4), Duration::from_secs(16));
        assert_eq!(backoff(5), Duration::from_secs(30));
        assert_eq!(backoff(31), Duration::from_secs(30));
        assert_eq!(backoff(100), Duration::from_secs(30));
    }
}
