//! Light Emitting Diode (LED) electronic component model.

use sim_core::component::Component;
use sim_core::netlist::PinId;

/// Available color variants for standard 5mm through-hole LEDs.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, serde::Serialize, serde::Deserialize,
)]
pub enum LedColor {
    #[default]
    Red,
    Green,
    Blue,
    Yellow,
    Orange,
    White,
}

impl LedColor {
    /// Human-readable name of the color variant.
    pub fn name(self) -> &'static str {
        match self {
            Self::Red => "Red",
            Self::Green => "Green",
            Self::Blue => "Blue",
            Self::Yellow => "Yellow",
            Self::Orange => "Orange",
            Self::White => "White",
        }
    }

    /// Typical forward voltage drop in Volts (Vf) required for emission.
    pub fn forward_voltage(self) -> f32 {
        match self {
            Self::Red => 1.8,
            Self::Green => 2.1,
            Self::Blue => 3.0,
            Self::Yellow => 2.0,
            Self::Orange => 2.0,
            Self::White => 3.0,
        }
    }

    /// Primary RGB representation of the illuminated emitter lens.
    pub fn rgb(self) -> [u8; 3] {
        match self {
            Self::Red => [244, 67, 54],
            Self::Green => [76, 175, 80],
            Self::Blue => [33, 150, 243],
            Self::Yellow => [255, 235, 59],
            Self::Orange => [255, 152, 0],
            Self::White => [245, 245, 250],
        }
    }

    /// Dim non-illuminated RGB representation of the unpowered lens epoxy.
    pub fn off_rgb(self) -> [u8; 3] {
        match self {
            Self::Red => [80, 25, 25],
            Self::Green => [25, 60, 25],
            Self::Blue => [25, 40, 75],
            Self::Yellow => [75, 70, 25],
            Self::Orange => [80, 45, 20],
            Self::White => [55, 55, 60],
        }
    }

    /// List of all supported LED color variants.
    pub const ALL: [LedColor; 6] = [
        Self::Red,
        Self::Green,
        Self::Blue,
        Self::Yellow,
        Self::Orange,
        Self::White,
    ];
}

/// Simulated model of a discrete 5mm through-hole LED.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Led {
    name: String,
    anode: PinId,
    cathode: PinId,
    pub color: LedColor,
    is_lit: bool,
    brightness: f32,
}

impl Led {
    /// Constructs a new LED with anode and cathode terminal PinIds and a color preset.
    pub fn new(anode: PinId, cathode: PinId, color: LedColor) -> Self {
        Self {
            name: format!("{} LED", color.name()),
            anode,
            cathode,
            color,
            is_lit: false,
            brightness: 0.0,
        }
    }

    /// Terminal PinId of the positive anode terminal (+).
    pub fn anode(&self) -> PinId {
        self.anode
    }

    /// Terminal PinId of the negative cathode terminal (-).
    pub fn cathode(&self) -> PinId {
        self.cathode
    }

    /// Returns true if current flows through the forward-biased diode.
    pub fn is_lit(&self) -> bool {
        self.is_lit
    }

    /// Normalized illumination intensity from 0.0 (off) to 1.0 (full brightness).
    pub fn brightness(&self) -> f32 {
        self.brightness
    }

    /// Swaps the physical polarity of the anode and cathode pins.
    pub fn flip_polarity(&mut self) {
        std::mem::swap(&mut self.anode, &mut self.cathode);
    }

    /// Updates the conduction state based on the potential difference across terminals.
    pub fn update_electrical(&mut self, anode_voltage: f32, cathode_voltage: f32) {
        let v_diff = anode_voltage - cathode_voltage;
        let v_f = self.color.forward_voltage();

        if v_diff >= v_f {
            self.is_lit = true;
            self.brightness = ((v_diff - v_f) / 0.5 + 0.3).clamp(0.2, 1.0);
        } else {
            self.is_lit = false;
            self.brightness = 0.0;
        }
    }
}

impl Component for Led {
    fn name(&self) -> &str {
        &self.name
    }

    fn pins(&self) -> Vec<PinId> {
        vec![self.anode, self.cathode]
    }

    fn update(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_led_forward_bias_illumination() {
        let mut led = Led::new(PinId(1), PinId(2), LedColor::Red);
        assert!(!led.is_lit());

        led.update_electrical(5.0, 0.0);
        assert!(led.is_lit());
        assert!(led.brightness() > 0.0);

        led.update_electrical(0.0, 5.0);
        assert!(!led.is_lit());
        assert_eq!(led.brightness(), 0.0);
    }

    #[test]
    fn test_led_polarity_flip() {
        let mut led = Led::new(PinId(10), PinId(20), LedColor::Green);
        assert_eq!(led.anode(), PinId(10));
        assert_eq!(led.cathode(), PinId(20));

        led.flip_polarity();
        assert_eq!(led.anode(), PinId(20));
        assert_eq!(led.cathode(), PinId(10));
    }
}
