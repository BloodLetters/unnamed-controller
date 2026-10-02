use egui::{Color32, Painter, Pos2};

use crate::app::SimulatorApp;
use crate::types::SelectedItem;
use crate::wire::{calculate_orthogonal_path, render_wire_path};

/// Renders all schematic wires with orthogonal paths, labels, and selection highlights.
pub fn draw_wires(painter: &Painter, origin: Pos2, app: &SimulatorApp) {
    for (i, wire) in app.wires.iter().enumerate() {
        let s_opt = app.get_pin_pos(wire.from);
        let e_opt = app.get_pin_pos(wire.to);
        if let (Some(s_pos), Some(e_pos)) = (s_opt, e_opt) {
            let s_screen = app.to_screen(s_pos, origin);
            let e_screen = app.to_screen(e_pos, origin);
            let path = calculate_orthogonal_path(s_screen, e_screen);
            let is_sel = app.selected == SelectedItem::Wire(i);
            let color = Color32::from_rgb(wire.color[0], wire.color[1], wire.color[2]);
            render_wire_path(painter, &path, color, is_sel, &wire.label);
        }
    }
}
