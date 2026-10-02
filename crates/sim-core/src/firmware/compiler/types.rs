/// Compilation error with line number and diagnostic message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileError {
    pub line: usize,
    pub message: String,
}

impl CompileError {
    /// Constructs a new compile error.
    pub fn new(line: usize, message: impl Into<String>) -> Self {
        Self {
            line,
            message: message.into(),
        }
    }
}

/// Tokenized intermediate instruction representation before bytecode emission.
#[derive(Debug, Clone)]
pub enum IntermediateOp {
    SetHigh(u8),
    SetLow(u8),
    Delay(u16),
    Jump(String),
    JumpIf(String),
    JumpIfNot(String),
    ReadPin(u8),
    UartString(String),
    LedcWrite(u8, u8),
    WifiConnect(String),
    LcdPrint(String),
    LcdClear,
    LcdSetCursor(u8, u8),
    OledPrint(String),
    OledClear,
    Halt,
    Label(String),
}
