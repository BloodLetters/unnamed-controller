//! Canvas renderer for SG90 9g micro servo motor components.

use egui::{Color32, FontId, Painter, Pos2, Rect, Stroke, Vec2};
use sim_components::component::Servo;

/// Horizontal distance between adjacent terminal header pins.
pub const SERVO_PIN_PITCH: f32 = 12.0;
/// Vertical offset from the servo center down to the connector pins.
pub const SERVO_PIN_OFFSET_Y: f32 = 28.0;

/// Returns relative canvas coordinate offsets for GND, VCC, and PWM terminals.
pub fn servo_pin_offsets() -> [(f32, f32); 3] {
    [
        (-SERVO_PIN_PITCH, SERVO_PIN_OFFSET_Y),
        (0.0, SERVO_PIN_OFFSET_Y),
        (SERVO_PIN_PITCH, SERVO_PIN_OFFSET_Y),
    ]
}

/// Renders the three-wire ribbon cable (Brown, Red, Orange) connecting case to header pins.
fn draw_servo_cables(painter: &Painter, center: Pos2, zoom: f32, rot: u16) {
    let wire_colors = [
        Color32::from_rgb(110, 65, 35),
        Color32::from_rgb(220, 50, 45),
        Color32::from_rgb(245, 145, 30),
    ];
    let offsets = servo_pin_offsets();

    for (i, &(ox, oy)) in offsets.iter().enumerate() {
        let p_start = center + crate::canvas::rotate_offset(Vec2::new(ox, 12.0), rot) * zoom;
        let p_end = center + crate::canvas::rotate_offset(Vec2::new(ox, oy), rot) * zoom;
        let wire_stroke = Stroke::new(2.4 * zoom, wire_colors[i]);
        painter.line_segment([p_start, p_end], wire_stroke);
    }
}

/// Renders horizontal mounting flanges with screw eyelets on left and right sides.
fn draw_servo_mounting_tabs(painter: &Painter, center: Pos2, zoom: f32, rot: u16) {
    let tab_color = Color32::from_rgb(18, 92, 172);
    let border_stroke = Stroke::new(1.0 * zoom, Color32::from_rgb(12, 60, 115));
    let eyelet_color = Color32::from_rgb(30, 32, 38);

    for sign in [-1.0_f32, 1.0_f32] {
        let tab_center_off = Vec2::new(sign * 36.0, 0.0);
        let tab_center = center + crate::canvas::rotate_offset(tab_center_off, rot) * zoom;
        let tab_size = if rot == 90 || rot == 270 {
            Vec2::new(14.0, 18.0) * zoom
        } else {
            Vec2::new(18.0, 14.0) * zoom
        };
        let tab_rect = Rect::from_center_size(tab_center, tab_size);
        painter.rect_filled(tab_rect, 2.0 * zoom, tab_color);
        painter.rect_stroke(tab_rect, 2.0 * zoom, border_stroke);

        let eyelet_pos =
            center + crate::canvas::rotate_offset(Vec2::new(sign * 38.0, 0.0), rot) * zoom;
        painter.circle_filled(eyelet_pos, 2.2 * zoom, eyelet_color);
    }
}

/// Renders the main translucent blue rectangular housing with label badge and shaft tower.
fn draw_servo_body(painter: &Painter, center: Pos2, zoom: f32, rot: u16, is_selected: bool) {
    let body_color = Color32::from_rgb(26, 115, 212);
    let border_color = if is_selected {
        Color32::from_rgb(0, 229, 255)
    } else {
        Color32::from_rgb(14, 70, 135)
    };
    let border_width = if is_selected { 2.0 } else { 1.2 };
    let border_stroke = Stroke::new(border_width * zoom, border_color);

    let body_size = if rot == 90 || rot == 270 {
        Vec2::new(26.0, 56.0) * zoom
    } else {
        Vec2::new(56.0, 26.0) * zoom
    };
    let body_rect = Rect::from_center_size(center, body_size);
    painter.rect_filled(body_rect, 3.0 * zoom, body_color);
    painter.rect_stroke(body_rect, 3.0 * zoom, border_stroke);

    let label_size = if rot == 90 || rot == 270 {
        Vec2::new(18.0, 26.0) * zoom
    } else {
        Vec2::new(26.0, 18.0) * zoom
    };
    let label_rect = Rect::from_center_size(center, label_size);
    painter.rect_filled(label_rect, 1.5 * zoom, Color32::from_rgb(15, 20, 28));
    painter.text(
        center,
        egui::Align2::CENTER_CENTER,
        "SG90",
        FontId::proportional(7.5 * zoom),
        Color32::from_rgb(255, 215, 64),
    );

    let shaft_center = center + crate::canvas::rotate_offset(Vec2::new(-16.0, 0.0), rot) * zoom;
    painter.circle_filled(shaft_center, 8.5 * zoom, Color32::from_rgb(20, 85, 160));
    painter.circle_stroke(
        shaft_center,
        8.5 * zoom,
        Stroke::new(1.0 * zoom, Color32::from_rgb(10, 45, 90)),
    );
}

