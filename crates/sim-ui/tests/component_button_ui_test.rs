//! Integration tests for Push Button component UI lifecycle, simulation, and persistence.

use egui::Pos2;
use sim_components::component::ButtonColor;
use sim_ui::app::SimulatorApp;
use sim_ui::component::ComponentInstance;
use sim_ui::state::snapshot::{load_project_json, save_project_json};
use sim_ui::types::{SelectedItem, SpawningComponent};

/// Verifies that spawning a push button populates the component list with correct initial properties.
#[test]
fn test_spawn_push_button_lifecycle() {
    let mut app = SimulatorApp {
        spawning: SpawningComponent::BUTTON,
        ..Default::default()
    };
    app.spawn_component_at(Pos2::new(150.0, 200.0));

    assert_eq!(app.components.len(), 1);
    let comp = &app.components[0];
    assert_eq!(comp.instance.name(), "Push Button (Tactile)");
    assert_eq!(comp.instance.pins().len(), 4);
    assert_eq!(comp.pos, Pos2::new(150.0, 200.0));
    assert_eq!(app.selected, SelectedItem::Component(0));

    if let ComponentInstance::Button(button) = &comp.instance {
        assert_eq!(button.color(), ButtonColor::Blue);
        assert!(!button.is_pressed());
        assert!(!button.is_latching());
    } else {
        panic!("expected ComponentInstance::Button");
    }
}

/// Verifies that duplicating a push button preserves configuration with allocated distinct pins.
#[test]
fn test_duplicate_push_button() {
    let mut app = SimulatorApp {
        spawning: SpawningComponent::BUTTON,
        ..Default::default()
    };
    app.spawn_component_at(Pos2::new(100.0, 100.0));

    if let ComponentInstance::Button(button) = &mut app.components[0].instance {
        button.set_color(ButtonColor::Red);
        button.set_latching(true);
    }

    app.duplicate_selected();
    assert_eq!(app.components.len(), 2);

    let original_pins = app.components[0].instance.pins();
    let cloned_pins = app.components[1].instance.pins();
    assert_ne!(original_pins, cloned_pins);

    if let ComponentInstance::Button(cloned_button) = &app.components[1].instance {
        assert_eq!(cloned_button.color(), ButtonColor::Red);
        assert!(cloned_button.is_latching());
    } else {
        panic!("expected cloned ComponentInstance::Button");
    }
}

/// Verifies that clicking the button on canvas invokes toggle actuation.
#[test]
fn test_canvas_click_actuation() {
    let mut app = SimulatorApp {
        spawning: SpawningComponent::BUTTON,
        ..Default::default()
    };
    app.spawn_component_at(Pos2::new(100.0, 100.0));

    assert!(!match &app.components[0].instance {
        ComponentInstance::Button(b) => b.is_pressed(),
        _ => true,
    });

    app.components[0].instance.on_canvas_click();

    assert!(match &app.components[0].instance {
        ComponentInstance::Button(b) => b.is_pressed(),
        _ => false,
    });

    app.components[0].instance.on_canvas_click();

    assert!(!match &app.components[0].instance {
        ComponentInstance::Button(b) => b.is_pressed(),
        _ => true,
    });
}

/// Verifies that push button instances survive serialization and deserialization intact.
#[test]
fn test_roundtrip_preserves_push_button() {
    let mut app = SimulatorApp {
        spawning: SpawningComponent::BUTTON,
        ..Default::default()
    };
    app.spawn_component_at(Pos2::new(240.0, 180.0));

    if let ComponentInstance::Button(button) = &mut app.components[0].instance {
        button.set_color(ButtonColor::Green);
        button.set_latching(true);
    }

    let snap = app.snapshot();
    assert_eq!(snap.components.len(), 1);

    let json = save_project_json(&snap).expect("serialize");
    let restored_data = load_project_json(&json).expect("deserialize");
    assert_eq!(restored_data.components.len(), 1);

    let mut restored_app = SimulatorApp::default();
    restored_app.restore_snapshot(restored_data);
    assert_eq!(restored_app.components.len(), 1);
    assert_eq!(restored_app.components[0].pos, Pos2::new(240.0, 180.0));

    if let ComponentInstance::Button(restored_btn) = &restored_app.components[0].instance {
        assert_eq!(restored_btn.color(), ButtonColor::Green);
        assert!(restored_btn.is_latching());
    } else {
        panic!("expected restored ComponentInstance::Button");
    }
}

/// Verifies that pressing a button bridges terminals into the resistive simulation network.
#[test]
fn test_push_button_electrical_simulation_bridging() {
    let mut app = SimulatorApp {
        spawning: SpawningComponent::BUTTON,
        power_on: true,
        ..Default::default()
    };
    app.spawn_component_at(Pos2::new(50.0, 50.0));

    let (p1a, p1b, p2a, p2b) = match &app.components[0].instance {
        ComponentInstance::Button(btn) => (btn.pin_1a(), btn.pin_1b(), btn.pin_2a(), btn.pin_2b()),
        _ => unreachable!(),
    };

    let node_1a = app.engine.netlist.add_node();
    let node_1b = app.engine.netlist.add_node();
    let node_2a = app.engine.netlist.add_node();
    let node_2b = app.engine.netlist.add_node();

    app.engine.netlist.connect_pin(p1a, node_1a);
    app.engine.netlist.connect_pin(p1b, node_1b);
    app.engine.netlist.connect_pin(p2a, node_2a);
    app.engine.netlist.connect_pin(p2b, node_2b);

    sim_ui::sim::tick_simulation(&mut app);

    assert_eq!(app.engine.network.resistor_count(), 2);

    if let ComponentInstance::Button(btn) = &mut app.components[0].instance {
        btn.press();
    }

    sim_ui::sim::tick_simulation(&mut app);

    assert_eq!(app.engine.network.resistor_count(), 3);
}
