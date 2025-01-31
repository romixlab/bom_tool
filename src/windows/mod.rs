use crate::context::Context;
use egui::{Ui, Window};
use serde::{Deserialize, Serialize};
use strum::{EnumIter, EnumMessage, IntoEnumIterator};

mod about;
mod debug;
mod log_viewer;
mod settings;

#[derive(EnumIter, EnumMessage, Serialize, Deserialize)]
pub enum WindowKind {
    #[strum(message = "About")]
    About(about::AboutWindow),
    #[strum(message = "Settings", detailed_message = "Open configuration window")]
    Settings(settings::SettingsWindow),
    #[strum(message = "Log viewer")]
    LogViewer(log_viewer::LogViewer),
    #[strum(message = "Debug")]
    Debug(debug::DebugWindow),
}

impl WindowKind {
    pub fn ui(&mut self, cx: &mut Context, ui: &mut Ui) {
        match self {
            WindowKind::About(about) => about.ui(ui, cx),
            WindowKind::Settings(settings) => settings.ui(ui),
            WindowKind::LogViewer(log_viewer) => log_viewer.ui(ui, cx),
            WindowKind::Debug(debug) => debug.ui(ui),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct UniqueWindows {
    pub windows: Vec<(WindowKind, bool)>,
}

impl Default for UniqueWindows {
    fn default() -> Self {
        Self {
            windows: WindowKind::iter().map(|w| (w, false)).collect(),
        }
    }
}

pub enum WindowToggleButtonsLocations {
    File,
    Window,
    Help,
}

impl UniqueWindows {
    pub fn toggle_buttons(&mut self, location: WindowToggleButtonsLocations, ui: &mut Ui) -> bool {
        let mut clicked = false;
        let filter = |item: &&mut (WindowKind, bool)| match location {
            WindowToggleButtonsLocations::File => {
                matches!(item.0, WindowKind::Settings(_))
            }
            WindowToggleButtonsLocations::Window => {
                matches!(item.0, WindowKind::LogViewer(_)) || matches!(item.0, WindowKind::Debug(_))
            }
            WindowToggleButtonsLocations::Help => {
                matches!(item.0, WindowKind::About(_))
            }
        };
        for (window, is_visible) in self.windows.iter_mut().filter(filter) {
            let r = ui.toggle_value(is_visible, window.get_message().unwrap_or("W"));
            let r = if let Some(detailed_message) = window.get_detailed_message() {
                r.on_hover_text(detailed_message)
            } else {
                r
            };
            if r.clicked() {
                clicked = true;
            }
        }
        clicked
    }

    pub fn show_open_windows(&mut self, cx: &mut Context, ctx: &egui::Context) {
        for (window, is_visible) in &mut self.windows {
            if !*is_visible {
                continue;
            }
            Window::new(window.get_message().unwrap_or("W"))
                .open(is_visible)
                .collapsible(true)
                .scroll([true, true])
                .show(ctx, |ui| {
                    window.ui(cx, ui);
                });
        }
    }
}
