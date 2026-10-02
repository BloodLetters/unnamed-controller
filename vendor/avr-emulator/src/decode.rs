//! Decoder and instruction metadata.

use crate::profile::{CoreProfile, Profile};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum OpcodeKind {
    Nop,
    Add,
    Adc,
    Adiw,
    Sub,
    Subi,
    Sbc,
    Sbci,
    Sbiw,
    And,
    Andi,
    Or,
    Ori,
    Eor,
    Com,
    Neg,
    Inc,
    Dec,
    Asr,
    Lsr,
    Ror,
    Swap,
    Lsl,
    Rol,
    Mov,
    Movw,
    Mul,
    Muls,
    Mulsu,
    Fmul,
    Fmuls,
    Fmulsu,
    Cp,
    Cpc,
    Cpi,
    Cpse,
    Ldi,
    In,
    Out,
    Sbi,
    Cbi,
    Sbic,
    Sbis,
    Bld,
    Bst,
    Bclr,
    Bset,
    Brbs,
    Brbc,
    Rjmp,
    Rcall,
    Jmp,
    Call,
    Ijmp,
    Icall,
    Eijmp,
    Eicall,
    Ret,
    Reti,
    Ld,
    St,
    Lds,
    Sts,
    Lpm,
    Elpm,
    Spm,
    Xch,
    Las,
    Lac,
    Lat,
    Push,
    Pop,
    Sleep,
    Break,
    Wdr,
    Des,
    Sbrc,
    Sbrs,
}

