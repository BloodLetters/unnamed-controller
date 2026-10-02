//! Command sequence parsing and address pointer advancement for SSD1306.

use super::Ssd1306;
use super::types::{AddressingMode, PendingCommand};

/// Executes a single command byte or resolves multi-byte arguments.
pub fn execute_command(ssd: &mut Ssd1306, cmd: u8) {
    if let Some(pending) = ssd.pending.take() {
        handle_pending_command(ssd, pending, cmd);
        return;
    }

    match cmd {
        0x00..=0x0F => ssd.col_ptr = (ssd.col_ptr & 0xF0) | (cmd & 0x0F),
        0x10..=0x1F => ssd.col_ptr = (ssd.col_ptr & 0x0F) | ((cmd & 0x0F) << 4),
        0x20 => ssd.pending = Some(PendingCommand::AddressingMode),
        0x21 => ssd.pending = Some(PendingCommand::ColumnAddressStart),
        0x22 => ssd.pending = Some(PendingCommand::PageAddressStart),
        0x40..=0x7F => ssd.display_start_line = cmd & 0x3F,
        0x81 => ssd.pending = Some(PendingCommand::Contrast),
        0x8D | 0xA8 | 0xD5 | 0xD9 | 0xDA | 0xDB => {
            ssd.pending = Some(PendingCommand::IgnoreSingleByte);
        }
        0xA0 => ssd.segment_remap = false,
        0xA1 => ssd.segment_remap = true,
        0xA4 => ssd.entire_display_on = false,
        0xA5 => ssd.entire_display_on = true,
        0xA6 => ssd.inverse_display = false,
        0xA7 => ssd.inverse_display = true,
        0xAE => ssd.display_on = false,
        0xAF => ssd.display_on = true,
        0xB0..=0xB7 => ssd.page_ptr = cmd & 0x07,
        0xC0 => ssd.com_scan_remapped = false,
        0xC8 => ssd.com_scan_remapped = true,
        0xD3 => ssd.pending = Some(PendingCommand::DisplayOffset),
        _ => {}
    }
}

/// Processes argument byte for pending multi-byte commands.
fn handle_pending_command(ssd: &mut Ssd1306, pending: PendingCommand, arg: u8) {
    match pending {
        PendingCommand::AddressingMode => {
            ssd.addressing_mode = match arg & 0x03 {
                0 => AddressingMode::Horizontal,
                1 => AddressingMode::Vertical,
                _ => AddressingMode::Page,
            };
        }
        PendingCommand::ColumnAddressStart => {
            ssd.pending = Some(PendingCommand::ColumnAddressEnd(arg.min(127)));
        }
        PendingCommand::ColumnAddressEnd(start) => {
            ssd.col_start = start;
            ssd.col_end = arg.min(127);
            ssd.col_ptr = start;
        }
        PendingCommand::PageAddressStart => {
            ssd.pending = Some(PendingCommand::PageAddressEnd(arg.min(7)));
        }
        PendingCommand::PageAddressEnd(start) => {
            ssd.page_start = start;
            ssd.page_end = arg.min(7);
            ssd.page_ptr = start;
        }
        PendingCommand::Contrast => ssd.contrast = arg,
        PendingCommand::DisplayOffset => ssd.display_offset = arg & 0x3F,
        PendingCommand::IgnoreSingleByte => {}
    }
}

/// Advances column and page pointers according to the active addressing mode.
pub fn advance_pointer(ssd: &mut Ssd1306) {
    match ssd.addressing_mode {
        AddressingMode::Horizontal => {
            if ssd.col_ptr >= ssd.col_end {
                ssd.col_ptr = ssd.col_start;
                if ssd.page_ptr >= ssd.page_end {
                    ssd.page_ptr = ssd.page_start;
                } else {
                    ssd.page_ptr += 1;
                }
            } else {
                ssd.col_ptr += 1;
            }
        }
        AddressingMode::Vertical => {
            if ssd.page_ptr >= ssd.page_end {
                ssd.page_ptr = ssd.page_start;
                if ssd.col_ptr >= ssd.col_end {
                    ssd.col_ptr = ssd.col_start;
                } else {
                    ssd.col_ptr += 1;
                }
            } else {
                ssd.page_ptr += 1;
            }
        }
        AddressingMode::Page => {
            if ssd.col_ptr >= ssd.col_end {
                ssd.col_ptr = ssd.col_start;
            } else {
                ssd.col_ptr += 1;
            }
        }
    }
}
