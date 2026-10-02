//! Runtime instance variant and behavior delegation for placed components.

use egui::{Painter, Pos2, Ui, Vec2};
use sim_components::component::{Buzzer, Dht22, Lcd1602, Led, Mq135, Servo, Ssd1306};
use sim_core::component::Component;
use sim_core::engine::Engine;
use sim_core::netlist::PinId;

/// Supported discrete electronic component variants placed on the schematic canvas.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ComponentInstance {
    Led(Led),
    Ssd1306(Ssd1306),
    Lcd1602(Lcd1602),
    Servo(Servo),
    Dht22(Dht22),
    Mq135(Mq135),
    Buzzer(Buzzer),
}

impl ComponentInstance {
    /// Human-readable display name of the component type.
    pub fn name(&self) -> &str {
        match self {
            Self::Led(_) => "5mm Discrete LED",
            Self::Ssd1306(_) => "SSD1306 OLED Display",
            Self::Lcd1602(_) => "HD44780 1602 LCD",
            Self::Servo(_) => "SG90 Micro Servo",
            Self::Dht22(_) => "DHT22 / AM2302 Sensor",
            Self::Mq135(_) => "MQ-135 Gas Sensor",
            Self::Buzzer(_) => "Active Buzzer",
        }
    }

    /// List of all terminal pins owned by this component.
    pub fn pins(&self) -> Vec<PinId> {
        match self {
            Self::Led(c) => c.pins(),
            Self::Ssd1306(c) => c.pins(),
            Self::Lcd1602(c) => vec![c.gnd(), c.vcc(), c.sda(), c.scl()],
            Self::Servo(c) => c.pins(),
            Self::Dht22(c) => c.pins(),
            Self::Mq135(c) => c.pins(),
            Self::Buzzer(c) => c.pins(),
        }
    }

    /// Relative offsets (dx, dy) of each terminal pin from component center.
    pub fn pin_offsets(&self) -> Vec<(PinId, Vec2)> {
        match self {
            Self::Led(c) => {
                let ox = crate::canvas::components::led::LED_PIN_OFFSET_X;
                let oy = crate::canvas::components::led::LED_PIN_OFFSET_Y;
                vec![
                    (c.anode(), Vec2::new(-ox, oy)),
                    (c.cathode(), Vec2::new(ox, oy)),
                ]
            }
            Self::Ssd1306(c) => {
                let offsets = crate::canvas::components::ssd1306::ssd1306_pin_offsets();
                vec![
                    (c.gnd(), Vec2::new(offsets[0].0, offsets[0].1)),
                    (c.vcc(), Vec2::new(offsets[1].0, offsets[1].1)),
                    (c.sda(), Vec2::new(offsets[2].0, offsets[2].1)),
                    (c.scl(), Vec2::new(offsets[3].0, offsets[3].1)),
                ]
            }
            Self::Lcd1602(c) => {
                let offsets = crate::canvas::components::lcd1602::lcd1602_pin_offsets();
                vec![
                    (c.gnd(), Vec2::new(offsets[0].0, offsets[0].1)),
                    (c.vcc(), Vec2::new(offsets[1].0, offsets[1].1)),
                    (c.sda(), Vec2::new(offsets[2].0, offsets[2].1)),
                    (c.scl(), Vec2::new(offsets[3].0, offsets[3].1)),
                ]
            }
            Self::Servo(c) => {
                let offsets = crate::canvas::components::servo::servo_pin_offsets();
                vec![
                    (c.gnd(), Vec2::new(offsets[0].0, offsets[0].1)),
                    (c.vcc(), Vec2::new(offsets[1].0, offsets[1].1)),
                    (c.pwm(), Vec2::new(offsets[2].0, offsets[2].1)),
                ]
            }
            Self::Dht22(c) => {
                let offsets = crate::canvas::components::dht22::dht22_pin_offsets();
                vec![
                    (c.vcc(), Vec2::new(offsets[0].0, offsets[0].1)),
                    (c.data(), Vec2::new(offsets[1].0, offsets[1].1)),
                    (c.nc(), Vec2::new(offsets[2].0, offsets[2].1)),
                    (c.gnd(), Vec2::new(offsets[3].0, offsets[3].1)),
                ]
            }
            Self::Mq135(c) => {
                let offsets = crate::canvas::components::mq135::mq135_pin_offsets();
                vec![
                    (c.vcc(), Vec2::new(offsets[0].0, offsets[0].1)),
                    (c.gnd(), Vec2::new(offsets[1].0, offsets[1].1)),
                    (c.aout(), Vec2::new(offsets[2].0, offsets[2].1)),
                    (c.dout(), Vec2::new(offsets[3].0, offsets[3].1)),
                ]
            }
            Self::Buzzer(c) => {
                let offsets = crate::canvas::components::buzzer::buzzer_pin_offsets();
                vec![
                    (c.vcc(), Vec2::new(offsets[0].0, offsets[0].1)),
                    (c.gnd(), Vec2::new(offsets[1].0, offsets[1].1)),
                ]
            }
        }
    }

