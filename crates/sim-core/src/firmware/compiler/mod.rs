pub mod emitter;
pub mod parser;
pub mod types;

pub use emitter::emit_bytecode;
pub use parser::{parse_statement, strip_comments};
pub use types::{CompileError, IntermediateOp};

/// Classification of an open brace block while compiling structured source.
enum BlockKind {
    Loop,
    If(String),
    Other,
}

/// Parsed condition of an `if (digitalRead(pin) == LEVEL)` header.
struct IfCondition {
    pin: u8,
    jump_if_zero: bool,
}

/// Compiles C/Arduino-like or assembly source code into executable MCU bytecode.
pub fn compile(source: &str) -> Result<Vec<u8>, CompileError> {
    let mut intermediate = Vec::new();
    let mut blocks: Vec<BlockKind> = Vec::new();
    let mut loop_start: Option<String> = None;
    let mut label_counter = 0usize;

    for (line_idx, raw_line) in source.lines().enumerate() {
        let line_num = line_idx + 1;
        let line = strip_comments(raw_line);
        if line.is_empty() {
            continue;
        }

        if let Some(header) = line.strip_suffix('{') {
            let header = header.trim();
            if header.contains("void loop") || header.starts_with("loop:") {
                let label = "__loop_start".to_string();
                intermediate.push(IntermediateOp::Label(label.clone()));
                loop_start = Some(label);
                blocks.push(BlockKind::Loop);
            } else if header.contains("void setup") {
                blocks.push(BlockKind::Other);
            } else if let Some(condition) = parse_if_header(header, line_num)? {
                let label = format!("__if_end_{}", label_counter);
                label_counter += 1;
                intermediate.push(IntermediateOp::ReadPin(condition.pin));
                if condition.jump_if_zero {
                    intermediate.push(IntermediateOp::JumpIfNot(label.clone()));
                } else {
                    intermediate.push(IntermediateOp::JumpIf(label.clone()));
                }
                blocks.push(BlockKind::If(label));
            } else {
                blocks.push(BlockKind::Other);
            }
            continue;
        }

        if line == "}" {
            match blocks.pop() {
                Some(BlockKind::Loop) => {
                    if let Some(label) = &loop_start {
                        intermediate.push(IntermediateOp::Jump(label.clone()));
                    }
                }
                Some(BlockKind::If(label)) => intermediate.push(IntermediateOp::Label(label)),
                _ => {}
            }
            continue;
        }

        if line.ends_with(':') && !line.contains(' ') {
            let label = line.trim_end_matches(':').trim().to_string();
            intermediate.push(IntermediateOp::Label(label));
            continue;
        }

        parse_statement(line, line_num, &mut intermediate)?;
    }

    while let Some(block) = blocks.pop() {
        match block {
            BlockKind::Loop => {
                if let Some(label) = &loop_start {
                    intermediate.push(IntermediateOp::Jump(label.clone()));
                }
            }
            BlockKind::If(label) => intermediate.push(IntermediateOp::Label(label)),
            BlockKind::Other => {}
        }
    }

    emit_bytecode(intermediate)
}

/// Parses an `if` header, returning the pin read and the sense of the skip branch.
fn parse_if_header(header: &str, line_num: usize) -> Result<Option<IfCondition>, CompileError> {
    let trimmed = header.trim();
    if !trimmed.starts_with("if") {
        return Ok(None);
    }

    let open = trimmed
        .find('(')
        .ok_or_else(|| CompileError::new(line_num, "Missing '(' in if statement"))?;
    let close = trimmed
        .rfind(')')
        .ok_or_else(|| CompileError::new(line_num, "Missing ')' in if statement"))?;
    if close <= open {
        return Err(CompileError::new(line_num, "Mismatched parentheses in if"));
    }

    let condition = &trimmed[open + 1..close];
    let read_open = condition
        .find("digitalRead(")
        .ok_or_else(|| CompileError::new(line_num, "if condition must call digitalRead(pin)"))?;
    let read_close = condition[read_open..]
        .find(')')
        .map(|offset| read_open + offset)
        .ok_or_else(|| CompileError::new(line_num, "Missing ')' in digitalRead"))?;

    let pin: u8 = condition[read_open + "digitalRead(".len()..read_close]
        .trim()
        .parse()
        .map_err(|_| CompileError::new(line_num, "Invalid pin in digitalRead"))?;

    let rest = condition[read_close + 1..].trim();
    let jump_if_zero = if rest.is_empty() {
        true
    } else if let Some(value) = rest.strip_prefix("==") {
        parse_level_condition(value.trim(), line_num, true)?
    } else if let Some(value) = rest.strip_prefix("!=") {
        parse_level_condition(value.trim(), line_num, false)?
    } else {
        return Err(CompileError::new(line_num, "Unsupported if condition"));
    };

    Ok(Some(IfCondition { pin, jump_if_zero }))
}

