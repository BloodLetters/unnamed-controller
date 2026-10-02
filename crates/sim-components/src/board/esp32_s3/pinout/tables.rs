//! Static pinout specification tables for the 44-pin ESP32-S3-DevKitC-1 headers.

use crate::board::types::HeaderSide;

/// Raw metadata for defining DevKitC-1 header pin mapping.
pub struct PinSpec {
    pub name: &'static str,
    pub side: HeaderSide,
    pub index: usize,
    pub gpio: Option<u8>,
    pub is_power: bool,
    pub is_ground: bool,
}

impl PinSpec {
    /// Constructs a power supply rail pin specification.
    pub const fn power(name: &'static str, side: HeaderSide, index: usize) -> Self {
        Self {
            name,
            side,
            index,
            gpio: None,
            is_power: true,
            is_ground: false,
        }
    }

    /// Constructs a circuit ground return pin specification.
    pub const fn ground(name: &'static str, side: HeaderSide, index: usize) -> Self {
        Self {
            name,
            side,
            index,
            gpio: None,
            is_power: false,
            is_ground: true,
        }
    }

    /// Constructs an active-low reset / enable pin specification.
    pub const fn reset(name: &'static str, side: HeaderSide, index: usize) -> Self {
        Self {
            name,
            side,
            index,
            gpio: None,
            is_power: false,
            is_ground: false,
        }
    }

    /// Constructs a general purpose I/O pin specification.
    pub const fn gpio(name: &'static str, side: HeaderSide, index: usize, gpio: u8) -> Self {
        Self {
            name,
            side,
            index,
            gpio: Some(gpio),
            is_power: false,
            is_ground: false,
        }
    }
}

/// Specifications for the 22-pin Left Header (J1) of the ESP32-S3-DevKitC-1.
pub const LEFT_HEADER_SPECS: [PinSpec; 22] = [
    PinSpec::power("3V3", HeaderSide::Left, 0),
    PinSpec::power("3V3", HeaderSide::Left, 1),
    PinSpec::reset("RST", HeaderSide::Left, 2),
    PinSpec::gpio("IO4", HeaderSide::Left, 3, 4),
    PinSpec::gpio("IO5", HeaderSide::Left, 4, 5),
    PinSpec::gpio("IO6", HeaderSide::Left, 5, 6),
    PinSpec::gpio("IO7", HeaderSide::Left, 6, 7),
    PinSpec::gpio("IO15", HeaderSide::Left, 7, 15),
    PinSpec::gpio("IO16", HeaderSide::Left, 8, 16),
    PinSpec::gpio("IO17", HeaderSide::Left, 9, 17),
    PinSpec::gpio("IO18", HeaderSide::Left, 10, 18),
    PinSpec::gpio("IO8", HeaderSide::Left, 11, 8),
    PinSpec::gpio("IO3", HeaderSide::Left, 12, 3),
    PinSpec::gpio("IO46", HeaderSide::Left, 13, 46),
    PinSpec::gpio("IO9", HeaderSide::Left, 14, 9),
    PinSpec::gpio("IO10", HeaderSide::Left, 15, 10),
    PinSpec::gpio("IO11", HeaderSide::Left, 16, 11),
    PinSpec::gpio("IO12", HeaderSide::Left, 17, 12),
    PinSpec::gpio("IO13", HeaderSide::Left, 18, 13),
    PinSpec::gpio("IO14", HeaderSide::Left, 19, 14),
    PinSpec::power("5V", HeaderSide::Left, 20),
    PinSpec::ground("GND", HeaderSide::Left, 21),
];

/// Specifications for the 22-pin Right Header (J3) of the ESP32-S3-DevKitC-1.
pub const RIGHT_HEADER_SPECS: [PinSpec; 22] = [
    PinSpec::ground("GND", HeaderSide::Right, 0),
    PinSpec::gpio("TX", HeaderSide::Right, 1, 43),
    PinSpec::gpio("RX", HeaderSide::Right, 2, 44),
    PinSpec::gpio("IO1", HeaderSide::Right, 3, 1),
    PinSpec::gpio("IO2", HeaderSide::Right, 4, 2),
    PinSpec::gpio("IO42", HeaderSide::Right, 5, 42),
    PinSpec::gpio("IO41", HeaderSide::Right, 6, 41),
    PinSpec::gpio("IO40", HeaderSide::Right, 7, 40),
    PinSpec::gpio("IO39", HeaderSide::Right, 8, 39),
    PinSpec::gpio("IO38", HeaderSide::Right, 9, 38),
    PinSpec::gpio("IO37", HeaderSide::Right, 10, 37),
    PinSpec::gpio("IO36", HeaderSide::Right, 11, 36),
    PinSpec::gpio("IO35", HeaderSide::Right, 12, 35),
    PinSpec::gpio("IO0", HeaderSide::Right, 13, 0),
    PinSpec::gpio("IO45", HeaderSide::Right, 14, 45),
    PinSpec::gpio("IO48", HeaderSide::Right, 15, 48),
    PinSpec::gpio("IO47", HeaderSide::Right, 16, 47),
    PinSpec::gpio("IO21", HeaderSide::Right, 17, 21),
    PinSpec::gpio("IO20", HeaderSide::Right, 18, 20),
    PinSpec::gpio("IO19", HeaderSide::Right, 19, 19),
    PinSpec::ground("GND", HeaderSide::Right, 20),
    PinSpec::ground("GND", HeaderSide::Right, 21),
];
