use sim_core::netlist::PinId;
use std::fmt;

/// Identifies the specific model and architecture of a development board.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum BoardType {
    Esp32S3DevKit,
}

/// Identifies which physical edge of the PCB a header terminal belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum HeaderSide {
    Left,
    Right,
    Top,
    Bottom,
}

/// Metadata and electrical identification for a single board header terminal.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HeaderPin {
    pub pin_id: PinId,
    pub name: String,
    pub side: HeaderSide,
    pub index: usize,
    pub gpio: Option<u8>,
    pub is_power: bool,
    pub is_ground: bool,
}

impl HeaderPin {
    /// Constructs a new header pin definition.
    pub fn new(
        pin_id: PinId,
        name: impl Into<String>,
        side: HeaderSide,
        index: usize,
        gpio: Option<u8>,
        is_power: bool,
        is_ground: bool,
    ) -> Self {
        Self {
            pin_id,
            name: name.into(),
            side,
            index,
            gpio,
            is_power,
            is_ground,
        }
    }
}

/// Physical PCB dimensions and terminal spacing for schematic representation.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BoardDimensions {
    pub width: f32,
    pub height: f32,
    pub pin_pitch: f32,
}

impl Default for BoardDimensions {
    fn default() -> Self {
        Self {
            width: 140.0,
            height: 300.0,
            pin_pitch: 12.0,
        }
    }
}

/// Color state of an onboard addressable RGB status indicator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct BoardRgbLed {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub enabled: bool,
}

impl BoardRgbLed {
    /// Constructs a new RGB LED state.
    pub fn new(r: u8, g: u8, b: u8, enabled: bool) -> Self {
        Self { r, g, b, enabled }
    }

    /// Turns off the RGB indicator.
    pub fn turn_off(&mut self) {
        self.enabled = false;
    }

    /// Sets the color and enables the indicator.
    pub fn set_rgb(&mut self, r: u8, g: u8, b: u8) {
        self.r = r;
        self.g = g;
        self.b = b;
        self.enabled = true;
    }
}

/// Errors occurring during firmware flashing or binary loading.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FirmwareError {
    EmptyBinary,
    BinaryTooLarge { size: usize, max: usize },
    InvalidFormat(String),
    FlashFailed(String),
}

impl fmt::Display for FirmwareError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyBinary => write!(f, "Firmware binary is empty"),
            Self::BinaryTooLarge { size, max } => {
                write!(f, "Firmware size ({} B) exceeds limit ({} B)", size, max)
            }
            Self::InvalidFormat(msg) => write!(f, "Invalid firmware format: {}", msg),
            Self::FlashFailed(msg) => write!(f, "Flashing failed: {}", msg),
        }
    }
}

impl std::error::Error for FirmwareError {}
