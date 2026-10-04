use std::borrow::Cow;

use gpui_kit::assets::{Assets, icon_assets};
use gpui_kit::{AssetSource, Result, SharedString};

icon_assets!(
    pub ExtraIcons,
    [
        Anchor, Bomb, Boxes, Circle, CircleDot, Crosshair, Download, Flag, HeartPulse, Keyboard,
        Layers, Lock, LockOpen, Pickaxe, ShieldHalf, Swords, Truck,
        LogOut, Pencil, Radar, RotateCcw, Rocket, Satellite, Share2, Shield, Ship, Square,
        StickyNote, Trash, Users, X, Zap
    ]
);

const LOGO: &[u8] = include_bytes!("../icons/icon.svg");

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
        if path == "logo.svg" {
            return Ok(Some(Cow::Borrowed(LOGO)));
        }
        if let Some(name) = path
            .strip_prefix("brackets/")
            .and_then(|p| p.strip_suffix(".png"))
            && let Some((_, bytes)) = BRACKETS.iter().find(|(n, _)| *n == name)
        {
            return Ok(Some(Cow::Borrowed(bytes)));
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
