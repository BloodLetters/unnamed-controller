//! MQ-135 semiconductor gas sensor component model.

use sim_core::component::Component;
use sim_core::netlist::PinId;

/// Minimum supply voltage for sensor heater operation (2.5 V).
pub const MQ135_MIN_OPERATING_VOLTAGE: f32 = 2.5;
/// Maximum safe supply voltage (5.0 V).
pub const MQ135_MAX_OPERATING_VOLTAGE: f32 = 5.0;
/// Minimum detectable gas concentration in parts per million (10 ppm).
pub const MQ135_MIN_PPM: f32 = 10.0;
/// Maximum detectable gas concentration in parts per million (1000 ppm).
pub const MQ135_MAX_PPM: f32 = 1000.0;
/// Default simulated ambient gas concentration in clean air (400 ppm CO2 equivalent).
pub const MQ135_DEFAULT_PPM: f32 = 400.0;
/// Default digital output threshold in parts per million (500 ppm).
pub const MQ135_DEFAULT_THRESHOLD_PPM: f32 = 500.0;

/// Simulated model of an MQ-135 air quality gas sensor module.
///
/// The module exposes four terminals: VCC (5 V supply), GND, AOUT (analog voltage
/// proportional to gas concentration), and DOUT (digital TTL output driven LOW when
/// concentration exceeds the adjustable potentiometer threshold).
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Mq135 {
    name: String,
    vcc: PinId,
    gnd: PinId,
    aout: PinId,
    dout: PinId,
    ppm: f32,
    threshold_ppm: f32,
    is_powered: bool,
}

impl Mq135 {
    /// Constructs a new MQ-135 sensor with default clean-air 400 ppm reading.
    pub fn new(vcc: PinId, gnd: PinId, aout: PinId, dout: PinId) -> Self {
        Self {
            name: "MQ-135 Gas Sensor".to_string(),
            vcc,
            gnd,
            aout,
            dout,
            ppm: MQ135_DEFAULT_PPM,
            threshold_ppm: MQ135_DEFAULT_THRESHOLD_PPM,
            is_powered: false,
        }
    }

    /// Power supply positive terminal PinId (VCC).
    pub fn vcc(&self) -> PinId {
        self.vcc
    }

    /// Ground reference terminal PinId (GND).
    pub fn gnd(&self) -> PinId {
        self.gnd
    }

    /// Analog output terminal PinId (AOUT).
    pub fn aout(&self) -> PinId {
        self.aout
    }

    /// Digital threshold output terminal PinId (DOUT).
    pub fn dout(&self) -> PinId {
        self.dout
    }

    /// Current simulated gas concentration in parts per million.
    pub fn ppm(&self) -> f32 {
        self.ppm
    }

    /// Commanded gas concentration clamped to [10.0, 1000.0] ppm.
    pub fn set_ppm(&mut self, ppm: f32) {
        self.ppm = ppm.clamp(MQ135_MIN_PPM, MQ135_MAX_PPM);
    }

    /// Current digital output trigger threshold in parts per million.
    pub fn threshold_ppm(&self) -> f32 {
        self.threshold_ppm
    }

    /// Commanded digital threshold clamped to the sensor detection range.
    pub fn set_threshold_ppm(&mut self, threshold: f32) {
        self.threshold_ppm = threshold.clamp(MQ135_MIN_PPM, MQ135_MAX_PPM);
    }

    /// Returns true if adequate supply voltage is present across VCC and GND.
    pub fn is_powered(&self) -> bool {
        self.is_powered
    }

    /// Analog output voltage proportional to gas concentration (0.0-5.0 V).
    ///
    /// Voltage scales linearly from 0.1 V at minimum ppm to 4.9 V at maximum ppm,
    /// mirroring the MQ sensor characteristic curve approximation used in simulation.
    pub fn analog_output_voltage(&self) -> f32 {
        if !self.is_powered {
            return 0.0;
        }
        let normalized = (self.ppm - MQ135_MIN_PPM) / (MQ135_MAX_PPM - MQ135_MIN_PPM);
        0.1 + normalized * 4.8
    }

    /// Returns true when DOUT is driven LOW (gas concentration exceeds threshold).
    pub fn digital_output_active(&self) -> bool {
        self.is_powered && self.ppm >= self.threshold_ppm
    }

    /// Updates internal power state from supply terminal voltages.
    pub fn update_electrical(&mut self, v_vcc: f32, v_gnd: f32) {
        let supply = v_vcc - v_gnd;
        self.is_powered = supply >= MQ135_MIN_OPERATING_VOLTAGE;
    }
}

impl Component for Mq135 {
    fn name(&self) -> &str {
        &self.name
    }

    fn pins(&self) -> Vec<PinId> {
        vec![self.vcc, self.gnd, self.aout, self.dout]
    }

    fn update(&mut self) {}
}
