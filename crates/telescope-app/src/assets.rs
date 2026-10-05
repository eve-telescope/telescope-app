use std::borrow::Cow;

use gpui_kit::assets::{Assets, icon_assets};
use gpui_kit::{AssetSource, Result, SharedString};

icon_assets!(
    pub ExtraIcons,
    [
        Anchor, Bomb, Boxes, Circle, CircleDot, Crosshair, Download, Flag, HeartPulse, Keyboard,
        Layers, Lock, LockOpen, Pickaxe, ShieldHalf, Swords, Truck,
        LogOut, Pencil, Radar, RotateCcw, Rocket, Satellite, Share2, Shield, Ship, Square,
        StickyNote, Trash, Users, UserX, CloudOff, X, Zap
    ]
);

const LOGO: &[u8] = include_bytes!("../icons/logo.png");

macro_rules! brackets {
    ($($name:literal),* $(,)?) => {
        &[$(($name, include_bytes!(concat!("../assets/brackets/", $name, ".png")))),*]
    };
}

/// Overview bracket icons from the game client, fetched by
/// `scripts/fetch-brackets.sh`. Served as `brackets/{name}.png`.
const BRACKETS: &[(&str, &[u8])] = brackets!(
    "battlecruiser",
    "battleship",
    "capsule",
    "cynosuralfield",
    "carrier",
    "cruiser",
    "destroyer",
    "dreadnought",
    "forceauxiliary",
    "freighter",
    "frigate",
    "industrial",
    "industrialcommand",
    "miningbarge",
    "miningfrigate",
    "rookie",
    "shuttle",
    "supercarrier",
    "titan",
);

pub struct AppAssets;

impl AssetSource for AppAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if path == "logo.png" {
            return Ok(Some(Cow::Borrowed(LOGO)));
        }
        if let Some(name) = path
            .strip_prefix("brackets/")
            .and_then(|p| p.strip_suffix(".png"))
        {
            let (stem, tint) = match name.split_once('@') {
                Some((stem, tint)) => (stem, Some(tint)),
                None => (name, None),
            };
            if let Some((_, bytes)) = BRACKETS.iter().find(|(n, _)| *n == stem) {
                return Ok(Some(match tint {
                    Some(tint) => Cow::Owned(tinted(bytes, tint)?),
                    None => Cow::Borrowed(*bytes),
                }));
            }
        }
        if let Some(bytes) = ExtraIcons.load(path)? {
            return Ok(Some(bytes));
        }
        Assets.load(path)
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut paths = Assets.list(path)?;
        paths.extend(ExtraIcons.list(path)?);
        paths.sort();
        paths.dedup();
        Ok(paths)
    }
}

/// Path of a bracket icon recolored to `hex` (`#rrggbb` or `rrggbb`).
pub fn bracket(stem: &str, hex: &str) -> String {
    format!("brackets/{stem}@{}.png", hex.trim_start_matches('#'))
}

/// The client's brackets are white on transparent, so multiplying by the
/// tint recolors them while keeping their shading and alpha.
fn tinted(png: &[u8], hex: &str) -> Result<Vec<u8>> {
    let tint = u32::from_str_radix(hex, 16)?;
    let [_, r, g, b] = tint.to_be_bytes();
    let mut image = image::load_from_memory_with_format(png, image::ImageFormat::Png)?.to_rgba8();
    for pixel in image.pixels_mut() {
        let [pr, pg, pb, _] = &mut pixel.0;
        *pr = (*pr as u16 * r as u16 / 255) as u8;
        *pg = (*pg as u16 * g as u16 / 255) as u8;
        *pb = (*pb as u16 * b as u16 / 255) as u8;
    }
    let mut out = Vec::new();
    image.write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)?;
    Ok(out)
}
