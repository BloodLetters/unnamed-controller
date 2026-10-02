use egui::Pos2;
use sim_core::component::Component;
use sim_core::netlist::PinId;

use crate::app::SimulatorApp;
use crate::registry::CATALOG;
use crate::types::SelectedItem;

/// Helper to delete a single component from a vector and collect its pins.
pub fn remove_indexed_component<T: Component>(
    vec: &mut Vec<(Pos2, T)>,
    index: usize,
    removed_pins: &mut Vec<PinId>,
) {
    if index < vec.len() {
        let comp = vec.remove(index).1;
        removed_pins.extend(comp.pins());
    }
}

/// Deletes the currently selected component, wire, or group and purges orphan wires.
pub fn delete_selected_item(app: &mut SimulatorApp) {
    let mut removed_pins = Vec::new();

    match &app.selected {
        SelectedItem::None => {}
        SelectedItem::Wire(i) => {
            if *i < app.wires.len() {
                app.wires.remove(*i);
            }
        }
        SelectedItem::Group(items) => {
            let cloned = items.clone();
            delete_group_items(app, &cloned, &mut removed_pins);
        }
        SelectedItem::Esp32S3(idx) => {
            remove_indexed_component(&mut app.esp32_s3_boards, *idx, &mut removed_pins);
            if *idx < app.esp32_s3_rotations.len() {
                app.esp32_s3_rotations.remove(*idx);
            }
        }
        SelectedItem::Component(idx) => {
            if *idx < app.components.len() {
                let comp = app.components.remove(*idx);
                removed_pins.extend(comp.pins());
            }
        }
    }

    app.selected = SelectedItem::None;
    app.wires
        .retain(|w| !removed_pins.contains(&w.from) && !removed_pins.contains(&w.to));
}

/// Deletes all components specified in a multi-selection group in descending index order.
pub fn delete_group_items(
    app: &mut SimulatorApp,
    items: &[SelectedItem],
    removed_pins: &mut Vec<PinId>,
) {
    for spec in CATALOG {
        let mut indices: Vec<usize> = items.iter().filter_map(|item| (spec.index)(item)).collect();
        if indices.is_empty() {
            continue;
        }
        indices.sort_unstable_by(|a, b| b.cmp(a));
        indices.dedup();
        for index in indices {
            (spec.remove)(app, index, removed_pins);
        }
    }
}
