//! Modular Firmware Studio view components and panel renderers.

pub mod libraries;
pub mod serial;
pub mod tabs;
pub mod toolbar;

use egui::{Color32, RichText, Ui};

use crate::app::SimulatorApp;
use crate::editor::types::EditorSection;
pub use libraries::render_library_panel;
pub use serial::render_serial_monitor;
pub use tabs::{render_code_area, render_tabs_bar};
pub use toolbar::render_toolbar;

/// Renders the embedded code editor content suitable for panel or central placement.
pub fn render_editor_content(ui: &mut Ui, app: &mut SimulatorApp) {
    match app.code_editor.section {
        EditorSection::Code => {
            render_toolbar(ui, app);
            ui.separator();
            render_tabs_bar(ui, app);
            ui.separator();

            let available_height = ui.available_height();
            let serial_height = if app.code_editor.show_serial_monitor {
                140.0_f32.min(available_height * 0.4)
            } else {
                0.0
            };
            let code_height = (available_height - serial_height - 28.0).max(100.0);

            ui.allocate_ui(egui::Vec2::new(ui.available_width(), code_height), |ui| {
                render_code_area(ui, app);
            });

            if app.code_editor.show_serial_monitor {
                ui.separator();
                ui.allocate_ui(egui::Vec2::new(ui.available_width(), serial_height), |ui| {
                    render_serial_monitor(ui, app);
                });
            }

            ui.separator();
            render_status_bar(ui, app);
        }
        EditorSection::Libraries => {
            render_library_panel(ui, app);
        }
    }
}

/// Renders the full-screen Firmware Studio window.
pub fn render_code_editor(ctx: &egui::Context, app: &mut SimulatorApp) {
    egui::CentralPanel::default().show(ctx, |ui| {
        render_editor_content(ui, app);
    });
}

/// Renders the bottom diagnostic status line with the cursor position.
fn render_status_bar(ui: &mut Ui, app: &SimulatorApp) {
    ui.horizontal(|ui| {
        let color = if app.code_editor.status_is_error {
            Color32::from_rgb(255, 100, 100)
        } else {
            Color32::from_rgb(120, 220, 120)
        };
        ui.label(RichText::new(&app.code_editor.status_message).color(color));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                RichText::new(format!(
                    "Ln {}, Col {}",
                    app.code_editor.cursor_line, app.code_editor.cursor_col
                ))
                .monospace()
                .color(Color32::from_gray(150)),
            );
        });
    });
}
