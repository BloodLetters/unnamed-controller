# TODO.md — Rencana Lanjutan (serah-terima ke chat baru)

Dokumen ini merangkum **status terkini** dan **semua pekerjaan yang belum selesai**, cukup lengkap
untuk melanjutkan di chat baru tanpa membaca seluruh riwayat percakapan. Detail tambahan ada di
`ISSUE.md` (rujukan per item).

---

## 0. Ringkasan status

### Sudah selesai
- **Rewrite ESP32 → ESP32-S3 (esp32sim)** — emulator CPU Xtensa LX7 nyata, ganti total VM bytecode.
  Stage 1–6 selesai; firmware ESP-IDF asli terbukti boot di emulator (lihat §6).
- **Pass lanjutan (TODO-2 .. TODO-9)**: jalur baca I2C + `Wire` (termasuk **interupsi TWI**),
  register map MPU-6050, decode UART UTF-8, versioning `ProjectData`, integration test `sim-ui`,
  build WASM `app-web` lolos (`cargo check --target wasm32-unknown-unknown -p app-web`),
  `avr/mod.rs` dipecah (`bus.rs`/`peripherals.rs`), dan verifikasi Library Manager.
- **Korektnis mesin simulasi & komponen** (lihat `ISSUE.md` bagian "Catatan: yang sudah diselesaikan").
- **Registry komponen terpusat** (`crates/sim-ui/src/registry.rs`): palette/selection/deletion/mutation.
- **Editor Firmware full-page** (`crates/sim-ui/src/editor/`): tab Editor/Libraries, syntax
  highlighting ala VS Code (`highlight.rs`), Library Manager (cari/install/update via `arduino-cli`),
  posisi kursor `Ln/Col`.
- **Jalur AVR (Route 1) — sebagian besar**:
  - `crates/sim-components/src/mcu/avr/` — core CPU (via `avr-emulator`), GPIO (PIN/DDR/PORT),
    Timer0 + interupsi, Timer1/2 + write-1-clear TIFR, USART0 (Serial), TWI/I2C (jalur **tulis**),
    ADC, EEPROM, SPI (tanpa perangkat), model waktu 16 MHz (16.000 siklus/tick 1 ms).
  - Registrasi UI: palette "Arduino Uno (AVR)", inspector, kanvas, snapshot (`AvrImage`).
  - Toolchain `arduino-cli` (`crates/sim-ui/src/toolchain.rs`) + peringatan startup bila CLI tidak ada.
  - **Terverifikasi end-to-end**: `blink.ino` (dikompilasi `arduino-cli`) → Serial mencetak `BOOT`
    dan LED D13 berkedip (GPIO + Timer0 + interupsi + `delay()/millis()` + USART bekerja).
  - **Presisi waktu terverifikasi** (TODO-1, lihat di bawah): `delay(100)` = 100 tick, `micros()`
    monoton, dan digit `Serial.print` benar.

### Bug yang ditemukan saat verifikasi (sudah diperbaiki) — JANGAN di-regress
1. Profil CPU harus `Profile::avre_plus()` (avr5 = ATmega328P), bukan `Profile::avr()`.
2. **Crate pihak ketiga `avr-emulator` 0.1.2 punya bug decoder**: mask grup satu-operand salah
   (`word & 0xff0f` → harus `0xfe0f`), sehingga `COM/NEG/SWAP/INC/ASR/LSR/ROR/DEC` gagal untuk
   register 16–31. **Solusi: crate di-vendor** ke `vendor/avr-emulator/` (BSD-3-Clause) dan
   ditambal di `vendor/avr-emulator/src/decode.rs`; `crates/sim-components/Cargo.toml` menunjuk
   ke path itu. Jika kelak update crate, tambalan harus diterapkan ulang.
3. **Alamat vektor interupsi**: ATmega328P memakai tabel vektor `JMP` (2 word/vektor), jadi word
   address = indeks × 2 (TIMER0_OVF = 32, bukan 16). Lihat konstanta `VECTOR_TIMER*` di
   `crates/sim-components/src/mcu/avr/mod.rs`.
4. **Presisi waktu `delay()`/`micros()` (TODO-1)**: `TIFR0.TOV0` harus dibersihkan hardware saat
   vektor interupsi dieksekusi. Sebelumnya `advance_timer0` menulis ulang TIFR dari latch internal
   dan mengosongkan `overflow_flag` saat interrupt diantre, sehingga `TOV0` tetap 1 setelah
   `timer0_overflow_count` naik → `micros()` menghitung ganda lalu berbalik (non-monoton ~1024 µs).
   Akibatnya `delay()` berhenti setelah lompatan mundur pertama (`delay(100)` ≈ 8,5 tick).
   Perbaikan: flag timer kini *sticky*, TIFR hanya di-set (tidak dihapus) oleh `advance_*`,
   interrupt diantre sekali (`queue_interrupt`), dan flag di-*acknowledge* (`acknowledge_timer_vector`)
   saat vektor diambil. **Jangan kembalikan** penulisan ulang TIFR yang menghapus bit.
