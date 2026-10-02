//! Top navigation bar and action header.

use egui::{Color32, RichText, Ui, Vec2};

use crate::app::SimulatorApp;
use crate::types::ViewMode;

/// Renders the unified top bar with brand logo, menus, mode switchers, and controls.
pub fn render_navbar(ctx: &egui::Context, app: &mut SimulatorApp) {
    egui::TopBottomPanel::top("navbar_panel").show(ctx, |ui| {
        ui.horizontal(|ui| {
            render_brand_and_menus(ctx, ui, app);
            ui.separator();
            render_center_controls(ui, app);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                render_right_actions(ui, app);
            });
        });
    });
}

/// Renders drop-down menu items.
fn render_brand_and_menus(ctx: &egui::Context, ui: &mut Ui, app: &mut SimulatorApp) {
    ui.menu_button("File", |ui| {
        if ui.button("New Project").clicked() {
            crate::state::new_project(app);
        }
        if ui.button("Open Project…").clicked() {
            crate::state::open_project(app);
        }
        if ui.button("Save Project").clicked() {
            crate::state::save_project(app, false);
        }
        if ui.button("Save Project As…").clicked() {
            crate::state::save_project(app, true);
        }
        ui.separator();
        if ui.button("Quit").clicked() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    });

    ui.menu_button("Edit", |ui| {
        if ui
            .add_enabled(app.history.can_undo(), egui::Button::new("Undo (Ctrl+Z)"))
            .clicked()
        {
            app.undo();
        }
        if ui
            .add_enabled(app.history.can_redo(), egui::Button::new("Redo (Ctrl+Y)"))
            .clicked()
        {
            app.redo();
        }
    });

    ui.menu_button("View", |ui| {
        if ui.button("Reset Canvas View").clicked() {
            app.pan = Vec2::ZERO;
            app.zoom = 1.0;
        }
        ui.checkbox(&mut app.show_workspace, "Show Workspace Panel");
    });
}

/// Renders the center view switches, play/pause controls, and diagnostic shortcuts.
fn render_center_controls(ui: &mut Ui, app: &mut SimulatorApp) {
    ui.selectable_value(&mut app.view_mode, ViewMode::Code, "< > Code");
    ui.selectable_value(&mut app.view_mode, ViewMode::Both, "[|] Both");
    ui.selectable_value(&mut app.view_mode, ViewMode::Circuit, "+ Circuit");

    ui.separator();

    let (sim_text, sim_color) = if app.power_on {
        ("⏹ Stop", Color32::from_rgb(239, 68, 68))
    } else {
        ("▶ Run", Color32::from_rgb(34, 197, 94))
    };

    if ui
        .button(RichText::new(sim_text).strong().color(sim_color))
        .clicked()
    {
        app.power_on = !app.power_on;
    }

    if ui.button("⟳ Reboot").clicked() {
        crate::editor::actions::reset_target(app);
    }

    ui.separator();

    let is_lib_active = app.code_editor.section == crate::editor::types::EditorSection::Libraries;
    if ui.selectable_label(is_lib_active, "📚 Libraries").clicked() {
        app.code_editor.section = if is_lib_active {
            crate::editor::types::EditorSection::Code
        } else {
            crate::editor::types::EditorSection::Libraries
        };
    }

    if ui
        .selectable_label(app.code_editor.show_serial_monitor, "📟 Serial")
        .clicked()
    {
        app.code_editor.show_serial_monitor = !app.code_editor.show_serial_monitor;
    }

    if ui
        .selectable_label(app.instruments.open, "🔬 Scope")
        .clicked()
    {
        app.instruments.open = !app.instruments.open;
    }
}

/// Renders the prominent bright blue Add button on the right.
fn render_right_actions(ui: &mut Ui, app: &mut SimulatorApp) {
    let add_btn = egui::Button::new(
        RichText::new("+ Add")
            .strong()
            .size(13.0)
            .color(Color32::WHITE),
    )
    .fill(Color32::from_rgb(13, 153, 255))
    .rounding(4.0);

    if ui.add(add_btn).clicked() {
        app.show_add_component = true;
    }
}

/// Renders the one-time toolchain warning window when a compiler is missing.
pub fn render_toolchain_warning(ctx: &egui::Context, app: &mut SimulatorApp) {
    if !app.show_toolchain_warning {
        return;
    }
    let Some(message) = app.toolchain_warning.clone() else {
        app.show_toolchain_warning = false;
        return;
    };

    let mut open = true;
    let mut recheck = false;
    egui::Window::new("Toolchain Arduino belum terpasang")
        .collapsible(false)
        .resizable(false)
        .open(&mut open)
        .show(ctx, |ui| {
            ui.label(message);
            ui.separator();
            recheck = ui.button("Periksa lagi / Saya sudah memasang").clicked();
        });

    if recheck && crate::toolchain::ArduinoCliCompiler::default().is_available() {
        app.toolchain_warning = None;
        app.show_toolchain_warning = false;
    } else if !open {
        app.show_toolchain_warning = false;
    }
}
