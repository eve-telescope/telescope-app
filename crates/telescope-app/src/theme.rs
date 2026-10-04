//! The Telescope palette (ported from the Vue app's global.css) and the
//! gpui-kit component theme built from it.

use std::rc::Rc;

use gpui_kit::component::{Theme, ThemeMode, ThemeSet};
use gpui_kit::{App, Hsla, Rgba, rgb, rgba};

pub const BG_0: u32 = 0x0a0a0b;
pub const BG_1: u32 = 0x101012;
pub const BG_2: u32 = 0x16161a;
pub const BG_3: u32 = 0x232329;
pub const BG_HOVER: u32 = 0x26262d;
pub const BORDER: u32 = 0x2c2c34;
pub const TEXT_1: u32 = 0xf2f2f4;
pub const TEXT_2: u32 = 0xb4b4bd;
pub const TEXT_3: u32 = 0x8b8b96;
pub const CYAN: u32 = 0x00d4ff;
pub const CYAN_DIM: u32 = 0x0099bb;
pub const ORANGE: u32 = 0xff9944;
pub const GREEN: u32 = 0x44ddaa;
pub const RED: u32 = 0xff5566;
pub const WIN_CLOSE: u32 = 0xc42b1c;
pub const LOGO_TILE: u32 = 0x12151a;

pub fn color(hex: u32) -> Hsla {
    rgb(hex).into()
}

/// `hex` with an alpha byte, like the `#rrggbbaa` tints the Vue app used.
pub fn tint(hex: u32, alpha: u8) -> Hsla {
    rgba((hex << 8) | alpha as u32).into()
}

/// Parses `#rgb`, `#rrggbb` or `#rrggbbaa`, as stored on annotations.
pub fn parse_hex(value: &str) -> Option<Rgba> {
    let hex = value.trim().trim_start_matches('#');
    let expanded: String = match hex.len() {
        3 => hex.chars().flat_map(|c| [c, c]).collect(),
        6 | 8 => hex.to_string(),
        _ => return None,
    };
    let n = u32::from_str_radix(&expanded, 16).ok()?;
    Some(if expanded.len() == 6 { rgb(n) } else { rgba(n) })
}

pub fn hex_or(value: &str, fallback: u32) -> Hsla {
    parse_hex(value)
        .map(Into::into)
        .unwrap_or_else(|| color(fallback))
}

pub fn hex_tint(value: &str, alpha: u8, fallback: u32) -> Hsla {
    let mut c = hex_or(value, fallback);
    c.a = alpha as f32 / 255.0;
    c
}

pub fn threat_color(level: &str) -> Hsla {
    color(match level.to_ascii_lowercase().as_str() {
        "extreme" => 0xff2255,
        "high" => 0xff6633,
        "moderate" => 0xffaa22,
        "low" => 0x44ddbb,
        "minimal" => 0x7f8fa0,
        _ => 0x75849a,
    })
}

pub fn init(cx: &mut App) {
    let set: ThemeSet = serde_json::from_str(include_str!("../assets/theme.json"))
        .expect("bundled theme.json is valid");
    let config = Rc::new(
        set.themes
            .into_iter()
            .next()
            .expect("theme.json has a theme"),
    );
    Theme::change(ThemeMode::Dark, None, cx);
    Theme::update(cx, |theme| {
        theme.dark_theme = config.clone();
        theme.apply_config(&config);
    });
}
