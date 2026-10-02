/// Represents the discrete digital state of a pin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum DigitalState {
    /// Driven to high voltage (e.g., 5V or 3.3V)
    High,
    /// Driven to ground (0V)
    Low,
    /// High impedance state (disconnected or floating)
    HighZ,
}

/// Output drive capability and impedance mode of a pin.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum DriveStrength {
    /// High impedance, negligible current drive (floating).
    HighZ = 0,
    /// Resistive pull-up or pull-down (~10k-50kΩ).
    Weak = 1,
    /// Push-pull low impedance drive (<50Ω).
    Strong = 2,
}

/// Configured output driver mode for a pin terminal.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, serde::Serialize, serde::Deserialize,
)]
pub enum PinDrive {
    /// Actively driven to a specific digital level with low output impedance.
    Driven(DigitalState),
    /// Internally pulled high through a weak pull-up resistor.
    PullUp,
    /// Internally pulled low through a weak pull-down resistor.
    PullDown,
    /// High-impedance input mode without internal pull resistors.
    #[default]
    HighZ,
}

impl PinDrive {
    /// Converts the drive mode to its idle or intended digital state.
    pub fn to_digital_state(self) -> DigitalState {
        match self {
            Self::Driven(state) => state,
            Self::PullUp => DigitalState::High,
            Self::PullDown => DigitalState::Low,
            Self::HighZ => DigitalState::HighZ,
        }
    }

    /// Returns the effective electrical driving strength of this mode.
    pub fn strength(self) -> DriveStrength {
        match self {
            Self::HighZ | Self::Driven(DigitalState::HighZ) => DriveStrength::HighZ,
            Self::PullUp | Self::PullDown => DriveStrength::Weak,
            Self::Driven(_) => DriveStrength::Strong,
        }
    }
}

/// Represents the voltage level of an analog pin in volts.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AnalogVoltage(pub f32);

/// The overall logical state of a pin which can be either Digital or Analog.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum PinState {
    Digital(DigitalState),
    Analog(AnalogVoltage),
}

impl Default for PinState {
    fn default() -> Self {
        Self::Digital(DigitalState::HighZ)
    }
}
