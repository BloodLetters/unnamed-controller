use sim_core::pins::{DigitalState, PinDrive};

/// Operating mode of an individual GPIO pin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GpioMode {
    #[default]
    HighZ,
    Input,
    Output,
    OpenDrain,
}

/// Internal resistor pull configuration of an individual GPIO pin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GpioPull {
    #[default]
    Floating,
    PullUp,
    PullDown,
}

/// Logical electrical state of a single ESP32-S3 GPIO line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GpioPinState {
    pub mode: GpioMode,
    pub pull: GpioPull,
    pub output_level: bool,
    pub input_level: bool,
}

impl Default for GpioPinState {
    fn default() -> Self {
        Self {
            mode: GpioMode::HighZ,
            pull: GpioPull::Floating,
            output_level: false,
            input_level: false,
        }
    }
}

/// Total count of GPIO lines defined for the ESP32-S3 architecture (0..=48).
pub const GPIO_COUNT: usize = 49;

/// Controller managing the 49 GPIO lines, directions, pull resistors, and driver states.
#[derive(Debug, Clone)]
pub struct Esp32S3Gpio {
    pins: [GpioPinState; GPIO_COUNT],
}

impl Default for Esp32S3Gpio {
    fn default() -> Self {
        Self::new()
    }
}

impl Esp32S3Gpio {
    /// Constructs a fresh GPIO bank with default strapping and floating states.
    pub fn new() -> Self {
        let mut bank = Self {
            pins: [GpioPinState::default(); GPIO_COUNT],
        };
        bank.pins[0].pull = GpioPull::PullUp;
        bank.pins[0].input_level = true;
        bank
    }

    /// Resets all pins to initial post-reset states.
    pub fn reset(&mut self) {
        for state in &mut self.pins {
            *state = GpioPinState::default();
        }
        self.pins[0].pull = GpioPull::PullUp;
        self.pins[0].input_level = true;
    }

    /// Sets the direction mode of a specific GPIO line.
    pub fn set_mode(&mut self, gpio: u8, mode: GpioMode) {
        if let Some(state) = self.pins.get_mut(gpio as usize) {
            state.mode = mode;
        }
    }

    /// Configures internal pull resistors for a specific GPIO line.
    pub fn set_pull(&mut self, gpio: u8, pull: GpioPull) {
        if let Some(state) = self.pins.get_mut(gpio as usize) {
            state.pull = pull;
        }
    }

    /// Sets the digital output level written by the microcontroller.
    pub fn set_output(&mut self, gpio: u8, level: bool) {
        if let Some(state) = self.pins.get_mut(gpio as usize) {
            state.output_level = level;
        }
    }

    /// Updates the input level observed from external circuitry or buttons.
    pub fn set_input(&mut self, gpio: u8, level: bool) {
        if let Some(state) = self.pins.get_mut(gpio as usize) {
            state.input_level = level;
        }
    }

    /// Reads the input level recorded for a specific GPIO pin.
    pub fn read_input(&self, gpio: u8) -> bool {
        self.pins
            .get(gpio as usize)
            .is_some_and(|state| state.input_level)
    }

    /// Calculates the effective logical level of a GPIO pin considering drive and pull.
    pub fn effective_level(&self, gpio: u8) -> bool {
        let Some(state) = self.pins.get(gpio as usize) else {
            return false;
        };

        match state.mode {
            GpioMode::Output => state.output_level,
            GpioMode::OpenDrain => {
                if !state.output_level {
                    false
                } else {
                    state.input_level || matches!(state.pull, GpioPull::PullUp)
                }
            }
            GpioMode::Input | GpioMode::HighZ => match state.pull {
                GpioPull::PullUp => true,
                GpioPull::PullDown => false,
                GpioPull::Floating => state.input_level,
            },
        }
    }

    /// Computes the netlist pin drive configuration for a GPIO terminal.
    pub fn pin_drive(&self, gpio: u8) -> PinDrive {
        let Some(state) = self.pins.get(gpio as usize) else {
            return PinDrive::HighZ;
        };

        match state.mode {
            GpioMode::Output => {
                let digital = if state.output_level {
                    DigitalState::High
                } else {
                    DigitalState::Low
                };
                PinDrive::Driven(digital)
            }
            GpioMode::OpenDrain => {
                if !state.output_level {
                    PinDrive::Driven(DigitalState::Low)
                } else {
                    Self::resolve_pull_drive(state.pull)
                }
            }
            GpioMode::Input | GpioMode::HighZ => Self::resolve_pull_drive(state.pull),
        }
    }

    /// Resolves pull configuration into an equivalent PinDrive state.
    fn resolve_pull_drive(pull: GpioPull) -> PinDrive {
        match pull {
            GpioPull::PullUp => PinDrive::PullUp,
            GpioPull::PullDown => PinDrive::PullDown,
            GpioPull::Floating => PinDrive::HighZ,
        }
    }

    /// Synchronizes GPIO output levels and enable masks from the SoC peripheral registers.
    pub fn sync_from_soc(&mut self, out_mask: u64, enable_mask: u64) {
        for (i, state) in self.pins.iter_mut().enumerate() {
            let bit = 1u64 << i;
            let is_enabled = (enable_mask & bit) != 0;
            let level = (out_mask & bit) != 0;

            if is_enabled {
                state.mode = GpioMode::Output;
                state.output_level = level;
            } else if state.mode == GpioMode::Output {
                state.mode = GpioMode::Input;
            }
        }
    }

    /// Packs active input states into a 64-bit integer mask for the SoC peripheral bus.
    pub fn pack_soc_inputs(&self) -> u64 {
        let mut mask = 0u64;
        for (i, state) in self.pins.iter().enumerate() {
            if state.input_level || matches!(state.pull, GpioPull::PullUp) {
                mask |= 1u64 << i;
            }
        }
        mask
    }
}
