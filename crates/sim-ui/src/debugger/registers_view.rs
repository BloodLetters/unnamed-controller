use egui::{Color32, RichText, ScrollArea, Ui};
use sim_core::debugger::CpuRegisters;

/// Renders CPU register states with hexadecimal and decimal formatted inspection.
pub fn render_registers_view(ui: &mut Ui, regs: &mut CpuRegisters) {
    ui.label(RichText::new("CPU Core Registers").strong());
    ui.separator();

    ui.horizontal(|ui| {
        ui.monospace(format!("PC: 0x{:04X}", regs.pc));
        ui.monospace(format!("SP: 0x{:04X}", regs.sp));
        ui.monospace(format!("FLAGS: 0b{:08b}", regs.flags));
        ui.monospace(format!("CYCLES: {}", regs.cycles));
    });

    ui.separator();

    ScrollArea::vertical()
        .id_source("registers_scroll")
        .max_height(140.0)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            egui::Grid::new("registers_grid")
                .striped(true)
                .spacing([12.0, 4.0])
                .show(ui, |ui| {
                    ui.label(RichText::new("Register").small());
                    ui.label(RichText::new("Hex (u32)").small());
                    ui.label(RichText::new("Decimal").small());
                    ui.end_row();

                    for i in 0..16 {
                        ui.monospace(format!("R{:<2}", i));

                        let mut hex_str = format!("0x{:08X}", regs.r[i]);
                        if ui
                            .add(egui::TextEdit::singleline(&mut hex_str).desired_width(90.0))
                            .lost_focus()
                            && let Some(val_str) = hex_str.strip_prefix("0x")
                            && let Ok(val) = u32::from_str_radix(val_str, 16)
                        {
                            regs.r[i] = val;
                        }

                        ui.monospace(
                            RichText::new(format!("{}", regs.r[i]))
                                .color(Color32::from_rgb(100, 181, 246)),
                        );
                        ui.end_row();
                    }
                });
        });
}