/// Renders the rotating nylon servo horn indicating the exact current shaft angle.
fn draw_servo_horn(painter: &Painter, center: Pos2, zoom: f32, rot: u16, servo: &Servo) {
    let shaft_center = center + crate::canvas::rotate_offset(Vec2::new(-16.0, 0.0), rot) * zoom;
    let horn_color = Color32::from_rgb(240, 243, 248);
    let horn_border = Stroke::new(1.0 * zoom, Color32::from_rgb(160, 165, 175));

    let effective_angle_deg = rot as f32 + (servo.angle() - 90.0);
    let rad = effective_angle_deg.to_radians();
    let dir = Vec2::new(rad.cos(), rad.sin());
    let normal = Vec2::new(-rad.sin(), rad.cos());

    painter.circle_filled(shaft_center, 6.0 * zoom, horn_color);
    painter.circle_stroke(shaft_center, 6.0 * zoom, horn_border);

    let arm_length = 22.0 * zoom;
    let base_half_w = 3.5 * zoom;
    let tip_half_w = 2.0 * zoom;

    let p0 = shaft_center - normal * base_half_w;
    let p1 = shaft_center + normal * base_half_w;
    let p2 = shaft_center + dir * arm_length + normal * tip_half_w;
    let p3 = shaft_center + dir * arm_length - normal * tip_half_w;

    let points = vec![p0, p1, p2, p3];
    painter.add(egui::Shape::convex_polygon(points, horn_color, horn_border));
    painter.circle_filled(shaft_center + dir * arm_length, tip_half_w, horn_color);

    for dist in [10.0, 15.0, 20.0] {
        let hole_pos = shaft_center + dir * (dist * zoom);
        painter.circle_filled(hole_pos, 1.2 * zoom, Color32::from_rgb(50, 55, 65));
    }

    painter.circle_filled(shaft_center, 2.2 * zoom, Color32::from_rgb(190, 195, 205));
    painter.circle_filled(shaft_center, 1.0 * zoom, Color32::from_rgb(80, 85, 95));

    let angle_label_pos = shaft_center + Vec2::new(0.0, -13.0 * zoom);
    painter.text(
        angle_label_pos,
        egui::Align2::CENTER_CENTER,
        format!("{:.0}°", servo.angle()),
        FontId::proportional(7.0 * zoom),
        Color32::from_rgb(220, 235, 255),
    );
}

/// Renders the solder pads and pin silk-screen labels for GND, VCC, and PWM.
fn draw_servo_terminals(painter: &Painter, center: Pos2, zoom: f32, rot: u16) {
    let offsets = servo_pin_offsets();
    let labels = ["G", "V", "S"];
    let label_colors = [
        Color32::from_rgb(180, 140, 110),
        Color32::from_rgb(255, 110, 110),
        Color32::from_rgb(255, 190, 80),
    ];

    for (i, &(ox, oy)) in offsets.iter().enumerate() {
        let terminal_pos = center + crate::canvas::rotate_offset(Vec2::new(ox, oy), rot) * zoom;
        painter.circle_filled(terminal_pos, 3.2 * zoom, Color32::from_rgb(212, 168, 68));
        painter.circle_filled(terminal_pos, 1.6 * zoom, Color32::from_rgb(12, 14, 18));

        let text_offset = crate::canvas::rotate_offset(Vec2::new(0.0, 7.5), rot) * zoom;
        painter.text(
            terminal_pos + text_offset,
            egui::Align2::CENTER_CENTER,
            labels[i],
            FontId::proportional(6.0 * zoom),
            label_colors[i],
        );
    }
}

/// Renders a single SG90 servo motor component at the designated screen coordinates.
pub fn draw_single_servo(
    painter: &Painter,
    center: Pos2,
    zoom: f32,
    rot: u16,
    servo: &Servo,
    is_selected: bool,
) {
    draw_servo_cables(painter, center, zoom, rot);
    draw_servo_mounting_tabs(painter, center, zoom, rot);
    draw_servo_body(painter, center, zoom, rot, is_selected);
    draw_servo_horn(painter, center, zoom, rot, servo);
    draw_servo_terminals(painter, center, zoom, rot);
}

/// Renders quick angle presets in the context menu for a servo motor.
pub fn render_servo_context_options(ui: &mut egui::Ui, servo: &mut Servo) {
    ui.menu_button("🎯 Set Angle", |ui| {
        for &(label, angle) in &[
            ("0° (Min)", 0.0),
            ("90° (Center)", 90.0),
            ("180° (Max)", 180.0),
        ] {
            if ui.button(label).clicked() {
                servo.set_angle(angle);
                ui.close_menu();
            }
        }
    });
}
