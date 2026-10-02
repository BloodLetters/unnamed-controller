use crate::netlist::PinId;
use std::collections::VecDeque;

/// A single digital logic recording channel.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LogicChannel {
    pub name: String,
    pub pin: Option<PinId>,
    pub history: VecDeque<(u64, bool)>,
}

impl LogicChannel {
    /// Creates a new logic channel with default name and pin mapping.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            pin: None,
            history: VecDeque::with_capacity(600),
        }
    }
}

/// An 8-channel digital logic analyzer with waveform capture and VCD export.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LogicAnalyzer {
    pub channels: Vec<LogicChannel>,
    pub max_samples: usize,
    pub paused: bool,
}

impl Default for LogicAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl LogicAnalyzer {
    /// Creates an 8-channel logic analyzer with standard D0 through D7 labels.
    pub fn new() -> Self {
        let channels = (0..8)
            .map(|i| LogicChannel::new(format!("D{}", i)))
            .collect();
        Self {
            channels,
            max_samples: 500,
            paused: false,
        }
    }

    /// Records current digital states for each channel at the given simulation timestamp.
    pub fn sample(&mut self, time_us: u64, states: &[bool; 8]) {
        if self.paused {
            return;
        }

        for (i, &st) in states.iter().enumerate() {
            if let Some(ch) = self.channels.get_mut(i) {
                let should_push = match ch.history.back() {
                    Some(&(_, last_val)) => last_val != st,
                    None => true,
                };

                if should_push {
                    if ch.history.len() >= self.max_samples {
                        ch.history.pop_front();
                    }
                    ch.history.push_back((time_us, st));
                }
            }
        }
    }

    /// Clears all recorded transition history across all channels.
    pub fn clear(&mut self) {
        for ch in &mut self.channels {
            ch.history.clear();
        }
    }

    /// Generates standard Value Change Dump (VCD) text suitable for GTKWave or PulseView.
    pub fn export_vcd(&self) -> String {
        let mut out = String::new();
        out.push_str("$version Unnamed Controller Logic Analyzer $end\n");
        out.push_str("$timescale 1us $end\n");
        out.push_str("$scope module logic_analyzer $end\n");

        let id_chars = ['!', '"', '#', '$', '%', '&', '\'', '('];
        for (i, ch) in self.channels.iter().enumerate() {
            out.push_str(&format!("$var wire 1 {} {} $end\n", id_chars[i], ch.name));
        }
        out.push_str("$upscope $end\n");
        out.push_str("$enddefinitions $end\n");

        let mut events: Vec<(u64, usize, bool)> = Vec::new();
        for (ch_idx, ch) in self.channels.iter().enumerate() {
            for &(time, state) in &ch.history {
                events.push((time, ch_idx, state));
            }
        }
        events.sort_by_key(|e| e.0);

        let mut last_time: Option<u64> = None;
        for (time, ch_idx, state) in events {
            if last_time != Some(time) {
                out.push_str(&format!("#{}\n", time));
                last_time = Some(time);
            }
            let bit_char = if state { '1' } else { '0' };
            out.push_str(&format!("{}{}\n", bit_char, id_chars[ch_idx]));
        }

        out
    }
}
