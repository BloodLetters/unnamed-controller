use egui::{Pos2, Vec2};
use sim_core::engine::Engine;
use sim_core::netlist::PinId;

use crate::history::HistoryStack;
use crate::types::{ProjectData, SelectedItem, SpawningComponent, ViewMode, Wire};
use crate::view::render_central_canvas;

/// Interactive circuit simulator workspace application.
pub struct SimulatorApp {
    pub wires: Vec<Wire>,
    pub esp32_s3_boards: Vec<(Pos2, sim_components::board::Esp32S3DevKit)>,
    pub esp32_s3_rotations: Vec<u16>,
    pub components: Vec<crate::component::PlacedComponent>,

    pub drawing_wire: Option<PinId>,
    pub wire_waypoints: Vec<Pos2>,
    pub canvas_rect: egui::Rect,
    pub dragging_item: bool,
    pub engine: Engine,
    pub power_on: bool,
    pub spawning: SpawningComponent,
    pub selected: SelectedItem,
    pub pan: Vec2,
    pub zoom: f32,
    pub box_select_start: Option<Pos2>,
    pub box_select_current: Option<Pos2>,
    pub history: HistoryStack,
    pub code_editor: crate::editor::CodeEditorState,
    pub instruments: crate::instruments::InstrumentsBenchState,
    pub debugger_state: crate::debugger::DebuggerUiState,
    pub audio_state: crate::audio::AudioUiState,
    pub audio_mixer: crate::audio::AudioMixer,
    pub time_accumulator: f64,
    pub simulated_seconds: f64,
    pub toolchain_warning: Option<String>,
    pub show_toolchain_warning: bool,
    pub project_path: Option<std::path::PathBuf>,
    pub project_message: Option<String>,
    pub view_mode: ViewMode,
    pub show_add_component: bool,
    pub add_component_search: String,
    pub add_component_category: crate::add_component::CategoryKind,
    pub show_workspace: bool,
    #[cfg(not(target_arch = "wasm32"))]
    pub build_receiver: Option<std::sync::mpsc::Receiver<crate::editor::BuildJobResult>>,
    #[cfg(not(target_arch = "wasm32"))]
    pub library_receiver: Option<std::sync::mpsc::Receiver<crate::editor::types::LibraryJobResult>>,
}

impl Default for SimulatorApp {
    fn default() -> Self {
        let engine = Engine::new();

        let mut app = Self {
            wires: vec![],
            esp32_s3_boards: vec![],
            esp32_s3_rotations: vec![],
            components: vec![],
            drawing_wire: None,
            wire_waypoints: vec![],
            canvas_rect: egui::Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 600.0)),

            dragging_item: false,
            engine,
            power_on: false,
            spawning: SpawningComponent::None,
            selected: SelectedItem::None,
            pan: Vec2::ZERO,
            zoom: 1.0_f32,
            box_select_start: None,
            box_select_current: None,
            history: HistoryStack::default(),
            code_editor: crate::editor::CodeEditorState::default(),
            instruments: crate::instruments::InstrumentsBenchState::default(),
            debugger_state: crate::debugger::DebuggerUiState::default(),
            audio_state: crate::audio::AudioUiState::default(),
            audio_mixer: crate::audio::AudioMixer::default(),
            time_accumulator: 0.0,
            simulated_seconds: 0.0,
            toolchain_warning: None,
            show_toolchain_warning: false,
            project_path: None,
            project_message: None,
            view_mode: ViewMode::Both,
            show_add_component: false,
            add_component_search: String::new(),
            add_component_category: crate::add_component::CategoryKind::All,
            show_workspace: true,
            #[cfg(not(target_arch = "wasm32"))]
            build_receiver: None,
            #[cfg(not(target_arch = "wasm32"))]
            library_receiver: None,
        };
        app.rebuild_netlist();
        app
    }
}

impl SimulatorApp {
    /// Constructs a new simulator instance from the eframe context.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self::default();

        let compiler = crate::toolchain::ArduinoCliCompiler::default();
        if !compiler.is_available() {
            app.toolchain_warning = Some(
                "arduino-cli tidak ditemukan di PATH.\n\
                 Untuk mengompilasi sketch (tombol \"Build & Flash\"), pasang dulu:\n\n\
                 \u{2022} winget install ArduinoSA.CLI\n\
                 \u{2022} arduino-cli core install esp32:esp32\n\n\
                 Setelah terpasang, klik \"Periksa lagi\"."
                    .to_string(),
            );
            app.show_toolchain_warning = true;
        }

