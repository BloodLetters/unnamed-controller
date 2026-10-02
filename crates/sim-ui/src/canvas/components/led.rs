//! Canvas renderer for 5mm through-hole LED components.

use egui::{Color32, FontId, Painter, Pos2, Rect, Stroke, Ui, Vec2};
use sim_components::component::{Led, LedColor};

/// Horizontal distance from LED center to each terminal lead.
pub const LED_PIN_OFFSET_X: f32 = 8.0;
/// Vertical distance from LED center down to the terminal leads.
pub const LED_PIN_OFFSET_Y: f32 = 18.0;

/// Renders the metallic leads connecting the LED bulb to through-hole pads.
fn draw_led_leads(painter: &Painter, center: Pos2, zoom: f32, rot: u16) {
    let lead_stroke = Stroke::new(1.8 * zoom, Color32::from_rgb(170, 175, 185));
    let a_start =
        center + crate::canvas::rotate_offset(Vec2::new(-LED_PIN_OFFSET_X, 6.0), rot) * zoom;
    let a_end = center
        + crate::canvas::rotate_offset(Vec2::new(-LED_PIN_OFFSET_X, LED_PIN_OFFSET_Y), rot) * zoom;
    let k_start =
        center + crate::canvas::rotate_offset(Vec2::new(LED_PIN_OFFSET_X, 6.0), rot) * zoom;
    let k_end = center
        + crate::canvas::rotate_offset(Vec2::new(LED_PIN_OFFSET_X, LED_PIN_OFFSET_Y), rot) * zoom;

    painter.line_segment([a_start, a_end], lead_stroke);
    painter.line_segment([k_start, k_end], lead_stroke);
}

/// Renders the epoxy dome, flange rim, light emission glow, and selection border.
fn draw_led_body(
    painter: &Painter,
    center: Pos2,
    zoom: f32,
    rot: u16,
    led: &sim_components::component::Led,
    is_selected: bool,
) {
    let radius = 10.0 * zoom;
    let [r, g, b] = if led.is_lit() {
        led.color.rgb()
    } else {
        led.color.off_rgb()
    };

    if led.is_lit() {
        let glow_radius = (16.0 + 10.0 * led.brightness()) * zoom;
        let alpha = (90.0 * led.brightness()) as u8;
        painter.circle_filled(
            center,
            glow_radius,
            Color32::from_rgba_unmultiplied(r, g, b, alpha),
        );
    }

    let border_stroke = if is_selected {
        Stroke::new(2.0 * zoom, Color32::from_rgb(0, 229, 255))
    } else {
        Stroke::new(1.2 * zoom, Color32::from_rgb(30, 32, 36))
    };

    let flange_size = if rot == 90 || rot == 270 {
        Vec2::new(10.0, 22.0) * zoom
    } else {
        Vec2::new(22.0, 10.0) * zoom
    };
    let flange_offset = crate::canvas::rotate_offset(Vec2::new(0.0, 3.0), rot) * zoom;
    let flange_rect = Rect::from_center_size(center + flange_offset, flange_size);
    painter.rect_filled(
        flange_rect,
        2.0 * zoom,
        Color32::from_rgba_unmultiplied(r, g, b, 200),
    );
    painter.rect_stroke(flange_rect, 2.0 * zoom, border_stroke);

    painter.circle_filled(center, radius, Color32::from_rgb(r, g, b));
    painter.circle_stroke(center, radius, border_stroke);

    let highlight_offset = crate::canvas::rotate_offset(Vec2::new(-2.5, -2.5), rot) * zoom;
    if led.is_lit() {
        painter.circle_filled(
            center + highlight_offset,
            3.5 * zoom,
            Color32::from_rgba_unmultiplied(255, 255, 255, 210),
        );
    } else {
        painter.circle_filled(
            center + highlight_offset,
            2.5 * zoom,
            Color32::from_rgba_unmultiplied(255, 255, 255, 45),
        );
    }
}

/// Renders through-hole solder pads with silkscreen polarity indicators.
fn draw_led_terminals(painter: &Painter, center: Pos2, zoom: f32, rot: u16) {
    let anode_offset =
        crate::canvas::rotate_offset(Vec2::new(-LED_PIN_OFFSET_X, LED_PIN_OFFSET_Y), rot) * zoom;
    let cathode_offset =
        crate::canvas::rotate_offset(Vec2::new(LED_PIN_OFFSET_X, LED_PIN_OFFSET_Y), rot) * zoom;
    let anode_pos = center + anode_offset;
    let cathode_pos = center + cathode_offset;

    for pad_pos in [anode_pos, cathode_pos] {
        painter.circle_filled(pad_pos, 3.2 * zoom, Color32::from_rgb(212, 168, 68));
        painter.circle_filled(pad_pos, 1.6 * zoom, Color32::from_rgb(12, 14, 18));
    }

    let a_label_off = crate::canvas::rotate_offset(Vec2::new(-4.5, 0.0), rot) * zoom;
    let k_label_off = crate::canvas::rotate_offset(Vec2::new(4.5, 0.0), rot) * zoom;
    painter.text(
        anode_pos + a_label_off,
        egui::Align2::CENTER_CENTER,
        "A",
        FontId::proportional(6.5 * zoom),
        Color32::from_rgb(255, 120, 120),
    );
    painter.text(
        cathode_pos + k_label_off,
        egui::Align2::CENTER_CENTER,
        "K",
        FontId::proportional(6.5 * zoom),
        Color32::from_rgb(120, 180, 255),
    );
}

/// Renders a single discrete LED component at the designated screen coordinates.
pub fn draw_single_led(
    painter: &Painter,
    center: Pos2,
    zoom: f32,
    rot: u16,
    led: &Led,
    is_selected: bool,
) {
    draw_led_leads(painter, center, zoom, rot);
    draw_led_body(painter, center, zoom, rot, led, is_selected);
    draw_led_terminals(painter, center, zoom, rot);
}

/// Renders right-click color preset picker buttons for a selected LED.
pub fn render_led_context_options(ui: &mut Ui, led: &mut Led) {
    let colors = [
        ("🔴 Red", LedColor::Red),
        ("🟢 Green", LedColor::Green),
        ("🔵 Blue", LedColor::Blue),
        ("🟡 Yellow", LedColor::Yellow),
        ("⚪ White", LedColor::White),
    ];
    ui.menu_button("🎨 Change Color", |ui| {
        for (name, color) in colors {
            if ui.button(name).clicked() {
                led.color = color;
                ui.close_menu();
            }
        }
    });
}
