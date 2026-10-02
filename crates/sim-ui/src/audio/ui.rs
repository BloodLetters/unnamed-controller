use egui::{Color32, Pos2, RichText, Stroke, Ui, Vec2};
use sim_core::audio::{
    RtttlMelody, RtttlPlayer, WaveformGenerator, WaveformKind, get_preset_melodies,
};

use super::mixer::AudioMixer;

/// UI state tracker for acoustic peripherals, melody player, and audio mixer.
pub struct AudioUiState {
    pub open: bool,
    pub selected_rtttl_preset: usize,
    pub rtttl_input: String,
    pub rtttl_player: Option<RtttlPlayer>,
    pub test_tone_freq: f32,
    pub test_tone_playing: bool,
    pub test_waveform_kind: WaveformKind,
    pub test_oscillator: WaveformGenerator,
}

impl Default for AudioUiState {
    fn default() -> Self {
        let presets = get_preset_melodies();
        let default_rtttl = presets.first().map(|p| p.1.to_string()).unwrap_or_default();
        Self {
            open: false,
            selected_rtttl_preset: 0,
            rtttl_input: default_rtttl,
            rtttl_player: None,
            test_tone_freq: 440.0,
            test_tone_playing: false,
            test_waveform_kind: WaveformKind::Sine,
            test_oscillator: WaveformGenerator::new(WaveformKind::Sine, 440.0),
        }
    }
}

use crate::app::SimulatorApp;

/// Renders the floating acoustic peripherals and audio streaming inspector window.
pub fn render_audio_window(ctx: &egui::Context, app: &mut SimulatorApp) {
    if !app.audio_state.open {
        return;
    }

    let mut is_open = app.audio_state.open;
    egui::Window::new("🔊 Acoustic Peripherals & Audio Scope")
        .open(&mut is_open)
        .default_size([580.0, 480.0])
        .resizable(true)
        .show(ctx, |ui| {
            render_audio_contents(ui, app);
        });
    app.audio_state.open = is_open;
}

/// Internal layout renderer for audio controls, scope, and melody jukebox.
fn render_audio_contents(ui: &mut Ui, app: &mut SimulatorApp) {
    let state = &mut app.audio_state;
    let mixer = &mut app.audio_mixer;

    ui.horizontal(|ui| {
        ui.label(RichText::new("Master Volume:").strong());
        ui.add(egui::Slider::new(&mut mixer.master_volume, 0.0..=1.0).text("Gain"));
        let mute_text = if mixer.muted {
            "🔇 Unmute"
        } else {
            "🔊 Mute"
        };
        if ui.button(mute_text).clicked() {
            mixer.muted = !mixer.muted;
        }

        let status_color = if mixer.device.is_active {
            Color32::from_rgb(76, 175, 80)
        } else {
            Color32::from_rgb(255, 179, 0)
        };
        let status_label = if mixer.device.is_active {
            format!("Stream Active ({}Hz)", mixer.sample_rate())
        } else {
            "Audio Standby".to_string()
        };
        ui.label(RichText::new(status_label).color(status_color).small());
    });

    ui.separator();
    ui.label(RichText::new("Live Audio Waveform Scope").strong());
    render_audio_scope(ui, &mixer.scope_buffer);

    ui.separator();
    ui.columns(2, |cols| {
        render_component_telemetry(&mut cols[0]);

        render_jukebox_panel(&mut cols[1], state, mixer);
    });
}

/// Draws an oscilloscope visualizer displaying live audio oscillations.
fn render_audio_scope(ui: &mut Ui, buffer: &[f32]) {
    let (rect, _) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), 70.0), egui::Sense::hover());
    let painter = ui.painter_at(rect);

    painter.rect_filled(rect, 4.0, Color32::from_rgb(18, 22, 28));
    painter.rect_stroke(rect, 4.0, Stroke::new(1.0_f32, Color32::from_gray(60)));

    let mid_y = rect.center().y;
    painter.line_segment(
        [
            Pos2::new(rect.left(), mid_y),
            Pos2::new(rect.right(), mid_y),
        ],
        Stroke::new(0.5_f32, Color32::from_rgb(40, 50, 60)),
    );

    if buffer.len() > 1 {
        let step_x = rect.width() / (buffer.len() - 1) as f32;
        let points: Vec<Pos2> = buffer
            .iter()
            .enumerate()
            .map(|(i, &sample)| {
                let x = rect.left() + i as f32 * step_x;
                let y = mid_y - sample * (rect.height() * 0.45);
                Pos2::new(x, y.clamp(rect.top() + 2.0, rect.bottom() - 2.0))
            })
            .collect();

        for pair in points.windows(2) {
            painter.line_segment(
                [pair[0], pair[1]],
                Stroke::new(1.5_f32, Color32::from_rgb(0, 230, 118)),
            );
        }
    }
}

