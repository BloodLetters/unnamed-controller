use egui::{Color32, FontId, Painter, Pos2, Rect, Stroke, Vec2};
use sim_components::board::{BoardRgbLed, HeaderPin};

/// Renders a rounded PCB substrate with dark soldermask and selection border.
pub fn draw_pcb_substrate(
    painter: &Painter,
    center: Pos2,
    size: Vec2,
    corner_radius: f32,
    is_selected: bool,
    zoom: f32,
) {
    let pcb_rect = Rect::from_center_size(center, size * zoom);
    let border_stroke = if is_selected {
        Stroke::new(2.5 * zoom, Color32::from_rgb(0, 229, 255))
    } else {
        Stroke::new(1.0 * zoom, Color32::from_rgb(45, 50, 58))
    };

    painter.rect_filled(
        pcb_rect,
        corner_radius * zoom,
        Color32::from_rgb(22, 24, 28),
    );
    painter.rect_stroke(pcb_rect, corner_radius * zoom, border_stroke);
}

/// Renders plated mounting holes with gold annular rings and dark drill bores.
pub fn draw_mounting_holes(painter: &Painter, center: Pos2, offsets: &[Vec2], zoom: f32) {
    for offset in offsets {
        let hole_pos = center + *offset * zoom;
        painter.circle_filled(hole_pos, 3.2 * zoom, Color32::from_rgb(212, 168, 68));
        painter.circle_filled(hole_pos, 1.8 * zoom, Color32::from_rgb(10, 10, 12));
    }
}

/// Renders a surface-mount tactile pushbutton with a colored actuator plunger and label.
pub fn draw_tactile_button(
    painter: &Painter,
    pos: Pos2,
    zoom: f32,
    label: &str,
    is_pressed: bool,
    plunger_color: Color32,
) {
    let base_rect = Rect::from_center_size(pos, Vec2::new(12.0, 10.0) * zoom);
    painter.rect_filled(base_rect, 1.5 * zoom, Color32::from_rgb(145, 150, 158));

    let plunger_radius = if is_pressed { 2.5 } else { 3.2 } * zoom;
    painter.circle_filled(pos, plunger_radius, plunger_color);

    painter.text(
        pos + Vec2::new(0.0, -8.5) * zoom,
        egui::Align2::CENTER_CENTER,
        label,
        FontId::proportional(5.8 * zoom),
        Color32::from_rgb(175, 180, 190),
    );
}

/// Renders a monochromatic status indicator LED with glow when illuminated.
pub fn draw_status_led(
    painter: &Painter,
    pos: Pos2,
    zoom: f32,
    label: &str,
    is_on: bool,
    on_color: Color32,
    off_color: Color32,
) {
    let color = if is_on {
        painter.circle_filled(
            pos,
            4.5 * zoom,
            Color32::from_rgba_unmultiplied(on_color.r(), on_color.g(), on_color.b(), 70),
        );
        on_color
    } else {
        off_color
    };

    painter.circle_filled(pos, 2.5 * zoom, color);
    painter.text(
        pos + Vec2::new(0.0, -6.5) * zoom,
        egui::Align2::CENTER_CENTER,
        label,
        FontId::proportional(5.5 * zoom),
        Color32::from_rgb(150, 155, 165),
    );
}

/// Renders an onboard 5050/WS2812 addressable RGB LED package with light emission.
pub fn draw_rgb_led(painter: &Painter, pos: Pos2, zoom: f32, label: &str, rgb: BoardRgbLed) {
    let package_rect = Rect::from_center_size(pos, Vec2::new(8.0, 8.0) * zoom);
    painter.rect_filled(package_rect, 1.0 * zoom, Color32::from_rgb(40, 42, 48));

    let die_rect = Rect::from_center_size(pos, Vec2::new(5.0, 5.0) * zoom);
    if rgb.enabled {
        let color = Color32::from_rgb(rgb.r, rgb.g, rgb.b);
        painter.circle_filled(
            pos,
            6.0 * zoom,
            Color32::from_rgba_unmultiplied(rgb.r, rgb.g, rgb.b, 80),
        );
        painter.rect_filled(die_rect, 1.0 * zoom, color);
    } else {
        painter.rect_filled(die_rect, 1.0 * zoom, Color32::from_rgb(25, 26, 30));
    }

    painter.text(
        pos + Vec2::new(0.0, -6.5) * zoom,
        egui::Align2::CENTER_CENTER,
        label,
        FontId::proportional(5.5 * zoom),
        Color32::from_rgb(150, 155, 165),
    );
}

/// Renders a metallic USB receptacle connector with silkscreen port designation.
pub fn draw_usb_port(painter: &Painter, pos: Pos2, zoom: f32, label: &str) {
    let port_rect = Rect::from_center_size(pos, Vec2::new(16.0, 12.0) * zoom);
    painter.rect_filled(port_rect, 2.0 * zoom, Color32::from_rgb(140, 145, 152));
    painter.rect_filled(
        Rect::from_center_size(port_rect.center(), Vec2::new(9.0, 3.5) * zoom),
        1.0 * zoom,
        Color32::from_rgb(25, 26, 30),
    );
    painter.text(
        pos + Vec2::new(0.0, -10.0) * zoom,
        egui::Align2::CENTER_CENTER,
        label,
        FontId::proportional(6.0 * zoom),
        Color32::from_rgb(170, 175, 185),
    );
}

/// Renders a single DIP header terminal with annular copper pad and silkscreen label.
pub fn draw_header_terminal(
    painter: &Painter,
    pad_center: Pos2,
    zoom: f32,
    label_pos: Pos2,
    text_align: egui::Align2,
    pin: &HeaderPin,
) {
    painter.circle_filled(pad_center, 3.5 * zoom, Color32::from_rgb(212, 168, 68));
    painter.circle_filled(pad_center, 1.8 * zoom, Color32::from_rgb(12, 14, 18));

    let text_color = if pin.is_power {
        Color32::from_rgb(255, 110, 110)
    } else if pin.is_ground {
        Color32::from_rgb(105, 180, 255)
    } else if pin.name == "RST" {
        Color32::from_rgb(255, 175, 75)
    } else {
        Color32::from_rgb(220, 225, 235)
    };

    painter.text(
        label_pos,
        text_align,
        &pin.name,
        FontId::proportional(7.2 * zoom),
        text_color,
    );
}
