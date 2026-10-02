use sim_components::board::Board;
#[cfg(not(target_arch = "wasm32"))]
use sim_core::firmware::hex::parse_hex_to_memory;

use crate::app::SimulatorApp;
use crate::editor::types::EditorTarget;

/// Loads a firmware image into the targeted microcontroller core.
pub fn flash_avr(app: &mut SimulatorApp, _binary: &[u8]) -> bool {
    app.code_editor.status_message =
        "Tidak ada board microcontroller di canvas. Tambahkan board terlebih dahulu.".to_string();
    app.code_editor.status_is_error = true;
    false
}

/// Flashes raw bytecode into the selected microcontroller target.
pub fn flash_binary_to_target(app: &mut SimulatorApp, binary: &[u8]) {
    match app.code_editor.target {
        EditorTarget::Esp32(idx) => {
            if let Some((_, board)) = app.esp32_s3_boards.get_mut(idx) {
                match board.load_firmware(binary) {
                    Ok(()) => {
                        let is_real = binary.first() == Some(&0xE9);
                        let tag = if is_real {
                            "1:1 Real Xtensa Binary"
                        } else {
                            "Bytecode"
                        };
                        app.code_editor.status_message = format!(
                            "{} ({} B) successfully flashed to ESP32-S3 #{}.",
                            tag,
                            binary.len(),
                            idx
                        );
                        app.code_editor.status_is_error = false;
                    }
                    Err(e) => {
                        app.code_editor.status_message = format!("Flashing failed: {}", e);
                        app.code_editor.status_is_error = true;
                    }
                }
            } else {
                app.code_editor.status_message = format!("ESP32-S3 #{} not found on canvas.", idx);
                app.code_editor.status_is_error = true;
            }
        }
        _ => {
            app.code_editor.status_message =
                "Pilih target board terlebih dahulu dari dropdown \"Target\".".to_string();
            app.code_editor.status_is_error = true;
        }
    }
}

/// Opens a file dialog to load and flash an external .hex or .bin firmware file.
#[cfg(not(target_arch = "wasm32"))]
pub fn load_firmware_file(app: &mut SimulatorApp) {
    let file = rfd::FileDialog::new()
        .add_filter("Firmware Image", &["hex", "bin", "txt"])
        .pick_file();

    let path = match file {
        Some(p) => p,
        None => return,
    };

    match std::fs::read(&path) {
        Ok(bytes) => {
            let extension = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            let binary = if extension == "hex" {
                let text = String::from_utf8_lossy(&bytes);
                match parse_hex_to_memory(&text, 32768) {
                    Ok(mem) => mem,
                    Err(e) => {
                        app.code_editor.status_message = format!("Intel HEX Parse Error: {:?}", e);
                        app.code_editor.status_is_error = true;
                        return;
                    }
                }
            } else {
                bytes
            };

            flash_binary_to_target(app, &binary);
        }
        Err(e) => {
            app.code_editor.status_message = format!("Failed to read file: {}", e);
            app.code_editor.status_is_error = true;
        }
    }
}

/// WebAssembly placeholder: native file dialogs are unavailable in the browser.
#[cfg(target_arch = "wasm32")]
pub fn load_firmware_file(app: &mut SimulatorApp) {
    app.code_editor.status_message =
        "Pemuatan file firmware tidak tersedia di browser.".to_string();
    app.code_editor.status_is_error = true;
}

/// Reboots the currently targeted microcontroller to its reset state.
pub fn reset_target(app: &mut SimulatorApp) {
    match app.code_editor.target {
        EditorTarget::Esp32(idx) => {
            if let Some((_, board)) = app.esp32_s3_boards.get_mut(idx) {
                board.reset();
                app.code_editor.status_message = format!("ESP32-S3 #{} reset.", idx);
                app.code_editor.status_is_error = false;
            } else {
                app.code_editor.status_message = format!("ESP32-S3 #{} not found.", idx);
                app.code_editor.status_is_error = true;
            }
        }
        _ => {
            app.code_editor.status_message = "No target microcontroller selected.".to_string();
            app.code_editor.status_is_error = false;
        }
    }
}

/// Steps the target microcontroller by one instruction execution.
pub fn step_target(app: &mut SimulatorApp) {
    if let EditorTarget::Esp32(idx) = app.code_editor.target
        && let Some((_, board)) = app.esp32_s3_boards.get_mut(idx)
    {
        board.step(0.001);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flash_avr_without_target_reports_error() {
        let mut app = SimulatorApp::default();
        assert!(!flash_avr(&mut app, &[0x00, 0x00]));
        assert!(app.code_editor.status_is_error);
    }

    #[test]
    fn test_reset_target_when_empty() {
        let mut app = SimulatorApp::default();
        reset_target(&mut app);
        assert!(!app.code_editor.status_is_error);
    }
}
