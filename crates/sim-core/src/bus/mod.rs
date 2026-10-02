pub mod i2c;
pub mod spi;

pub use i2c::{I2cBus, I2cDevice, I2cError, I2cTransaction};
pub use spi::{SpiBus, SpiDevice, SpiError, SpiMode};

/// Supported communication bus types in the simulator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BusProtocol {
    I2C,
    SPI,
    UART,
}
