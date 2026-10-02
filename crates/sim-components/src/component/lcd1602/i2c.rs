//! I2C bus device protocol implementation for the LCD1602 PCF8574 expander.

use sim_core::bus::i2c::{I2cDevice, I2cError};

use super::Lcd1602;

impl I2cDevice for Lcd1602 {
    /// Returns the 7-bit slave address on the I2C bus.
    fn address(&self) -> u8 {
        self.address
    }

    /// Handles incoming I2C write frames from master.
    fn on_write(&mut self, data: &[u8]) -> Result<(), I2cError> {
        if data.is_empty() {
            return Ok(());
        }

        if data.len() == 2 && data[0] == 0x00 {
            self.write_command(data[1]);
            return Ok(());
        }

        if data[0] == 0x40
            && data.len() > 1
            && data[1..]
                .iter()
                .all(|&b| (32..=126).contains(&b) || b == b'\n')
        {
            for &c in &data[1..] {
                self.write_char(c);
            }
            return Ok(());
        }

        for &byte in data {
            self.process_pcf8574_byte(byte);
        }

        Ok(())
    }

    /// Fills the output buffer with zeroes as LCD1602 read is not implemented.
    fn on_read(&mut self, buffer: &mut [u8]) -> Result<usize, I2cError> {
        buffer.fill(0);
        Ok(buffer.len())
    }
}

impl Lcd1602 {
    /// Processes a single byte written to the PCF8574 GPIO port pins.
    fn process_pcf8574_byte(&mut self, byte: u8) {
        let current_en = (byte & 0x04) != 0;
        let rs = (byte & 0x01) != 0;
        self.backlight_on = (byte & 0x08) != 0;

        if self.last_en && !current_en {
            let nibble = byte & 0xF0;
            if !rs && nibble == 0x30 && self.high_nibble.is_none() {
                self.is_4bit = false;
                self.high_nibble = None;
            } else if !rs && nibble == 0x20 && !self.is_4bit {
                self.is_4bit = true;
                self.high_nibble = None;
            } else {
                match self.high_nibble {
                    None => {
                        self.high_nibble = Some(nibble);
                    }
                    Some(hi) => {
                        let full_byte = hi | (nibble >> 4);
                        self.high_nibble = None;
                        if rs {
                            self.write_char(full_byte);
                        } else {
                            self.write_command(full_byte);
                        }
                    }
                }
            }
        }

        self.last_en = current_en;
    }
}
