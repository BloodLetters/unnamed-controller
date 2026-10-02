use sim_core::engine::Engine;

use crate::app::SimulatorApp;
use crate::types::{PROJECT_VERSION, ProjectData, SelectedItem};

/// Creates a serializable snapshot of the current project state.
pub fn create_snapshot(app: &SimulatorApp) -> ProjectData {
    let esp32_s3_boards = app
        .esp32_s3_boards
        .iter()
        .map(|(pos, _)| [pos.x, pos.y])
        .collect();

    ProjectData {
        version: PROJECT_VERSION,
        wires: app.wires.clone(),
        esp32_s3_boards,
        esp32_s3_rotations: app.esp32_s3_rotations.clone(),
        components: app.components.clone(),
        ..Default::default()
    }
}

/// Restores the simulator workspace from a project snapshot and synchronizes the netlist.
pub fn restore_snapshot(app: &mut SimulatorApp, data: ProjectData) {
    app.wires = data.wires;
    app.esp32_s3_boards.clear();
    let board_count = data.esp32_s3_boards.len();
    for (i, [x, y]) in data.esp32_s3_boards.into_iter().enumerate() {
        let base_id = (i * sim_components::board::esp32_s3::ESP32_S3_DEVKIT_PIN_COUNT) + 500;
        let board = sim_components::board::Esp32S3DevKit::new(sim_core::netlist::PinId(base_id));
        app.esp32_s3_boards.push((egui::Pos2::new(x, y), board));
    }
    app.esp32_s3_rotations = data.esp32_s3_rotations;
    app.esp32_s3_rotations.resize(app.esp32_s3_boards.len(), 0);

    app.components.clear();
    if !data.components.is_empty() {
        app.components = data.components;
    } else {
        let mut next_id =
            board_count * sim_components::board::esp32_s3::ESP32_S3_DEVKIT_PIN_COUNT + 500;
        for (i, ([x, y], color)) in data.leds.into_iter().enumerate() {
            let rot = data.led_rotations.get(i).copied().unwrap_or(0);
            let led = sim_components::component::Led::new(
                sim_core::netlist::PinId(next_id),
                sim_core::netlist::PinId(next_id + 1),
                color,
            );
            next_id += 2;
            app.components.push(crate::component::PlacedComponent::new(
                egui::Pos2::new(x, y),
                rot,
                crate::component::ComponentInstance::Led(led),
            ));
        }
        for (i, [x, y]) in data.ssd1306_displays.into_iter().enumerate() {
            let rot = data.ssd1306_rotations.get(i).copied().unwrap_or(180);
            let display = sim_components::component::Ssd1306::new(
                sim_core::netlist::PinId(next_id),
                sim_core::netlist::PinId(next_id + 1),
                sim_core::netlist::PinId(next_id + 2),
                sim_core::netlist::PinId(next_id + 3),
            );
            next_id += 4;
            app.components.push(crate::component::PlacedComponent::new(
                egui::Pos2::new(x, y),
                rot,
                crate::component::ComponentInstance::Ssd1306(display),
            ));
        }
        for (i, [x, y]) in data.lcd1602_displays.into_iter().enumerate() {
            let rot = data.lcd1602_rotations.get(i).copied().unwrap_or(0);
            let display = sim_components::component::Lcd1602::new(
                sim_core::netlist::PinId(next_id),
                sim_core::netlist::PinId(next_id + 1),
                sim_core::netlist::PinId(next_id + 2),
                sim_core::netlist::PinId(next_id + 3),
            );
            next_id += 4;
            app.components.push(crate::component::PlacedComponent::new(
                egui::Pos2::new(x, y),
                rot,
                crate::component::ComponentInstance::Lcd1602(display),
            ));
        }
        for (i, [x, y]) in data.servos.into_iter().enumerate() {
            let rot = data.servo_rotations.get(i).copied().unwrap_or(0);
            let servo = sim_components::component::Servo::new(
                sim_core::netlist::PinId(next_id),
                sim_core::netlist::PinId(next_id + 1),
                sim_core::netlist::PinId(next_id + 2),
            );
            next_id += 3;
            app.components.push(crate::component::PlacedComponent::new(
                egui::Pos2::new(x, y),
                rot,
                crate::component::ComponentInstance::Servo(servo),
            ));
        }
    }

    app.selected = SelectedItem::None;
    app.drawing_wire = None;
    app.rebuild_netlist();
}

/// Rebuilds the netlist connections in the simulation engine.
pub fn rebuild_netlist(app: &mut SimulatorApp) {
    use sim_components::board::Board;
    app.engine = Engine::new();

    for (_, board) in &app.esp32_s3_boards {
        for pin in Board::pins(board) {
            let node = app.engine.netlist.add_node();
            app.engine.netlist.connect_pin(pin, node);
        }
    }

    for comp in &app.components {
        for pin in comp.pins() {
            let node = app.engine.netlist.add_node();
            app.engine.netlist.connect_pin(pin, node);
        }
    }

    for wire in &app.wires {
        let n1 = app.engine.netlist.get_node_for_pin(wire.from);
        let n2 = app.engine.netlist.get_node_for_pin(wire.to);
        match (n1, n2) {
            (Some(n), None) => app.engine.netlist.connect_pin(wire.to, n),
            (None, Some(n)) => app.engine.netlist.connect_pin(wire.from, n),
            (None, None) => {
                let node = app.engine.netlist.add_node();
                app.engine.netlist.connect_pin(wire.from, node);
                app.engine.netlist.connect_pin(wire.to, node);
            }
            (Some(node1), Some(node2)) => {
                if node1 != node2
                    && let Some(pins) = app.engine.netlist.get_connected_pins(node2).cloned()
                {
                    for p in pins {
                        app.engine.netlist.connect_pin(p, node1);
                    }
                }
            }
        }
    }
}

