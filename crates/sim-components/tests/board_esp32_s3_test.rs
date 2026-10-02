use sim_components::board::esp32_s3::{
    ESP32_S3_DEVKIT_PIN_COUNT, Esp32S3DevKit, GpioMode, GpioPull,
};
use sim_components::board::{Board, BoardType, FirmwareError, HeaderSide};
use sim_core::firmware::opcodes::{OP_DELAY, OP_HALT, OP_SET_HIGH, OP_SET_LOW};
use sim_core::netlist::PinId;
use sim_core::pins::{DigitalState, PinDrive};

/// Verifies that the ESP32-S3 DevKit creates exactly 44 pins split evenly across headers.
#[test]
fn test_pinout_counts_and_sides() {
    let board = Esp32S3DevKit::new(PinId(100));
    assert_eq!(board.pins().len(), ESP32_S3_DEVKIT_PIN_COUNT);
    assert_eq!(board.header_pins().len(), ESP32_S3_DEVKIT_PIN_COUNT);

    let left_count = board
        .header_pins()
        .iter()
        .filter(|p| p.side == HeaderSide::Left)
        .count();
    let right_count = board
        .header_pins()
        .iter()
        .filter(|p| p.side == HeaderSide::Right)
        .count();

    assert_eq!(left_count, 22);
    assert_eq!(right_count, 22);
    assert_eq!(board.board_type(), BoardType::Esp32S3DevKit);
}

/// Verifies resolution of GPIO pins from header numbers.
#[test]
fn test_gpio_pin_resolution() {
    let board = Esp32S3DevKit::new(PinId(0));

    let io0 = board.gpio_pin(0);
    assert!(io0.is_some());
    assert_eq!(board.pin_name(io0.unwrap()), Some("IO0"));

    let io48 = board.gpio_pin(48);
    assert!(io48.is_some());
    assert_eq!(board.pin_name(io48.unwrap()), Some("IO48"));

    let tx = board.gpio_pin(43);
    assert!(tx.is_some());
    assert_eq!(board.pin_name(tx.unwrap()), Some("TX"));

    let rx = board.gpio_pin(44);
    assert!(rx.is_some());
    assert_eq!(board.pin_name(rx.unwrap()), Some("RX"));

    assert_eq!(board.gpio_pin(99), None);
}

/// Verifies power and ground terminal resolution and electrical output states.
#[test]
fn test_power_and_ground_terminals() {
    let mut board = Esp32S3DevKit::new(PinId(10));
    let (vcc, grounds) = board.power_terminals();

    assert!(vcc.is_some());
    assert_eq!(grounds.len(), 4);

    let vcc_pin = vcc.unwrap();
    assert_eq!(
        board.pin_drive(vcc_pin),
        PinDrive::Driven(DigitalState::High)
    );
    assert_eq!(
        board.pin_drive(grounds[0]),
        PinDrive::Driven(DigitalState::Low)
    );

    board.set_powered(false);
    assert!(!board.is_powered());
    assert_eq!(board.pin_drive(vcc_pin), PinDrive::HighZ);
    assert!(!board.is_power_led_on());
}

/// Verifies GPIO output driving and pull modes.
#[test]
fn test_gpio_driver_modes() {
    let mut board = Esp32S3DevKit::new(PinId(0));
    let pin_io4 = board.gpio_pin(4).unwrap();

    assert_eq!(board.pin_drive(pin_io4), PinDrive::HighZ);

    board.gpio_controller_mut().set_mode(4, GpioMode::Output);
    board.gpio_controller_mut().set_output(4, true);
    assert_eq!(
        board.pin_drive(pin_io4),
        PinDrive::Driven(DigitalState::High)
    );

    board.gpio_controller_mut().set_output(4, false);
    assert_eq!(
        board.pin_drive(pin_io4),
        PinDrive::Driven(DigitalState::Low)
    );

    board.gpio_controller_mut().set_mode(4, GpioMode::Input);
    board.gpio_controller_mut().set_pull(4, GpioPull::PullUp);
    assert_eq!(board.pin_drive(pin_io4), PinDrive::PullUp);

    board.gpio_controller_mut().set_pull(4, GpioPull::PullDown);
    assert_eq!(board.pin_drive(pin_io4), PinDrive::PullDown);
}

/// Verifies that pressing the Boot button pulls GPIO0 to ground.
#[test]
fn test_boot_button_action() {
    let mut board = Esp32S3DevKit::new(PinId(0));
    assert!(!board.is_boot_pressed());
    assert!(board.gpio_controller().read_input(0));

    board.set_boot_pressed(true);
    assert!(board.is_boot_pressed());
    assert!(!board.gpio_controller().read_input(0));

    board.set_boot_pressed(false);
    assert!(!board.is_boot_pressed());
    assert!(board.gpio_controller().read_input(0));
}

