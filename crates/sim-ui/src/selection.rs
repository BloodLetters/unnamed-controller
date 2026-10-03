use egui::{Pos2, Rect};

use crate::app::SimulatorApp;
use crate::registry::CATALOG;
use crate::types::SelectedItem;
use crate::wire::{build_wire_path, distance_to_path};

/// Selects all components enclosed within the canvas selection bounding box.
pub fn select_items_in_rect(app: &mut SimulatorApp, box_rect: Rect) {
    let mut group = Vec::new();

    for spec in CATALOG {
        for index in 0..(spec.count)(app) {
            if let Some(position) = (spec.pos)(app, index)
                && box_rect.contains(position)
            {
                group.push((spec.make)(index));
            }
        }
    }

    if group.len() > 1 {
        app.selected = SelectedItem::Group(group);
    } else if group.len() == 1 {
        app.selected = group.remove(0);
    } else {
        app.selected = SelectedItem::None;
    }
}

/// Hit-tests schematic wires and selects the closest wire under the cursor.
pub fn select_wire_at_pos(app: &mut SimulatorApp, click_pos: Pos2, origin: Pos2) -> bool {
    for (i, wire) in app.wires.iter().enumerate() {
        let s_opt = app.get_pin_pos(wire.from);
        let e_opt = app.get_pin_pos(wire.to);
        if let (Some(s), Some(e)) = (s_opt, e_opt) {
            let s_screen = app.to_screen(s, origin);
            let e_screen = app.to_screen(e, origin);
            let waypoints: Vec<Pos2> = wire
                .waypoints
                .iter()
                .map(|wp| app.to_screen(Pos2::new(wp[0], wp[1]), origin))
                .collect();
            let path = build_wire_path(s_screen, &waypoints, e_screen);
            if distance_to_path(click_pos, &path) <= 6.0_f32 {
                app.selected = SelectedItem::Wire(i);
                return true;
            }
        }
    }
    false
}

/// Identifies the component body under the cursor, returning None if within pin snap range.
pub fn find_component_at(app: &SimulatorApp, canvas_pos: Pos2) -> Option<SelectedItem> {
    if app.find_closest_pin(canvas_pos).is_some() {
        return None;
    }

    for spec in CATALOG {
        for index in (0..(spec.count)(app)).rev() {
            let bounds = crate::registry::item_bounds(spec, app, index);
            if let Some(position) = (spec.pos)(app, index)
                && Rect::from_center_size(position, bounds).contains(canvas_pos)
            {
                return Some((spec.make)(index));
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::pos2;

    #[test]
    fn test_find_component_empty_canvas() {
        let app = SimulatorApp::default();
        assert_eq!(find_component_at(&app, pos2(100.0, 100.0)), None);
    }
}
