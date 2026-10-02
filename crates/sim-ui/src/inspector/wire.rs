use egui::{RichText, Ui};

use crate::app::SimulatorApp;
use crate::types::SelectedItem;

/// Renders inspector controls and electrical state for a selected wire.
pub fn render_wire_inspector(ui: &mut Ui, app: &mut SimulatorApp, index: usize) {
    if index >= app.wires.len() {
        return;
    }

    ui.label(RichText::new("Schematic Wire").strong());
    let wire = &mut app.wires[index];
    ui.label(format!("From Pin: {:?}", wire.from));
    ui.label(format!("To Pin: {:?}", wire.to));

    if let Some(node) = app.engine.netlist.get_node_for_pin(wire.from) {
        let state = app.engine.get_node_state(node);
        let voltage = app.engine.get_node_voltage(node);
        ui.separator();
        ui.label(format!("Net State: {:?}", state));
        ui.label(format!("Net Voltage: {:.2} V", voltage));
    }

    ui.separator();
    ui.label("Wire Label:");
    ui.text_edit_singleline(&mut wire.label);

    ui.separator();
    ui.label("Color Preset:");
    ui.horizontal(|ui| {
        if ui.button("Signal (Green)").clicked() {
            wire.color = [76, 175, 80];
        }
        if ui.button("Power (Red)").clicked() {
            wire.color = [244, 67, 54];
        }
        if ui.button("GND (Blue)").clicked() {
            wire.color = [33, 150, 243];
        }
        if ui.button("Clock (Amber)").clicked() {
            wire.color = [255, 179, 0];
        }
    });

    ui.separator();
    if ui.button("Delete Wire").clicked() {
        app.wires.remove(index);
        app.selected = SelectedItem::None;
    }
}
