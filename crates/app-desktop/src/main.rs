#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // Hide console window on release

use eframe::egui;

/// Application entrypoint
fn main() -> eframe::Result<()> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0])
            .with_min_inner_size([600.0, 400.0])
            .with_title("Unnamed Controller Simulator (Native)"),
        ..Default::default()
    };

    eframe::run_native(
        "Unnamed Controller Simulator",
        options,
        Box::new(|cc| Box::new(sim_ui::SimulatorApp::new(cc))),
    )
}
