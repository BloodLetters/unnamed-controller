//! Header terminal specifications and pinout mapping for ESP32-S3-DevKitC-1.

pub mod tables;

use sim_core::netlist::PinId;
use std::collections::HashMap;

use crate::board::types::HeaderPin;
use tables::{LEFT_HEADER_SPECS, RIGHT_HEADER_SPECS};

/// Total count of physical header pins exposed by the ESP32-S3-DevKitC-1 (2x22 headers).
pub const ESP32_S3_DEVKIT_PIN_COUNT: usize = 44;

/// Builds the complete list of 44 header pins using sequentially allocated PinIds.
pub fn build_devkit_pins(pins: &[PinId]) -> Vec<HeaderPin> {
    let mut header_pins = Vec::with_capacity(ESP32_S3_DEVKIT_PIN_COUNT);

    let specs = LEFT_HEADER_SPECS.iter().chain(RIGHT_HEADER_SPECS.iter());
    for (i, spec) in specs.enumerate() {
        let pin_id = pins.get(i).copied().unwrap_or(PinId(i));
        header_pins.push(HeaderPin::new(
            pin_id,
            spec.name,
            spec.side,
            spec.index,
            spec.gpio,
            spec.is_power,
            spec.is_ground,
        ));
    }

    header_pins
}

/// Builds a mapping from GPIO number to its corresponding PinId.
pub fn build_gpio_map(header_pins: &[HeaderPin]) -> HashMap<u8, PinId> {
    let mut map = HashMap::new();
    for pin in header_pins {
        if let Some(gpio) = pin.gpio {
            map.insert(gpio, pin.pin_id);
        }
    }
    map
}
