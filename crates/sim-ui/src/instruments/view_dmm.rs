use egui::{Color32, FontId, RichText, Ui};
use sim_core::instruments::DmmMode;
use sim_core::netlist::PinId;

use crate::app::SimulatorApp;

/// Renders the Digital Multimeter interface, rotary function selector, and probe terminals.
pub fn render_dmm_view(ui: &mut Ui, app: &mut SimulatorApp) {
    let dmm = &mut app.instruments.dmm;

    ui.horizontal(|ui| {
        ui.label(RichText::new("Measurement Mode:").strong());
        ui.selectable_value(&mut dmm.mode, DmmMode::VoltageDc, "DC V");
        ui.selectable_value(&mut dmm.mode, DmmMode::VoltageAc, "AC V");
        ui.selectable_value(&mut dmm.mode, DmmMode::Continuity, "Continuity 🔊");
        ui.selectable_value(&mut dmm.mode, DmmMode::Frequency, "Frequency (Hz)");
        ui.selectable_value(&mut dmm.mode, DmmMode::DutyCycle, "Duty Cycle (%)");
    });

    ui.separator();

    ui.horizontal(|ui| {
        ui.label(RichText::new("Probe Inputs:").strong());

        let mut red_val = dmm.probe_red.map(|p| p.0).unwrap_or(0);
        ui.label(RichText::new("Red Probe (+):").color(Color32::from_rgb(255, 80, 80)));
        if ui
            .add(egui::DragValue::new(&mut red_val).prefix("Pin "))
            .changed()
        {
            dmm.probe_red = if red_val > 0 {
                Some(PinId(red_val))
            } else {
                None
            };
        }

        let mut black_val = dmm.probe_black.map(|p| p.0).unwrap_or(1);
        ui.label(RichText::new("Black Probe (-/GND):").color(Color32::from_rgb(160, 160, 170)));
        if ui
            .add(egui::DragValue::new(&mut black_val).prefix("Pin "))
            .changed()
        {
            dmm.probe_black = Some(PinId(black_val));
        }
    });

    ui.separator();

    render_dmm_display(ui, app);
}

/// Renders the high-contrast segmented LCD digital multimeter display panel.
fn render_dmm_display(ui: &mut Ui, app: &SimulatorApp) {
    let dmm = &app.instruments.dmm;
    let reading_text = dmm.display_string();

    let bg_color = Color32::from_rgb(15, 24, 20);
    let border_color = Color32::from_rgb(40, 80, 60);

    egui::Frame::none()
        .fill(bg_color)
        .stroke(egui::Stroke::new(2.0_f32, border_color))
        .rounding(6.0)
        .inner_margin(egui::Margin::symmetric(24.0, 18.0))
        .show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new("PRECISION DIGITAL MULTIMETER")
                        .font(FontId::proportional(11.0))
                        .color(Color32::from_rgb(100, 160, 120)),
                );

                let display_color = if dmm.continuity_active {
                    Color32::from_rgb(255, 220, 0)
                } else {
                    Color32::from_rgb(120, 255, 170)
                };

                ui.label(
                    RichText::new(&reading_text)
                        .font(FontId::monospace(32.0))
                        .color(display_color),
                );

                if dmm.continuity_active {
                    ui.label(
                        RichText::new("⚡ CONTINUITY DETECTED - CIRCUIT CLOSED ⚡")
                            .font(FontId::proportional(12.0))
                            .color(Color32::from_rgb(255, 220, 50)),
                    );
                }
            });
        });
}
