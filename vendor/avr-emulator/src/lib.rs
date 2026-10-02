#![cfg_attr(not(feature = "std"), no_std)]

//! `avr-emulator` is a no-heap AVR CPU core for embedding in simulators,
//! test runners, firmware tools, and game emulators.
//!
//! The CPU owns only architectural state. Program memory, data memory, I/O,
//! SPM/NVM, sleep, break, and watchdog behavior are supplied through hooks.
//! The decoder follows the Microchip AVR Instruction Set Manual. AVRrc is
//! exposed only as an explicit rejected profile; its reduced encodings are not
//! partially accepted.

mod bus;
mod cpu;
mod decode;
mod des;
mod profile;

pub use bus::{
    DataBus, ExecutionHooks, IoBus, NoHooks, ProgramBus, SleepMode, SliceData, SliceIo,
    SliceProgram, SpmOperation,
};
pub use cpu::{
    Cpu, CycleCount, InterruptResult, StepError, StepEvent, StepResult, FLAG_C, FLAG_H, FLAG_I,
    FLAG_N, FLAG_S, FLAG_T, FLAG_V, FLAG_Z,
};
pub use decode::{
    decode, AddressMode, DecodeError, DecodedInstruction, InstructionFields, OpcodeKind,
    OPCODE_INVENTORY,
};
pub use des::des_round;
pub use profile::{CoreProfile, Profile};

/// Decode all 65,536 possible first words and return the number that are
/// recognized by a profile. This is useful for host diagnostics and tests.
pub fn decoded_word_count(profile: Profile) -> usize {
    (0..=u16::MAX)
        .filter(|word| decode(*word, profile).is_ok())
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_first_word_has_one_classification() {
        for profile in [
            Profile::avr(),
            Profile::avre(),
            Profile::avre_plus(),
            Profile::avrxm(),
            Profile::avrxt(),
        ] {
            let mut recognized = 0usize;
            for word in 0..=u16::MAX {
                if decode(word, profile).is_ok() {
                    recognized += 1;
                }
            }
            assert!(recognized > 20);
            assert_eq!(recognized, decoded_word_count(profile));
        }
    }

    #[test]
    fn avr_rc_is_rejected_before_decoding() {
        assert_eq!(
            decode(0x0000, Profile::avrrc()),
            Err(DecodeError::UnsupportedProfile {
                profile: CoreProfile::Avrrc
            })
        );
    }

    #[test]
    fn representative_fields_and_lengths_are_exhaustively_bounded() {
        for word in 0..=u16::MAX {
            if let Ok(instruction) = decode(word, Profile::avre_plus()) {
                assert!(instruction.length == 1 || instruction.length == 2);
                assert!(instruction.fields.rd < 32);
                assert!(instruction.fields.rr < 32);
                assert!(instruction.fields.bit < 8);
                assert!(instruction.fields.io < 64);
                assert!(instruction.fields.q < 64);
            }
        }
    }
}
