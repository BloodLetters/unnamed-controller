//! Property inspector panel for MQ-135 gas sensor components.

use egui::{Color32, RichText, Ui, Vec2};
use sim_components::component::Mq135;

/// Renders inspector controls, live gas concentration telemetry, and threshold adjustment for MQ-135.
pub fn render_mq135_inspector_content(
    ui: &mut Ui,
    sensor: &mut Mq135,
    engine: &sim_core::engine::Engine,
    on_delete: impl FnOnce(),
) {
    ui.label(RichText::new("MQ-135 Gas Sensor").strong().size(15.0));
    ui.label(
        RichText::new("Air Quality / Multi-Gas Semiconductor Sensor")
            .weak()
            .size(11.0),
    );
    ui.separator();

    render_mq135_status(ui, sensor);
    ui.separator();

    render_mq135_controls(ui, sensor);
    ui.separator();

    render_mq135_electrical(ui, engine, sensor);
    ui.separator();

    if ui.button("u{1F5D1} Delete Sensor").clicked() {
        on_delete();
    }
}

/// Renders power indicator and live gas concentration with digital alarm state.
fn render_mq135_status(ui: &mut Ui, sensor: &Mq135) {
    ui.horizontal(|ui| {
        ui.label("Power:");
        let (rect, _resp) = ui.allocate_exact_size(Vec2::new(12.0, 12.0), egui::Sense::hover());
        let power_color = if sensor.is_powered() {
            Color32::from_rgb(76, 175, 80)
        } else {
            Color32::from_rgb(80, 85, 95)
        };
        ui.painter().circle_filled(rect.center(), 5.0, power_color);

        if sensor.is_powered() {
            ui.label(RichText::new("Powered (Active)").color(Color32::from_rgb(100, 255, 100)));
        } else {
            ui.label(RichText::new("Unpowered (<2.5V)").weak());
        }
    });

    ui.add_space(3.0);
    ui.horizontal(|ui| {
        ui.label("Gas Concentration:");
        let ppm_color = if sensor.digital_output_active() {
            Color32::from_rgb(255, 120, 60)
        } else {
            Color32::from_rgb(80, 230, 140)
        };
        ui.label(
            RichText::new(format!("{:.0} ppm", sensor.ppm()))
                .strong()
                .color(ppm_color),
        );
    });

    ui.horizontal(|ui| {
        ui.label("Analog Output:");
        ui.label(
            RichText::new(format!("{:.2} V", sensor.analog_output_voltage()))
                .strong()
                .color(Color32::from_rgb(251, 191, 36)),
        );
    });

    ui.horizontal(|ui| {
        ui.label("Digital Output:");
        let (dout_text, dout_color) = if sensor.digital_output_active() {
            ("ALARM (LOW)", Color32::from_rgb(255, 100, 60))
        } else {
            ("Normal (HIGH)", Color32::from_rgb(100, 200, 120))
        };
        ui.label(RichText::new(dout_text).strong().color(dout_color));
    });
}

/// Renders interactive sliders for gas concentration and alarm threshold adjustments.
fn render_mq135_controls(ui: &mut Ui, sensor: &mut Mq135) {
    ui.label(RichText::new("Simulation Controls").strong());

    let mut ppm = sensor.ppm();
    if ui
        .add(egui::Slider::new(&mut ppm, 10.0..=1000.0).text("Gas (ppm)"))
        .changed()
    {
        sensor.set_ppm(ppm);
    }

    let mut threshold = sensor.threshold_ppm();
    if ui
        .add(egui::Slider::new(&mut threshold, 10.0..=1000.0).text("Threshold (ppm)"))
        .changed()
    {
        sensor.set_threshold_ppm(threshold);
    }

    ui.add_space(4.0);
    ui.horizontal(|ui| {
        if ui.button("Clean Air (400 ppm)").clicked() {
            sensor.set_ppm(400.0);
        }
        if ui.button("Mild (600 ppm)").clicked() {
            sensor.set_ppm(600.0);
        }
    });

    ui.horizontal(|ui| {
        if ui.button("High CO2 (800 ppm)").clicked() {
            sensor.set_ppm(800.0);
        }
        if ui.button("Danger (1000 ppm)").clicked() {
            sensor.set_ppm(1000.0);
        }
    });
}

/// Renders real-time voltage measurements on each sensor terminal pin.
fn render_mq135_electrical(ui: &mut Ui, engine: &sim_core::engine::Engine, sensor: &Mq135) {
    ui.label(RichText::new("Terminal Telemetry").strong());

    let v_vcc = crate::sim::get_pin_voltage(engine, sensor.vcc());
    let v_gnd = crate::sim::get_pin_voltage(engine, sensor.gnd());
    let v_aout = crate::sim::get_pin_voltage(engine, sensor.aout());
    let v_dout = crate::sim::get_pin_voltage(engine, sensor.dout());

    ui.label(format!("VCC Pin: {:?} ({:.2} V)", sensor.vcc(), v_vcc));
    ui.label(format!("GND Pin: {:?} ({:.2} V)", sensor.gnd(), v_gnd));
    ui.label(format!("AOUT Pin: {:?} ({:.2} V)", sensor.aout(), v_aout));
    ui.label(format!("DOUT Pin: {:?} ({:.2} V)", sensor.dout(), v_dout));
    ui.label(format!("Supply Potential: {:.2} V", v_vcc - v_gnd));
}
