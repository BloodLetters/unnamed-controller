/// Watchpoint access type for memory data debugging.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum WatchpointKind {
    Read,
    Write,
    Access,
}

/// Execution halt reason for MCU debugger state.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HaltReason {
    #[default]
    None,
    Step,
    Breakpoint(u32),
    Watchpoint {
        address: u32,
        is_write: bool,
    },
    HaltInstruction,
}

/// Hardware or software execution breakpoint.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Breakpoint {
    pub address: u32,
    pub enabled: bool,
    pub hit_count: u64,
}

impl Breakpoint {
    /// Constructs a new enabled breakpoint at the given address.
    pub fn new(address: u32) -> Self {
        Self {
            address,
            enabled: true,
            hit_count: 0,
        }
    }
}

/// Memory data watchpoint monitoring memory accesses.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Watchpoint {
    pub address: u32,
    pub size: usize,
    pub kind: WatchpointKind,
    pub enabled: bool,
    pub hit_count: u64,
}

impl Watchpoint {
    /// Constructs a new memory watchpoint.
    pub fn new(address: u32, size: usize, kind: WatchpointKind) -> Self {
        Self {
            address,
            size,
            kind,
            enabled: true,
            hit_count: 0,
        }
    }

    /// Returns true if a memory access overlaps with this watchpoint range.
    pub fn matches(&self, access_addr: u32, is_write: bool) -> bool {
        if !self.enabled {
            return false;
        }
        let type_match = match self.kind {
            WatchpointKind::Read => !is_write,
            WatchpointKind::Write => is_write,
            WatchpointKind::Access => true,
        };
        type_match
            && access_addr >= self.address
            && access_addr < self.address.saturating_add(self.size as u32)
    }
}

/// CPU architecture registers snapshot for inspection and GDB remote debugging.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CpuRegisters {
    pub pc: u32,
    pub sp: u32,
    pub flags: u8,
    pub r: [u32; 16],
    pub cycles: u64,
}

impl Default for CpuRegisters {
    fn default() -> Self {
        Self {
            pc: 0,
            sp: 0x1000,
            flags: 0,
            r: [0; 16],
            cycles: 0,
        }
    }
}
