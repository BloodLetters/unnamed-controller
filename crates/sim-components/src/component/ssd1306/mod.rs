//! SSD1306 0.96" Monochrome 128x64 OLED display controller simulation.

pub mod commands;
pub mod font;
pub mod i2c;
pub mod types;

use sim_core::component::Component;
use sim_core::netlist::PinId;

pub use types::{
    AddressingMode, PendingCommand, SSD1306_BUFFER_SIZE, SSD1306_DEFAULT_ADDRESS, SSD1306_HEIGHT,
    SSD1306_PAGES, SSD1306_WIDTH,
};

/// Simulated SSD1306 OLED display controller with full GDDRAM matrix.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Ssd1306 {
    pub(crate) gnd: PinId,
    pub(crate) vcc: PinId,
    pub(crate) sda: PinId,
    pub(crate) scl: PinId,
    pub(crate) address: u8,
    pub(crate) gddram: Vec<u8>,
    pub(crate) addressing_mode: AddressingMode,
    pub(crate) col_start: u8,
    pub(crate) col_end: u8,
    pub(crate) col_ptr: u8,
    pub(crate) page_start: u8,
    pub(crate) page_end: u8,
    pub(crate) page_ptr: u8,
    pub(crate) display_on: bool,
    pub(crate) entire_display_on: bool,
    pub(crate) inverse_display: bool,
    pub(crate) contrast: u8,
    pub(crate) segment_remap: bool,
    pub(crate) com_scan_remapped: bool,
    pub(crate) display_offset: u8,
    pub(crate) display_start_line: u8,
    pub(crate) pending: Option<PendingCommand>,
    pub(crate) cursor_x: usize,
    pub(crate) cursor_y: usize,
}

impl Ssd1306 {
    /// Constructs a new SSD1306 display instance with assigned 4-pin header.
    pub fn new(gnd: PinId, vcc: PinId, sda: PinId, scl: PinId) -> Self {
        Self {
            gnd,
            vcc,
            sda,
            scl,
            address: SSD1306_DEFAULT_ADDRESS,
            gddram: vec![0u8; SSD1306_BUFFER_SIZE],
            addressing_mode: AddressingMode::Page,
            col_start: 0,
            col_end: 127,
            col_ptr: 0,
            page_start: 0,
            page_end: 7,
            page_ptr: 0,
            display_on: true,
            entire_display_on: false,
            inverse_display: false,
            contrast: 0x7F,
            segment_remap: false,
            com_scan_remapped: false,
            display_offset: 0,
            display_start_line: 0,
            pending: None,
            cursor_x: 0,
            cursor_y: 0,
        }
    }

    /// Ground terminal pin ID.
    pub fn gnd(&self) -> PinId {
        self.gnd
    }

    /// Power supply terminal pin ID.
    pub fn vcc(&self) -> PinId {
        self.vcc
    }

    /// I2C Serial Data pin ID.
    pub fn sda(&self) -> PinId {
        self.sda
    }

    /// I2C Serial Clock pin ID.
    pub fn scl(&self) -> PinId {
        self.scl
    }

    /// Returns a direct reference to the raw GDDRAM buffer.
    pub fn gddram(&self) -> &[u8] {
        &self.gddram
    }

    /// Checks if the display output is currently active.
    pub fn is_display_on(&self) -> bool {
        self.display_on
    }

    /// Clears the entire internal GDDRAM to blank.
    pub fn clear(&mut self) {
        self.gddram.fill(0);
        self.cursor_x = 0;
        self.cursor_y = 0;
    }

    /// Sets or clears a single pixel at (x, y) coordinates directly.
    pub fn set_pixel(&mut self, x: usize, y: usize, on: bool) {
        if x >= SSD1306_WIDTH || y >= SSD1306_HEIGHT {
            return;
        }
        let page = y / 8;
        let bit = y % 8;
        let index = page * SSD1306_WIDTH + x;
        if on {
            self.gddram[index] |= 1 << bit;
        } else {
            self.gddram[index] &= !(1 << bit);
        }
    }

    /// Draws a text string onto GDDRAM at (x, y) using 5x7 monospace font.
    pub fn draw_text(&mut self, x: usize, y: usize, text: &str) {
        font::draw_text_on_display(self, x, y, text);
    }

    /// Computes whether a screen pixel at (x, y) is illuminated.
    pub fn pixel_at(&self, x: usize, y: usize) -> bool {
        if !self.display_on || x >= SSD1306_WIDTH || y >= SSD1306_HEIGHT {
            return false;
        }
        if self.entire_display_on {
            return !self.inverse_display;
        }

        let effective_x = if self.segment_remap {
            SSD1306_WIDTH - 1 - x
        } else {
            x
        };

        let effective_y = if self.com_scan_remapped {
            SSD1306_HEIGHT - 1 - y
        } else {
            y
        };

        let shifted_y = (effective_y + self.display_offset as usize) % SSD1306_HEIGHT;
        let page = shifted_y / 8;
        let bit = shifted_y % 8;
        let index = page * SSD1306_WIDTH + effective_x;

        let raw_bit = (self.gddram[index] & (1 << bit)) != 0;
        if self.inverse_display {
            !raw_bit
        } else {
            raw_bit
        }
    }

    /// I2C 7-bit bus address.
    pub fn i2c_address(&self) -> u8 {
        self.address
    }

    /// Current memory addressing mode configured in controller.
    pub fn addressing_mode(&self) -> AddressingMode {
        self.addressing_mode
    }

    /// Whether display pixels are inverted.
    pub fn is_inverted(&self) -> bool {
        self.inverse_display
    }

    /// Whether entire display force-on mode is active.
    pub fn is_entire_on(&self) -> bool {
        self.entire_display_on
    }

    /// Current contrast setting (0-255).
    pub fn contrast(&self) -> u8 {
        self.contrast
    }

    /// Toggles display power output state on or off.
    pub fn toggle_power(&mut self) {
        self.display_on = !self.display_on;
    }

    /// Toggles pixel inversion mode.
    pub fn toggle_inverse(&mut self) {
        self.inverse_display = !self.inverse_display;
    }

    /// Writes a geometric and textual test pattern to GDDRAM for diagnostics.
    pub fn draw_test_pattern(&mut self) {
        self.clear();
        for x in 0..SSD1306_WIDTH {
            self.set_pixel(x, 0, true);
            self.set_pixel(x, SSD1306_HEIGHT - 1, true);
        }
        for y in 0..SSD1306_HEIGHT {
            self.set_pixel(0, y, true);
            self.set_pixel(SSD1306_WIDTH - 1, y, true);
        }
        self.draw_text(20, 14, "SSD1306 OLED");
        self.draw_text(26, 28, "128x64 PIXELS");
        self.draw_text(14, 42, "I2C ADDR: 0x3C");
    }

    /// Writes one byte to GDDRAM at the current pointer and advances according to mode.
    pub(crate) fn write_gddram_byte(&mut self, byte: u8) {
        let index = (self.page_ptr as usize) * SSD1306_WIDTH + (self.col_ptr as usize);
        if index < self.gddram.len() {
            self.gddram[index] = byte;
        }
        commands::advance_pointer(self);
    }
}

impl Component for Ssd1306 {
    fn name(&self) -> &str {
        "SSD1306 OLED Display"
    }

    fn pins(&self) -> Vec<PinId> {
        vec![self.gnd, self.vcc, self.sda, self.scl]
    }

    fn update(&mut self) {}
}
