//! DHT22 (AM2302) digital relative humidity and temperature sensor component model.

use sim_core::component::Component;
use sim_core::netlist::PinId;

/// Minimum supply voltage required for nominal sensor operation (3.0 V).
pub const DHT22_MIN_OPERATING_VOLTAGE: f32 = 3.0;
/// Maximum safe supply voltage (6.0 V).
pub const DHT22_MAX_OPERATING_VOLTAGE: f32 = 6.0;
/// Minimum rated temperature in degrees Celsius (-40.0 °C).
pub const DHT22_MIN_TEMPERATURE: f32 = -40.0;
/// Maximum rated temperature in degrees Celsius (80.0 °C).
pub const DHT22_MAX_TEMPERATURE: f32 = 80.0;
/// Minimum relative humidity percentage (0.0%).
pub const DHT22_MIN_HUMIDITY: f32 = 0.0;
/// Maximum relative humidity percentage (100.0%).
pub const DHT22_MAX_HUMIDITY: f32 = 100.0;
/// Default factory room temperature in degrees Celsius (25.0 °C).
pub const DHT22_DEFAULT_TEMPERATURE: f32 = 25.0;
/// Default factory ambient relative humidity percentage (50.0%).
pub const DHT22_DEFAULT_HUMIDITY: f32 = 50.0;

/// Operational phase of the DHT22 single-bus 1-wire communication sequence.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum Dht22ProtocolState {
    /// Sensor idle, waiting for host microprocessor to initiate start pulse.
    #[default]
    Idle,
    /// Host driving data line low to request data transmission.
    HostStartLow,
    /// Sensor acknowledging host request by pulling data line low for 80 µs.
    ResponseLow,
    /// Sensor pulling data line high for 80 µs before transmitting data bits.
    ResponseHigh,
    /// Sensor transmitting bit preamble by pulling data line low for 50 µs.
    BitLow,
    /// Sensor transmitting high pulse duration encoding bit 0 or bit 1.
    BitHigh,
    /// Sensor signaling end of transmission by pulling data line low for 50 µs.
    EndLow,
}

/// Simulated model of a DHT22 (AM2302) environmental humidity and temperature sensor.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Dht22 {
    name: String,
    vcc: PinId,
    data: PinId,
    nc: PinId,
    gnd: PinId,
    temperature: f32,
    humidity: f32,
    is_powered: bool,
    protocol_state: Dht22ProtocolState,
    bit_index: usize,
    cached_frame: [u8; 5],
    last_data_high: bool,
    low_start_us: u64,
    state_start_us: u64,
    read_count: u32,
    last_read_us: u64,
    drive_output: Option<bool>,
}

impl Dht22 {
    /// Constructs a new DHT22 sensor with terminal PinIds and default 25 °C, 50% RH readings.
    pub fn new(vcc: PinId, data: PinId, nc: PinId, gnd: PinId) -> Self {
        let frame = encode_dht22_frame(DHT22_DEFAULT_TEMPERATURE, DHT22_DEFAULT_HUMIDITY);
        Self {
            name: "DHT22 / AM2302 Sensor".to_string(),
            vcc,
            data,
            nc,
            gnd,
            temperature: DHT22_DEFAULT_TEMPERATURE,
            humidity: DHT22_DEFAULT_HUMIDITY,
            is_powered: false,
            protocol_state: Dht22ProtocolState::Idle,
            bit_index: 0,
            cached_frame: frame,
            last_data_high: true,
            low_start_us: 0,
            state_start_us: 0,
            read_count: 0,
            last_read_us: 0,
            drive_output: None,
        }
    }

    /// Power supply positive terminal PinId (Pin 1).
    pub fn vcc(&self) -> PinId {
        self.vcc
    }

    /// Bidirectional single-bus data communication terminal PinId (Pin 2).
    pub fn data(&self) -> PinId {
        self.data
    }

