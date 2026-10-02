//! Property inspector panel for SG90 micro servo actuator components.

use egui::{Color32, RichText, Ui, Vec2};

/// Renders inspector controls and telemetry for a single SG90 servo component.
pub fn render_servo_inspector_content(
    ui: &mut Ui,
    servo: &mut sim_components::component::Servo,
    engine: &sim_core::engine::Engine,
    on_delete: impl FnOnce(),
) {
    ui.label(RichText::new("SG90 Micro Servo").strong().size(15.0));
    ui.label(
        RichText::new("9g Positional Actuator (0° – 180°)")
            .weak()
            .size(11.0),
    );
    ui.separator();

    render_servo_status(ui, servo);
    ui.separator();

    render_servo_controls(ui, servo);
    ui.separator();

    render_electrical_specs(ui, engine, servo);
    ui.separator();

    if ui.button("🗑 Delete Servo").clicked() {
        on_delete();
    }
}

/// Renders power indicator, current physical angle, and PWM pulse width telemetry.
fn render_servo_status(ui: &mut Ui, servo: &sim_components::component::Servo) {
    ui.horizontal(|ui| {
        ui.label("Power:");
        let (rect, _resp) = ui.allocate_exact_size(Vec2::new(12.0, 12.0), egui::Sense::hover());
        let power_color = if servo.is_powered() {
            Color32::from_rgb(76, 175, 80)
        } else {
            Color32::from_rgb(80, 85, 95)
        };
        ui.painter().circle_filled(rect.center(), 5.0, power_color);

        if servo.is_powered() {
            ui.label(RichText::new("Powered (Active)").color(Color32::from_rgb(100, 255, 100)));
        } else {
            ui.label(RichText::new("Unpowered (<3.8V)").weak());
        }
    });

    ui.add_space(3.0);
    ui.horizontal(|ui| {
        ui.label("Current Angle:");
        ui.label(
            RichText::new(format!("{:.1}°", servo.angle()))
                .strong()
                .color(Color32::from_rgb(0, 229, 255)),
        );
    });

    ui.horizontal(|ui| {
        ui.label("Target Angle:");
        ui.label(format!("{:.1}°", servo.target_angle()));
    });

    ui.horizontal(|ui| {
        ui.label("PWM Pulse:");
        ui.label(format!("{} µs", servo.last_pulse_width_us()));
    });
}

/// Renders interactive angle slider and quick preset position buttons.
fn render_servo_controls(ui: &mut Ui, servo: &mut sim_components::component::Servo) {
    ui.label(RichText::new("Manual Position").strong());

    let mut current_target = servo.target_angle();
    if ui
        .add(egui::Slider::new(&mut current_target, 0.0..=180.0).text("Angle (°)"))
        .changed()
    {
        servo.set_target_angle(current_target);
    }

    ui.add_space(4.0);
    ui.horizontal(|ui| {
        if ui.button("0° (Min)").clicked() {
            servo.set_target_angle(0.0);
        }
        if ui.button("90° (Center)").clicked() {
            servo.set_target_angle(90.0);
        }
        if ui.button("180° (Max)").clicked() {
            servo.set_target_angle(180.0);
        }
    });
}

/// Renders live electrical voltages on GND, VCC, and PWM signal pins.
fn render_electrical_specs(
    ui: &mut Ui,
    engine: &sim_core::engine::Engine,
    servo: &sim_components::component::Servo,
) {
    ui.label(RichText::new("Terminal Telemetry").strong());

    let v_gnd = crate::sim::get_pin_voltage(engine, servo.gnd());
    let v_vcc = crate::sim::get_pin_voltage(engine, servo.vcc());
    let v_pwm = crate::sim::get_pin_voltage(engine, servo.pwm());

    ui.label(format!("GND Pin: {:?} ({:.2} V)", servo.gnd(), v_gnd));
    ui.label(format!("VCC Pin: {:?} ({:.2} V)", servo.vcc(), v_vcc));
    ui.label(format!(
        "PWM/Signal Pin: {:?} ({:.2} V)",
        servo.pwm(),
        v_pwm
    ));
    ui.label(format!("Supply Potential: {:.2} V", v_vcc - v_gnd));
}
