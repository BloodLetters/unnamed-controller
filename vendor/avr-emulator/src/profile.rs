//! AVR core profiles and their architectural capabilities.

use crate::decode::OpcodeKind;

/// AVR core families named by the Microchip instruction manual.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum CoreProfile {
    /// The original AVR core.
    Avr,
    /// The enhanced AVR core.
    Avre,
    /// The enhanced core with the extended program counter.
    AvrePlus,
    /// XMEGA's AVRxm core.
    Avrxm,
    /// Modern tinyAVR/Dx and related devices' AVRxt core.
    Avrxt,
    /// The reduced-register AVRrc core is intentionally not implemented.
    Avrrc,
}

impl CoreProfile {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Avr => "AVR",
            Self::Avre => "AVRe",
            Self::AvrePlus => "AVRe+",
            Self::Avrxm => "AVRxm/XMEGA",
            Self::Avrxt => "AVRxt",
            Self::Avrrc => "AVRrc",
        }
    }
}

/// A complete set of CPU-facing choices for an AVR core.
///
/// A profile does not describe a device. Peripherals, memory sizes, reset
/// vectors, and fuse behavior belong in the host's bus and hook implementations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Profile {
    core: CoreProfile,
    pc_bits: u8,
    data_bits: u8,
    ramp_x: bool,
    ramp_y: bool,
    ramp_z: bool,
    ramp_d: bool,
    eind: bool,
    des: bool,
    rmw: bool,
    elpm: bool,
    spm: bool,
    long_control: bool,
}

impl Profile {
    pub const fn new(core: CoreProfile) -> Self {
        match core {
            CoreProfile::Avr => Self {
                core,
                pc_bits: 16,
                data_bits: 16,
                ramp_x: false,
                ramp_y: false,
                ramp_z: false,
                ramp_d: false,
                eind: false,
                des: false,
                rmw: false,
                elpm: false,
                spm: false,
                long_control: true,
            },
            CoreProfile::Avre => Self {
                core,
                pc_bits: 16,
                data_bits: 16,
                ramp_x: false,
                ramp_y: false,
                ramp_z: false,
                ramp_d: false,
                eind: false,
                des: false,
                rmw: false,
                elpm: false,
                spm: true,
                long_control: true,
            },
            CoreProfile::AvrePlus => Self {
                core,
                pc_bits: 22,
                data_bits: 24,
                ramp_x: true,
                ramp_y: true,
                ramp_z: true,
                ramp_d: true,
                eind: true,
                des: false,
                rmw: false,
                elpm: true,
                spm: true,
                long_control: true,
            },
            CoreProfile::Avrxm => Self {
                core,
                pc_bits: 22,
                data_bits: 24,
                ramp_x: true,
                ramp_y: true,
                ramp_z: true,
                ramp_d: true,
                eind: true,
                des: true,
                rmw: true,
                elpm: true,
                spm: true,
                long_control: true,
            },
            CoreProfile::Avrxt => Self {
                core,
                pc_bits: 22,
                data_bits: 24,
                ramp_x: true,
                ramp_y: true,
                ramp_z: true,
                ramp_d: true,
                eind: true,
                des: false,
                rmw: false,
                elpm: true,
                spm: true,
                long_control: true,
            },
            CoreProfile::Avrrc => Self {
                core,
                pc_bits: 16,
                data_bits: 16,
                ramp_x: false,
                ramp_y: false,
                ramp_z: false,
                ramp_d: false,
                eind: false,
                des: false,
                rmw: false,
                elpm: false,
                spm: false,
                long_control: false,
            },
        }
    }

    pub const fn avr() -> Self {
        Self::new(CoreProfile::Avr)
    }

    pub const fn avre() -> Self {
        Self::new(CoreProfile::Avre)
    }

    pub const fn avre_plus() -> Self {
        Self::new(CoreProfile::AvrePlus)
    }

    pub const fn avrxm() -> Self {
        Self::new(CoreProfile::Avrxm)
    }

    pub const fn avrxt() -> Self {
        Self::new(CoreProfile::Avrxt)
    }

    /// Construct the deliberately unsupported reduced-register profile.
    pub const fn avrrc() -> Self {
        Self::new(CoreProfile::Avrrc)
    }

    /// Select a 16-bit or 22-bit word-addressed program counter.
    ///
    /// This is useful for a device family whose data sheet uses a smaller
    /// program counter than the family-wide core profile. Values other than
    /// 16 and 22 leave the profile unchanged.
    pub const fn with_program_counter_bits(mut self, bits: u8) -> Self {
        if bits == 16 || bits == 22 {
            self.pc_bits = bits;
        }
        self
    }

    /// Select the physical data address width. The CPU still stores the stack
    /// pointer in the two architectural SP bytes; this setting controls RAMP
    /// address formation and bus address masking.
    pub const fn with_data_address_bits(mut self, bits: u8) -> Self {
        if bits == 16 || bits == 24 {
            self.data_bits = bits;
        }
        self
    }

    pub const fn core(self) -> CoreProfile {
        self.core
    }

    pub const fn program_counter_bits(self) -> u8 {
        self.pc_bits
    }

    pub const fn program_counter_mask(self) -> u32 {
        (1u32 << self.pc_bits) - 1
    }

    pub const fn data_address_bits(self) -> u8 {
        self.data_bits
    }

    pub const fn data_address_mask(self) -> u32 {
        (1u32 << self.data_bits) - 1
    }

    pub const fn return_address_bytes(self) -> u8 {
        if self.pc_bits > 16 {
            3
        } else {
            2
        }
    }

    pub const fn has_ramp_x(self) -> bool {
        self.ramp_x
    }
    pub const fn has_ramp_y(self) -> bool {
        self.ramp_y
    }
    pub const fn has_ramp_z(self) -> bool {
        self.ramp_z
    }
    pub const fn has_ramp_d(self) -> bool {
        self.ramp_d
    }
    pub const fn has_eind(self) -> bool {
        self.eind
    }

    pub(crate) fn supports(self, kind: OpcodeKind) -> bool {
        if matches!(self.core, CoreProfile::Avrrc) {
            return false;
        }
        match kind {
            OpcodeKind::Adiw | OpcodeKind::Sbiw => true,
            OpcodeKind::Movw => self.core != CoreProfile::Avr,
            OpcodeKind::Mul
            | OpcodeKind::Muls
            | OpcodeKind::Mulsu
            | OpcodeKind::Fmul
            | OpcodeKind::Fmuls
            | OpcodeKind::Fmulsu => {
                matches!(
                    self.core,
                    CoreProfile::AvrePlus | CoreProfile::Avrxm | CoreProfile::Avrxt
                )
            }
            OpcodeKind::Des => self.des,
            OpcodeKind::Elpm => self.elpm,
            OpcodeKind::Eijmp | OpcodeKind::Eicall => self.eind,
            OpcodeKind::Xch | OpcodeKind::Las | OpcodeKind::Lac | OpcodeKind::Lat => self.rmw,
            OpcodeKind::Jmp | OpcodeKind::Call => self.long_control,
            OpcodeKind::Spm => self.spm,
            _ => true,
        }
    }

    pub(crate) const fn modern_timing(self) -> bool {
        matches!(self.core, CoreProfile::Avrxm | CoreProfile::Avrxt)
    }
}

impl Default for Profile {
    fn default() -> Self {
        Self::avre_plus()
    }
}
