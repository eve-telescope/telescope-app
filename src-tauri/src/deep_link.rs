use log::{error, info, warn};
use tauri::{AppHandle, Emitter, Manager};

const DEEP_LINK_SCHEME: &str = "telescope";

/// Event carrying a share code to the frontend (payload: the code string).
pub const SHARE_EVENT: &str = "deep-link-share";

#[derive(Debug, PartialEq, Eq)]
pub enum DeepLink {
    Auth { token: String },
    Share { code: String },
}

/// Parses a telescope:// deep link.
///
/// Tolerant of the variants browsers and URL builders produce: trailing
/// slashes (`telescope://auth/?token=X`), extra query parameters,
/// scheme/host casing, and percent-encoded tokens (decoded by the URL
/// machinery, never by hand).
pub fn parse(raw: &str) -> Option<DeepLink> {
    let parsed = url::Url::parse(raw).ok()?;
    if !parsed.scheme().eq_ignore_ascii_case(DEEP_LINK_SCHEME) {
        return None;
    }

    match parsed.host_str()?.to_ascii_lowercase().as_str() {
        // telescope://auth?token=XXX
        "auth" => parsed
            .query_pairs()
            .find(|(key, _)| key == "token")
            .map(|(_, value)| value.into_owned())
            .filter(|token| !token.is_empty())
            .map(|token| DeepLink::Auth { token }),

        // telescope://s/{code} — the alphanumeric gate keeps the code safe
        // to interpolate into the share API path.
        "s" => {
            let code = parsed.path().trim_matches('/');
            if !code.is_empty() && code.chars().all(|c| c.is_ascii_alphanumeric()) {
                Some(DeepLink::Share {
                    code: code.to_string(),
                })
            } else {
                None
            }
        }

        _ => None,
    }
}

/// Share code from a deep link that arrived before the frontend mounted
/// (cold start via link click); the frontend collects it once on startup.
#[derive(Default)]
pub struct PendingShare(std::sync::Mutex<Option<String>>);

#[tauri::command]
pub fn take_pending_deep_link_share(state: tauri::State<'_, PendingShare>) -> Option<String> {
    state.0.lock().ok().and_then(|mut guard| guard.take())
}

/// Handles a deep link received while the app is running: auth tokens are
/// applied entirely backend-side; share codes are forwarded to the frontend.
pub fn handle_runtime(app: &AppHandle, url: &str) {
    info!("[DeepLink] Runtime URL received: {}", url);
    match parse(url) {
        Some(DeepLink::Auth { token }) => spawn_apply_token(app, token),
        Some(DeepLink::Share { code }) => {
            info!("[DeepLink] Forwarding share code to frontend: {}", code);
            let _ = app.emit(SHARE_EVENT, code);
        }
        None => warn!("[DeepLink] Ignoring unrecognized URL: {}", url),
    }
}

/// Handles deep links present at startup, before the webview exists: auth
/// applies immediately; share codes are parked until the frontend asks.
pub fn handle_startup(app: &AppHandle, url: &str) {
    info!("[DeepLink] Startup URL received: {}", url);
    match parse(url) {
        Some(DeepLink::Auth { token }) => spawn_apply_token(app, token),
        Some(DeepLink::Share { code }) => {
            if let Ok(mut pending) = app.state::<PendingShare>().0.lock() {
                *pending = Some(code);
            }
        }
        None => warn!("[DeepLink] Ignoring unrecognized URL: {}", url),
    }
}

fn spawn_apply_token(app: &AppHandle, token: String) {
    info!("[DeepLink] Applying auth token from deep link");
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        match crate::intel_commands::apply_api_token(&app, token).await {
            Ok(()) => info!("[DeepLink] Auth token applied"),
            Err(e) => error!("[DeepLink] Failed to apply auth token: {}", e),
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_canonical_auth_link() {
        assert_eq!(
            parse("telescope://auth?token=abc123"),
            Some(DeepLink::Auth {
                token: "abc123".into()
            })
        );
    }

    #[test]
    fn parses_auth_link_with_trailing_slash() {
        assert_eq!(
            parse("telescope://auth/?token=abc123"),
            Some(DeepLink::Auth {
                token: "abc123".into()
            })
        );
    }

    #[test]
    fn extracts_token_among_extra_query_params() {
        assert_eq!(
            parse("telescope://auth?token=abc123&source=web&foo=bar"),
            Some(DeepLink::Auth {
                token: "abc123".into()
            })
        );
    }

    #[test]
    fn percent_decodes_the_token() {
        assert_eq!(
            parse("telescope://auth?token=abc%7C123%3D%3D"),
            Some(DeepLink::Auth {
                token: "abc|123==".into()
            })
        );
    }

    #[test]
    fn scheme_and_host_are_case_insensitive() {
        assert_eq!(
            parse("TELESCOPE://AUTH?token=abc"),
            Some(DeepLink::Auth {
                token: "abc".into()
            })
        );
    }

    #[test]
    fn rejects_missing_or_empty_token() {
        assert_eq!(parse("telescope://auth"), None);
        assert_eq!(parse("telescope://auth?token="), None);
        assert_eq!(parse("telescope://auth?other=x"), None);
    }

    #[test]
    fn parses_canonical_share_link() {
        assert_eq!(
            parse("telescope://s/Xy12Za"),
            Some(DeepLink::Share {
                code: "Xy12Za".into()
            })
        );
    }

    #[test]
    fn parses_share_link_with_trailing_slash() {
        assert_eq!(
            parse("telescope://s/Xy12Za/"),
            Some(DeepLink::Share {
                code: "Xy12Za".into()
            })
        );
    }

    #[test]
    fn rejects_share_codes_with_unexpected_characters() {
        assert_eq!(parse("telescope://s/code%20name"), None);
        assert_eq!(parse("telescope://s/"), None);
    }

    #[test]
    fn collapses_dot_segments_before_validation() {
        // ".." is resolved by URL parsing; the alphanumeric gate on the
        // result is what keeps the code safe.
        assert_eq!(
            parse("telescope://s/../etc"),
            Some(DeepLink::Share { code: "etc".into() })
        );
    }

    #[test]
    fn rejects_other_schemes_and_hosts() {
        assert_eq!(parse("https://auth?token=abc"), None);
        assert_eq!(parse("spyglass://auth?token=abc"), None);
        assert_eq!(parse(""), None);
        assert_eq!(parse("not a url at all"), None);
        assert_eq!(parse("telescope://unknown?token=abc"), None);
        assert_eq!(parse("telescope://"), None);
    }
}
