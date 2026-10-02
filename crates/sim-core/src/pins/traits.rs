use super::types::PinState;

/// Defines the behavior of an object that can act as a pin in a circuit.
pub trait PinBehavior {
    /// Reads the current logical state of the pin.
    fn read_state(&self) -> PinState;

    /// Updates the pin's state.
    fn write_state(&mut self, state: PinState);
}
