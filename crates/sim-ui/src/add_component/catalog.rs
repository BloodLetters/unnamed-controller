//! Static catalog repository defining all available and upcoming devices.

use sim_components::component::LedColor;

use super::types::{CategoryKind, ComponentItem};
use crate::component::ComponentKind;
use crate::types::SpawningComponent;

/// Returns the complete registry of all catalog components.
pub fn get_catalog() -> &'static [ComponentItem] {
    &CATALOG_ITEMS
}

/// Static catalog array containing all currently implemented components.
const CATALOG_ITEMS: [ComponentItem; 13] = [
    ComponentItem {
        id: "esp32_s3_devkit",
        name: "ESP32-S3 DevKit",
        category: CategoryKind::Boards,
        specs: "Xtensa LX7 dual-core, 240MHz, Wi-Fi+BT, AI accel",
        badge: Some("Ready"),
        is_pro: false,
        is_ready: true,
        spawn: SpawningComponent::Esp32S3,
    },
    ComponentItem {
        id: "ssd1306_oled",
        name: "SSD1306 OLED (128x64)",
        category: CategoryKind::Displays,
        specs: "I2C monochrome graphic display, address 0x3C",
        badge: Some("Ready"),
        is_pro: false,
        is_ready: true,
        spawn: SpawningComponent::SSD1306,
    },
    ComponentItem {
        id: "lcd1602",
        name: "LCD 1602 (I2C)",
        category: CategoryKind::Displays,
        specs: "16x2 character alphanumeric LCD with PCF8574 I2C backpack",
        badge: Some("Ready"),
        is_pro: false,
        is_ready: true,
        spawn: SpawningComponent::LCD1602,
    },
    ComponentItem {
        id: "led_red",
        name: "LED 5mm (Red)",
        category: CategoryKind::Output,
        specs: "Forward Voltage: 2.0V, 630nm standard indicator",
        badge: Some("Ready"),
        is_pro: false,
        is_ready: true,
        spawn: SpawningComponent::Component(ComponentKind::Led(LedColor::Red)),
    },
    ComponentItem {
        id: "led_green",
        name: "LED 5mm (Green)",
        category: CategoryKind::Output,
        specs: "Forward Voltage: 2.2V, 525nm high efficiency",
        badge: Some("Ready"),
        is_pro: false,
        is_ready: true,
        spawn: SpawningComponent::Component(ComponentKind::Led(LedColor::Green)),
    },
    ComponentItem {
        id: "led_blue",
        name: "LED 5mm (Blue)",
        category: CategoryKind::Output,
        specs: "Forward Voltage: 3.2V, 470nm blue indicator",
        badge: Some("Ready"),
        is_pro: false,
        is_ready: true,
        spawn: SpawningComponent::Component(ComponentKind::Led(LedColor::Blue)),
    },
    ComponentItem {
        id: "led_yellow",
        name: "LED 5mm (Yellow)",
        category: CategoryKind::Output,
        specs: "Forward Voltage: 2.1V, 590nm amber diode",
        badge: Some("Ready"),
        is_pro: false,
        is_ready: true,
        spawn: SpawningComponent::Component(ComponentKind::Led(LedColor::Yellow)),
    },
    ComponentItem {
        id: "led_white",
        name: "LED 5mm (White)",
        category: CategoryKind::Output,
        specs: "Forward Voltage: 3.2V, phosphor-converted white",
        badge: Some("Ready"),
        is_pro: false,
        is_ready: true,
        spawn: SpawningComponent::Component(ComponentKind::Led(LedColor::White)),
    },
    ComponentItem {
        id: "led_orange",
        name: "LED 5mm (Orange)",
        category: CategoryKind::Output,
        specs: "Forward Voltage: 2.0V, 605nm orange indicator",
        badge: Some("Ready"),
        is_pro: false,
        is_ready: true,
        spawn: SpawningComponent::Component(ComponentKind::Led(LedColor::Orange)),
    },
    ComponentItem {
        id: "servo_sg90",
        name: "SG90 Micro Servo",
        category: CategoryKind::Output,
        specs: "9g micro servo, 0°-180° rotation, 50Hz PWM control (1-2ms pulse)",
        badge: Some("Ready"),
        is_pro: false,
        is_ready: true,
        spawn: SpawningComponent::SERVO,
    },
    ComponentItem {
        id: "dht22_sensor",
        name: "DHT22 / AM2302 Sensor",
        category: CategoryKind::Sensors,
        specs: "Digital relative humidity and temperature sensor (-40°C to 80°C, 0 to 100% RH)",
        badge: Some("Ready"),
        is_pro: false,
        is_ready: true,
        spawn: SpawningComponent::DHT22,
    },
    ComponentItem {
        id: "mq135_sensor",
        name: "MQ-135 Gas Sensor",
        category: CategoryKind::Sensors,
        specs: "Air quality sensor: NH3, NOx, CO2, benzene, smoke (10-1000 ppm, 5V, analog+digital out)",
        badge: Some("Ready"),
        is_pro: false,
        is_ready: true,
        spawn: SpawningComponent::MQ135,
    },
    ComponentItem {
        id: "active_buzzer",
        name: "Active Buzzer",
        category: CategoryKind::Output,
        specs: "5V internal-oscillator piezoelectric buzzer, 2400 Hz tone, 30 mA",
        badge: Some("Ready"),
        is_pro: false,
        is_ready: true,
        spawn: SpawningComponent::BUZZER,
    },
];
