use egui::Ui;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
pub struct SettingsWindow {}

impl SettingsWindow {
    pub fn ui(&mut self, ui: &mut Ui) {
        ui.label("Settings:");
    }
}
