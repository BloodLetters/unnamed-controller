//! Canvas renderer for 6x6mm 4-pin tactile pushbutton switches.

use egui::{Color32, FontId, Painter, Pos2, Rect, Stroke, Ui, Vec2};
use sim_components::component::{Button, ButtonColor};

/// Horizontal distance from switch center to terminal solder leads.
pub const BUTTON_PIN_OFFSET_X: f32 = 20.0;
/// Vertical distance from switch center to upper and lower terminal pairs.
pub const BUTTON_PIN_OFFSET_Y: f32 = 8.0;

/// Returns relative terminal offsets for [pin_1a, pin_1b, pin_2a, pin_2b].
pub fn button_pin_offsets() -> [(f32, f32); 4] {
    [
        (-BUTTON_PIN_OFFSET_X, -BUTTON_PIN_OFFSET_Y),
        (BUTTON_PIN_OFFSET_X, -BUTTON_PIN_OFFSET_Y),
        (-BUTTON_PIN_OFFSET_X, BUTTON_PIN_OFFSET_Y),
        (BUTTON_PIN_OFFSET_X, BUTTON_PIN_OFFSET_Y),
    ]
}

/// Returns RGB color tuple for the specified button cap color and state.
fn cap_rgb(color: ButtonColor, is_pressed: bool) -> (Color32, Color32) {
    match (color, is_pressed) {
        (ButtonColor::Blue, false) => (
            Color32::from_rgb(37, 99, 235),
            Color32::from_rgb(96, 165, 250),
        ),
        (ButtonColor::Blue, true) => (
            Color32::from_rgb(29, 78, 216),
            Color32::from_rgb(59, 130, 246),
        ),
        (ButtonColor::Red, false) => (
            Color32::from_rgb(220, 38, 38),
            Color32::from_rgb(248, 113, 113),
        ),
        (ButtonColor::Red, true) => (
            Color32::from_rgb(185, 28, 28),
            Color32::from_rgb(239, 68, 68),
        ),
        (ButtonColor::Green, false) => (
            Color32::from_rgb(22, 163, 74),
            Color32::from_rgb(74, 222, 128),
        ),
        (ButtonColor::Green, true) => (
            Color32::from_rgb(21, 128, 61),
            Color32::from_rgb(34, 197, 94),
        ),
        (ButtonColor::Yellow, false) => (
            Color32::from_rgb(234, 179, 8),
            Color32::from_rgb(253, 224, 71),
        ),
        (ButtonColor::Yellow, true) => (
            Color32::from_rgb(202, 138, 4),
            Color32::from_rgb(234, 179, 8),
        ),
        (ButtonColor::Black, false) => {
            (Color32::from_rgb(30, 32, 37), Color32::from_rgb(55, 65, 81))
        }
        (ButtonColor::Black, true) => {
            (Color32::from_rgb(17, 20, 24), Color32::from_rgb(40, 45, 55))
        }
        (ButtonColor::White, false) => (
            Color32::from_rgb(226, 232, 240),
            Color32::from_rgb(255, 255, 255),
        ),
        (ButtonColor::White, true) => (
            Color32::from_rgb(203, 213, 225),
            Color32::from_rgb(241, 245, 249),
        ),
    }
}

/// Renders the four solder terminals and metallic lead legs.
fn draw_button_leads(painter: &Painter, center: Pos2, zoom: f32, rot: u16) {
    let lead_stroke = Stroke::new(1.8 * zoom, Color32::from_rgb(170, 175, 185));
    let pad_gold = Color32::from_rgb(212, 168, 68);
    let hole_dark = Color32::from_rgb(12, 14, 18);

    for (ox, oy) in button_pin_offsets() {
        let pad_pos = center + crate::canvas::rotate_offset(Vec2::new(ox, oy), rot) * zoom;
        let inner_x = if ox < 0.0 { -13.0 } else { 13.0 };
        let body_edge = center + crate::canvas::rotate_offset(Vec2::new(inner_x, oy), rot) * zoom;

        painter.line_segment([body_edge, pad_pos], lead_stroke);
        painter.circle_filled(pad_pos, 3.2 * zoom, pad_gold);
        painter.circle_filled(pad_pos, 1.5 * zoom, hole_dark);
    }
}

