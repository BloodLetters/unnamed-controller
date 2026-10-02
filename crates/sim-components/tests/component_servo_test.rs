use sim_components::component::{
    SERVO_DEFAULT_SPEED_DPS, SERVO_MAX_PULSE_US, SERVO_MIN_PULSE_US, Servo, pulse_width_to_angle,
};
use sim_core::component::Component;
use sim_core::netlist::PinId;

#[test]
fn test_servo_creation_and_pins() {
    let servo = Servo::new(PinId(10), PinId(11), PinId(12));
    assert_eq!(servo.gnd(), PinId(10));
    assert_eq!(servo.vcc(), PinId(11));
    assert_eq!(servo.pwm(), PinId(12));
    assert_eq!(servo.pins(), vec![PinId(10), PinId(11), PinId(12)]);
    assert_eq!(servo.angle(), 90.0);
    assert_eq!(servo.target_angle(), 90.0);
    assert!(!servo.is_powered());
    assert_eq!(servo.speed_deg_per_sec(), SERVO_DEFAULT_SPEED_DPS);
}

#[test]
fn test_servo_power_requirements() {
    let mut servo = Servo::new(PinId(1), PinId(2), PinId(3));
    servo.update_electrical(3.3, 0.0, false, 0, 0.001);
    assert!(!servo.is_powered());

    servo.update_electrical(5.0, 0.0, false, 1000, 0.001);
    assert!(servo.is_powered());

    servo.update_electrical(0.0, 0.0, false, 2000, 0.001);
    assert!(!servo.is_powered());
}

#[test]
fn test_servo_pwm_pulse_to_angle_mapping() {
    assert_eq!(pulse_width_to_angle(SERVO_MIN_PULSE_US), 0.0);
    assert_eq!(pulse_width_to_angle(1500), 90.0);
    assert_eq!(pulse_width_to_angle(SERVO_MAX_PULSE_US), 180.0);
    assert_eq!(pulse_width_to_angle(2500), 180.0);
    assert_eq!(pulse_width_to_angle(500), 0.0);
}

#[test]
fn test_servo_pwm_pulse_reception() {
    let mut servo = Servo::new(PinId(1), PinId(2), PinId(3));
    servo.update_electrical(5.0, 0.0, false, 0, 0.001);

    servo.update_electrical(5.0, 0.0, true, 1000, 0.001);
    servo.update_electrical(5.0, 0.0, false, 2000, 0.001);
    assert_eq!(servo.last_pulse_width_us(), 1000);
    assert_eq!(servo.target_angle(), 0.0);

    servo.update_electrical(5.0, 0.0, true, 21000, 0.001);
    servo.update_electrical(5.0, 0.0, false, 23000, 0.001);
    assert_eq!(servo.last_pulse_width_us(), 2000);
    assert_eq!(servo.target_angle(), 180.0);
}

#[test]
fn test_servo_transit_movement() {
    let mut servo = Servo::new(PinId(1), PinId(2), PinId(3));
    servo.set_angle(0.0);
    servo.set_target_angle(180.0);
    servo.update_electrical(5.0, 0.0, false, 0, 0.1);
    assert!((servo.angle() - 60.0).abs() < 1.0);

    servo.update_electrical(5.0, 0.0, false, 100_000, 0.3);
    assert_eq!(servo.angle(), 180.0);
}

#[test]
fn test_servo_unpowered_transit_lock() {
    let mut servo = Servo::new(PinId(1), PinId(2), PinId(3));
    servo.set_angle(45.0);
    servo.set_target_angle(150.0);
    servo.update_electrical(0.0, 0.0, false, 0, 1.0);
    assert_eq!(servo.angle(), 45.0);
}

#[test]
fn test_servo_angle_clamping() {
    let mut servo = Servo::new(PinId(1), PinId(2), PinId(3));
    servo.set_angle(-20.0);
    assert_eq!(servo.angle(), 0.0);
    servo.set_angle(250.0);
    assert_eq!(servo.angle(), 180.0);
    servo.set_target_angle(-50.0);
    assert_eq!(servo.target_angle(), 0.0);
    servo.set_target_angle(300.0);
    assert_eq!(servo.target_angle(), 180.0);
}
