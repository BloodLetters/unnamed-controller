use egui::{Color32, RichText, Ui};

use crate::app::SimulatorApp;
use crate::editor::actions::{
    build_and_flash_esp32, load_firmware_file, reset_target, step_target,
};
use crate::editor::types::EditorTarget;

/// Renders the action toolbar for compiling, flashing, and execution control.
pub fn render_toolbar(ui: &mut Ui, app: &mut SimulatorApp) {
    ui.horizontal(|ui| {
        render_target_selector(ui, app);
        ui.separator();
        render_action_buttons(ui, app);
    });
}

/// Renders the target MCU selector dropdown with dynamic board listing.
fn render_target_selector(ui: &mut Ui, app: &mut SimulatorApp) {
    ui.label(RichText::new("Target:").strong());
    let current_label = match app.code_editor.target {
        EditorTarget::None => "Select".to_string(),
        EditorTarget::Esp32(idx) => format!("ESP32-S3 #{}", idx),
        EditorTarget::Mcu(idx) => format!("DummyMCU #{}", idx),
        EditorTarget::Avr(idx) => format!("Arduino Uno #{}", idx),
    };

    egui::ComboBox::from_id_source("target_mcu_select")
        .selected_text(current_label)
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut app.code_editor.target, EditorTarget::None, "Select");
            if app.esp32_s3_boards.is_empty() {
                ui.label("No boards on canvas");
            } else {
                for i in 0..app.esp32_s3_boards.len() {
                    ui.selectable_value(
                        &mut app.code_editor.target,
                        EditorTarget::Esp32(i),
                        format!("ESP32-S3 #{}", i),
                    );
                }
            }
        });
}

/// Renders action buttons for flashing, building, stepping, and reset.
fn render_action_buttons(ui: &mut Ui, app: &mut SimulatorApp) {
    let build_btn = egui::Button::new(
        RichText::new("⬆ Build & Flash")
            .strong()
            .color(Color32::from_rgb(255, 170, 90)),
    );

    if ui.add(build_btn).clicked() {
        match app.code_editor.target {
            EditorTarget::Esp32(idx) => build_and_flash_esp32(app, idx),
            _ => {
                if !app.esp32_s3_boards.is_empty() {
                    app.code_editor.target = EditorTarget::Esp32(0);
                    build_and_flash_esp32(app, 0);
                } else {
                    app.code_editor.status_message =
                        "Tambahkan board ESP32-S3 ke canvas terlebih dahulu.".to_string();
                    app.code_editor.status_is_error = true;
                }
            }
        }
    }

    if ui.button("⟳ Reboot").clicked() {
        reset_target(app);
    }

    if ui.button("⏭ Step").clicked() {
        step_target(app);
    }

    if ui.button("📂 Load .bin/.hex").clicked() {
        load_firmware_file(app);
    }

    ui.separator();

    ui.checkbox(
        &mut app.code_editor.show_serial_monitor,
        "Show Serial Monitor",
    );
}
