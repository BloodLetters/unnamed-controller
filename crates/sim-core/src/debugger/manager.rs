use crate::debugger::types::{Breakpoint, HaltReason, Watchpoint, WatchpointKind};

/// Core debugger manager handling breakpoints, watchpoints, and execution control.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DebuggerCore {
    pub breakpoints: Vec<Breakpoint>,
    pub watchpoints: Vec<Watchpoint>,
    pub halt_reason: HaltReason,
    pub paused: bool,
    pub step_requested: bool,
    pub step_over_target: Option<u32>,
}

impl DebuggerCore {
    /// Creates a new debugger manager.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a breakpoint at the specified target address.
    pub fn add_breakpoint(&mut self, address: u32) -> bool {
        if self.breakpoints.iter().any(|b| b.address == address) {
            false
        } else {
            self.breakpoints.push(Breakpoint::new(address));
            true
        }
    }

    /// Removes a breakpoint at the specified target address.
    pub fn remove_breakpoint(&mut self, address: u32) -> bool {
        let initial_len = self.breakpoints.len();
        self.breakpoints.retain(|b| b.address != address);
        self.breakpoints.len() < initial_len
    }

    /// Toggles a breakpoint at the specified address.
    pub fn toggle_breakpoint(&mut self, address: u32) {
        if !self.remove_breakpoint(address) {
            self.add_breakpoint(address);
        }
    }

    /// Checks if a breakpoint is active at the given address.
    pub fn has_breakpoint(&self, address: u32) -> bool {
        self.breakpoints
            .iter()
            .any(|b| b.address == address && b.enabled)
    }

    /// Adds a memory watchpoint.
    pub fn add_watchpoint(&mut self, address: u32, size: usize, kind: WatchpointKind) {
        self.remove_watchpoint(address);
        self.watchpoints.push(Watchpoint::new(address, size, kind));
    }

    /// Removes a watchpoint starting at the specified address.
    pub fn remove_watchpoint(&mut self, address: u32) -> bool {
        let initial_len = self.watchpoints.len();
        self.watchpoints.retain(|w| w.address != address);
        self.watchpoints.len() < initial_len
    }

    /// Verifies if a memory access triggers an active watchpoint, halting execution if matched.
    pub fn check_memory_access(&mut self, address: u32, is_write: bool) -> bool {
        for w in &mut self.watchpoints {
            if w.matches(address, is_write) {
                w.hit_count += 1;
                self.halt_reason = HaltReason::Watchpoint { address, is_write };
                self.paused = true;
                return true;
            }
        }
        false
    }

    /// Checks if the Program Counter matches an active breakpoint or step request before instruction execution.
    pub fn check_pc_before_step(&mut self, pc: u32) -> bool {
        if self.step_requested {
            self.step_requested = false;
            self.halt_reason = HaltReason::Step;
            self.paused = true;
            return true;
        }

        if let Some(target) = self.step_over_target
            && pc == target
        {
            self.step_over_target = None;
            self.halt_reason = HaltReason::Step;
            self.paused = true;
            return true;
        }

        for bp in &mut self.breakpoints {
            if bp.enabled && bp.address == pc {
                bp.hit_count += 1;
                self.halt_reason = HaltReason::Breakpoint(pc);
                self.paused = true;
                return true;
            }
        }

        false
    }

    /// Requests a single-step execution.
    pub fn request_step(&mut self) {
        self.paused = false;
        self.step_requested = true;
        self.halt_reason = HaltReason::None;
    }

    /// Resumes normal continuous execution.
    pub fn resume(&mut self) {
        self.paused = false;
        self.step_requested = false;
        self.step_over_target = None;
        self.halt_reason = HaltReason::None;
    }

    /// Manually pauses execution.
    pub fn pause(&mut self) {
        self.paused = true;
        self.halt_reason = HaltReason::None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_breakpoint_management() {
        let mut dbg = DebuggerCore::new();
        assert!(dbg.add_breakpoint(0x10));
        assert!(!dbg.add_breakpoint(0x10));
        assert!(dbg.has_breakpoint(0x10));

        assert!(dbg.check_pc_before_step(0x10));
        assert!(dbg.paused);
        assert_eq!(dbg.halt_reason, HaltReason::Breakpoint(0x10));

        dbg.toggle_breakpoint(0x10);
        assert!(!dbg.has_breakpoint(0x10));
    }

    #[test]
    fn test_watchpoint_trigger() {
        let mut dbg = DebuggerCore::new();
        dbg.add_watchpoint(0x200, 4, WatchpointKind::Write);

        assert!(!dbg.check_memory_access(0x200, false));
        assert!(dbg.check_memory_access(0x202, true));
        assert!(dbg.paused);
        assert_eq!(
            dbg.halt_reason,
            HaltReason::Watchpoint {
                address: 0x202,
                is_write: true
            }
        );
    }
}