5. **`SUBI` salah di crate `avr-emulator`**: `sub_immediate` memakai carry-in (`self.sreg & FLAG_C`)
   untuk SUBI, padahal hanya SBCI yang mengurangi carry. Akibatnya SEMUA digit `Serial.print`
   kurang satu (`0`→`/`, `9`→`8`, `123456`→`012345`). Ditambal di
   `vendor/avr-emulator/src/cpu.rs` (`sub_immediate(rd, imm, carry_in, preserve_z)`), plus regresi
   unit (`subi_ignores_carry_in`, `subi_builds_ascii_digit`, `sbci_uses_carry_in`).
6. **Interupsi TWI tidak dimodelkan**: library `Wire` bersifat interrupt-driven (`TWI_vect`,
   vektor 24). Tanpa interupsi ini, `Wire.endTransmission()`/`requestFrom` menggantung selamanya.
   Sim kini menaikkan interupsi TWI dari `twi_complete()` (saat `TWIE` aktif) dan menahan `TWINT`
   pada `SLA+R` sampai host menyuplai respons. **Jangan hilangkan pemodelan interupsi TWI.**

### Lingkungan (mesin ini)
- `arduino-cli` 1.5.1: `C:\Program Files\Arduino CLI\arduino-cli.exe` (tidak di PATH; aplikasi
  auto-detect lewat `toolchain::resolve_cli_path`).
- Core terpasang: `arduino:avr 1.8.8`. Sketchbook: `C:\Users\ADVAN\Documents\Arduino`.
- Library: `Servo` 1.3.0 terpasang (hasil verifikasi Library Manager).
- **ESP-IDF v6.1** terpasang: `C:\Users\ADVAN\Downloads\esp-idf-v6.1` (`export.ps1` ada, `idf.py` di PATH).
  Mask ROM ELF: `C:\Users\ADVAN\.espressif\tools\esp-rom-elfs\20241011\esp32s3_rev0_rom.elf`.

---

## 1. Perintah validasi (selalu jalankan sebelum selesai)
```
cargo fmt --all
cargo fmt --all -- --check
cargo clippy --workspace --all-targets
cargo test --workspace
```
Uji AVR end-to-end (env-gated, dilewati bila env tidak diisi):
```powershell
$cli = Join-Path $env:ProgramFiles 'Arduino CLI\arduino-cli.exe'
$sketch = '<folder berisi blink\blink.ino>'
& $cli compile --fqbn arduino:avr:uno --output-dir "$sketch\out" "$sketch\blink"
$env:AVR_TEST_HEX = "$sketch\out\blink.ino.hex"
$env:AVR_TEST_DELAY_MS = "100"   # opsional: sketch berkedip D13 dengan delay(100)
cargo test -p sim-components --test avr_e2e -- --nocapture
# Uji I2C/Wire (MPU-6050) di level komponen dan UI:
$env:AVR_TEST_WIRE_HEX = "$sketch\out\wire.ino.hex"
cargo test -p sim-components --test avr_e2e -- --nocapture
cargo test -p sim-ui --test avr_sensor_e2e -- --nocapture
# Uji ESP32-S3 (butuh ESP-IDF; jalankan dari terminal ESP-IDF):
$env:ESP32_TEST_BIN = "<proyek-esp-idf>\build\merged.bin"
$env:ESP32_TEST_ROM = "$env:USERPROFILE\.espressif\tools\esp-rom-elfs\<ver>\esp32s3_rev0_rom.elf"
cargo test -p sim-components --test esp32_s3_e2e -- --nocapture
```
`avr-objdump` tersedia di
`%LOCALAPPDATA%\Arduino15\packages\arduino\tools\avr-gcc\7.3.0-atmel3.6.1-arduino7\bin\avr-objdump.exe`.

---

## 2. TODO — prioritas tinggi

### TODO-1 — Presisi waktu AVR (`delay()`/`millis()`)
**Status:** SELESAI.

