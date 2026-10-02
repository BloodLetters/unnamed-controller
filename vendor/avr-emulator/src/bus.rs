//! Host interfaces. The CPU never owns a heap-backed memory image.

/// Word-addressed Harvard program-memory access.
pub trait ProgramBus {
    fn read_word(&mut self, word_address: u32) -> u16;
}

/// Byte-addressed data-memory access outside the CPU register file and I/O area.
pub trait DataBus {
    fn read(&mut self, address: u32) -> u8;
    fn write(&mut self, address: u32, value: u8);
}

/// Six-bit direct I/O access for IN, OUT, SBI, CBI, SBIC, and SBIS.
pub trait IoBus {
    fn read_io(&mut self, address: u8) -> u8;
    fn write_io(&mut self, address: u8, value: u8);
}

/// SPM/NVM operation handed to the host. `address` is a byte address in the
/// program address space; `word` is the R1:R0 value supplied by the CPU.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpmOperation {
    pub address: u32,
    pub word: u16,
    pub post_increment: bool,
}

/// Sleep modes are intentionally abstract: a device decides which clock and
/// peripheral domains stop.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SleepMode {
    Instruction,
}

/// Non-CPU events that need device integration.
pub trait ExecutionHooks {
    fn spm(&mut self, _operation: SpmOperation) {}
    fn sleep(&mut self, _mode: SleepMode) {}
    fn break_instruction(&mut self) {}
    fn watchdog_reset(&mut self) {}
}

#[derive(Default)]
pub struct NoHooks;
impl ExecutionHooks for NoHooks {}

/// A no-heap program-memory adapter for tests and small hosts.
pub struct SliceProgram<'a> {
    pub words: &'a [u16],
}
impl<'a> SliceProgram<'a> {
    pub const fn new(words: &'a [u16]) -> Self {
        Self { words }
    }
}
impl ProgramBus for SliceProgram<'_> {
    fn read_word(&mut self, address: u32) -> u16 {
        self.words.get(address as usize).copied().unwrap_or(0)
    }
}

/// A no-heap data-memory adapter.
pub struct SliceData<'a> {
    pub bytes: &'a mut [u8],
}
impl<'a> SliceData<'a> {
    pub const fn new(bytes: &'a mut [u8]) -> Self {
        Self { bytes }
    }
}
impl DataBus for SliceData<'_> {
    fn read(&mut self, address: u32) -> u8 {
        self.bytes.get(address as usize).copied().unwrap_or(0)
    }
    fn write(&mut self, address: u32, value: u8) {
        if let Some(byte) = self.bytes.get_mut(address as usize) {
            *byte = value;
        }
    }
}

/// A no-heap I/O adapter.
pub struct SliceIo<'a> {
    pub ports: &'a mut [u8],
}
impl<'a> SliceIo<'a> {
    pub const fn new(ports: &'a mut [u8]) -> Self {
        Self { ports }
    }
}
impl IoBus for SliceIo<'_> {
    fn read_io(&mut self, address: u8) -> u8 {
        self.ports.get(address as usize).copied().unwrap_or(0)
    }
    fn write_io(&mut self, address: u8, value: u8) {
        if let Some(port) = self.ports.get_mut(address as usize) {
            *port = value;
        }
    }
}
