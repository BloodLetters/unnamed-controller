use super::backend::AudioOutputDevice;

/// Audio mixer managing volume, mute, visualizer buffers, and streaming to device.
pub struct AudioMixer {
    pub device: AudioOutputDevice,
    pub master_volume: f32,
    pub muted: bool,
    pub scope_buffer: Vec<f32>,
    scope_index: usize,
}

impl AudioMixer {
    /// Creates a new audio mixer with default 8192 sample queue and 256-sample visualizer buffer.
    pub fn new() -> Self {
        Self {
            device: AudioOutputDevice::new(8192),
            master_volume: 0.8,
            muted: false,
            scope_buffer: vec![0.0; 256],
            scope_index: 0,
        }
    }

    /// Feeds mixed audio samples into the output device and the oscilloscope buffer.
    pub fn push_samples(&mut self, samples: &[f32]) {
        if samples.is_empty() {
            return;
        }

        let gain = if self.muted {
            0.0
        } else {
            self.master_volume.clamp(0.0, 1.0)
        };
        let scaled: Vec<f32> = samples
            .iter()
            .map(|&s| (s * gain).clamp(-1.0, 1.0))
            .collect();

        for &s in &scaled {
            self.scope_buffer[self.scope_index] = s;
            self.scope_index = (self.scope_index + 1) % self.scope_buffer.len();
        }

        self.device.write_samples(&scaled);
    }

    /// Returns the hardware sample rate in Hz.
    pub fn sample_rate(&self) -> u32 {
        self.device.sample_rate
    }
}

impl Default for AudioMixer {
    fn default() -> Self {
        Self::new()
    }
}
