use crate::app::SimulatorApp;
use crate::sim::SIMULATION_TICK_SECONDS;

/// Samples acoustic peripherals and DAC channels, rendering one simulated tick of audio.
pub fn sample_audio(app: &mut SimulatorApp) {
    if !app.power_on {
        return;
    }

    let sample_rate = app.audio_mixer.sample_rate() as f32;
    let dt = SIMULATION_TICK_SECONDS;
    let block_len = (dt * sample_rate).round().max(1.0) as usize;
    let block = vec![0.0_f32; block_len];

    app.audio_mixer.push_samples(&block);
}
