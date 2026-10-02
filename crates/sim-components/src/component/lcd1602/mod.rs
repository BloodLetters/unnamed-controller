//! HD44780 1602 LCD character display with PCF8574 I2C backpack.

pub mod i2c;
pub mod types;

use sim_core::netlist::PinId;
pub use types::{
    LCD1602_ALT_ADDRESS, LCD1602_COLS, LCD1602_DEFAULT_ADDRESS, LCD1602_ROWS, LcdBacklightColor,
};

/// Simulated 16x2 character alphanumeric display.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Lcd1602 {
    gnd: PinId,
    vcc: PinId,
    sda: PinId,
    scl: PinId,
    pub(crate) address: u8,
    pub(crate) backlight_on: bool,
    pub(crate) display_on: bool,
    pub(crate) cursor_on: bool,
    pub(crate) blink_on: bool,
    pub(crate) cursor_col: usize,
    pub(crate) cursor_row: usize,
    pub(crate) ddram: [[u8; LCD1602_COLS]; LCD1602_ROWS],
    pub(crate) last_en: bool,
    pub(crate) high_nibble: Option<u8>,
    pub(crate) is_4bit: bool,
    pub(crate) backlight_color: LcdBacklightColor,
}

impl Lcd1602 {
    /// Constructs a new 1602 LCD display with default 0x27 address and blue backlight.
    pub fn new(gnd: PinId, vcc: PinId, sda: PinId, scl: PinId) -> Self {
        Self {
            gnd,
            vcc,
            sda,
            scl,
            address: LCD1602_DEFAULT_ADDRESS,
            backlight_on: true,
            display_on: true,
            cursor_on: false,
            blink_on: false,
            cursor_col: 0,
            cursor_row: 0,
            ddram: [[b' '; LCD1602_COLS]; LCD1602_ROWS],
            last_en: false,
            high_nibble: None,
            is_4bit: true,
            backlight_color: LcdBacklightColor::Blue,
        }
    }

    /// Sets the custom 7-bit I2C slave address.
    pub fn with_address(mut self, address: u8) -> Self {
        self.address = address;
        self
    }

    /// Returns the ground pin terminal identifier.
    pub fn gnd(&self) -> PinId {
        self.gnd
    }

    /// Returns the power supply pin terminal identifier.
    pub fn vcc(&self) -> PinId {
        self.vcc
    }

    /// Returns the I2C serial data pin terminal identifier.
    pub fn sda(&self) -> PinId {
        self.sda
    }

    /// Returns the I2C serial clock pin terminal identifier.
    pub fn scl(&self) -> PinId {
        self.scl
    }

    /// Returns the 7-bit I2C device address.
    pub fn address(&self) -> u8 {
        self.address
    }

    /// Returns whether the backlight is currently illuminated.
    pub fn is_backlight_on(&self) -> bool {
        self.backlight_on
    }

    /// Returns whether the character display layer is turned on.
    pub fn is_display_on(&self) -> bool {
        self.display_on
    }

    /// Returns the backlight color theme.
    pub fn backlight_color(&self) -> LcdBacklightColor {
        self.backlight_color
    }

    /// Sets the backlight color theme.
    pub fn set_backlight_color(&mut self, color: LcdBacklightColor) {
        self.backlight_color = color;
    }

    /// Clears the display screen by writing spaces to all DDRAM cells and resetting cursor.
    pub fn clear(&mut self) {
        self.ddram = [[b' '; LCD1602_COLS]; LCD1602_ROWS];
        self.cursor_col = 0;
        self.cursor_row = 0;
    }

    /// Returns the cursor to the first column of the first row.
    pub fn home(&mut self) {
        self.cursor_col = 0;
        self.cursor_row = 0;
    }

    /// Repositions the active cursor coordinate within 16x2 bounds.
    pub fn set_cursor(&mut self, col: usize, row: usize) {
        self.cursor_col = col.min(LCD1602_COLS.saturating_sub(1));
        self.cursor_row = row.min(LCD1602_ROWS.saturating_sub(1));
    }

    /// Writes an ASCII character at current cursor position and advances cursor.
    pub fn write_char(&mut self, c: u8) {
        if self.cursor_row < LCD1602_ROWS && self.cursor_col < LCD1602_COLS {
            self.ddram[self.cursor_row][self.cursor_col] = c;
            self.cursor_col += 1;
            if self.cursor_col >= LCD1602_COLS {
                self.cursor_col = 0;
                self.cursor_row = (self.cursor_row + 1) % LCD1602_ROWS;
            }
        }
    }

    /// Executes an HD44780 standard command code.
    pub fn write_command(&mut self, cmd: u8) {
        if cmd == 0x01 {
            self.clear();
        } else if (cmd & 0xFE) == 0x02 {
            self.home();
        } else if (cmd & 0xF8) == 0x08 {
            self.display_on = (cmd & 0x04) != 0;
            self.cursor_on = (cmd & 0x02) != 0;
            self.blink_on = (cmd & 0x01) != 0;
        } else if (cmd & 0x80) != 0 {
            let addr = cmd & 0x7F;
            if addr >= 0x40 {
                self.cursor_row = 1;
                self.cursor_col = (addr - 0x40) as usize % LCD1602_COLS;
            } else {
                self.cursor_row = 0;
                self.cursor_col = addr as usize % LCD1602_COLS;
            }
        }
    }

    /// Returns the character byte at the specified grid position.
    pub fn get_char(&self, col: usize, row: usize) -> u8 {
        if row < LCD1602_ROWS && col < LCD1602_COLS {
            self.ddram[row][col]
        } else {
            b' '
        }
    }

    /// Returns a full line of characters as a trimmed display string.
    pub fn get_row_str(&self, row: usize) -> String {
        if row < LCD1602_ROWS {
            String::from_utf8_lossy(&self.ddram[row]).to_string()
        } else {
            String::new()
        }
    }
}

impl sim_core::component::Component for Lcd1602 {
    fn name(&self) -> &str {
        "HD44780 1602 LCD Display"
    }

    fn pins(&self) -> Vec<PinId> {
        vec![self.gnd, self.vcc, self.sda, self.scl]
    }

    fn update(&mut self) {}
}
