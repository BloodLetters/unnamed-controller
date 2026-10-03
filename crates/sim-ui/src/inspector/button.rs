//! Property inspector panel for tactile pushbutton switches.

use egui::{Color32, RichText, Ui, Vec2};
use sim_components::component::{Button, ButtonColor};
use sim_core::engine::Engine;

/// Renders inspector controls, live telemetry, and tactile actuation for a pushbutton switch.
pub fn render_button_inspector_content(
    ui: &mut Ui,
    button: &mut Button,
    engine: &Engine,
    on_delete: impl FnOnce(),
) {
    ui.label(
        RichText::new("Push Button (Tactile Switch)")
            .strong()
            .size(15.0),
    );
    ui.label(
        RichText::new("6x6mm SPST Momentary / Latching Switch")
            .weak()
            .size(11.0),
    );
    ui.separator();

    ui.horizontal(|ui| {
        ui.label("Cap Color:");
        let current_color = button.color();
        egui::ComboBox::from_id_source("button_color_picker")
            .selected_text(current_color.label())
            .show_ui(ui, |ui| {
                for color in ButtonColor::ALL {
                    if ui
                        .selectable_label(current_color == color, color.label())
                        .clicked()
                    {
                        button.set_color(color);
                    }
                }
            });
    });

    ui.horizontal(|ui| {
        let mut latching = button.is_latching();
        if ui.checkbox(&mut latching, "Latching Mode").changed() {
            button.set_latching(latching);
        }
    });

    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label("Contact State:");
        let (rect, _resp) = ui.allocate_exact_size(Vec2::new(12.0, 12.0), egui::Sense::hover());
        let indicator_color = if button.is_conductive() {
            Color32::from_rgb(34, 197, 94)
        } else {
            Color32::from_rgb(100, 116, 139)
        };
        ui.painter()
            .circle_filled(rect.center(), 5.0, indicator_color);

        if button.is_conductive() {
            ui.label(
                RichText::new("CLOSED (Conducting)")
                    .strong()
                    .color(Color32::from_rgb(74, 222, 128)),
            );
        } else {
            ui.label(RichText::new("OPEN (High-Z)").weak());
        }
    });

    ui.add_space(4.0);
    let actuate_text = if button.is_latching() {
        if button.is_pressed() {
            "🔓 Unlatch Switch"
        } else {
            "🔒 Latch Switch"
        }
    } else {
        if button.is_pressed() {
            "⬆ Release Switch"
        } else {
            "⬇ Press Switch"
        }
    };
    if ui.button(RichText::new(actuate_text).size(13.0)).clicked() {
        button.toggle();
    }

    ui.separator();
    ui.label(RichText::new("Terminal Telemetry").strong());

    let v_t1 = engine
        .netlist
        .get_node_for_pin(button.pin_1a())
        .map(|n| engine.get_node_voltage(n))
        .unwrap_or(0.0);
    let v_t2 = engine
        .netlist
        .get_node_for_pin(button.pin_2a())
        .map(|n| engine.get_node_voltage(n))
        .unwrap_or(0.0);

    ui.label(format!("Terminal 1 (Pins 1a, 1b): {:.2} V", v_t1));
    ui.label(format!("Terminal 2 (Pins 2a, 2b): {:.2} V", v_t2));
    ui.label(format!(
        "Path Resistance: {}",
        if button.is_conductive() {
            "~0.001 \u{3a9}"
        } else {
            "\u{221e} (Open)"
        }
    ));

    ui.separator();
    if ui.button("Delete Button").clicked() {
        on_delete();
    }
}
