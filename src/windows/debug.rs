use egui::Ui;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
pub struct DebugWindow {}

impl DebugWindow {
    pub fn ui(&mut self, ui: &mut Ui) {
        ui.label("Debug");
    }
}