/// Serializes a project snapshot to pretty-printed JSON.
pub fn save_project_json(data: &ProjectData) -> Result<String, String> {
    serde_json::to_string_pretty(data).map_err(|error| error.to_string())
}

/// Deserializes a project snapshot, rejecting data saved by a newer schema version.
pub fn load_project_json(json: &str) -> Result<ProjectData, String> {
    let data: ProjectData = serde_json::from_str(json).map_err(|error| error.to_string())?;
    if data.version > PROJECT_VERSION {
        return Err(format!(
            "versi proyek {} lebih baru dari yang didukung ({PROJECT_VERSION})",
            data.version
        ));
    }
    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_preserves_version() {
        let data = ProjectData {
            version: PROJECT_VERSION,
            ..ProjectData::default()
        };
        let json = save_project_json(&data).expect("serialize");
        let restored = load_project_json(&json).expect("deserialize");
        assert_eq!(restored.version, PROJECT_VERSION);
    }

    #[test]
    fn legacy_json_without_version_defaults_to_current() {
        let data = ProjectData {
            version: PROJECT_VERSION,
            ..ProjectData::default()
        };
        let json = save_project_json(&data).expect("serialize");
        let legacy = json.replace(&format!("\"version\": {PROJECT_VERSION},\n"), "");

        let restored = load_project_json(&legacy).expect("deserialize legacy");
        assert_eq!(restored.version, PROJECT_VERSION);
    }

    #[test]
    fn newer_version_is_rejected() {
        let data = ProjectData {
            version: PROJECT_VERSION + 1,
            ..ProjectData::default()
        };
        let json = save_project_json(&data).expect("serialize");
        assert!(load_project_json(&json).is_err());
    }

    /// Verifies that SSD1306 display instances serialize and restore cleanly.
    #[test]
    fn roundtrip_preserves_ssd1306() {
        let mut app = SimulatorApp {
            spawning: crate::types::SpawningComponent::SSD1306,
            ..Default::default()
        };
        app.spawn_component_at(egui::Pos2::new(120.0, 80.0));
        assert_eq!(app.components.len(), 1);

        let snap = app.snapshot();
        assert_eq!(snap.components.len(), 1);

        let json = save_project_json(&snap).expect("serialize");
        let restored_data = load_project_json(&json).expect("deserialize");
        assert_eq!(restored_data.components.len(), 1);

        let mut restored_app = SimulatorApp::default();
        restored_app.restore_snapshot(restored_data);
        assert_eq!(restored_app.components.len(), 1);
        assert_eq!(restored_app.components[0].pos, egui::Pos2::new(120.0, 80.0));
    }

    /// Verifies that LCD1602 display instances serialize and restore cleanly.
    #[test]
    fn roundtrip_preserves_lcd1602() {
        let mut app = SimulatorApp {
            spawning: crate::types::SpawningComponent::LCD1602,
            ..Default::default()
        };
        app.spawn_component_at(egui::Pos2::new(200.0, 150.0));
        assert_eq!(app.components.len(), 1);

        let snap = app.snapshot();
        assert_eq!(snap.components.len(), 1);

        let json = save_project_json(&snap).expect("serialize");
        let restored_data = load_project_json(&json).expect("deserialize");
        assert_eq!(restored_data.components.len(), 1);

        let mut restored_app = SimulatorApp::default();
        restored_app.restore_snapshot(restored_data);
        assert_eq!(restored_app.components.len(), 1);
        assert_eq!(
            restored_app.components[0].pos,
            egui::Pos2::new(200.0, 150.0)
        );
    }

    /// Verifies that SG90 servo instances serialize and restore cleanly.
    #[test]
    fn roundtrip_preserves_servo() {
        let mut app = SimulatorApp {
            spawning: crate::types::SpawningComponent::SERVO,
            ..Default::default()
        };
        app.spawn_component_at(egui::Pos2::new(50.0, 60.0));
        assert_eq!(app.components.len(), 1);

        let snap = app.snapshot();
        assert_eq!(snap.components.len(), 1);

        let json = save_project_json(&snap).expect("serialize");
        let restored_data = load_project_json(&json).expect("deserialize");
        assert_eq!(restored_data.components.len(), 1);

        let mut restored_app = SimulatorApp::default();
        restored_app.restore_snapshot(restored_data);
        assert_eq!(restored_app.components.len(), 1);
        assert_eq!(restored_app.components[0].pos, egui::Pos2::new(50.0, 60.0));
    }

    /// Verifies that DHT22 sensor instances serialize and restore cleanly.
    #[test]
    fn roundtrip_preserves_dht22() {
        let mut app = SimulatorApp {
            spawning: crate::types::SpawningComponent::DHT22,
            ..Default::default()
        };
        app.spawn_component_at(egui::Pos2::new(75.0, 85.0));
        assert_eq!(app.components.len(), 1);

        let snap = app.snapshot();
        let json = save_project_json(&snap).expect("serialize");
        let restored_data = load_project_json(&json).expect("deserialize");
        assert_eq!(restored_data.components.len(), 1);

        let mut restored_app = SimulatorApp::default();
        restored_app.restore_snapshot(restored_data);
        assert_eq!(restored_app.components.len(), 1);
        assert_eq!(restored_app.components[0].pos, egui::Pos2::new(75.0, 85.0));
    }
}
