//! Right-click context menu rendering for components and circuit canvas.

use egui::{Color32, RichText, Ui};

use crate::app::SimulatorApp;
use crate::types::SelectedItem;

/// Renders the right-click context menu for canvas components or empty workbench space.
pub fn render_canvas_context_menu(ui: &mut Ui, app: &mut SimulatorApp) {
    ui.set_min_width(175.0);

    match app.selected.clone() {
        SelectedItem::Esp32S3(_) => {
            render_header(ui, "ESP32-S3 DevKitC-1");
            render_rotation_actions(ui, app);
            ui.separator();
            render_standard_actions(ui, app);
        }
        SelectedItem::Component(idx) => {
            let name = app
                .components
                .get(idx)
                .map(|c| c.instance.name().to_string());
            if let Some(title) = name {
                render_header(ui, &title);
                render_rotation_actions(ui, app);
                ui.separator();
                if let Some(comp) = app.components.get_mut(idx) {
                    comp.instance.render_context_menu(ui);
                }
                ui.separator();
                render_standard_actions(ui, app);
            }
        }
        SelectedItem::Group(ref items) => {
            render_header(ui, &format!("Selected Group ({} items)", items.len()));
            render_rotation_actions(ui, app);
            ui.separator();
            render_standard_actions(ui, app);
        }
        SelectedItem::Wire(_) => {
            render_header(ui, "Wire Connection");
            if ui.button("🗑 Delete Wire\tDel").clicked() {
                app.delete_selected();
                ui.close_menu();
            }
        }
        SelectedItem::None => {
            render_empty_canvas_menu(ui, app);
        }
    }
}

/// Renders a distinctive title header at the top of the context menu.
fn render_header(ui: &mut Ui, title: &str) {
    ui.label(
        RichText::new(title)
            .strong()
            .color(Color32::from_rgb(0, 229, 255)),
    );
    ui.separator();
}

/// Renders rotation options for the selected component or group.
fn render_rotation_actions(ui: &mut Ui, app: &mut SimulatorApp) {
    if ui.button("⟳ Rotate 90° CW\tR").clicked() {
        app.rotate_selected(90);
        ui.close_menu();
    }
    if ui.button("⟲ Rotate 90° CCW\tShift+R").clicked() {
        app.rotate_selected(-90);
        ui.close_menu();
    }
    if ui.button("🔃 Rotate 180°").clicked() {
        app.rotate_selected(180);
        ui.close_menu();
    }
}

/// Renders common duplicate and delete actions for components.
fn render_standard_actions(ui: &mut Ui, app: &mut SimulatorApp) {
    if ui.button("📋 Duplicate\tCtrl+D").clicked() {
        app.duplicate_selected();
        ui.close_menu();
    }
    if ui.button("🗑 Delete\tDel").clicked() {
        app.delete_selected();
        ui.close_menu();
    }
}

/// Renders canvas utility actions when right clicking on empty canvas space.
fn render_empty_canvas_menu(ui: &mut Ui, app: &mut SimulatorApp) {
    render_header(ui, "Circuit Workbench");

    if ui.button("🔍 Reset Zoom (100%)").clicked() {
        app.zoom = 1.0;
        ui.close_menu();
    }
    if ui.button("🎯 Reset View Center").clicked() {
        app.pan = egui::Vec2::ZERO;
        ui.close_menu();
    }
}
