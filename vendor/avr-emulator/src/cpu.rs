//! CPU state and execution.

use crate::bus::{DataBus, ExecutionHooks, IoBus, ProgramBus, SleepMode, SpmOperation};
use crate::decode::{decode, AddressMode, DecodeError, DecodedInstruction, OpcodeKind};
use crate::des::des_round;
use crate::profile::{CoreProfile, Profile};

pub const FLAG_C: u8 = 1 << 0;
pub const FLAG_Z: u8 = 1 << 1;
pub const FLAG_N: u8 = 1 << 2;
pub const FLAG_V: u8 = 1 << 3;
pub const FLAG_S: u8 = 1 << 4;
pub const FLAG_H: u8 = 1 << 5;
pub const FLAG_T: u8 = 1 << 6;
pub const FLAG_I: u8 = 1 << 7;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CycleCount {
    pub min: u8,
    pub max: u8,
}
impl CycleCount {
    pub const fn exact(cycles: u8) -> Self {
        Self {
            min: cycles,
            max: cycles,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StepEvent {
    None,
    Skipped { words: u8 },
    Sleep,
    Break,
    Watchdog,
    Spm,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StepResult {
    pub instruction: DecodedInstruction,
    pub cycles: CycleCount,
    pub pc_before: u32,
    pub pc_after: u32,
    pub event: StepEvent,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StepError {
    Decode(DecodeError),
    UndefinedOperand { word: u16, register: u8 },
}
impl From<DecodeError> for StepError {
    fn from(value: DecodeError) -> Self {
        Self::Decode(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InterruptResult {
    pub accepted: bool,
    pub cycles: u8,
    pub vector: u32,
}

/// CPU state. Register file, flags, pointers, stack pointer, and PC are all
/// stored inline; program/data/I/O storage stays with the host buses.
pub struct Cpu {
    profile: Profile,
    registers: [u8; 32],
    sreg: u8,
    sp: u32,
    pc: u32,
    ramp_x: u8,
    ramp_y: u8,
    ramp_z: u8,
    ramp_d: u8,
    eind: u8,
    sleeping: bool,
    break_requested: bool,
    last_was_des: bool,
    total_cycles: u64,
}

impl Cpu {
    pub const fn new(profile: Profile) -> Self {
        Self {
            profile,
            registers: [0; 32],
            sreg: 0,
            sp: 0,
            pc: 0,
            ramp_x: 0,
            ramp_y: 0,
            ramp_z: 0,
            ramp_d: 0,
            eind: 0,
            sleeping: false,
            break_requested: false,
            last_was_des: false,
            total_cycles: 0,
        }
    }

    pub const fn profile(&self) -> Profile {
        self.profile
    }
    pub const fn pc(&self) -> u32 {
        self.pc
    }
    pub const fn sp(&self) -> u32 {
        self.sp
    }
    pub const fn sreg(&self) -> u8 {
        self.sreg
    }
    pub const fn register(&self, index: u8) -> u8 {
        self.registers[index as usize & 31]
    }
    pub const fn sleeping(&self) -> bool {
        self.sleeping
    }
    pub const fn break_requested(&self) -> bool {
        self.break_requested
    }
    pub const fn total_cycles(&self) -> u64 {
        self.total_cycles
    }
    pub const fn ramp_x(&self) -> u8 {
        self.ramp_x
    }
    pub const fn ramp_y(&self) -> u8 {
        self.ramp_y
    }
    pub const fn ramp_z(&self) -> u8 {
        self.ramp_z
    }
    pub const fn ramp_d(&self) -> u8 {
        self.ramp_d
    }
    pub const fn eind(&self) -> u8 {
        self.eind
    }

    pub fn set_pc(&mut self, value: u32) {
        self.pc = value & self.profile.program_counter_mask();
    }
    pub fn set_sp(&mut self, value: u32) {
        self.sp = value & 0xffff;
    }
    pub fn set_sreg(&mut self, value: u8) {
        self.sreg = value;
    }
    pub fn set_register(&mut self, index: u8, value: u8) {
        self.registers[index as usize & 31] = value;
    }
    pub fn set_ramp_x(&mut self, value: u8) {
        self.ramp_x = value;
    }
    pub fn set_ramp_y(&mut self, value: u8) {
        self.ramp_y = value;
    }
    pub fn set_ramp_z(&mut self, value: u8) {
        self.ramp_z = value;
    }
    pub fn set_ramp_d(&mut self, value: u8) {
        self.ramp_d = value;
    }
    pub fn set_eind(&mut self, value: u8) {
        self.eind = value;
    }
    pub fn wake(&mut self) {
        self.sleeping = false;
    }
    pub fn clear_break(&mut self) {
        self.break_requested = false;
    }

    /// Execute one instruction. A reserved or profile-incompatible word is an
    /// error and has no architectural side effects.
    pub fn step<P, D, I, H>(
        &mut self,
        program: &mut P,
        data: &mut D,
        io: &mut I,
        hooks: &mut H,
    ) -> Result<StepResult, StepError>
    where
        P: ProgramBus,
        D: DataBus,
        I: IoBus,
        H: ExecutionHooks,
    {
        if self.sleeping {
            return Ok(StepResult {
                instruction: DecodedInstruction {
                    kind: OpcodeKind::Sleep,
                    word: 0x9588,
                    length: 1,
                    fields: Default::default(),
                },
                cycles: CycleCount::exact(0),
                pc_before: self.pc,
                pc_after: self.pc,
                event: StepEvent::Sleep,
            });
        }
        let pc_before = self.pc;
        let instruction = decode(program.read_word(self.pc), self.profile)?;
        let second = if instruction.is_two_word() {
            program.read_word(self.add_pc(1))
        } else {
            0
        };
        let mut next_pc = self.add_pc(instruction.length as u32);
        let mut event = StepEvent::None;
        let mut branch_taken = false;
        let mut skipped_words = 0;

        match instruction.kind {
            OpcodeKind::Nop => {}
            OpcodeKind::Add | OpcodeKind::Lsl => {
                let d = self.get(instruction.fields.rd);
                let r = self.get(instruction.fields.rr);
                let result = d.wrapping_add(r);
                self.set(instruction.fields.rd, result);
                self.add_flags(d, r, false, result, false);
            }
            OpcodeKind::Adc | OpcodeKind::Rol => {
                let d = self.get(instruction.fields.rd);
                let r = self.get(instruction.fields.rr);
                let result = d
                    .wrapping_add(r)
                    .wrapping_add((self.sreg & FLAG_C != 0) as u8);
                self.set(instruction.fields.rd, result);
                self.add_flags(d, r, self.sreg & FLAG_C != 0, result, true);
            }
            OpcodeKind::Sub => {
                let d = self.get(instruction.fields.rd);
                let r = self.get(instruction.fields.rr);
                let result = d.wrapping_sub(r);
                self.set(instruction.fields.rd, result);
                self.sub_flags(d, r, false, result, false);
            }
            OpcodeKind::Sbc => {
                let d = self.get(instruction.fields.rd);
                let r = self.get(instruction.fields.rr);
                let old_z = self.sreg & FLAG_Z != 0;
                let borrow = self.sreg & FLAG_C != 0;
                let result = d.wrapping_sub(r).wrapping_sub(borrow as u8);
                self.set(instruction.fields.rd, result);
                self.sub_flags_with_z(d, r, borrow, result, old_z);
            }
            OpcodeKind::Cp => {
                let d = self.get(instruction.fields.rd);
                let r = self.get(instruction.fields.rr);
                self.sub_flags(d, r, false, d.wrapping_sub(r), false);
            }
            OpcodeKind::Cpc => {
                let d = self.get(instruction.fields.rd);
                let r = self.get(instruction.fields.rr);
                let old_z = self.sreg & FLAG_Z != 0;
                let borrow = self.sreg & FLAG_C != 0;
                self.sub_flags_with_z(
                    d,
                    r,
                    borrow,
                    d.wrapping_sub(r).wrapping_sub(borrow as u8),
                    old_z,
                );
            }
            OpcodeKind::Cpse => {
                if self.get(instruction.fields.rd) == self.get(instruction.fields.rr) {
                    skipped_words = self.skip_words(program, next_pc)?;
                    next_pc = self.add_pc(1 + skipped_words as u32);
                    event = StepEvent::Skipped {
                        words: skipped_words,
                    };
                }
            }
            OpcodeKind::Subi => {
                self.sub_immediate(
                    instruction.fields.rd,
                    instruction.fields.immediate as u8,
                    false,
                    false,
                );
            }
            OpcodeKind::Sbci => {
                self.sub_immediate(
                    instruction.fields.rd,
                    instruction.fields.immediate as u8,
                    self.sreg & FLAG_C != 0,
                    true,
                );
            }
            OpcodeKind::Cpi => {
                let d = self.get(instruction.fields.rd);
                let r = instruction.fields.immediate as u8;
                self.sub_flags(d, r, false, d.wrapping_sub(r), false);
            }
            OpcodeKind::And => {
                let result = self.get(instruction.fields.rd) & self.get(instruction.fields.rr);
                self.set(instruction.fields.rd, result);
                self.logic_flags(result);
            }
            OpcodeKind::Andi => {
                let result = self.get(instruction.fields.rd) & instruction.fields.immediate as u8;
                self.set(instruction.fields.rd, result);
                self.logic_flags(result);
            }
            OpcodeKind::Or => {
                let result = self.get(instruction.fields.rd) | self.get(instruction.fields.rr);
                self.set(instruction.fields.rd, result);
                self.logic_flags(result);
            }
            OpcodeKind::Ori => {
                let result = self.get(instruction.fields.rd) | instruction.fields.immediate as u8;
                self.set(instruction.fields.rd, result);
                self.logic_flags(result);
            }
            OpcodeKind::Eor => {
                let result = self.get(instruction.fields.rd) ^ self.get(instruction.fields.rr);
                self.set(instruction.fields.rd, result);
                self.logic_flags(result);
            }
            OpcodeKind::Com => {
                let result = !self.get(instruction.fields.rd);
                self.set(instruction.fields.rd, result);
                self.logic_flags(result);
                // COM defines C but leaves H unchanged.
                self.sreg |= FLAG_C;
            }
            OpcodeKind::Neg => {
                let old = self.get(instruction.fields.rd);
                let result = 0u8.wrapping_sub(old);
                self.set(instruction.fields.rd, result);
                self.neg_flags(old, result);
            }
            OpcodeKind::Inc => {
                let result = self.get(instruction.fields.rd).wrapping_add(1);
                self.set(instruction.fields.rd, result);
                self.inc_dec_flags(result, true);
            }
            OpcodeKind::Dec => {
                let result = self.get(instruction.fields.rd).wrapping_sub(1);
                self.set(instruction.fields.rd, result);
                self.inc_dec_flags(result, false);
            }
            OpcodeKind::Asr => {
                let old = self.get(instruction.fields.rd);
                let result = (old >> 1) | (old & 0x80);
                self.set(instruction.fields.rd, result);
                self.shift_flags(old, result);
            }
            OpcodeKind::Lsr => {
                let old = self.get(instruction.fields.rd);
                let result = old >> 1;
                self.set(instruction.fields.rd, result);
                self.shift_flags(old, result);
            }
            OpcodeKind::Ror => {
                let old = self.get(instruction.fields.rd);
                let result = (old >> 1) | ((self.sreg & FLAG_C != 0) as u8) << 7;
                self.set(instruction.fields.rd, result);
                self.shift_flags(old, result);
            }
            OpcodeKind::Swap => {
                let old = self.get(instruction.fields.rd);
                self.set(instruction.fields.rd, old.rotate_left(4));
            }

            OpcodeKind::Mov => {
                self.set(instruction.fields.rd, self.get(instruction.fields.rr));
            }
            OpcodeKind::Movw => {
                let d = instruction.fields.pair;
                let r = instruction.fields.immediate as u8;
                let value = self.get(r) as u16 | (self.get(r + 1) as u16) << 8;
                self.set(d, value as u8);
                self.set(d + 1, (value >> 8) as u8);
            }
            OpcodeKind::Mul
            | OpcodeKind::Muls
            | OpcodeKind::Mulsu
            | OpcodeKind::Fmul
            | OpcodeKind::Fmuls
            | OpcodeKind::Fmulsu => {
                self.multiply(instruction);
            }
            OpcodeKind::Adiw => {
                self.adiw(instruction.fields.pair, instruction.fields.immediate, true);
            }
            OpcodeKind::Sbiw => {
                self.adiw(instruction.fields.pair, instruction.fields.immediate, false);
            }
            OpcodeKind::Ldi => {
                self.set(instruction.fields.rd, instruction.fields.immediate as u8);
            }
            OpcodeKind::In => {
                let value = self.io_read(io, instruction.fields.io);
                self.set(instruction.fields.rd, value);
            }
            OpcodeKind::Out => {
                let value = self.get(instruction.fields.rd);
                self.io_write(io, instruction.fields.io, value);
            }
            OpcodeKind::Sbi | OpcodeKind::Cbi | OpcodeKind::Sbic | OpcodeKind::Sbis => {
                self.io_bit_instruction::<I, P>(
                    instruction,
                    io,
                    &mut next_pc,
                    program,
                    &mut skipped_words,
                    &mut event,
                )?;
            }
            OpcodeKind::Bld => {
                let value = self.get(instruction.fields.rd);
                self.set(
                    instruction.fields.rd,
                    if self.sreg & FLAG_T != 0 {
                        value | 1 << instruction.fields.bit
                    } else {
                        value & !(1 << instruction.fields.bit)
                    },
                );
            }
            OpcodeKind::Bst => {
                if self.get(instruction.fields.rd) & (1 << instruction.fields.bit) != 0 {
                    self.sreg |= FLAG_T;
                } else {
                    self.sreg &= !FLAG_T;
                }
            }
            OpcodeKind::Bset => {
                self.sreg |= 1 << instruction.fields.bit;
            }
            OpcodeKind::Bclr => {
                self.sreg &= !(1 << instruction.fields.bit);
            }
            OpcodeKind::Brbs | OpcodeKind::Brbc => {
                let set = self.sreg & (1 << instruction.fields.bit) != 0;
                branch_taken = if instruction.kind == OpcodeKind::Brbs {
                    set
                } else {
                    !set
                };
                if branch_taken {
                    next_pc = self.add_signed(next_pc, instruction.fields.relative as i32);
                }
            }
            OpcodeKind::Rjmp => {
                next_pc = self.add_signed(next_pc, instruction.fields.relative as i32);
            }
            OpcodeKind::Rcall => {
                self.push_return(data, io, next_pc);
                next_pc = self.add_signed(next_pc, instruction.fields.relative as i32);
            }
            OpcodeKind::Jmp => {
                next_pc = self.long_target(instruction, second);
            }
            OpcodeKind::Call => {
                self.push_return(data, io, next_pc);
                next_pc = self.long_target(instruction, second);
            }
            OpcodeKind::Ijmp => {
                next_pc = self.pair(30) & self.profile.program_counter_mask();
            }
            OpcodeKind::Eijmp => {
                next_pc = ((self.eind as u32) << 16 | self.pair(30))
                    & self.profile.program_counter_mask();
            }
            OpcodeKind::Icall => {
                self.push_return(data, io, next_pc);
                next_pc = self.pair(30) & self.profile.program_counter_mask();
            }
            OpcodeKind::Eicall => {
                self.push_return(data, io, next_pc);
                next_pc = ((self.eind as u32) << 16 | self.pair(30))
                    & self.profile.program_counter_mask();
            }
            OpcodeKind::Ret | OpcodeKind::Reti => {
                next_pc = self.pop_return(data, io);
                if instruction.kind == OpcodeKind::Reti {
                    self.sreg |= FLAG_I;
                }
            }
            OpcodeKind::Ld => {
                self.load_store(instruction, data, io, false)?;
            }
            OpcodeKind::St => {
                self.load_store(instruction, data, io, true)?;
            }
            OpcodeKind::Lds => {
                let address = self.direct_address(instruction, second);
                let value = self.data_read(data, io, address);
                self.set(instruction.fields.rd, value);
            }
            OpcodeKind::Sts => {
                let address = self.direct_address(instruction, second);
                let value = self.get(instruction.fields.rd);
                self.data_write(data, io, address, value);
            }
            OpcodeKind::Lpm | OpcodeKind::Elpm => {
                self.program_load(instruction, program)?;
            }
            OpcodeKind::Spm => {
                let address = self.program_address(true);
                let value = self.get(0) as u16 | (self.get(1) as u16) << 8;
                let post_increment = instruction.word == 0x95f8;
                hooks.spm(SpmOperation {
                    address,
                    word: value,
                    post_increment,
                });
                if post_increment {
                    self.set_program_address(address.wrapping_add(2), true);
                }
                event = StepEvent::Spm;
            }
            OpcodeKind::Xch | OpcodeKind::Las | OpcodeKind::Lac | OpcodeKind::Lat => {
                self.read_modify_write(instruction, data, io)?;
            }
            OpcodeKind::Push => {
                let value = self.get(instruction.fields.rd);
                self.push_byte(data, io, value);
            }
            OpcodeKind::Pop => {
                let value = self.pop_byte(data, io);
                self.set(instruction.fields.rd, value);
            }
            OpcodeKind::Sleep => {
                self.sleeping = true;
                hooks.sleep(SleepMode::Instruction);
                event = StepEvent::Sleep;
            }
            OpcodeKind::Break => {
                self.break_requested = true;
                hooks.break_instruction();
                event = StepEvent::Break;
            }
            OpcodeKind::Wdr => {
                hooks.watchdog_reset();
                event = StepEvent::Watchdog;
            }
            OpcodeKind::Des => {
                let mut data_block = [0; 8];
                let mut key = [0; 8];
                data_block.copy_from_slice(&self.registers[0..8]);
                key.copy_from_slice(&self.registers[8..16]);
                let result = des_round(
                    data_block,
                    key,
                    instruction.fields.immediate as u8,
                    self.sreg & FLAG_H != 0,
                );
                self.registers[0..8].copy_from_slice(&result);
            }
            OpcodeKind::Sbrc | OpcodeKind::Sbrs => {
                let set = self.get(instruction.fields.rd) & (1 << instruction.fields.bit) != 0;
                let skip = if instruction.kind == OpcodeKind::Sbrs {
                    set
                } else {
                    !set
                };
                if skip {
                    skipped_words = self.skip_words(program, next_pc)?;
                    next_pc = self.add_pc(1 + skipped_words as u32);
                    event = StepEvent::Skipped {
                        words: skipped_words,
                    };
                }
            }
        }

        self.pc = next_pc & self.profile.program_counter_mask();
        let cycles = self.cycles(instruction, branch_taken, skipped_words);
        self.total_cycles = self.total_cycles.wrapping_add(cycles.max as u64);
        self.last_was_des = instruction.kind == OpcodeKind::Des;
        Ok(StepResult {
            instruction,
            cycles,
            pc_before,
            pc_after: self.pc,
            event,
        })
    }

    /// Enter an interrupt if I is set. The vector is a word address.
    pub fn interrupt_entry<D, I>(
        &mut self,
        data: &mut D,
        io: &mut I,
        vector: u32,
    ) -> InterruptResult
    where
        D: DataBus,
        I: IoBus,
    {
        if self.sreg & FLAG_I == 0 {
            return InterruptResult {
                accepted: false,
                cycles: 0,
                vector,
            };
        }
        self.push_return(data, io, self.pc);
        self.sreg &= !FLAG_I;
        self.pc = vector & self.profile.program_counter_mask();
        self.sleeping = false;
        InterruptResult {
            accepted: true,
            cycles: if self.profile.modern_timing() { 3 } else { 4 },
            vector: self.pc,
        }
    }
    pub fn service_interrupt<D: DataBus, I: IoBus>(
        &mut self,
        data: &mut D,
        io: &mut I,
        vector: u32,
    ) -> InterruptResult {
        self.interrupt_entry(data, io, vector)
    }

    fn get(&self, index: u8) -> u8 {
        self.registers[index as usize & 31]
    }
    fn set(&mut self, index: u8, value: u8) {
        self.registers[index as usize & 31] = value;
    }
    fn pair(&self, low: u8) -> u32 {
        self.get(low) as u32 | (self.get(low + 1) as u32) << 8
    }
    fn set_pair(&mut self, low: u8, value: u32) {
        self.set(low, value as u8);
        self.set(low + 1, (value >> 8) as u8);
    }
    fn add_pc(&self, value: u32) -> u32 {
        self.pc.wrapping_add(value) & self.profile.program_counter_mask()
    }
    fn add_signed(&self, value: u32, offset: i32) -> u32 {
        value.wrapping_add(offset as u32) & self.profile.program_counter_mask()
    }

    fn logic_flags(&mut self, result: u8) {
        self.sreg &= !(FLAG_V | FLAG_N | FLAG_S | FLAG_Z);
        if result == 0 {
            self.sreg |= FLAG_Z;
        }
        if result & 0x80 != 0 {
            self.sreg |= FLAG_N | FLAG_S;
        }
    }
    fn add_flags(&mut self, d: u8, r: u8, carry: bool, result: u8, preserve_z: bool) {
        let sum = d as u16 + r as u16 + carry as u16;
        let h = (d & 0x0f) as u16 + (r & 0x0f) as u16 + carry as u16 > 0x0f;
        let v = (!(d ^ r) & (d ^ result) & 0x80) != 0;
        let old_z = self.sreg & FLAG_Z != 0;
        self.sreg = set_flag(self.sreg, FLAG_H, h);
        self.sreg = set_flag(self.sreg, FLAG_C, sum > 0xff);
        self.sreg = set_flag(self.sreg, FLAG_N, result & 0x80 != 0);
        self.sreg = set_flag(self.sreg, FLAG_V, v);
        self.sreg = set_flag(self.sreg, FLAG_S, (result & 0x80 != 0) ^ v);
        self.sreg = set_flag(self.sreg, FLAG_Z, result == 0 && (!preserve_z || old_z));
    }
    fn sub_flags(&mut self, d: u8, r: u8, borrow: bool, result: u8, preserve_z: bool) {
        let old_z = if preserve_z {
            self.sreg & FLAG_Z != 0
        } else {
            true
        };
        self.sub_flags_with_z(d, r, borrow, result, old_z);
    }
    fn sub_flags_with_z(&mut self, d: u8, r: u8, borrow: bool, result: u8, old_z: bool) {
        let full_r = r as u16 + borrow as u16;
        let h = (d & 0x0f) < ((r & 0x0f) + borrow as u8);
        let c = (d as u16) < full_r;
        let v = ((d ^ r) & (d ^ result) & 0x80) != 0;
        self.sreg = set_flag(self.sreg, FLAG_H, h);
        self.sreg = set_flag(self.sreg, FLAG_C, c);
        self.sreg = set_flag(self.sreg, FLAG_N, result & 0x80 != 0);
        self.sreg = set_flag(self.sreg, FLAG_V, v);
        self.sreg = set_flag(self.sreg, FLAG_S, (result & 0x80 != 0) ^ v);
        self.sreg = set_flag(self.sreg, FLAG_Z, result == 0 && old_z);
    }
    fn sub_immediate(&mut self, rd: u8, immediate: u8, carry_in: bool, preserve_z: bool) {
        let d = self.get(rd);
        let result = d.wrapping_sub(immediate).wrapping_sub(carry_in as u8);
        self.set(rd, result);
        let old_z = if preserve_z {
            self.sreg & FLAG_Z != 0
        } else {
            true
        };
        self.sub_flags_with_z(d, immediate, carry_in, result, old_z);
    }
    fn neg_flags(&mut self, old: u8, result: u8) {
        self.sreg = set_flag(self.sreg, FLAG_H, result & 0x08 != 0 || old & 0x08 != 0);
        self.sreg = set_flag(self.sreg, FLAG_C, result != 0);
        self.sreg = set_flag(self.sreg, FLAG_Z, result == 0);
        self.sreg = set_flag(self.sreg, FLAG_N, result & 0x80 != 0);
        self.sreg = set_flag(self.sreg, FLAG_V, result == 0x80);
        self.sreg = set_flag(self.sreg, FLAG_S, (result & 0x80 != 0) ^ (result == 0x80));
    }
    fn inc_dec_flags(&mut self, result: u8, increment: bool) {
        let v = if increment {
            result == 0x80
        } else {
            result == 0x7f
        };
        self.sreg = set_flag(self.sreg, FLAG_V, v);
        self.sreg = set_flag(self.sreg, FLAG_N, result & 0x80 != 0);
        self.sreg = set_flag(self.sreg, FLAG_S, (result & 0x80 != 0) ^ v);
        self.sreg = set_flag(self.sreg, FLAG_Z, result == 0);
    }
    fn shift_flags(&mut self, old: u8, result: u8) {
        let c = old & 1 != 0;
        let n = result & 0x80 != 0;
        let v = n ^ c;
        self.sreg = set_flag(self.sreg, FLAG_C, c);
        self.sreg = set_flag(self.sreg, FLAG_N, n);
        self.sreg = set_flag(self.sreg, FLAG_V, v);
        self.sreg = set_flag(self.sreg, FLAG_S, n ^ v);
        self.sreg = set_flag(self.sreg, FLAG_Z, result == 0);
    }

    fn multiply(&mut self, instruction: DecodedInstruction) {
        let d = self.get(instruction.fields.rd);
        let r = self.get(instruction.fields.rr);
        let value = match instruction.kind {
            OpcodeKind::Mul | OpcodeKind::Fmul => d as i16 * r as i16,
            OpcodeKind::Muls | OpcodeKind::Fmuls => (d as i8 as i16) * (r as i8 as i16),
            OpcodeKind::Mulsu | OpcodeKind::Fmulsu => (d as i8 as i16) * r as i16,
            _ => 0,
        };
        let mut result = value as u16;
        if matches!(
            instruction.kind,
            OpcodeKind::Fmul | OpcodeKind::Fmuls | OpcodeKind::Fmulsu
        ) {
            result <<= 1;
        }
        self.set(0, result as u8);
        self.set(1, (result >> 8) as u8);
        self.sreg = set_flag(self.sreg, FLAG_Z, result == 0);
        self.sreg = set_flag(self.sreg, FLAG_C, result & 0x8000 != 0);
    }
    fn adiw(&mut self, pair: u8, immediate: u16, add: bool) {
        let old = self.pair(pair) as u16;
        let result = if add {
            old.wrapping_add(immediate)
        } else {
            old.wrapping_sub(immediate)
        };
        self.set_pair(pair, result as u32);
        let n = result & 0x8000 != 0;
        let old_n = old & 0x8000 != 0;
        let v = if add { !old_n && n } else { old_n && !n };
        let c = if add { !n && old_n } else { n && !old_n };
        self.sreg = set_flag(self.sreg, FLAG_Z, result == 0);
        self.sreg = set_flag(self.sreg, FLAG_N, n);
        self.sreg = set_flag(self.sreg, FLAG_V, v);
        self.sreg = set_flag(self.sreg, FLAG_S, n ^ v);
        self.sreg = set_flag(self.sreg, FLAG_C, c);
    }

    fn io_read<I: IoBus>(&mut self, io: &mut I, address: u8) -> u8 {
        match address {
            0x3f => self.sreg,
            0x3e => (self.sp >> 8) as u8,
            0x3d => self.sp as u8,
            0x3c if self.profile.has_eind() => self.eind,
            0x3b if self.profile.has_ramp_z() => self.ramp_z,
            0x3a if self.profile.has_ramp_y() => self.ramp_y,
            0x39 if self.profile.has_ramp_x() => self.ramp_x,
            0x38 if self.profile.has_ramp_d() => self.ramp_d,
            _ => io.read_io(address),
        }
    }
    fn io_write<I: IoBus>(&mut self, io: &mut I, address: u8, value: u8) {
        match address {
            0x3f => self.sreg = value,
            0x3e => self.sp = (self.sp & 0xff) | (value as u32) << 8,
            0x3d => self.sp = (self.sp & 0xff00) | value as u32,
            0x3c if self.profile.has_eind() => self.eind = value,
            0x3b if self.profile.has_ramp_z() => self.ramp_z = value,
            0x3a if self.profile.has_ramp_y() => self.ramp_y = value,
            0x39 if self.profile.has_ramp_x() => self.ramp_x = value,
            0x38 if self.profile.has_ramp_d() => self.ramp_d = value,
            _ => io.write_io(address, value),
        }
    }
    fn data_read<D: DataBus, I: IoBus>(&mut self, data: &mut D, io: &mut I, address: u32) -> u8 {
        let address = address & self.profile.data_address_mask();
        if address < 32 {
            self.get(address as u8)
        } else if address < 0x60 {
            self.io_read(io, (address - 0x20) as u8)
        } else {
            data.read(address)
        }
    }
    fn data_write<D: DataBus, I: IoBus>(
        &mut self,
        data: &mut D,
        io: &mut I,
        address: u32,
        value: u8,
    ) {
        let address = address & self.profile.data_address_mask();
        if address < 32 {
            self.set(address as u8, value);
        } else if address < 0x60 {
            self.io_write(io, (address - 0x20) as u8, value);
        } else {
            data.write(address, value);
        }
    }
    fn pointer_address(&self, mode: AddressMode) -> (u32, u8, bool) {
        let (pair, ramp, use_ramp, pre, post) = match mode {
            AddressMode::X => (26, self.ramp_x, self.profile.has_ramp_x(), false, false),
            AddressMode::XPostIncrement => {
                (26, self.ramp_x, self.profile.has_ramp_x(), false, true)
            }
            AddressMode::XPreDecrement => (26, self.ramp_x, self.profile.has_ramp_x(), true, false),
            AddressMode::Y => (28, self.ramp_y, self.profile.has_ramp_y(), false, false),
            AddressMode::YPostIncrement => {
                (28, self.ramp_y, self.profile.has_ramp_y(), false, true)
            }
            AddressMode::YPreDecrement => (28, self.ramp_y, self.profile.has_ramp_y(), true, false),
            AddressMode::Z => (30, self.ramp_z, self.profile.has_ramp_z(), false, false),
            AddressMode::ZPostIncrement => {
                (30, self.ramp_z, self.profile.has_ramp_z(), false, true)
            }
            AddressMode::ZPreDecrement => (30, self.ramp_z, self.profile.has_ramp_z(), true, false),
            AddressMode::YDisplacement(_) => {
                (28, self.ramp_y, self.profile.has_ramp_y(), false, false)
            }
            AddressMode::ZDisplacement(_) => {
                (30, self.ramp_z, self.profile.has_ramp_z(), false, false)
            }
        };
        let base = self.pair(pair)
            | if self.profile.data_address_bits() > 16 && use_ramp {
                (ramp as u32) << 16
            } else {
                0
            };
        (base, pair, pre || post)
    }
    fn load_store<D: DataBus, I: IoBus>(
        &mut self,
        instruction: DecodedInstruction,
        data: &mut D,
        io: &mut I,
        store: bool,
    ) -> Result<(), StepError> {
        let mode = instruction.fields.address_mode.unwrap_or(AddressMode::Z);
        let (base, pair, changes_pointer) = self.pointer_address(mode);
        if changes_pointer && (instruction.fields.rd == pair || instruction.fields.rd == pair + 1) {
            return Err(StepError::UndefinedOperand {
                word: instruction.word,
                register: instruction.fields.rd,
            });
        }
        let (address, updates_pointer) = match mode {
            AddressMode::YDisplacement(q) | AddressMode::ZDisplacement(q) => {
                (base.wrapping_add(q as u32), false)
            }
            AddressMode::XPreDecrement
            | AddressMode::YPreDecrement
            | AddressMode::ZPreDecrement => (base.wrapping_sub(1), true),
            AddressMode::XPostIncrement
            | AddressMode::YPostIncrement
            | AddressMode::ZPostIncrement => (base, true),
            _ => (base, false),
        };
        if store {
            let value = self.get(instruction.fields.rd);
            self.data_write(data, io, address, value);
        } else {
            let value = self.data_read(data, io, address);
            self.set(instruction.fields.rd, value);
        }
        if updates_pointer {
            let updated = match mode {
                AddressMode::XPostIncrement
                | AddressMode::YPostIncrement
                | AddressMode::ZPostIncrement => address.wrapping_add(1),
                _ => address,
            };
            self.update_pointer(mode, updated);
        }
        Ok(())
    }
    fn update_pointer(&mut self, mode: AddressMode, address: u32) {
        let (pair, ramp) = match mode {
            AddressMode::X | AddressMode::XPostIncrement | AddressMode::XPreDecrement => (26, 0),
            AddressMode::Y
            | AddressMode::YPostIncrement
            | AddressMode::YPreDecrement
            | AddressMode::YDisplacement(_) => (28, 1),
            AddressMode::Z
            | AddressMode::ZPostIncrement
            | AddressMode::ZPreDecrement
            | AddressMode::ZDisplacement(_) => (30, 2),
        };
        self.set_pair(pair, address);
        if self.profile.data_address_bits() > 16 {
            match ramp {
                0 if self.profile.has_ramp_x() => self.ramp_x = (address >> 16) as u8,
                1 if self.profile.has_ramp_y() => self.ramp_y = (address >> 16) as u8,
                2 if self.profile.has_ramp_z() => self.ramp_z = (address >> 16) as u8,
                _ => {}
            }
        }
    }
    fn io_bit_instruction<I: IoBus, P: ProgramBus>(
        &mut self,
        instruction: DecodedInstruction,
        io: &mut I,
        next_pc: &mut u32,
        program: &mut P,
        skipped_words: &mut u8,
        event: &mut StepEvent,
    ) -> Result<(), StepError> {
        let address = instruction.fields.io;
        let bit = 1 << instruction.fields.bit;
        match instruction.kind {
            OpcodeKind::Sbi => {
                let value = self.io_read(io, address);
                self.io_write(io, address, value | bit);
            }
            OpcodeKind::Cbi => {
                let value = self.io_read(io, address);
                self.io_write(io, address, value & !bit);
            }
            OpcodeKind::Sbic | OpcodeKind::Sbis => {
                let set = self.io_read(io, address) & bit != 0;
                let skip = if instruction.kind == OpcodeKind::Sbis {
                    set
                } else {
                    !set
                };
                if skip {
                    *skipped_words = self.skip_words(program, *next_pc)?;
                    *next_pc = self.add_pc(1 + *skipped_words as u32);
                    *event = StepEvent::Skipped {
                        words: *skipped_words,
                    };
                }
            }
            _ => {}
        }
        Ok(())
    }
    fn skip_words<P: ProgramBus>(&self, program: &mut P, address: u32) -> Result<u8, StepError> {
        Ok(decode(program.read_word(address), self.profile)?.length)
    }
    fn direct_address(&self, _instruction: DecodedInstruction, second: u16) -> u32 {
        (second as u32)
            | if self.profile.has_ramp_d() {
                (self.ramp_d as u32) << 16
            } else {
                0
            }
    }
    fn program_address(&self, extended: bool) -> u32 {
        self.pair(30)
            | if extended && self.profile.has_ramp_z() {
                (self.ramp_z as u32) << 16
            } else {
                0
            }
    }
    fn set_program_address(&mut self, address: u32, extended: bool) {
        self.set_pair(30, address);
        if extended && self.profile.has_ramp_z() {
            self.ramp_z = (address >> 16) as u8;
        }
    }
    fn program_load<P: ProgramBus>(
        &mut self,
        instruction: DecodedInstruction,
        program: &mut P,
    ) -> Result<(), StepError> {
        let post = instruction.fields.immediate & 1 != 0;
        if instruction.kind == OpcodeKind::Elpm && post && instruction.fields.rd >= 30 {
            return Err(StepError::UndefinedOperand {
                word: instruction.word,
                register: instruction.fields.rd,
            });
        }
        let extended =
            instruction.kind == OpcodeKind::Elpm || instruction.fields.immediate & 2 != 0;
        let address = self.program_address(extended);
        let word = program.read_word(address >> 1);
        let value = if address & 1 == 0 {
            word as u8
        } else {
            (word >> 8) as u8
        };
        let rd = if instruction.word == 0x95c8 || instruction.word == 0x95d8 {
            0
        } else {
            instruction.fields.rd
        };
        self.set(rd, value);
        if post {
            self.set_program_address(address.wrapping_add(1), extended);
        }
        Ok(())
    }
    fn long_target(&self, instruction: DecodedInstruction, second: u16) -> u32 {
        (((instruction.fields.high_address as u32) << 16) | second as u32)
            & self.profile.program_counter_mask()
    }
    fn push_byte<D: DataBus, I: IoBus>(&mut self, data: &mut D, io: &mut I, value: u8) {
        self.data_write(data, io, self.sp, value);
        self.sp = self.sp.wrapping_sub(1) & 0xffff;
    }
    fn pop_byte<D: DataBus, I: IoBus>(&mut self, data: &mut D, io: &mut I) -> u8 {
        self.sp = self.sp.wrapping_add(1) & 0xffff;
        self.data_read(data, io, self.sp)
    }
    fn push_return<D: DataBus, I: IoBus>(&mut self, data: &mut D, io: &mut I, value: u32) {
        let bytes = self.profile.return_address_bytes();
        let mut i = 0;
        while i < bytes {
            self.push_byte(data, io, (value >> (i * 8)) as u8);
            i += 1;
        }
    }
    fn pop_return<D: DataBus, I: IoBus>(&mut self, data: &mut D, io: &mut I) -> u32 {
        let bytes = self.profile.return_address_bytes();
        let mut result = 0;
        let mut i = bytes;
        while i != 0 {
            i -= 1;
            result = (result << 8) | self.pop_byte(data, io) as u32;
        }
        result & self.profile.program_counter_mask()
    }
    fn read_modify_write<D: DataBus, I: IoBus>(
        &mut self,
        instruction: DecodedInstruction,
        data: &mut D,
        io: &mut I,
    ) -> Result<(), StepError> {
        let z = self.pointer_address(AddressMode::Z).0;
        let old = self.data_read(data, io, z);
        let rd = self.get(instruction.fields.rd);
        let new = match instruction.kind {
            OpcodeKind::Xch => rd,
            OpcodeKind::Las => old | rd,
            OpcodeKind::Lac => old & !rd,
            OpcodeKind::Lat => old ^ rd,
            _ => old,
        };
        self.set(instruction.fields.rd, old);
        self.data_write(data, io, z, new);
        Ok(())
    }
    fn cycles(
        &self,
        instruction: DecodedInstruction,
        branch_taken: bool,
        skipped_words: u8,
    ) -> CycleCount {
        let kind = instruction.kind;
        let core = self.profile.core();
        let exact = CycleCount::exact;
        let with_return_byte =
            |base: u8| exact(base + u8::from(self.profile.return_address_bytes() == 3));
        match kind {
            OpcodeKind::Call => with_return_byte(
                if core == CoreProfile::Avrxm || core == CoreProfile::Avrxt {
                    3
                } else {
                    4
                },
            ),
            OpcodeKind::Rcall | OpcodeKind::Icall => with_return_byte(
                if core == CoreProfile::Avrxm || core == CoreProfile::Avrxt {
                    2
                } else {
                    3
                },
            ),
            OpcodeKind::Eicall => with_return_byte(
                if core == CoreProfile::Avrxm || core == CoreProfile::Avrxt {
                    3
                } else {
                    4
                },
            ),
            OpcodeKind::Ret | OpcodeKind::Reti => with_return_byte(4),
            OpcodeKind::Jmp => exact(3),
            OpcodeKind::Rjmp | OpcodeKind::Ijmp | OpcodeKind::Eijmp => exact(2),
            OpcodeKind::Lds => exact(if matches!(core, CoreProfile::Avrxm | CoreProfile::Avrxt) {
                3
            } else {
                2
            }),
            OpcodeKind::Sts => exact(2),
            OpcodeKind::Ld => {
                if core == CoreProfile::Avrxm
                    && matches!(
                        instruction.fields.address_mode,
                        Some(
                            AddressMode::XPreDecrement
                                | AddressMode::YPreDecrement
                                | AddressMode::ZPreDecrement
                                | AddressMode::YDisplacement(_)
                                | AddressMode::ZDisplacement(_)
                        )
                    )
                {
                    exact(3)
                } else {
                    exact(2)
                }
            }
            OpcodeKind::St => match core {
                CoreProfile::Avrxm
                    if matches!(
                        instruction.fields.address_mode,
                        Some(
                            AddressMode::XPreDecrement
                                | AddressMode::YPreDecrement
                                | AddressMode::ZPreDecrement
                                | AddressMode::YDisplacement(_)
                                | AddressMode::ZDisplacement(_)
                        )
                    ) =>
                {
                    exact(2)
                }
                CoreProfile::Avrxm | CoreProfile::Avrxt => exact(1),
                _ => exact(2),
            },
            OpcodeKind::Lpm | OpcodeKind::Elpm => exact(3),
            OpcodeKind::Push => exact(if matches!(core, CoreProfile::Avrxm | CoreProfile::Avrxt) {
                1
            } else {
                2
            }),
            OpcodeKind::Pop => exact(2),
            OpcodeKind::Sbi | OpcodeKind::Cbi => {
                exact(if matches!(core, CoreProfile::Avrxm | CoreProfile::Avrxt) {
                    1
                } else {
                    2
                })
            }
            OpcodeKind::Cpse | OpcodeKind::Sbrc | OpcodeKind::Sbrs => exact(1 + skipped_words),
            OpcodeKind::Sbic | OpcodeKind::Sbis => {
                exact(u8::from(core == CoreProfile::Avrxm) + 1 + skipped_words)
            }
            OpcodeKind::Brbs | OpcodeKind::Brbc => exact(if branch_taken { 2 } else { 1 }),
            OpcodeKind::Des if core == CoreProfile::Avrxm => {
                exact(if self.last_was_des { 1 } else { 2 })
            }
            OpcodeKind::Spm => exact(4),
            _ => exact(1),
        }
    }
}

fn set_flag(value: u8, flag: u8, set: bool) -> u8 {
    if set {
        value | flag
    } else {
        value & !flag
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::{NoHooks, SliceData, SliceIo, SliceProgram};

    fn execute(cpu: &mut Cpu, words: &[u16]) {
        let mut program = SliceProgram::new(words);
        let mut bytes = [0u8; 16];
        let mut data = SliceData::new(&mut bytes);
        let mut ports = [0u8; 64];
        let mut io = SliceIo::new(&mut ports);
        let mut hooks = NoHooks;
        cpu.step(&mut program, &mut data, &mut io, &mut hooks)
            .expect("instruksi harus dapat didekode");
    }

    #[test]
    fn subi_ignores_carry_in() {
        let mut cpu = Cpu::new(Profile::avre_plus());
        cpu.set_register(16, 0x00);
        cpu.set_sreg(FLAG_C);
        execute(&mut cpu, &[0x5001]);
        assert_eq!(cpu.register(16), 0xFF);
        assert_ne!(cpu.sreg() & FLAG_C, 0);
    }

    #[test]
    fn subi_builds_ascii_digit() {
        let mut cpu = Cpu::new(Profile::avre_plus());
        cpu.set_register(22, 0x06);
        cpu.set_sreg(FLAG_C);
        execute(&mut cpu, &[0x5D60]);
        assert_eq!(cpu.register(22), b'6');
    }

    #[test]
    fn sbci_uses_carry_in() {
        let mut cpu = Cpu::new(Profile::avre_plus());
        cpu.set_register(16, 0x05);
        cpu.set_sreg(FLAG_C);
        execute(&mut cpu, &[0x4001]);
        assert_eq!(cpu.register(16), 0x03);
    }
}