Akar masalah: `micros()` **non-monoton**. `advance_timer0` menulis ulang `TIFR0` dari latch internal
dan mengosongkan `overflow_flag` begitu interrupt diantre, sehingga `TOV0` tetap 1 setelah
`timer0_overflow_count` naik. `micros()` (`((m<<8)+t)*4`) kemudian menambah 256 pada `m` lagi →
lompat +1024 µs, lalu turun lagi saat `TOV0` dibersihkan tick berikutnya. `delay()` berbasis
`micros() - start`: satu lompatan mundur membuat selisih `unsigned long` *underflow* (~4 miliar),
sehingga loop dalam menguras semua `ms` sekaligus → `delay(100)` selesai di lompatan mundur pertama
(≈8,5 tick).

Perbaikan (di `crates/sim-components/src/mcu/avr/mod.rs`): semantik flag timer disamakan hardware —
flag bersifat *sticky*, `advance_*` hanya men-*set* `TIFR` (bukan menghapus bit), interrupt diantre
sekali (`queue_interrupt`), dan flag di-*acknowledge* saat vektor dieksekusi
(`acknowledge_timer_vector`). Ini memastikan `TOV0` bersih tepat sebelum ISR menaikkan
`timer0_overflow_count`, seperti perangkat nyata.

Verifikasi:
- Uji monotonicitas: `micros()` mundur 205× (drop terburuk 1020 µs) → **0×** setelah perbaikan.
- `delay(100)` = **100 tick** (sebelumnya ~8,5); `DELAY100_US` = 100012 µs.
- `avr_e2e.rs` kini meng-assert kestabilan setengah periode D13 dan (bila `AVR_TEST_DELAY_MS` diisi)
  kecocokan periode dengan `delay()` ±5%.
- Unit test regresi: `test_timer0_overflow_flag_cleared_on_interrupt_entry`.

**Temuan sampingan (ikut diperbaiki):** semua digit `Serial.print` kurang satu (mis. `9`→`8`,
`123456`→`012345`) karena bug `SUBI` di vendor `avr-emulator` (lihat “Bug yang ditemukan” no. 5).

### TODO-2 — Tahap 5: jalur baca I2C (TWI `SLA+R`)
**Status:** SELESAI.

- `process_twi` kini menangani `SLA+R`: mengirim `I2cTransaction::Read { address, length }` dan
  menahan `TWINT` sampai host menyuplai respons (`AvrMcu::supply_i2c_response`); byte dibaca
  berurutan (status `0x50` saat ACK, `0x58` saat NACK). Bila tak ada device, `update()` melaporkan
  NACK (`0x48`) setelah satu tick.
- **Temuan penting:** library `Wire` bersifat **interrupt-driven** (`TWI_vect`, vektor 24). Sim
  sebelumnya tidak memodelkan interupsi TWI, sehingga `Wire.endTransmission()`/`requestFrom`
  menggantung. Ditambahkan `twi_complete()` yang menaikkan interupsi TWI saat `TWIE` aktif — jalur
  **tulis** Wire pun baru benar-benar jalan.
- `sim.rs` (`dispatch_i2c_transaction`) merutekan Write **dan** Read ke `lcds`/`oleds`/`mpu6050s`
  dan menyuplai respons ke AVR.
- Register map **MPU-6050** (`ISSUE-013`): `WHO_AM_I` (0x68), `ACCEL_*` (0x3B–0x40), `GYRO_*`
  (0x43–0x48), pointer auto-increment.
- Verifikasi e2e (sketch `Wire.requestFrom`): `WHO=68`. Unit test:
  `test_twi_master_read_transaction`, `test_twi_read_without_device_reports_nack`,
  `test_mpu6050_register_map_over_i2c`; e2e `avr_e2e::run_wire_read_sketch`.

### TODO-3 — Tahap 8: target Web/WASM (ISSUE-070)
**Status:** SELESAI (build lolos; `trunk` belum dipasang di mesin ini).

- `crates/app-web/Cargo.toml`: dependency wasm (`wasm-bindgen-futures`, `web-sys`,
  `console_error_panic_hook`) di-scope ke `cfg(target_arch = "wasm32")`.
- `index.html`: tambah `<link data-trunk rel="rust" />`; tambah `Trunk.toml` di root.
- `sim-ui`/`sim-core` dipastikan bisa di-*build* untuk wasm: `rfd::FileDialog` dan method Library
  Manager di-gate `#[cfg(not(target_arch = "wasm32"))]` dengan stub error ramah di browser;
  `main.rs` disesuaikan ke API eframe 0.27 (`WebRunner::start` memakai id kanvas).
- **Terverifikasi:** `cargo check --target wasm32-unknown-unknown -p app-web` lolos tanpa warning.
  Sisa opsional: `cargo install trunk` lalu `trunk serve`.

### TODO-4 — Pecah `crates/sim-components/src/mcu/avr/mod.rs`
**Status:** SELESAI.

