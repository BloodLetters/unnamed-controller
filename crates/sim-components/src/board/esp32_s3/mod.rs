//! ESP32-S3 microcontroller and development board module.

pub mod board;
pub mod gpio;
pub mod pinout;
pub(crate) mod soc;
mod trait_impl;
pub mod vm;

pub use board::Esp32S3DevKit;
pub use gpio::{Esp32S3Gpio, GPIO_COUNT, GpioMode, GpioPinState, GpioPull};
pub use pinout::{ESP32_S3_DEVKIT_PIN_COUNT, build_devkit_pins, build_gpio_map};
pub use vm::BytecodeVm;
