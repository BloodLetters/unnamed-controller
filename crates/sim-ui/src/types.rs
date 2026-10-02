use sim_components::component::LedColor;
use sim_core::netlist::PinId;

/// Screen layout mode controlling visibility of code editor and circuit canvas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum ViewMode {
    Code,
    #[default]
    Both,
    Circuit,
}

/// Component or board type selected for placement on the canvas.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SpawningComponent {
    #[default]
    None,
    Esp32S3,
    Component(crate::component::ComponentKind),
}

impl SpawningComponent {
    /// Helper to spawn a standard Red LED.
    pub const LED: Self = Self::Component(crate::component::ComponentKind::Led(LedColor::Red));
    /// Helper to spawn an SSD1306 OLED display.
    pub const SSD1306: Self = Self::Component(crate::component::ComponentKind::Ssd1306);
    /// Helper to spawn a 1602 LCD display.
    pub const LCD1602: Self = Self::Component(crate::component::ComponentKind::Lcd1602);
    /// Helper to spawn an SG90 servo motor.
    pub const SERVO: Self = Self::Component(crate::component::ComponentKind::Servo);
    /// Helper to spawn a DHT22 humidity and temperature sensor.
    pub const DHT22: Self = Self::Component(crate::component::ComponentKind::Dht22);
    /// Helper to spawn an MQ-135 gas sensor.
    pub const MQ135: Self = Self::Component(crate::component::ComponentKind::Mq135);
    /// Helper to spawn an active buzzer.
    pub const BUZZER: Self = Self::Component(crate::component::ComponentKind::Buzzer);
}

/// Identifies the component or element currently selected for inspection or manipulation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectedItem {
    None,
    Wire(usize),
    Group(Vec<SelectedItem>),
    Esp32S3(usize),
    Component(usize),
}

impl SelectedItem {
    /// Returns true if the selected item or group contains the target item.
    pub fn contains_item(&self, item: &SelectedItem) -> bool {
        if self == item {
            return true;
        }
        if let SelectedItem::Group(group) = self {
            return group.contains(item);
        }
        false
    }

    /// Returns true if the selected item is a wire.
    pub fn is_wire(&self) -> bool {
        matches!(self, SelectedItem::Wire(_))
    }

    /// Returns true if no item is selected.
    pub fn is_none(&self) -> bool {
        matches!(self, SelectedItem::None)
    }
}

/// Connection wire between two pins with customizable color and label.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Wire {
    pub from: PinId,
    pub to: PinId,
    pub color: [u8; 3],
    pub label: String,
}

impl Wire {
    /// Creates a new wire with default signal emerald styling.
    pub fn new(from: PinId, to: PinId) -> Self {
        Self {
            from,
            to,
            color: [76, 175, 80],
            label: String::new(),
        }
    }
}

/// Current serialized schema version of [`ProjectData`].
pub const PROJECT_VERSION: u32 = 1;

/// Schema version assumed for snapshots saved before the field existed.
fn default_project_version() -> u32 {
    PROJECT_VERSION
}

/// Serializable circuit project state representing a complete circuit snapshot.
#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ProjectData {
    #[serde(default = "default_project_version")]
    pub version: u32,
    #[serde(default)]
    pub wires: Vec<Wire>,
    #[serde(default)]
    pub esp32_s3_boards: Vec<[f32; 2]>,
    #[serde(default)]
    pub esp32_s3_rotations: Vec<u16>,
    #[serde(default)]
    pub components: Vec<crate::component::PlacedComponent>,
    #[serde(default)]
    pub leds: Vec<([f32; 2], LedColor)>,
    #[serde(default)]
    pub ssd1306_displays: Vec<[f32; 2]>,
    #[serde(default)]
    pub lcd1602_displays: Vec<[f32; 2]>,
    #[serde(default)]
    pub servos: Vec<[f32; 2]>,
    #[serde(default)]
    pub led_rotations: Vec<u16>,
    #[serde(default)]
    pub ssd1306_rotations: Vec<u16>,
    #[serde(default)]
    pub lcd1602_rotations: Vec<u16>,
    #[serde(default)]
    pub servo_rotations: Vec<u16>,
}
