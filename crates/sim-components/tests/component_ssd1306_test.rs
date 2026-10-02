use sim_components::component::{SSD1306_DEFAULT_ADDRESS, SSD1306_HEIGHT, SSD1306_WIDTH, Ssd1306};
use sim_core::bus::i2c::I2cDevice;
use sim_core::component::Component;
use sim_core::netlist::PinId;

/// Verifies that the SSD1306 initializes with the correct 4-pin layout and address.
#[test]
fn test_ssd1306_creation_and_pins() {
    let display = Ssd1306::new(PinId(1), PinId(2), PinId(3), PinId(4));
    assert_eq!(display.gnd(), PinId(1));
    assert_eq!(display.vcc(), PinId(2));
    assert_eq!(display.sda(), PinId(3));
    assert_eq!(display.scl(), PinId(4));
    assert_eq!(display.pins().len(), 4);
    assert_eq!(display.address(), SSD1306_DEFAULT_ADDRESS);
    assert!(display.is_display_on());
}

/// Verifies direct pixel manipulation and display buffer clearing.
#[test]
fn test_ssd1306_pixel_manipulation_and_clear() {
    let mut display = Ssd1306::new(PinId(1), PinId(2), PinId(3), PinId(4));
    assert!(!display.pixel_at(10, 20));

    display.set_pixel(10, 20, true);
    assert!(display.pixel_at(10, 20));

    display.set_pixel(10, 20, false);
    assert!(!display.pixel_at(10, 20));

    display.set_pixel(0, 0, true);
    display.set_pixel(SSD1306_WIDTH - 1, SSD1306_HEIGHT - 1, true);
    assert!(display.pixel_at(0, 0));
    assert!(display.pixel_at(SSD1306_WIDTH - 1, SSD1306_HEIGHT - 1));

    display.clear();
    assert!(!display.pixel_at(0, 0));
    assert!(!display.pixel_at(SSD1306_WIDTH - 1, SSD1306_HEIGHT - 1));
}

/// Verifies that I2C command streams configure memory addressing mode.
#[test]
fn test_ssd1306_i2c_addressing_mode_configuration() {
    let mut display = Ssd1306::new(PinId(1), PinId(2), PinId(3), PinId(4));

    let cmd_horiz = [0x00, 0x20, 0x00];
    assert!(display.on_write(&cmd_horiz).is_ok());

    let data_payload = [0x40, 0b00000001, 0b00000010];
    assert!(display.on_write(&data_payload).is_ok());

    assert!(display.pixel_at(0, 0));
    assert!(!display.pixel_at(0, 1));
    assert!(!display.pixel_at(1, 0));
    assert!(display.pixel_at(1, 1));
}

/// Verifies normal, inverted, and entire-display-on command operations.
#[test]
fn test_ssd1306_display_mode_commands() {
    let mut display = Ssd1306::new(PinId(1), PinId(2), PinId(3), PinId(4));
    display.set_pixel(5, 5, true);
    assert!(display.pixel_at(5, 5));
    assert!(!display.pixel_at(6, 5));

    let cmd_inverse = [0x80, 0xA7];
    assert!(display.on_write(&cmd_inverse).is_ok());
    assert!(!display.pixel_at(5, 5));
    assert!(display.pixel_at(6, 5));

    let cmd_normal = [0x80, 0xA6];
    assert!(display.on_write(&cmd_normal).is_ok());
    assert!(display.pixel_at(5, 5));

    let cmd_entire_on = [0x80, 0xA5];
    assert!(display.on_write(&cmd_entire_on).is_ok());
    assert!(display.pixel_at(0, 0));
    assert!(display.pixel_at(50, 50));

    let cmd_ram_resume = [0x80, 0xA4];
    assert!(display.on_write(&cmd_ram_resume).is_ok());
    assert!(!display.pixel_at(6, 5));
    assert!(display.pixel_at(5, 5));
}
