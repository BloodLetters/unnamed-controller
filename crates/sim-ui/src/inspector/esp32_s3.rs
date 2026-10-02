use egui::{Color32, RichText, Ui, Vec2};
use sim_components::board::Board;

use crate::app::SimulatorApp;

/// Renders interactive controls, peripherals, and firmware state for an ESP32-S3 board.
pub fn render_esp32_s3_inspector(ui: &mut Ui, app: &mut SimulatorApp, index: usize) {
    let Some((_pos, board)) = app.esp32_s3_boards.get_mut(index) else {
        return;
    };

    ui.label(RichText::new("ESP32-S3 DevKitC-1").strong().size(15.0));
    ui.label(
        RichText::new("Xtensa LX7 Dual-Core 240 MHz")
            .weak()
            .size(11.0),
    );
    ui.separator();

    let mut powered = board.is_powered();
    if ui
        .checkbox(&mut powered, "Electrical Power (3.3V / 5V)")
        .changed()
    {
        board.set_powered(powered);
    }

    let pwr_text = if board.is_power_led_on() {
        "PWR LED: Illuminated"
    } else {
        "PWR LED: Off"
    };
    ui.label(RichText::new(pwr_text).weak());

    ui.separator();
    ui.label(RichText::new("Onboard Peripherals").strong());

    let rgb = board.rgb_led();
    ui.horizontal(|ui| {
        ui.label("WS2812 RGB (IO48):");
        let (rect, _resp) = ui.allocate_exact_size(Vec2::new(14.0, 14.0), egui::Sense::hover());
        let color = if rgb.enabled {
            Color32::from_rgb(rgb.r, rgb.g, rgb.b)
        } else {
            Color32::from_rgb(40, 42, 48)
        };
        ui.painter().rect_filled(rect, 2.0, color);
        if rgb.enabled {
            ui.label(format!("({}, {}, {})", rgb.r, rgb.g, rgb.b));
        } else {
            ui.label("(Off)");
        }
    });

    let boot_pressed = board.is_boot_pressed();
    let boot_label = if boot_pressed {
        "Release BOOT (IO0)"
    } else {
        "Press BOOT (IO0)"
    };
    if ui.button(boot_label).clicked() {
        board.set_boot_pressed(!boot_pressed);
    }

    if ui.button("Reset / Restart (EN)").clicked() {
        board.reset();
    }

    ui.separator();
    ui.label(RichText::new("Firmware (Flash)").strong());
    if let Some(fw) = board.firmware() {
        ui.label(format!("Status: Loaded ({} KB)", fw.len() / 1024));
    } else {
        ui.label("Status: No firmware flashed");
    }

    if ui.button("Upload Binary (.bin)…").clicked()
        && let Some(path) = rfd::FileDialog::new()
            .add_filter("ESP Firmware", &["bin"])
            .pick_file()
        && let Ok(bytes) = std::fs::read(&path)
    {
        let _ = board.load_firmware(&bytes);
    }

    ui.separator();
    ui.label(RichText::new("UART0 Serial Monitor").strong());
    let logs = board.serial_output();
    let display_logs = if logs.is_empty() {
        "No serial data."
    } else {
        logs
    };
    egui::ScrollArea::vertical()
        .max_height(80.0)
        .show(ui, |ui| {
            ui.label(egui::RichText::new(display_logs).monospace().size(10.0));
        });

    if ui.button("Clear Serial Buffer").clicked() {
        board.clear_serial_output();
    }

    ui.separator();
    if ui.button("Delete Board").clicked() {
        app.delete_selected();
    }
}
