//! SG90 9g Micro Servo actuator electronic component model.

use sim_core::component::Component;
use sim_core::netlist::PinId;

/// Standard minimum pulse duration corresponding to 0 degrees in microseconds.
pub const SERVO_MIN_PULSE_US: u32 = 1000;
/// Standard maximum pulse duration corresponding to 180 degrees in microseconds.
pub const SERVO_MAX_PULSE_US: u32 = 2000;
/// Minimum operating voltage difference required across VCC and GND terminals.
pub const SERVO_MIN_OPERATING_VOLTAGE: f32 = 3.8;
/// Typical transit speed of SG90 servo in degrees per second (~0.1s / 60 degrees).
pub const SERVO_DEFAULT_SPEED_DPS: f32 = 600.0;

/// Simulated model of an SG90 9g micro positional servo motor.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Servo {
    name: String,
    gnd: PinId,
    vcc: PinId,
    pwm: PinId,
    angle: f32,
    target_angle: f32,
    speed_deg_per_sec: f32,
    is_powered: bool,
    last_pwm_high: bool,
    pulse_start_us: u64,
    last_pulse_width_us: u32,
}

impl Servo {
    /// Constructs a new SG90 servo with terminal PinIds and neutral 90 degree position.
    pub fn new(gnd: PinId, vcc: PinId, pwm: PinId) -> Self {
        Self {
            name: "SG90 Micro Servo".to_string(),
            gnd,
            vcc,
            pwm,
            angle: 90.0,
            target_angle: 90.0,
            speed_deg_per_sec: SERVO_DEFAULT_SPEED_DPS,
            is_powered: false,
            last_pwm_high: false,
            pulse_start_us: 0,
            last_pulse_width_us: 1500,
        }
    }

    /// Ground terminal PinId (brown lead).
    pub fn gnd(&self) -> PinId {
        self.gnd
    }

    /// Power supply positive terminal PinId (red lead).
    pub fn vcc(&self) -> PinId {
        self.vcc
    }

    /// PWM signal input terminal PinId (orange lead).
    pub fn pwm(&self) -> PinId {
        self.pwm
    }

    /// Current mechanical angle of the output horn in degrees (0.0 to 180.0).
    pub fn angle(&self) -> f32 {
        self.angle
    }

    /// Target angle commanded by the PWM pulse width or controller in degrees.
    pub fn target_angle(&self) -> f32 {
        self.target_angle
    }

    /// Duration of the most recently measured high PWM pulse in microseconds.
    pub fn last_pulse_width_us(&self) -> u32 {
        self.last_pulse_width_us
    }

    /// Returns true if adequate operating potential exists across power terminals.
    pub fn is_powered(&self) -> bool {
        self.is_powered
    }

    /// Commanded transit speed of the servo horn in degrees per second.
    pub fn speed_deg_per_sec(&self) -> f32 {
        self.speed_deg_per_sec
    }

    /// Sets the transit speed of the servo horn in degrees per second.
    pub fn set_speed_deg_per_sec(&mut self, speed: f32) {
        self.speed_deg_per_sec = speed.max(10.0);
    }

    /// Sets the target angle in degrees, clamped to valid mechanical range [0.0, 180.0].
    pub fn set_target_angle(&mut self, angle: f32) {
        self.target_angle = angle.clamp(0.0, 180.0);
    }

    /// Immediately overrides both physical angle and target angle to the given position.
    pub fn set_angle(&mut self, angle: f32) {
        let clamped = angle.clamp(0.0, 180.0);
        self.angle = clamped;
        self.target_angle = clamped;
    }

    /// Updates internal electrical state, pulse measurement, and motor transit.
    pub fn update_electrical(
        &mut self,
        v_vcc: f32,
        v_gnd: f32,
        pwm_high: bool,
        time_us: u64,
        dt_seconds: f32,
    ) {
        let v_supply = v_vcc - v_gnd;
        self.is_powered = v_supply >= SERVO_MIN_OPERATING_VOLTAGE;

        self.process_pwm_transitions(pwm_high, time_us);
        self.advance_horn_transit(dt_seconds);
    }

    /// Detects rising and falling edges on the control pin to compute high pulse duration.
    fn process_pwm_transitions(&mut self, pwm_high: bool, time_us: u64) {
        if pwm_high && !self.last_pwm_high {
            self.pulse_start_us = time_us;
        } else if !pwm_high && self.last_pwm_high {
            let width = time_us.saturating_sub(self.pulse_start_us) as u32;
            self.last_pulse_width_us = width;
            if self.is_powered && (400..=2600).contains(&width) {
                self.target_angle = pulse_width_to_angle(width);
            }
        }
        self.last_pwm_high = pwm_high;
    }

    /// Smoothly steps physical angle toward target angle when powered.
    fn advance_horn_transit(&mut self, dt_seconds: f32) {
        if !self.is_powered {
            return;
        }
        let diff = self.target_angle - self.angle;
        let max_delta = self.speed_deg_per_sec * dt_seconds;
        if diff.abs() <= max_delta {
            self.angle = self.target_angle;
        } else {
            self.angle += max_delta * diff.signum();
        }
    }
}

/// Converts a measured PWM pulse width in microseconds to an angle between 0.0 and 180.0.
pub fn pulse_width_to_angle(pulse_us: u32) -> f32 {
    let min_p = SERVO_MIN_PULSE_US as f32;
    let max_p = SERVO_MAX_PULSE_US as f32;
    let pulse_f = pulse_us as f32;

    if (SERVO_MIN_PULSE_US..=SERVO_MAX_PULSE_US).contains(&pulse_us) {
        ((pulse_f - min_p) / (max_p - min_p) * 180.0).clamp(0.0, 180.0)
    } else if (500..SERVO_MIN_PULSE_US).contains(&pulse_us) {
        ((pulse_f - 500.0) / 1000.0 * 90.0).clamp(0.0, 90.0)
    } else {
        (90.0 + (pulse_f - 1500.0) / 1000.0 * 90.0).clamp(0.0, 180.0)
    }
}

impl Component for Servo {
    fn name(&self) -> &str {
        &self.name
    }

    fn pins(&self) -> Vec<PinId> {
        vec![self.gnd, self.vcc, self.pwm]
    }

    fn update(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_servo_initial_state() {
        let servo = Servo::new(PinId(1), PinId(2), PinId(3));
        assert_eq!(servo.gnd(), PinId(1));
        assert_eq!(servo.vcc(), PinId(2));
        assert_eq!(servo.pwm(), PinId(3));
        assert_eq!(servo.angle(), 90.0);
        assert_eq!(servo.target_angle(), 90.0);
        assert!(!servo.is_powered());
    }

    #[test]
    fn test_pulse_width_to_angle_mapping() {
        assert!((pulse_width_to_angle(1000) - 0.0).abs() < 0.1);
        assert!((pulse_width_to_angle(1500) - 90.0).abs() < 0.1);
        assert!((pulse_width_to_angle(2000) - 180.0).abs() < 0.1);
    }

    #[test]
    fn test_electrical_power_and_pulse_update() {
        let mut servo = Servo::new(PinId(1), PinId(2), PinId(3));
        servo.update_electrical(5.0, 0.0, true, 0, 0.001);
        assert!(servo.is_powered());

        servo.update_electrical(5.0, 0.0, false, 1000, 0.001);
        assert_eq!(servo.last_pulse_width_us(), 1000);
        assert!((servo.target_angle() - 0.0).abs() < 0.1);

        servo.update_electrical(5.0, 0.0, false, 2000, 0.5);
        assert_eq!(servo.angle(), 0.0);
    }
}
