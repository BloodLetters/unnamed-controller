//! Resistor electronic component model.

use sim_core::component::Component;
use sim_core::netlist::PinId;

/// Smallest resistance a simulated resistor may be set to, mirroring the nodal solver floor.
pub const RESISTOR_MIN_OHMS: f32 = 0.001;

/// Default resistance of a newly placed resistor in ohms.
pub const RESISTOR_DEFAULT_OHMS: f32 = 220.0;

/// Largest resistance a resistor may be set to, keeping the nodal matrix well conditioned.
pub const RESISTOR_MAX_OHMS: f32 = 1_000_000.0;

/// Simulated model of an axial through-hole resistor with an editable resistance value.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Resistor {
    name: String,
    terminal_a: PinId,
    terminal_b: PinId,
    resistance_ohms: f32,
}

impl Resistor {
    /// Constructs a new resistor between two terminal PinIds at the default resistance.
    pub fn new(terminal_a: PinId, terminal_b: PinId) -> Self {
        Self::with_resistance(terminal_a, terminal_b, RESISTOR_DEFAULT_OHMS)
    }

    /// Constructs a new resistor with an explicit resistance clamped to the supported range.
    pub fn with_resistance(terminal_a: PinId, terminal_b: PinId, resistance_ohms: f32) -> Self {
        Self {
            name: "Resistor".to_string(),
            terminal_a,
            terminal_b,
            resistance_ohms: clamp_resistance(resistance_ohms),
        }
    }

    /// Terminal PinId of the first resistor lead.
    pub fn terminal_a(&self) -> PinId {
        self.terminal_a
    }

    /// Terminal PinId of the second resistor lead.
    pub fn terminal_b(&self) -> PinId {
        self.terminal_b
    }

    /// Returns the configured resistance in ohms.
    pub fn resistance_ohms(&self) -> f32 {
        self.resistance_ohms
    }

    /// Sets the resistance, clamping out-of-range values to the supported span.
    pub fn set_resistance_ohms(&mut self, resistance_ohms: f32) {
        self.resistance_ohms = clamp_resistance(resistance_ohms);
    }

    /// Returns the voltage drop across the resistor for a given terminal potential difference.
    pub fn voltage_drop(&self, terminal_a_voltage: f32, terminal_b_voltage: f32) -> f32 {
        terminal_a_voltage - terminal_b_voltage
    }

    /// Returns the current flowing from terminal A to terminal B under the given potential difference.
    pub fn current_amps(&self, terminal_a_voltage: f32, terminal_b_voltage: f32) -> f32 {
        self.voltage_drop(terminal_a_voltage, terminal_b_voltage) / self.resistance_ohms
    }

    /// Dissipated power in watts under the given potential difference.
    pub fn power_watts(&self, terminal_a_voltage: f32, terminal_b_voltage: f32) -> f32 {
        let current = self.current_amps(terminal_a_voltage, terminal_b_voltage);
        current * current * self.resistance_ohms
    }
}

/// Constrains a resistance request to the span the nodal solver remains stable over.
pub fn clamp_resistance(resistance_ohms: f32) -> f32 {
    if resistance_ohms.is_nan() {
        return RESISTOR_DEFAULT_OHMS;
    }
    resistance_ohms.clamp(RESISTOR_MIN_OHMS, RESISTOR_MAX_OHMS)
}

impl Component for Resistor {
    fn name(&self) -> &str {
        &self.name
    }

    fn pins(&self) -> Vec<PinId> {
        vec![self.terminal_a, self.terminal_b]
    }

    fn update(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resistor_defaults_and_pins() {
        let resistor = Resistor::new(PinId(1), PinId(2));
        assert_eq!(resistor.terminal_a(), PinId(1));
        assert_eq!(resistor.terminal_b(), PinId(2));
        assert_eq!(resistor.pins(), vec![PinId(1), PinId(2)]);
        assert_eq!(resistor.resistance_ohms(), RESISTOR_DEFAULT_OHMS);
    }

    #[test]
    fn test_resistor_ohm_law_current() {
        let resistor = Resistor::with_resistance(PinId(1), PinId(2), 100.0);
        assert!((resistor.current_amps(5.0, 0.0) - 0.05).abs() < 1e-6);
        assert!((resistor.voltage_drop(5.0, 0.0) - 5.0).abs() < 1e-6);
        assert!((resistor.power_watts(5.0, 0.0) - 0.25).abs() < 1e-6);
    }

    #[test]
    fn test_resistor_current_is_signed_by_potential() {
        let resistor = Resistor::with_resistance(PinId(1), PinId(2), 100.0);
        assert!(resistor.current_amps(0.0, 5.0) < 0.0);
    }

    #[test]
    fn test_resistor_clamps_out_of_range_values() {
        let mut resistor = Resistor::new(PinId(1), PinId(2));
        resistor.set_resistance_ohms(-10.0);
        assert_eq!(resistor.resistance_ohms(), RESISTOR_MIN_OHMS);

        resistor.set_resistance_ohms(0.0);
        assert_eq!(resistor.resistance_ohms(), RESISTOR_MIN_OHMS);

        resistor.set_resistance_ohms(9_999_999.0);
        assert_eq!(resistor.resistance_ohms(), RESISTOR_MAX_OHMS);

        resistor.set_resistance_ohms(f32::NAN);
        assert_eq!(resistor.resistance_ohms(), RESISTOR_DEFAULT_OHMS);
    }
}
