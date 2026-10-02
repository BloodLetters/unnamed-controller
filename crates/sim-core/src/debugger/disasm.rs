use crate::firmware::opcodes::*;

/// A disassembled bytecode instruction with address and mnemonic representation.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DisassembledInstruction {
    pub address: usize,
    pub raw_bytes: Vec<u8>,
    pub mnemonic: String,
    pub operands: String,
}

/// Disassembles a slice of firmware bytecode into human-readable instructions.
pub fn disassemble(bytecode: &[u8]) -> Vec<DisassembledInstruction> {
    let mut instructions = Vec::new();
    let mut pc = 0;

    while pc < bytecode.len() {
        let addr = pc;
        let op = bytecode[pc];
        pc += 1;

        match op {
            OP_NOP => {
                instructions.push(DisassembledInstruction {
                    address: addr,
                    raw_bytes: vec![op],
                    mnemonic: "NOP".into(),
                    operands: String::new(),
                });
            }
            OP_SET_HIGH => {
                let gpio = bytecode.get(pc).copied().unwrap_or(0);
                pc = (pc + 1).min(bytecode.len());
                instructions.push(DisassembledInstruction {
                    address: addr,
                    raw_bytes: vec![op, gpio],
                    mnemonic: "SET_HIGH".into(),
                    operands: format!("GPIO{}", gpio),
                });
            }
            OP_SET_LOW => {
                let gpio = bytecode.get(pc).copied().unwrap_or(0);
                pc = (pc + 1).min(bytecode.len());
                instructions.push(DisassembledInstruction {
                    address: addr,
                    raw_bytes: vec![op, gpio],
                    mnemonic: "SET_LOW".into(),
                    operands: format!("GPIO{}", gpio),
                });
            }
            OP_DELAY => {
                let low = bytecode.get(pc).copied().unwrap_or(0);
                let high = bytecode.get(pc + 1).copied().unwrap_or(0);
                pc = (pc + 2).min(bytecode.len());
                let delay = u16::from_le_bytes([low, high]);
                instructions.push(DisassembledInstruction {
                    address: addr,
                    raw_bytes: vec![op, low, high],
                    mnemonic: "DELAY".into(),
                    operands: format!("{}ms", delay),
                });
            }
            OP_JUMP => {
                let low = bytecode.get(pc).copied().unwrap_or(0);
                let high = bytecode.get(pc + 1).copied().unwrap_or(0);
                pc = (pc + 2).min(bytecode.len());
                let target = u16::from_le_bytes([low, high]);
                instructions.push(DisassembledInstruction {
                    address: addr,
                    raw_bytes: vec![op, low, high],
                    mnemonic: "JUMP".into(),
                    operands: format!("0x{:04X}", target),
                });
            }
            OP_JUMP_IF | OP_JUMP_IF_NOT => {
                let low = bytecode.get(pc).copied().unwrap_or(0);
                let high = bytecode.get(pc + 1).copied().unwrap_or(0);
                pc = (pc + 2).min(bytecode.len());
                let target = u16::from_le_bytes([low, high]);
                let mnemonic = if op == OP_JUMP_IF {
                    "JUMP_IF"
                } else {
                    "JUMP_IF_NOT"
                };
                instructions.push(DisassembledInstruction {
                    address: addr,
                    raw_bytes: vec![op, low, high],
                    mnemonic: mnemonic.into(),
                    operands: format!("0x{:04X}", target),
                });
            }
            OP_READ_PIN => {
                let pin = bytecode.get(pc).copied().unwrap_or(0);
                pc = (pc + 1).min(bytecode.len());
                instructions.push(DisassembledInstruction {
                    address: addr,
                    raw_bytes: vec![op, pin],
                    mnemonic: "READ_PIN".into(),
                    operands: format!("GPIO{}", pin),
                });
            }
            OP_UART_SEND => {
                let ch = bytecode.get(pc).copied().unwrap_or(0);
                pc = (pc + 1).min(bytecode.len());
                instructions.push(DisassembledInstruction {
                    address: addr,
                    raw_bytes: vec![op, ch],
                    mnemonic: "UART_SEND".into(),
                    operands: format!("'{}' (0x{:02X})", ch as char, ch),
                });
            }
            OP_LEDC_WRITE => {
                let gpio = bytecode.get(pc).copied().unwrap_or(0);
                let duty = bytecode.get(pc + 1).copied().unwrap_or(0);
                pc = (pc + 2).min(bytecode.len());
                instructions.push(DisassembledInstruction {
                    address: addr,
                    raw_bytes: vec![op, gpio, duty],
                    mnemonic: "LEDC_WRITE".into(),
                    operands: format!("GPIO{}, duty={}", gpio, duty),
                });
            }
            OP_WIFI_CONNECT => {
                let len = bytecode.get(pc).copied().unwrap_or(0) as usize;
                pc = (pc + 1).min(bytecode.len());
                let end = (pc + len).min(bytecode.len());
                let ssid_bytes = &bytecode[pc..end];
                let ssid = String::from_utf8_lossy(ssid_bytes).to_string();
                pc = end;
                let mut bytes = vec![op, len as u8];
                bytes.extend_from_slice(ssid_bytes);
                instructions.push(DisassembledInstruction {
                    address: addr,
                    raw_bytes: bytes,
                    mnemonic: "WIFI_CONNECT".into(),
                    operands: format!("\"{}\"", ssid),
                });
            }
            OP_ENABLE_INTERRUPTS => {
                instructions.push(DisassembledInstruction {
                    address: addr,
                    raw_bytes: vec![op],
                    mnemonic: "SEI".into(),
                    operands: String::new(),
                });
            }
            OP_DISABLE_INTERRUPTS => {
                instructions.push(DisassembledInstruction {
                    address: addr,
                    raw_bytes: vec![op],
                    mnemonic: "CLI".into(),
                    operands: String::new(),
                });
            }
            OP_RETI => {
                instructions.push(DisassembledInstruction {
                    address: addr,
                    raw_bytes: vec![op],
                    mnemonic: "RETI".into(),
                    operands: String::new(),
                });
            }
            OP_DAC_WRITE => {
                let ch = bytecode.get(pc).copied().unwrap_or(1);
                let val = bytecode.get(pc + 1).copied().unwrap_or(0);
                pc = (pc + 2).min(bytecode.len());
                instructions.push(DisassembledInstruction {
                    address: addr,
                    raw_bytes: vec![op, ch, val],
                    mnemonic: "DAC_WRITE".into(),
                    operands: format!("CH{}, {}", ch, val),
                });
            }
            OP_TONE => {
                let pin = bytecode.get(pc).copied().unwrap_or(0);
                let freq_h = bytecode.get(pc + 1).copied().unwrap_or(0) as u16;
                let freq_l = bytecode.get(pc + 2).copied().unwrap_or(0) as u16;
                let freq = (freq_h << 8) | freq_l;
                pc = (pc + 3).min(bytecode.len());
                instructions.push(DisassembledInstruction {
                    address: addr,
                    raw_bytes: vec![op, pin, freq_h as u8, freq_l as u8],
                    mnemonic: "TONE".into(),
                    operands: format!("GPIO{}, {}Hz", pin, freq),
                });
            }
            OP_ATTACH_INTERRUPT => {
                let pin = bytecode.get(pc).copied().unwrap_or(0);
                let mode = bytecode.get(pc + 1).copied().unwrap_or(0);
                let isr = bytecode.get(pc + 2).copied().unwrap_or(0);
                pc = (pc + 3).min(bytecode.len());
                instructions.push(DisassembledInstruction {
                    address: addr,
                    raw_bytes: vec![op, pin, mode, isr],
                    mnemonic: "ATTACH_IRQ".into(),
                    operands: format!("pin={}, mode={}, isr={}", pin, mode, isr),
                });
            }
            OP_CONFIG_TIMER => {
                let low = bytecode.get(pc).copied().unwrap_or(0);
                let high = bytecode.get(pc + 1).copied().unwrap_or(0);
                let compare = bytecode.get(pc + 2).copied().unwrap_or(0);
                let isr_low = bytecode.get(pc + 3).copied().unwrap_or(0);
                let isr_high = bytecode.get(pc + 4).copied().unwrap_or(0);
                pc = (pc + 5).min(bytecode.len());
                let prescaler = u16::from_le_bytes([low, high]);
                let isr = u16::from_le_bytes([isr_low, isr_high]);
                instructions.push(DisassembledInstruction {
                    address: addr,
                    raw_bytes: vec![op, low, high, compare, isr_low, isr_high],
                    mnemonic: "CONFIG_TIMER".into(),
                    operands: format!("prescaler={}, compare={}, isr={}", prescaler, compare, isr),
                });
            }
            OP_RESET_WATCHDOG => {
                instructions.push(DisassembledInstruction {
                    address: addr,
                    raw_bytes: vec![op],
                    mnemonic: "WDR".into(),
                    operands: String::new(),
                });
            }
            OP_I2C_WRITE => {
                let address = bytecode.get(pc).copied().unwrap_or(0);
                let length = bytecode.get(pc + 1).copied().unwrap_or(0) as usize;
                let start = pc + 2;
                let end = (start + length).min(bytecode.len());
                let payload = bytecode[start..end].to_vec();
                pc = end;
                let mut raw = vec![op, address, length as u8];
                raw.extend_from_slice(&payload);
                instructions.push(DisassembledInstruction {
                    address: addr,
                    raw_bytes: raw,
                    mnemonic: "I2C_WRITE".into(),
                    operands: format!("addr=0x{:02X}, len={}", address, payload.len()),
                });
            }
            OP_HALT => {
                instructions.push(DisassembledInstruction {
                    address: addr,
                    raw_bytes: vec![op],
                    mnemonic: "HALT".into(),
                    operands: String::new(),
                });
            }
            unknown => {
                instructions.push(DisassembledInstruction {
                    address: addr,
                    raw_bytes: vec![unknown],
                    mnemonic: "DB".into(),
                    operands: format!("0x{:02X}", unknown),
                });
            }
        }
    }

    instructions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disassembly_basic_sequence() {
        let code = vec![OP_SET_HIGH, 2, OP_DELAY, 100, 0, OP_HALT];
        let dis = disassemble(&code);
        assert_eq!(dis.len(), 3);
        assert_eq!(dis[0].mnemonic, "SET_HIGH");
        assert_eq!(dis[0].operands, "GPIO2");
        assert_eq!(dis[1].mnemonic, "DELAY");
        assert_eq!(dis[1].operands, "100ms");
        assert_eq!(dis[2].mnemonic, "HALT");
    }
}
