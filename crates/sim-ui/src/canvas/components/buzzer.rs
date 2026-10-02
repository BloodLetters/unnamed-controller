//! Canvas renderer for active buzzer components.

use egui::{Color32, FontId, Painter, Pos2, Rect, Stroke, Vec2};
use sim_components::component::Buzzer;

/// Width of the buzzer cylindrical body projected onto the canvas in canvas units.
pub const BUZZER_BODY_RADIUS: f32 = 16.0;
/// Horizontal distance from buzzer center to each terminal lead.
pub const BUZZER_PIN_OFFSET_X: f32 = 7.0;
/// Vertical distance from buzzer center down to terminal solder pads.
pub const BUZZER_PIN_OFFSET_Y: f32 = 24.0;

/// Returns relative coordinate offsets for VCC and GND terminals from component center.
pub fn buzzer_pin_offsets() -> [(f32, f32); 2] {
    [
        (-BUZZER_PIN_OFFSET_X, BUZZER_PIN_OFFSET_Y),
        (BUZZER_PIN_OFFSET_X, BUZZER_PIN_OFFSET_Y),
    ]
}

/// Renders metallic lead wires from the housing base down to solder pads.
fn draw_buzzer_leads(painter: &Painter, center: Pos2, zoom: f32, rot: u16) {
    let lead_stroke = Stroke::new(1.8 * zoom, Color32::from_rgb(170, 175, 185));
    let offsets = buzzer_pin_offsets();
    for &(ox, oy) in &offsets {
        let p_start = center
            + crate::canvas::rotate_offset(Vec2::new(ox, BUZZER_BODY_RADIUS - 2.0), rot) * zoom;
        let p_end = center + crate::canvas::rotate_offset(Vec2::new(ox, oy), rot) * zoom;
        painter.line_segment([p_start, p_end], lead_stroke);
    }
}

/// Renders the black epoxy cylindrical housing with polarity stripe and sound port.
fn draw_buzzer_body(
    painter: &Painter,
    center: Pos2,
    zoom: f32,
    rot: u16,
    buzzer: &Buzzer,
    is_selected: bool,
) {
    let body_color = Color32::from_rgb(28, 28, 30);
    let border_color = if is_selected {
        Color32::from_rgb(0, 229, 255)
    } else {
        Color32::from_rgb(60, 62, 68)
    };
    let border_width = if is_selected { 2.0 } else { 1.2 };

    painter.circle_filled(center, BUZZER_BODY_RADIUS * zoom, body_color);
    painter.circle_stroke(
        center,
        BUZZER_BODY_RADIUS * zoom,
        Stroke::new(border_width * zoom, border_color),
    );

    draw_buzzer_sound_port(painter, center, zoom, rot, buzzer);
    draw_buzzer_polarity_stripe(painter, center, zoom, rot);
}

/// Renders the circular sound emission port on the top face of the buzzer housing.
fn draw_buzzer_sound_port(painter: &Painter, center: Pos2, zoom: f32, rot: u16, buzzer: &Buzzer) {
    let port_center = center + crate::canvas::rotate_offset(Vec2::new(0.0, -3.0), rot) * zoom;

    let port_color = if buzzer.is_active() {
        Color32::from_rgb(255, 200, 60)
    } else {
        Color32::from_rgb(50, 52, 56)
    };

    painter.circle_filled(port_center, 6.5 * zoom, port_color);
    painter.circle_stroke(
        port_center,
        6.5 * zoom,
        Stroke::new(0.8 * zoom, Color32::from_rgb(80, 82, 88)),
    );

    for i in 0..4 {
        let angle = i as f32 * std::f32::consts::TAU / 4.0;
        let spoke_end = port_center + Vec2::new(angle.cos(), angle.sin()) * 4.5 * zoom;
        painter.line_segment(
            [port_center, spoke_end],
            Stroke::new(0.7 * zoom, Color32::from_rgb(30, 32, 36)),
        );
    }

    if buzzer.is_active() {
        let wave_color = Color32::from_rgba_unmultiplied(255, 200, 60, 60);
        painter.circle_stroke(
            port_center,
            10.0 * zoom,
            Stroke::new(1.5 * zoom, wave_color),
        );
        painter.circle_stroke(
            port_center,
            14.0 * zoom,
            Stroke::new(
                1.0 * zoom,
                Color32::from_rgba_unmultiplied(255, 200, 60, 30),
            ),
        );
    }
}

/// Renders the positive polarity identification stripe on the buzzer casing.
fn draw_buzzer_polarity_stripe(painter: &Painter, center: Pos2, zoom: f32, rot: u16) {
    let stripe_rect_center =
        center + crate::canvas::rotate_offset(Vec2::new(-BUZZER_PIN_OFFSET_X, 9.0), rot) * zoom;
    let stripe_size = if rot == 90 || rot == 270 {
        Vec2::new(5.0, 2.5) * zoom
    } else {
        Vec2::new(2.5, 5.0) * zoom
    };
    let stripe_rect = Rect::from_center_size(stripe_rect_center, stripe_size);
    painter.rect_filled(stripe_rect, 0.5 * zoom, Color32::from_rgb(240, 230, 200));

    let plus_pos =
        center + crate::canvas::rotate_offset(Vec2::new(-BUZZER_PIN_OFFSET_X, 14.0), rot) * zoom;
    painter.text(
        plus_pos,
        egui::Align2::CENTER_CENTER,
        "+",
        FontId::proportional(6.0 * zoom),
        Color32::from_rgb(240, 230, 200),
    );
}

/// Renders through-hole solder pads with VCC and GND silkscreen labels.
fn draw_buzzer_terminals(painter: &Painter, center: Pos2, zoom: f32, rot: u16) {
    let offsets = buzzer_pin_offsets();
    let labels = ["VCC", "GND"];
    let label_colors = [
        Color32::from_rgb(248, 113, 113),
        Color32::from_rgb(130, 138, 150),
    ];

    for (i, &(ox, oy)) in offsets.iter().enumerate() {
        let terminal_pos = center + crate::canvas::rotate_offset(Vec2::new(ox, oy), rot) * zoom;
        painter.circle_filled(terminal_pos, 3.2 * zoom, Color32::from_rgb(212, 168, 68));
        painter.circle_filled(terminal_pos, 1.6 * zoom, Color32::from_rgb(12, 14, 18));

        let text_offset = crate::canvas::rotate_offset(Vec2::new(0.0, 6.8), rot) * zoom;
        painter.text(
            terminal_pos + text_offset,
            egui::Align2::CENTER_CENTER,
            labels[i],
            FontId::proportional(5.2 * zoom),
            label_colors[i],
        );
    }
}

/// Renders a single active buzzer at the specified canvas coordinates and rotation.
pub fn draw_single_buzzer(
    painter: &Painter,
    center: Pos2,
    zoom: f32,
    rot: u16,
    comp: &Buzzer,
    is_selected: bool,
) {
    draw_buzzer_leads(painter, center, zoom, rot);
    draw_buzzer_body(painter, center, zoom, rot, comp, is_selected);
    draw_buzzer_terminals(painter, center, zoom, rot);
}