/// Renders the metallic tactile switch body, corner rivets, and cylindrical plunger cap.
fn draw_button_body(
    painter: &Painter,
    center: Pos2,
    zoom: f32,
    rot: u16,
    button: &Button,
    is_selected: bool,
) {
    let body_half = 14.0 * zoom;
    let body_rect = Rect::from_center_size(center, Vec2::splat(body_half * 2.0));
    let border_stroke = if is_selected {
        Stroke::new(2.0 * zoom, Color32::from_rgb(0, 229, 255))
    } else {
        Stroke::new(1.2 * zoom, Color32::from_rgb(45, 49, 58))
    };

    painter.rect_filled(body_rect, 3.0 * zoom, Color32::from_rgb(32, 35, 42));
    painter.rect_stroke(body_rect, 3.0 * zoom, border_stroke);

    let rivet_color = Color32::from_rgb(130, 138, 150);
    let rivet_dist = 10.0;
    for (rx, ry) in [
        (-rivet_dist, -rivet_dist),
        (rivet_dist, -rivet_dist),
        (-rivet_dist, rivet_dist),
        (rivet_dist, rivet_dist),
    ] {
        let rivet_pos = center + crate::canvas::rotate_offset(Vec2::new(rx, ry), rot) * zoom;
        painter.circle_filled(rivet_pos, 1.4 * zoom, rivet_color);
    }

    let bezel_radius = 10.5 * zoom;
    painter.circle_filled(center, bezel_radius, Color32::from_rgb(22, 24, 28));
    painter.circle_stroke(
        center,
        bezel_radius,
        Stroke::new(1.0 * zoom, Color32::from_rgb(70, 75, 85)),
    );

    let is_pressed = button.is_pressed();
    let plunger_radius = if is_pressed { 7.5 * zoom } else { 9.0 * zoom };
    let (cap_main, cap_highlight) = cap_rgb(button.color(), is_pressed);

    if is_pressed {
        painter.circle_filled(
            center,
            plunger_radius + 1.5 * zoom,
            Color32::from_rgba_unmultiplied(0, 229, 255, 60),
        );
    }

    painter.circle_filled(center, plunger_radius, cap_main);
    painter.circle_stroke(
        center,
        plunger_radius,
        Stroke::new(0.8 * zoom, Color32::from_black_alpha(80)),
    );

    let highlight_off = crate::canvas::rotate_offset(Vec2::new(-2.0, -2.0), rot) * zoom;
    let hl_radius = if is_pressed { 2.0 * zoom } else { 2.8 * zoom };
    painter.circle_filled(
        center + highlight_off,
        hl_radius,
        Color32::from_rgba_unmultiplied(
            cap_highlight.r(),
            cap_highlight.g(),
            cap_highlight.b(),
            if is_pressed { 120 } else { 180 },
        ),
    );
}

/// Renders silkscreen pin group identifiers.
fn draw_button_silkscreen(painter: &Painter, center: Pos2, zoom: f32, rot: u16) {
    let font = FontId::proportional(6.0 * zoom);
    let label_color = Color32::from_rgb(140, 148, 160);

    let top_label_pos = center + crate::canvas::rotate_offset(Vec2::new(0.0, -10.5), rot) * zoom;
    let btm_label_pos = center + crate::canvas::rotate_offset(Vec2::new(0.0, 10.5), rot) * zoom;

    painter.text(
        top_label_pos,
        egui::Align2::CENTER_CENTER,
        "1",
        font.clone(),
        label_color,
    );
    painter.text(
        btm_label_pos,
        egui::Align2::CENTER_CENTER,
        "2",
        font,
        label_color,
    );
}

/// Renders a single discrete tactile pushbutton switch at canvas coordinates.
pub fn draw_single_button(
    painter: &Painter,
    center: Pos2,
    zoom: f32,
    rot: u16,
    button: &Button,
    is_selected: bool,
) {
    draw_button_leads(painter, center, zoom, rot);
    draw_button_body(painter, center, zoom, rot, button, is_selected);
    draw_button_silkscreen(painter, center, zoom, rot);
}

/// Renders context menu quick-action entries for the tactile button.
pub fn render_button_context_options(ui: &mut Ui, button: &mut Button) {
    let toggle_label = if button.is_pressed() {
        "⬆ Release Button"
    } else {
        "⬇ Press Button"
    };
    if ui.button(toggle_label).clicked() {
        button.toggle();
        ui.close_menu();
    }

    let latch_label = if button.is_latching() {
        "⚙ Switch to Momentary"
    } else {
        "⚙ Switch to Latching"
    };
    if ui.button(latch_label).clicked() {
        button.set_latching(!button.is_latching());
        ui.close_menu();
    }

    ui.menu_button("🎨 Change Cap Color", |ui| {
        for color in ButtonColor::ALL {
            if ui.button(color.label()).clicked() {
                button.set_color(color);
                ui.close_menu();
            }
        }
    });
}
