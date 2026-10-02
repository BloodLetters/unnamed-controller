use egui::{Color32, FontId, Painter, Pos2, Rect, Stroke, Ui, Vec2};
use sim_core::netlist::PinId;

use crate::app::SimulatorApp;

/// Renders the 8-Channel Digital Logic Analyzer timing diagram and channel mapping controls.
pub fn render_logic_analyzer_view(ui: &mut Ui, app: &mut SimulatorApp) {
    ui.horizontal(|ui| {
        if ui.button("🗑 Clear Buffer").clicked() {
            app.instruments.logic_analyzer.clear();
        }
        let pause_label = if app.instruments.logic_analyzer.paused {
            "▶ Resume"
        } else {
            "⏸ Pause"
        };
        if ui.button(pause_label).clicked() {
            app.instruments.logic_analyzer.paused = !app.instruments.logic_analyzer.paused;
        }

        ui.separator();

        if ui.button("💾 Export VCD (GTKWave)").clicked() {
            export_vcd_file(app);
        }
    });

    ui.separator();

    render_channel_pin_pickers(ui, app);

    ui.separator();

    let avail_width = ui.available_width();
    let lane_height = 24.0;
    let total_height = lane_height * 8.0 + 30.0;

    let (response, painter) =
        ui.allocate_painter(Vec2::new(avail_width, total_height), egui::Sense::hover());
    let rect = response.rect;

    draw_timing_lanes(&painter, rect, app, lane_height);
}

/// Renders pin assignment dropdowns or drag values for channels D0 through D7.
fn render_channel_pin_pickers(ui: &mut Ui, app: &mut SimulatorApp) {
    ui.horizontal_wrapped(|ui| {
        for i in 0..app.instruments.logic_analyzer.channels.len() {
            let ch = &mut app.instruments.logic_analyzer.channels[i];
            let mut pin_val = ch.pin.map(|p| p.0).unwrap_or(0);
            ui.label(format!("{}:", ch.name));
            if ui
                .add(egui::DragValue::new(&mut pin_val).clamp_range(0..=999))
                .changed()
            {
                ch.pin = if pin_val > 0 {
                    Some(PinId(pin_val))
                } else {
                    None
                };
            }
        }
    });
}

/// Prompts the user to save the captured digital trace as an IEEE 1364 VCD file.
#[cfg(not(target_arch = "wasm32"))]
fn export_vcd_file(app: &SimulatorApp) {
    let file = rfd::FileDialog::new()
        .set_file_name("trace.vcd")
        .add_filter("VCD Trace File", &["vcd"])
        .save_file();

    if let Some(path) = file {
        let vcd_content = app.instruments.logic_analyzer.export_vcd();
        let _ = std::fs::write(path, vcd_content);
    }
}

/// WebAssembly placeholder: native file dialogs are unavailable in the browser.
#[cfg(target_arch = "wasm32")]
fn export_vcd_file(_app: &SimulatorApp) {}

/// Draws square-wave timing diagrams across all 8 logic analyzer channels.
fn draw_timing_lanes(painter: &Painter, rect: Rect, app: &SimulatorApp, lane_height: f32) {
    painter.rect_filled(rect, 4.0, Color32::from_rgb(18, 20, 24));
    painter.rect_stroke(
        rect,
        4.0,
        Stroke::new(1.0_f32, Color32::from_rgb(45, 55, 65)),
    );

    let channels = &app.instruments.logic_analyzer.channels;
    let wave_left = rect.left() + 50.0;
    let wave_width = rect.width() - 60.0;

    for (i, ch) in channels.iter().enumerate() {
        let top = rect.top() + (i as f32) * lane_height;
        let bottom = top + lane_height;
        let mid = (top + bottom) / 2.0;

        painter.line_segment(
            [
                Pos2::new(rect.left(), bottom),
                Pos2::new(rect.right(), bottom),
            ],
            Stroke::new(0.5_f32, Color32::from_rgb(35, 45, 55)),
        );

        painter.text(
            Pos2::new(rect.left() + 10.0, mid),
            egui::Align2::LEFT_CENTER,
            &ch.name,
            FontId::monospace(11.0),
            Color32::from_rgb(180, 200, 220),
        );

        draw_channel_wave(painter, wave_left, top, wave_width, lane_height, ch);
    }
}

/// Plots the step-function digital state transitions for a single channel lane.
fn draw_channel_wave(
    painter: &Painter,
    left: f32,
    top: f32,
    width: f32,
    height: f32,
    ch: &sim_core::instruments::LogicChannel,
) {
    if ch.history.is_empty() {
        return;
    }

    let y_low = top + height - 5.0;
    let y_high = top + 5.0;
    let stroke = Stroke::new(1.5_f32, Color32::from_rgb(80, 255, 120));

    let len = ch.history.len();
    let mut prev_pos: Option<Pos2> = None;

    for (idx, &(_, state)) in ch.history.iter().enumerate() {
        let x = left + (idx as f32 / len as f32) * width;
        let y = if state { y_high } else { y_low };
        let curr = Pos2::new(x, y);

        if let Some(prev) = prev_pos {
            if prev.y != curr.y {
                painter.line_segment([prev, Pos2::new(curr.x, prev.y)], stroke);
                painter.line_segment([Pos2::new(curr.x, prev.y), curr], stroke);
            } else {
                painter.line_segment([prev, curr], stroke);
            }
        }
        prev_pos = Some(curr);
    }
}
