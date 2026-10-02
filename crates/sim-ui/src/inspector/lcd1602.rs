//! Property inspector panel for HD44780 1602 character LCD modules.

use egui::{Color32, FontId, RichText, Ui, Vec2};
use sim_components::component::{Lcd1602, LcdBacklightColor};

use crate::sim::get_pin_voltage;

/// Renders inspector controls and telemetry for a single 1602 LCD module.
pub fn render_lcd1602_inspector_content(
    ui: &mut Ui,
    display: &mut Lcd1602,
    engine: &sim_core::engine::Engine,
    on_delete: impl FnOnce(),
) {
    ui.label(
        RichText::new("LCD 1602 Character Display")
            .strong()
            .size(15.0),
    );
    ui.label(
        RichText::new("16x2 HD44780 with PCF8574 I2C Backpack")
            .weak()
            .size(11.0),
    );
    ui.separator();

    render_display_state(ui, display);
    ui.separator();

    render_screen_preview(ui, display);
    ui.separator();

    render_protocol_specs(ui, display);
    ui.separator();

    render_pin_voltages(ui, engine, display);
    ui.separator();

    render_diagnostic_buttons(ui, display, on_delete);
}

/// Renders the operational power state and backlight configuration.
fn render_display_state(ui: &mut Ui, display: &mut Lcd1602) {
    ui.horizontal(|ui| {
        ui.label("Display Power:");
        let (rect, _resp) = ui.allocate_exact_size(Vec2::new(12.0, 12.0), egui::Sense::hover());
        let (dot_color, status_text) = if display.is_display_on() {
            (Color32::from_rgb(0, 229, 255), "Active (ON)")
        } else {
            (Color32::from_rgb(100, 110, 125), "Sleep (OFF)")
        };

        ui.painter().circle_filled(rect.center(), 5.0, dot_color);
        ui.label(
            RichText::new(status_text)
                .color(dot_color)
                .size(12.0)
                .strong(),
        );
    });

    ui.horizontal(|ui| {
        ui.label("Backlight:");
        let bl_text = if display.is_backlight_on() {
            "Illuminated"
        } else {
            "Dark"
        };
        ui.label(RichText::new(bl_text).strong());
    });

    ui.horizontal(|ui| {
        ui.label("Color Theme:");
        let mut color = display.backlight_color();
        ui.selectable_value(&mut color, LcdBacklightColor::Blue, "Blue");
        ui.selectable_value(&mut color, LcdBacklightColor::Green, "Green");
        display.set_backlight_color(color);
    });
}

/// Renders a monospace preview of the current 16x2 character text cells.
fn render_screen_preview(ui: &mut Ui, display: &Lcd1602) {
    ui.label(RichText::new("Screen Contents:").strong());

    let (bg, fg) = match display.backlight_color() {
        LcdBacklightColor::Blue => (
            Color32::from_rgb(10, 60, 160),
            Color32::from_rgb(230, 245, 255),
        ),
        LcdBacklightColor::Green => (
            Color32::from_rgb(110, 165, 20),
            Color32::from_rgb(20, 35, 10),
        ),
    };

    let pcb_rect = ui.allocate_space(Vec2::new(ui.available_width(), 44.0)).1;
    ui.painter().rect_filled(pcb_rect, 3.0, bg);

    let l1 = display.get_row_str(0);
    let l2 = display.get_row_str(1);

    ui.painter().text(
        pcb_rect.min + Vec2::new(8.0, 12.0),
        egui::Align2::LEFT_CENTER,
        &l1,
        FontId::monospace(12.0),
        fg,
    );
    ui.painter().text(
        pcb_rect.min + Vec2::new(8.0, 30.0),
        egui::Align2::LEFT_CENTER,
        &l2,
        FontId::monospace(12.0),
        fg,
    );
}

/// Renders bus communications details.
fn render_protocol_specs(ui: &mut Ui, display: &Lcd1602) {
    ui.label(RichText::new("I2C Telemetry").strong());
    ui.horizontal(|ui| {
        ui.label("Bus Address:");
        ui.label(
            RichText::new(format!("0x{:02X}", display.address()))
                .strong()
                .color(Color32::from_rgb(0, 229, 255)),
        );
    });
    ui.label("Architecture: HD44780 4-bit parallel over PCF8574");
}

/// Renders live analog voltage measurements on connected header pins.
fn render_pin_voltages(ui: &mut Ui, engine: &sim_core::engine::Engine, display: &Lcd1602) {
    ui.label(RichText::new("Terminal Voltage Measurements").strong());

    let pins = [
        ("GND", display.gnd()),
        ("VCC", display.vcc()),
        ("SDA", display.sda()),
        ("SCL", display.scl()),
    ];

    for (name, pin_id) in pins {
        let v = get_pin_voltage(engine, pin_id);
        ui.horizontal(|ui| {
            ui.label(format!("{name}:"));
            ui.label(format!("{v:.2} V"));
        });
    }
}

/// Renders test actions for manipulating the active display.
fn render_diagnostic_buttons(ui: &mut Ui, display: &mut Lcd1602, on_delete: impl FnOnce()) {
    ui.horizontal(|ui| {
        if ui.button("🧹 Clear Display").clicked() {
            display.clear();
        }

        if ui.button("⚡ Test Pattern").clicked() {
            display.clear();
            for &c in b"UNNAMED CONTROLR" {
                display.write_char(c);
            }
            display.set_cursor(0, 1);
            for &c in b"LCD 1602 READY! " {
                display.write_char(c);
            }
        }
    });

    ui.separator();
    if ui.button("Delete Display").clicked() {
        on_delete();
    }
}
