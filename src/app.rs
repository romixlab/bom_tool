use crate::context::Context;
use crate::prelude::*;
use crate::tabs::{Tab, TabKindDiscriminants, TreeBehavior};
use crate::windows::{UniqueWindows, WindowKind, WindowToggleButtonsLocations};
use egui::{CentralPanel, ScrollArea, SidePanel, TopBottomPanel, Ui};
use egui_modal::Modal;
use egui_tracing::EventCollector;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

pub struct BomToolApp {
    cx: Context,
    state: State,
    shutdown_modal: Modal,
    shutdown_confirmed: bool,
}

#[derive(Serialize, Deserialize)]
struct State {
    tabs: egui_tiles::Tree<Tab>,
    #[serde(skip)]
    tabs_behavior: TreeBehavior,
    side_panel_expanded: bool,
    windows: UniqueWindows,
}

impl Default for State {
    fn default() -> Self {
        let mut next_view_nr = 0;
        let mut gen_view = |kind: TabKindDiscriminants| {
            let view = Tab::from_kind(kind, next_view_nr);
            next_view_nr += 1;
            view
        };

        let mut tiles = egui_tiles::Tiles::default();
        let mut tabs = vec![];

        tabs.push(tiles.insert_pane(gen_view(TabKindDiscriminants::TabBomImporter)));
        tabs.push(tiles.insert_pane(gen_view(TabKindDiscriminants::TabBomImporter)));
        tabs.push(tiles.insert_pane(gen_view(TabKindDiscriminants::TabB)));

        let root = tiles.insert_tab_tile(tabs);

        let tabs = egui_tiles::Tree::new("my_tree", root, tiles);

        Self {
            tabs,
            tabs_behavior: Default::default(),
            side_panel_expanded: true,
            windows: Default::default(),
        }
    }
}

impl BomToolApp {
    /// Called once before the first frame.
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        cx: Context,
        #[cfg(not(target_arch = "wasm32"))] event_collector: EventCollector,
    ) -> Self {
        // Load previous app state (if any).
        let mut state = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or(State::default())
        } else {
            State::default()
        };

        // When adding a new window during development, restored state won't have new windows, take care of it:
        if state.windows.windows.len() != WindowKind::iter().size_hint().0 {
            state.windows = Default::default();
        }

        // Restore contexts for windows
        for (window, _) in &mut state.windows.windows {
            #[cfg(not(target_arch = "wasm32"))]
            if let WindowKind::LogViewer(log_viewer) = window {
                log_viewer.set_collector(event_collector);
                break;
            }
        }

        // Restore context for tabs
        state.tabs_behavior.feed_cx(cx.clone());

        BomToolApp {
            cx,
            state,
            // log_viewer,
            shutdown_modal: Modal::new(&cc.egui_ctx, "shutdown_modal"),
            shutdown_confirmed: false,
        }
    }

    fn menu_bar(&mut self, ui: &mut Ui) {
        ui.toggle_value(&mut self.state.side_panel_expanded, "*")
            .on_hover_text("Show/Hide side panel");
        ui.separator();
        let is_web = cfg!(target_arch = "wasm32");
        ui.menu_button("File", |ui| {
            let is_clicked = self
                .state
                .windows
                .toggle_buttons(WindowToggleButtonsLocations::File, ui);
            if is_clicked {
                ui.close_menu();
            }
            // NOTE: no File->Quit on web pages!
            if !is_web {
                if ui.button("Quit").clicked() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        });
        ui.menu_button("Window", |ui| {
            let is_clicked = self
                .state
                .windows
                .toggle_buttons(WindowToggleButtonsLocations::Window, ui);
            if is_clicked {
                ui.close_menu();
            }
        });
        ui.menu_button("Help", |ui| {
            let mut is_clicked = self
                .state
                .windows
                .toggle_buttons(WindowToggleButtonsLocations::Help, ui);
            if ui.button("Reset mem").clicked() {
                ui.ctx().memory_mut(|memory| {
                    *memory = Default::default();
                });
                self.state = State::default();
                self.state.tabs_behavior.feed_cx(self.cx.clone());
                is_clicked = true;
            }
            if is_clicked {
                ui.close_menu();
            }
        });
        ui.add_space(16.0);

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            egui::warn_if_debug_build(ui);
            egui::widgets::global_theme_preference_buttons(ui);
        });
    }

    fn side_panel(&mut self, ui: &mut Ui) {
        self.state.tabs_behavior.ui(ui);

        ui.collapsing("Tree", |ui| {
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
            let tree_debug = format!("{:#?}", self.state.tabs);
            ui.monospace(&tree_debug);
        });

        ui.separator();

        ui.collapsing("Active tiles", |ui| {
            let active = self.state.tabs.active_tiles();
            for tile_id in active {
                use egui_tiles::Behavior as _;
                let name = self
                    .state
                    .tabs_behavior
                    .tab_title_for_tile(&self.state.tabs.tiles, tile_id);
                ui.label(format!("{} - {tile_id:?}", name.text()));
            }
        });

        ui.separator();

        if let Some(root) = self.state.tabs.root() {
            crate::sidepanel::tree_ui(
                ui,
                &mut self.state.tabs_behavior,
                &mut self.state.tabs.tiles,
                root,
            );
        }
    }
}

impl eframe::App for BomToolApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                self.menu_bar(ui);
            });
        });

        SidePanel::left("side_panel").resizable(true).show_animated(
            ctx,
            self.state.side_panel_expanded,
            |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    self.side_panel(ui);
                });
            },
        );

        if let Some(parent) = self.state.tabs_behavior.add_child_to.take() {
            debug!("Add child to {:?}", parent);
        }

        self.state.windows.show_open_windows(&mut self.cx, ctx);

        CentralPanel::default().show(ctx, |ui| {
            self.state.tabs.ui(&mut self.state.tabs_behavior, ui);
        });

        let modal = &self.shutdown_modal;
        if ctx.input(|i| i.viewport().close_requested()) {
            if !self.shutdown_confirmed {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                modal.open();
            }
        }
        modal.show(|ui| {
            modal.title(ui, "Confirm exit");
            modal.frame(ui, |ui| {
                ui.label("Are you sure you want to exit?");
            });
            modal.buttons(ui, |ui| {
                modal.button(ui, "Cancel");
                if modal.suggested_button(ui, "Save & Exit").clicked() {
                    // TODO: Save things, set shutdown_confirmed to true and send a Close command
                };
                if modal.caution_button(ui, "Discard & Exit").clicked() {
                    self.shutdown_confirmed = true;
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                };
            });
        });
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, &self.state);
    }

    // fn on_exit(&mut self) {
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // if let Some(tx) = self.shutdown_event_tx.take() {
        //     _ = tx.send(());
        // }
    }

    fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] {
        visuals.panel_fill.to_normalized_gamma_f32()
    }
}

impl State {}
