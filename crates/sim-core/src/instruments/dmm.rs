use crate::netlist::PinId;

/// Measurement function mode for the digital multimeter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DmmMode {
    VoltageDc,
    VoltageAc,
    Continuity,
    Frequency,
    DutyCycle,
}

/// A Digital Multimeter model measuring DC/AC voltage, continuity, frequency, and duty cycle.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DigitalMultimeter {
    pub mode: DmmMode,
    pub probe_red: Option<PinId>,
    pub probe_black: Option<PinId>,
    pub reading: f32,
    pub continuity_active: bool,
}

impl Default for DigitalMultimeter {
    fn default() -> Self {
        Self {
            mode: DmmMode::VoltageDc,
            probe_red: None,
            probe_black: None,
            reading: 0.0,
            continuity_active: false,
        }
    }
}

impl DigitalMultimeter {
    /// Updates measurement values based on probed node voltages and connectivity.
    pub fn update(
        &mut self,
        v_red: f32,
        v_black: f32,
        nodes_connected: bool,
        measured_freq: Option<f32>,
        measured_duty: Option<f32>,
    ) {
        match self.mode {
            DmmMode::VoltageDc => {
                self.reading = v_red - v_black;
                self.continuity_active = false;
            }
            DmmMode::VoltageAc => {
                self.reading = (v_red - v_black).abs() * std::f32::consts::FRAC_1_SQRT_2;
                self.continuity_active = false;
            }
            DmmMode::Continuity => {
                self.continuity_active = nodes_connected || (v_red - v_black).abs() < 0.05;
                self.reading = if self.continuity_active { 0.0 } else { 9999.0 };
            }
            DmmMode::Frequency => {
                self.reading = measured_freq.unwrap_or(0.0);
                self.continuity_active = false;
            }
            DmmMode::DutyCycle => {
                self.reading = measured_duty.unwrap_or(0.0);
                self.continuity_active = false;
            }
        }
    }

    /// Formats the current measurement into a digital LCD display string with unit.
    pub fn display_string(&self) -> String {
        match self.mode {
            DmmMode::VoltageDc => format!("{:.3} V DC", self.reading),
            DmmMode::VoltageAc => format!("{:.3} V AC", self.reading),
            DmmMode::Continuity => {
                if self.continuity_active {
                    "BEEP (0.0 Ω)".to_string()
                } else {
                    "O.L (Open)".to_string()
                }
            }
            DmmMode::Frequency => {
                if self.reading >= 1000.0 {
                    format!("{:.2} kHz", self.reading / 1000.0)
                } else {
                    format!("{:.1} Hz", self.reading)
                }
            }
            DmmMode::DutyCycle => format!("{:.1} %", self.reading),
        }
    }
}
