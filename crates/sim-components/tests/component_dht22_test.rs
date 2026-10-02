//! Integration tests for DHT22 (AM2302) digital humidity and temperature sensor.

use sim_components::component::{
    DHT22_DEFAULT_HUMIDITY, DHT22_DEFAULT_TEMPERATURE, DHT22_MAX_HUMIDITY, DHT22_MAX_TEMPERATURE,
    DHT22_MIN_HUMIDITY, DHT22_MIN_TEMPERATURE, Dht22, Dht22ProtocolState, encode_dht22_frame,
};
use sim_core::component::Component;
use sim_core::netlist::PinId;

/// Verifies pin assignments, initial default readings, and component trait queries.
#[test]
fn test_dht22_creation_and_pins() {
    let dht = Dht22::new(PinId(10), PinId(11), PinId(12), PinId(13));
    assert_eq!(dht.vcc(), PinId(10));
    assert_eq!(dht.data(), PinId(11));
    assert_eq!(dht.nc(), PinId(12));
    assert_eq!(dht.gnd(), PinId(13));
    assert_eq!(dht.name(), "DHT22 / AM2302 Sensor");
    assert_eq!(dht.pins(), vec![PinId(10), PinId(11), PinId(12), PinId(13)]);
    assert_eq!(dht.temperature(), DHT22_DEFAULT_TEMPERATURE);
    assert_eq!(dht.humidity(), DHT22_DEFAULT_HUMIDITY);
    assert!(!dht.is_powered());
    assert_eq!(dht.protocol_state(), Dht22ProtocolState::Idle);
    assert_eq!(dht.read_count(), 0);
    assert!(dht.checksum_valid());
}

/// Verifies that temperature and humidity settings clamp to rated sensor operating limits.
#[test]
fn test_dht22_temperature_and_humidity_clamping() {
    let mut dht = Dht22::new(PinId(1), PinId(2), PinId(3), PinId(4));

    dht.set_temperature(120.0);
    assert_eq!(dht.temperature(), DHT22_MAX_TEMPERATURE);

    dht.set_temperature(-70.0);
    assert_eq!(dht.temperature(), DHT22_MIN_TEMPERATURE);

    dht.set_humidity(150.0);
    assert_eq!(dht.humidity(), DHT22_MAX_HUMIDITY);

    dht.set_humidity(-10.0);
    assert_eq!(dht.humidity(), DHT22_MIN_HUMIDITY);

    dht.set_temperature(23.4);
    dht.set_humidity(56.7);
    assert!((dht.temperature() - 23.4).abs() < 0.01);
    assert!((dht.humidity() - 56.7).abs() < 0.01);
    assert!(dht.checksum_valid());
}

/// Verifies accurate 40-bit frame serialization and checksum generation matching AM2302 specification.
#[test]
fn test_dht22_frame_encoding_positive_and_negative() {
    let frame_pos = encode_dht22_frame(26.5, 65.2);
    assert_eq!(frame_pos[0], 0x02);
    assert_eq!(frame_pos[1], 0x8C);
    assert_eq!(frame_pos[2], 0x01);
    assert_eq!(frame_pos[3], 0x09);
    assert_eq!(frame_pos[4], 0x98);

    let frame_neg = encode_dht22_frame(-10.1, 80.0);
    assert_eq!(frame_neg[0], 0x03);
    assert_eq!(frame_neg[1], 0x20);
    assert_eq!(frame_neg[2], 0x80);
    assert_eq!(frame_neg[3], 0x65);
    let expected_checksum = (0x03u16 + 0x20 + 0x80 + 0x65) as u8;
    assert_eq!(frame_neg[4], expected_checksum);
}

/// Verifies that adequate potential between VCC and GND is required for operation.
#[test]
fn test_dht22_power_requirements() {
    let mut dht = Dht22::new(PinId(1), PinId(2), PinId(3), PinId(4));

    dht.update_electrical(2.5, 0.0, true, 100);
    assert!(!dht.is_powered());

    dht.update_electrical(3.3, 0.0, true, 200);
    assert!(dht.is_powered());

    dht.update_electrical(5.0, 0.0, true, 300);
    assert!(dht.is_powered());

    dht.update_electrical(0.0, 0.0, true, 400);
    assert!(!dht.is_powered());
}

/// Verifies that brief noise spikes below 800 microseconds do not trigger sensor response.
#[test]
fn test_dht22_short_host_pulse_ignored() {
    let mut dht = Dht22::new(PinId(1), PinId(2), PinId(3), PinId(4));
    dht.update_electrical(5.0, 0.0, true, 0);

    dht.update_electrical(5.0, 0.0, false, 1000);
    assert_eq!(dht.protocol_state(), Dht22ProtocolState::HostStartLow);

    dht.update_electrical(5.0, 0.0, true, 1500);
    assert_eq!(dht.protocol_state(), Dht22ProtocolState::Idle);
    assert_eq!(dht.read_count(), 0);
    assert_eq!(dht.drive_output(), None);
}

/// Verifies the complete single-bus handshake and full 40-bit frame transmission.
#[test]
fn test_dht22_full_communication_handshake() {
    let mut dht = Dht22::new(PinId(1), PinId(2), PinId(3), PinId(4));
    dht.set_temperature(26.5);
    dht.set_humidity(65.2);

    dht.update_electrical(5.0, 0.0, true, 0);
    assert!(dht.is_powered());

    dht.update_electrical(5.0, 0.0, false, 10_000);
    assert_eq!(dht.protocol_state(), Dht22ProtocolState::HostStartLow);

    dht.update_electrical(5.0, 0.0, true, 20_000);
    assert_eq!(dht.protocol_state(), Dht22ProtocolState::ResponseLow);
    assert_eq!(dht.read_count(), 1);
    assert_eq!(dht.drive_output(), Some(false));

    dht.update_electrical(5.0, 0.0, false, 20_085);
    assert_eq!(dht.protocol_state(), Dht22ProtocolState::ResponseHigh);
    assert_eq!(dht.drive_output(), None);

    dht.update_electrical(5.0, 0.0, true, 20_170);
    assert_eq!(dht.protocol_state(), Dht22ProtocolState::BitLow);
    assert_eq!(dht.drive_output(), Some(false));

    let mut current_us = 20_170_u64;
    for _ in 0..40 {
        current_us += 55;
        dht.update_electrical(5.0, 0.0, true, current_us);
        assert_eq!(dht.protocol_state(), Dht22ProtocolState::BitHigh);

        current_us += 75;
        dht.update_electrical(5.0, 0.0, true, current_us);
    }

    assert_eq!(dht.protocol_state(), Dht22ProtocolState::EndLow);
    assert_eq!(dht.drive_output(), Some(false));

    current_us += 55;
    dht.update_electrical(5.0, 0.0, true, current_us);
    assert_eq!(dht.protocol_state(), Dht22ProtocolState::Idle);
    assert_eq!(dht.drive_output(), None);
}
