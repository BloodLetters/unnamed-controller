use sim_core::netlist::PinId;
use sim_core::pins::{DigitalState, PinDrive};

use super::types::{BoardDimensions, BoardType, FirmwareError, HeaderPin};

/// Interface defining the behavior and physical properties of a simulated development board.
pub trait Board {
    /// Human-readable board model name (e.g., "ESP32-S3 DevKitC-1").
    fn name(&self) -> &str;

    /// Architecture and board variant tag.
    fn board_type(&self) -> BoardType;

    /// Returns a list of all header terminal PinIds on this board.
    fn pins(&self) -> Vec<PinId>;

    /// Returns the physical and electrical definition of all header pins.
    fn header_pins(&self) -> &[HeaderPin];

    /// Queries a pin by its terminal identifier.
    fn pin_name(&self, pin: PinId) -> Option<&str>;

    /// Queries the pin assigned to a specific GPIO number.
    fn gpio_pin(&self, gpio: u8) -> Option<PinId>;

    /// Resolves the power supply terminal and all ground terminals.
    fn power_terminals(&self) -> (Option<PinId>, Vec<PinId>);

    /// Returns physical dimensions for canvas layout and schematic rendering.
    fn dimensions(&self) -> BoardDimensions;

    /// Returns true if the board is actively powered.
    fn is_powered(&self) -> bool;

    /// Controls the electrical power state of the board.
    fn set_powered(&mut self, powered: bool);

    /// Reboots the onboard microcontroller and peripherals to their initial state.
    fn reset(&mut self);

    /// Advances the board's internal simulation clock and hardware peripherals.
    fn step(&mut self, dt_seconds: f32);

    /// Flashes compiled firmware into the board's onboard memory.
    fn load_firmware(&mut self, binary: &[u8]) -> Result<(), FirmwareError>;

    /// Sends text input to the onboard UART / serial interface.
    fn send_serial_input(&mut self, text: &str);

    /// Reads output accumulated in the serial monitor buffer.
    fn serial_output(&self) -> &str;

    /// Clears the serial monitor output buffer.
    fn clear_serial_output(&mut self);

    /// Returns the active output driver mode for a terminal.
    fn pin_drive(&self, pin: PinId) -> PinDrive;

    /// Returns the fixed potential of a power rail terminal in volts, or None when the terminal is not a rail.
    fn rail_voltage(&self, pin: PinId) -> Option<f32>;

    /// Notifies the board that an external netlist signal changed state.
    fn notify_pin_change(&mut self, pin: PinId, state: DigitalState);
}