`mod.rs` kini **571 baris** (dari ~1030) dengan pemisahan tanggung jawab:
- `avr/bus.rs` — adapter bus CPU (`FlashBus`, `DataMem`, `IoMem`).
- `avr/peripherals.rs` — model periferal (timer, TWI, USART, ADC, SPI, EEPROM, interupsi).
- `avr/mod.rs` — peta register, struct `AvrMcu`, API host, dan glue.

---

## 3. Status item prioritas sedang

### TODO-5 — Verifikasi I2C tulis dengan sketch nyata + LCD/OLED
**Status:** SELESAI (akar masalah). Interupsi TWI yang hilang (TODO-2) membuat seluruh jalur tulis
`Wire` menggantung; setelah diperbaiki, sketch yang menulis register MPU-6050 berhasil
(`endTransmission() == 0`). Verifikasi visual LCD/OLED dengan library eksternal tetap opsional.

### TODO-6 — Library eksternal di Library Manager
**Status:** SELESAI (terverifikasi di mesin ini). `lib update-index` (unduh 4.31 MiB) → `lib search`
→ `lib install Servo` (1.3.0) → sketch `#include <Servo.h>` terkompilasi (`Sketch uses 1752 bytes`).
Library masuk ke `C:\Users\ADVAN\Documents\Arduino\libraries` dan terdeteksi saat kompilasi.
(Parser `parse_search_results`/`parse_installed_libraries` unit-tested.)

### TODO-7 — USART: baud rate & framing (ISSUE-010)
**Status:** SEBAGIAN. Decode ASCII/UTF-8 sudah benar (byte TX dikumpulkan dan di-decode UTF-8,
dengan `U+FFFD` untuk sekuens tidak valid; test multi-byte ditambahkan). **Belum**: model waktu
kirim/baud (UBRR) — design sengaja menjaga `UDRE0` selalu siap agar `Serial` tak butuh interupsi
USART-UDRE; menambah baud berarti harus memodelkan interupsi TX USART.

### TODO-8 — Persistence versioning (ISSUE-050)
**Status:** SELESAI. `ProjectData` punya `version` (`PROJECT_VERSION = 1`, `#[serde(default)]` untuk
data lama) + API `save_project_json`/`load_project_json` (menolak versi lebih baru). Catatan: tombol
Save/Load `.json` di UI masih belum terpasang (gap terpisah dari ROAD Phase 6). Unit test:
roundtrip, legacy tanpa versi, penolakan versi lebih baru.

### TODO-9 — Integration test end-to-end di level UI (ISSUE-052)
**Status:** SELESAI. `crates/sim-ui/tests/avr_sensor_e2e.rs` menjalankan firmware AVR melalui
`tick_simulation` asli (termasuk `dispatch_i2c_transaction`) → MPU-6050 → `uart_buffer` (`WHO=68`).
Env-gated `AVR_TEST_WIRE_HEX`.

---

## 4. TODO — prioritas rendah / kebersihan

- **ISSUE-002** Driver transient analog belum dipakai `Engine` (`analog/transient.rs`).
- **ISSUE-003** Scheduler event `Engine` belum dimanfaatkan.
- **ISSUE-040** LED belum menghitung arus / resistor seri.
- **ISSUE-041** Kontensi driver digital selalu → Low.
- **ISSUE-042** Model transistor masih pendekatan.
- **ISSUE-031** `step()` debugger melewati breakpoint/periferal.
- **ISSUE-032** Wi-Fi mock hardcoded.
- **ISSUE-051** Audio bergantung frame rate (wall-clock).
- **ISSUE-012** SPI tanpa perangkat (transfer no-op 0xFF).
- **ISSUE-020/021/022** (compiler VM mainan: `else`/variabel, statement timer/watchdog,
  `//` di dalam string) — prioritas turun karena jalur AVR sudah ada.
- **ISSUE-060** Selesaikan sentralisasi registry (inspector/kanvas/pin_positions ke `Spec`).

---

## 5. Peta file kunci

