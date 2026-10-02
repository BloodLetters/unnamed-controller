use egui::{Color32, RichText, ScrollArea, Ui};
use sim_core::debugger::Watchpoint;

/// Renders an interactive 16-bytes-per-row memory hex editor with ASCII preview and watchpoint highlighting.
pub fn render_hex_view(
    ui: &mut Ui,
    memory: &mut [u8],
    watchpoints: &[Watchpoint],
    page_offset: &mut usize,
) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("Memory Hex Editor").strong());
        ui.separator();
        if ui.button("◀ Prev 256B").clicked() && *page_offset >= 256 {
            *page_offset -= 256;
        }
        ui.monospace(format!("Offset: 0x{:04X}", *page_offset));
        if ui.button("Next 256B ▶").clicked() && *page_offset + 256 < memory.len() {
            *page_offset += 256;
        }
    });

    ui.separator();

    let page_size = 256;
    let start = (*page_offset).min(memory.len());
    let end = (start + page_size).min(memory.len());

    ScrollArea::vertical()
        .id_source("hex_view_scroll")
        .max_height(160.0)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            egui::Grid::new("memory_hex_grid")
                .striped(true)
                .spacing([6.0, 3.0])
                .show(ui, |ui| {
                    ui.label(RichText::new("Offset").small());
                    for col in 0..16 {
                        ui.label(RichText::new(format!("{:02X}", col)).small());
                    }
                    ui.label(RichText::new("ASCII").small());
                    ui.end_row();

                    for row_addr in (start..end).step_by(16) {
                        ui.monospace(
                            RichText::new(format!("{:04X}:", row_addr))
                                .color(Color32::from_rgb(180, 180, 180)),
                        );

                        for col in 0..16 {
                            let curr_addr = row_addr + col;
                            if curr_addr < memory.len() {
                                let byte_val = memory[curr_addr];
                                let is_watched = watchpoints
                                    .iter()
                                    .any(|w| w.matches(curr_addr as u32, true));

                                let text_color = if is_watched {
                                    Color32::from_rgb(255, 179, 0)
                                } else {
                                    Color32::WHITE
                                };

                                let mut hex_str = format!("{:02X}", byte_val);
                                if ui
                                    .add(
                                        egui::TextEdit::singleline(&mut hex_str)
                                            .desired_width(22.0)
                                            .text_color(text_color),
                                    )
                                    .lost_focus()
                                    && let Ok(new_val) = u8::from_str_radix(&hex_str, 16)
                                {
                                    memory[curr_addr] = new_val;
                                }
                            } else {
                                ui.label("  ");
                            }
                        }

                        let ascii: String = (0..16)
                            .map(|col| {
                                let curr_addr = row_addr + col;
                                if curr_addr < memory.len() {
                                    let b = memory[curr_addr];
                                    if b.is_ascii_graphic() || b == b' ' {
                                        b as char
                                    } else {
                                        '.'
                                    }
                                } else {
                                    ' '
                                }
                            })
                            .collect();
                        ui.monospace(RichText::new(ascii).color(Color32::LIGHT_BLUE));
                        ui.end_row();
                    }
                });
        });
}
