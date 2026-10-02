pub mod dmm;
pub mod frequency;
pub mod logic_analyzer;
pub mod oscilloscope;
pub mod trace;

pub use dmm::{DigitalMultimeter, DmmMode};
pub use frequency::PwmAnalyzer;
pub use logic_analyzer::{LogicAnalyzer, LogicChannel};
pub use oscilloscope::{
    OscilloscopeChannel, TriggerEdge, TriggerMode, TriggerSource, VirtualOscilloscope,
};
pub use trace::{SignalSample, TraceBuffer};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_buffer_peak_and_rms() {
        let mut buf = TraceBuffer::new(10);
        buf.push(0, 0.0, false);
        buf.push(10, 5.0, true);
        buf.push(20, 2.5, true);

        assert_eq!(buf.peak_to_peak(), 5.0);
        assert!(buf.rms_voltage() > 0.0);
        assert_eq!(buf.latest().unwrap().voltage, 2.5);
    }

    #[test]
    fn test_logic_analyzer_vcd_export() {
        let mut la = LogicAnalyzer::new();
        la.sample(0, &[false, false, false, false, false, false, false, false]);
        la.sample(
            100,
            &[true, false, false, false, false, false, false, false],
        );
        la.sample(
            200,
            &[false, true, false, false, false, false, false, false],
        );

        let vcd = la.export_vcd();
        assert!(vcd.contains("$version Unnamed Controller Logic Analyzer $end"));
        assert!(vcd.contains("#100"));
        assert!(vcd.contains("1!"));
        assert!(vcd.contains("#200"));
    }

    #[test]
    fn test_pwm_analyzer_frequency_and_duty() {
        let mut pwm = PwmAnalyzer::new();
        pwm.update(0, true);
        pwm.update(500, false);
        pwm.update(1000, true);

        assert_eq!(pwm.frequency_hz, 1000.0);
        assert_eq!(pwm.duty_percent, 50.0);
    }

    #[test]
    fn test_dmm_continuity_and_voltage() {
        let mut dmm = DigitalMultimeter::default();
        dmm.update(5.0, 0.0, false, None, None);
        assert_eq!(dmm.reading, 5.0);

        dmm.mode = DmmMode::Continuity;
        dmm.update(0.0, 0.01, true, None, None);
        assert!(dmm.continuity_active);
        assert_eq!(dmm.display_string(), "BEEP (0.0 Ω)");
    }
}