    /// Unrotated visual bounding dimensions used for drag selection and hit testing.
    pub fn bounds(&self) -> Vec2 {
        match self {
            Self::Led(_) => Vec2::new(26.0, 36.0),
            Self::Ssd1306(_) => Vec2::new(110.0, 95.0),
            Self::Lcd1602(_) => Vec2::new(220.0, 90.0),
            Self::Servo(_) => Vec2::new(86.0, 56.0),
            Self::Dht22(_) => Vec2::new(52.0, 78.0),
            Self::Mq135(_) => Vec2::new(56.0, 70.0),
            Self::Buzzer(_) => Vec2::new(40.0, 60.0),
        }
    }

    /// Renders the component package, labels, terminals, and interactive feedback.
    pub fn draw(&self, painter: &Painter, center: Pos2, zoom: f32, rot: u16, is_selected: bool) {
        match self {
            Self::Led(c) => {
                crate::canvas::components::led::draw_single_led(
                    painter,
                    center,
                    zoom,
                    rot,
                    c,
                    is_selected,
                );
            }
            Self::Ssd1306(c) => {
                crate::canvas::components::ssd1306::draw_single_ssd1306(
                    painter,
                    center,
                    zoom,
                    rot,
                    c,
                    is_selected,
                );
            }
            Self::Lcd1602(c) => {
                crate::canvas::components::lcd1602::draw_single_lcd1602(
                    painter,
                    center,
                    zoom,
                    rot,
                    c,
                    is_selected,
                );
            }
            Self::Servo(c) => {
                crate::canvas::components::servo::draw_single_servo(
                    painter,
                    center,
                    zoom,
                    rot,
                    c,
                    is_selected,
                );
            }
            Self::Dht22(c) => {
                crate::canvas::components::dht22::draw_single_dht22(
                    painter,
                    center,
                    zoom,
                    rot,
                    c,
                    is_selected,
                );
            }
            Self::Mq135(c) => {
                crate::canvas::components::mq135::draw_single_mq135(
                    painter,
                    center,
                    zoom,
                    rot,
                    c,
                    is_selected,
                );
            }
            Self::Buzzer(c) => {
                crate::canvas::components::buzzer::draw_single_buzzer(
                    painter,
                    center,
                    zoom,
                    rot,
                    c,
                    is_selected,
                );
            }
        }
    }

    /// Evaluates electrical voltages and advances internal state for this component.
    pub fn update_electrical(&mut self, engine: &Engine, sim_time_us: u64, dt_seconds: f32) {
        match self {
            Self::Led(led) => {
                let v_anode = crate::sim::get_pin_voltage(engine, led.anode());
                let v_cathode = crate::sim::get_pin_voltage(engine, led.cathode());
                led.update_electrical(v_anode, v_cathode);
            }
            Self::Servo(servo) => {
                let v_vcc = crate::sim::get_pin_voltage(engine, servo.vcc());
                let v_gnd = crate::sim::get_pin_voltage(engine, servo.gnd());
                let v_pwm = crate::sim::get_pin_voltage(engine, servo.pwm());
                let pwm_high = v_pwm >= 2.0;
                servo.update_electrical(v_vcc, v_gnd, pwm_high, sim_time_us, dt_seconds);
            }
            Self::Dht22(dht) => {
                let v_vcc = crate::sim::get_pin_voltage(engine, dht.vcc());
                let v_gnd = crate::sim::get_pin_voltage(engine, dht.gnd());
                let v_data = crate::sim::get_pin_voltage(engine, dht.data());
                let data_high = v_data >= 2.0;
                dht.update_electrical(v_vcc, v_gnd, data_high, sim_time_us);
            }
            Self::Mq135(sensor) => {
                let v_vcc = crate::sim::get_pin_voltage(engine, sensor.vcc());
                let v_gnd = crate::sim::get_pin_voltage(engine, sensor.gnd());
                sensor.update_electrical(v_vcc, v_gnd);
            }
            Self::Buzzer(buzzer) => {
                let v_vcc = crate::sim::get_pin_voltage(engine, buzzer.vcc());
                let v_gnd = crate::sim::get_pin_voltage(engine, buzzer.gnd());
                buzzer.update_electrical(v_vcc, v_gnd);
            }
            Self::Ssd1306(_) | Self::Lcd1602(_) => {}
        }
    }

