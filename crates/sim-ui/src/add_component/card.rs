//! Interactive grid card widget representing one placeable device.

use egui::{Color32, FontId, Pos2, Rect, Response, RichText, Stroke, Ui, Vec2};

use super::types::ComponentItem;

/// Renders a single device card with thumbnail, title, badge, and specifications.
pub fn render_component_card(ui: &mut Ui, item: &ComponentItem) -> Response {
    let desired_size = Vec2::new(176.0, 134.0);
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

    let is_hovered = response.hovered();
    let bg_color = if is_hovered {
        Color32::from_rgb(38, 42, 50)
    } else {
        Color32::from_rgb(28, 30, 36)
    };

    let border_stroke = if is_hovered {
        Stroke::new(1.5_f32, Color32::from_rgb(13, 153, 255))
    } else {
        Stroke::new(1.0_f32, Color32::from_rgb(44, 48, 58))
    };

    let painter = ui.painter();
    painter.rect(rect, 4.0, bg_color, border_stroke);

    let preview_rect = Rect::from_min_size(rect.min + Vec2::new(8.0, 8.0), Vec2::new(160.0, 52.0));
    painter.rect_filled(preview_rect, 3.0, Color32::from_rgb(20, 22, 26));
    draw_card_preview(ui, preview_rect, item);

    if let Some(badge) = item.badge {
        let (badge_bg, badge_fg) = match badge {
            "PRO" => (Color32::from_rgb(217, 119, 6), Color32::WHITE),
            "Ready" => (
                Color32::from_rgb(22, 101, 52),
                Color32::from_rgb(187, 247, 208),
            ),
            _ => (
                Color32::from_rgb(51, 65, 85),
                Color32::from_rgb(203, 213, 225),
            ),
        };
        let badge_pos = rect.min + Vec2::new(138.0, 14.0);
        painter.rect_filled(
            Rect::from_center_size(badge_pos, Vec2::new(34.0, 14.0)),
            2.0,
            badge_bg,
        );
        painter.text(
            badge_pos,
            egui::Align2::CENTER_CENTER,
            badge,
            FontId::proportional(8.5),
            badge_fg,
        );
    }

    let text_x = rect.min.x + 10.0;
    let title_y = rect.min.y + 70.0;
    let title_color = if item.is_ready {
        Color32::from_rgb(240, 242, 245)
    } else {
        Color32::from_rgb(170, 175, 185)
    };

    painter.text(
        Pos2::new(text_x, title_y),
        egui::Align2::LEFT_TOP,
        item.name,
        FontId::proportional(11.5),
        title_color,
    );

    let desc_rect = Rect::from_min_max(
        Pos2::new(text_x, title_y + 18.0),
        Pos2::new(rect.max.x - 8.0, rect.max.y - 6.0),
    );
    let mut child = ui.child_ui(desc_rect, egui::Layout::top_down(egui::Align::Min));
    child.label(
        RichText::new(item.specs)
            .size(9.0)
            .color(Color32::from_rgb(130, 138, 150)),
    );

    response
}

/// Draws an illustrative visual glyph or color preview for the item.
fn draw_card_preview(ui: &Ui, rect: Rect, item: &ComponentItem) {
    let painter = ui.painter();
    let center = rect.center();

    if item.id.starts_with("esp32") {
        painter.rect_filled(
            Rect::from_center_size(center, Vec2::new(32.0, 42.0)),
            2.0,
            Color32::from_rgb(30, 32, 38),
        );
        painter.rect_filled(
            Rect::from_center_size(center + Vec2::new(0.0, -8.0), Vec2::new(20.0, 16.0)),
            1.5,
            Color32::from_rgb(160, 165, 172),
        );
        painter.text(
            center + Vec2::new(0.0, 10.0),
            egui::Align2::CENTER_CENTER,
            "ESP32-S3",
            FontId::proportional(7.0),
            Color32::from_rgb(200, 205, 215),
        );
    } else if item.id.starts_with("led_") {
        let color = match item.id {
            "led_red" => Color32::from_rgb(239, 68, 68),
            "led_green" => Color32::from_rgb(34, 197, 94),
            "led_blue" => Color32::from_rgb(59, 130, 246),
            "led_yellow" => Color32::from_rgb(234, 179, 8),
            "led_white" => Color32::from_rgb(248, 250, 252),
            "led_orange" => Color32::from_rgb(249, 115, 22),
            _ => Color32::from_rgb(239, 68, 68),
        };
        painter.circle_filled(center, 9.0, color);
        painter.circle_stroke(center, 9.0, Stroke::new(1.0_f32, Color32::WHITE));
        painter.line_segment(
            [
                center + Vec2::new(-4.0, 9.0),
                center + Vec2::new(-4.0, 18.0),
            ],
            Stroke::new(1.5_f32, Color32::from_rgb(180, 185, 195)),
        );
        painter.line_segment(
            [center + Vec2::new(4.0, 9.0), center + Vec2::new(4.0, 18.0)],
            Stroke::new(1.5_f32, Color32::from_rgb(180, 185, 195)),
        );
    } else if item.id.starts_with("ssd1306") {
        painter.rect_filled(
            Rect::from_center_size(center, Vec2::new(44.0, 32.0)),
            2.0,
            Color32::from_rgb(18, 22, 30),
        );
        painter.rect_stroke(
            Rect::from_center_size(center, Vec2::new(44.0, 32.0)),
            2.0,
            Stroke::new(1.0_f32, Color32::from_rgb(45, 55, 72)),
        );
        painter.rect_filled(
            Rect::from_center_size(center + Vec2::new(0.0, 3.0), Vec2::new(34.0, 18.0)),
            1.0,
            Color32::from_rgb(5, 7, 10),
        );
        painter.text(
            center + Vec2::new(0.0, 3.0),
            egui::Align2::CENTER_CENTER,
            "128x64",
            FontId::monospace(6.5),
            Color32::from_rgb(0, 229, 255),
        );
    } else {
        painter.text(
            center,
            egui::Align2::CENTER_CENTER,
            item.name,
            FontId::proportional(9.0),
            Color32::from_rgb(140, 145, 155),
        );
    }
}
