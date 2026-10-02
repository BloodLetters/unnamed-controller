/// Defines clock polarity and phase modes for SPI communication.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpiMode {
    Mode0,
    Mode1,
    Mode2,
    Mode3,
}

/// Represents errors that can occur during SPI transactions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpiError {
    DeviceNotSelected,
    BufferMismatch,
}

/// Interface for hardware peripherals connected to an SPI bus.
pub trait SpiDevice {
    /// Signals the device that its Chip Select line has transitioned low (active).
    fn select(&mut self);

    /// Signals the device that its Chip Select line has transitioned high (inactive).
    fn deselect(&mut self);

    /// Transfers a single byte full-duplex through the SPI interface.
    fn transfer_byte(&mut self, byte: u8) -> u8;
}

/// Simulated SPI controller for managing bus transfers.
pub struct SpiBus {
    pub mode: SpiMode,
}

impl Default for SpiBus {
    fn default() -> Self {
        Self::new(SpiMode::Mode0)
    }
}

impl SpiBus {
    /// Constructs a new SPI bus controller with the specified clock mode.
    pub fn new(mode: SpiMode) -> Self {
        Self { mode }
    }

    /// Performs a single-byte transfer with an active SPI device.
    pub fn transfer(&self, device: &mut dyn SpiDevice, byte: u8) -> u8 {
        device.transfer_byte(byte)
    }

    /// Performs a full-duplex buffer exchange with an active SPI device.
    pub fn transfer_slice(
        &self,
        device: &mut dyn SpiDevice,
        tx: &[u8],
        rx: &mut [u8],
    ) -> Result<(), SpiError> {
        if tx.len() != rx.len() {
            return Err(SpiError::BufferMismatch);
        }

        for (i, &tx_byte) in tx.iter().enumerate() {
            rx[i] = device.transfer_byte(tx_byte);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockSpiEchoDevice {
        selected: bool,
    }

    impl MockSpiEchoDevice {
        fn new() -> Self {
            Self { selected: false }
        }
    }

    impl SpiDevice for MockSpiEchoDevice {
        fn select(&mut self) {
            self.selected = true;
        }

        fn deselect(&mut self) {
            self.selected = false;
        }

        fn transfer_byte(&mut self, byte: u8) -> u8 {
            if self.selected {
                byte.wrapping_add(1)
            } else {
                0xFF
            }
        }
    }

    #[test]
    fn test_spi_transfer_selected() {
        let bus = SpiBus::new(SpiMode::Mode0);
        let mut dev = MockSpiEchoDevice::new();

        dev.select();
        let res = bus.transfer(&mut dev, 0x10);
        assert_eq!(res, 0x11);

        dev.deselect();
        let res_unselected = bus.transfer(&mut dev, 0x10);
        assert_eq!(res_unselected, 0xFF);
    }

    #[test]
    fn test_spi_transfer_slice() {
        let bus = SpiBus::new(SpiMode::Mode0);
        let mut dev = MockSpiEchoDevice::new();
        dev.select();

        let tx = [0x01, 0x02, 0x03];
        let mut rx = [0u8; 3];

        let result = bus.transfer_slice(&mut dev, &tx, &mut rx);
        assert!(result.is_ok());
        assert_eq!(rx, [0x02, 0x03, 0x04]);
    }
}
