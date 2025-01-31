use crate::context::Context;
use egui::{Ui, WidgetText};

pub trait TabUi {
    fn title(&self) -> WidgetText;
    fn ui(&mut self, ui: &mut Ui, cx: &mut Context);
    fn is_closeable(&self) -> bool;
}

// pub struct AppTabViewer<'a, 'b> {
//     pub added_nodes: &'a mut Vec<Tab>,
//     pub cx: &'b mut Context,
// }
//
// impl<'a, 'b> TabViewer for AppTabViewer<'a, 'b> {
//     type Tab = Tab;
//
//     fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
//         tab.title()
//     }
//
//     fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
//         tab.ui(ui, self.cx);
//     }
//
//     fn id(&mut self, tab: &mut Self::Tab) -> Id {
//         Id::new("tab").with(tab.node.0)
//     }
//
//     fn add_popup(&mut self, ui: &mut Ui, surface: SurfaceIndex, node: NodeIndex) {
//         ui.set_min_width(120.);
//         ui.style_mut().visuals.button_frame = false;
//         Tab::add_popup(&mut self.added_nodes, surface, node, ui);
//     }
// }
