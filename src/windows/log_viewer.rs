use crate::prelude::*;
use egui_tracing::EventCollector;

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct LogViewer {
    #[serde(skip)]
    event_collector: EventCollector,
}

impl LogViewer {
    pub fn ui(&mut self, ui: &mut Ui, _cx: &mut Context) {
        ui.add(egui_tracing::Logs::new(self.event_collector.clone()));
    }
}

impl LogViewer {
    pub fn set_collector(&mut self, event_collector: EventCollector) {
        self.event_collector = event_collector;
    }
}
