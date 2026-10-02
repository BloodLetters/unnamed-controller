//! Property inspector panel for SSD1306 OLED display modules.

use egui::{Color32, RichText, Ui, Vec2};

use crate::sim::get_pin_voltage;

/// Renders inspector controls and telemetry for a single SSD1306 display module.
pub fn render_ssd1306_inspector_content(
    ui: &mut Ui,
    display: &mut sim_components::component::Ssd1306,
    engine: &sim_core::engine::Engine,
    on_delete: impl FnOnce(),
) {
    ui.label(RichText::new("SSD1306 OLED 0.96\"").strong().size(15.0));
    ui.label(
        RichText::new("128x64 Monochrome I2C Graphic Display")
            .weak()
            .size(11.0),
    );
    ui.separator();

    render_display_state(ui, display);
    ui.separator();

    render_protocol_specs(ui, display);
    ui.separator();

    render_pin_voltages(ui, engine, display);
    ui.separator();

    render_diagnostic_buttons(ui, display, on_delete);
}

/// Renders the operational power state and active display modes.
fn render_display_state(ui: &mut Ui, display: &sim_components::component::Ssd1306) {
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

    ui.add_space(2.0);
    ui.label("Resolution: 128 x 64 pixels (1024 B)");
    ui.label(format!("Contrast: {} / 255", display.contrast()));

    if display.is_inverted() {
        ui.label(
            RichText::new("Inverse Display Mode Active")
                .color(Color32::from_rgb(250, 204, 21))
                .size(11.0),
        );
    }
    if display.is_entire_on() {
        ui.label(
            RichText::new("Entire Display Force ON (A5h)")
                .color(Color32::from_rgb(250, 204, 21))
                .size(11.0),
        );
    }
}

/// Renders I2C bus address and memory addressing configuration.
fn render_protocol_specs(ui: &mut Ui, display: &sim_components::component::Ssd1306) {
    ui.label(RichText::new("Controller Architecture").strong());
    ui.label(format!(
        "I2C Address: 0x{:02X} (7-bit)",
        display.i2c_address()
    ));

    let mode_str = match display.addressing_mode() {
        sim_components::component::AddressingMode::Page => "Page Addressing (02h)",
        sim_components::component::AddressingMode::Horizontal => "Horizontal Addressing (00h)",
        sim_components::component::AddressingMode::Vertical => "Vertical Addressing (01h)",
    };
    ui.label(format!("Addressing: {mode_str}"));
}

/// Renders the 4 terminal pins with real-time netlist voltages.
fn render_pin_voltages(
    ui: &mut Ui,
    engine: &sim_core::engine::Engine,
    display: &sim_components::component::Ssd1306,
) {
    ui.label(RichText::new("Terminal Pinout & Voltages").strong());

    let pins = [
        ("Pin 1 [GND]", display.gnd()),
        ("Pin 2 [VCC]", display.vcc()),
        ("Pin 3 [SDA]", display.sda()),
        ("Pin 4 [SCL]", display.scl()),
    ];

    for (name, pin) in pins {
        let voltage = get_pin_voltage(engine, pin);
        ui.label(format!("{name}: PinId({}) → {voltage:.2} V", pin.0));
    }
}

/// Renders interactive action buttons for test pattern, clear, and deletion.
fn render_diagnostic_buttons(
    ui: &mut Ui,
    display: &mut sim_components::component::Ssd1306,
    on_delete: impl FnOnce(),
) {
    ui.label(RichText::new("Diagnostics & Controls").strong());

    ui.horizontal(|ui| {
        if ui.button("⚡ Test Pattern").clicked() {
            display.draw_test_pattern();
        }
        if ui.button("⌧ Clear Screen").clicked() {
            display.clear();
        }
    });

    ui.horizontal(|ui| {
        let pwr_label = if display.is_display_on() {
            "Power Off"
        } else {
            "Power On"
        };
        if ui.button(pwr_label).clicked() {
            display.toggle_power();
        }
        if ui.button("Invert").clicked() {
            display.toggle_inverse();
        }
    });

    ui.separator();
    if ui.button("Delete Display").clicked() {
        on_delete();
    }
}
