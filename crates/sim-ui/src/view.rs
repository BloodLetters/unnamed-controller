//! Central circuit schematic canvas rendering and pointer interaction handlers.

use egui::{Color32, Rect, RichText, Stroke, Vec2};

use crate::app::SimulatorApp;
use crate::canvas::{draw_grid, draw_selection_box, draw_wires};
use crate::selection::{find_component_at, select_items_in_rect, select_wire_at_pos};
use crate::types::{SelectedItem, SpawningComponent, Wire};

/// Renders the central circuit canvas with pan, zoom, grid, components, and wire interactions.
pub fn render_central_canvas(ctx: &egui::Context, app: &mut SimulatorApp) {
    egui::CentralPanel::default().show(ctx, |ui| {
        if ctx.input(|i| i.key_pressed(egui::Key::Delete)) {
            app.delete_selected();
        }
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Z)) {
            app.undo();
        }
        if ctx.input(|i| {
            (i.modifiers.command && i.key_pressed(egui::Key::Y))
                || (i.modifiers.command && i.modifiers.shift && i.key_pressed(egui::Key::Z))
        }) {
            app.redo();
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            app.spawning = SpawningComponent::None;
            app.drawing_wire = None;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::R) && !i.modifiers.command) {
            let delta = if ctx.input(|i| i.modifiers.shift) {
                -90
            } else {
                90
            };
            app.rotate_selected(delta);
        }
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::D)) {
            app.duplicate_selected();
        }

        let (response, painter) =
            ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());
        let rect = response.rect;
        let origin = rect.min;

        draw_grid(&painter, rect, app.pan, app.zoom);

        crate::canvas::draw_esp32_s3_boards(&painter, origin, app);
        for (idx, comp) in app.components.iter().enumerate() {
            let is_selected = app.selected.contains_item(&SelectedItem::Component(idx));
            comp.draw(&painter, origin, app.pan, app.zoom, is_selected);
        }

        draw_wires(&painter, origin, app);

        let hover_pos = response.hover_pos();
        let hover_canvas = hover_pos.map(|p| app.to_canvas(p, origin));
        let hovered_pin = hover_canvas.and_then(|p| app.find_closest_pin(p));

        if hovered_pin.is_some() {
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Crosshair);
        } else if app.dragging_item || response.dragged_by(egui::PointerButton::Primary) {
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Grabbing);
        } else if hover_canvas.is_some_and(|p| find_component_at(app, p).is_some()) {
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Grab);
        } else if response.hovered() {
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Grab);
        }

        if let Some(pos) = response
            .interact_pointer_pos()
            .filter(|_| response.clicked() && app.spawning != SpawningComponent::None)
        {
            app.record_history();
            let canvas_pos = app.to_canvas(pos, rect.min);
            app.spawn_component_at(canvas_pos);
        }

        if let Some(start_pin) = app.drawing_wire {
            if let Some(start_pos) = app.get_pin_pos(start_pin) {
                let s_screen = app.to_screen(start_pos, rect.min);
                let end_screen = if let Some(target_pin) = hovered_pin.filter(|&p| p != start_pin) {
                    app.get_pin_pos(target_pin)
                        .map(|p| app.to_screen(p, rect.min))
                        .unwrap_or_else(|| hover_pos.unwrap_or(s_screen))
                } else {
                    hover_pos.unwrap_or(s_screen)
                };
                let path = crate::wire::calculate_orthogonal_path(s_screen, end_screen);
                crate::wire::render_wire_path(&painter, &path, Color32::YELLOW, true, "");
            }

            if response.drag_stopped_by(egui::PointerButton::Primary)
                && let Some(end_pin) = hovered_pin.filter(|&ep| ep != start_pin)
            {
                app.record_history();
                app.wires.push(Wire::new(start_pin, end_pin));
                app.rebuild_netlist();
                app.drawing_wire = None;
            }

            if response.clicked_by(egui::PointerButton::Primary) {
                if let Some(end_pin) = hovered_pin.filter(|&ep| ep != start_pin) {
                    app.record_history();
                    app.wires.push(Wire::new(start_pin, end_pin));
                    app.rebuild_netlist();
                    app.drawing_wire = None;
                } else if hovered_pin.is_none() {
                    app.drawing_wire = None;
                }
            }
        } else if app.spawning == SpawningComponent::None {
            let pointer_canvas = response
                .interact_pointer_pos()
                .map(|p| app.to_canvas(p, rect.min));
            let clicked = response.clicked_by(egui::PointerButton::Primary);
            let secondary_clicked = response.secondary_clicked();
            let drag_started = response.drag_started_by(egui::PointerButton::Primary);
            let shift_held = ui.input(|i| i.modifiers.shift);

            if clicked || secondary_clicked {
                if let Some(pos) = pointer_canvas {
                    if let Some(comp) = find_component_at(app, pos) {
                        if !app.selected.contains_item(&comp) {
                            app.selected = comp;
                        }
                    } else if let Some(click_pos) = response.interact_pointer_pos()
                        && select_wire_at_pos(app, click_pos, origin)
                    {
                    } else if clicked {
                        app.selected = SelectedItem::None;
                    }
                }
            } else if drag_started {
                if let Some(pin) = hovered_pin {
                    app.drawing_wire = Some(pin);
                } else if let Some(comp) = pointer_canvas.and_then(|p| find_component_at(app, p)) {
                    if !app.selected.contains_item(&comp) {
                        app.selected = comp;
                    }
                    app.dragging_item = true;
                    app.record_history();
                } else if shift_held {
                    app.selected = SelectedItem::None;
                    app.box_select_start = response.interact_pointer_pos();
                    app.box_select_current = response.interact_pointer_pos();
                }
            }

            if response.dragged_by(egui::PointerButton::Primary) {
                if app.dragging_item {
                    app.move_item_or_group(app.selected.clone(), response.drag_delta() / app.zoom);
                } else if app.box_select_start.is_some() {
                    app.box_select_current = response.interact_pointer_pos();
                } else if app.drawing_wire.is_none() {
                    app.pan += response.drag_delta();
                }
            }

            if response.drag_stopped_by(egui::PointerButton::Primary) {
                app.dragging_item = false;
            }

            if let (Some(start), Some(curr)) = (app.box_select_start, app.box_select_current) {
                draw_selection_box(&painter, Rect::from_two_pos(start, curr));
            }
            if response.drag_stopped_by(egui::PointerButton::Primary)
                && let (Some(start), Some(curr)) = (app.box_select_start, app.box_select_current)
            {
                let s_c = app.to_canvas(start, rect.min);
                let c_c = app.to_canvas(curr, rect.min);
                let box_rect = Rect::from_two_pos(s_c, c_c);
                if box_rect.width() > 5.0 || box_rect.height() > 5.0 {
                    select_items_in_rect(app, box_rect);
                }
                app.box_select_start = None;
                app.box_select_current = None;
            }
        }

        if response.dragged_by(egui::PointerButton::Secondary)
            || response.dragged_by(egui::PointerButton::Middle)
        {
            app.pan += response.drag_delta();
        }

        let scroll_delta = ui.input(|i| i.raw_scroll_delta.y);
        if scroll_delta != 0.0 && !app.show_add_component && response.hovered() {
            let zoom_factor = (scroll_delta * 0.0015).exp();
            let new_zoom = (app.zoom * zoom_factor).clamp(0.2, 5.0);
            if let Some(pos) = hover_pos {
                let pointer_canvas = app.to_canvas(pos, origin);
                app.zoom = new_zoom;
                let new_screen = app.to_screen(pointer_canvas, origin);
                app.pan += pos - new_screen;
            } else {
                app.zoom = new_zoom;
            }
        }

        render_floating_zoom_control(ui, rect, app);
        render_spawning_banner(ui, rect, app);

        response.context_menu(|ui| {
            crate::canvas::context_menu::render_canvas_context_menu(ui, app);
        });
    });
}

