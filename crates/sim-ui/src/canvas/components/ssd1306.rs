//! Canvas renderer for 0.96" SSD1306 OLED display modules.

use egui::{Color32, FontId, Painter, Pos2, Rect, Stroke, Vec2};
use sim_components::component::{SSD1306_HEIGHT, SSD1306_WIDTH, Ssd1306};

/// Module dimensions in canvas units.
pub const SSD1306_MODULE_WIDTH: f32 = 110.0;
pub const SSD1306_MODULE_HEIGHT: f32 = 95.0;

/// Header terminal pin horizontal pitch.
pub const SSD1306_PIN_PITCH: f32 = 14.0;
/// Vertical offset from display center to header terminal row.
pub const SSD1306_PIN_OFFSET_Y: f32 = -38.0;

/// Returns the 4 header pin offsets (GND, VCC, SDA, SCL) relative to module center.
pub fn ssd1306_pin_offsets() -> [(f32, f32); 4] {
    [
        (-1.5 * SSD1306_PIN_PITCH, SSD1306_PIN_OFFSET_Y),
        (-0.5 * SSD1306_PIN_PITCH, SSD1306_PIN_OFFSET_Y),
        (0.5 * SSD1306_PIN_PITCH, SSD1306_PIN_OFFSET_Y),
        (1.5 * SSD1306_PIN_PITCH, SSD1306_PIN_OFFSET_Y),
    ]
}

/// Renders the FR4 PCB substrate, mounting holes, and silkscreen labels.
fn draw_display_pcb(painter: &Painter, center: Pos2, zoom: f32, rot: u16, is_selected: bool) {
    let raw_size = if rot == 90 || rot == 270 {
        Vec2::new(SSD1306_MODULE_HEIGHT, SSD1306_MODULE_WIDTH)
    } else {
        Vec2::new(SSD1306_MODULE_WIDTH, SSD1306_MODULE_HEIGHT)
    };
    let pcb_rect = Rect::from_center_size(center, raw_size * zoom);

    let border_stroke = if is_selected {
        Stroke::new(2.0 * zoom, Color32::from_rgb(0, 229, 255))
    } else {
        Stroke::new(1.2 * zoom, Color32::from_rgb(45, 55, 72))
    };

    painter.rect_filled(pcb_rect, 4.0 * zoom, Color32::from_rgb(18, 22, 30));
    painter.rect_stroke(pcb_rect, 4.0 * zoom, border_stroke);

    let hole_offsets = [
        Vec2::new(-48.0, -40.0),
        Vec2::new(48.0, -40.0),
        Vec2::new(-48.0, 40.0),
        Vec2::new(48.0, 40.0),
    ];
    for offset in hole_offsets {
        let rotated_hole = crate::canvas::rotate_offset(offset, rot);
        let hole_pos = center + rotated_hole * zoom;
        painter.circle_filled(hole_pos, 3.2 * zoom, Color32::from_rgb(212, 168, 68));
        painter.circle_filled(hole_pos, 1.8 * zoom, Color32::from_rgb(10, 12, 16));
    }
}

/// Renders through-hole solder pads with silkscreen pin labels.
fn draw_display_header(painter: &Painter, center: Pos2, zoom: f32, rot: u16) {
    let pin_labels = ["GND", "VCC", "SDA", "SCL"];
    let pin_colors = [
        Color32::from_rgb(130, 138, 150),
        Color32::from_rgb(248, 113, 113),
        Color32::from_rgb(56, 189, 248),
        Color32::from_rgb(250, 204, 21),
    ];

    let offsets = ssd1306_pin_offsets();
    for (i, &(ox, oy)) in offsets.iter().enumerate() {
        let rotated_pin = crate::canvas::rotate_offset(Vec2::new(ox, oy), rot);
        let pin_pos = center + rotated_pin * zoom;

        painter.circle_filled(pin_pos, 3.2 * zoom, Color32::from_rgb(212, 168, 68));
        painter.circle_filled(pin_pos, 1.6 * zoom, Color32::from_rgb(12, 14, 18));

        let label_offset = crate::canvas::rotate_offset(Vec2::new(0.0, 6.5), rot) * zoom;
        painter.text(
            pin_pos + label_offset,
            egui::Align2::CENTER_CENTER,
            pin_labels[i],
            FontId::proportional(5.8 * zoom),
            pin_colors[i],
        );
    }
}

/// Renders the glass OLED panel, frame bezel, and active GDDRAM pixel matrix.
fn draw_oled_screen(painter: &Painter, center: Pos2, zoom: f32, rot: u16, display: &Ssd1306) {
    let (screen_w, screen_h) = if rot == 90 || rot == 270 {
        (44.0 * zoom, 88.0 * zoom)
    } else {
        (88.0 * zoom, 44.0 * zoom)
    };
    let screen_offset = crate::canvas::rotate_offset(Vec2::new(0.0, 15.0), rot) * zoom;
    let screen_center = center + screen_offset;
    let screen_rect = Rect::from_center_size(screen_center, Vec2::new(screen_w, screen_h));

    painter.rect_filled(screen_rect, 1.5 * zoom, Color32::from_rgb(5, 7, 10));
    painter.rect_stroke(
        screen_rect,
        1.5 * zoom,
        Stroke::new(1.0 * zoom, Color32::from_rgb(32, 40, 54)),
    );

    if !display.is_display_on() {
        return;
    }

    draw_display_glow(painter, screen_rect, zoom);
    draw_screen_pixels(painter, screen_center, zoom, rot, display);
}

/// Renders the ambient OLED glass glow when display emission is active.
fn draw_display_glow(painter: &Painter, rect: Rect, zoom: f32) {
    painter.rect_filled(
        rect.expand(1.5 * zoom),
        2.0 * zoom,
        Color32::from_rgba_unmultiplied(0, 229, 255, 8),
    );
}

/// Iterates through GDDRAM content to draw illuminated pixels.
fn draw_screen_pixels(
    painter: &Painter,
    screen_center: Pos2,
    zoom: f32,
    rot: u16,
    display: &Ssd1306,
) {
    let lit_color = Color32::from_rgb(0, 229, 255);
    let dot_w = (88.0 * zoom / SSD1306_WIDTH as f32).max(1.0);
    let dot_h = (44.0 * zoom / SSD1306_HEIGHT as f32).max(1.0);
    let dot_size = if rot == 90 || rot == 270 {
        Vec2::new(dot_h, dot_w)
    } else {
        Vec2::new(dot_w, dot_h)
    };

    for y in 0..SSD1306_HEIGHT {
        for x in 0..SSD1306_WIDTH {
            if display.pixel_at(x, y) {
                let off_x = (x as f32 - 63.5) / SSD1306_WIDTH as f32 * 88.0;
                let off_y = (y as f32 - 31.5) / SSD1306_HEIGHT as f32 * 44.0;
                let rotated_off = crate::canvas::rotate_offset(Vec2::new(off_x, off_y), rot);
                let pixel_center = screen_center + rotated_off * zoom;
                painter.rect_filled(
                    Rect::from_center_size(pixel_center, dot_size),
                    0.0,
                    lit_color,
                );
            }
        }
    }
}

/// Renders a single SSD1306 OLED display module at the designated screen coordinates.
pub fn draw_single_ssd1306(
    painter: &Painter,
    center: Pos2,
    zoom: f32,
    rot: u16,
    display: &Ssd1306,
    is_selected: bool,
) {
    draw_display_pcb(painter, center, zoom, rot, is_selected);
    draw_display_header(painter, center, zoom, rot);
    draw_oled_screen(painter, center, zoom, rot, display);
}