    /// No connection terminal PinId (Pin 3).
    pub fn nc(&self) -> PinId {
        self.nc
    }

    /// Ground terminal PinId (Pin 4).
    pub fn gnd(&self) -> PinId {
        self.gnd
    }

    /// Current simulated ambient temperature in degrees Celsius.
    pub fn temperature(&self) -> f32 {
        self.temperature
    }

    /// Commanded ambient temperature clamped to [-40.0 °C, 80.0 °C].
    pub fn set_temperature(&mut self, temp: f32) {
        self.temperature = temp.clamp(DHT22_MIN_TEMPERATURE, DHT22_MAX_TEMPERATURE);
        self.cached_frame = encode_dht22_frame(self.temperature, self.humidity);
    }

    /// Current simulated relative humidity percentage.
    pub fn humidity(&self) -> f32 {
        self.humidity
    }

    /// Commanded relative humidity percentage clamped to [0.0%, 100.0%].
    pub fn set_humidity(&mut self, hum: f32) {
        self.humidity = hum.clamp(DHT22_MIN_HUMIDITY, DHT22_MAX_HUMIDITY);
        self.cached_frame = encode_dht22_frame(self.temperature, self.humidity);
    }

    /// Returns true if adequate operating potential exists across power terminals.
    pub fn is_powered(&self) -> bool {
        self.is_powered
    }

    /// Current communication protocol state machine stage.
    pub fn protocol_state(&self) -> Dht22ProtocolState {
        self.protocol_state
    }

    /// Cumulative count of completed read request transactions.
    pub fn read_count(&self) -> u32 {
        self.read_count
    }

    /// Timestamp in microseconds of the most recent read query.
    pub fn last_read_us(&self) -> u64 {
        self.last_read_us
    }

    /// Returns the active digital output drive level if the sensor is driving the bus.
    pub fn drive_output(&self) -> Option<bool> {
        self.drive_output
    }

    /// The 5-byte encoded data frame (16-bit RH, 16-bit Temp, 8-bit Checksum).
    pub fn raw_frame(&self) -> [u8; 5] {
        self.cached_frame
    }

    /// Verifies whether the cached frame checksum matches the sum of data bytes.
    pub fn checksum_valid(&self) -> bool {
        let sum = self.cached_frame[0]
            .wrapping_add(self.cached_frame[1])
            .wrapping_add(self.cached_frame[2])
            .wrapping_add(self.cached_frame[3]);
        sum == self.cached_frame[4]
    }

    /// Updates internal electrical state, detects bus edges, and steps protocol timers.
    pub fn update_electrical(&mut self, v_vcc: f32, v_gnd: f32, data_high: bool, time_us: u64) {
        let supply = v_vcc - v_gnd;
        self.is_powered = supply >= DHT22_MIN_OPERATING_VOLTAGE;

        if !self.is_powered {
            self.protocol_state = Dht22ProtocolState::Idle;
            self.drive_output = None;
            self.last_data_high = data_high;
            return;
        }

        self.step_protocol(data_high, time_us);
        self.last_data_high = data_high;
    }

