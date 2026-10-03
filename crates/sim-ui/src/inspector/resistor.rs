//! Property inspector panel for resistor components.

use egui::{DragValue, RichText, Ui};
use sim_components::component::Resistor;

/// Power rating of a standard quarter-watt through-hole resistor.
const POWER_RATING_MW: f32 = 250.0;

/// Renders inspector controls and live telemetry for a single resistor.
pub fn render_resistor_inspector_content(
    ui: &mut Ui,
    resistor: &mut Resistor,
    engine: &sim_core::engine::Engine,
    on_delete: impl FnOnce(),
) {
    ui.label(RichText::new("Resistor").strong().size(15.0));
    ui.label(
        RichText::new("Axial Through-Hole Passive (E24 series)")
            .weak()
            .size(11.0),
    );
    ui.separator();

    let mut resistance = resistor.resistance_ohms();
    let response = ui.horizontal(|ui| {
        ui.label("Resistance:");
        ui.add(
            DragValue::new(&mut resistance)
                .speed(1.0)
                .suffix(" \u{3a9}"),
        )
    });
    if response.inner.changed() {
        resistor.set_resistance_ohms(resistance);
    }

    ui.horizontal(|ui| {
        for preset in [220.0_f32, 1_000.0, 10_000.0] {
            if ui
                .button(crate::canvas::components::resistor::format_resistance(
                    preset,
                ))
                .clicked()
            {
                resistor.set_resistance_ohms(preset);
            }
        }
    });

    ui.separator();
    ui.label(RichText::new("Live Electrical Readout").strong());

    let terminal_a = engine
        .netlist
        .get_node_for_pin(resistor.terminal_a())
        .map(|node| engine.get_node_voltage(node))
        .unwrap_or(0.0);
    let terminal_b = engine
        .netlist
        .get_node_for_pin(resistor.terminal_b())
        .map(|node| engine.get_node_voltage(node))
        .unwrap_or(0.0);

    let current_ma = resistor.current_amps(terminal_a, terminal_b) * 1_000.0;
    let power_mw = resistor.power_watts(terminal_a, terminal_b) * 1_000.0;

    ui.label(format!("Terminal A: {:.2} V", terminal_a));
    ui.label(format!("Terminal B: {:.2} V", terminal_b));
    ui.label(format!(
        "Voltage Drop: {:.3} V",
        resistor.voltage_drop(terminal_a, terminal_b)
    ));
    ui.label(format!("Current: {:.2} mA", current_ma));
    ui.label(format!("Dissipated: {:.2} mW", power_mw));

    if power_mw > POWER_RATING_MW {
        ui.label(
            RichText::new(format!(
                "Exceeds the {POWER_RATING_MW:.0} mW rating of a 1/4W part"
            ))
            .color(egui::Color32::from_rgb(255, 130, 130)),
        );
    }

    ui.separator();
    if ui.button("Delete Resistor").clicked() {
        on_delete();
    }
}