/// Renders the floating bottom-right zoom control widget.
fn render_floating_zoom_control(ui: &mut egui::Ui, rect: Rect, app: &mut SimulatorApp) {
    let zoom_rect = Rect::from_min_size(rect.max - Vec2::new(140.0, 36.0), Vec2::new(130.0, 26.0));
    let painter = ui.painter();
    painter.rect_filled(
        zoom_rect,
        4.0,
        Color32::from_rgba_premultiplied(24, 26, 32, 230),
    );
    painter.rect_stroke(
        zoom_rect,
        4.0,
        Stroke::new(1.0_f32, Color32::from_rgb(45, 48, 56)),
    );

    let mut zoom_ui = ui.child_ui(zoom_rect, egui::Layout::left_to_right(egui::Align::Center));
    zoom_ui.add_space(4.0);
    if zoom_ui.button(RichText::new("-").strong()).clicked() {
        app.zoom = (app.zoom / 1.15).max(0.2);
    }
    zoom_ui.label(RichText::new(format!("{:.0}%", app.zoom * 100.0)).size(10.5));
    if zoom_ui.button(RichText::new("+").strong()).clicked() {
        app.zoom = (app.zoom * 1.15).min(5.0);
    }
}

/// Renders a top-centered placement banner when a component is armed for placement.
fn render_spawning_banner(ui: &mut egui::Ui, rect: Rect, app: &mut SimulatorApp) {
    if app.spawning == SpawningComponent::None {
        return;
    }
    let banner_rect = Rect::from_center_size(
        Rect::from_min_size(rect.min, Vec2::new(rect.width(), 32.0)).center(),
        Vec2::new(320.0, 26.0),
    );
    let painter = ui.painter();
    painter.rect_filled(banner_rect, 4.0, Color32::from_rgb(13, 153, 255));
    painter.text(
        banner_rect.center(),
        egui::Align2::CENTER_CENTER,
        "Click canvas to place component (Esc to cancel)",
        egui::FontId::proportional(11.0),
        Color32::WHITE,
    );
}