| Area | Lokasi |
|---|---|
| Core AVR | `crates/sim-components/src/mcu/avr/{mod.rs,bus.rs,peripherals.rs,timer8.rs,timer16.rs,tests.rs}` |
| Core ESP32-S3 | `crates/sim-components/src/mcu/esp32_s3.rs` (+ `pinout.rs`), dep git `esp32s3`/`esp-soc` |
| Toolchain ESP-IDF | `crates/sim-ui/src/toolchain.rs` (`EspIdfCompiler`, `resolve_export_ps1`, `resolve_rom_elf`) |
| Uji e2e ESP32-S3 | `crates/sim-components/tests/esp32_s3_e2e.rs` |
| Crate vendor (ditambal) | `vendor/avr-emulator/` (lihat `src/decode.rs` + `src/cpu.rs`) |
| Uji e2e AVR | `crates/sim-components/tests/avr_e2e.rs`, `crates/sim-ui/tests/avr_sensor_e2e.rs` |
| Persistensi proyek | `crates/sim-ui/src/state/snapshot.rs` (+ `types.rs` `PROJECT_VERSION`) |
| Build Web/WASM | `crates/app-web/{Cargo.toml,index.html,src/main.rs}` + `Trunk.toml` |
| Toolchain CLI | `crates/sim-ui/src/toolchain.rs` |
| Library Manager UI | `crates/sim-ui/src/editor/view.rs` (`render_library_panel`) |
| Editor + highlight | `crates/sim-ui/src/editor/{view.rs,types.rs,highlight.rs,actions.rs}` |
| Wiring listrik | `crates/sim-ui/src/sim.rs` |
| App/loop utama | `crates/sim-ui/src/app.rs` |
| Registry komponen | `crates/sim-ui/src/registry.rs` |

---

## 6. Rewrite ESP32 → ESP32-S3 (esp32sim)

**Tujuan:** ganti total VM bytecode ESP32 dengan emulator CPU nyata (Xtensa LX7) dari
`joakimeriksson/esp32sim` (MIT), agar bisa menjalankan firmware ESP-IDF asli seperti Wokwi.

**Keputusan (dikonfirmasi):** target **ESP32-S3**; pengguna menyiapkan **ESP-IDF**; **ganti total**
VM lama (bukan varian berdampingan).

**Status Stage 1 — SELESAI (terverifikasi).**
- `crates/sim-components/Cargo.toml`: dependency git dipin ke commit
  `4ab7e900fee998d0137c2c57d59de9d05339d093` — `esp-soc` dan `s3emu` (package `esp32s3`).
- `crates/sim-components/src/mcu/esp32_s3.rs`: `Esp32S3Mcu` membungkus `s3emu::Machine`
  (`s3emu::machine(mac)`), dengan:
  - `load_rom(&[u8])`, `load_flash(&[u8])` (write_flash 0 + `boot_rom`), `reset`, `set_powered`.
  - `notify_pin_change` → `SocBus::gpio_set_input`; output via `observe_gpio(true)` +
    `take_gpio_events()` → `pin_drives`/`pin_states`.
  - `send_serial_input` → `SocBus::serial_input`; `console_take()` → `uart_buffer`.
  - `update()` menjalankan `machine.run(INSTRUCTIONS_PER_TICK)` per tick 1 ms (tangani `Stop::SwReset`).
- Diverifikasi: `cargo check/test -p sim-components` lolos, smoke test `constructs_and_runs_without_firmware`,
  `cargo clippy --workspace --all-targets` bersih.

**Status Stage 2 — SELESAI (terverifikasi).**
- `crates/sim-components/src/mcu/pinout.rs`: `build_esp32s3_pinout` (layout 30-pin DevKitC: nama pin
  + peta GPIO → pin, mis. `IO4`=GPIO4, `TX0`=GPIO43, `RX0`=GPIO44).
- `Esp32S3Mcu` diperluas: `pin_names()`, `power_pins()`, `load_firmware()`/`flash_image()`, `pc()`,
  `elapsed_seconds()`, `instructions()`, `running`. Di-re-export dari crate (`build_esp32s3_pinout`).
- Tes: `pinout_exposes_named_gpios`, `constructs_and_runs_without_firmware`. `cargo clippy` bersih,
  81 tes sim-components lolos.

**API kunci esp32sim (hasil recon).**
- `esp32s3::{machine, Machine, S3}`; `Machine::run(max_insns) -> Stop`,
  `load_rom/write_flash/boot_app/boot_rom/reboot`, `seconds()`, `insns()`, `dump_regs()`.
- Trait `esp_soc::BoardModel` (di `esp-soc`): `gpio_changes`, `take_edges`, `input_levels`,
  `i2c_devices`, `named_pin`, `next_deadline`/`advance_to`, `display`, `leds`, `touch`.
- Trait `esp_soc::SocBus`: `gpio_set_input`, `take_gpio_events`, `console_take`, `serial_input`,
  `uart_input`, `set_flash_size`, `set_psram_size`, `set_strap`, `board()`.

**Status Stage 3 & 4 — SELESAI (terverifikasi compile + tes).**
- `ProjectData.esp32s` kini `Vec<((f32,f32), Esp32S3Image)>` (`Esp32S3Image { pins, firmware }`,
  serde + `#[serde(default)]`); snapshot menyimpan **bytes firmware**, `restore` membangun ulang
  `Esp32S3Mcu::from_image`.
