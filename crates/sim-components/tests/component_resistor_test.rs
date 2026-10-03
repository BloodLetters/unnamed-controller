use sim_components::board::esp32_s3::Esp32S3DevKit;
use sim_components::board::{Board, RAIL_VOLTAGE_3V3, RAIL_VOLTAGE_5V};
use sim_components::component::{RESISTOR_DEFAULT_OHMS, Resistor};
use sim_core::component::Component;
use sim_core::engine::Engine;
use sim_core::netlist::{NodeId, PinId};
use sim_core::pins::{DigitalState, PinDrive};

/// Resolves the first header pin carrying a given silkscreen label.
fn pin_named(board: &Esp32S3DevKit, name: &str) -> PinId {
    board
        .header_pins()
        .iter()
        .find(|p| p.name == name)
        .map(|p| p.pin_id)
        .expect("header pin")
}

#[test]
fn test_resistor_creation_and_pins() {
    let resistor = Resistor::new(PinId(201), PinId(202));
    assert_eq!(resistor.terminal_a(), PinId(201));
    assert_eq!(resistor.terminal_b(), PinId(202));
    assert_eq!(resistor.pins(), vec![PinId(201), PinId(202)]);
    assert_eq!(resistor.resistance_ohms(), RESISTOR_DEFAULT_OHMS);
}

#[test]
fn test_resistor_ohm_law() {
    let resistor = Resistor::with_resistance(PinId(1), PinId(2), 470.0);
    let current = resistor.current_amps(3.3, 0.0);
    assert!((current - 3.3 / 470.0).abs() < 1e-6);
    assert!(current > 0.0);
}

#[test]
fn test_resistor_set_resistance_is_clamped() {
    let mut resistor = Resistor::new(PinId(1), PinId(2));
    resistor.set_resistance_ohms(1000.0);
    assert_eq!(resistor.resistance_ohms(), 1000.0);

    resistor.set_resistance_ohms(-5.0);
    assert!(resistor.resistance_ohms() > 0.0);
}

#[test]
fn test_board_rail_voltage_by_name() {
    let mut board = Esp32S3DevKit::new(PinId(0));
    board.set_powered(true);

    assert_eq!(
        board.rail_voltage(pin_named(&board, "3V3")),
        Some(RAIL_VOLTAGE_3V3)
    );
    assert_eq!(
        board.rail_voltage(pin_named(&board, "5V")),
        Some(RAIL_VOLTAGE_5V)
    );
    assert_eq!(board.rail_voltage(pin_named(&board, "GND")), Some(0.0));

    let io4 = pin_named(&board, "IO4");
    assert_eq!(board.rail_voltage(io4), None);
}

#[test]
fn test_unpowered_board_has_no_rail_voltage() {
    let mut board = Esp32S3DevKit::new(PinId(0));
    board.set_powered(false);

    assert_eq!(board.rail_voltage(pin_named(&board, "3V3")), None);
}

/// Solves a divider formed by two resistors between a board rail and ground.
fn solve_divider(top_ohms: f32, bottom_ohms: f32) -> f32 {
    let mut board = Esp32S3DevKit::new(PinId(0));
    board.set_powered(true);

    let vcc = pin_named(&board, "3V3");
    let gnd = pin_named(&board, "GND");

    let top = Resistor::with_resistance(PinId(900), PinId(901), top_ohms);
    let bottom = Resistor::with_resistance(PinId(902), PinId(903), bottom_ohms);

    let mut engine = Engine::new();

    let rail_node = engine.netlist.add_node();
    let mid_node = engine.netlist.add_node();
    let ground_node = engine.netlist.add_node();

    engine.netlist.connect_pin(vcc, rail_node);
    engine.netlist.connect_pin(gnd, ground_node);
    engine.netlist.connect_pin(top.terminal_a(), rail_node);
    engine.netlist.connect_pin(top.terminal_b(), mid_node);
    engine.netlist.connect_pin(bottom.terminal_a(), mid_node);
    engine.netlist.connect_pin(bottom.terminal_b(), ground_node);

    engine
        .network
        .claim_rail_pin(&engine.netlist, vcc, RAIL_VOLTAGE_3V3);
    engine.network.claim_rail_pin(&engine.netlist, gnd, 0.0);
    engine.network.add_resistor_pin(
        &engine.netlist,
        top.terminal_a(),
        top.terminal_b(),
        top.resistance_ohms(),
    );
    engine.network.add_resistor_pin(
        &engine.netlist,
        bottom.terminal_a(),
        bottom.terminal_b(),
        bottom.resistance_ohms(),
    );

    engine.solve_analog_network();
    engine.get_node_voltage(mid_node)
}

#[test]
fn test_even_divider_halves_the_rail() {
    let mid = solve_divider(1000.0, 1000.0);
    assert!(
        (mid - RAIL_VOLTAGE_3V3 / 2.0).abs() < 1e-3,
        "expected ~1.65 V, got {mid}"
    );
}

#[test]
fn test_unequal_divider_follows_ohm_ratio() {
    let mid = solve_divider(30_000.0, 10_000.0);
    let expected = RAIL_VOLTAGE_3V3 * 10_000.0 / 40_000.0;
    assert!(
        (mid - expected).abs() < 1e-3,
        "expected {expected}, got {mid}"
    );
}

#[test]
fn test_gpio_output_driving_a_divider_holds_its_level() {
    let mut board = Esp32S3DevKit::new(PinId(0));
    board.set_powered(true);

    let io4 = board.gpio_pin(4).expect("IO4");
    let gnd = pin_named(&board, "GND");

    let pull_down = Resistor::with_resistance(PinId(910), PinId(911), 10_000.0);

    let mut engine = Engine::new();

    let gpio_node = engine.netlist.add_node();
    let ground_node = engine.netlist.add_node();

    engine.netlist.connect_pin(io4, gpio_node);
    engine.netlist.connect_pin(gnd, ground_node);
    engine
        .netlist
        .connect_pin(pull_down.terminal_a(), gpio_node);
    engine
        .netlist
        .connect_pin(pull_down.terminal_b(), ground_node);

    engine.network.claim_rail_pin(&engine.netlist, gnd, 0.0);
    engine.network.add_resistor_pin(
        &engine.netlist,
        pull_down.terminal_a(),
        pull_down.terminal_b(),
        pull_down.resistance_ohms(),
    );

    let mut drives = std::collections::HashMap::new();
    drives.insert(io4, PinDrive::Driven(DigitalState::High));
    engine.evaluate_netlist_drives(&drives);
    engine.solve_analog_network();

    let node = engine.netlist.get_node_for_pin(io4).unwrap();
    assert!(
        engine.get_node_voltage(node) > 3.0,
        "driven output should stay near the logic rail, got {}",
        engine.get_node_voltage(node)
    );
}

#[test]
fn test_conflicting_rails_are_detected_and_left_unpinned() {
    let mut builder = sim_core::analog::NetworkBuilder::new();
    builder.claim_rail(NodeId(0), 3.3);
    builder.claim_rail(NodeId(0), 5.0);
    assert!(!builder.is_conflict_free());
}
