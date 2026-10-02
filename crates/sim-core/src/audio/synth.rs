use super::types::WaveformKind;
use std::f32::consts::PI;

/// Configurable periodic waveform oscillator.
#[derive(Clone, Debug)]
pub struct WaveformGenerator {
    pub kind: WaveformKind,
    pub frequency: f32,
    pub amplitude: f32,
    pub duty_cycle: f32,
    phase: f32,
    noise_seed: u32,
}

impl WaveformGenerator {
    /// Creates a new waveform oscillator with specified kind and frequency.
    pub fn new(kind: WaveformKind, frequency: f32) -> Self {
        Self {
            kind,
            frequency,
            amplitude: 1.0,
            duty_cycle: 0.5,
            phase: 0.0,
            noise_seed: 1234567,
        }
    }

    /// Advances the oscillator phase by one sample step and returns the normalized sample.
    pub fn next_sample(&mut self, sample_rate: f32) -> f32 {
        if self.frequency <= 0.0 || sample_rate <= 0.0 {
            return 0.0;
        }

        let raw = match self.kind {
            WaveformKind::Sine => (self.phase * 2.0 * PI).sin(),
            WaveformKind::Square => {
                if self.phase < self.duty_cycle {
                    1.0
                } else {
                    -1.0
                }
            }
            WaveformKind::Triangle => {
                if self.phase < 0.5 {
                    4.0 * self.phase - 1.0
                } else {
                    3.0 - 4.0 * self.phase
                }
            }
            WaveformKind::Sawtooth => 2.0 * self.phase - 1.0,
            WaveformKind::Noise => {
                self.noise_seed = self
                    .noise_seed
                    .wrapping_mul(1664525)
                    .wrapping_add(1013904223);
                ((self.noise_seed >> 16) as f32 / 32767.5) - 1.0
            }
        };

        let phase_increment = self.frequency / sample_rate;
        self.phase = (self.phase + phase_increment).fract();

        (raw * self.amplitude).clamp(-1.0, 1.0)
    }

    /// Fills an output buffer with sequential synthesized audio samples.
    pub fn fill_buffer(&mut self, buffer: &mut [f32], sample_rate: f32) {
        for sample in buffer.iter_mut() {
            *sample = self.next_sample(sample_rate);
        }
    }

    /// Resets the phase accumulator of the generator.
    pub fn reset_phase(&mut self) {
        self.phase = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sine_waveform_synthesis() {
        let mut generator = WaveformGenerator::new(WaveformKind::Sine, 1000.0);
        let mut samples = [0.0; 4];
        generator.fill_buffer(&mut samples, 4000.0);

        assert!((samples[0] - 0.0).abs() < 1e-4);
        assert!((samples[1] - 1.0).abs() < 1e-4);
        assert!((samples[2] - 0.0).abs() < 1e-4);
        assert!((samples[3] - (-1.0)).abs() < 1e-4);
    }

    #[test]
    fn test_square_waveform_synthesis() {
        let mut generator = WaveformGenerator::new(WaveformKind::Square, 1000.0);
        assert_eq!(generator.next_sample(4000.0), 1.0);
        assert_eq!(generator.next_sample(4000.0), 1.0);
        assert_eq!(generator.next_sample(4000.0), -1.0);
        assert_eq!(generator.next_sample(4000.0), -1.0);
    }
}
