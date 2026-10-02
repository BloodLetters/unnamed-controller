//! Factory metadata and specifications for instantiating component types.

use sim_components::component::{Buzzer, Dht22, Lcd1602, Led, LedColor, Mq135, Servo, Ssd1306};
use sim_core::netlist::PinId;

use crate::component::instance::ComponentInstance;

/// Category metadata and factory recipe for creating new component instances.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ComponentKind {
    Led(LedColor),
    Ssd1306,
    Lcd1602,
    Servo,
    Dht22,
    Mq135,
    Buzzer,
}

impl ComponentKind {
    /// Instantiates a new component instance with PinIds allocated from the provider.
    pub fn create(&self, next_pin: &mut dyn FnMut() -> PinId) -> ComponentInstance {
        match self {
            Self::Led(color) => ComponentInstance::Led(Led::new(next_pin(), next_pin(), *color)),
            Self::Ssd1306 => ComponentInstance::Ssd1306(Ssd1306::new(
                next_pin(),
                next_pin(),
                next_pin(),
                next_pin(),
            )),
            Self::Lcd1602 => ComponentInstance::Lcd1602(Lcd1602::new(
                next_pin(),
                next_pin(),
                next_pin(),
                next_pin(),
            )),
            Self::Servo => ComponentInstance::Servo(Servo::new(next_pin(), next_pin(), next_pin())),
            Self::Dht22 => {
                ComponentInstance::Dht22(Dht22::new(next_pin(), next_pin(), next_pin(), next_pin()))
            }
            Self::Mq135 => {
                ComponentInstance::Mq135(Mq135::new(next_pin(), next_pin(), next_pin(), next_pin()))
            }
            Self::Buzzer => ComponentInstance::Buzzer(Buzzer::new(next_pin(), next_pin())),
        }
    }

    /// Default canvas orientation angle in degrees when spawned.
    pub fn default_rotation(&self) -> u16 {
        match self {
            Self::Ssd1306 => 180,
            _ => 0,
        }
    }
}
