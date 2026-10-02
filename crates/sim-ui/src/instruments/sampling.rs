use sim_core::netlist::PinId;
use sim_core::pins::{DigitalState, PinState};

use crate::app::SimulatorApp;

/// Samples node voltages and logic levels for all active test instruments.
pub fn sample_instruments(app: &mut SimulatorApp) {
    if !app.power_on {
        return;
    }

    app.instruments.sim_time_us += 1_000;
    let t = app.instruments.sim_time_us;

    sample_oscilloscope(app, t);
    sample_logic_analyzer(app, t);
    sample_multimeter(app);
    sample_pwm_counter(app, t);
}

/// Ingests channel voltages into the 2-channel oscilloscope.
fn sample_oscilloscope(app: &mut SimulatorApp, time_us: u64) {
    let ch1_pin = app.instruments.oscilloscope.ch1.pin;
    let ch2_pin = app.instruments.oscilloscope.ch2.pin;

    let v1 = ch1_pin.map(|p| read_pin_voltage(app, p));
    let v2 = ch2_pin.map(|p| read_pin_voltage(app, p));

    app.instruments.oscilloscope.sample(time_us, v1, v2);
}

/// Ingests digital logic levels across 8 analyzer channels.
fn sample_logic_analyzer(app: &mut SimulatorApp, time_us: u64) {
    let mut states = [false; 8];
    for (i, ch) in app.instruments.logic_analyzer.channels.iter().enumerate() {
        if let Some(pin) = ch.pin {
            states[i] = read_pin_digital(app, pin);
        }
    }
    app.instruments.logic_analyzer.sample(time_us, &states);
}

/// Updates multimeter measurement calculations from probe pins.
fn sample_multimeter(app: &mut SimulatorApp) {
    let red_pin = app.instruments.dmm.probe_red;
    let black_pin = app.instruments.dmm.probe_black;

    let v_red = red_pin.map(|p| read_pin_voltage(app, p)).unwrap_or(0.0);
    let v_black = black_pin.map(|p| read_pin_voltage(app, p)).unwrap_or(0.0);

    let same_node = match (red_pin, black_pin) {
        (Some(p1), Some(p2)) => {
            let n1 = app.engine.netlist.get_node_for_pin(p1);
            let n2 = app.engine.netlist.get_node_for_pin(p2);
            n1.is_some() && n1 == n2
        }
        _ => false,
    };

    let freq = app.instruments.oscilloscope.calculate_frequency(1);
    let duty = if app.instruments.pwm.duty_percent > 0.0 {
        Some(app.instruments.pwm.duty_percent)
    } else {
        None
    };

    app.instruments
        .dmm
        .update(v_red, v_black, same_node, freq, duty);
}

/// Updates frequency counter and PWM duty cycle accumulator.
fn sample_pwm_counter(app: &mut SimulatorApp, time_us: u64) {
    if let Some(pin) = app.instruments.pwm.pin {
        let state = read_pin_digital(app, pin);
        app.instruments.pwm.update(time_us, state);
    }
}

/// Reads the analog voltage associated with a netlist pin terminal.
pub fn read_pin_voltage(app: &SimulatorApp, pin: PinId) -> f32 {
    if let Some(node) = app.engine.netlist.get_node_for_pin(pin) {
        let volt = app.engine.get_node_voltage(node);
        if volt > 0.001 {
            return volt;
        }
        match app.engine.get_node_state(node) {
            PinState::Digital(DigitalState::High) => 5.0,
            PinState::Digital(DigitalState::Low) => 0.0,
            PinState::Digital(DigitalState::HighZ) => 0.0,
            PinState::Analog(v) => v.0,
        }
    } else {
        0.0
    }
}

/// Reads the binary logic state associated with a netlist pin terminal.
pub fn read_pin_digital(app: &SimulatorApp, pin: PinId) -> bool {
    if let Some(node) = app.engine.netlist.get_node_for_pin(pin) {
        match app.engine.get_node_state(node) {
            PinState::Digital(DigitalState::High) => true,
            PinState::Digital(DigitalState::Low) | PinState::Digital(DigitalState::HighZ) => false,
            PinState::Analog(v) => v.0 >= 2.5,
        }
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::tick_simulation;

    #[test]
    fn test_sample_instruments_recording() {
        let mut app = SimulatorApp {
            power_on: true,
            ..Default::default()
        };
        app.instruments.oscilloscope.ch1.pin = Some(PinId(0));

        for _ in 0..10 {
            tick_simulation(&mut app);
        }

        assert!(!app.instruments.oscilloscope.ch1.buffer.samples.is_empty());
        assert!(app.instruments.sim_time_us > 0);
    }
}
