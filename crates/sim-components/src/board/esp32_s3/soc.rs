//! Xtensa LX7 SoC machine initialization and hardware peripherals for ESP32-S3.

use std::sync::{Arc, Mutex};

use crate::board::types::FirmwareError;

pub(crate) type I2cSharedQueue = Arc<Mutex<Vec<(u8, Vec<u8>)>>>;

const DEFAULT_BOOTLOADER: &[u8] =
    include_bytes!("../../../../../assets/rom/esp32s3_bootloader.bin");
const DEFAULT_PARTITIONS: &[u8] =
    include_bytes!("../../../../../assets/rom/esp32s3_partitions.bin");
const DEFAULT_ROM_ELF: &[u8] = include_bytes!("../../../../../assets/rom/esp32s3_rev0_rom.elf");

/// I2C device adapter routing hardware I2C master frames to the shared output queue.
pub(crate) struct S3emuI2cBridge {
    pub(crate) addr: u8,
    pub(crate) buffer: Vec<u8>,
    pub(crate) queue: I2cSharedQueue,
}

impl s3emu::i2c::I2cDevice for S3emuI2cBridge {
    fn start(&mut self, _read: bool) -> bool {
        self.buffer.clear();
        true
    }

    fn write(&mut self, b: u8) -> bool {
        self.buffer.push(b);
        true
    }

    fn read(&mut self) -> u8 {
        0
    }

    fn stop(&mut self) {
        if !self.buffer.is_empty()
            && let Ok(mut q) = self.queue.lock()
        {
            q.push((self.addr, std::mem::take(&mut self.buffer)));
        }
    }
}

/// Spawns and boots an instruction-accurate Xtensa LX7 machine for ESP32-S3 firmware.
pub(crate) fn boot_machine(
    binary: &[u8],
    queue: &I2cSharedQueue,
) -> Result<Option<s3emu::Machine>, FirmwareError> {
    let is_xtensa_app = binary.len() >= 4 && binary[0] == 0xE9;
    if !is_xtensa_app {
        return Ok(None);
    }

    let mut machine = s3emu::soc::machine([0x44, 0x1b, 0xf6, 0x75, 0xdc, 0xe0]);
    machine.console.capture = true;
    machine.vq_max = 256;
    machine.bb_max = 16;

    for &addr in &[0x3C, 0x27, 0x3F] {
        machine.bus.periph.i2c[0].attach(
            addr,
            Box::new(S3emuI2cBridge {
                addr,
                buffer: Vec::new(),
                queue: queue.clone(),
            }),
        );
        machine.bus.periph.i2c[1].attach(
            addr,
            Box::new(S3emuI2cBridge {
                addr,
                buffer: Vec::new(),
                queue: queue.clone(),
            }),
        );
    }

    machine
        .load_rom(DEFAULT_ROM_ELF)
        .map_err(FirmwareError::FlashFailed)?;

    if binary.len() > 1024 * 1024 {
        machine
            .write_flash(0x0, binary)
            .map_err(FirmwareError::FlashFailed)?;
    } else {
        machine
            .write_flash(0x0, DEFAULT_BOOTLOADER)
            .map_err(FirmwareError::FlashFailed)?;
        machine
            .write_flash(0x8000, DEFAULT_PARTITIONS)
            .map_err(FirmwareError::FlashFailed)?;
        machine
            .write_flash(0x10000, binary)
            .map_err(FirmwareError::FlashFailed)?;
    }

    machine.boot_rom();
    Ok(Some(machine))
}
