pub mod sampling;
pub mod types;
pub mod view_dmm;
pub mod view_logic;
pub mod view_oscilloscope;
pub mod view_pwm;

pub use sampling::sample_instruments;
pub use types::{InstrumentTab, InstrumentsBenchState};

use egui::{RichText, Ui, Vec2};
use view_dmm::render_dmm_view;
use view_logic::render_logic_analyzer_view;
use view_oscilloscope::render_oscilloscope_view;
use view_pwm::render_pwm_analyzer_view;

use crate::app::SimulatorApp;

/// Renders the floating Virtual Test Instruments Workbench window.
pub fn render_instruments_bench(ctx: &egui::Context, app: &mut SimulatorApp) {
    if !app.instruments.open {
        return;
    }

    let mut open = app.instruments.open;
    egui::Window::new("🔬 Virtual Test Instruments Workbench")
        .open(&mut open)
        .default_size(Vec2::new(760.0, 480.0))
        .min_size(Vec2::new(540.0, 360.0))
        .show(ctx, |ui| {
            render_tab_selector(ui, app);
            ui.separator();

            match app.instruments.active_tab {
                InstrumentTab::Oscilloscope => render_oscilloscope_view(ui, app),
                InstrumentTab::LogicAnalyzer => render_logic_analyzer_view(ui, app),
                InstrumentTab::Multimeter => render_dmm_view(ui, app),
                InstrumentTab::FrequencyCounter => render_pwm_analyzer_view(ui, app),
            }
        });

    app.instruments.open = open;
}

/// Renders the navigation tab bar across instruments.
fn render_tab_selector(ui: &mut Ui, app: &mut SimulatorApp) {
    ui.horizontal(|ui| {
        let active = app.instruments.active_tab;

        if ui
            .selectable_label(
                active == InstrumentTab::Oscilloscope,
                RichText::new("📈 2-Ch Oscilloscope").strong(),
            )
            .clicked()
        {
            app.instruments.active_tab = InstrumentTab::Oscilloscope;
        }

        if ui
            .selectable_label(
                active == InstrumentTab::LogicAnalyzer,
                RichText::new("📊 8-Ch Logic Analyzer").strong(),
            )
            .clicked()
        {
            app.instruments.active_tab = InstrumentTab::LogicAnalyzer;
        }

        if ui
            .selectable_label(
                active == InstrumentTab::Multimeter,
                RichText::new("📟 Digital Multimeter").strong(),
            )
            .clicked()
        {
            app.instruments.active_tab = InstrumentTab::Multimeter;
        }

        if ui
            .selectable_label(
                active == InstrumentTab::FrequencyCounter,
                RichText::new("⏱ Frequency / PWM").strong(),
            )
            .clicked()
        {
            app.instruments.active_tab = InstrumentTab::FrequencyCounter;
        }
    });
}
