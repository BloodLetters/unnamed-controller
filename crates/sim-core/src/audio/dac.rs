use std::f32::consts::PI;

/// Operational mode for an integrated microcontroller DAC channel.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DacMode {
    DirectVoltage,
    CosineGenerator,
}

/// State tracking for a single digital-to-analog converter channel.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DacChannel {
    pub enabled: bool,
    pub mode: DacMode,
    pub value: u8,
    pub frequency: f32,
    pub amplitude_scale: f32,
    pub phase: f32,
    pub v_ref: f32,
}

impl DacChannel {
    /// Creates a default DAC channel with specified reference voltage.
    pub fn new(v_ref: f32) -> Self {
        Self {
            enabled: false,
            mode: DacMode::DirectVoltage,
            value: 0,
            frequency: 1000.0,
            amplitude_scale: 1.0,
            phase: 0.0,
            v_ref,
        }
    }

    /// Calculates the instantaneous output voltage produced by the DAC channel.
    pub fn output_voltage(&self) -> f32 {
        if !self.enabled {
            return 0.0;
        }

        match self.mode {
            DacMode::DirectVoltage => (self.value as f32 / 255.0) * self.v_ref,
            DacMode::CosineGenerator => {
                let norm = (self.phase * 2.0 * PI).cos() * 0.5 + 0.5;
                norm * self.v_ref * self.amplitude_scale
            }
        }
    }

    /// Advances the cosine generator phase by delta time in seconds.
    pub fn step(&mut self, dt_seconds: f32) {
        if self.enabled && self.mode == DacMode::CosineGenerator {
            self.phase = (self.phase + self.frequency * dt_seconds).fract();
        }
    }

    /// Extracts an audio sample in the range [-1.0, 1.0] from current DAC state.
    pub fn audio_sample(&self) -> f32 {
        if !self.enabled || self.v_ref.abs() < 1e-9 {
            return 0.0;
        }
        let v = self.output_voltage();
        (v / self.v_ref) * 2.0 - 1.0
    }
}

/// Dual-channel DAC peripheral modeled after MCU architectures.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DacPeripheral {
    pub channel_1: DacChannel,
    pub channel_2: DacChannel,
}

impl DacPeripheral {
    /// Creates a dual-channel DAC peripheral with 3.3V reference.
    pub fn new(v_ref: f32) -> Self {
        Self {
            channel_1: DacChannel::new(v_ref),
            channel_2: DacChannel::new(v_ref),
        }
    }

    /// Writes raw 8-bit digital output value to specified DAC channel (1 or 2).
    pub fn write_channel(&mut self, channel: u8, value: u8) {
        match channel {
            1 => {
                self.channel_1.enabled = true;
                self.channel_1.mode = DacMode::DirectVoltage;
                self.channel_1.value = value;
            }
            2 => {
                self.channel_2.enabled = true;
                self.channel_2.mode = DacMode::DirectVoltage;
                self.channel_2.value = value;
            }
            _ => {}
        }
    }

    /// Configures the hardware cosine tone generator on specified DAC channel.
    pub fn configure_cosine(&mut self, channel: u8, frequency: f32, amplitude: f32) {
        let ch = match channel {
            1 => &mut self.channel_1,
            2 => &mut self.channel_2,
            _ => return,
        };
        ch.enabled = true;
        ch.mode = DacMode::CosineGenerator;
        ch.frequency = frequency;
        ch.amplitude_scale = amplitude.clamp(0.0, 1.0);
    }

    /// Steps both DAC channels forward in time.
    pub fn step(&mut self, dt_seconds: f32) {
        self.channel_1.step(dt_seconds);
        self.channel_2.step(dt_seconds);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dac_direct_voltage() {
        let mut dac = DacPeripheral::new(3.3);
        dac.write_channel(1, 128);
        let v = dac.channel_1.output_voltage();
        assert!((v - 1.656).abs() < 0.05);

        let sample = dac.channel_1.audio_sample();
        assert!(sample.abs() < 0.05);
    }

    #[test]
    fn test_dac_cosine_generator() {
        let mut dac = DacPeripheral::new(3.3);
        dac.configure_cosine(2, 440.0, 1.0);
        assert_eq!(dac.channel_2.mode, DacMode::CosineGenerator);
        assert!(dac.channel_2.enabled);

        dac.step(0.001);
        assert!(dac.channel_2.phase > 0.0);
    }
}
