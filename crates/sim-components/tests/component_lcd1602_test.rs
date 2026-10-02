use sim_components::component::Lcd1602;
use sim_core::bus::i2c::I2cDevice;
use sim_core::netlist::PinId;

#[test]
fn test_lcd1602_creation_and_pins() {
    let lcd = Lcd1602::new(PinId(10), PinId(11), PinId(12), PinId(13));
    assert_eq!(lcd.gnd(), PinId(10));
    assert_eq!(lcd.vcc(), PinId(11));
    assert_eq!(lcd.sda(), PinId(12));
    assert_eq!(lcd.scl(), PinId(13));
    assert_eq!(lcd.address(), 0x27);
    assert!(lcd.is_backlight_on());
    assert!(lcd.is_display_on());
    assert_eq!(lcd.get_char(0, 0), b' ');
}

#[test]
fn test_lcd1602_text_and_cursor() {
    let mut lcd = Lcd1602::new(PinId(1), PinId(2), PinId(3), PinId(4));
    lcd.write_char(b'H');
    lcd.write_char(b'i');
    assert_eq!(lcd.get_char(0, 0), b'H');
    assert_eq!(lcd.get_char(1, 0), b'i');
    assert_eq!(lcd.get_char(2, 0), b' ');

    lcd.set_cursor(0, 1);
    lcd.write_char(b'B');
    assert_eq!(lcd.get_char(0, 1), b'B');

    lcd.clear();
    assert_eq!(lcd.get_char(0, 0), b' ');
    assert_eq!(lcd.get_char(0, 1), b' ');
}

#[test]
fn test_lcd1602_i2c_high_level_packet() {
    let mut lcd = Lcd1602::new(PinId(1), PinId(2), PinId(3), PinId(4));
    let frame = [0x40, b'H', b'e', b'l', b'l', b'o'];
    let res = lcd.on_write(&frame);
    assert!(res.is_ok());
    assert_eq!(&lcd.get_row_str(0)[0..5], "Hello");
}

#[test]
fn test_lcd1602_i2c_pcf8574_nibbles() {
    let mut lcd = Lcd1602::new(PinId(1), PinId(2), PinId(3), PinId(4));
    let packet = [0x4D, 0x49, 0x1D, 0x19];
    let res = lcd.on_write(&packet);
    assert!(res.is_ok());
    assert_eq!(lcd.get_char(0, 0), b'A');
}

fn pcf8574_send_nibble(lcd: &mut Lcd1602, nibble: u8, rs: bool) {
    let mode = if rs { 0x01 } else { 0x00 };
    let bl = 0x08;
    let n = nibble & 0xF0;
    let _ = lcd.on_write(&[n | mode | bl | 0x04, n | mode | bl]);
}

fn pcf8574_send_byte(lcd: &mut Lcd1602, val: u8, rs: bool) {
    pcf8574_send_nibble(lcd, val & 0xF0, rs);
    pcf8574_send_nibble(lcd, (val << 4) & 0xF0, rs);
}

#[test]
fn test_lcd1602_real_liquidcrystal_i2c_init_and_print() {
    let mut lcd = Lcd1602::new(PinId(1), PinId(2), PinId(3), PinId(4));

    pcf8574_send_nibble(&mut lcd, 0x30, false);
    pcf8574_send_nibble(&mut lcd, 0x30, false);
    pcf8574_send_nibble(&mut lcd, 0x30, false);
    pcf8574_send_nibble(&mut lcd, 0x20, false);

    pcf8574_send_byte(&mut lcd, 0x28, false);
    pcf8574_send_byte(&mut lcd, 0x0C, false);
    pcf8574_send_byte(&mut lcd, 0x01, false);
    pcf8574_send_byte(&mut lcd, 0x06, false);

    for &c in b"HELLO" {
        pcf8574_send_byte(&mut lcd, c, true);
    }

    pcf8574_send_byte(&mut lcd, 0xC2, false);

    for &c in b"WORLD" {
        pcf8574_send_byte(&mut lcd, c, true);
    }

    assert_eq!(&lcd.get_row_str(0)[0..5], "HELLO");
    assert_eq!(&lcd.get_row_str(1)[2..7], "WORLD");
}

/// Verifies end-to-end integration where an ESP32-S3 executes compiled sketch bytecode driving LCD1602 via I2C.
#[test]
fn test_lcd1602_esp32_s3_bytecode_integration() {
    use sim_components::board::{Board, Esp32S3DevKit};
    use sim_core::firmware::compile;

    let source = r#"
        #include <LiquidCrystal_I2C.h>
        LiquidCrystal_I2C lcd(0x27, 16, 2);

        void setup() {
            lcd.init();
            lcd.backlight();
            lcd.setCursor(0, 0);
            lcd.print("Antigravity");
            lcd.setCursor(0, 1);
            lcd.print("LCD 1602 OK");
        }
        void loop() {
        }
    "#;
    let bytecode = compile(source).expect("Compilation must succeed");
    let mut board = Esp32S3DevKit::new(PinId(0));
    assert!(board.load_firmware(&bytecode).is_ok());

    let mut lcd = Lcd1602::new(PinId(10), PinId(11), PinId(12), PinId(13));
    for _ in 0..10 {
        board.step(0.001);
        for (addr, bytes) in board.drain_i2c() {
            if addr == lcd.address() {
                let _ = lcd.on_write(&bytes);
            }
        }
    }

    assert_eq!(&lcd.get_row_str(0)[0..11], "Antigravity");
    assert_eq!(&lcd.get_row_str(1)[0..11], "LCD 1602 OK");
}
