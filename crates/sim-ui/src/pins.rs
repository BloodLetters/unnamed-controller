use egui::Pos2;
use sim_components::board::{Board, HeaderSide};
use sim_core::netlist::PinId;

use crate::app::SimulatorApp;

/// Horizontal distance from the board center to the left and right header pin rows.
pub const HEADER_PIN_OFFSET_X: f32 = 58.0;
/// Vertical starting offset for the top pin (index 0).
pub const HEADER_PIN_START_Y: f32 = -126.0;
/// Vertical pitch between adjacent pins along the header.
pub const HEADER_PIN_PITCH: f32 = 12.0;

/// Iterates over every pin terminal in the circuit with its canvas coordinate.
pub fn for_each_pin(app: &SimulatorApp, mut f: impl FnMut(PinId, Pos2)) {
    for (board_idx, (board_pos, board)) in app.esp32_s3_boards.iter().enumerate() {
        let rot = app.esp32_s3_rotations.get(board_idx).copied().unwrap_or(0);
        for pin in board.header_pins() {
            let offset_x = match pin.side {
                HeaderSide::Left => -HEADER_PIN_OFFSET_X,
                HeaderSide::Right => HEADER_PIN_OFFSET_X,
                HeaderSide::Top => 0.0,
                HeaderSide::Bottom => 0.0,
            };
            let offset_y = HEADER_PIN_START_Y + pin.index as f32 * HEADER_PIN_PITCH;
            let rotated = crate::canvas::rotate_offset(egui::Vec2::new(offset_x, offset_y), rot);
            let pin_pos = Pos2::new(board_pos.x + rotated.x, board_pos.y + rotated.y);
            f(pin.pin_id, pin_pos);
        }
    }

    for comp in &app.components {
        for (pin_id, pos) in comp.pin_positions() {
            f(pin_id, pos);
        }
    }
}

/// Resolves the absolute visual canvas coordinate of a specific pin.
pub fn get_pin_pos(app: &SimulatorApp, target: PinId) -> Option<Pos2> {
    let mut result = None;
    for_each_pin(app, |pin_id, pos| {
        if pin_id == target && result.is_none() {
            result = Some(pos);
        }
    });
    result
}

/// Finds the closest pin terminal to the given canvas coordinate within snap range.
pub fn find_closest_pin(app: &SimulatorApp, local_pos: Pos2) -> Option<PinId> {
    let mut closest = None;
    let mut min_dist = 10.0_f32;
    for_each_pin(app, |pin_id, pos| {
        let dist = local_pos.distance(pos);
        if dist < min_dist {
            min_dist = dist;
            closest = Some(pin_id);
        }
    });
    closest
}

/// Returns labeled terminals for canvas overlay text.
pub fn pin_labels(_app: &SimulatorApp) -> Vec<(String, Pos2)> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pin_query_default_board() {
        let mut app = SimulatorApp {
            spawning: crate::types::SpawningComponent::Esp32S3,
            ..Default::default()
        };
        app.spawn_component_at(Pos2::new(0.0, 0.0));
        assert!(!app.esp32_s3_boards.is_empty());
        let first_pin = app.esp32_s3_boards[0].1.header_pins()[0].pin_id;
        let pin0_pos = get_pin_pos(&app, first_pin);
        assert!(pin0_pos.is_some());
    }

    /// Verifies that all 4 SSD1306 pins are resolvable on the canvas.
    #[test]
    fn test_ssd1306_pins_query() {
        let mut app = SimulatorApp {
            spawning: crate::types::SpawningComponent::SSD1306,
            ..Default::default()
        };
        app.spawn_component_at(Pos2::new(100.0, 100.0));
        assert_eq!(app.components.len(), 1);

        if let crate::component::ComponentInstance::Ssd1306(ref disp) = app.components[0].instance {
            assert!(get_pin_pos(&app, disp.gnd()).is_some());
            assert!(get_pin_pos(&app, disp.vcc()).is_some());
            assert!(get_pin_pos(&app, disp.sda()).is_some());
            assert!(get_pin_pos(&app, disp.scl()).is_some());
        } else {
            panic!("Expected Ssd1306 component");
        }
    }

    /// Verifies that all 4 LCD1602 pins are resolvable on the canvas.
    #[test]
    fn test_lcd1602_pins_query() {
        let mut app = SimulatorApp {
            spawning: crate::types::SpawningComponent::LCD1602,
            ..Default::default()
        };
        app.spawn_component_at(Pos2::new(100.0, 100.0));
        assert_eq!(app.components.len(), 1);

        if let crate::component::ComponentInstance::Lcd1602(ref disp) = app.components[0].instance {
            assert!(get_pin_pos(&app, disp.gnd()).is_some());
            assert!(get_pin_pos(&app, disp.vcc()).is_some());
            assert!(get_pin_pos(&app, disp.sda()).is_some());
            assert!(get_pin_pos(&app, disp.scl()).is_some());
        } else {
            panic!("Expected Lcd1602 component");
        }
    }

    /// Verifies that all 3 SG90 servo pins are resolvable on the canvas.
    #[test]
    fn test_servo_pins_query() {
        let mut app = SimulatorApp {
            spawning: crate::types::SpawningComponent::SERVO,
            ..Default::default()
        };
        app.spawn_component_at(Pos2::new(100.0, 100.0));
        assert_eq!(app.components.len(), 1);

        if let crate::component::ComponentInstance::Servo(ref s) = app.components[0].instance {
            assert!(get_pin_pos(&app, s.gnd()).is_some());
            assert!(get_pin_pos(&app, s.vcc()).is_some());
            assert!(get_pin_pos(&app, s.pwm()).is_some());
        } else {
            panic!("Expected Servo component");
        }
    }

    /// Verifies that all 4 DHT22 pins are resolvable on the canvas.
    #[test]
    fn test_dht22_pins_query() {
        let mut app = SimulatorApp {
            spawning: crate::types::SpawningComponent::DHT22,
            ..Default::default()
        };
        app.spawn_component_at(Pos2::new(100.0, 100.0));
        assert_eq!(app.components.len(), 1);

        if let crate::component::ComponentInstance::Dht22(ref d) = app.components[0].instance {
            assert!(get_pin_pos(&app, d.vcc()).is_some());
            assert!(get_pin_pos(&app, d.data()).is_some());
            assert!(get_pin_pos(&app, d.nc()).is_some());
            assert!(get_pin_pos(&app, d.gnd()).is_some());
        } else {
            panic!("Expected Dht22 component");
        }
    }
}