    /// Advances the single-bus protocol state machine according to edge detection and elapsed time.
    fn step_protocol(&mut self, data_high: bool, time_us: u64) {
        match self.protocol_state {
            Dht22ProtocolState::Idle => {
                self.drive_output = None;
                if !data_high && self.last_data_high {
                    self.low_start_us = time_us;
                    self.protocol_state = Dht22ProtocolState::HostStartLow;
                }
            }
            Dht22ProtocolState::HostStartLow => {
                if data_high && !self.last_data_high {
                    let duration = time_us.saturating_sub(self.low_start_us);
                    if duration >= 800 {
                        self.cached_frame = encode_dht22_frame(self.temperature, self.humidity);
                        self.protocol_state = Dht22ProtocolState::ResponseLow;
                        self.state_start_us = time_us;
                        self.drive_output = Some(false);
                        self.read_count += 1;
                        self.last_read_us = time_us;
                    } else {
                        self.protocol_state = Dht22ProtocolState::Idle;
                    }
                }
            }
            Dht22ProtocolState::ResponseLow => {
                self.drive_output = Some(false);
                if time_us.saturating_sub(self.state_start_us) >= 80 {
                    self.protocol_state = Dht22ProtocolState::ResponseHigh;
                    self.state_start_us = time_us;
                    self.drive_output = None;
                }
            }
            Dht22ProtocolState::ResponseHigh => {
                self.drive_output = None;
                if time_us.saturating_sub(self.state_start_us) >= 80 {
                    self.protocol_state = Dht22ProtocolState::BitLow;
                    self.bit_index = 0;
                    self.state_start_us = time_us;
                    self.drive_output = Some(false);
                }
            }
            Dht22ProtocolState::BitLow => {
                self.drive_output = Some(false);
                if time_us.saturating_sub(self.state_start_us) >= 50 {
                    self.protocol_state = Dht22ProtocolState::BitHigh;
                    self.state_start_us = time_us;
                    self.drive_output = None;
                }
            }
            Dht22ProtocolState::BitHigh => {
                self.drive_output = None;
                let bit_val = get_frame_bit(&self.cached_frame, self.bit_index);
                let duration = if bit_val { 70 } else { 28 };
                if time_us.saturating_sub(self.state_start_us) >= duration {
                    self.bit_index += 1;
                    if self.bit_index >= 40 {
                        self.protocol_state = Dht22ProtocolState::EndLow;
                        self.state_start_us = time_us;
                        self.drive_output = Some(false);
                    } else {
                        self.protocol_state = Dht22ProtocolState::BitLow;
                        self.state_start_us = time_us;
                        self.drive_output = Some(false);
                    }
                }
            }
            Dht22ProtocolState::EndLow => {
                self.drive_output = Some(false);
                if time_us.saturating_sub(self.state_start_us) >= 50 {
                    self.protocol_state = Dht22ProtocolState::Idle;
                    self.drive_output = None;
                }
            }
        }
    }
}

/// Extracts a specific bit from the 40-bit transmission frame (MSB first).
fn get_frame_bit(frame: &[u8; 5], bit_index: usize) -> bool {
    if bit_index >= 40 {
        return false;
    }
    let byte = frame[bit_index / 8];
    let bit_offset = 7 - (bit_index % 8);
    ((byte >> bit_offset) & 1) != 0
}

/// Encodes ambient temperature and humidity into a 5-byte AM2302 data frame.
pub fn encode_dht22_frame(temperature: f32, humidity: f32) -> [u8; 5] {
    let hum_clamped = humidity.clamp(DHT22_MIN_HUMIDITY, DHT22_MAX_HUMIDITY);
    let hum_raw = (hum_clamped * 10.0).round() as u16;

    let temp_clamped = temperature.clamp(DHT22_MIN_TEMPERATURE, DHT22_MAX_TEMPERATURE);
    let temp_raw = if temp_clamped < 0.0 {
        let magnitude = ((-temp_clamped) * 10.0).round() as u16;
        magnitude | 0x8000
    } else {
        (temp_clamped * 10.0).round() as u16
    };

    let b0 = (hum_raw >> 8) as u8;
    let b1 = (hum_raw & 0xFF) as u8;
    let b2 = (temp_raw >> 8) as u8;
    let b3 = (temp_raw & 0xFF) as u8;
    let checksum = b0.wrapping_add(b1).wrapping_add(b2).wrapping_add(b3);

    [b0, b1, b2, b3, checksum]
}

impl Component for Dht22 {
    fn name(&self) -> &str {
        &self.name
    }

    fn pins(&self) -> Vec<PinId> {
        vec![self.vcc, self.data, self.nc, self.gnd]
    }

    fn update(&mut self) {}
}
