use egui::{Color32, RichText, Ui};
use sim_core::debugger::GdbServer;

/// Renders GDB Remote Serial Protocol (RSP) TCP server controls and status diagnostics.
pub fn render_gdb_panel(ui: &mut Ui, server: &mut GdbServer) {
    ui.label(RichText::new("GDB Remote Serial Protocol (RSP) Server").strong());
    ui.separator();

    ui.horizontal(|ui| {
        ui.label("TCP Port:");
        let mut port_str = server.port.to_string();
        if ui
            .add_enabled(
                !server.is_running,
                egui::TextEdit::singleline(&mut port_str).desired_width(50.0),
            )
            .lost_focus()
            && let Ok(p) = port_str.parse::<u16>()
        {
            server.port = p;
        }

        if server.is_running {
            if ui.button("⏹ Stop Server").clicked() {
                server.stop();
            }
            ui.label(
                RichText::new(format!("● LISTENING (127.0.0.1:{})", server.port))
                    .color(Color32::from_rgb(76, 175, 80))
                    .strong(),
            );
        } else {
            if ui.button("▶ Start GDB Server").clicked() {
                let _ = server.start();
            }
            ui.label(
                RichText::new("○ INACTIVE")
                    .color(Color32::from_gray(120))
                    .strong(),
            );
        }
    });

    ui.separator();
    ui.horizontal(|ui| {
        ui.label(RichText::new("Debugger Client Command:").small());
        ui.monospace(
            RichText::new(format!("target remote 127.0.0.1:{}", server.port))
                .color(Color32::from_rgb(100, 181, 246)),
        );
    });
}