/// Maps `HIGH`/`LOW` (or `1`/`0`) to whether a jump-if-zero should be taken when equal.
fn parse_level_condition(
    value: &str,
    line_num: usize,
    for_equality: bool,
) -> Result<bool, CompileError> {
    let is_high = value.eq_ignore_ascii_case("HIGH") || value == "1";
    let is_low = value.eq_ignore_ascii_case("LOW") || value == "0";
    if !is_high && !is_low {
        return Err(CompileError::new(
            line_num,
            "Unsupported level in if condition",
        ));
    }
    let skip_when_zero = is_high == for_equality;
    Ok(skip_when_zero)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::firmware::opcodes::*;

    #[test]
    fn test_compile_arduino_blink_syntax() {
        let source = r#"
            void setup() {
                pinMode(2, OUTPUT);
                Serial.println("Ready");
            }
            void loop() {
                digitalWrite(2, HIGH);
                delay(100);
                digitalWrite(2, LOW);
                delay(100);
            }
        "#;
        let bytes = compile(source).expect("Compilation failed");
        assert!(!bytes.is_empty());
        assert_eq!(bytes[0], OP_UART_SEND);
    }

    #[test]
    fn test_compile_assembly_syntax() {
        let source = r#"
            SET_HIGH 4
            DELAY 10
            SET_LOW 4
            JUMP 0
        "#;
        let bytes = compile(source).expect("Assembly compile failed");
        assert_eq!(
            bytes,
            vec![
                OP_SET_HIGH,
                4,
                OP_DELAY,
                10,
                0,
                OP_SET_LOW,
                4,
                OP_JUMP,
                0,
                0
            ]
        );
    }

    #[test]
    fn test_compile_lcd_print() {
        let bytes = compile("lcd.print(\"Hi\");").expect("LCD print compile failed");
        assert_eq!(bytes, vec![OP_I2C_WRITE, 0x27, 3, 0x40, b'H', b'i']);
    }

    #[test]
    fn test_compile_lcd_clear() {
        let bytes = compile("lcd.clear();").expect("LCD clear compile failed");
        assert_eq!(bytes, vec![OP_I2C_WRITE, 0x27, 2, 0x00, 0x01]);
    }

    #[test]
    fn test_compile_full_arduino_lcd_sketch() {
        let sketch = r#"
            #include <LiquidCrystal_I2C.h>

            LiquidCrystal_I2C lcd(0x27, 16, 2);

            void setup() {
                lcd.init();
                lcd.backlight();
                lcd.setCursor(0, 0);
                lcd.print("Hello World!");
                lcd.setCursor(0, 1);
                lcd.print("LCD 1602 Test");
            }

            void loop() {
            }
        "#;
        let bytes = compile(sketch).expect("Full LCD sketch compile failed");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_compile_oled_print() {
        let bytes = compile("display.print(\"Hi\");").expect("OLED print compile failed");
        assert_eq!(bytes, vec![OP_I2C_WRITE, 0x3C, 3, 0x40, b'H', b'i']);
    }

    #[test]
    fn test_compile_oled_clear() {
        let bytes = compile("display.clearDisplay();").expect("OLED clear compile failed");
        assert_eq!(bytes, vec![OP_I2C_WRITE, 0x3C, 2, 0x00, 0x01]);
    }

    #[test]
    fn test_compile_full_arduino_oled_sketch() {
        let sketch = r#"
            #include <Wire.h>
            #include <Adafruit_GFX.h>
            #include <Adafruit_SSD1306.h>

            #define SCREEN_WIDTH 128
            #define SCREEN_HEIGHT 64
            #define OLED_RESET -1
            #define SCREEN_ADDRESS 0x3C

            Adafruit_SSD1306 display(SCREEN_WIDTH, SCREEN_HEIGHT, &Wire, OLED_RESET);

            void setup() {
                Wire.begin(8, 9);
                display.begin(SSD1306_SWITCHCAPVCC, SCREEN_ADDRESS);
                display.clearDisplay();
                display.setTextSize(1);
                display.setTextColor(SSD1306_WHITE);
                display.setCursor(0, 0);
                display.println("ESP32-S3 Ready!");
                display.display();
            }

            void loop() {
            }
        "#;
        let bytes = compile(sketch).expect("Full sketch compile failed");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_compile_if_digital_read_high() {
        let source = r#"
            void loop() {
                if (digitalRead(2) == HIGH) {
                    digitalWrite(3, HIGH);
                }
            }
        "#;
        let bytes = compile(source).expect("if compile failed");
        assert_eq!(
            bytes,
            vec![
                OP_READ_PIN,
                2,
                OP_JUMP_IF_NOT,
                7,
                0,
                OP_SET_HIGH,
                3,
                OP_JUMP,
                0,
                0
            ]
        );
    }

    #[test]
    fn test_compile_if_digital_read_low() {
        let source = r#"
            void loop() {
                if (digitalRead(4) == LOW) {
                    digitalWrite(5, LOW);
                }
            }
        "#;
        let bytes = compile(source).expect("if compile failed");
        assert_eq!(
            bytes,
            vec![
                OP_READ_PIN,
                4,
                OP_JUMP_IF,
                7,
                0,
                OP_SET_LOW,
                5,
                OP_JUMP,
                0,
                0
            ]
        );
    }
}
