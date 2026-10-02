use sim_core::instruments::{DigitalMultimeter, LogicAnalyzer, PwmAnalyzer, VirtualOscilloscope};

/// Active view tab in the virtual instruments workbench window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum InstrumentTab {
    Oscilloscope,
    LogicAnalyzer,
    Multimeter,
    FrequencyCounter,
}

/// State container for all connected virtual test instruments and their probes.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InstrumentsBenchState {
    pub open: bool,
    pub active_tab: InstrumentTab,
    pub oscilloscope: VirtualOscilloscope,
    pub logic_analyzer: LogicAnalyzer,
    pub dmm: DigitalMultimeter,
    pub pwm: PwmAnalyzer,
    pub sim_time_us: u64,
}

impl Default for InstrumentsBenchState {
    fn default() -> Self {
        Self {
            open: false,
            active_tab: InstrumentTab::Oscilloscope,
            oscilloscope: VirtualOscilloscope::default(),
            logic_analyzer: LogicAnalyzer::new(),
            dmm: DigitalMultimeter::default(),
            pwm: PwmAnalyzer::new(),
            sim_time_us: 0,
        }
    }
}
