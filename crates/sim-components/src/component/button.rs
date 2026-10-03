//! Tactile momentary and latching pushbutton switch model.

use sim_core::component::Component;
use sim_core::netlist::PinId;

/// Visual and physical cap color options for the tactile button.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum ButtonColor {
    #[default]
    Blue,
    Red,
    Green,
    Yellow,
    Black,
    White,
}

impl ButtonColor {
    /// Human-readable display label for the button cap color.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Blue => "Blue",
            Self::Red => "Red",
            Self::Green => "Green",
            Self::Yellow => "Yellow",
            Self::Black => "Black",
            Self::White => "White",
        }
    }

    /// All available color choices for inspector pickers.
    pub const ALL: [Self; 6] = [
        Self::Blue,
        Self::Red,
        Self::Green,
        Self::Yellow,
        Self::Black,
        Self::White,
    ];
}

/// Simulated model of a 6x6mm 4-pin momentary/latching tactile pushbutton switch.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Button {
    name: String,
    pin_1a: PinId,
    pin_1b: PinId,
    pin_2a: PinId,
    pin_2b: PinId,
    is_pressed: bool,
    is_latching: bool,
    color: ButtonColor,
}

impl Button {
    /// Constructs a new 4-pin tactile switch with the given pin terminals.
    pub fn new(pin_1a: PinId, pin_1b: PinId, pin_2a: PinId, pin_2b: PinId) -> Self {
        Self {
            name: "Push Button".to_string(),
            pin_1a,
            pin_1b,
            pin_2a,
            pin_2b,
            is_pressed: false,
            is_latching: false,
            color: ButtonColor::Blue,
        }
    }

    /// Top-left terminal lead (internally connected to pin 1b).
    pub fn pin_1a(&self) -> PinId {
        self.pin_1a
    }

    /// Top-right terminal lead (internally connected to pin 1a).
    pub fn pin_1b(&self) -> PinId {
        self.pin_1b
    }

    /// Bottom-left terminal lead (internally connected to pin 2b).
    pub fn pin_2a(&self) -> PinId {
        self.pin_2a
    }

    /// Bottom-right terminal lead (internally connected to pin 2a).
    pub fn pin_2b(&self) -> PinId {
        self.pin_2b
    }

    /// Returns whether the switch contacts are currently closed.
    pub fn is_pressed(&self) -> bool {
        self.is_pressed
    }

    /// Updates the mechanical actuation state of the button.
    pub fn set_pressed(&mut self, pressed: bool) {
        self.is_pressed = pressed;
    }

    /// Presses the button down.
    pub fn press(&mut self) {
        self.is_pressed = true;
    }

    /// Releases the button up.
    pub fn release(&mut self) {
        self.is_pressed = false;
    }

    /// Toggles the pressed state (used for clicking or latching mode).
    pub fn toggle(&mut self) {
        self.is_pressed = !self.is_pressed;
    }

    /// Returns whether the button acts as a latching toggle switch instead of momentary.
    pub fn is_latching(&self) -> bool {
        self.is_latching
    }

    /// Configures momentary versus latching actuation mode.
    pub fn set_latching(&mut self, latching: bool) {
        self.is_latching = latching;
    }

    /// Returns the active button cap color.
    pub fn color(&self) -> ButtonColor {
        self.color
    }

    /// Sets the button cap color.
    pub fn set_color(&mut self, color: ButtonColor) {
        self.color = color;
    }

    /// Returns true if contacts are bridged, allowing current to flow between terminal pairs.
    pub fn is_conductive(&self) -> bool {
        self.is_pressed
    }
}

impl Component for Button {
    /// Returns the human-readable identifier.
    fn name(&self) -> &str {
        &self.name
    }

    /// Returns all terminal pins belonging to this button.
    fn pins(&self) -> Vec<PinId> {
        vec![self.pin_1a, self.pin_1b, self.pin_2a, self.pin_2b]
    }

    /// Updates internal simulation state.
    fn update(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_button_initial_state_and_pins() {
        let btn = Button::new(PinId(1), PinId(2), PinId(3), PinId(4));
        assert_eq!(btn.pin_1a(), PinId(1));
        assert_eq!(btn.pin_1b(), PinId(2));
        assert_eq!(btn.pin_2a(), PinId(3));
        assert_eq!(btn.pin_2b(), PinId(4));
        assert_eq!(btn.pins(), vec![PinId(1), PinId(2), PinId(3), PinId(4)]);
        assert!(!btn.is_pressed());
        assert!(!btn.is_conductive());
        assert!(!btn.is_latching());
        assert_eq!(btn.color(), ButtonColor::Blue);
    }

    #[test]
    fn test_button_press_and_release() {
        let mut btn = Button::new(PinId(10), PinId(11), PinId(12), PinId(13));
        btn.press();
        assert!(btn.is_pressed());
        assert!(btn.is_conductive());

        btn.release();
        assert!(!btn.is_pressed());
        assert!(!btn.is_conductive());

        btn.set_pressed(true);
        assert!(btn.is_pressed());
        btn.set_pressed(false);
        assert!(!btn.is_pressed());
    }

    #[test]
    fn test_button_toggle() {
        let mut btn = Button::new(PinId(1), PinId(2), PinId(3), PinId(4));
        btn.toggle();
        assert!(btn.is_pressed());
        btn.toggle();
        assert!(!btn.is_pressed());
    }

    #[test]
    fn test_button_latching_and_color() {
        let mut btn = Button::new(PinId(1), PinId(2), PinId(3), PinId(4));
        btn.set_latching(true);
        assert!(btn.is_latching());

        btn.set_color(ButtonColor::Red);
        assert_eq!(btn.color(), ButtonColor::Red);
        assert_eq!(btn.color().label(), "Red");
    }
}
