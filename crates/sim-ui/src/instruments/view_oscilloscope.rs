use egui::{Color32, FontId, Painter, Pos2, Rect, Stroke, Ui, Vec2};
use sim_core::instruments::{TriggerEdge, TriggerMode, TriggerSource};
use sim_core::netlist::PinId;

use crate::app::SimulatorApp;

/// Renders the 2-Channel Oscilloscope screen and interactive instrument control dials.
pub fn render_oscilloscope_view(ui: &mut Ui, app: &mut SimulatorApp) {
    ui.horizontal(|ui| {
        render_channel_controls(ui, app);
    });

    ui.separator();

    let avail_width = ui.available_width();
    let screen_height = 240.0;
    let (response, painter) =
        ui.allocate_painter(Vec2::new(avail_width, screen_height), egui::Sense::hover());
    let rect = response.rect;

    draw_oscilloscope_screen(&painter, rect, app);
    draw_hud(&painter, rect, app);
}

/// Renders controls for channel scales, timebase, trigger settings, and probe pin routing.
fn render_channel_controls(ui: &mut Ui, app: &mut SimulatorApp) {
    let osc = &mut app.instruments.oscilloscope;

    ui.group(|ui| {
        ui.label(
            egui::RichText::new("CH1 (Yellow)")
                .color(Color32::from_rgb(255, 220, 0))
                .strong(),
        );
        ui.checkbox(&mut osc.ch1.enabled, "Enable");
        ui.add(egui::Slider::new(&mut osc.ch1.volts_per_div, 0.5..=10.0).text("V/Div"));
        let mut pin_val = osc.ch1.pin.map(|p| p.0).unwrap_or(0);
        if ui
            .add(egui::DragValue::new(&mut pin_val).prefix("Pin: "))
            .changed()
        {
            osc.ch1.pin = Some(PinId(pin_val));
        }
    });

    ui.group(|ui| {
        ui.label(
            egui::RichText::new("CH2 (Cyan)")
                .color(Color32::from_rgb(0, 230, 255))
                .strong(),
        );
        ui.checkbox(&mut osc.ch2.enabled, "Enable");
        ui.add(egui::Slider::new(&mut osc.ch2.volts_per_div, 0.5..=10.0).text("V/Div"));
        let mut pin_val = osc.ch2.pin.map(|p| p.0).unwrap_or(0);
        if ui
            .add(egui::DragValue::new(&mut pin_val).prefix("Pin: "))
            .changed()
        {
            osc.ch2.pin = Some(PinId(pin_val));
        }
    });

    ui.group(|ui| {
        ui.label(egui::RichText::new("Timebase & Trigger").strong());
        ui.add(egui::Slider::new(&mut osc.timebase_ms_per_div, 1.0..=50.0).text("ms/Div"));
        ui.horizontal(|ui| {
            ui.selectable_value(&mut osc.trigger_mode, TriggerMode::Auto, "Auto");
            ui.selectable_value(&mut osc.trigger_mode, TriggerMode::Normal, "Norm");
            ui.selectable_value(&mut osc.trigger_mode, TriggerMode::Single, "Single");
        });
        ui.horizontal(|ui| {
            ui.selectable_value(&mut osc.trigger_source, TriggerSource::Ch1, "Src CH1");
            ui.selectable_value(&mut osc.trigger_source, TriggerSource::Ch2, "Src CH2");
            ui.selectable_value(&mut osc.trigger_edge, TriggerEdge::Rising, "↑ Edge");
            ui.selectable_value(&mut osc.trigger_edge, TriggerEdge::Falling, "↓ Edge");
        });
        ui.add(egui::Slider::new(&mut osc.trigger_level_volts, 0.0..=5.0).text("Trig V"));
        if ui
            .button(if osc.paused {
                "▶ Resume"
            } else {
                "⏸ Pause"
            })
            .clicked()
        {
            osc.paused = !osc.paused;
        }
    });
}

/// Draws the cathode-ray tube styled dark display and graticule grid.
fn draw_oscilloscope_screen(painter: &Painter, rect: Rect, app: &SimulatorApp) {
    painter.rect_filled(rect, 4.0, Color32::from_rgb(12, 18, 14));
    painter.rect_stroke(
        rect,
        4.0,
        Stroke::new(1.5_f32, Color32::from_rgb(40, 70, 50)),
    );

    let grid_stroke = Stroke::new(0.5_f32, Color32::from_rgba_unmultiplied(60, 110, 80, 70));
    let center_stroke = Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(70, 130, 90, 140));

    for i in 1..8 {
        let x = rect.left() + rect.width() * (i as f32 / 8.0);
        let s = if i == 4 { center_stroke } else { grid_stroke };
        painter.line_segment([Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())], s);
    }

    for i in 1..6 {
        let y = rect.top() + rect.height() * (i as f32 / 6.0);
        let s = if i == 3 { center_stroke } else { grid_stroke };
        painter.line_segment([Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)], s);
    }

    let osc = &app.instruments.oscilloscope;
    if osc.ch1.enabled {
        draw_channel_trace(
            painter,
            rect,
            &osc.ch1.buffer,
            osc.ch1.volts_per_div,
            Color32::from_rgb(255, 230, 40),
        );
    }
    if osc.ch2.enabled {
        draw_channel_trace(
            painter,
            rect,
            &osc.ch2.buffer,
            osc.ch2.volts_per_div,
            Color32::from_rgb(0, 230, 255),
        );
    }
}

/// Plots the continuous voltage curve for a single oscilloscope channel.
fn draw_channel_trace(
    painter: &Painter,
    rect: Rect,
    buffer: &sim_core::instruments::TraceBuffer,
    volts_per_div: f32,
    color: Color32,
) {
    if buffer.samples.len() < 2 {
        return;
    }

    let points: Vec<Pos2> = buffer
        .samples
        .iter()
        .enumerate()
        .map(|(idx, sample)| {
            let x = rect.left() + (idx as f32 / buffer.samples.len() as f32) * rect.width();
            let y_norm = (sample.voltage / (volts_per_div * 6.0)).clamp(0.0, 1.0);
            let y = rect.bottom() - y_norm * rect.height();
            Pos2::new(x, y)
        })
        .collect();

    for i in 1..points.len() {
        painter.line_segment([points[i - 1], points[i]], Stroke::new(1.8_f32, color));
    }
}

/// Overlays numeric measurements and status parameters on the oscilloscope screen.
fn draw_hud(painter: &Painter, rect: Rect, app: &SimulatorApp) {
    let osc = &app.instruments.oscilloscope;
    let ch1_vpp = osc.ch1.buffer.peak_to_peak();
    let ch1_rms = osc.ch1.buffer.rms_voltage();
    let ch1_freq = osc
        .calculate_frequency(1)
        .map(|f| format!("{:.1} Hz", f))
        .unwrap_or_else(|| "--".to_string());

    let ch2_vpp = osc.ch2.buffer.peak_to_peak();
    let ch2_freq = osc
        .calculate_frequency(2)
        .map(|f| format!("{:.1} Hz", f))
        .unwrap_or_else(|| "--".to_string());

    let hud_text = format!(
        "CH1: {:.2}V Vpp | {:.2}V RMS | Freq: {}\nCH2: {:.2}V Vpp | Freq: {}",
        ch1_vpp, ch1_rms, ch1_freq, ch2_vpp, ch2_freq
    );

    painter.text(
        rect.left_top() + Vec2::new(12.0, 10.0),
        egui::Align2::LEFT_TOP,
        hud_text,
        FontId::monospace(11.0),
        Color32::from_rgb(170, 240, 190),
    );
}
