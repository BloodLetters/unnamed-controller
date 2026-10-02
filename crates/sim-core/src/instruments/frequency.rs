use crate::netlist::PinId;

/// Frequency counter and PWM duty cycle measurement instrument.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PwmAnalyzer {
    pub pin: Option<PinId>,
    pub frequency_hz: f32,
    pub duty_percent: f32,
    pub high_duration_us: u64,
    pub low_duration_us: u64,
    pub last_state: bool,
    pub last_transition_us: u64,
}

impl Default for PwmAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl PwmAnalyzer {
    /// Creates a new unassigned PWM and frequency counter analyzer.
    pub fn new() -> Self {
        Self {
            pin: None,
            frequency_hz: 0.0,
            duty_percent: 0.0,
            high_duration_us: 0,
            low_duration_us: 0,
            last_state: false,
            last_transition_us: 0,
        }
    }

    /// Evaluates edge transitions and computes real-time frequency and duty cycle.
    pub fn update(&mut self, time_us: u64, current_state: bool) {
        if current_state != self.last_state {
            let duration = time_us.saturating_sub(self.last_transition_us);
            if self.last_state {
                self.high_duration_us = duration;
            } else {
                self.low_duration_us = duration;
            }

            let period_us = self.high_duration_us + self.low_duration_us;
            if period_us > 0 {
                self.frequency_hz = 1_000_000.0 / period_us as f32;
                self.duty_percent = (self.high_duration_us as f32 / period_us as f32) * 100.0;
            }

            self.last_state = current_state;
            self.last_transition_us = time_us;
        }
    }

    /// Clears accumulated transition timing data.
    pub fn reset(&mut self) {
        self.frequency_hz = 0.0;
        self.duty_percent = 0.0;
        self.high_duration_us = 0;
        self.low_duration_us = 0;
        self.last_transition_us = 0;
    }
}
