use egui::Pos2;
use sim_components::board::Esp32S3DevKit;
use sim_core::netlist::PinId;

use crate::app::SimulatorApp;
use crate::state::mutation::next_pin_id;
use crate::types::{SelectedItem, SpawningComponent};

/// Spawns a component or board instance at the specified canvas coordinate.
pub fn spawn_component_at(app: &mut SimulatorApp, local_pos: Pos2) {
    match app.spawning {
        SpawningComponent::None => {}
        SpawningComponent::Esp32S3 => {
            let base_id = next_pin_id(app);
            let board = Esp32S3DevKit::new(PinId(base_id));
            let new_index = app.esp32_s3_boards.len();
            app.esp32_s3_boards.push((local_pos, board));
            app.esp32_s3_rotations.push(0);
            app.selected = SelectedItem::Esp32S3(new_index);
            app.spawning = SpawningComponent::None;
            app.rebuild_netlist();
        }
        SpawningComponent::Component(kind) => {
            let mut pin_counter = next_pin_id(app);
            let instance = kind.create(&mut || {
                let id = pin_counter;
                pin_counter += 1;
                PinId(id)
            });
            let rotation = kind.default_rotation();
            let new_index = app.components.len();
            app.components.push(crate::component::PlacedComponent::new(
                local_pos, rotation, instance,
            ));
            app.selected = SelectedItem::Component(new_index);
            app.spawning = SpawningComponent::None;
            app.rebuild_netlist();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_none_noop() {
        let mut app = SimulatorApp::default();
        let initial_count = app.esp32_s3_boards.len();
        spawn_component_at(&mut app, Pos2::ZERO);
        assert_eq!(app.esp32_s3_boards.len(), initial_count);
    }
}