- Seluruh `sim-ui` di-rewire ke `Esp32S3Mcu`: `app/types/spawn/snapshot/sim/pins/canvas/inspector/
  editor/{view,actions}/audio/debugger`.
- **Dihapus (ganti total):** `mcu/esp32.rs` (VM), `mcu/esp32_tests.rs`, `mcu/ledc.rs`, `mcu/wifi.rs`,
  `build_esp32_pinout`. (`interrupt.rs`/`timer.rs` dipertahankan karena dipakai `DummyMcu`.)
- **Fitur lama yang hilang** (sesuai kesepakatan): debugger VM (disassembly/breakpoint/hex editor VM),
  panel LEDC/DAC/WiFi-mock. **Tersisa:** Serial Monitor (via `console_take`), Flash `.bin`,
  debugger S3 (Step/Resume/Pause + disasm + register dump), inspector S3.
- Verifikasi: `cargo fmt` ✔, `cargo clippy --workspace --all-targets` bersih, `cargo test --workspace`
  hijau (77 sim-components + 8 vendor + 41 core + 26 ui + 2+1 e2e), `cargo check` WASM `app-web` ✔.

**Status Stage 5 — SELESAI (terverifikasi end-to-end).**
- `crates/sim-ui/src/toolchain.rs`: `EspIdfCompiler` + `resolve_export_ps1()` (env `ESP_IDF_EXPORT_PS1`
  / `IDF_PATH`, atau lewat `idf.py` di PATH). Build dijalankan lewat `export.ps1` (ESP-IDF v6 wajib
  env-nya), lalu `idf.py set-target esp32s3 / build / merge-bin -o merged.bin`. `resolve_rom_elf()`
  mencari `~/.espressif/tools/esp-rom-elfs/*/esp32s3_rev0_rom.elf` atau env `ESP32SIM_ROM_ELF`.
- `editor/actions.rs` `build_and_flash_esp32()` + tombol **"⬆ Build & Flash ESP32-S3"**.
- **Temuan penting (jangan di-regress):**
  1. `idf.py` **tidak bisa dipanggil langsung** di Windows (skrip `.py`, butuh env `ESP_IDF_VERSION`
     dsb.) → jalankan via `powershell … & export.ps1; & idf.py …`.
  2. `idf.py merge-bin -o <path>` relatif terhadap folder `build/` → gunakan `-o merged.bin`, lalu
     baca `<project>/build/merged.bin`.
  3. Project build **harus di path pendek** (`%TEMP%\uc_idf_<pid>` OK). Path dalam (mis. scratchpad)
     membuat xtensa-GCC *segfault* (`internal compiler error`).
  4. Console ESP-IDF diambil lewat `Machine::drain_console()` + `console.uart0` (bukan `bus.console_take()`),
     dengan `console.capture = true`.

**Stage 6 — verifikasi (SELESAI).**
- Firmware ESP-IDF asli (`hello_world` + `printf`) dibangun lewat pipeline di atas dan **boot di emulator**:
  `HELLO_FROM_S3`, `TICK …`, `DONE` tertangkap di UART0. Uji: `crates/sim-components/tests/esp32_s3_e2e.rs`
  (env `ESP32_TEST_BIN`, `ESP32_TEST_ROM`, `ESP32_TEST_EXPECT`). Terverifikasi di mesin ini (ESP-IDF v6.1).
- Sisa opsional: WiFi (`esp_soc::wifi` + NAT), dan GPIO↔netlist bridge (uji LED).

**Bridge I2C ESP32-S3 → komponen simulator — SELESAI (terverifikasi).**
- `crates/sim-components/src/mcu/esp32_s3.rs`: `BridgeBoard` (impl `esp_soc::BoardModel`) +
  `BusCapture` (impl `s3emu::i2c::I2cDevice`) yang menangkap byte tulis dan meneruskannya sebagai
  `I2cTransaction::Write` lewat antrean bersama (`Arc<Mutex<…>>`) → di-*drain* ke `i2c_transactions`.
- `sim.rs` sudah mengalirkan `i2c_transactions` (macro `sync_mcu!`) ke `dispatch_i2c_transaction`
  → otomatis sampai ke `app.oleds`/`app.lcds`/sensor by address. **Jalur AVR tidak berubah.**
- Alamat yang dijawab: `0x3C` dan `0x3D` (SSD1306). Read belum didukung (proxy meng-NACK).
- Verifikasi: firmware ESP-IDF `i2c_master_transmit(0x3C, {0x40,'A','B','C','D'})` →
  komponen menangkap `Write { address: 0x3C, data: [0x40,0x41,0x42,0x43,0x44] }`. Unit test:
  `bridge_attaches_i2c_display_devices`. AVR e2e tetap hijau.
