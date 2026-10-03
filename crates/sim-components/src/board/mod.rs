//! Board architecture and development board definitions.

pub mod esp32_s3;
pub mod traits;
pub mod types;

pub use esp32_s3::Esp32S3DevKit;
pub use traits::Board;
pub use types::{
    BoardDimensions, BoardRgbLed, BoardType, FirmwareError, HeaderPin, HeaderSide,
    RAIL_VOLTAGE_3V3, RAIL_VOLTAGE_5V,
};
