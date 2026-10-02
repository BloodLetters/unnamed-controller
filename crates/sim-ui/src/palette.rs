use egui::Ui;

use crate::registry::{Category, PALETTE};
use crate::types::SpawningComponent;

/// Renders the component and board library sidebar panel.
pub fn render_palette(ui: &mut Ui, spawning: &mut SpawningComponent) {
    ui.heading("Components & Boards");
    ui.separator();

    if PALETTE.is_empty() {
        ui.label("Palette is empty.");
        ui.label("Ready for board & component definitions.");
        ui.separator();
        ui.label("Tool: Pointer / Select");
        return;
    }

    ui.label("Select an item to place on the canvas:");

    for category in Category::ALL {
        ui.collapsing(category.title(), |ui| {
            for entry in PALETTE.iter().filter(|entry| entry.category == category) {
                if ui.button(entry.label).clicked() {
                    *spawning = entry.spawn;
                }
            }
        });
    }

    ui.separator();
    match PALETTE.iter().find(|entry| entry.spawn == *spawning) {
        Some(entry) => ui.label(format!("Tool: Place {}", entry.label)),
        None => ui.label("Tool: Pointer / Select"),
    };

    if *spawning != SpawningComponent::None && ui.button("Cancel Placement").clicked() {
        *spawning = SpawningComponent::None;
    }
}
