//! Fullscreen modal dialog for exploring and spawning components and boards.

use egui::{Color32, RichText, Ui, Vec2};

use super::card::render_component_card;
use super::catalog::get_catalog;
use super::types::{CategoryKind, ComponentItem};
use crate::app::SimulatorApp;
use crate::types::SpawningComponent;

/// Renders the Add Component modal window.
pub fn render_add_component_dialog(ctx: &egui::Context, app: &mut SimulatorApp) {
    if !app.show_add_component {
        return;
    }

    let screen_rect = ctx.screen_rect();
    let modal_width = (screen_rect.width() * 0.70).clamp(650.0, 920.0);
    let modal_height = (screen_rect.height() * 0.72).clamp(460.0, 640.0);

    let mut open = true;
    let mut selected_to_spawn: Option<SpawningComponent> = None;

    egui::Window::new("Add Component")
        .open(&mut open)
        .collapsible(false)
        .resizable(true)
        .movable(true)
        .pivot(egui::Align2::CENTER_CENTER)
        .default_pos(screen_rect.center())
        .default_size(Vec2::new(modal_width, modal_height))
        .min_size(Vec2::new(560.0, 380.0))
        .show(ctx, |ui| {
            render_search_bar(ui, app);
            ui.separator();

            let full_height = ui.available_height();
            ui.horizontal(|ui| {
                ui.set_height(full_height);
                render_categories_sidebar(ui, app);
                ui.separator();
                render_catalog_grid(ui, app, &mut selected_to_spawn);
            });
        });

    if !open {
        app.show_add_component = false;
    }

    if let Some(spawn) = selected_to_spawn {
        let spawn_pos = app.camera_center();
        app.record_history();
        app.spawning = spawn;
        app.spawn_component_at(spawn_pos);
        app.show_add_component = false;
    }
}

/// Renders the prominent search input at the top of the modal.
fn render_search_bar(ui: &mut Ui, app: &mut SimulatorApp) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("🔍").size(15.0));
        let search_edit = egui::TextEdit::singleline(&mut app.add_component_search)
            .hint_text("Search components...")
            .desired_width(ui.available_width() - 30.0);
        ui.add(search_edit);
        if !app.add_component_search.is_empty() && ui.button("✕").clicked() {
            app.add_component_search.clear();
        }
    });
}

/// Renders the category selection sidebar on the left with item count badges.
fn render_categories_sidebar(ui: &mut Ui, app: &mut SimulatorApp) {
    ui.vertical(|ui| {
        ui.set_width(170.0);
        let catalog = get_catalog();

        for cat in CategoryKind::ALL {
            let count = if cat == CategoryKind::All {
                catalog.len()
            } else {
                catalog.iter().filter(|i| i.category == cat).count()
            };

            let is_selected = app.add_component_category == cat;
            let label_text = cat.label();

            ui.horizontal(|ui| {
                if ui
                    .selectable_label(
                        is_selected,
                        RichText::new(label_text).color(if is_selected {
                            Color32::from_rgb(13, 153, 255)
                        } else {
                            Color32::from_rgb(210, 215, 225)
                        }),
                    )
                    .clicked()
                {
                    app.add_component_category = cat;
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("{count}"))
                            .size(10.0)
                            .color(Color32::from_gray(130)),
                    );
                });
            });
        }
    });
}

/// Renders the filtered devices card grid in the main section.
fn render_catalog_grid(
    ui: &mut Ui,
    app: &mut SimulatorApp,
    selected_to_spawn: &mut Option<SpawningComponent>,
) {
    let catalog = get_catalog();
    let query = app.add_component_search.trim().to_lowercase();
    let category = app.add_component_category;

    let filtered: Vec<&ComponentItem> = catalog
        .iter()
        .filter(|item| {
            let matches_cat = category == CategoryKind::All || item.category == category;
            let matches_query = query.is_empty()
                || item.name.to_lowercase().contains(&query)
                || item.specs.to_lowercase().contains(&query);
            matches_cat && matches_query
        })
        .collect();

    ui.vertical(|ui| {
        ui.set_width(ui.available_width());
        ui.set_height(ui.available_height());
        ui.horizontal(|ui| {
            let title = if !query.is_empty() {
                format!("SEARCH RESULTS ({})", filtered.len())
            } else {
                format!("{} ({})", category.label().to_uppercase(), filtered.len())
            };
            ui.label(
                RichText::new(title)
                    .strong()
                    .size(12.0)
                    .color(Color32::from_gray(170)),
            );
        });
        ui.add_space(4.0);

        egui::ScrollArea::vertical()
            .id_source("add_component_grid_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(10.0, 10.0);
                    for item in &filtered {
                        let resp = render_component_card(ui, item);
                        if resp.clicked() {
                            if item.is_ready {
                                *selected_to_spawn = Some(item.spawn);
                            } else {
                                app.code_editor.status_message = format!(
                                    "Komponen '{}' sedang dalam tahap pengembangan!",
                                    item.name
                                );
                                app.code_editor.status_is_error = false;
                            }
                        }
                    }
                });
            });
    });
}