    /// Dispatches an I2C transaction packet if the component address matches.
    pub fn handle_i2c_write(&mut self, address: u8, bytes: &[u8]) {
        use sim_core::bus::i2c::I2cDevice;
        match self {
            Self::Ssd1306(d) if d.address() == address => {
                let _ = d.on_write(bytes);
            }
            Self::Lcd1602(d) if d.address() == address => {
                let _ = d.on_write(bytes);
            }
            _ => {}
        }
    }

    /// Renders component-specific inspection controls and diagnostics in the right pane.
    pub fn render_inspector(
        &mut self,
        ui: &mut Ui,
        engine: &Engine,
        on_delete: impl FnOnce(),
        on_rebuild_netlist: impl FnOnce(),
    ) {
        match self {
            Self::Led(led) => {
                crate::inspector::led::render_led_inspector_content(
                    ui,
                    led,
                    engine,
                    on_delete,
                    on_rebuild_netlist,
                );
            }
            Self::Ssd1306(display) => {
                crate::inspector::ssd1306::render_ssd1306_inspector_content(
                    ui, display, engine, on_delete,
                );
            }
            Self::Lcd1602(display) => {
                crate::inspector::lcd1602::render_lcd1602_inspector_content(
                    ui, display, engine, on_delete,
                );
            }
            Self::Servo(servo) => {
                crate::inspector::servo::render_servo_inspector_content(
                    ui, servo, engine, on_delete,
                );
            }
            Self::Dht22(dht) => {
                crate::inspector::dht22::render_dht22_inspector_content(ui, dht, engine, on_delete);
            }
            Self::Mq135(sensor) => {
                crate::inspector::mq135::render_mq135_inspector_content(
                    ui, sensor, engine, on_delete,
                );
            }
            Self::Buzzer(buzzer) => {
                crate::inspector::buzzer::render_buzzer_inspector_content(
                    ui, buzzer, engine, on_delete,
                );
            }
        }
    }

    /// Renders custom right-click context menu action buttons.
    pub fn render_context_menu(&mut self, ui: &mut Ui) {
        match self {
            Self::Led(led) => {
                crate::canvas::components::led::render_led_context_options(ui, led);
            }
            Self::Ssd1306(display) => {
                if ui.button("🧹 Clear Display").clicked() {
                    display.clear();
                    ui.close_menu();
                }
            }
            Self::Lcd1602(display) => {
                if ui.button("🧹 Clear Display").clicked() {
                    display.clear();
                    ui.close_menu();
                }
            }
            Self::Servo(servo) => {
                crate::canvas::components::servo::render_servo_context_options(ui, servo);
            }
            Self::Dht22(dht) => {
                crate::canvas::components::dht22::render_dht22_context_options(ui, dht);
            }
            Self::Mq135(sensor) => {
                crate::canvas::components::mq135::render_mq135_context_options(ui, sensor);
            }
            Self::Buzzer(_) => {}
        }
    }

    /// Duplicates the component model with fresh PinIds allocated by the provider.
    pub fn duplicate(&self, next_pin: &mut dyn FnMut() -> PinId) -> Self {
        match self {
            Self::Led(c) => Self::Led(Led::new(next_pin(), next_pin(), c.color)),
            Self::Ssd1306(_) => {
                Self::Ssd1306(Ssd1306::new(next_pin(), next_pin(), next_pin(), next_pin()))
            }
            Self::Lcd1602(_) => {
                Self::Lcd1602(Lcd1602::new(next_pin(), next_pin(), next_pin(), next_pin()))
            }
            Self::Servo(_) => Self::Servo(Servo::new(next_pin(), next_pin(), next_pin())),
            Self::Dht22(c) => {
                let mut dup = Dht22::new(next_pin(), next_pin(), next_pin(), next_pin());
                dup.set_temperature(c.temperature());
                dup.set_humidity(c.humidity());
                Self::Dht22(dup)
            }
            Self::Mq135(c) => {
                let mut dup = Mq135::new(next_pin(), next_pin(), next_pin(), next_pin());
                dup.set_ppm(c.ppm());
                dup.set_threshold_ppm(c.threshold_ppm());
                Self::Mq135(dup)
            }
            Self::Buzzer(_) => Self::Buzzer(Buzzer::new(next_pin(), next_pin())),
        }
    }
}
