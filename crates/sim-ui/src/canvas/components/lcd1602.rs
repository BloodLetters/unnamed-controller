//! Canvas renderer for HD44780 1602 LCD character display modules.

use egui::{Color32, FontId, Painter, Pos2, Rect, Stroke, Vec2};
use sim_components::component::{Lcd1602, LcdBacklightColor};

/// Module body width in canvas units.
pub const LCD1602_MODULE_WIDTH: f32 = 220.0;
/// Module body height in canvas units.
pub const LCD1602_MODULE_HEIGHT: f32 = 90.0;

/// Header terminal pin vertical pitch.
pub const LCD1602_PIN_PITCH: f32 = 14.0;
/// Horizontal offset from display center to header terminal column.
pub const LCD1602_PIN_OFFSET_X: f32 = -100.0;

/// Returns the 4 header pin offsets (GND, VCC, SDA, SCL) relative to module center.
pub fn lcd1602_pin_offsets() -> [(f32, f32); 4] {
    [
        (LCD1602_PIN_OFFSET_X, -1.5 * LCD1602_PIN_PITCH),
        (LCD1602_PIN_OFFSET_X, -0.5 * LCD1602_PIN_PITCH),
        (LCD1602_PIN_OFFSET_X, 0.5 * LCD1602_PIN_PITCH),
        (LCD1602_PIN_OFFSET_X, 1.5 * LCD1602_PIN_PITCH),
    ]
}

/// Renders the FR4 substrate, metallic bezel, and silkscreen labels.
fn draw_lcd_pcb(painter: &Painter, center: Pos2, zoom: f32, rot: u16, is_selected: bool) {
    let raw_size = if rot == 90 || rot == 270 {
        Vec2::new(LCD1602_MODULE_HEIGHT, LCD1602_MODULE_WIDTH)
    } else {
        Vec2::new(LCD1602_MODULE_WIDTH, LCD1602_MODULE_HEIGHT)
    };
    let pcb_rect = Rect::from_center_size(center, raw_size * zoom);

    let border_stroke = if is_selected {
        Stroke::new(2.0 * zoom, Color32::from_rgb(0, 229, 255))
    } else {
        Stroke::new(1.2 * zoom, Color32::from_rgb(30, 45, 60))
    };

    painter.rect(
        pcb_rect,
        6.0 * zoom,
        Color32::from_rgb(18, 48, 28),
        border_stroke,
    );

    let bezel_size = if rot == 90 || rot == 270 {
        Vec2::new(48.0, 168.0)
    } else {
        Vec2::new(168.0, 48.0)
    };
    let bezel_offset = crate::canvas::rotate_offset(Vec2::new(14.0, 0.0), rot);
    let bezel_rect = Rect::from_center_size(center + bezel_offset * zoom, bezel_size * zoom);
    painter.rect(
        bezel_rect,
        3.0 * zoom,
        Color32::from_rgb(24, 24, 26),
        Stroke::new(1.0 * zoom, Color32::from_rgb(50, 52, 58)),
    );

    draw_mounting_holes(painter, center, zoom, rot);
}

/// Renders corner mounting holes with copper annular rings.
fn draw_mounting_holes(painter: &Painter, center: Pos2, zoom: f32, rot: u16) {
    let hole_offsets = [
        Vec2::new(-98.0, -36.0),
        Vec2::new(98.0, -36.0),
        Vec2::new(-98.0, 36.0),
        Vec2::new(98.0, 36.0),
    ];
    for offset in hole_offsets {
        let rotated = crate::canvas::rotate_offset(offset, rot);
        let hole_center = center + rotated * zoom;
        painter.circle(
            hole_center,
            4.0 * zoom,
            Color32::from_rgb(160, 140, 60),
            Stroke::NONE,
        );
        painter.circle(
            hole_center,
            2.5 * zoom,
            Color32::from_rgb(20, 22, 26),
            Stroke::NONE,
        );
    }
}

/// Renders the 4-pin I2C backpack connector pads and silkscreen labels.
fn draw_lcd_header(painter: &Painter, center: Pos2, zoom: f32, rot: u16) {
    let pin_labels = ["GND", "VCC", "SDA", "SCL"];
    let offsets = lcd1602_pin_offsets();

    for (i, &(ox, oy)) in offsets.iter().enumerate() {
        let pin_pos = center + crate::canvas::rotate_offset(Vec2::new(ox, oy), rot) * zoom;
        let pad_radius = 4.2 * zoom;

        painter.circle(
            pin_pos,
            pad_radius,
            Color32::from_rgb(212, 175, 55),
            Stroke::new(0.8 * zoom, Color32::from_rgb(80, 70, 20)),
        );
        painter.circle(
            pin_pos,
            1.8 * zoom,
            Color32::from_rgb(15, 18, 22),
            Stroke::NONE,
        );

        let label_pos = center + crate::canvas::rotate_offset(Vec2::new(ox + 12.0, oy), rot) * zoom;
        painter.text(
            label_pos,
            egui::Align2::LEFT_CENTER,
            pin_labels[i],
            FontId::proportional(7.5 * zoom),
            Color32::from_rgb(210, 215, 220),
        );
    }
}

/// Renders the LCD dot-matrix glass panel and 16x2 text.
fn draw_lcd_screen(painter: &Painter, center: Pos2, zoom: f32, rot: u16, display: &Lcd1602) {
    let screen_size = if rot == 90 || rot == 270 {
        Vec2::new(40.0, 156.0)
    } else {
        Vec2::new(156.0, 40.0)
    };
    let screen_offset = crate::canvas::rotate_offset(Vec2::new(14.0, 0.0), rot);
    let screen_rect = Rect::from_center_size(center + screen_offset * zoom, screen_size * zoom);

    let (bg_color, fg_color) = if !display.is_backlight_on() {
        (Color32::from_rgb(20, 26, 32), Color32::from_rgb(40, 50, 60))
    } else {
        match display.backlight_color() {
            LcdBacklightColor::Blue => (
                Color32::from_rgb(10, 75, 190),
                Color32::from_rgb(230, 245, 255),
            ),
            LcdBacklightColor::Green => (
                Color32::from_rgb(125, 180, 25),
                Color32::from_rgb(25, 45, 10),
            ),
        }
    };

    painter.rect_filled(screen_rect, 2.0 * zoom, bg_color);

    if !display.is_display_on() {
        return;
    }

    let line1 = display.get_row_str(0);
    let line2 = display.get_row_str(1);

    let line1_offset = crate::canvas::rotate_offset(Vec2::new(14.0, -8.0), rot);
    let line2_offset = crate::canvas::rotate_offset(Vec2::new(14.0, 8.0), rot);

    painter.text(
        center + line1_offset * zoom,
        egui::Align2::CENTER_CENTER,
        &line1,
        FontId::monospace(13.0 * zoom),
        fg_color,
    );
    painter.text(
        center + line2_offset * zoom,
        egui::Align2::CENTER_CENTER,
        &line2,
        FontId::monospace(13.0 * zoom),
        fg_color,
    );
}

/// Renders a single 1602 LCD module at the designated screen coordinates.
pub fn draw_single_lcd1602(
    painter: &Painter,
    center: Pos2,
    zoom: f32,
    rot: u16,
    display: &sim_components::component::Lcd1602,
    is_selected: bool,
) {
    draw_lcd_pcb(painter, center, zoom, rot, is_selected);
    draw_lcd_header(painter, center, zoom, rot);
    draw_lcd_screen(painter, center, zoom, rot, display);
}