/// Canonical forms covered by the decoder. Branch aliases such as `BREQ` and
/// `BRNE` are encoded by `BRBS`/`BRBC` with the corresponding bit field.
pub const OPCODE_INVENTORY: &[OpcodeKind] = &[
    OpcodeKind::Nop,
    OpcodeKind::Add,
    OpcodeKind::Adc,
    OpcodeKind::Adiw,
    OpcodeKind::Sub,
    OpcodeKind::Subi,
    OpcodeKind::Sbc,
    OpcodeKind::Sbci,
    OpcodeKind::Sbiw,
    OpcodeKind::And,
    OpcodeKind::Andi,
    OpcodeKind::Or,
    OpcodeKind::Ori,
    OpcodeKind::Eor,
    OpcodeKind::Com,
    OpcodeKind::Neg,
    OpcodeKind::Inc,
    OpcodeKind::Dec,
    OpcodeKind::Asr,
    OpcodeKind::Lsr,
    OpcodeKind::Ror,
    OpcodeKind::Swap,
    OpcodeKind::Lsl,
    OpcodeKind::Rol,
    OpcodeKind::Mov,
    OpcodeKind::Movw,
    OpcodeKind::Mul,
    OpcodeKind::Muls,
    OpcodeKind::Mulsu,
    OpcodeKind::Fmul,
    OpcodeKind::Fmuls,
    OpcodeKind::Fmulsu,
    OpcodeKind::Cp,
    OpcodeKind::Cpc,
    OpcodeKind::Cpi,
    OpcodeKind::Cpse,
    OpcodeKind::Ldi,
    OpcodeKind::In,
    OpcodeKind::Out,
    OpcodeKind::Sbi,
    OpcodeKind::Cbi,
    OpcodeKind::Sbic,
    OpcodeKind::Sbis,
    OpcodeKind::Bld,
    OpcodeKind::Bst,
    OpcodeKind::Bclr,
    OpcodeKind::Bset,
    OpcodeKind::Brbs,
    OpcodeKind::Brbc,
    OpcodeKind::Rjmp,
    OpcodeKind::Rcall,
    OpcodeKind::Jmp,
    OpcodeKind::Call,
    OpcodeKind::Ijmp,
    OpcodeKind::Icall,
    OpcodeKind::Eijmp,
    OpcodeKind::Eicall,
    OpcodeKind::Ret,
    OpcodeKind::Reti,
    OpcodeKind::Ld,
    OpcodeKind::St,
    OpcodeKind::Lds,
    OpcodeKind::Sts,
    OpcodeKind::Lpm,
    OpcodeKind::Elpm,
    OpcodeKind::Spm,
    OpcodeKind::Xch,
    OpcodeKind::Las,
    OpcodeKind::Lac,
    OpcodeKind::Lat,
    OpcodeKind::Push,
    OpcodeKind::Pop,
    OpcodeKind::Sleep,
    OpcodeKind::Break,
    OpcodeKind::Wdr,
    OpcodeKind::Des,
    OpcodeKind::Sbrc,
    OpcodeKind::Sbrs,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum AddressMode {
    X,
    XPostIncrement,
    XPreDecrement,
    Y,
    YPostIncrement,
    YPreDecrement,
    Z,
    ZPostIncrement,
    ZPreDecrement,
    YDisplacement(u8),
    ZDisplacement(u8),
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct InstructionFields {
    pub rd: u8,
    pub rr: u8,
    pub immediate: u16,
    pub bit: u8,
    pub io: u8,
    pub q: u8,
    pub address: u16,
    pub high_address: u8,
    pub relative: i16,
    pub pair: u8,
    pub address_mode: Option<AddressMode>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DecodedInstruction {
    pub kind: OpcodeKind,
    pub word: u16,
    pub length: u8,
    pub fields: InstructionFields,
}

impl DecodedInstruction {
    pub const fn new(kind: OpcodeKind, word: u16, length: u8, fields: InstructionFields) -> Self {
        Self {
            kind,
            word,
            length,
            fields,
        }
    }

    pub const fn is_two_word(self) -> bool {
        self.length == 2
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecodeError {
    Reserved {
        word: u16,
    },
    Unsupported {
        word: u16,
        kind: OpcodeKind,
        profile: CoreProfile,
    },
    UnsupportedProfile {
        profile: CoreProfile,
    },
}

impl DecodeError {
    pub const fn word(self) -> u16 {
        match self {
            Self::Reserved { word } | Self::Unsupported { word, .. } => word,
            Self::UnsupportedProfile { .. } => 0,
        }
    }
}

pub fn decode(word: u16, profile: Profile) -> Result<DecodedInstruction, DecodeError> {
    if profile.core() == CoreProfile::Avrrc {
        return Err(DecodeError::UnsupportedProfile {
            profile: CoreProfile::Avrrc,
        });
    }
    let f = InstructionFields::default();
    let decoded = if word == 0x0000 {
        d(OpcodeKind::Nop, word, 1, f)
    } else if word == 0x9508 {
        d(OpcodeKind::Ret, word, 1, f)
    } else if word == 0x9518 {
        d(OpcodeKind::Reti, word, 1, f)
    } else if word == 0x9409 {
        d(OpcodeKind::Ijmp, word, 1, f)
    } else if word == 0x9509 {
        d(OpcodeKind::Icall, word, 1, f)
    } else if word == 0x9419 {
        d(OpcodeKind::Eijmp, word, 1, f)
    } else if word == 0x9519 {
        d(OpcodeKind::Eicall, word, 1, f)
    } else if word == 0x9588 {
        d(OpcodeKind::Sleep, word, 1, f)
    } else if word == 0x9598 {
        d(OpcodeKind::Break, word, 1, f)
    } else if word == 0x95a8 {
        d(OpcodeKind::Wdr, word, 1, f)
    } else if word == 0x95e8 || word == 0x95f8 {
        d(OpcodeKind::Spm, word, 1, f)
    } else if word == 0x95c8 {
        d(OpcodeKind::Lpm, word, 1, f)
    } else if word == 0x95d8 {
        d(OpcodeKind::Elpm, word, 1, f)
    } else if word & 0xff8f == 0x9408 {
        d(OpcodeKind::Bset, word, 1, bit_fields(word))
    } else if word & 0xff8f == 0x9488 {
        d(OpcodeKind::Bclr, word, 1, bit_fields(word))
    } else if word & 0xfe0e == 0x940c {
        d(OpcodeKind::Jmp, word, 2, long_fields(word))
    } else if word & 0xfe0e == 0x940e {
        d(OpcodeKind::Call, word, 2, long_fields(word))
    } else if word & 0xff0f == 0x940b {
        d(
            OpcodeKind::Des,
            word,
            1,
            immediate_fields(word, (word >> 4) & 0x0f),
        )
    } else if word & 0xfe0f == 0x9400 {
        d(OpcodeKind::Com, word, 1, register_fields(word))
    } else if word & 0xfe0f == 0x9401 {
        d(OpcodeKind::Neg, word, 1, register_fields(word))
    } else if word & 0xfe0f == 0x9402 {
        d(OpcodeKind::Swap, word, 1, register_fields(word))
    } else if word & 0xfe0f == 0x9403 {
        d(OpcodeKind::Inc, word, 1, register_fields(word))
    } else if word & 0xfe0f == 0x9405 {
        d(OpcodeKind::Asr, word, 1, register_fields(word))
    } else if word & 0xfe0f == 0x9406 {
        d(OpcodeKind::Lsr, word, 1, register_fields(word))
    } else if word & 0xfe0f == 0x9407 {
        d(OpcodeKind::Ror, word, 1, register_fields(word))
    } else if word & 0xfe0f == 0x940a {
        d(OpcodeKind::Dec, word, 1, register_fields(word))
    } else if word & 0xff00 == 0x9600 {
        d(OpcodeKind::Adiw, word, 1, word_pair_immediate_fields(word))
    } else if word & 0xff00 == 0x9700 {
        d(OpcodeKind::Sbiw, word, 1, word_pair_immediate_fields(word))
    } else if word & 0xfe0f == 0x9000 {
        d(OpcodeKind::Lds, word, 2, register_fields(word))
    } else if word & 0xfe0f == 0x9200 {
        d(OpcodeKind::Sts, word, 2, register_fields(word))
    } else if word & 0xfe0c == 0x9004 {
        let kind = if word & 2 != 0 {
            OpcodeKind::Elpm
        } else {
            OpcodeKind::Lpm
        };
        d(kind, word, 1, program_load_fields(word))
    } else if word & 0xfe0e == 0x900c
        || word & 0xfe0f == 0x900e
        || word & 0xfe07 == 0x9001
        || word & 0xfe07 == 0x9002
    {
        d(OpcodeKind::Ld, word, 1, pointer_fields(word))
    } else if word & 0xd200 == 0x8000 {
        d(OpcodeKind::Ld, word, 1, displacement_fields(word))
    } else if word & 0xfe0c == 0x9204 {
        let mut pf = register_fields(word);
        pf.address_mode = Some(AddressMode::Z);
        let kind = match word & 3 {
            0 => OpcodeKind::Xch,
            1 => OpcodeKind::Las,
            2 => OpcodeKind::Lac,
            _ => OpcodeKind::Lat,
        };
        d(kind, word, 1, pf)
    } else if word & 0xfe0f == 0x900f {
        d(OpcodeKind::Pop, word, 1, register_fields(word))
    } else if word & 0xfe0f == 0x920f {
        d(OpcodeKind::Push, word, 1, register_fields(word))
    } else if word & 0xfe0e == 0x920c
        || word & 0xfe0f == 0x920e
        || word & 0xfe07 == 0x9201
        || word & 0xfe07 == 0x9202
    {
        d(OpcodeKind::St, word, 1, pointer_fields(word))
    } else if word & 0xd200 == 0x8200 {
        d(OpcodeKind::St, word, 1, displacement_fields(word))
    } else if word & 0xff00 == 0x0100 {
        d(OpcodeKind::Movw, word, 1, movw_fields(word))
    } else if word & 0xff00 == 0x0200 {
        d(
            OpcodeKind::Muls,
            word,
            1,
            restricted_mul_fields(word, 16, 4),
        )
    } else if word & 0xff88 == 0x0300 {
        d(
            OpcodeKind::Mulsu,
            word,
            1,
            restricted_mul_fields(word, 16, 3),
        )
    } else if word & 0xff88 == 0x0308 {
        d(
            OpcodeKind::Fmul,
            word,
            1,
            restricted_mul_fields(word, 16, 3),
        )
    } else if word & 0xff80 == 0x0380 {
        d(
            if word & 0x0008 == 0 {
                OpcodeKind::Fmuls
            } else {
                OpcodeKind::Fmulsu
            },
            word,
            1,
            restricted_mul_fields(word, 16, 3),
        )
    } else if word & 0xfc00 == 0x0400 {
        d(OpcodeKind::Cpc, word, 1, rr_fields(word))
    } else if word & 0xfc00 == 0x0800 {
        d(OpcodeKind::Sbc, word, 1, rr_fields(word))
    } else if word & 0xfc00 == 0x0c00 {
        let fields = rr_fields(word);
        d(
            if fields.rd == fields.rr {
                OpcodeKind::Lsl
            } else {
                OpcodeKind::Add
            },
            word,
            1,
            fields,
        )
    } else if word & 0xfc00 == 0x1000 {
        d(OpcodeKind::Cpse, word, 1, rr_fields(word))
    } else if word & 0xfc00 == 0x1400 {
        d(OpcodeKind::Cp, word, 1, rr_fields(word))
    } else if word & 0xfc00 == 0x1800 {
        d(OpcodeKind::Sub, word, 1, rr_fields(word))
    } else if word & 0xfc00 == 0x1c00 {
        let fields = rr_fields(word);
        d(
            if fields.rd == fields.rr {
                OpcodeKind::Rol
            } else {
                OpcodeKind::Adc
            },
            word,
            1,
            fields,
        )
    } else if word & 0xfc00 == 0x2000 {
        d(OpcodeKind::And, word, 1, rr_fields(word))
    } else if word & 0xfc00 == 0x2400 {
        d(OpcodeKind::Eor, word, 1, rr_fields(word))
    } else if word & 0xfc00 == 0x2800 {
        d(OpcodeKind::Or, word, 1, rr_fields(word))
    } else if word & 0xfc00 == 0x2c00 {
        d(OpcodeKind::Mov, word, 1, rr_fields(word))
    } else if word & 0xf000 == 0x3000 {
        d(OpcodeKind::Cpi, word, 1, high_immediate_fields(word))
    } else if word & 0xf000 == 0x4000 {
        d(OpcodeKind::Sbci, word, 1, high_immediate_fields(word))
    } else if word & 0xf000 == 0x5000 {
        d(OpcodeKind::Subi, word, 1, high_immediate_fields(word))
    } else if word & 0xf000 == 0x6000 {
        d(OpcodeKind::Ori, word, 1, high_immediate_fields(word))
    } else if word & 0xf000 == 0x7000 {
        d(OpcodeKind::Andi, word, 1, high_immediate_fields(word))
    } else if word & 0xff00 == 0x9800 {
        d(OpcodeKind::Cbi, word, 1, io_bit_fields(word))
    } else if word & 0xff00 == 0x9900 {
        d(OpcodeKind::Sbic, word, 1, io_bit_fields(word))
    } else if word & 0xff00 == 0x9a00 {
        d(OpcodeKind::Sbi, word, 1, io_bit_fields(word))
    } else if word & 0xff00 == 0x9b00 {
        d(OpcodeKind::Sbis, word, 1, io_bit_fields(word))
    } else if word & 0xfc00 == 0x9c00 {
        d(OpcodeKind::Mul, word, 1, rr_fields(word))
    } else if word & 0xf800 == 0xb000 {
        d(OpcodeKind::In, word, 1, io_register_fields(word))
    } else if word & 0xf800 == 0xb800 {
        d(OpcodeKind::Out, word, 1, io_register_fields(word))
    } else if word & 0xf000 == 0xc000 {
        d(OpcodeKind::Rjmp, word, 1, relative_fields(word, 12))
    } else if word & 0xf000 == 0xd000 {
        d(OpcodeKind::Rcall, word, 1, relative_fields(word, 12))
    } else if word & 0xf000 == 0xe000 {
        d(OpcodeKind::Ldi, word, 1, high_immediate_fields(word))
    } else if word & 0xfc00 == 0xf000 {
        d(OpcodeKind::Brbs, word, 1, branch_fields(word))
    } else if word & 0xfc00 == 0xf400 {
        d(OpcodeKind::Brbc, word, 1, branch_fields(word))
    } else if word & 0xfe08 == 0xf800 {
        d(OpcodeKind::Bld, word, 1, bit_register_fields(word))
    } else if word & 0xfe08 == 0xfa00 {
        d(OpcodeKind::Bst, word, 1, bit_register_fields(word))
    } else if word & 0xfe08 == 0xfe00 {
        d(OpcodeKind::Sbrs, word, 1, bit_register_fields(word))
    } else if word & 0xfc08 == 0xfc00 {
        d(OpcodeKind::Sbrc, word, 1, bit_register_fields(word))
    } else {
        return Err(DecodeError::Reserved { word });
    };

    let profile_form_supported = match decoded.kind {
        OpcodeKind::Lpm => decoded.word == 0x95c8 || profile.core() != CoreProfile::Avr,
        OpcodeKind::Spm => {
            decoded.word != 0x95f8
                || matches!(profile.core(), CoreProfile::Avrxm | CoreProfile::Avrxt)
        }
        _ => true,
    };
    if !profile.supports(decoded.kind) || !profile_form_supported {
        return Err(DecodeError::Unsupported {
            word,
            kind: decoded.kind,
            profile: profile.core(),
        });
    }
    Ok(decoded)
}

const fn d(
    kind: OpcodeKind,
    word: u16,
    length: u8,
    fields: InstructionFields,
) -> DecodedInstruction {
    DecodedInstruction::new(kind, word, length, fields)
}

fn register_fields(word: u16) -> InstructionFields {
    InstructionFields {
        rd: ((word >> 4) & 0x1f) as u8,
        ..InstructionFields::default()
    }
}
fn rr_fields(word: u16) -> InstructionFields {
    InstructionFields {
        rd: ((word >> 4) & 0x1f) as u8,
        rr: ((word & 0x0f) as u8) | (((word >> 9) & 1) as u8) << 4,
        ..InstructionFields::default()
    }
}
fn movw_fields(word: u16) -> InstructionFields {
    InstructionFields {
        pair: (((word >> 4) & 0x0f) as u8) * 2,
        immediate: (word & 0x0f) * 2,
        ..InstructionFields::default()
    }
}
fn word_pair_immediate_fields(word: u16) -> InstructionFields {
    InstructionFields {
        pair: 24 + (((word >> 4) & 3) as u8) * 2,
        immediate: ((word >> 2) & 0x30) | (word & 0x0f),
        ..InstructionFields::default()
    }
}
fn restricted_mul_fields(word: u16, base: u8, width: u8) -> InstructionFields {
    let mask = (1u16 << width) - 1;
    InstructionFields {
        rd: base + ((word >> 4) as u8 & mask as u8),
        rr: base + (word as u8 & mask as u8),
        ..InstructionFields::default()
    }
}
fn high_immediate_fields(word: u16) -> InstructionFields {
    InstructionFields {
        rd: 16 + ((word >> 4) & 0x0f) as u8,
        immediate: ((word >> 4) & 0xf0) | (word & 0x0f),
        ..InstructionFields::default()
    }
}
fn immediate_fields(_word: u16, immediate: u16) -> InstructionFields {
    InstructionFields {
        immediate,
        ..InstructionFields::default()
    }
}
fn bit_fields(word: u16) -> InstructionFields {
    InstructionFields {
        bit: ((word >> 4) & 7) as u8,
        ..InstructionFields::default()
    }
}
fn io_bit_fields(word: u16) -> InstructionFields {
    InstructionFields {
        io: ((word >> 3) & 0x1f) as u8,
        bit: (word & 7) as u8,
        ..InstructionFields::default()
    }
}
fn io_register_fields(word: u16) -> InstructionFields {
    InstructionFields {
        rd: ((word >> 4) & 0x1f) as u8,
        io: (((word >> 5) & 0x30) | (word & 0x0f)) as u8,
        ..InstructionFields::default()
    }
}
fn bit_register_fields(word: u16) -> InstructionFields {
    InstructionFields {
        rd: ((word >> 4) & 0x1f) as u8,
        bit: (word & 7) as u8,
        ..InstructionFields::default()
    }
}
fn long_fields(word: u16) -> InstructionFields {
    InstructionFields {
        high_address: (((word >> 3) & 0x3e) | (word & 1)) as u8,
        ..InstructionFields::default()
    }
}
fn relative_fields(word: u16, bits: u8) -> InstructionFields {
    let raw = word & ((1u16 << bits) - 1);
    InstructionFields {
        relative: sign_extend(raw, bits),
        ..InstructionFields::default()
    }
}
fn branch_fields(word: u16) -> InstructionFields {
    InstructionFields {
        bit: (word & 7) as u8,
        relative: sign_extend((word >> 3) & 0x7f, 7),
        ..InstructionFields::default()
    }
}
fn program_load_fields(word: u16) -> InstructionFields {
    InstructionFields {
        rd: ((word >> 4) & 0x1f) as u8,
        immediate: word & 3,
        ..InstructionFields::default()
    }
}
fn pointer_fields(word: u16) -> InstructionFields {
    let rd = ((word >> 4) & 0x1f) as u8;
    let mode = word & 3;
    let is_x = (word & 0xfe0e == 0x900c)
        || (word & 0xfe0f == 0x900e)
        || (word & 0xfe0e == 0x920c)
        || (word & 0xfe0f == 0x920e);
    let address_mode = if is_x {
        Some(match mode {
            0 => AddressMode::X,
            1 => AddressMode::XPostIncrement,
            _ => AddressMode::XPreDecrement,
        })
    } else {
        let y = word & 8 != 0;
        match mode {
            1 => Some(if y {
                AddressMode::YPostIncrement
            } else {
                AddressMode::ZPostIncrement
            }),
            2 => Some(if y {
                AddressMode::YPreDecrement
            } else {
                AddressMode::ZPreDecrement
            }),
            _ => None,
        }
    };
    InstructionFields {
        rd,
        address_mode,
        ..InstructionFields::default()
    }
}
fn displacement_fields(word: u16) -> InstructionFields {
    let q = (((word >> 13) & 1) << 5 | ((word >> 10) & 3) << 3 | (word & 7)) as u8;
    let y = word & 8 != 0;
    InstructionFields {
        rd: ((word >> 4) & 0x1f) as u8,
        q,
        address_mode: Some(if q == 0 {
            if y {
                AddressMode::Y
            } else {
                AddressMode::Z
            }
        } else if y {
            AddressMode::YDisplacement(q)
        } else {
            AddressMode::ZDisplacement(q)
        }),
        ..InstructionFields::default()
    }
}
fn sign_extend(raw: u16, bits: u8) -> i16 {
    let value = raw as i16;
    let sign = 1i16 << (bits - 1);
    if value & sign != 0 {
        value | (!((1i16 << bits) - 1))
    } else {
        value
    }
}
