# Toolchain Setup (1:1 Hardware Match)

Untuk menghasilkan binary firmware yang **1:1 identik** dengan ESP32-S3 fisik dan dapat langsung di-flash via USB:

## 1. Persyaratan Sistem
- **Arduino CLI**: `C:\Program Files\Arduino CLI\arduino-cli.exe`
- **ESP-IDF v6.1 / EIM**: `C:\esp\v6.1\esp-idf` & `C:\Espressif\tools`

## 2. Platform & Library yang Sudah Dikonfigurasi
```bash
# Core ESP32 pada Arduino CLI
arduino-cli core install esp32:esp32

# Library OLED SSD1306 & Grafik
arduino-cli lib install "Adafruit SSD1306" "Adafruit GFX Library"
```

## 3. Alur Kerja 1:1 (Real Binary)
1. Tulis kode C++ sketch standar (dengan `#include <Adafruit_SSD1306.h>`, `#include <Wire.h>`, dll.) pada Code Editor simulator.
2. Pastikan Target di toolbar adalah **ESP32-S3 #0**.
3. Klik tombol **`▶ Flash to MCU`** atau **`⬆ Build & Flash ESP32-S3`**:
   - Compiler `arduino-cli` / `idf.py` akan dijalankan otomatis di latar belakang dengan FQBN `esp32:esp32:esp32s3`.
   - File biner berekstensi `.bin` (magic byte `0xE9`) dihasilkan secara nyata dan disimpan sebagai `./firmware.bin`.
   - Simulator langsung memuat biner tersebut ke emulator SoC Xtensa LX7 (`s3emu`).
4. **Flash ke ESP32 Fisik**:
   File `./firmware.bin` yang dihasilkan adalah biner mesin 1:1. Kamu bisa langsung mem-flash-nya ke ESP32 fisik via USB:
   ```bash
   esptool.py --chip esp32s3 write_flash 0x10000 firmware.bin
   ```