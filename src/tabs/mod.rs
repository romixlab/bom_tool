use crate::context::Context;
use crate::tab_viewer::TabUi;
use egui::{Rect, Vec2, WidgetText};
use egui_tiles::{Tile, TileId, Tiles, UiResponse};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};
use strum::{AsRefStr, EnumDiscriminants, EnumIter};

mod tab_a;
mod tab_b;

#[derive(Serialize, Deserialize, EnumDiscriminants)]
#[strum_discriminants(derive(Serialize, Deserialize, EnumIter, AsRefStr))]
pub enum TabKind {
    TabA(tab_a::TabA),
    TabB(tab_b::TabB),
}

// impl TabKindDiscriminants {
//     pub fn create_tab(&self, nr: usize) -> Tab {
//         let kind = match self {
//             TabKindDiscriminants::TabA => TabKind::TabA(tab_a::TabA::default()),
//             TabKindDiscriminants::TabB => TabKind::TabB(tab_b::TabB::default()),
//         };
//         Tab { kind, nr }
//     }
// }

impl Debug for TabKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "TabKind({})", TabKindDiscriminants::from(self).as_ref())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tab {
    pub kind: TabKind,
    pub nr: usize,
}

impl Tab {
    pub fn ui(&mut self, ui: &mut egui::Ui, cx: &mut Context) -> UiResponse {
        let color = egui::epaint::Hsva::new(0.103 * self.nr as f32, 0.5, 0.5, 1.0);
        let max_rect = ui.max_rect();
        let header_rect = Rect::from_min_size(max_rect.min, Vec2::new(max_rect.width(), 32.0));
        ui.painter().rect_filled(header_rect, 0.0, color);
        let dragged = ui
            .allocate_rect(header_rect, egui::Sense::click_and_drag())
            .on_hover_cursor(egui::CursorIcon::Grab)
            .dragged();
        match &mut self.kind {
            TabKind::TabA(t) => t.ui(ui, cx),
            TabKind::TabB(t) => t.ui(ui, cx),
        }
        if dragged {
            UiResponse::DragStarted
        } else {
            UiResponse::None
        }
    }

    pub fn title(&self) -> WidgetText {
        match &self.kind {
            TabKind::TabA(t) => t.title(),
            TabKind::TabB(t) => t.title(),
        }
    }

    // pub fn add_popup(to: &mut Vec<Tab>, surface: SurfaceIndex, node: NodeIndex, ui: &mut egui::Ui) {
    //     for tab_kind in TabKindDiscriminants::iter() {
    //         // Skip log viewer as it needs context to function properly, open through Window -> Log Viewer menu
    //         if matches!(tab_kind, TabKindDiscriminants::LogViewer) {
    //             continue;
    //         }
    //         if ui.button(tab_kind.as_ref()).clicked() {
    //             to.push(tab_kind.create_tab(surface, node))
    //         }
    //     }
    // }
    //
    // pub fn default_tabs() -> Vec<Tab> {
    //     vec![Tab::tab_a(SurfaceIndex::main(), NodeIndex(1))]
    // }

    pub fn tab_b(nr: usize) -> Self {
        Tab {
            kind: TabKind::TabA(tab_a::TabA::default()),
            nr,
        }
    }
}

pub struct TreeBehavior {
    simplification_options: egui_tiles::SimplificationOptions,
    tab_bar_height: f32,
    gap_width: f32,
    pub(crate) add_child_to: Option<TileId>,
    cx: Option<Context>,
}

impl Default for TreeBehavior {
    fn default() -> Self {
        Self {
            simplification_options: Default::default(),
            tab_bar_height: 24.0,
            gap_width: 2.0,
            add_child_to: None,
            cx: None,
        }
    }
}

impl TreeBehavior {
    pub(crate) fn ui(&mut self, ui: &mut egui::Ui) {
        let Self {
            simplification_options,
            tab_bar_height,
            gap_width,
            add_child_to: _,
            cx: _,
        } = self;

        egui::Grid::new("behavior_ui")
            .num_columns(2)
            .show(ui, |ui| {
                ui.label("All panes must have tabs:");
                ui.checkbox(&mut simplification_options.all_panes_must_have_tabs, "");
                ui.end_row();

                ui.label("Join nested containers:");
                ui.checkbox(
                    &mut simplification_options.join_nested_linear_containers,
                    "",
                );
                ui.end_row();

                ui.label("Tab bar height:");
                ui.add(
                    egui::DragValue::new(tab_bar_height)
                        .range(0.0..=100.0)
                        .speed(1.0),
                );
                ui.end_row();

                ui.label("Gap width:");
                ui.add(egui::DragValue::new(gap_width).range(0.0..=20.0).speed(1.0));
                ui.end_row();
            });
    }

    pub fn feed_cx(&mut self, cx: Context) {
        self.cx = Some(cx);
    }
}

impl egui_tiles::Behavior<Tab> for TreeBehavior {
    fn pane_ui(&mut self, ui: &mut egui::Ui, _tile_id: TileId, view: &mut Tab) -> UiResponse {
        if let Some(cx) = &mut self.cx {
            view.ui(ui, cx)
        } else {
            UiResponse::None
        }
    }

    fn tab_title_for_pane(&mut self, view: &Tab) -> WidgetText {
        // format!("View {}", view.nr).into()
        view.title()
    }

    fn is_tab_closable(&self, _tiles: &Tiles<Tab>, _tile_id: TileId) -> bool {
        true
    }

    // ---
    // Settings:

    fn on_tab_close(&mut self, tiles: &mut Tiles<Tab>, tile_id: TileId) -> bool {
        if let Some(tile) = tiles.get(tile_id) {
            match tile {
                Tile::Pane(pane) => {
                    // Single pane removal
                    let tab_title = self.tab_title_for_pane(pane);
                    tracing::debug!("Closing tab: {}, tile ID: {tile_id:?}", tab_title.text());
                }
                Tile::Container(container) => {
                    // Container removal
                    tracing::debug!("Closing container: {:?}", container.kind());
                    let children_ids = container.children();
                    for child_id in children_ids {
                        if let Some(Tile::Pane(pane)) = tiles.get(*child_id) {
                            let tab_title = self.tab_title_for_pane(pane);
                            tracing::debug!(
                                "Closing tab: {}, tile ID: {tile_id:?}",
                                tab_title.text()
                            );
                        }
                    }
                }
            }
        }

        // Proceed to removing the tab
        true
    }

    fn top_bar_right_ui(
        &mut self,
        _tiles: &Tiles<Tab>,
        ui: &mut egui::Ui,
        tile_id: TileId,
        _tabs: &egui_tiles::Tabs,
        _scroll_offset: &mut f32,
    ) {
        if ui.button("➕").clicked() {
            self.add_child_to = Some(tile_id);
        }
    }

    fn tab_bar_height(&self, _style: &egui::Style) -> f32 {
        self.tab_bar_height
    }

    fn gap_width(&self, _style: &egui::Style) -> f32 {
        self.gap_width
    }

    fn simplification_options(&self) -> egui_tiles::SimplificationOptions {
        self.simplification_options
    }
}
