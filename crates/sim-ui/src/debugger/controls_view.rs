use egui::{Color32, RichText, Ui};
use sim_core::debugger::{DebuggerCore, HaltReason, WatchpointKind};

/// User actions triggered from the debugger control toolbar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DebuggerAction {
    Resume,
    StepInto,
    StepOver,
    Pause,
    Reset,
}

/// Parameters passed into the debugger controls toolbar and status view.
pub struct ControlParams<'a> {
    pub is_running: bool,
    pub current_pc: u32,
    pub new_wp_addr: &'a mut String,
    pub new_wp_size: &'a mut usize,
    pub new_wp_kind: &'a mut WatchpointKind,
}

/// Renders debugger execution controls, status banner, and breakpoint/watchpoint management.
pub fn render_controls_view(
    ui: &mut Ui,
    debugger: &mut DebuggerCore,
    params: ControlParams<'_>,
) -> Option<DebuggerAction> {
    let mut triggered_action = None;

    ui.horizontal(|ui| {
        if debugger.paused || !params.is_running {
            if ui.button(RichText::new("▶ Continue").strong()).clicked() {
                triggered_action = Some(DebuggerAction::Resume);
            }
            if ui.button("⤵ Step Into").clicked() {
                triggered_action = Some(DebuggerAction::StepInto);
            }
            if ui.button("↷ Step Over").clicked() {
                triggered_action = Some(DebuggerAction::StepOver);
            }
        } else if ui.button(RichText::new("⏸ Pause").strong()).clicked() {
            triggered_action = Some(DebuggerAction::Pause);
        }

        if ui.button("⟳ Reset").clicked() {
            triggered_action = Some(DebuggerAction::Reset);
        }

        ui.separator();

        match debugger.halt_reason {
            HaltReason::None => {
                let status_text = if debugger.paused || !params.is_running {
                    "PAUSED"
                } else {
                    "RUNNING"
                };
                let color = if debugger.paused || !params.is_running {
                    Color32::from_rgb(255, 179, 0)
                } else {
                    Color32::from_rgb(76, 175, 80)
                };
                ui.label(RichText::new(status_text).color(color).strong());
            }
            HaltReason::Step => {
                ui.label(
                    RichText::new(format!("STEPPED AT 0x{:04X}", params.current_pc))
                        .color(Color32::from_rgb(33, 150, 243))
                        .strong(),
                );
            }
            HaltReason::Breakpoint(addr) => {
                ui.label(
                    RichText::new(format!("BREAKPOINT HIT AT 0x{:04X}", addr))
                        .color(Color32::from_rgb(244, 67, 54))
                        .strong(),
                );
            }
            HaltReason::Watchpoint { address, is_write } => {
                let mode = if is_write { "WRITE" } else { "READ" };
                ui.label(
                    RichText::new(format!("WATCHPOINT HIT AT 0x{:04X} ({})", address, mode))
                        .color(Color32::from_rgb(255, 152, 0))
                        .strong(),
                );
            }
            HaltReason::HaltInstruction => {
                ui.label(
                    RichText::new("HALT INSTRUCTION")
                        .color(Color32::from_gray(180))
                        .strong(),
                );
            }
        }
    });

    ui.separator();

    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(RichText::new("Active Breakpoints").strong());
            if debugger.breakpoints.is_empty() {
                ui.label(RichText::new("No breakpoints set (click disassembly gutter)").small());
            } else {
                let mut to_remove = None;
                for (idx, bp) in debugger.breakpoints.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut bp.enabled, format!("0x{:04X}", bp.address));
                        ui.monospace(format!("hits: {}", bp.hit_count));
                        if ui.button("✖").clicked() {
                            to_remove = Some(idx);
                        }
                    });
                }
                if let Some(idx) = to_remove {
                    debugger.breakpoints.remove(idx);
                }
            }
        });

        ui.separator();

        ui.vertical(|ui| {
            ui.label(RichText::new("Memory Watchpoints").strong());

            ui.horizontal(|ui| {
                ui.label("Addr (hex): 0x");
                ui.add(egui::TextEdit::singleline(params.new_wp_addr).desired_width(50.0));
                ui.label("Size:");
                egui::ComboBox::from_id_source("wp_size_combo")
                    .selected_text(format!("{}B", *params.new_wp_size))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(params.new_wp_size, 1, "1B");
                        ui.selectable_value(params.new_wp_size, 2, "2B");
                        ui.selectable_value(params.new_wp_size, 4, "4B");
                    });
                egui::ComboBox::from_id_source("wp_kind_combo")
                    .selected_text(match params.new_wp_kind {
                        WatchpointKind::Read => "Read",
                        WatchpointKind::Write => "Write",
                        WatchpointKind::Access => "Access",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(params.new_wp_kind, WatchpointKind::Write, "Write");
                        ui.selectable_value(params.new_wp_kind, WatchpointKind::Read, "Read");
                        ui.selectable_value(params.new_wp_kind, WatchpointKind::Access, "Access");
                    });
                if ui.button("+ Add").clicked()
                    && let Ok(addr) = u32::from_str_radix(params.new_wp_addr.trim(), 16)
                {
                    debugger.add_watchpoint(addr, *params.new_wp_size, *params.new_wp_kind);
                    params.new_wp_addr.clear();
                }
            });

            if debugger.watchpoints.is_empty() {
                ui.label(RichText::new("No watchpoints configured").small());
            } else {
                let mut to_remove = None;
                for (idx, wp) in debugger.watchpoints.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        let kind_str = match wp.kind {
                            WatchpointKind::Read => "R",
                            WatchpointKind::Write => "W",
                            WatchpointKind::Access => "RW",
                        };
                        ui.checkbox(
                            &mut wp.enabled,
                            format!("0x{:04X} [{}B, {}]", wp.address, wp.size, kind_str),
                        );
                        ui.monospace(format!("hits: {}", wp.hit_count));
                        if ui.button("✖").clicked() {
                            to_remove = Some(idx);
                        }
                    });
                }
                if let Some(idx) = to_remove {
                    debugger.watchpoints.remove(idx);
                }
            }
        });
    });

    triggered_action
}
