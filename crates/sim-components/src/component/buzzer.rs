//! Active buzzer electronic component model.

use sim_core::component::Component;
use sim_core::netlist::PinId;

/// Minimum supply voltage required to activate the internal oscillator (3.0 V).
pub const BUZZER_MIN_OPERATING_VOLTAGE: f32 = 3.0;
/// Maximum safe continuous supply voltage (5.0 V).
pub const BUZZER_MAX_OPERATING_VOLTAGE: f32 = 5.0;
/// Typical operating current at 5 V in milliamps (30 mA).
pub const BUZZER_TYPICAL_CURRENT_MA: f32 = 30.0;
/// Typical sound frequency produced by the internal oscillator (2400 Hz).
pub const BUZZER_TYPICAL_FREQUENCY_HZ: u32 = 2400;

/// Simulated model of a 5V active buzzer module.
///
/// An active buzzer contains an internal oscillator and emits a continuous tone
/// whenever adequate supply voltage is applied across VCC and GND. No external
/// signal is required to drive the frequency.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Buzzer {
    name: String,
    vcc: PinId,
    gnd: PinId,
    is_active: bool,
}

impl Buzzer {
    /// Constructs a new active buzzer with VCC and GND terminal PinIds.
    pub fn new(vcc: PinId, gnd: PinId) -> Self {
        Self {
            name: "Active Buzzer".to_string(),
            vcc,
            gnd,
            is_active: false,
        }
    }

    /// Power supply positive terminal PinId (VCC).
    pub fn vcc(&self) -> PinId {
        self.vcc
    }

    /// Ground reference terminal PinId (GND).
    pub fn gnd(&self) -> PinId {
        self.gnd
    }

    /// Returns true if the buzzer is emitting a tone.
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    /// Updates the activation state from supply terminal voltages.
    pub fn update_electrical(&mut self, v_vcc: f32, v_gnd: f32) {
        let supply = v_vcc - v_gnd;
        self.is_active = supply >= BUZZER_MIN_OPERATING_VOLTAGE;
    }
}

impl Component for Buzzer {
    fn name(&self) -> &str {
        &self.name
    }

    fn pins(&self) -> Vec<PinId> {
        vec![self.vcc, self.gnd]
    }

    fn update(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buzzer_activates_above_threshold() {
        let mut b = Buzzer::new(PinId(1), PinId(2));
        assert!(!b.is_active());

        b.update_electrical(5.0, 0.0);
        assert!(b.is_active());
    }

    #[test]
    fn test_buzzer_silent_below_threshold() {
        let mut b = Buzzer::new(PinId(1), PinId(2));
        b.update_electrical(2.9, 0.0);
        assert!(!b.is_active());
    }

    #[test]
    fn test_buzzer_silent_when_reversed() {
        let mut b = Buzzer::new(PinId(1), PinId(2));
        b.update_electrical(0.0, 5.0);
        assert!(!b.is_active());
    }
}
