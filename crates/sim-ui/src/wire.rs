use egui::{Color32, FontId, Painter, Pos2, Rect, Stroke, Vec2};

/// Constructs the screen-coordinate polyline for wire endpoints and intermediate waypoints.
pub fn build_wire_path(start: Pos2, waypoints: &[Pos2], end: Pos2) -> Vec<Pos2> {
    let mut path = Vec::with_capacity(waypoints.len() + 2);
    path.push(start);
    path.extend_from_slice(waypoints);
    path.push(end);
    path
}

/// Computes an orthogonal 90-degree routing path between two canvas coordinates.
pub fn calculate_orthogonal_path(start: Pos2, end: Pos2) -> Vec<Pos2> {
    let dx = (end.x - start.x).abs();
    let dy = (end.y - start.y).abs();

    if dx < 2.0_f32 || dy < 2.0_f32 {
        vec![start, end]
    } else if dx >= dy {
        let mid_x = (start.x + end.x) * 0.5_f32;
        vec![
            start,
            Pos2::new(mid_x, start.y),
            Pos2::new(mid_x, end.y),
            end,
        ]
    } else {
        let mid_y = (start.y + end.y) * 0.5_f32;
        vec![
            start,
            Pos2::new(start.x, mid_y),
            Pos2::new(end.x, mid_y),
            end,
        ]
    }
}

/// Calculates distance from a point to a line segment.
fn distance_to_segment(p: Pos2, a: Pos2, b: Pos2) -> f32 {
    let ab = b - a;
    let len_sq = ab.length_sq();
    if len_sq == 0.0_f32 {
        return p.distance(a);
    }
    let t = ((p - a).dot(ab) / len_sq).clamp(0.0_f32, 1.0_f32);
    let projection = a + ab * t;
    p.distance(projection)
}

/// Calculates the minimum distance from a click coordinate to an orthogonal wire path.
pub fn distance_to_path(point: Pos2, path: &[Pos2]) -> f32 {
    let mut min_d = f32::MAX;
    for window in path.windows(2) {
        let d = distance_to_segment(point, window[0], window[1]);
        if d < min_d {
            min_d = d;
        }
    }
    min_d
}

/// Renders an orthogonal wire path with optional label and selection highlight.
pub fn render_wire_path(
    painter: &Painter,
    path: &[Pos2],
    color: Color32,
    is_selected: bool,
    label: &str,
) {
    if path.len() < 2 {
        return;
    }

    let wire_color = if is_selected {
        Color32::from_rgb(255, 235, 59)
    } else {
        color
    };
    let stroke_width = if is_selected { 3.0_f32 } else { 2.0_f32 };
    let stroke = Stroke::new(stroke_width, wire_color);

    for window in path.windows(2) {
        painter.line_segment([window[0], window[1]], stroke);
    }

    if !label.is_empty() {
        let mid_idx = path.len() / 2;
        let mid_pt = path[mid_idx];
        let label_rect = Rect::from_center_size(
            mid_pt,
            Vec2::new((label.len() as f32 * 6.5_f32).max(24.0_f32), 14.0_f32),
        );
        painter.rect_filled(label_rect, 2.0_f32, Color32::from_rgb(24, 26, 32));
        painter.rect_stroke(label_rect, 2.0_f32, Stroke::new(1.0_f32, wire_color));
        painter.text(
            mid_pt,
            egui::Align2::CENTER_CENTER,
            label,
            FontId::monospace(9.0_f32),
            Color32::WHITE,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orthogonal_path_horizontal_dominant() {
        let s = Pos2::new(0.0, 0.0);
        let e = Pos2::new(100.0, 40.0);
        let path = calculate_orthogonal_path(s, e);
        assert_eq!(path.len(), 4);
        assert_eq!(path[1], Pos2::new(50.0, 0.0));
        assert_eq!(path[2], Pos2::new(50.0, 40.0));
    }

    #[test]
    fn test_orthogonal_path_vertical_dominant() {
        let s = Pos2::new(0.0, 0.0);
        let e = Pos2::new(40.0, 100.0);
        let path = calculate_orthogonal_path(s, e);
        assert_eq!(path.len(), 4);
        assert_eq!(path[1], Pos2::new(0.0, 50.0));
        assert_eq!(path[2], Pos2::new(40.0, 50.0));
    }

    #[test]
    fn test_straight_line() {
        let s = Pos2::new(10.0, 10.0);
        let e = Pos2::new(10.0, 100.0);
        let path = calculate_orthogonal_path(s, e);
        assert_eq!(path.len(), 2);
    }

    #[test]
    fn test_distance_to_path() {
        let path = vec![
            Pos2::new(0.0, 0.0),
            Pos2::new(50.0, 0.0),
            Pos2::new(50.0, 50.0),
        ];
        assert_eq!(distance_to_path(Pos2::new(25.0, 2.0), &path), 2.0);
        assert_eq!(distance_to_path(Pos2::new(52.0, 25.0), &path), 2.0);
    }

    #[test]
    fn test_build_wire_path() {
        let s = Pos2::new(0.0, 0.0);
        let wp = vec![Pos2::new(20.0, 0.0), Pos2::new(20.0, 50.0)];
        let e = Pos2::new(100.0, 50.0);
        let path = build_wire_path(s, &wp, e);
        assert_eq!(path.len(), 4);
        assert_eq!(path[0], s);
        assert_eq!(path[1], wp[0]);
        assert_eq!(path[2], wp[1]);
        assert_eq!(path[3], e);
    }
}