- **SSD1306 `on_write` diperbaiki**: kini menghormati perintah addressing (0xB0–0xB7 page, 0x00/0x10
  kolom, 0x21/0x22 window) dan memakai kursor GDDRAM — driver standar (kirim 128 byte per page)
  bekerja. Unit test: `test_ssd1306_page_addressing_targets_the_selected_page`.
- **Terverifikasi visual (data)**: firmware OLED (`i2c_master`, page addressing) direplay ke
  `Ssd1306Oled` → terbaca "HELLO" (page 2) dan "ESP32!" (page 4).
- **I2C kini WAJIB tersambung kabel (realistis seperti Wokwi)**: `dispatch_i2c_transaction` hanya
  mengirim ke device yang **SDA & SCL**-nya berada di node netlist yang sama dengan pin MCU pengirim
  (`wired_to_mcu`). Konsekuensi: OLED/LCD/sensor I2C **tidak** lagi jalan tanpa wiring.
  - AVR: sambungkan OLED/LCD ke **A4 (SDA)** & **A5 (SCL)**.
  - ESP32-S3: sambungkan ke dua pin MCU (eks. IO8/IO9 sesuai firmware). *Catatan:* belum pin-exact
    (boleh pin mana saja yang tersambung) karena emulator tidak mengekspos pin I2C yang dipilih
    firmware; untuk pin-exact perlu baca GPIO-matrix dari esp32sim.
  - Uji: `avr_sensor_e2e::firmware_reads_sensor_through_sim_ui` (tersambung → `WHO=68`) dan
    `avr_sensor_e2e::unwired_sensor_is_unreachable` (tanpa kabel → tidak terbaca).

## §7 Model power implisit + label pin (realisme ala Wokwi)

**Battery dihapus, MCU dianggap sudah berpower.**
- Komponen `Battery` dihapus total (komponen, palette, catalog, snapshot, inspector, renderer,
  `app.batteries`) — 13 file, tanpa sisa.
- `mcu/pinout.rs` kini jadi sumber nama pin: `build_esp32s3_pinout` (3V3/5V/EN/25 GPIO/GND×2) dan
  `build_avr_pinout` (D0–D13, A0–A5 dengan `SDA(A4)`/`SCL(A5)`, lalu 5V/GND/3V3). `ESP32S3_PIN_COUNT`
  = 30, `AVR_PIN_COUNT` = 23.
- `resolve_power_pins()` dipakai bersama `AvrMcu::power_pins()` dan `Esp32S3Mcu::power_pins()`
  (sebelumnya ESP32-S3 selalu salah karena `pin_names` tak punya "GND").
- `sim.rs::drive_power_terminals` menggerakkan terminal suplai MCU: 5V/3V3 → High (+ fixed 5.0/3.3 V
  di nodal solver), GND → Low (+ fixed 0.0 V). Tidak ada lagi battery sebagai referensi.
- `DummyMcu` memakai `power_positive_pin`/`power_negative_pin` yang sudah ada.
- Uji: `sim-ui/tests/implicit_power.rs` — LED menyala dari power MCU tanpa battery, dan tetap mati
  kalau katodanya tidak tersambung.

**Label pin di semua komponen.**
- Tabel label terpusat di `sim-ui/src/pins.rs::labels_for()` (kunci = nama komponen): A/K, SDA/SCL,
  VCC/GND, DO/AO, TRIG/ECHO, Q0–Q7, DP/COM, B/C/E, G/D/S, dan seterusnya. MCU memakai `pin_names`.
- `pins::pin_labels()` memasangkan label dengan koordinat kanvas; `view.rs` menggambarnya di atas
  wire untuk setiap pin. Label nama ESP32 yang lama di `canvas/mcu.rs` dihapus (sekarang terpusat,
  jadi AVR juga berlabel).
- Uji: `pins::tests::pin_labels_expose_terminal_names`.

**Realisme mekanisme komponen.**
- `PushButton` punya model *contact bounce* deterministik (`contact_closed()`, `BOUNCE_TICKS = 4`);
  `sim.rs` memakai `contact_closed()` sehingga firmware bisa melihat pantulan kontak.
  Uji: `button_chatters_before_settling_closed`, `button_settles_open_after_release`.
- Potensiometer sudah realistis (pembagi tegangan dua resistor di nodal solver) — tidak diubah.

