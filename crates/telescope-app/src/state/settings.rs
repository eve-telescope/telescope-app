use std::path::PathBuf;

use gpui_kit::{App, AppContext as _, Context, Entity};
use telescope_core::settings::Settings;

use crate::services::Services;

pub struct SettingsStore {
    settings: Settings,
    data_dir: PathBuf,
}

impl SettingsStore {
    pub fn get(&self) -> &Settings {
        &self.settings
    }

    pub fn update(&mut self, cx: &mut Context<Self>, edit: impl FnOnce(&mut Settings)) {
        let before = self.settings.clone();
        edit(&mut self.settings);
        if self.settings != before {
            self.settings.save(&self.data_dir);
            cx.notify();
        }
    }
}

pub fn init(cx: &mut App) -> Entity<SettingsStore> {
    let data_dir = Services::get(cx).paths.data.clone();
    cx.new(|_| SettingsStore {
        settings: Settings::load(&data_dir),
        data_dir,
    })
}
