use super::types::{CompileError, IntermediateOp};
use crate::firmware::opcodes::*;
use std::collections::HashMap;

/// Default I2C address of the HD44780 LCD backpack used by `lcd.*` statements.
const LCD_I2C_ADDRESS: u8 = 0x27;

/// Default I2C address of the SSD1306 OLED display.
const SSD1306_I2C_ADDRESS: u8 = 0x3C;

/// Resolves labels and emits final binary bytecode vector.
pub fn emit_bytecode(ops: Vec<IntermediateOp>) -> Result<Vec<u8>, CompileError> {
    let mut label_positions: HashMap<String, usize> = HashMap::new();
    let mut current_offset: usize = 0;

    for op in &ops {
        match op {
            IntermediateOp::Label(name) => {
                label_positions.insert(name.clone(), current_offset);
            }
            IntermediateOp::SetHigh(_) | IntermediateOp::SetLow(_) => {
                current_offset += 2;
            }
            IntermediateOp::Delay(_) | IntermediateOp::Jump(_) => {
                current_offset += 3;
            }
            IntermediateOp::JumpIf(_) | IntermediateOp::JumpIfNot(_) => {
                current_offset += 3;
            }
            IntermediateOp::ReadPin(_) => {
                current_offset += 2;
            }
            IntermediateOp::UartString(s) => {
                current_offset += s.len() * 2;
            }
            IntermediateOp::LedcWrite(_, _) => {
                current_offset += 3;
            }
            IntermediateOp::WifiConnect(s) => {
                current_offset += 2 + s.len();
            }
            IntermediateOp::LcdPrint(s) => {
                current_offset += 4 + s.len();
            }
            IntermediateOp::LcdClear | IntermediateOp::LcdSetCursor(_, _) => {
                current_offset += 5;
            }
            IntermediateOp::OledPrint(s) => {
                current_offset += 4 + s.len();
            }
            IntermediateOp::OledClear => {
                current_offset += 5;
            }
            IntermediateOp::Halt => {
                current_offset += 1;
            }
        }
    }

    let mut bytecode = Vec::with_capacity(current_offset);

    for op in ops {
        match op {
            IntermediateOp::Label(_) => {}
            IntermediateOp::SetHigh(pin) => {
                bytecode.push(OP_SET_HIGH);
                bytecode.push(pin);
            }
            IntermediateOp::SetLow(pin) => {
                bytecode.push(OP_SET_LOW);
                bytecode.push(pin);
            }
            IntermediateOp::Delay(d) => {
                bytecode.push(OP_DELAY);
                bytecode.extend_from_slice(&d.to_le_bytes());
            }
            IntermediateOp::Jump(label) => {
                emit_jump(&mut bytecode, OP_JUMP, &label_positions, &label)?;
            }
            IntermediateOp::JumpIf(label) => {
                emit_jump(&mut bytecode, OP_JUMP_IF, &label_positions, &label)?;
            }
            IntermediateOp::JumpIfNot(label) => {
                emit_jump(&mut bytecode, OP_JUMP_IF_NOT, &label_positions, &label)?;
            }
            IntermediateOp::ReadPin(pin) => {
                bytecode.push(OP_READ_PIN);
                bytecode.push(pin);
            }
            IntermediateOp::UartString(s) => {
                for b in s.bytes() {
                    bytecode.push(OP_UART_SEND);
                    bytecode.push(b);
                }
            }
            IntermediateOp::LedcWrite(pin, duty) => {
                bytecode.push(OP_LEDC_WRITE);
                bytecode.push(pin);
                bytecode.push(duty);
            }
            IntermediateOp::WifiConnect(ssid) => {
                bytecode.push(OP_WIFI_CONNECT);
                bytecode.push(ssid.len() as u8);
                bytecode.extend_from_slice(ssid.as_bytes());
            }
            IntermediateOp::LcdPrint(s) => {
                let mut data = Vec::with_capacity(1 + s.len());
                data.push(0x40);
                data.extend_from_slice(s.as_bytes());
                if data.len() > u8::MAX as usize {
                    return Err(CompileError::new(
                        0,
                        "LCD string exceeds maximum frame length",
                    ));
                }
                bytecode.push(OP_I2C_WRITE);
                bytecode.push(LCD_I2C_ADDRESS);
                bytecode.push(data.len() as u8);
                bytecode.extend_from_slice(&data);
            }
            IntermediateOp::LcdClear => {
                bytecode.push(OP_I2C_WRITE);
                bytecode.push(LCD_I2C_ADDRESS);
                bytecode.push(2);
                bytecode.push(0x00);
                bytecode.push(0x01);
            }
            IntermediateOp::LcdSetCursor(col, row) => {
                let addr = 0x80 | (if row > 0 { 0x40 } else { 0 }) | (col & 0x0F);
                bytecode.push(OP_I2C_WRITE);
                bytecode.push(LCD_I2C_ADDRESS);
                bytecode.push(2);
                bytecode.push(0x00);
                bytecode.push(addr);
            }
            IntermediateOp::OledPrint(s) => {
                let mut data = Vec::with_capacity(1 + s.len());
                data.push(0x40);
                data.extend_from_slice(s.as_bytes());
                if data.len() > u8::MAX as usize {
                    return Err(CompileError::new(
                        0,
                        "OLED string exceeds maximum frame length",
                    ));
                }
                bytecode.push(OP_I2C_WRITE);
                bytecode.push(SSD1306_I2C_ADDRESS);
                bytecode.push(data.len() as u8);
                bytecode.extend_from_slice(&data);
            }
            IntermediateOp::OledClear => {
                bytecode.push(OP_I2C_WRITE);
                bytecode.push(SSD1306_I2C_ADDRESS);
                bytecode.push(2);
                bytecode.push(0x00);
                bytecode.push(0x01);
            }
            IntermediateOp::Halt => {
                bytecode.push(OP_HALT);
            }
        }
    }

    Ok(bytecode)
}

/// Resolves a jump label to its byte offset, accepting a numeric literal as a fallback.
fn resolve_target(
    label_positions: &HashMap<String, usize>,
    label: &str,
) -> Result<usize, CompileError> {
    match label_positions.get(label) {
        Some(&position) => Ok(position),
        None => label
            .parse::<usize>()
            .map_err(|_| CompileError::new(0, format!("Undefined label: {}", label))),
    }
}

/// Emits a jump opcode followed by a resolved 16-bit little-endian target address.
fn emit_jump(
    bytecode: &mut Vec<u8>,
    opcode: u8,
    label_positions: &HashMap<String, usize>,
    label: &str,
) -> Result<(), CompileError> {
    let target = resolve_target(label_positions, label)?;
    if target > u16::MAX as usize {
        return Err(CompileError::new(
            0,
            format!("Jump target {} exceeds addressable range", target),
        ));
    }
    bytecode.push(opcode);
    bytecode.extend_from_slice(&(target as u16).to_le_bytes());
    Ok(())
}
