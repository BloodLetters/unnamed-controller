//! Data types and constants for the HD44780 1602 LCD module.

/// Number of visible character columns per row.
pub const LCD1602_COLS: usize = 16;

/// Number of character rows.
pub const LCD1602_ROWS: usize = 2;

/// Default 7-bit I2C address for the PCF8574 LCD backpack.
pub const LCD1602_DEFAULT_ADDRESS: u8 = 0x27;

/// Alternate 7-bit I2C address used by PCF8574A backpacks.
pub const LCD1602_ALT_ADDRESS: u8 = 0x3F;

/// Backlight color theme for the LCD1602 display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum LcdBacklightColor {
    #[default]
    Blue,
    Green,
}
