//! Canvas geometry and painting delegation for placed component instances.
//!
//! Split out from `instance.rs` so the enum declaration and lifecycle delegation
//! stay readable as component types are added.

use egui::{Painter, Pos2, Vec2};
use sim_core::netlist::PinId;

use crate::component::instance::ComponentInstance;

impl ComponentInstance {
    /// Relative offsets (dx, dy) of each terminal pin from component center.
    pub fn pin_offsets(&self) -> Vec<(PinId, Vec2)> {
        match self {
            Self::Led(c) => {
                let ox = crate::canvas::components::led::LED_PIN_OFFSET_X;
                let oy = crate::canvas::components::led::LED_PIN_OFFSET_Y;
                vec![
                    (c.anode(), Vec2::new(-ox, oy)),
                    (c.cathode(), Vec2::new(ox, oy)),
                ]
            }
            Self::Ssd1306(c) => {
                let offsets = crate::canvas::components::ssd1306::ssd1306_pin_offsets();
                vec![
                    (c.gnd(), Vec2::new(offsets[0].0, offsets[0].1)),
                    (c.vcc(), Vec2::new(offsets[1].0, offsets[1].1)),
                    (c.sda(), Vec2::new(offsets[2].0, offsets[2].1)),
                    (c.scl(), Vec2::new(offsets[3].0, offsets[3].1)),
                ]
            }
            Self::Lcd1602(c) => {
                let offsets = crate::canvas::components::lcd1602::lcd1602_pin_offsets();
                vec![
                    (c.gnd(), Vec2::new(offsets[0].0, offsets[0].1)),
                    (c.vcc(), Vec2::new(offsets[1].0, offsets[1].1)),
                    (c.sda(), Vec2::new(offsets[2].0, offsets[2].1)),
                    (c.scl(), Vec2::new(offsets[3].0, offsets[3].1)),
                ]
            }
            Self::Servo(c) => {
                let offsets = crate::canvas::components::servo::servo_pin_offsets();
                vec![
                    (c.gnd(), Vec2::new(offsets[0].0, offsets[0].1)),
                    (c.vcc(), Vec2::new(offsets[1].0, offsets[1].1)),
                    (c.pwm(), Vec2::new(offsets[2].0, offsets[2].1)),
                ]
            }
            Self::Dht22(c) => {
                let offsets = crate::canvas::components::dht22::dht22_pin_offsets();
                vec![
                    (c.vcc(), Vec2::new(offsets[0].0, offsets[0].1)),
                    (c.data(), Vec2::new(offsets[1].0, offsets[1].1)),
                    (c.nc(), Vec2::new(offsets[2].0, offsets[2].1)),
                    (c.gnd(), Vec2::new(offsets[3].0, offsets[3].1)),
                ]
            }
            Self::Mq135(c) => {
                let offsets = crate::canvas::components::mq135::mq135_pin_offsets();
                vec![
                    (c.vcc(), Vec2::new(offsets[0].0, offsets[0].1)),
                    (c.gnd(), Vec2::new(offsets[1].0, offsets[1].1)),
                    (c.aout(), Vec2::new(offsets[2].0, offsets[2].1)),
                    (c.dout(), Vec2::new(offsets[3].0, offsets[3].1)),
                ]
            }
            Self::Buzzer(c) => {
                let offsets = crate::canvas::components::buzzer::buzzer_pin_offsets();
                vec![
                    (c.vcc(), Vec2::new(offsets[0].0, offsets[0].1)),
                    (c.gnd(), Vec2::new(offsets[1].0, offsets[1].1)),
                ]
            }
            Self::Resistor(c) => {
                let ox = crate::canvas::components::resistor::RESISTOR_PIN_OFFSET_X;
                vec![
                    (c.terminal_a(), Vec2::new(-ox, 0.0)),
                    (c.terminal_b(), Vec2::new(ox, 0.0)),
                ]
            }
            Self::Button(c) => {
                let offsets = crate::canvas::components::button::button_pin_offsets();
                vec![
                    (c.pin_1a(), Vec2::new(offsets[0].0, offsets[0].1)),
                    (c.pin_1b(), Vec2::new(offsets[1].0, offsets[1].1)),
                    (c.pin_2a(), Vec2::new(offsets[2].0, offsets[2].1)),
                    (c.pin_2b(), Vec2::new(offsets[3].0, offsets[3].1)),
                ]
            }
        }
    }

    /// Renders the component package, labels, terminals, and interactive feedback.
    pub fn draw(&self, painter: &Painter, center: Pos2, zoom: f32, rot: u16, is_selected: bool) {
        match self {
            Self::Led(c) => {
                crate::canvas::components::led::draw_single_led(
                    painter,
                    center,
                    zoom,
                    rot,
                    c,
                    is_selected,
                );
            }
            Self::Ssd1306(c) => {
                crate::canvas::components::ssd1306::draw_single_ssd1306(
                    painter,
                    center,
                    zoom,
                    rot,
                    c,
                    is_selected,
                );
            }
            Self::Lcd1602(c) => {
                crate::canvas::components::lcd1602::draw_single_lcd1602(
                    painter,
                    center,
                    zoom,
                    rot,
                    c,
                    is_selected,
                );
            }
            Self::Servo(c) => {
                crate::canvas::components::servo::draw_single_servo(
                    painter,
                    center,
                    zoom,
                    rot,
                    c,
                    is_selected,
                );
            }
            Self::Dht22(c) => {
                crate::canvas::components::dht22::draw_single_dht22(
                    painter,
                    center,
                    zoom,
                    rot,
                    c,
                    is_selected,
                );
            }
            Self::Mq135(c) => {
                crate::canvas::components::mq135::draw_single_mq135(
                    painter,
                    center,
                    zoom,
                    rot,
                    c,
                    is_selected,
                );
            }
            Self::Buzzer(c) => {
                crate::canvas::components::buzzer::draw_single_buzzer(
                    painter,
                    center,
                    zoom,
                    rot,
                    c,
                    is_selected,
                );
            }
            Self::Resistor(c) => {
                crate::canvas::components::resistor::draw_single_resistor(
                    painter,
                    center,
                    zoom,
                    rot,
                    c,
                    is_selected,
                );
            }
            Self::Button(c) => {
                crate::canvas::components::button::draw_single_button(
                    painter,
                    center,
                    zoom,
                    rot,
                    c,
                    is_selected,
                );
            }
        }
    }
}
