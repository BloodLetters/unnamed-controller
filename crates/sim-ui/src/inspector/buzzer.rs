//! Property inspector panel for active buzzer components.

use egui::{Color32, RichText, Ui, Vec2};
use sim_components::component::{BUZZER_TYPICAL_CURRENT_MA, BUZZER_TYPICAL_FREQUENCY_HZ, Buzzer};

/// Renders inspector status, electrical telemetry, and delete control for a buzzer.
pub fn render_buzzer_inspector_content(
    ui: &mut Ui,
    buzzer: &mut Buzzer,
    engine: &sim_core::engine::Engine,
    on_delete: impl FnOnce(),
) {
    ui.label(RichText::new("Active Buzzer").strong().size(15.0));
    ui.label(
        RichText::new("5V Internal-Oscillator Piezoelectric Transducer")
            .weak()
            .size(11.0),
    );
    ui.separator();

    render_buzzer_status(ui, buzzer);
    ui.separator();

    render_buzzer_specs(ui);
    ui.separator();

    render_buzzer_electrical(ui, engine, buzzer);
    ui.separator();

    if ui.button("Delete Buzzer").clicked() {
        on_delete();
    }
}

/// Renders the activation state indicator with visual feedback.
fn render_buzzer_status(ui: &mut Ui, buzzer: &Buzzer) {
    ui.horizontal(|ui| {
        ui.label("State:");
        let (rect, _resp) = ui.allocate_exact_size(Vec2::new(12.0, 12.0), egui::Sense::hover());
        let indicator_color = if buzzer.is_active() {
            Color32::from_rgb(255, 200, 60)
        } else {
            Color32::from_rgb(60, 62, 68)
        };
        ui.painter()
            .circle_filled(rect.center(), 5.0, indicator_color);

        if buzzer.is_active() {
            ui.label(RichText::new("Active (Beeping)").color(Color32::from_rgb(255, 210, 80)));
        } else {
            ui.label(RichText::new("Inactive (Silent)").weak());
        }
    });
}

/// Renders fixed datasheet specifications for the active buzzer.
fn render_buzzer_specs(ui: &mut Ui) {
    ui.label(RichText::new("Datasheet Specs").strong());
    ui.label(format!("Frequency: {} Hz", BUZZER_TYPICAL_FREQUENCY_HZ));
    ui.label(format!(
        "Typical Current: {:.0} mA @ 5 V",
        BUZZER_TYPICAL_CURRENT_MA
    ));
    ui.label("Min Operating Voltage: 3.0 V");
    ui.label("Type: Active (internal oscillator)");
}

/// Renders real-time voltage measurements on VCC and GND terminals.
fn render_buzzer_electrical(ui: &mut Ui, engine: &sim_core::engine::Engine, buzzer: &Buzzer) {
    ui.label(RichText::new("Terminal Telemetry").strong());

    let v_vcc = crate::sim::get_pin_voltage(engine, buzzer.vcc());
    let v_gnd = crate::sim::get_pin_voltage(engine, buzzer.gnd());

    ui.label(format!("VCC Pin: {:?} ({:.2} V)", buzzer.vcc(), v_vcc));
    ui.label(format!("GND Pin: {:?} ({:.2} V)", buzzer.gnd(), v_gnd));
    ui.label(format!("Supply Potential: {:.2} V", v_vcc - v_gnd));
}
