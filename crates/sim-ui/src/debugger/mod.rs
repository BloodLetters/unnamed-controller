pub mod controls_view;
pub mod disassembly_view;
pub mod gdb_panel;
pub mod hex_view;
pub mod registers_view;

use egui::{RichText, Ui};
use sim_core::debugger::{GdbServer, WatchpointKind};

use crate::app::SimulatorApp;
use gdb_panel::render_gdb_panel;

/// UI state tracker for the interactive MCU debugger tool.
pub struct DebuggerUiState {
    pub open: bool,
    pub target_esp32_idx: usize,
    pub new_wp_addr: String,
    pub new_wp_size: usize,
    pub new_wp_kind: WatchpointKind,
    pub hex_page_offset: usize,
    pub gdb_server: GdbServer,
}

impl Default for DebuggerUiState {
    fn default() -> Self {
        Self {
            open: false,
            target_esp32_idx: 0,
            new_wp_addr: String::new(),
            new_wp_size: 4,
            new_wp_kind: WatchpointKind::Write,
            hex_page_offset: 0,
            gdb_server: GdbServer::new(3333),
        }
    }
}

/// Renders the floating debugger window containing disassembly, registers, memory hex editor, and GDB RSP diagnostics.
pub fn render_debugger_window(ctx: &egui::Context, app: &mut SimulatorApp) {
    if !app.debugger_state.open {
        return;
    }

    let mut is_open = app.debugger_state.open;
    egui::Window::new("🐞 MCU Debugger & GDB Remote Inspector")
        .open(&mut is_open)
        .default_size([880.0, 560.0])
        .resizable(true)
        .show(ctx, |ui| {
            render_debugger_contents(ui, app);
        });
    app.debugger_state.open = is_open;
}

/// Internal layout renderer for the debugger window panels.
fn render_debugger_contents(ui: &mut Ui, app: &mut SimulatorApp) {
    ui.label(RichText::new("No MCU board present on the canvas.").italics());
    ui.separator();
    render_gdb_panel(ui, &mut app.debugger_state.gdb_server);
}
