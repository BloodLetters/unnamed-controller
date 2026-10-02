use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use sim_core::netlist::PinId;

use crate::board::esp32_s3::gpio::Esp32S3Gpio;
use crate::board::esp32_s3::pinout::{
    ESP32_S3_DEVKIT_PIN_COUNT, build_devkit_pins, build_gpio_map,
};
use crate::board::esp32_s3::soc::{I2cSharedQueue, boot_machine};
use crate::board::esp32_s3::vm::BytecodeVm;
use crate::board::types::{BoardDimensions, BoardRgbLed, FirmwareError, HeaderPin};

/// Complete simulation model for the ESP32-S3-DevKitC-1 development board.
pub struct Esp32S3DevKit {
    pub(crate) name: String,
    pub(crate) pins: Vec<PinId>,
    pub(crate) header_pins: Vec<HeaderPin>,
    pub(crate) gpio_to_pin: HashMap<u8, PinId>,
    pub(crate) pin_to_gpio: HashMap<PinId, u8>,
    pub(crate) gpio: Esp32S3Gpio,
    pub(crate) rgb_led: BoardRgbLed,
    pub(crate) boot_pressed: bool,
    pub(crate) reset_pressed: bool,
    pub(crate) is_powered: bool,
    pub(crate) serial_tx_buffer: String,
    pub(crate) firmware: Option<Vec<u8>>,
    pub(crate) dimensions: BoardDimensions,
    pub(crate) soc_machine: Option<s3emu::Machine>,
    pub(crate) vm: BytecodeVm,
    pub(crate) i2c_tx_buffer: Vec<(u8, Vec<u8>)>,
    pub(crate) shared_i2c_queue: I2cSharedQueue,
}

impl Default for Esp32S3DevKit {
    fn default() -> Self {
        Self::new(PinId(0))
    }
}

impl Esp32S3DevKit {
    /// Constructs an ESP32-S3 DevKit with sequential PinIds starting from a base identifier.
    pub fn new(base_pin: PinId) -> Self {
        let pins: Vec<PinId> = (0..ESP32_S3_DEVKIT_PIN_COUNT)
            .map(|i| PinId(base_pin.0 + i))
            .collect();
        Self::with_pins(pins)
    }

    /// Constructs an ESP32-S3 DevKit from an explicit set of 44 header PinIds.
    pub fn with_pins(pins: Vec<PinId>) -> Self {
        let header_pins = build_devkit_pins(&pins);
        let gpio_to_pin = build_gpio_map(&header_pins);
        let mut pin_to_gpio = HashMap::new();
        for (&gpio, &pin) in &gpio_to_pin {
            pin_to_gpio.insert(pin, gpio);
        }

        Self {
            name: "ESP32-S3 DevKitC-1".to_string(),
            pins,
            header_pins,
            gpio_to_pin,
            pin_to_gpio,
            gpio: Esp32S3Gpio::new(),
            rgb_led: BoardRgbLed::default(),
            boot_pressed: false,
            reset_pressed: false,
            is_powered: true,
            serial_tx_buffer: String::new(),
            firmware: None,
            dimensions: BoardDimensions {
                width: 140.0,
                height: 300.0,
                pin_pitch: 12.0,
            },
            soc_machine: None,
            vm: BytecodeVm::new(),
            i2c_tx_buffer: Vec::new(),
            shared_i2c_queue: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Extracts pending I2C transmission packets emitted by firmware.
    pub fn drain_i2c(&mut self) -> Vec<(u8, Vec<u8>)> {
        let mut packets = std::mem::take(&mut self.i2c_tx_buffer);
        if let Ok(mut q) = self.shared_i2c_queue.lock() {
            packets.append(&mut *q);
        }
        packets
    }

    /// Accessor for the onboard addressable RGB LED state (connected to GPIO48).
    pub fn rgb_led(&self) -> BoardRgbLed {
        self.rgb_led
    }

    /// Returns true if the 3.3V power indicator LED is illuminated.
    pub fn is_power_led_on(&self) -> bool {
        self.is_powered
    }

    /// Returns true if the onboard Boot button (GPIO0) is held down.
    pub fn is_boot_pressed(&self) -> bool {
        self.boot_pressed
    }

    /// Updates the state of the onboard Boot button and alters GPIO0 input level.
    pub fn set_boot_pressed(&mut self, pressed: bool) {
        self.boot_pressed = pressed;
        self.gpio.set_input(0, !pressed);
        if let Some(machine) = &mut self.soc_machine {
            machine.bus.periph.gpio.set_input(0, !pressed);
        }
    }

    /// Returns true if the onboard Reset / EN button is held down.
    pub fn is_reset_pressed(&self) -> bool {
        self.reset_pressed
    }

    /// Updates the state of the onboard Reset button, resetting the board on release.
    pub fn set_reset_pressed(&mut self, pressed: bool) {
        let was_pressed = self.reset_pressed;
        self.reset_pressed = pressed;
        if was_pressed && !pressed {
            self.reset_board();
        }
    }

    /// Internal board reset procedure.
    pub(crate) fn reset_board(&mut self) {
        self.gpio.reset();
        self.boot_pressed = false;
        self.reset_pressed = false;
        self.serial_tx_buffer.clear();
        self.rgb_led.turn_off();
        self.vm.reset();
        if let Ok(mut q) = self.shared_i2c_queue.lock() {
            q.clear();
        }

        if let Some(fw) = self.firmware.clone() {
            let _ = self.init_soc_machine(&fw);
        }
    }

    /// Reference to the internal GPIO bank controller.
    pub fn gpio_controller(&self) -> &Esp32S3Gpio {
        &self.gpio
    }

    /// Mutable reference to the internal GPIO bank controller.
    pub fn gpio_controller_mut(&mut self) -> &mut Esp32S3Gpio {
        &mut self.gpio
    }

    /// Reference to the currently loaded firmware binary slice.
    pub fn firmware(&self) -> Option<&[u8]> {
        self.firmware.as_deref()
    }

    /// Initializes and boots the internal Xtensa LX7 SoC emulator machine.
    pub(crate) fn init_soc_machine(&mut self, binary: &[u8]) -> Result<(), FirmwareError> {
        self.vm.reset();
        self.soc_machine = boot_machine(binary, &self.shared_i2c_queue)?;
        Ok(())
    }

    /// Reads RGB LED strip output from the SoC board bus or GPIO48.
    pub(crate) fn update_rgb_led(&mut self) {
        if let Some(machine) = &self.soc_machine
            && let Some((leds, _)) = machine.bus.board.leds()
            && let Some(&[r, g, b]) = leds.first()
        {
            self.rgb_led.set_rgb(r, g, b);
            return;
        }

        let io48_active = self.gpio.effective_level(48);
        if io48_active {
            self.rgb_led.set_rgb(255, 255, 255);
        } else {
            self.rgb_led.turn_off();
        }
    }
}
