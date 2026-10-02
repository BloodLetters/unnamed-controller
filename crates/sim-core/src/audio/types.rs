/// Standard audio sample representation.
pub type Sample = f32;

/// Supported periodic waveform types for audio synthesis.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WaveformKind {
    Sine,
    Square,
    Triangle,
    Sawtooth,
    Noise,
}

/// Specifications for an audio stream.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AudioFormat {
    pub sample_rate: u32,
    pub channels: u16,
}

impl Default for AudioFormat {
    /// Default standard 44.1 kHz mono audio format.
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            channels: 1,
        }
    }
}

/// A musical tone note with pitch frequency and play duration.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ToneNote {
    pub frequency: f32,
    pub duration_ms: u32,
    pub is_pause: bool,
}

impl ToneNote {
    /// Constructs a playable musical pitch note.
    pub fn note(frequency: f32, duration_ms: u32) -> Self {
        Self {
            frequency,
            duration_ms,
            is_pause: false,
        }
    }

    /// Constructs a silent pause note.
    pub fn pause(duration_ms: u32) -> Self {
        Self {
            frequency: 0.0,
            duration_ms,
            is_pause: true,
        }
    }
}
