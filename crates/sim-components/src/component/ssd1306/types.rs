//! Constants and types for the SSD1306 OLED display controller.

/// Display width in pixels.
pub const SSD1306_WIDTH: usize = 128;
/// Display height in pixels.
pub const SSD1306_HEIGHT: usize = 64;
/// Number of 8-pixel high GDDRAM memory pages.
pub const SSD1306_PAGES: usize = 8;
/// Total capacity of GDDRAM in bytes.
pub const SSD1306_BUFFER_SIZE: usize = SSD1306_WIDTH * SSD1306_PAGES;
/// Standard 7-bit I2C address for SSD1306 modules.
pub const SSD1306_DEFAULT_ADDRESS: u8 = 0x3C;

/// Memory addressing mode governing automatic address pointer advancement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum AddressingMode {
    Horizontal,
    Vertical,
    #[default]
    Page,
}

/// Multi-byte command sequence currently expecting trailing operand bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PendingCommand {
    AddressingMode,
    ColumnAddressStart,
    ColumnAddressEnd(u8),
    PageAddressStart,
    PageAddressEnd(u8),
    Contrast,
    DisplayOffset,
    IgnoreSingleByte,
}
