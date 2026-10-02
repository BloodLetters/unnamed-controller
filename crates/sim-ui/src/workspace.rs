//! Workspace tree view panel for project file exploration.

use egui::{Color32, RichText, Ui};

use crate::app::SimulatorApp;
use crate::editor::types::{CodeTab, EditorSection};
use crate::types::SelectedItem;

/// Renders the VS Code style project workspace explorer panel.
pub fn render_workspace_panel(ui: &mut Ui, app: &mut SimulatorApp) {
    ui.add_space(4.0);
    render_workspace_header(ui, app);
    ui.separator();
    render_workspace_tree(ui, app);
}

/// Renders the uppercase title bar with quick action buttons.
fn render_workspace_header(ui: &mut Ui, app: &mut SimulatorApp) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("WORKSPACE")
                .size(11.0)
                .strong()
                .color(Color32::from_gray(160)),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .button(RichText::new("+📄").size(11.0))
                .on_hover_text("New file")
                .clicked()
            {
                let id = app.code_editor.tabs.len() + 1;
                app.code_editor.tabs.push(CodeTab::new(
                    format!("sketch{}.ino", id),
                    "void loop() {\n    // Code here\n}\n",
                ));
                app.code_editor.active_tab = app.code_editor.tabs.len() - 1;
            }
        });
    });
}

/// Renders the tree hierarchy containing boards, sketch files, and project assets.
fn render_workspace_tree(ui: &mut Ui, app: &mut SimulatorApp) {
    egui::ScrollArea::vertical()
        .id_source("workspace_tree_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            render_boards_section(ui, app);
            ui.add_space(4.0);
            render_files_section(ui, app);
            ui.add_space(4.0);
            render_libraries_section(ui, app);
        });
}

/// Renders the connected development boards in the workspace tree.
fn render_boards_section(ui: &mut Ui, app: &mut SimulatorApp) {
    ui.collapsing(
        RichText::new("⚡ Circuit Boards").strong().size(11.5),
        |ui| {
            if app.esp32_s3_boards.is_empty() {
                ui.label(RichText::new("No boards placed").weak().size(10.5));
                return;
            }

            for i in 0..app.esp32_s3_boards.len() {
                let is_selected = app.selected == SelectedItem::Esp32S3(i);
                let label = format!("ESP32-S3 #{}", i);
                if ui
                    .selectable_label(
                        is_selected,
                        RichText::new(format!("📟 {}", label)).size(11.0),
                    )
                    .clicked()
                {
                    app.selected = SelectedItem::Esp32S3(i);
                }
            }
        },
    );
}

/// Renders open and available firmware source files.
fn render_files_section(ui: &mut Ui, app: &mut SimulatorApp) {
    ui.collapsing(RichText::new("📁 Sketches").strong().size(11.5), |ui| {
        let active = app.code_editor.active_tab;
        for i in 0..app.code_editor.tabs.len() {
            let is_active = active == i && app.code_editor.section == EditorSection::Code;
            let title = app.code_editor.tabs[i].title.clone();
            let text = RichText::new(format!("📄 {}", title))
                .size(11.0)
                .color(if is_active {
                    Color32::from_rgb(13, 153, 255)
                } else {
                    Color32::from_gray(200)
                });

            if ui.selectable_label(is_active, text).clicked() {
                app.code_editor.active_tab = i;
                app.code_editor.section = EditorSection::Code;
            }
        }
    });
}

/// Renders the libraries shortcut in the workspace tree.
fn render_libraries_section(ui: &mut Ui, app: &mut SimulatorApp) {
    let is_active = app.code_editor.section == EditorSection::Libraries;
    let label = RichText::new("📚 libraries.json")
        .size(11.0)
        .color(if is_active {
            Color32::from_rgb(13, 153, 255)
        } else {
            Color32::from_gray(200)
        });

    if ui.selectable_label(is_active, label).clicked() {
        app.code_editor.section = EditorSection::Libraries;
    }
}
