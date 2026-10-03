use sim_components::component::{Button, ButtonColor};
use sim_core::component::Component;
use sim_core::engine::Engine;
use sim_core::netlist::PinId;

#[test]
fn test_button_creation_and_pins() {
    let btn = Button::new(PinId(101), PinId(102), PinId(103), PinId(104));
    assert_eq!(btn.name(), "Push Button");
    assert_eq!(btn.pin_1a(), PinId(101));
    assert_eq!(btn.pin_1b(), PinId(102));
    assert_eq!(btn.pin_2a(), PinId(103));
    assert_eq!(btn.pin_2b(), PinId(104));
    assert_eq!(
        btn.pins(),
        vec![PinId(101), PinId(102), PinId(103), PinId(104)]
    );
    assert!(!btn.is_pressed());
    assert!(!btn.is_conductive());
    assert!(!btn.is_latching());
    assert_eq!(btn.color(), ButtonColor::Blue);
}

#[test]
fn test_button_momentary_press_cycle() {
    let mut btn = Button::new(PinId(1), PinId(2), PinId(3), PinId(4));
    btn.press();
    assert!(btn.is_pressed());
    assert!(btn.is_conductive());

    btn.release();
    assert!(!btn.is_pressed());
    assert!(!btn.is_conductive());
}

#[test]
fn test_button_latching_mode_toggle() {
    let mut btn = Button::new(PinId(1), PinId(2), PinId(3), PinId(4));
    btn.set_latching(true);
    assert!(btn.is_latching());

    btn.toggle();
    assert!(btn.is_pressed());

    btn.toggle();
    assert!(!btn.is_pressed());
}

#[test]
fn test_button_colors() {
    let mut btn = Button::new(PinId(1), PinId(2), PinId(3), PinId(4));
    for color in ButtonColor::ALL {
        btn.set_color(color);
        assert_eq!(btn.color(), color);
        assert!(!color.label().is_empty());
    }
}

#[test]
fn test_button_bridges_nodes_when_pressed() {
    let btn = Button::new(PinId(10), PinId(11), PinId(12), PinId(13));
    let mut engine = Engine::new();

    let node_vcc = engine.netlist.add_node();
    let node_gnd = engine.netlist.add_node();
    let node_btn_out = engine.netlist.add_node();

    engine.netlist.connect_pin(PinId(1), node_vcc);
    engine.netlist.connect_pin(PinId(2), node_gnd);
    engine.netlist.connect_pin(btn.pin_1a(), node_vcc);
    engine.netlist.connect_pin(btn.pin_2a(), node_btn_out);

    engine
        .network
        .claim_rail_pin(&engine.netlist, PinId(1), 5.0);
    engine
        .network
        .claim_rail_pin(&engine.netlist, PinId(2), 0.0);

    engine
        .network
        .add_resistor_pin(&engine.netlist, btn.pin_2a(), PinId(2), 10_000.0);

    engine.solve_analog_network();
    assert!(
        engine.get_node_voltage(node_btn_out) < 0.1,
        "unpressed button node should be pulled down near 0V"
    );

    engine
        .network
        .add_resistor_pin(&engine.netlist, btn.pin_1a(), btn.pin_2a(), 0.001);
    engine.solve_analog_network();
    assert!(
        engine.get_node_voltage(node_btn_out) > 4.9,
        "pressed button should bridge to 5V rail"
    );
}
