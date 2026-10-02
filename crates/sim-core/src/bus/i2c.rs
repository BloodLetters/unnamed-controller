use std::collections::HashMap;

/// A pending I2C master transaction produced by firmware for the host to route.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum I2cTransaction {
    /// Writes a payload to a slave device at the given 7-bit address.
    Write { address: u8, data: Vec<u8> },
    /// Requests a read of the given length from a slave device.
    Read { address: u8, length: u8 },
}

/// Represents errors that can occur during I2C bus transactions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum I2cError {
    DeviceNotFound(u8),
    WriteFailed,
    ReadFailed,
    BusBusy,
}

/// Interface for hardware components that communicate as an I2C slave device.
pub trait I2cDevice {
    /// Returns the 7-bit bus address of this device.
    fn address(&self) -> u8;

    /// Handles incoming data written by the I2C master.
    fn on_write(&mut self, data: &[u8]) -> Result<(), I2cError>;

    /// Fills the provided buffer with data requested by the I2C master.
    fn on_read(&mut self, buffer: &mut [u8]) -> Result<usize, I2cError>;
}

/// Simulated I2C communication bus manager.
pub struct I2cBus {
    devices: HashMap<u8, Box<dyn I2cDevice>>,
}

impl Default for I2cBus {
    fn default() -> Self {
        Self::new()
    }
}

impl I2cBus {
    /// Constructs a new empty I2C bus.
    pub fn new() -> Self {
        Self {
            devices: HashMap::new(),
        }
    }

    /// Attaches an I2C slave device to the bus.
    pub fn attach_device(&mut self, device: Box<dyn I2cDevice>) {
        self.devices.insert(device.address(), device);
    }

    /// Detaches an I2C slave device from the bus by its 7-bit address.
    pub fn detach_device(&mut self, address: u8) -> Option<Box<dyn I2cDevice>> {
        self.devices.remove(&address)
    }

    /// Performs an I2C write transaction to the target device.
    pub fn write(&mut self, address: u8, data: &[u8]) -> Result<(), I2cError> {
        let device = self
            .devices
            .get_mut(&address)
            .ok_or(I2cError::DeviceNotFound(address))?;
        device.on_write(data)
    }

    /// Performs an I2C read transaction from the target device.
    pub fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<usize, I2cError> {
        let device = self
            .devices
            .get_mut(&address)
            .ok_or(I2cError::DeviceNotFound(address))?;
        device.on_read(buffer)
    }

    /// Returns whether a device with the given address is connected to the bus.
    pub fn has_device(&self, address: u8) -> bool {
        self.devices.contains_key(&address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockMemoryDevice {
        addr: u8,
        storage: [u8; 16],
        pointer: usize,
    }

    impl MockMemoryDevice {
        fn new(addr: u8) -> Self {
            Self {
                addr,
                storage: [0; 16],
                pointer: 0,
            }
        }
    }

    impl I2cDevice for MockMemoryDevice {
        fn address(&self) -> u8 {
            self.addr
        }

        fn on_write(&mut self, data: &[u8]) -> Result<(), I2cError> {
            if data.is_empty() {
                return Ok(());
            }
            self.pointer = (data[0] as usize) % self.storage.len();
            for (idx, byte) in data[1..].iter().enumerate() {
                let target = (self.pointer + idx) % self.storage.len();
                self.storage[target] = *byte;
            }
            Ok(())
        }

        fn on_read(&mut self, buffer: &mut [u8]) -> Result<usize, I2cError> {
            for (idx, byte) in buffer.iter_mut().enumerate() {
                let target = (self.pointer + idx) % self.storage.len();
                *byte = self.storage[target];
            }
            Ok(buffer.len())
        }
    }

    #[test]
    fn test_i2c_write_and_read() {
        let mut bus = I2cBus::new();
        bus.attach_device(Box::new(MockMemoryDevice::new(0x3C)));

        let write_res = bus.write(0x3C, &[0x02, 0xAB, 0xCD]);
        assert!(write_res.is_ok());

        let mut read_buf = [0u8; 2];
        let read_res = bus.read(0x3C, &mut read_buf);
        assert_eq!(read_res, Ok(2));
        assert_eq!(read_buf, [0xAB, 0xCD]);
    }

    #[test]
    fn test_i2c_device_not_found() {
        let mut bus = I2cBus::new();
        let res = bus.write(0x40, &[0x01]);
        assert_eq!(res, Err(I2cError::DeviceNotFound(0x40)));
    }
}
