//! Property inspector and debug panels for schematic elements.

pub mod buzzer;
pub mod dht22;
pub mod esp32_s3;
pub mod lcd1602;
pub mod led;
pub mod mq135;
pub mod servo;
pub mod ssd1306;
pub mod wire;

use egui::{RichText, Ui};

use crate::app::SimulatorApp;
use crate::types::SelectedItem;
pub use esp32_s3::render_esp32_s3_inspector;
pub use wire::render_wire_inspector;

/// Renders the property inspector and debug panel.
pub fn render_inspector(ui: &mut Ui, app: &mut SimulatorApp) {
    ui.heading("Inspect & Debug");
    ui.separator();

    match app.selected {
        SelectedItem::None => {
            ui.label("Click an element on the canvas to inspect its properties.");
        }
        SelectedItem::Wire(index) => {
            render_wire_inspector(ui, app, index);
        }
        SelectedItem::Esp32S3(index) => {
            render_esp32_s3_inspector(ui, app, index);
        }
        SelectedItem::Component(index) => {
            render_component_inspector(ui, app, index);
        }
        SelectedItem::Group(ref items) => {
            ui.label(RichText::new("Selected Group").strong());
            ui.label(format!("Items selected: {}", items.len()));
            ui.separator();
            ui.label("Use mouse drag to move all selected items together.");
        }
    }
}

/// Renders the property inspector panel for any placed discrete component.
pub fn render_component_inspector(ui: &mut Ui, app: &mut SimulatorApp, index: usize) {
    let mut delete_requested = false;
    let mut rebuild_requested = false;
    if let Some(comp) = app.components.get_mut(index) {
        comp.instance.render_inspector(
            ui,
            &app.engine,
            || delete_requested = true,
            || rebuild_requested = true,
        );
    }
    if delete_requested {
        app.delete_selected();
    } else if rebuild_requested {
        app.rebuild_netlist();
    }
}
