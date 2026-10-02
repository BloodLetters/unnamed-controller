use egui::{Color32, RichText, ScrollArea, Ui};
use sim_core::debugger::DisassembledInstruction;

/// Renders the interactive MCU disassembly view with PC pointer and clickable breakpoint gutter.
pub fn render_disassembly_view(
    ui: &mut Ui,
    instructions: &[DisassembledInstruction],
    current_pc: usize,
    active_breakpoints: &[u32],
    on_toggle_breakpoint: &mut dyn FnMut(u32),
) {
    ui.label(RichText::new("Instruction Disassembly").strong());
    ui.separator();

    ScrollArea::vertical()
        .id_source("disasm_scroll")
        .max_height(180.0)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            egui::Grid::new("disasm_grid")
                .striped(true)
                .spacing([8.0, 4.0])
                .show(ui, |ui| {
                    ui.label(RichText::new("BP").small());
                    ui.label(RichText::new("Address").small());
                    ui.label(RichText::new("Hex").small());
                    ui.label(RichText::new("Instruction").small());
                    ui.end_row();

                    for inst in instructions {
                        let is_current = inst.address == current_pc;
                        let has_bp = active_breakpoints.contains(&(inst.address as u32));

                        let bp_text = if has_bp {
                            RichText::new("●").color(Color32::from_rgb(244, 67, 54))
                        } else {
                            RichText::new("○").color(Color32::from_gray(80))
                        };

                        if ui.button(bp_text).clicked() {
                            on_toggle_breakpoint(inst.address as u32);
                        }

                        let addr_color = if is_current {
                            Color32::from_rgb(255, 193, 7)
                        } else {
                            Color32::LIGHT_GRAY
                        };
                        let prefix = if is_current { "➜ " } else { "  " };
                        ui.monospace(
                            RichText::new(format!("{}{:04X}", prefix, inst.address))
                                .color(addr_color),
                        );

                        let hex_bytes = inst
                            .raw_bytes
                            .iter()
                            .map(|b| format!("{:02X}", b))
                            .collect::<Vec<_>>()
                            .join(" ");
                        ui.monospace(RichText::new(hex_bytes).color(Color32::from_gray(140)));

                        let inst_label = if inst.operands.is_empty() {
                            inst.mnemonic.clone()
                        } else {
                            format!("{:<10} {}", inst.mnemonic, inst.operands)
                        };
                        let inst_color = if is_current {
                            Color32::from_rgb(255, 235, 59)
                        } else {
                            Color32::WHITE
                        };
                        ui.monospace(RichText::new(inst_label).color(inst_color));

                        ui.end_row();
                    }
                });
        });
}
