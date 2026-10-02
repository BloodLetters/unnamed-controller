use std::collections::VecDeque;

/// A single discrete signal sample captured at a point in simulation time.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SignalSample {
    pub time_us: u64,
    pub voltage: f32,
    pub digital: bool,
}

impl SignalSample {
    /// Creates a new signal sample with analog and digital representations.
    pub fn new(time_us: u64, voltage: f32, digital: bool) -> Self {
        Self {
            time_us,
            voltage,
            digital,
        }
    }
}

/// Circular buffer storing recent signal samples for waveform plotting.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TraceBuffer {
    pub samples: VecDeque<SignalSample>,
    pub max_samples: usize,
}

impl Default for TraceBuffer {
    fn default() -> Self {
        Self::new(600)
    }
}

impl TraceBuffer {
    /// Creates a new trace buffer with the specified maximum sample capacity.
    pub fn new(max_samples: usize) -> Self {
        Self {
            samples: VecDeque::with_capacity(max_samples),
            max_samples,
        }
    }

    /// Appends a new sample to the buffer, popping the oldest sample if at capacity.
    pub fn push(&mut self, time_us: u64, voltage: f32, digital: bool) {
        if self.samples.len() >= self.max_samples {
            self.samples.pop_front();
        }
        self.samples
            .push_back(SignalSample::new(time_us, voltage, digital));
    }

    /// Clears all stored samples in the trace buffer.
    pub fn clear(&mut self) {
        self.samples.clear();
    }

    /// Retrieves the most recently recorded sample, if available.
    pub fn latest(&self) -> Option<&SignalSample> {
        self.samples.back()
    }

    /// Computes peak-to-peak voltage ($V_{max} - V_{min}$) across recorded samples.
    pub fn peak_to_peak(&self) -> f32 {
        if self.samples.is_empty() {
            return 0.0;
        }
        let mut min = f32::INFINITY;
        let mut max = f32::NEG_INFINITY;
        for s in &self.samples {
            if s.voltage < min {
                min = s.voltage;
            }
            if s.voltage > max {
                max = s.voltage;
            }
        }
        (max - min).max(0.0)
    }

    /// Computes the midpoint voltage ((V_max + V_min) / 2) across recorded samples.
    pub fn midpoint(&self) -> f32 {
        if self.samples.is_empty() {
            return 0.0;
        }
        let mut min = f32::INFINITY;
        let mut max = f32::NEG_INFINITY;
        for s in &self.samples {
            min = min.min(s.voltage);
            max = max.max(s.voltage);
        }
        (max + min) / 2.0
    }

    /// Calculates RMS voltage across recorded analog samples.
    pub fn rms_voltage(&self) -> f32 {
        if self.samples.is_empty() {
            return 0.0;
        }
        let sum_sq: f32 = self.samples.iter().map(|s| s.voltage * s.voltage).sum();
        (sum_sq / self.samples.len() as f32).sqrt()
    }
}
