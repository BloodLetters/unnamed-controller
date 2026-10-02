//! I2C bus device interface implementation for SSD1306.

use sim_core::bus::i2c::{I2cDevice, I2cError};

use super::commands;
use super::{SSD1306_HEIGHT, Ssd1306};

impl I2cDevice for Ssd1306 {
    fn address(&self) -> u8 {
        self.address
    }

    fn on_write(&mut self, data: &[u8]) -> Result<(), I2cError> {
        if data.is_empty() {
            return Ok(());
        }

        if data.len() >= 2 && data[0] == 0x00 && data[1] == 0x01 {
            self.clear();
            return Ok(());
        }

        if data[0] == 0x40
            && data.len() > 1
            && data[1..]
                .iter()
                .all(|&b| (32..=126).contains(&b) || b == b'\n')
            && let Ok(text) = std::str::from_utf8(&data[1..])
        {
            self.draw_text(self.cursor_x, self.cursor_y, text);
            self.cursor_y = (self.cursor_y + 9) % SSD1306_HEIGHT;
            return Ok(());
        }

        let mut idx = 0;
        while idx < data.len() {
            let control = data[idx];
            idx += 1;

            let is_data = (control & 0x40) != 0;
            let single_byte = (control & 0x80) != 0;

            if single_byte {
                if idx < data.len() {
                    let val = data[idx];
                    idx += 1;
                    if is_data {
                        self.write_gddram_byte(val);
                    } else {
                        commands::execute_command(self, val);
                    }
                }
            } else {
                while idx < data.len() {
                    let val = data[idx];
                    idx += 1;
                    if is_data {
                        self.write_gddram_byte(val);
                    } else {
                        commands::execute_command(self, val);
                    }
                }
            }
        }

        Ok(())
    }

    fn on_read(&mut self, buffer: &mut [u8]) -> Result<usize, I2cError> {
        buffer.fill(0);
        Ok(buffer.len())
    }
}
