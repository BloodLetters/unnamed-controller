use super::trace::TraceBuffer;
use crate::netlist::PinId;

/// Trigger capture operating mode for the oscilloscope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TriggerMode {
    Auto,
    Normal,
    Single,
}

/// Trigger edge detection condition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TriggerEdge {
    Rising,
    Falling,
}

/// Trigger source channel selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TriggerSource {
    Ch1,
    Ch2,
}

/// An individual oscilloscope input channel with scaling and trace storage.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OscilloscopeChannel {
    pub pin: Option<PinId>,
    pub volts_per_div: f32,
    pub vertical_offset: f32,
    pub buffer: TraceBuffer,
    pub enabled: bool,
}

impl Default for OscilloscopeChannel {
    fn default() -> Self {
        Self {
            pin: None,
            volts_per_div: 1.0,
            vertical_offset: 0.0,
            buffer: TraceBuffer::new(500),
            enabled: true,
        }
    }
}

/// A 2-channel Virtual Oscilloscope with adjustable timebase, scaling, and triggering.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VirtualOscilloscope {
    pub ch1: OscilloscopeChannel,
    pub ch2: OscilloscopeChannel,
    pub timebase_ms_per_div: f32,
    pub trigger_mode: TriggerMode,
    pub trigger_edge: TriggerEdge,
    pub trigger_source: TriggerSource,
    pub trigger_level_volts: f32,
    pub triggered: bool,
    pub paused: bool,
    pub last_trigger_val: f32,
}

impl Default for VirtualOscilloscope {
    fn default() -> Self {
        Self {
            ch1: OscilloscopeChannel::default(),
            ch2: OscilloscopeChannel::default(),
            timebase_ms_per_div: 5.0,
            trigger_mode: TriggerMode::Auto,
            trigger_edge: TriggerEdge::Rising,
            trigger_source: TriggerSource::Ch1,
            trigger_level_volts: 2.5,
            triggered: false,
            paused: false,
            last_trigger_val: 0.0,
        }
    }
}

impl VirtualOscilloscope {
    /// Ingests signal voltages for CH1 and CH2 and evaluates trigger conditions.
    pub fn sample(&mut self, time_us: u64, ch1_volt: Option<f32>, ch2_volt: Option<f32>) {
        if self.paused {
            return;
        }

        let v1 = ch1_volt.unwrap_or(0.0);
        let v2 = ch2_volt.unwrap_or(0.0);

        if self.ch1.enabled && ch1_volt.is_some() {
            self.ch1.buffer.push(time_us, v1, v1 >= 2.5);
        }
        if self.ch2.enabled && ch2_volt.is_some() {
            self.ch2.buffer.push(time_us, v2, v2 >= 2.5);
        }

        let trigger_input = match self.trigger_source {
            TriggerSource::Ch1 => v1,
            TriggerSource::Ch2 => v2,
        };

        self.evaluate_trigger(trigger_input);
    }

    /// Evaluates edge threshold crossing against the trigger level.
    fn evaluate_trigger(&mut self, current_val: f32) {
        let level = self.trigger_level_volts;
        let edge_detected = match self.trigger_edge {
            TriggerEdge::Rising => self.last_trigger_val < level && current_val >= level,
            TriggerEdge::Falling => self.last_trigger_val > level && current_val <= level,
        };

        if edge_detected {
            self.triggered = true;
            if self.trigger_mode == TriggerMode::Single {
                self.paused = true;
            }
        }

        self.last_trigger_val = current_val;
    }

    /// Calculates fundamental signal frequency in Hertz based on periodic threshold crossings.
    pub fn calculate_frequency(&self, ch: usize) -> Option<f32> {
        let buf = if ch == 1 {
            &self.ch1.buffer
        } else {
            &self.ch2.buffer
        };

        if buf.samples.len() < 10 {
            return None;
        }

        let mut crossings = Vec::new();
        let mid = buf.midpoint();

        for i in 1..buf.samples.len() {
            let prev = buf.samples[i - 1];
            let curr = buf.samples[i];
            if prev.voltage < mid && curr.voltage >= mid {
                crossings.push(curr.time_us);
            }
        }

        if crossings.len() < 2 {
            return None;
        }

        let last_idx = crossings.len() - 1;
        let total_dt_us = crossings[last_idx] - crossings[0];
        let periods = (crossings.len() - 1) as f32;
        let avg_period_us = total_dt_us as f32 / periods;

        if avg_period_us <= 0.0 {
            None
        } else {
            Some(1_000_000.0 / avg_period_us)
        }
    }
}
