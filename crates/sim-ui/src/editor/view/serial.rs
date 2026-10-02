use egui::{Color32, RichText, Ui};
use sim_components::board::Board;

use crate::app::SimulatorApp;
use crate::editor::types::{EditorTarget, MonitorTab};

/// Renders the bottom console panel with selectable Serial and Build tabs.
pub fn render_serial_monitor(ui: &mut Ui, app: &mut SimulatorApp) {
    render_tab_headers(ui, app);
    ui.separator();

    match app.code_editor.monitor_tab {
        MonitorTab::Serial => render_serial_tab(ui, app),
        MonitorTab::Build => render_build_tab(ui, app),
    }
}

/// Renders tab switch buttons for Serial and Build logs.
fn render_tab_headers(ui: &mut Ui, app: &mut SimulatorApp) {
    ui.horizontal(|ui| {
        let is_serial = app.code_editor.monitor_tab == MonitorTab::Serial;
        let is_build = app.code_editor.monitor_tab == MonitorTab::Build;

        let serial_color = if is_serial {
            Color32::from_rgb(100, 220, 255)
        } else {
            Color32::from_gray(170)
        };
        if ui
            .selectable_label(is_serial, RichText::new("📟 Serial").color(serial_color))
            .clicked()
        {
            app.code_editor.monitor_tab = MonitorTab::Serial;
        }

        let build_title = if app.code_editor.build_in_progress {
            "🔨 Build (Compiling...)"
        } else {
            "🔨 Build"
        };
        let build_color = if is_build {
            Color32::from_rgb(255, 200, 80)
        } else {
            Color32::from_gray(170)
        };
        if ui
            .selectable_label(is_build, RichText::new(build_title).color(build_color))
            .clicked()
        {
            app.code_editor.monitor_tab = MonitorTab::Build;
        }

        if app.code_editor.build_in_progress {
            ui.spinner();
            ui.label(
                RichText::new("Background compilation active")
                    .italics()
                    .size(11.0),
            );
        }
    });
}

/// Renders UART serial console with active board text buffer and input controls.
fn render_serial_tab(ui: &mut Ui, app: &mut SimulatorApp) {
    let mut uart_text = match app.code_editor.target {
        EditorTarget::Esp32(idx) => app
            .esp32_s3_boards
            .get(idx)
            .map(|(_, b)| b.serial_output().to_string())
            .unwrap_or_default(),
        _ => String::new(),
    };

    egui::ScrollArea::vertical()
        .id_source("serial_console_scroll")
        .max_height(85.0)
        .stick_to_bottom(true)
        .show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut uart_text)
                    .font(egui::TextStyle::Monospace)
                    .desired_width(f32::INFINITY)
                    .interactive(false),
            );
        });

    ui.horizontal(|ui| {
        let input_resp = ui.text_edit_singleline(&mut app.code_editor.serial_input);
        let send_clicked = ui.button("Send").clicked();
        let enter_pressed =
            input_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

        if (send_clicked || enter_pressed) && !app.code_editor.serial_input.is_empty() {
            if let EditorTarget::Esp32(idx) = app.code_editor.target
                && let Some((_, board)) = app.esp32_s3_boards.get_mut(idx)
            {
                board.send_serial_input(&app.code_editor.serial_input);
            }
            app.code_editor.serial_input.clear();
        }

        if ui.button("Clear Console").clicked()
            && let EditorTarget::Esp32(idx) = app.code_editor.target
            && let Some((_, board)) = app.esp32_s3_boards.get_mut(idx)
        {
            board.clear_serial_output();
        }
    });
}

/// Renders compiler diagnostics and build progress log.
fn render_build_tab(ui: &mut Ui, app: &mut SimulatorApp) {
    let mut log_text = app.code_editor.build_log.clone();

    egui::ScrollArea::vertical()
        .id_source("build_console_scroll")
        .max_height(85.0)
        .stick_to_bottom(true)
        .show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut log_text)
                    .font(egui::TextStyle::Monospace)
                    .desired_width(f32::INFINITY)
                    .interactive(false),
            );
        });

    ui.horizontal(|ui| {
        if ui.button("Clear Build Log").clicked() {
            app.code_editor.build_log.clear();
        }
        if app.code_editor.build_in_progress {
            ui.label(RichText::new("Compiling firmware...").color(Color32::from_rgb(255, 200, 80)));
        } else {
            ui.label(
                RichText::new("Build idle")
                    .color(Color32::from_gray(140))
                    .size(11.0),
            );
        }
    });
}
