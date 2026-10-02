use egui::{Color32, Painter, Pos2, Rect, Stroke, Vec2};

/// Draws the scalable schematic grid on the circuit canvas.
pub fn draw_grid(painter: &Painter, rect: Rect, pan: Vec2, zoom: f32) {
    let spacing = 20.0_f32 * zoom;
    let offset_x = (pan.x % spacing + spacing) % spacing;
    let offset_y = (pan.y % spacing + spacing) % spacing;

    let mut x = rect.min.x + offset_x;
    while x < rect.max.x {
        painter.line_segment(
            [Pos2::new(x, rect.min.y), Pos2::new(x, rect.max.y)],
            Stroke::new(1.0_f32, Color32::from_gray(38)),
        );
        x += spacing;
    }
    let mut y = rect.min.y + offset_y;
    while y < rect.max.y {
        painter.line_segment(
            [Pos2::new(rect.min.x, y), Pos2::new(rect.max.x, y)],
            Stroke::new(1.0_f32, Color32::from_gray(38)),
        );
        y += spacing;
    }
}

/// Renders the multi-selection bounding box on canvas.
pub fn draw_selection_box(painter: &Painter, rect: Rect) {
    painter.rect_filled(
        rect,
        0.0_f32,
        Color32::from_rgba_unmultiplied(33, 150, 243, 30),
    );
    painter.rect_stroke(
        rect,
        0.0_f32,
        Stroke::new(1.0_f32, Color32::from_rgb(33, 150, 243)),
    );
}
