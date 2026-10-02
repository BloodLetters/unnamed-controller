//! Background job dispatcher and receiver for Arduino Library Manager.

use crate::app::SimulatorApp;
use crate::editor::types::LibraryJobResult;
use crate::toolchain::ArduinoCliCompiler;

/// Polls background library tasks and updates state when jobs complete.
pub fn poll_background_library(app: &mut SimulatorApp, ctx: &egui::Context) {
    #[cfg(not(target_arch = "wasm32"))]
    if let Some(rx) = &app.library_receiver {
        match rx.try_recv() {
            Ok(result) => {
                app.code_editor.library.is_busy = false;
                match result {
                    LibraryJobResult::InstallDone {
                        name,
                        result,
                        installed,
                        updatable,
                    } => {
                        app.code_editor.library.installed = installed;
                        app.code_editor.library.updatable = updatable;
                        match result {
                            Ok(_) => {
                                app.code_editor.library.status = format!(
                                    "Library '{}' berhasil di-download dan terpasang di sketchbook.",
                                    name
                                );
                                app.code_editor.library.status_is_error = false;
                            }
                            Err(err) => {
                                app.code_editor.library.status =
                                    format!("Gagal install '{}': {}", name, err);
                                app.code_editor.library.status_is_error = true;
                            }
                        }
                    }
                    LibraryJobResult::SearchDone { results } => match results {
                        Ok(list) => {
                            app.code_editor.library.results = list;
                            app.code_editor.library.status = "Pencarian selesai.".to_string();
                            app.code_editor.library.status_is_error = false;
                        }
                        Err(err) => {
                            app.code_editor.library.status = err;
                            app.code_editor.library.status_is_error = true;
                        }
                    },
                    LibraryJobResult::UpdateIndexDone {
                        result,
                        installed,
                        updatable,
                    } => {
                        app.code_editor.library.installed = installed;
                        app.code_editor.library.updatable = updatable;
                        match result {
                            Ok(_) => {
                                app.code_editor.library.status =
                                    "Index library berhasil diperbarui.".to_string();
                                app.code_editor.library.status_is_error = false;
                            }
                            Err(err) => {
                                app.code_editor.library.status =
                                    format!("Gagal update index: {}", err);
                                app.code_editor.library.status_is_error = true;
                            }
                        }
                    }
                    LibraryJobResult::RefreshDone {
                        installed,
                        updatable,
                    } => {
                        app.code_editor.library.installed = installed;
                        app.code_editor.library.updatable = updatable;
                    }
                }
                app.library_receiver = None;
                ctx.request_repaint();
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                ctx.request_repaint();
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                app.code_editor.library.is_busy = false;
                app.library_receiver = None;
            }
        }
    }
}

/// Spawns a background thread to download and install a library.
pub fn start_background_install(app: &mut SimulatorApp, name: String) {
    if app.code_editor.library.is_busy {
        return;
    }
    app.code_editor.library.is_busy = true;
    app.code_editor.library.busy_text = format!("Mendownload dan memasang '{}'...", name);
    app.code_editor.library.status = format!("Sedang mendownload '{}' di background...", name);
    app.code_editor.library.status_is_error = false;

    #[cfg(not(target_arch = "wasm32"))]
    {
        let (tx, rx) = std::sync::mpsc::channel();
        app.library_receiver = Some(rx);
        let lib_name = name;

        std::thread::spawn(move || {
            let compiler = ArduinoCliCompiler::default();
            let result = compiler.install_library(&lib_name, None);
            let installed = compiler.installed_libraries().unwrap_or_default();
            let updatable = compiler.updatable_libraries().unwrap_or_default();
            let _ = tx.send(LibraryJobResult::InstallDone {
                name: lib_name,
                result,
                installed,
                updatable,
            });
        });
    }
}

/// Spawns a background thread to search libraries.
pub fn start_background_search(app: &mut SimulatorApp) {
    if app.code_editor.library.is_busy {
        return;
    }
    let query = app.code_editor.library.query.clone();
    app.code_editor.library.is_busy = true;
    app.code_editor.library.busy_text = format!("Mencari '{}'...", query);
    app.code_editor.library.status = "Sedang mencari library...".to_string();
    app.code_editor.library.status_is_error = false;

    #[cfg(not(target_arch = "wasm32"))]
    {
        let (tx, rx) = std::sync::mpsc::channel();
        app.library_receiver = Some(rx);

        std::thread::spawn(move || {
            let compiler = ArduinoCliCompiler::default();
            let results = compiler.search_libraries(&query);
            let _ = tx.send(LibraryJobResult::SearchDone { results });
        });
    }
}

/// Spawns a background thread to update the library index over the network.
pub fn start_background_update_index(app: &mut SimulatorApp) {
    if app.code_editor.library.is_busy {
        return;
    }
    app.code_editor.library.is_busy = true;
    app.code_editor.library.busy_text = "Memperbarui index library...".to_string();
    app.code_editor.library.status = "Sedang mengunduh index library...".to_string();
    app.code_editor.library.status_is_error = false;

    #[cfg(not(target_arch = "wasm32"))]
    {
        let (tx, rx) = std::sync::mpsc::channel();
        app.library_receiver = Some(rx);

        std::thread::spawn(move || {
            let compiler = ArduinoCliCompiler::default();
            let result = compiler.update_library_index();
            let installed = compiler.installed_libraries().unwrap_or_default();
            let updatable = compiler.updatable_libraries().unwrap_or_default();
            let _ = tx.send(LibraryJobResult::UpdateIndexDone {
                result,
                installed,
                updatable,
            });
        });
    }
}

/// Spawns a background thread to refresh installed and updatable library lists.
pub fn start_background_refresh(app: &mut SimulatorApp) {
    if app.code_editor.library.is_busy {
        return;
    }
    app.code_editor.library.is_busy = true;
    app.code_editor.library.busy_text = "Memuat daftar library...".to_string();

    #[cfg(not(target_arch = "wasm32"))]
    {
        let (tx, rx) = std::sync::mpsc::channel();
        app.library_receiver = Some(rx);

        std::thread::spawn(move || {
            let compiler = ArduinoCliCompiler::default();
            if compiler.is_available() {
                let installed = compiler.installed_libraries().unwrap_or_default();
                let updatable = compiler.updatable_libraries().unwrap_or_default();
                let _ = tx.send(LibraryJobResult::RefreshDone {
                    installed,
                    updatable,
                });
            } else {
                let _ = tx.send(LibraryJobResult::RefreshDone {
                    installed: Vec::new(),
                    updatable: Vec::new(),
                });
            }
        });
    }
}
