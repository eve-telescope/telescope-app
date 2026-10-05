//! Number, ratio, time and EVE image-server URL formatting.

use chrono::{DateTime, Utc};

const IMAGE_SERVER: &str = "https://images.evetech.net";

/// `Number.prototype.toFixed`: rounds half away from zero on the exact
/// binary value, where Rust's formatter rounds ties to even.
pub fn to_fixed(x: f64, digits: usize) -> String {
    if x.is_nan() {
        return "NaN".to_string();
    }
    if x.is_infinite() {
        return if x > 0.0 { "Infinity" } else { "-Infinity" }.to_string();
    }
    let sign = if x < 0.0 { "-" } else { "" };
    let a = x.abs();
    let body = match round_up_on_tie(a, digits) {
        Some(n) => {
            let s = format!("{n:0>width$}", width = digits + 1);
            if digits == 0 {
                s
            } else {
                let (int, frac) = s.split_at(s.len() - digits);
                format!("{int}.{frac}")
            }
        }
        None => format!("{a:.digits$}"),
    };
    format!("{sign}{body}")
}

/// If `a * 10^digits` is exactly `k + 0.5`, returns `k + 1`.
fn round_up_on_tie(a: f64, digits: usize) -> Option<u128> {
    if digits > 20 || a == 0.0 {
        return None;
    }
    let bits = a.to_bits();
    let exp_bits = ((bits >> 52) & 0x7ff) as i32;
    let frac = bits & ((1u64 << 52) - 1);
    let (mantissa, exp) = if exp_bits == 0 {
        (frac, -1074)
    } else {
        (frac | (1u64 << 52), exp_bits - 1075)
    };
    // twice the scaled value: mantissa * 10^digits * 2^(exp + 1)
    let scaled = u128::from(mantissa) * 10u128.pow(digits as u32);
    let shift = exp + 1;
    let doubled = if shift == 0 {
        scaled
    } else if shift < 0 && shift > -128 {
        let s = (-shift) as u32;
        if scaled.trailing_zeros() < s {
            return None;
        }
        scaled >> s
    } else {
        return None;
    };
    (doubled % 2 == 1).then(|| doubled / 2 + 1)
}

pub fn format_isk(value: f64) -> String {
    if value >= 1e12 {
        format!("{}T", to_fixed(value / 1e12, 1))
    } else if value >= 1e9 {
        format!("{}B", to_fixed(value / 1e9, 1))
    } else if value >= 1e6 {
        format!("{}M", to_fixed(value / 1e6, 1))
    } else if value >= 1e3 {
        format!("{}K", to_fixed(value / 1e3, 0))
    } else {
        to_fixed(value, 0)
    }
}

/// Kills per loss, or the raw kill count when nothing was lost so undefeated
/// pilots still sort sensibly.
pub fn kd_ratio_value(destroyed: i64, lost: i64) -> f64 {
    if lost == 0 {
        destroyed as f64
    } else {
        destroyed as f64 / lost as f64
    }
}

pub fn kd_ratio(destroyed: i64, lost: i64) -> String {
    if lost == 0 {
        return if destroyed > 0 { "∞" } else { "0" }.to_string();
    }
    to_fixed(kd_ratio_value(destroyed, lost), 1)
}

pub fn ppk(points: i64, kills: i64) -> f64 {
    if kills == 0 {
        0.0
    } else {
        points as f64 / kills as f64
    }
}

pub fn format_ppk(points: i64, kills: i64) -> String {
    let value = ppk(points, kills);
    if value >= 1000.0 {
        format!("{}k", to_fixed(value / 1000.0, 1))
    } else {
        to_fixed(value, 0)
    }
}

pub fn month_name(month: u32) -> &'static str {
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    month
        .checked_sub(1)
        .and_then(|i| MONTHS.get(i as usize))
        .copied()
        .unwrap_or("???")
}

pub fn character_portrait_url(character_id: i64, size: u32) -> String {
    format!("{IMAGE_SERVER}/characters/{character_id}/portrait?size={size}")
}

pub fn ship_icon_url(type_id: i64, size: u32) -> String {
    format!("{IMAGE_SERVER}/types/{type_id}/icon?size={size}")
}

pub fn ship_render_url(type_id: i64, size: u32) -> String {
    format!("{IMAGE_SERVER}/types/{type_id}/render?size={size}")
}

pub fn corporation_logo_url(corporation_id: i64, size: u32) -> String {
    format!("{IMAGE_SERVER}/corporations/{corporation_id}/logo?size={size}")
}

pub fn alliance_logo_url(alliance_id: i64, size: u32) -> String {
    format!("{IMAGE_SERVER}/alliances/{alliance_id}/logo?size={size}")
}

enum Elapsed {
    JustNow,
    Minutes(i64),
    Hours(i64),
    Days(i64),
}

fn elapsed_since(iso: &str, now: DateTime<Utc>) -> Option<Elapsed> {
    let then = DateTime::parse_from_rfc3339(iso).ok()?;
    let seconds = (now - then.with_timezone(&Utc))
        .num_milliseconds()
        .div_euclid(1000);
    Some(if seconds < 60 {
        Elapsed::JustNow
    } else if seconds < 3600 {
        Elapsed::Minutes(seconds / 60)
    } else if seconds < 86_400 {
        Elapsed::Hours(seconds / 3600)
    } else {
        Elapsed::Days(seconds / 86_400)
    })
}

