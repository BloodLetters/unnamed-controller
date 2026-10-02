use crate::netlist::PinId;

/// Core trait that all electronic components must implement.
pub trait Component {
    /// Returns the human-readable name of this component instance.
    fn name(&self) -> &str;

    /// Returns a list of pins belonging to this component.
    fn pins(&self) -> Vec<PinId>;

    /// Called per simulation tick or when a pin state changes.
    fn update(&mut self);
}
