use egui::{Color32, RichText, Ui};

use crate::app::SimulatorApp;

/// Renders the integrated Arduino Library Manager: search, install, and update.
pub fn render_library_panel(ui: &mut Ui, app: &mut SimulatorApp) {
    if !app.code_editor.library.loaded {
        app.code_editor.library.loaded = true;
        crate::editor::start_background_refresh(app);
    }

    let is_busy = app.code_editor.library.is_busy;
    let mut run_search = false;
    let mut update_index = false;
    let mut install: Option<String> = None;

    egui::Frame::group(ui.style())
        .inner_margin(6.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Cari library").strong());
                let response = ui.add_enabled(
                    !is_busy,
                    egui::TextEdit::singleline(&mut app.code_editor.library.query)
                        .hint_text("mis. Adafruit SSD1306, LiquidCrystal I2C")
                        .desired_width(280.0),
                );
                let enter =
                    response.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));
                if (ui
                    .add_enabled(!is_busy, egui::Button::new("🔍 Cari"))
                    .clicked()
                    || (enter && !is_busy))
                    && !app.code_editor.library.query.trim().is_empty()
                {
                    run_search = true;
                }
                if ui
                    .add_enabled(!is_busy, egui::Button::new("⟳ Update Index"))
                    .clicked()
                {
                    update_index = true;
                }
            });
        });

    if is_busy {
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.spinner();
            ui.label(
                RichText::new(&app.code_editor.library.busy_text)
                    .italics()
                    .color(Color32::from_rgb(100, 180, 255)),
            );
        });
    }

    render_library_lists(ui, app, &mut install);

    if update_index {
        crate::editor::start_background_update_index(app);
    }

    if run_search {
        crate::editor::start_background_search(app);
    }

    if let Some(name) = install {
        crate::editor::start_background_install(app, name);
    }
}

/// Renders installed libraries and search results sections.
fn render_library_lists(ui: &mut Ui, app: &mut SimulatorApp, install: &mut Option<String>) {
    let library = &app.code_editor.library;
    let is_busy = library.is_busy;

    ui.add_space(10.0);
    ui.label(RichText::new(format!("Terpasang ({})", library.installed.len())).strong());
    ui.add_space(4.0);
    if library.installed.is_empty() {
        ui.label(
            RichText::new("Belum ada library terpasang.")
                .italics()
                .color(Color32::from_gray(140)),
        );
    } else {
        egui::ScrollArea::vertical()
            .id_source("library_installed")
            .max_height(150.0)
            .auto_shrink([false, true])
            .show(ui, |ui| {
                for entry in &library.installed {
                    let has_update = library.updatable.iter().any(|name| name == &entry.name);
                    egui::Frame::group(ui.style())
                        .inner_margin(6.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(format!("{}  v{}", entry.name, entry.version))
                                        .strong(),
                                );
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if has_update
                                            && ui
                                                .add_enabled(!is_busy, egui::Button::new("Update"))
                                                .clicked()
                                        {
                                            *install = Some(entry.name.clone());
                                        }
                                    },
                                );
                            });
                        });
                }
            });
    }

    ui.add_space(10.0);
    ui.separator();
    ui.add_space(2.0);
    ui.label(RichText::new(format!("Hasil pencarian ({})", library.results.len())).strong());
    ui.add_space(2.0);

    let remaining = (ui.available_height() - 30.0).max(160.0);
    egui::ScrollArea::vertical()
        .id_source("library_results")
        .auto_shrink([false, false])
        .max_height(remaining)
        .show_rows(ui, 34.0, library.results.len(), |ui, range| {
            for index in range {
                let entry = &library.results[index];
                let current = library
                    .installed
                    .iter()
                    .find(|item| item.name == entry.name);
                let has_update = library.updatable.iter().any(|name| name == &entry.name);
                ui.allocate_ui_with_layout(
                    egui::vec2(ui.available_width(), 34.0),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        ui.add_space(4.0);
                        let label = ui.label(
                            RichText::new(format!("{}  v{}", entry.name, entry.version)).strong(),
                        );
                        if !entry.sentence.is_empty() {
                            label.on_hover_text(entry.sentence.as_str());
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.add_space(4.0);
                            match current {
                                Some(_) if has_update => {
                                    if ui
                                        .add_enabled(!is_busy, egui::Button::new("Update"))
                                        .clicked()
                                    {
                                        *install = Some(entry.name.clone());
                                    }
                                }
                                Some(item) => {
                                    ui.label(
                                        RichText::new(format!("Terpasang v{}", item.version))
                                            .color(Color32::from_gray(150)),
                                    );
                                }
                                None => {
                                    if ui
                                        .add_enabled(!is_busy, egui::Button::new("Install"))
                                        .clicked()
                                    {
                                        *install = Some(entry.name.clone());
                                    }
                                }
                            }
                        });
                    },
                );
            }
        });

    if !library.status.is_empty() {
        let color = if library.status_is_error {
            Color32::from_rgb(255, 100, 100)
        } else {
            Color32::from_rgb(120, 220, 120)
        };
        ui.label(RichText::new(&library.status).color(color));
    }
}