**Model arus LED (non-linier, realistis).**
- `Led` kini junction dengan `forward_voltage` (1.9 V), resistansi seri 20 Ω, leakage balik 10 MΩ,
  plus `companion(v_a, v_c)` mengikuti pola yang sama seperti `Diode`. `current_amps`, `brightness()`
  (skala 20 mA) dan `is_over_current()` (>30 mA) tersedia.
- LED di-stamp ke nodal solver bersama resistor, jadi arusnya **ikut terpecahkan**:
  `V(20 Ω)`-nya: I = (V_a − V_c − V_f) / (R_seri + 20 Ω). `sim.rs` menyimpan model yang dipakai saat
  solve (`led_models`) dan menghitung arus dari solusi dengan model itu, sehingga arus konsisten
  (bukan dievaluasi ulang dengan state yang berbeda). Konvergen lintas tick (1 ms).
- Tegangan rel digital kini mengikuti suplai MCU: node yang di-drive High memakai 5 V (AVR) atau
  3.3 V (ESP32-S3) lewat `record_supply`/`node_supply`, bukan selalu 5 V.
- Inspector LED menampilkan arus (mA), brightness (%), dan peringatan over-current.
- Uji: `led.rs` (4 unit) + `sim-ui/tests/led_current.rs` (resistor 220 Ω → ≈12.9 mA & brightness
  0.65; resistor 1 kΩ lebih redup; tanpa resistor → over-current).

**Ditindaklanjuti di §8:** gradien brightness LED di kanvas dan konsistensi model Diode/BJT/MOSFET.

---

## §8 Save/Load, I2C read ESP32-S3, WiFi, dan realisme lanjutan

**Save & Load project (menu File).**
- `sim-ui/src/state/project.rs`: `new_project`, `save_project(app, force_prompt)` (meminta path bila
  belum ada / Save As), `open_project` — pakai `rfd::FileDialog` + `std::fs`, native saja
  (`#[cfg(not(target_arch = "wasm32"))]`; di wasm jadi no-op agar build tetap jalan).
- `SimulatorApp` menyimpan `project_path` + `project_message`; menu File menampilkan status
  ("Saved x.json" / "Save failed: …").
- Uji: `new_project_clears_the_workspace`, `project_round_trips_through_a_file` (tulis→baca lewat fs).

**I2C read dari ESP32-S3 (bridge dua arah).**
- `Esp32S3Mcu::supply_i2c_response(address, registers)`; `BusProxy` + `Reg8Device` +
  `I2C_REGISTER_WINDOW` (256 byte): alamat tak terpasang → NACK, terpasang → ACK + data.
- `sim-ui/src/sim.rs` mengembalikan data ke MCU yang meminta (`esp.supply_i2c_response`), seperti
  jalur AVR. Jalur AVR tidak berubah.
- Uji: unit di `esp32_s3.rs` + e2e `tests/esp32_s3_wifi.rs`.

**WiFi / internet.**
- `apply_wifi()` memasang `s3emu::wifi::ApConfig`/`VirtualAp` + net NAT. Diaktifkan lewat env var
  `UC_ESP32_NETWORK` (default OFF: `wifi.ap`/`wifi.net` = `None`).
- **Belum ada toggle di UI** — masih env var; perlu ditambahkan ke inspector ESP32-S3.

**Realisme lanjutan.**
- LED di kanvas kini bergradasi `brightness()` (radius+alpha glow) dan diberi ring peringatan saat
  over-current; geometri/pin-offset tidak berubah (hit-test aman).
- `Diode`, BJT, dan MOSFET kini punya satu sumber kebenaran (`conduction_mode`/`region`/
  `channel_state`) dengan `DiodeModel`/`BjtModel`/`MosfetModel` + `resolve()`; `companion()` lama
  dipertahankan agar `sim.rs` tetap kompatibel.
- **Sisa:** `sim.rs` belum memakai `resolve(model)` dari model yang benar-benar di-stamp (patch
  eksaknya sudah ditulis), jadi state diode/BJT/MOSFET masih dilaporkan dari tegangan setelah solve.

---

**Catatan penting.**
- Prasyarat runtime: mask ROM ELF `esp32s3_rev0_rom.elf` + merged flash `.bin` (keduanya dari ESP-IDF).
- `esp32s3` crate hanya bergantung pada crate internal workspace (tanpa dependency pihak ketiga) →
  mudah di-*vendor* kelak bila tak ingin git dependency.

---

## 7. Catatan cara kerja
- Bahasa balasan: Indonesia; identifier kode tetap Inggris.
- Selalu jalankan fmt/clippy/test sebelum menyatakan selesai.
- Jangan mengubah file di `.commandcode/taste/`.
- Jangan regress 3 perbaikan di bagian "Bug yang ditemukan" di atas.
