//! Board and Component trait implementations for the ESP32-S3 DevKit.

use sim_core::component::Component;
use sim_core::netlist::PinId;
use sim_core::pins::{DigitalState, PinDrive};

use super::board::Esp32S3DevKit;
use crate::board::traits::Board;
use crate::board::types::{
    BoardDimensions, BoardType, FirmwareError, HeaderPin, RAIL_VOLTAGE_3V3, RAIL_VOLTAGE_5V,
};

/// Resolves the potential of a named supply rail terminal from its silkscreen label.
fn rail_voltage_for_name(name: &str) -> f32 {
    match name {
        "5V" => RAIL_VOLTAGE_5V,
        "3V3" => RAIL_VOLTAGE_3V3,
        _ => RAIL_VOLTAGE_3V3,
    }
}

impl Board for Esp32S3DevKit {
    fn name(&self) -> &str {
        &self.name
    }

    fn board_type(&self) -> BoardType {
        BoardType::Esp32S3DevKit
    }

    fn pins(&self) -> Vec<PinId> {
        self.pins.clone()
    }

    fn header_pins(&self) -> &[HeaderPin] {
        &self.header_pins
    }

    fn pin_name(&self, pin: PinId) -> Option<&str> {
        self.header_pins
            .iter()
            .find(|p| p.pin_id == pin)
            .map(|p| p.name.as_str())
    }

    fn gpio_pin(&self, gpio: u8) -> Option<PinId> {
        self.gpio_to_pin.get(&gpio).copied()
    }

    fn power_terminals(&self) -> (Option<PinId>, Vec<PinId>) {
        let mut power = None;
        let mut grounds = Vec::new();
        for pin in &self.header_pins {
            if pin.is_power && power.is_none() {
                power = Some(pin.pin_id);
            } else if pin.is_ground {
                grounds.push(pin.pin_id);
            }
        }
        (power, grounds)
    }

    fn dimensions(&self) -> BoardDimensions {
        self.dimensions
    }

    fn is_powered(&self) -> bool {
        self.is_powered
    }

    fn set_powered(&mut self, powered: bool) {
        self.is_powered = powered;
    }

    fn reset(&mut self) {
        self.gpio.reset();
        self.boot_pressed = false;
        self.reset_pressed = false;
        self.serial_tx_buffer.clear();
        self.i2c_tx_buffer.clear();
        self.rgb_led.turn_off();

        if let Some(fw) = self.firmware.clone() {
            let _ = self.init_soc_machine(&fw);
        }
    }

    fn step(&mut self, dt_seconds: f32) {
        if !self.is_powered || self.reset_pressed {
            return;
        }

        if let Some(machine) = &mut self.soc_machine {
            let cycles = ((dt_seconds as f64) * 240_000_000.0).clamp(5_000.0, 300_000.0) as u64;
            machine.run(cycles);

            if !machine.console.uart0.is_empty() {
                let text = String::from_utf8_lossy(&machine.console.uart0);
                self.serial_tx_buffer.push_str(&text);
                machine.console.uart0.clear();
            }

            let out_mask = machine.bus.periph.gpio.out;
            let enable_mask = machine.bus.periph.gpio.enable;
            self.gpio.sync_from_soc(out_mask, enable_mask);
        } else if let Some(code) = &self.firmware {
            self.vm.step(
                code,
                dt_seconds,
                &mut self.gpio,
                &mut self.serial_tx_buffer,
                &mut self.i2c_tx_buffer,
            );
        }

        self.update_rgb_led();
    }

    fn load_firmware(&mut self, binary: &[u8]) -> Result<(), FirmwareError> {
        if binary.is_empty() {
            return Err(FirmwareError::EmptyBinary);
        }
        if binary.len() > 16 * 1024 * 1024 {
            return Err(FirmwareError::BinaryTooLarge {
                size: binary.len(),
                max: 16 * 1024 * 1024,
            });
        }

        self.firmware = Some(binary.to_vec());
        self.init_soc_machine(binary)
    }

    fn send_serial_input(&mut self, text: &str) {
        if let Some(machine) = &mut self.soc_machine {
            machine.bus.periph.uart[0].host_input(text.as_bytes());
        }
    }

    fn serial_output(&self) -> &str {
        &self.serial_tx_buffer
    }

    fn clear_serial_output(&mut self) {
        self.serial_tx_buffer.clear();
    }

    fn pin_drive(&self, pin: PinId) -> PinDrive {
        let Some(header_pin) = self.header_pins.iter().find(|p| p.pin_id == pin) else {
            return PinDrive::HighZ;
        };

        if header_pin.is_ground {
            return PinDrive::Driven(DigitalState::Low);
        }

        if header_pin.is_power {
            return if self.is_powered {
                PinDrive::Driven(DigitalState::High)
            } else {
                PinDrive::HighZ
            };
        }

        if header_pin.name == "RST" {
            return if self.reset_pressed {
                PinDrive::Driven(DigitalState::Low)
            } else {
                PinDrive::PullUp
            };
        }

        if let Some(gpio) = header_pin.gpio {
            if !self.is_powered || self.reset_pressed {
                return PinDrive::HighZ;
            }
            return self.gpio.pin_drive(gpio);
        }

        PinDrive::HighZ
    }

    fn rail_voltage(&self, pin: PinId) -> Option<f32> {
        if !self.is_powered {
            return None;
        }

        let header_pin = self.header_pins.iter().find(|p| p.pin_id == pin)?;

        if header_pin.is_ground {
            return Some(0.0);
        }

        if header_pin.is_power {
            return Some(rail_voltage_for_name(&header_pin.name));
        }

        None
    }

    fn notify_pin_change(&mut self, pin: PinId, state: DigitalState) {
        if let Some(&gpio) = self.pin_to_gpio.get(&pin) {
            let level = matches!(state, DigitalState::High);
            self.gpio.set_input(gpio, level);
            if let Some(machine) = &mut self.soc_machine {
                machine.bus.periph.gpio.set_input(gpio, level);
            }
        }
    }
}

impl Component for Esp32S3DevKit {
    fn name(&self) -> &str {
        &self.name
    }

    fn pins(&self) -> Vec<PinId> {
        self.pins.clone()
    }

    fn update(&mut self) {
        self.step(0.001);
    }
}
