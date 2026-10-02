use egui::{Color32, FontId, ProgressBar, RichText, Ui};
use sim_core::netlist::PinId;

use crate::app::SimulatorApp;

/// Renders the Frequency Counter and PWM Duty Cycle analyzer display.
pub fn render_pwm_analyzer_view(ui: &mut Ui, app: &mut SimulatorApp) {
    let pwm = &mut app.instruments.pwm;

    ui.horizontal(|ui| {
        ui.label(RichText::new("Probe Input Pin:").strong());
        let mut pin_val = pwm.pin.map(|p| p.0).unwrap_or(0);
        if ui
            .add(egui::DragValue::new(&mut pin_val).prefix("Pin "))
            .changed()
        {
            pwm.pin = if pin_val > 0 {
                Some(PinId(pin_val))
            } else {
                None
            };
        }
        if ui.button("Reset Counter").clicked() {
            pwm.reset();
        }
    });

    ui.separator();

    let freq = pwm.frequency_hz;
    let duty = pwm.duty_percent;
    let period_us = pwm.high_duration_us + pwm.low_duration_us;

    egui::Frame::none()
        .fill(Color32::from_rgb(18, 22, 28))
        .stroke(egui::Stroke::new(1.5_f32, Color32::from_rgb(50, 70, 95)))
        .rounding(6.0)
        .inner_margin(egui::Margin::symmetric(24.0, 16.0))
        .show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new("DIGITAL FREQUENCY & PWM ANALYZER")
                        .font(FontId::proportional(11.0))
                        .color(Color32::from_rgb(120, 160, 210)),
                );

                let freq_str = if freq >= 1000.0 {
                    format!("{:.3} kHz", freq / 1000.0)
                } else {
                    format!("{:.1} Hz", freq)
                };

                ui.label(
                    RichText::new(&freq_str)
                        .font(FontId::monospace(34.0))
                        .color(Color32::from_rgb(80, 200, 255)),
                );

                ui.add_space(8.0);
                ui.label(format!("Duty Cycle: {:.1}%", duty));
                ui.add(ProgressBar::new(duty / 100.0).desired_width(280.0));

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.label(format!("T_high: {} µs", pwm.high_duration_us));
                    ui.separator();
                    ui.label(format!("T_low: {} µs", pwm.low_duration_us));
                    ui.separator();
                    ui.label(format!("Period: {} µs", period_us));
                });
            });
        });
}
