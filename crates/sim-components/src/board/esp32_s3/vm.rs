//! Virtual bytecode execution engine for high-level firmware sketches.

use sim_core::firmware::opcodes::*;

use super::gpio::{Esp32S3Gpio, GpioMode};

/// State of the virtual bytecode execution engine for user sketches.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BytecodeVm {
    pub pc: usize,
    pub delay_remaining_ms: f32,
    pub last_read_pin_high: bool,
    pub halted: bool,
}

impl BytecodeVm {
    /// Constructs a new virtual machine in its initial state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Resets the instruction pointer and timing counters.
    pub fn reset(&mut self) {
        self.pc = 0;
        self.delay_remaining_ms = 0.0;
        self.last_read_pin_high = false;
        self.halted = false;
    }

    /// Advances the bytecode execution by the elapsed time delta.
    pub fn step(
        &mut self,
        code: &[u8],
        dt_seconds: f32,
        gpio: &mut Esp32S3Gpio,
        serial_tx: &mut String,
        i2c_out: &mut Vec<(u8, Vec<u8>)>,
    ) {
        if self.halted || code.is_empty() {
            return;
        }

        let dt_ms = dt_seconds * 1000.0;
        if self.delay_remaining_ms > 0.0 {
            self.delay_remaining_ms -= dt_ms;
            if self.delay_remaining_ms > 0.0 {
                return;
            }
        }

        let mut instructions_executed = 0;
        const MAX_OPS_PER_STEP: usize = 100;

        while self.pc < code.len() && instructions_executed < MAX_OPS_PER_STEP && !self.halted {
            instructions_executed += 1;
            let op = code[self.pc];
            self.pc += 1;

            match op {
                OP_NOP => {}
                OP_SET_HIGH => {
                    if self.pc < code.len() {
                        let pin = code[self.pc];
                        self.pc += 1;
                        gpio.set_mode(pin, GpioMode::Output);
                        gpio.set_output(pin, true);
                    }
                }
                OP_SET_LOW => {
                    if self.pc < code.len() {
                        let pin = code[self.pc];
                        self.pc += 1;
                        gpio.set_mode(pin, GpioMode::Output);
                        gpio.set_output(pin, false);
                    }
                }
                OP_DELAY => {
                    if self.pc + 2 <= code.len() {
                        let delay_ms =
                            u16::from_le_bytes([code[self.pc], code[self.pc + 1]]) as f32;
                        self.pc += 2;
                        self.delay_remaining_ms = delay_ms;
                        break;
                    }
                }
                OP_JUMP => {
                    if self.pc + 2 <= code.len() {
                        let target =
                            u16::from_le_bytes([code[self.pc], code[self.pc + 1]]) as usize;
                        self.pc = target;
                    }
                }
                OP_READ_PIN => {
                    if self.pc < code.len() {
                        let pin = code[self.pc];
                        self.pc += 1;
                        self.last_read_pin_high = gpio.effective_level(pin);
                    }
                }
                OP_JUMP_IF => {
                    if self.pc + 2 <= code.len() {
                        let target =
                            u16::from_le_bytes([code[self.pc], code[self.pc + 1]]) as usize;
                        self.pc = if self.last_read_pin_high {
                            target
                        } else {
                            self.pc + 2
                        };
                    }
                }
                OP_JUMP_IF_NOT => {
                    if self.pc + 2 <= code.len() {
                        let target =
                            u16::from_le_bytes([code[self.pc], code[self.pc + 1]]) as usize;
                        self.pc = if !self.last_read_pin_high {
                            target
                        } else {
                            self.pc + 2
                        };
                    }
                }
                OP_UART_SEND => {
                    if self.pc < code.len() {
                        let b = code[self.pc];
                        self.pc += 1;
                        serial_tx.push(b as char);
                    }
                }
                OP_LEDC_WRITE => {
                    if self.pc + 2 <= code.len() {
                        let pin = code[self.pc];
                        let duty = code[self.pc + 1];
                        self.pc += 2;
                        gpio.set_mode(pin, GpioMode::Output);
                        gpio.set_output(pin, duty > 127);
                    }
                }
                OP_WIFI_CONNECT => {
                    if self.pc < code.len() {
                        let len = code[self.pc] as usize;
                        self.pc = (self.pc + 1 + len).min(code.len());
                    }
                }
                OP_I2C_WRITE => {
                    if self.pc + 2 <= code.len() {
                        let addr = code[self.pc];
                        let len = code[self.pc + 1] as usize;
                        let end = (self.pc + 2 + len).min(code.len());
                        let data = code[self.pc + 2..end].to_vec();
                        self.pc = end;
                        i2c_out.push((addr, data));
                    }
                }
                OP_HALT => {
                    self.halted = true;
                    break;
                }
                _ => {
                    break;
                }
            }
        }
    }
}
