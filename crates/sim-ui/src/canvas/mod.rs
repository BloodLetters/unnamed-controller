//! Circuit canvas rendering routines for the workbench grid, components, and wires.

pub mod boards;
pub mod components;
pub mod context_menu;
pub mod grid;
pub mod wires;

pub use boards::draw_esp32_s3_boards;
pub use grid::{draw_grid, draw_selection_box};
pub use wires::draw_wires;

/// Rotates a 2D offset vector around the origin by clockwise degree increments (0, 90, 180, 270).
pub fn rotate_offset(offset: egui::Vec2, degrees: u16) -> egui::Vec2 {
    match degrees % 360 {
        90 => egui::Vec2::new(-offset.y, offset.x),
        180 => egui::Vec2::new(-offset.x, -offset.y),
        270 => egui::Vec2::new(offset.y, -offset.x),
        _ => offset,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rotate_offset_all_quadrants() {
        let v = egui::Vec2::new(10.0, 20.0);
        assert_eq!(rotate_offset(v, 0), egui::Vec2::new(10.0, 20.0));
        assert_eq!(rotate_offset(v, 90), egui::Vec2::new(-20.0, 10.0));
        assert_eq!(rotate_offset(v, 180), egui::Vec2::new(-10.0, -20.0));
        assert_eq!(rotate_offset(v, 270), egui::Vec2::new(20.0, -10.0));
        assert_eq!(rotate_offset(v, 360), egui::Vec2::new(10.0, 20.0));
    }
}