/// Verifies that pressing the Reset button triggers board reset on release.
#[test]
fn test_reset_button_action() {
    let mut board = Esp32S3DevKit::new(PinId(0));
    board.gpio_controller_mut().set_mode(4, GpioMode::Output);
    board.gpio_controller_mut().set_output(4, true);

    board.set_reset_pressed(true);
    assert!(board.is_reset_pressed());

    let rst_pin = board
        .header_pins()
        .iter()
        .find(|p| p.name == "RST")
        .unwrap()
        .pin_id;
    assert_eq!(
        board.pin_drive(rst_pin),
        PinDrive::Driven(DigitalState::Low)
    );

    board.set_reset_pressed(false);
    assert!(!board.is_reset_pressed());
    assert_eq!(board.pin_drive(rst_pin), PinDrive::PullUp);
    assert_eq!(board.gpio_controller().pin_drive(4), PinDrive::HighZ);
}

/// Verifies firmware loading validation errors.
#[test]
fn test_firmware_loading_validation() {
    let mut board = Esp32S3DevKit::new(PinId(0));

    let empty_err = board.load_firmware(&[]).unwrap_err();
    assert_eq!(empty_err, FirmwareError::EmptyBinary);

    let huge_data = vec![0u8; 17 * 1024 * 1024];
    let too_large_err = board.load_firmware(&huge_data).unwrap_err();
    assert!(matches!(
        too_large_err,
        FirmwareError::BinaryTooLarge { .. }
    ));
}

/// Verifies serial monitor output clearing.
#[test]
fn test_serial_monitor_buffer() {
    let mut board = Esp32S3DevKit::new(PinId(0));
    assert_eq!(board.serial_output(), "");

    board.send_serial_input("AT\r\n");
    board.clear_serial_output();
    assert_eq!(board.serial_output(), "");
}

/// Verifies RGB LED state modifications.
#[test]
fn test_rgb_led_states() {
    let mut board = Esp32S3DevKit::new(PinId(0));
    let initial_led = board.rgb_led();
    assert!(!initial_led.enabled);

    board.gpio_controller_mut().set_mode(48, GpioMode::Output);
    board.gpio_controller_mut().set_output(48, true);
    board.step(0.001);

    let led_on = board.rgb_led();
    assert!(led_on.enabled);
    assert_eq!((led_on.r, led_on.g, led_on.b), (255, 255, 255));
}

/// Verifies that loading virtual bytecode and advancing simulation steps toggles GPIO pins correctly.
#[test]
fn test_bytecode_execution_and_gpio_toggle() {
    let mut board = Esp32S3DevKit::new(PinId(0));
    let bytecode = vec![OP_SET_HIGH, 4, OP_DELAY, 10, 0, OP_SET_LOW, 4, OP_HALT];
    assert!(board.load_firmware(&bytecode).is_ok());
    let pin_io4 = board.gpio_pin(4).unwrap();

    board.step(0.001);
    assert_eq!(
        board.pin_drive(pin_io4),
        PinDrive::Driven(DigitalState::High)
    );

    board.step(0.009);
    board.step(0.001);
    assert_eq!(
        board.pin_drive(pin_io4),
        PinDrive::Driven(DigitalState::Low)
    );
}

/// Verifies that Esp32S3DevKit drives the SSD1306 display via I2C.
#[test]
fn test_esp32_s3_firmware_and_ssd1306() {
    use sim_core::bus::i2c::I2cDevice;
    let fw = sim_core::firmware::compile("display.print(\"Hi\");").expect("OLED compile");
    let mut board = Esp32S3DevKit::new(PinId(0));
    assert!(board.load_firmware(&fw).is_ok());

    let mut display =
        sim_components::component::Ssd1306::new(PinId(100), PinId(101), PinId(102), PinId(103));

    for _ in 0..100 {
        board.step(0.001);
        for (addr, bytes) in board.drain_i2c() {
            if addr == 0x3C {
                let _ = display.on_write(&bytes);
            }
        }
    }

    let lit_pixels = (0..128)
        .flat_map(|x| (0..64).map(move |y| (x, y)))
        .filter(|&(x, y)| display.pixel_at(x, y))
        .count();
    assert!(
        lit_pixels > 0,
        "SSD1306 should render lit pixels from firmware execution"
    );
}