/// Compact age for the recent-scans list: `now`, `5m`, `3h`, `2d`.
/// Future timestamps read as `now`; `None` if the timestamp doesn't parse.
pub fn relative_time_short(iso: &str, now: DateTime<Utc>) -> Option<String> {
    Some(match elapsed_since(iso, now)? {
        Elapsed::JustNow => "now".to_string(),
        Elapsed::Minutes(m) => format!("{m}m"),
        Elapsed::Hours(h) => format!("{h}h"),
        Elapsed::Days(d) => format!("{d}d"),
    })
}

/// Scan-history age: `just now`, `5m ago`, `3h ago`, `2d ago`.
pub fn relative_time(iso: &str, now: DateTime<Utc>) -> Option<String> {
    Some(match elapsed_since(iso, now)? {
        Elapsed::JustNow => "just now".to_string(),
        Elapsed::Minutes(m) => format!("{m}m ago"),
        Elapsed::Hours(h) => format!("{h}h ago"),
        Elapsed::Days(d) => format!("{d}d ago"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_fixed_rounds_ties_away_from_zero() {
        assert_eq!(to_fixed(2.5, 0), "3");
        assert_eq!(to_fixed(0.5, 0), "1");
        assert_eq!(to_fixed(0.25, 1), "0.3");
        assert_eq!(to_fixed(-2.5, 0), "-3");
        assert_eq!(to_fixed(1.005, 2), "1.00");
        assert_eq!(to_fixed(1.45, 1), "1.4");
    }

    #[test]
    fn to_fixed_basics() {
        assert_eq!(to_fixed(0.0, 0), "0");
        assert_eq!(to_fixed(-0.0, 1), "0.0");
        assert_eq!(to_fixed(-0.4, 0), "-0");
        assert_eq!(to_fixed(3.0, 1), "3.0");
        assert_eq!(to_fixed(0.05, 1), "0.1");
        assert_eq!(to_fixed(123.456, 1), "123.5");
        assert_eq!(to_fixed(f64::NAN, 1), "NaN");
    }

    #[test]
    fn formats_isk_by_magnitude() {
        assert_eq!(format_isk(0.0), "0");
        assert_eq!(format_isk(999.0), "999");
        assert_eq!(format_isk(1_500.0), "2K");
        assert_eq!(format_isk(2_500.0), "3K");
        assert_eq!(format_isk(12_345_678.0), "12.3M");
        assert_eq!(format_isk(1.25e9), "1.3B");
        assert_eq!(format_isk(3.0e12), "3.0T");
    }

    #[test]
    fn kd_ratio_strings() {
        assert_eq!(kd_ratio(0, 0), "0");
        assert_eq!(kd_ratio(5, 0), "∞");
        assert_eq!(kd_ratio(30, 10), "3.0");
        assert_eq!(kd_ratio(1, 3), "0.3");
    }

    #[test]
    fn kd_ratio_values() {
        assert_eq!(kd_ratio_value(42, 0), 42.0);
        assert_eq!(kd_ratio_value(30, 10), 3.0);
    }

    #[test]
    fn ppk_values_and_format() {
        assert_eq!(ppk(100, 0), 0.0);
        assert_eq!(ppk(100, 20), 5.0);
        assert_eq!(format_ppk(100, 20), "5");
        assert_eq!(format_ppk(25_000, 10), "2.5k");
        assert_eq!(format_ppk(0, 0), "0");
    }

    #[test]
    fn month_names() {
        assert_eq!(month_name(1), "Jan");
        assert_eq!(month_name(12), "Dec");
        assert_eq!(month_name(0), "???");
        assert_eq!(month_name(13), "???");
    }

    #[test]
    fn image_urls() {
        assert_eq!(
            character_portrait_url(42, 32),
            "https://images.evetech.net/characters/42/portrait?size=32"
        );
        assert_eq!(
            ship_icon_url(587, 64),
            "https://images.evetech.net/types/587/icon?size=64"
        );
        assert_eq!(
            ship_render_url(587, 128),
            "https://images.evetech.net/types/587/render?size=128"
        );
        assert_eq!(
            corporation_logo_url(7, 32),
            "https://images.evetech.net/corporations/7/logo?size=32"
        );
        assert_eq!(
            alliance_logo_url(9, 64),
            "https://images.evetech.net/alliances/9/logo?size=64"
        );
    }

    fn now() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-04-03T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    fn relative_time_short_buckets() {
        let at = |iso| relative_time_short(iso, now()).unwrap();
        assert_eq!(at("2026-04-03T11:59:01Z"), "now");
        assert_eq!(at("2026-04-03T12:05:00Z"), "now");
        assert_eq!(at("2026-04-03T11:59:00Z"), "1m");
        assert_eq!(at("2026-04-03T11:00:01Z"), "59m");
        assert_eq!(at("2026-04-03T09:00:00+00:00"), "3h");
        assert_eq!(at("2026-04-02T12:00:01Z"), "23h");
        assert_eq!(at("2026-04-01T10:00:00+00:00"), "2d");
    }

    #[test]
    fn relative_time_long_buckets() {
        let at = |iso| relative_time(iso, now()).unwrap();
        assert_eq!(at("2026-04-03T11:59:30Z"), "just now");
        assert_eq!(at("2026-04-03T11:55:00Z"), "5m ago");
        assert_eq!(at("2026-04-03T14:00:00+05:00"), "3h ago");
        assert_eq!(at("2026-03-27T12:00:00Z"), "7d ago");
    }

    #[test]
    fn relative_time_rejects_garbage() {
        assert!(relative_time_short("yesterday", now()).is_none());
        assert!(relative_time("", now()).is_none());
    }
}
