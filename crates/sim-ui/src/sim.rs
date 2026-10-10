use std::collections::HashMap;

use sim_components::board::Board;
use sim_core::netlist::PinId;
use sim_core::pins::{DigitalState, PinState};

use crate::app::SimulatorApp;

/// Duration of one simulation tick in seconds, shared by all time-dependent subsystems.
pub const SIMULATION_TICK_SECONDS: f32 = 0.001;

/// Retrieves the computed electrical potential at a pin terminal.
pub fn get_pin_voltage(engine: &sim_core::engine::Engine, pin: PinId) -> f32 {
    if let Some(node) = engine.netlist.get_node_for_pin(pin) {
        if let Some(&v) = engine.node_voltages.get(&node) {
            return v;
        }
        match engine.get_node_state(node) {
            PinState::Digital(DigitalState::High) => 5.0,
            PinState::Digital(DigitalState::Low) => 0.0,
            PinState::Digital(DigitalState::HighZ) => 0.0,
            PinState::Analog(v) => v.0,
        }
    } else {
        0.0
    }
}

/// Rebuilds the resistive network from board supply rails and passive components.
///
/// Runs every tick so that changing a resistance or rewiring takes effect without an
/// explicit rebuild, and so the solver never retains branches from a deleted component.
fn build_resistive_network(app: &mut SimulatorApp) {
    app.engine.network.clear();

    let rail_pins: Vec<(sim_core::netlist::PinId, f32)> = app
        .esp32_s3_boards
        .iter()
        .flat_map(|(_, board)| {
            board
                .pins()
                .into_iter()
                .filter_map(|pin| board.rail_voltage(pin).map(|voltage| (pin, voltage)))
        })
        .collect();

    for (pin, voltage) in rail_pins {
        app.engine
            .network
            .claim_rail_pin(&app.engine.netlist, pin, voltage);
    }

    let netlist = &app.engine.netlist;
    for comp in &app.components {
        match &comp.instance {
            crate::component::ComponentInstance::Resistor(resistor) => {
                app.engine.network.add_resistor_pin(
                    netlist,
                    resistor.terminal_a(),
                    resistor.terminal_b(),
                    resistor.resistance_ohms(),
                );
            }
            crate::component::ComponentInstance::Button(button) => {
                app.engine.network.add_resistor_pin(
                    netlist,
                    button.pin_1a(),
                    button.pin_1b(),
                    0.001,
                );
                app.engine.network.add_resistor_pin(
                    netlist,
                    button.pin_2a(),
                    button.pin_2b(),
                    0.001,
                );
                if button.is_conductive() {
                    app.engine.network.add_resistor_pin(
                        netlist,
                        button.pin_1a(),
                        button.pin_2a(),
                        0.001,
                    );
                }
            }
            _ => {}
        }
    }
}

/// Evaluates all circuit states, simulation engine step, instrument sampling, and audio streaming.
pub fn tick_simulation(app: &mut SimulatorApp) {
    if !app.power_on {
        return;
    }

    let mut drives = HashMap::new();
    for (_, board) in &app.esp32_s3_boards {
        for pin in board.pins() {
            drives.insert(pin, board.pin_drive(pin));
        }
    }
    app.engine.evaluate_netlist_drives(&drives);

    build_resistive_network(app);

    app.engine.solve_analog_network();

    let mut i2c_packets = Vec::new();
    for (_, board) in &mut app.esp32_s3_boards {
        for pin in board.pins() {
            if let Some(node) = app.engine.netlist.get_node_for_pin(pin) {
                let digital = match app.engine.get_node_state(node) {
                    PinState::Digital(d) => d,
                    PinState::Analog(v) => {
                        if v.0 > 2.0 {
                            DigitalState::High
                        } else {
                            DigitalState::Low
                        }
                    }
                };
                board.notify_pin_change(pin, digital);
            }
        }
        board.step(SIMULATION_TICK_SECONDS);
        i2c_packets.extend(board.drain_i2c());
    }

    if !i2c_packets.is_empty() {
        for (addr, bytes) in &i2c_packets {
            for comp in &mut app.components {
                comp.handle_i2c_write(*addr, bytes);
            }
        }
    }

    let sim_time_us = (app.simulated_seconds * 1_000_000.0) as u64;
    for comp in &mut app.components {
        comp.update_electrical(&app.engine, sim_time_us, SIMULATION_TICK_SECONDS);
    }

    app.engine.tick();
    crate::instruments::sampling::sample_instruments(app);
    crate::audio::sampling::sample_audio(app);
}

const MAX_FRAME_SECONDS: f32 = 0.05;
const MAX_STEPS_PER_FRAME: u32 = 25;
const MAX_SIMULATION_BUDGET_MS: u128 = 8;

/// Advances the deterministic simulation by whole fixed steps based on elapsed frame time.
pub fn advance_simulation(app: &mut SimulatorApp, ctx: &egui::Context) {
    if !app.power_on {
        app.time_accumulator = 0.0;
        return;
    }

    let frame_start = std::time::Instant::now();
    let step = SIMULATION_TICK_SECONDS as f64;
    let elapsed = ctx.input(|input| input.stable_dt).min(MAX_FRAME_SECONDS) as f64;
    app.time_accumulator += elapsed;

    let mut steps = 0;
    while app.time_accumulator >= step && steps < MAX_STEPS_PER_FRAME {
        tick_simulation(app);
        app.time_accumulator -= step;
        app.simulated_seconds += step;
        steps += 1;

        if frame_start.elapsed().as_millis() >= MAX_SIMULATION_BUDGET_MS {
            break;
        }
    }

    if app.time_accumulator > step * 30.0 {
        app.time_accumulator = step * 30.0;
    }
}
