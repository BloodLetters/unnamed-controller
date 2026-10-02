use egui::Vec2;

use crate::app::SimulatorApp;
use crate::registry::{CATALOG, find};
use crate::types::SelectedItem;

/// Applies a displacement delta to an individual selectable component.
pub fn apply_delta_to_item(app: &mut SimulatorApp, item: &SelectedItem, delta: Vec2) {
    if let Some((spec, index)) = find(item) {
        (spec.move_by)(app, index, delta);
    }
}

/// Moves either a single item or the entire multi-selected group by a canvas displacement delta.
pub fn move_item_or_group(app: &mut SimulatorApp, item: SelectedItem, delta: Vec2) {
    match &app.selected {
        SelectedItem::Group(items) if items.contains(&item) => {
            let cloned = items.clone();
            for it in &cloned {
                apply_delta_to_item(app, it, delta);
            }
        }
        _ => {
            apply_delta_to_item(app, &item, delta);
        }
    }
}

/// Returns the next unused pin identifier across all components and wires.
#[allow(dead_code)]
pub(crate) fn next_pin_id(app: &SimulatorApp) -> usize {
    let mut next = 500;
    for spec in CATALOG {
        for index in 0..(spec.count)(app) {
            for pin in (spec.pins)(app, index) {
                next = next.max(pin.0.saturating_add(1));
            }
        }
    }
    for wire in &app.wires {
        next = next.max(wire.from.0.saturating_add(1));
        next = next.max(wire.to.0.saturating_add(1));
    }
    next
}

/// Duplicates the currently selected component with an offset and allocates fresh pins.
pub fn duplicate_selected(app: &mut SimulatorApp) {
    match app.selected.clone() {
        SelectedItem::Esp32S3(idx) => {
            if let Some(&(pos, _)) = app.esp32_s3_boards.get(idx) {
                let rot = app.esp32_s3_rotations.get(idx).copied().unwrap_or(0);
                let base_id = next_pin_id(app);
                let board =
                    sim_components::board::Esp32S3DevKit::new(sim_core::netlist::PinId(base_id));
                let new_idx = app.esp32_s3_boards.len();
                app.esp32_s3_boards
                    .push((pos + Vec2::new(30.0, 30.0), board));
                app.esp32_s3_rotations.push(rot);
                app.selected = SelectedItem::Esp32S3(new_idx);
                app.rebuild_netlist();
            }
        }
        SelectedItem::Component(idx) => {
            if let Some(comp) = app.components.get(idx) {
                let mut pin_counter = next_pin_id(app);
                let mut duplicated = comp.duplicate(&mut || {
                    let id = pin_counter;
                    pin_counter += 1;
                    sim_core::netlist::PinId(id)
                });
                duplicated.pos += Vec2::new(20.0, 20.0);
                let new_idx = app.components.len();
                app.components.push(duplicated);
                app.selected = SelectedItem::Component(new_idx);
                app.rebuild_netlist();
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::Pos2;

    #[test]
    fn test_duplicate_led() {
        let mut app = SimulatorApp {
            spawning: crate::types::SpawningComponent::LED,
            ..Default::default()
        };
        app.spawn_component_at(Pos2::new(50.0, 50.0));
        assert_eq!(app.components.len(), 1);
        app.selected = SelectedItem::Component(0);
        app.rotate_selected(90);
        assert_eq!(app.components[0].rotation, 90);

        app.duplicate_selected();
        assert_eq!(app.components.len(), 2);
        assert_eq!(app.components[1].rotation, 90);
        assert_eq!(app.selected, SelectedItem::Component(1));
    }

    #[test]
    fn test_duplicate_ssd1306() {
        let mut app = SimulatorApp {
            spawning: crate::types::SpawningComponent::SSD1306,
            ..Default::default()
        };
        app.spawn_component_at(Pos2::new(100.0, 100.0));
        assert_eq!(app.components.len(), 1);
        assert_eq!(app.components[0].rotation, 180);
        app.selected = SelectedItem::Component(0);
        app.rotate_selected(90);
        assert_eq!(app.components[0].rotation, 270);

        app.duplicate_selected();
        assert_eq!(app.components.len(), 2);
        assert_eq!(app.components[1].rotation, 270);
        assert_eq!(app.selected, SelectedItem::Component(1));
    }
}
