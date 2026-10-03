//! Canvas renderer for axial through-hole resistor components.

use egui::{Color32, FontId, Painter, Pos2, Rect, Stroke, Vec2};
use sim_components::component::Resistor;

/// Horizontal distance from the resistor center to each terminal lead.
pub const RESISTOR_PIN_OFFSET_X: f32 = 20.0;

/// Half-length of the resistor body along its axis.
const BODY_HALF_LENGTH: f32 = 9.0;

/// Radius of the resistor body measured perpendicular to its axis.
const BODY_RADIUS: f32 = 4.0;

/// E24 series resistance bands rendered from most to least significant digit.
fn band_colors(resistance_ohms: f32) -> [Color32; 4] {
    if resistance_ohms <= 0.0 {
        return [Color32::from_rgb(140, 140, 140); 4];
    }

    let exponent = resistance_ohms.log10().floor() - 1.0;
    let multiplier = exponent.clamp(0.0, 5.0) as u32;
    let normalized = resistance_ohms / 10f32.powf(multiplier as f32);
    let first_digit = normalized as u32 % 10;
    let second_digit = (normalized * 10.0) as u32 % 10;

    let digit_colors = [
        Color32::from_rgb(20, 20, 24),
        Color32::from_rgb(120, 66, 40),
        Color32::from_rgb(190, 30, 30),
        Color32::from_rgb(230, 140, 30),
        Color32::from_rgb(240, 210, 60),
        Color32::from_rgb(60, 160, 70),
        Color32::from_rgb(40, 90, 200),
        Color32::from_rgb(130, 70, 190),
        Color32::from_rgb(150, 150, 155),
        Color32::from_rgb(245, 245, 250),
    ];

    [
        digit_colors[first_digit as usize],
        digit_colors[second_digit as usize],
        multiplier_color(multiplier),
        Color32::from_rgb(190, 150, 60),
    ]
}

/// Returns the tolerance band color for a decade multiplier index.
fn multiplier_color(multiplier: u32) -> Color32 {
    match multiplier {
        0 => Color32::from_rgb(180, 120, 30),
        1 => Color32::from_rgb(180, 90, 30),
        2 => Color32::from_rgb(190, 40, 40),
        3 => Color32::from_rgb(230, 140, 30),
        4 => Color32::from_rgb(240, 210, 60),
        _ => Color32::from_rgb(230, 230, 235),
    }
}

/// Renders the metal leads running from the body out to the through-hole pads.
fn draw_resistor_leads(painter: &Painter, center: Pos2, zoom: f32, rot: u16) {
    let lead_stroke = Stroke::new(1.6 * zoom, Color32::from_rgb(178, 182, 190));
    for direction in [-1.0_f32, 1.0] {
        let lead_start = center
            + crate::canvas::rotate_offset(Vec2::new(direction * BODY_HALF_LENGTH, 0.0), rot)
                * zoom;
        let lead_end = center
            + crate::canvas::rotate_offset(Vec2::new(direction * RESISTOR_PIN_OFFSET_X, 0.0), rot)
                * zoom;
        painter.line_segment([lead_start, lead_end], lead_stroke);
    }
}

/// Renders the ceramic body with its four color bands and value label.
fn draw_resistor_body(
    painter: &Painter,
    center: Pos2,
    zoom: f32,
    rot: u16,
    resistor: &Resistor,
    is_selected: bool,
) {
    let along = if rot == 90 || rot == 270 {
        Vec2::new(BODY_RADIUS, BODY_HALF_LENGTH) * zoom
    } else {
        Vec2::new(BODY_HALF_LENGTH, BODY_RADIUS) * zoom
    };
    let body_rect = Rect::from_center_size(center, along);

    let border_stroke = if is_selected {
        Stroke::new(2.0 * zoom, Color32::from_rgb(0, 229, 255))
    } else {
        Stroke::new(1.2 * zoom, Color32::from_rgb(38, 40, 46))
    };

    painter.rect_filled(
        body_rect,
        BODY_RADIUS * zoom,
        Color32::from_rgb(214, 196, 152),
    );
    painter.rect_stroke(body_rect, BODY_RADIUS * zoom, border_stroke);

    draw_resistor_bands(painter, center, zoom, rot, resistor);

    let label_offset = crate::canvas::rotate_offset(Vec2::new(0.0, -12.0), rot) * zoom;
    painter.text(
        center + label_offset,
        egui::Align2::CENTER_CENTER,
        format_resistance(resistor.resistance_ohms()),
        FontId::monospace(7.5 * zoom),
        Color32::from_rgb(226, 230, 238),
    );
}

/// Renders the E24 color code bands across the resistor body.
fn draw_resistor_bands(painter: &Painter, center: Pos2, zoom: f32, rot: u16, resistor: &Resistor) {
    let band_positions = [-6.5, -2.5, 1.5, 5.5];
    let band_colors = band_colors(resistor.resistance_ohms());

    for (position, color) in band_positions.iter().zip(band_colors.iter()) {
        let offset = crate::canvas::rotate_offset(Vec2::new(*position, 0.0), rot) * zoom;
        let size = if rot == 90 || rot == 270 {
            Vec2::new(BODY_RADIUS * 2.4, 1.8) * zoom
        } else {
            Vec2::new(1.8, BODY_RADIUS * 2.4) * zoom
        };
        painter.rect_filled(
            Rect::from_center_size(center + offset, size),
            0.5 * zoom,
            *color,
        );
    }
}

/// Renders through-hole solder pads at both resistor leads.
fn draw_resistor_terminals(painter: &Painter, center: Pos2, zoom: f32, rot: u16) {
    for direction in [-1.0_f32, 1.0] {
        let offset =
            crate::canvas::rotate_offset(Vec2::new(direction * RESISTOR_PIN_OFFSET_X, 0.0), rot)
                * zoom;
        let pad_pos = center + offset;
        painter.circle_filled(pad_pos, 3.2 * zoom, Color32::from_rgb(212, 168, 68));
        painter.circle_filled(pad_pos, 1.6 * zoom, Color32::from_rgb(12, 14, 18));
    }
}

/// Formats a resistance value using SI prefixes for readability.
pub fn format_resistance(resistance_ohms: f32) -> String {
    if resistance_ohms >= 1_000_000.0 {
        format!("{:.1}M", resistance_ohms / 1_000_000.0)
    } else if resistance_ohms >= 1_000.0 {
        format!("{:.1}k", resistance_ohms / 1_000.0)
    } else {
        format!("{:.0}", resistance_ohms)
    }
}

/// Renders a single resistor component at the designated screen coordinates.
pub fn draw_single_resistor(
    painter: &Painter,
    center: Pos2,
    zoom: f32,
    rot: u16,
    resistor: &Resistor,
    is_selected: bool,
) {
    draw_resistor_leads(painter, center, zoom, rot);
    draw_resistor_body(painter, center, zoom, rot, resistor, is_selected);
    draw_resistor_terminals(painter, center, zoom, rot);
}
