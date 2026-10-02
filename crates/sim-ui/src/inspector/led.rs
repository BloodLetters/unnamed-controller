//! Property inspector panel for discrete LED components.

use egui::{Color32, RichText, Ui, Vec2};
use sim_components::component::LedColor;

/// Renders inspector controls and telemetry for a single LED component.
pub fn render_led_inspector_content(
    ui: &mut Ui,
    led: &mut sim_components::component::Led,
    engine: &sim_core::engine::Engine,
    on_delete: impl FnOnce(),
    on_rebuild_netlist: impl FnOnce(),
) {
    ui.label(RichText::new("5mm Discrete LED").strong().size(15.0));
    ui.label(
        RichText::new("Through-Hole Semiconductor Diode")
            .weak()
            .size(11.0),
    );
    ui.separator();

    ui.horizontal(|ui| {
        ui.label("Color Preset:");
        let current_color = led.color;
        egui::ComboBox::from_id_source("led_color_picker")
            .selected_text(current_color.name())
            .show_ui(ui, |ui| {
                for color in LedColor::ALL {
                    if ui
                        .selectable_label(current_color == color, color.name())
                        .clicked()
                    {
                        led.color = color;
                    }
                }
            });
    });

    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label("State:");
        let (rect, _resp) = ui.allocate_exact_size(Vec2::new(12.0, 12.0), egui::Sense::hover());
        let [r, g, b] = if led.is_lit() {
            led.color.rgb()
        } else {
            led.color.off_rgb()
        };
        ui.painter()
            .circle_filled(rect.center(), 5.0, Color32::from_rgb(r, g, b));

        if led.is_lit() {
            ui.label(
                RichText::new(format!("Illuminated ({:.0}%)", led.brightness() * 100.0))
                    .color(Color32::from_rgb(100, 255, 100)),
            );
        } else {
            ui.label(RichText::new("Off").weak());
        }
    });

    ui.separator();
    ui.label(RichText::new("Electrical Specs").strong());
    ui.label(format!(
        "Forward Voltage (Vf): {:.2} V",
        led.color.forward_voltage()
    ));

    let anode_node = engine.netlist.get_node_for_pin(led.anode());
    let cathode_node = engine.netlist.get_node_for_pin(led.cathode());
    let v_anode = anode_node
        .map(|n| engine.get_node_voltage(n))
        .unwrap_or(0.0);
    let v_cathode = cathode_node
        .map(|n| engine.get_node_voltage(n))
        .unwrap_or(0.0);

    ui.label(format!(
        "Anode Pin (+): {:?} ({:.2} V)",
        led.anode(),
        v_anode
    ));
    ui.label(format!(
        "Cathode Pin (-): {:?} ({:.2} V)",
        led.cathode(),
        v_cathode
    ));
    ui.label(format!("Potential Diff: {:.2} V", v_anode - v_cathode));

    ui.separator();
    if ui.button("⇄ Flip Polarity (A ↔ K)").clicked() {
        led.flip_polarity();
        on_rebuild_netlist();
    }

    ui.separator();
    if ui.button("Delete LED").clicked() {
        on_delete();
    }
}