/// Displays acoustic telemetry status.
fn render_component_telemetry(ui: &mut Ui) {
    ui.label(RichText::new("Acoustic Components").strong());
    ui.label(
        RichText::new("No buzzers or speakers on canvas.")
            .italics()
            .small(),
    );
}

/// Renders the RTTTL melody player and interactive tone synthesizer panel.
fn render_jukebox_panel(ui: &mut Ui, state: &mut AudioUiState, mixer: &mut AudioMixer) {
    ui.label(RichText::new("RTTTL Melody Jukebox").strong());
    let presets = get_preset_melodies();

    egui::ComboBox::from_id_source("rtttl_preset_combo")
        .selected_text(
            presets
                .get(state.selected_rtttl_preset)
                .map(|p| p.0)
                .unwrap_or("Select"),
        )
        .show_ui(ui, |ui| {
            for (idx, (name, rtttl)) in presets.iter().enumerate() {
                if ui
                    .selectable_value(&mut state.selected_rtttl_preset, idx, *name)
                    .clicked()
                {
                    state.rtttl_input = rtttl.to_string();
                }
            }
        });

    ui.horizontal(|ui| {
        if ui.button("▶ Play").clicked()
            && let Ok(melody) = RtttlMelody::parse(&state.rtttl_input)
        {
            let mut player = RtttlPlayer::new(melody);
            player.play();
            state.rtttl_player = Some(player);
        }
        if ui.button("⏹ Stop").clicked() {
            state.rtttl_player = None;
        }
    });

    if let Some(player) = &mut state.rtttl_player {
        let note_idx = player.current_index;
        let total_notes = player.melody.notes.len();
        ui.monospace(format!("Playing: {}/{} notes", note_idx + 1, total_notes));
        advance_rtttl_audio(player, mixer, &mut state.test_oscillator);
    }

    ui.separator();
    ui.label(RichText::new("Interactive Tone Generator").strong());
    ui.horizontal(|ui| {
        ui.label("Wave:");
        egui::ComboBox::from_id_source("tone_wave_combo")
            .selected_text(format!("{:?}", state.test_waveform_kind))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut state.test_waveform_kind, WaveformKind::Sine, "Sine");
                ui.selectable_value(
                    &mut state.test_waveform_kind,
                    WaveformKind::Square,
                    "Square",
                );
                ui.selectable_value(
                    &mut state.test_waveform_kind,
                    WaveformKind::Triangle,
                    "Triangle",
                );
                ui.selectable_value(
                    &mut state.test_waveform_kind,
                    WaveformKind::Sawtooth,
                    "Sawtooth",
                );
            });
    });

    ui.add(
        egui::Slider::new(&mut state.test_tone_freq, 50.0..=3000.0)
            .text("Hz")
            .logarithmic(true),
    );
    if ui
        .button(if state.test_tone_playing {
            "⏹ Stop Tone"
        } else {
            "▶ Test Tone"
        })
        .clicked()
    {
        state.test_tone_playing = !state.test_tone_playing;
    }

    if state.test_tone_playing {
        state.test_oscillator.kind = state.test_waveform_kind;
        state.test_oscillator.frequency = state.test_tone_freq;
        let mut block = [0.0; 128];
        state
            .test_oscillator
            .fill_buffer(&mut block, mixer.sample_rate() as f32);
        mixer.push_samples(&block);
    }
}

/// Advances RTTTL sequencer and generates tone audio samples into the mixer.
fn advance_rtttl_audio(
    player: &mut RtttlPlayer,
    mixer: &mut AudioMixer,
    osc: &mut WaveformGenerator,
) {
    let dt_ms = (128.0 / mixer.sample_rate() as f32) * 1000.0;
    let mut block = [0.0; 128];

    if let Some(freq) = player.advance_ms(dt_ms) {
        osc.kind = WaveformKind::Square;
        osc.frequency = freq;
        osc.amplitude = 0.5;
        osc.fill_buffer(&mut block, mixer.sample_rate() as f32);
    }
    mixer.push_samples(&block);
}
