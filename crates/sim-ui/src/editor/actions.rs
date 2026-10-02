use sim_components::board::Board;
use sim_core::firmware::compiler::compile;

use crate::app::SimulatorApp;
pub use crate::editor::control::*;
use crate::editor::types::{BuildJobResult, EditorTarget, MonitorTab};
use crate::toolchain::{ArduinoCliCompiler, EspIdfCompiler, FirmwareCompiler};

/// Polls background firmware compilation tasks and completes flashing.
pub fn poll_background_build(app: &mut SimulatorApp, ctx: &egui::Context) {
    #[cfg(not(target_arch = "wasm32"))]
    if let Some(rx) = &app.build_receiver {
        match rx.try_recv() {
            Ok(result) => {
                app.code_editor.build_in_progress = false;
                app.code_editor.build_log.push_str(&result.build_output);

                match result.binary {
                    Ok(bin) => {
                        if let Some((_, board)) = app.esp32_s3_boards.get_mut(result.target_index) {
                            match board.load_firmware(&bin) {
                                Ok(()) => {
                                    let is_real = bin.first() == Some(&0xE9);
                                    let tag = if is_real {
                                        "1:1 Real Binary (saved to ./firmware.bin)"
                                    } else {
                                        "Bytecode"
                                    };
                                    let msg = format!(
                                        "{} ({} B) successfully flashed to ESP32-S3 #{}.",
                                        tag,
                                        bin.len(),
                                        result.target_index
                                    );
                                    app.code_editor
                                        .build_log
                                        .push_str(&format!("\n[FLASH SUCCESS] {}\n", msg));
                                    app.code_editor.status_message = msg;
                                    app.code_editor.status_is_error = false;
                                }
                                Err(e) => {
                                    let err_msg = format!("Flashing failed: {}", e);
                                    app.code_editor
                                        .build_log
                                        .push_str(&format!("\n[FLASH ERROR] {}\n", err_msg));
                                    app.code_editor.status_message = err_msg;
                                    app.code_editor.status_is_error = true;
                                }
                            }
                        }
                    }
                    Err(err) => {
                        app.code_editor
                            .build_log
                            .push_str(&format!("\n[BUILD ERROR] {}\n", err));
                        app.code_editor.status_message =
                            "Kompilasi gagal. Lihat tab Build.".to_string();
                        app.code_editor.status_is_error = true;
                    }
                }
                app.build_receiver = None;
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                ctx.request_repaint();
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                app.code_editor.build_in_progress = false;
                app.build_receiver = None;
            }
        }
    }
}

/// Compiles the active sketch with the Arduino toolchain and flashes the target AVR core.
pub fn compile_and_upload_avr(app: &mut SimulatorApp) {
    let index = app.code_editor.active_tab;
    let source = match app.code_editor.tabs.get(index) {
        Some(tab) => tab.source.clone(),
        None => return,
    };

    let compiler = ArduinoCliCompiler::default();
    match compiler.compile(&source) {
        Ok(binary) => {
            flash_avr(app, &binary);
        }
        Err(error) => {
            app.code_editor.status_message = error;
            app.code_editor.status_is_error = true;
        }
    }
}

/// Compiles the source code in the active editor tab and flashes it to the targeted MCU.
pub fn flash_active_tab(app: &mut SimulatorApp) {
    match app.code_editor.target {
        EditorTarget::Esp32(idx) => {
            build_and_flash_esp32(app, idx);
        }
        _ => {
            let active_idx = app.code_editor.active_tab;
            let source = match app.code_editor.tabs.get(active_idx) {
                Some(tab) => tab.source.clone(),
                None => return,
            };
            match compile(&source) {
                Ok(binary) => {
                    flash_binary_to_target(app, &binary);
                }
                Err(err) => {
                    app.code_editor.status_message =
                        format!("Error on line {}: {}", err.line, err.message);
                    app.code_editor.status_is_error = true;
                }
            }
        }
    }
}

/// Builds the active tab with real ESP32 toolchains in a background thread and flashes the result.
pub fn build_and_flash_esp32(app: &mut SimulatorApp, index: usize) {
    if app.esp32_s3_boards.get(index).is_none() {
        app.code_editor.status_message =
            "Tidak ada ESP32 board di canvas. Tambahkan board terlebih dahulu.".to_string();
        app.code_editor.status_is_error = true;
        return;
    }

    if app.code_editor.build_in_progress {
        return;
    }

    let active_idx = app.code_editor.active_tab;
    let source = match app.code_editor.tabs.get(active_idx) {
        Some(tab) => tab.source.clone(),
        None => return,
    };

    app.code_editor.build_in_progress = true;
    app.code_editor.monitor_tab = MonitorTab::Build;
    app.code_editor.show_serial_monitor = true;
    app.code_editor.status_message = "Building ESP32-S3 firmware in background...".to_string();
    app.code_editor.status_is_error = false;
    app.code_editor.build_log = "[BUILD] Memulai kompilasi sketch ESP32-S3...\n".to_string();

    #[cfg(not(target_arch = "wasm32"))]
    {
        let (tx, rx) = std::sync::mpsc::channel();
        app.build_receiver = Some(rx);

        std::thread::spawn(move || {
            let mut logs = String::new();
            let result = if source.contains("app_main") {
                let idf = EspIdfCompiler::default();
                if idf.is_available() {
                    logs.push_str("[TOOLCHAIN] Menggunakan ESP-IDF (idf.py)...\n");
                    idf.build(&source)
                } else {
                    logs.push_str(
                        "[TOOLCHAIN] ESP-IDF tidak tersedia, fallback ke compiler bawaan.\n",
                    );
                    compile(&source).map_err(|e| format!("Line {}: {}", e.line, e.message))
                }
            } else {
                let arduino = ArduinoCliCompiler::for_esp32_s3();
                if arduino.is_available() {
                    logs.push_str("[TOOLCHAIN] Menggunakan Arduino CLI (esp32:esp32:esp32s3)...\n");
                    match arduino.compile(&source) {
                        Ok(bin) => {
                            let _ = std::fs::write("firmware.bin", &bin);
                            logs.push_str(
                                "[SUCCESS] Biner 1:1 berhasil dibuat dan disimpan ke ./firmware.bin\n",
                            );
                            Ok(bin)
                        }
                        Err(err) => Err(err),
                    }
                } else {
                    logs.push_str(
                        "[TOOLCHAIN] Arduino CLI tidak tersedia, fallback ke compiler bawaan.\n",
                    );
                    compile(&source).map_err(|e| format!("Line {}: {}", e.line, e.message))
                }
            };

            let _ = tx.send(BuildJobResult {
                target_index: index,
                binary: result,
                build_output: logs,
            });
        });
    }

    #[cfg(target_arch = "wasm32")]
    {
        app.code_editor.build_in_progress = false;
        match compile(&source) {
            Ok(binary) => {
                if let Some((_, board)) = app.esp32_s3_boards.get_mut(index) {
                    let _ = board.load_firmware(&binary);
                    app.code_editor.status_message =
                        format!("Bytecode ({} B) flashed to ESP32.", binary.len());
                }
            }
            Err(err) => {
                app.code_editor.status_message = format!("Line {}: {}", err.line, err.message);
                app.code_editor.status_is_error = true;
            }
        }
    }
}