        app
    }

    /// Converts a local canvas position to screen coordinates.
    pub fn to_screen(&self, canvas_pos: Pos2, origin: Pos2) -> Pos2 {
        origin + self.pan + canvas_pos.to_vec2() * self.zoom
    }

    /// Converts a screen coordinate to local canvas coordinates.
    pub fn to_canvas(&self, screen_pos: Pos2, origin: Pos2) -> Pos2 {
        Pos2::new(
            (screen_pos.x - origin.x - self.pan.x) / self.zoom,
            (screen_pos.y - origin.y - self.pan.y) / self.zoom,
        )
    }

    /// Returns the canvas coordinate at the center of the current canvas camera viewport.
    pub fn camera_center(&self) -> Pos2 {
        self.to_canvas(self.canvas_rect.center(), self.canvas_rect.min)
    }

    /// Queries the canvas coordinate of a specific pin terminal.
    pub fn get_pin_pos(&self, target: PinId) -> Option<Pos2> {
        crate::pins::get_pin_pos(self, target)
    }

    /// Finds the closest pin terminal within snap range.
    pub fn find_closest_pin(&self, local_pos: Pos2) -> Option<PinId> {
        crate::pins::find_closest_pin(self, local_pos)
    }

    /// Creates a project data snapshot.
    pub fn snapshot(&self) -> ProjectData {
        crate::state::create_snapshot(self)
    }

    /// Restores the circuit state from a snapshot.
    pub fn restore_snapshot(&mut self, data: ProjectData) {
        crate::state::restore_snapshot(self, data);
    }

    /// Saves the current workspace state to the undo history stack.
    pub fn record_history(&mut self) {
        let snap = self.snapshot();
        self.history.record_state(snap);
    }

    /// Reverts the circuit workspace to the previous undo state.
    pub fn undo(&mut self) {
        let current = self.snapshot();
        if let Some(prev) = self.history.undo(current) {
            self.restore_snapshot(prev);
        }
    }

    /// Restores a previously undone state from the redo history stack.
    pub fn redo(&mut self) {
        let current = self.snapshot();
        if let Some(next) = self.history.redo(current) {
            self.restore_snapshot(next);
        }
    }

    /// Displaces a selected component or entire group by delta.
    pub fn move_item_or_group(&mut self, item: SelectedItem, delta: Vec2) {
        crate::state::move_item_or_group(self, item, delta);
    }

    /// Spawns a component instance at the local position.
    pub fn spawn_component_at(&mut self, local_pos: Pos2) {
        crate::state::spawn_component_at(self, local_pos);
    }

    /// Deletes the selected component, wire, or group and purges orphan wires.
    pub fn delete_selected(&mut self) {
        self.record_history();
        crate::state::delete_selected_item(self);
    }

    /// Rebuilds electrical node connectivity in the netlist.
    pub fn rebuild_netlist(&mut self) {
        crate::state::rebuild_netlist(self);
    }

    /// Rotates a specific component item by delta degrees (positive = clockwise).
    pub fn rotate_item(&mut self, item: &SelectedItem, delta: i16) {
        if let Some((spec, index)) = crate::registry::find(item) {
            self.record_history();
            (spec.rotate)(self, index, delta);
        }
    }

    /// Rotates the currently selected component(s) by delta degrees.
    pub fn rotate_selected(&mut self, delta: i16) {
        match &self.selected {
            SelectedItem::Group(items) => {
                let items_clone = items.clone();
                for item in &items_clone {
                    self.rotate_item(item, delta);
                }
            }
            item if !item.is_none() => {
                let item_clone = item.clone();
                self.rotate_item(&item_clone, delta);
            }
            _ => {}
        }
    }

    /// Duplicates the selected component, offset slightly, and selects the new instance.
    pub fn duplicate_selected(&mut self) {
        self.record_history();
        crate::state::mutation::duplicate_selected(self);
    }
}

impl eframe::App for SimulatorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        crate::editor::actions::poll_background_build(self, ctx);
        crate::editor::poll_background_library(self, ctx);
        crate::sim::advance_simulation(self, ctx);
        crate::navbar::render_navbar(ctx, self);

        if self.show_workspace {
            egui::SidePanel::left("workspace_sidebar")
                .resizable(true)
                .default_width(190.0)
                .min_width(150.0)
                .max_width(280.0)
                .show(ctx, |ui| {
                    crate::workspace::render_workspace_panel(ui, self);
                });
        }

        match self.view_mode {
            ViewMode::Both => {
                egui::SidePanel::left("code_split_panel")
                    .resizable(true)
                    .default_width(460.0)
                    .min_width(280.0)
                    .show(ctx, |ui| {
                        crate::editor::view::render_editor_content(ui, self);
                    });
                crate::inspector::render_optional_inspector(ctx, self);
                render_central_canvas(ctx, self);
            }
            ViewMode::Code => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    crate::editor::view::render_editor_content(ui, self);
                });
            }
            ViewMode::Circuit => {
                crate::inspector::render_optional_inspector(ctx, self);
                render_central_canvas(ctx, self);
            }
        }

        crate::add_component::render_add_component_dialog(ctx, self);

        crate::instruments::render_instruments_bench(ctx, self);
        crate::debugger::render_debugger_window(ctx, self);
        crate::audio::render_audio_window(ctx, self);

        crate::navbar::render_toolchain_warning(ctx, self);

        if self.power_on {
            ctx.request_repaint();
        }
    }
}
