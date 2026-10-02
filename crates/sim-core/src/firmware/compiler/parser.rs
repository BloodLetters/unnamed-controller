use super::types::{CompileError, IntermediateOp};

/// Strips single-line and end-of-line comments from source text.
pub fn strip_comments(raw: &str) -> &str {
    let without_slashes = match raw.find("//") {
        Some(idx) => &raw[..idx],
        None => raw,
    };
    without_slashes.trim()
}

/// Parses a single statement line into intermediate instructions.
pub fn parse_statement(
    line: &str,
    line_num: usize,
    ops: &mut Vec<IntermediateOp>,
) -> Result<(), CompileError> {
    let clean = line.trim_end_matches(';').trim();
    if clean.is_empty()
        || clean == "{"
        || clean.starts_with("void setup")
        || clean.starts_with('#')
        || is_ignorable_declaration(clean)
    {
        return Ok(());
    }

    let lower = clean.to_ascii_lowercase();
    if clean.starts_with("digitalWrite(") {
        parse_digital_write(clean, line_num, ops)
    } else if clean.starts_with("delay(") {
        parse_delay(clean, line_num, ops)
    } else if clean.starts_with("Serial.println(") {
        parse_serial_print(clean, line_num, ops, true)
    } else if clean.starts_with("Serial.print(") {
        parse_serial_print(clean, line_num, ops, false)
    } else if clean.starts_with("ledcWrite(") || clean.starts_with("analogWrite(") {
        parse_ledc_write(clean, line_num, ops)
    } else if clean.starts_with("WiFi.begin(") {
        parse_wifi_begin(clean, line_num, ops)
    } else if lower.starts_with("lcd.setcursor(") {
        parse_lcd_set_cursor(clean, line_num, ops)
    } else if lower.starts_with("lcd.print(") {
        parse_lcd_print(clean, line_num, ops)
    } else if lower.starts_with("lcd.clear(") {
        parse_lcd_clear(clean, line_num, ops)
    } else if lower.starts_with("display.println(")
        || lower.starts_with("display.print(")
        || lower.starts_with("oled.println(")
        || lower.starts_with("oled.print(")
    {
        parse_oled_print(clean, line_num, ops)
    } else if lower.starts_with("display.cleardisplay(") || lower.starts_with("oled.cleardisplay(")
    {
        ops.push(IntermediateOp::OledClear);
        Ok(())
    } else {
        parse_assembly(clean, line_num, ops)
    }
}

/// Returns true if a statement is a benign C++ declaration or setup invocation.
fn is_ignorable_declaration(s: &str) -> bool {
    const PREFIXES: &[&str] = &[
        "Adafruit_SSD1306",
        "LiquidCrystal_I2C",
        "LiquidCrystal",
        "Wire.begin",
        "pinMode(",
        "Serial.begin(",
        "int ",
        "float ",
        "double ",
        "char ",
        "bool ",
        "long ",
        "unsigned ",
        "const ",
        "for (;;)",
        "for(;;)",
        "while (1)",
        "while(1)",
        "while (true)",
        "while(true)",
        "for (;;);",
        "for(;;);",
        "while (1);",
        "while(1);",
        "while (true);",
        "while(true);",
    ];
    if PREFIXES.iter().any(|&p| s.starts_with(p)) {
        return true;
    }
    let lower = s.to_ascii_lowercase();
    lower.starts_with("display.begin")
        || lower.starts_with("oled.begin")
        || lower.starts_with("if (!display.begin")
        || lower.starts_with("if (!oled.begin")
        || lower.starts_with("if (display.begin")
        || lower.starts_with("if (oled.begin")
        || lower.starts_with("display.settext")
        || lower.starts_with("oled.settext")
        || lower.starts_with("display.setcursor")
        || lower.starts_with("oled.setcursor")
        || lower.starts_with("display.display")
        || lower.starts_with("oled.display")
        || lower.starts_with("lcd.init")
        || lower.starts_with("lcd.begin")
        || lower.starts_with("lcd.backlight")
        || lower.starts_with("lcd.nobacklight")
}

/// Parses a display.print or display.println call.
fn parse_oled_print(
    s: &str,
    line_num: usize,
    ops: &mut Vec<IntermediateOp>,
) -> Result<(), CompileError> {
    let lower = s.to_ascii_lowercase();
    let fn_name = if lower.contains(".println(") {
        if lower.starts_with("oled.") {
            "oled.println"
        } else {
            "display.println"
        }
    } else if lower.starts_with("oled.") {
        "oled.print"
    } else {
        "display.print"
    };
    let inner = extract_arguments(s, fn_name, line_num)?;
    let message = inner.trim().trim_matches('"').to_string();
    ops.push(IntermediateOp::OledPrint(message));
    Ok(())
}

/// Parses a digitalWrite instruction.
fn parse_digital_write(
    s: &str,
    line_num: usize,
    ops: &mut Vec<IntermediateOp>,
) -> Result<(), CompileError> {
    let inner = extract_arguments(s, "digitalWrite", line_num)?;
    let parts: Vec<&str> = inner.split(',').map(str::trim).collect();
    if parts.len() != 2 {
        return Err(CompileError::new(
            line_num,
            "digitalWrite requires pin and state arguments",
        ));
    }

    let pin: u8 = parts[0]
        .parse()
        .map_err(|_| CompileError::new(line_num, "Invalid pin number in digitalWrite"))?;
    let high = parts[1].eq_ignore_ascii_case("HIGH") || parts[1] == "1";

    if high {
        ops.push(IntermediateOp::SetHigh(pin));
    } else {
        ops.push(IntermediateOp::SetLow(pin));
    }
    Ok(())
}

/// Parses a delay instruction.
fn parse_delay(
    s: &str,
    line_num: usize,
    ops: &mut Vec<IntermediateOp>,
) -> Result<(), CompileError> {
    let inner = extract_arguments(s, "delay", line_num)?;
    let duration: u32 = inner
        .parse()
        .map_err(|_| CompileError::new(line_num, "Invalid delay argument"))?;
    ops.push(IntermediateOp::Delay(duration.min(u16::MAX as u32) as u16));
    Ok(())
}

/// Parses a Serial.print or Serial.println call.
fn parse_serial_print(
    s: &str,
    line_num: usize,
    ops: &mut Vec<IntermediateOp>,
    newline: bool,
) -> Result<(), CompileError> {
    let fn_name = if newline {
        "Serial.println"
    } else {
        "Serial.print"
    };
    let inner = extract_arguments(s, fn_name, line_num)?;
    let unquoted = inner.trim_matches('"');
    let mut msg = unquoted.to_string();
    if newline {
        msg.push('\n');
    }
    ops.push(IntermediateOp::UartString(msg));
    Ok(())
}

/// Parses an analogWrite or ledcWrite call.
fn parse_ledc_write(
    s: &str,
    line_num: usize,
    ops: &mut Vec<IntermediateOp>,
) -> Result<(), CompileError> {
    let fn_name = if s.starts_with("ledcWrite") {
        "ledcWrite"
    } else {
        "analogWrite"
    };
    let inner = extract_arguments(s, fn_name, line_num)?;
    let parts: Vec<&str> = inner.split(',').map(str::trim).collect();
    if parts.len() != 2 {
        return Err(CompileError::new(
            line_num,
            "PWM write requires pin/channel and duty arguments",
        ));
    }
    let pin: u8 = parts[0]
        .parse()
        .map_err(|_| CompileError::new(line_num, "Invalid pin in PWM write"))?;
    let duty: u8 = parts[1]
        .parse()
        .map_err(|_| CompileError::new(line_num, "Invalid duty cycle in PWM write"))?;
    ops.push(IntermediateOp::LedcWrite(pin, duty));
    Ok(())
}

/// Parses a WiFi.begin call.
fn parse_wifi_begin(
    s: &str,
    line_num: usize,
    ops: &mut Vec<IntermediateOp>,
) -> Result<(), CompileError> {
    let inner = extract_arguments(s, "WiFi.begin", line_num)?;
    let ssid = inner
        .split(',')
        .next()
        .unwrap_or("")
        .trim()
        .trim_matches('"');
    ops.push(IntermediateOp::WifiConnect(ssid.to_string()));
    Ok(())
}

/// Parses an LCD write such as `lcd.print("Hello")`.
fn parse_lcd_print(
    s: &str,
    line_num: usize,
    ops: &mut Vec<IntermediateOp>,
) -> Result<(), CompileError> {
    let inner = extract_arguments(s, "lcd.print", line_num)?;
    let message = inner.trim().trim_matches('"').to_string();
    ops.push(IntermediateOp::LcdPrint(message));
    Ok(())
}

/// Parses an `lcd.clear()` command.
fn parse_lcd_clear(
    s: &str,
    line_num: usize,
    ops: &mut Vec<IntermediateOp>,
) -> Result<(), CompileError> {
    extract_arguments(s, "lcd.clear", line_num)?;
    ops.push(IntermediateOp::LcdClear);
    Ok(())
}

/// Parses an `lcd.setCursor(col, row)` command.
fn parse_lcd_set_cursor(
    s: &str,
    line_num: usize,
    ops: &mut Vec<IntermediateOp>,
) -> Result<(), CompileError> {
    let inner = extract_arguments(s, "lcd.setCursor", line_num)?;
    let mut parts = inner.split(',');
    let col_str = parts.next().unwrap_or("0").trim();
    let row_str = parts.next().unwrap_or("0").trim();
    let col: u8 = col_str.parse().unwrap_or(0);
    let row: u8 = row_str.parse().unwrap_or(0);
    ops.push(IntermediateOp::LcdSetCursor(col, row));
    Ok(())
}

/// Extracts arguments enclosed in parentheses for a given function call.
fn extract_arguments<'a>(
    s: &'a str,
    fn_name: &str,
    line_num: usize,
) -> Result<&'a str, CompileError> {
    let open = s
        .find('(')
        .ok_or_else(|| CompileError::new(line_num, format!("Missing '(' in {}", fn_name)))?;
    let close = s
        .rfind(')')
        .ok_or_else(|| CompileError::new(line_num, format!("Missing ')' in {}", fn_name)))?;
    if close <= open {
        return Err(CompileError::new(
            line_num,
            format!("Mismatched parentheses in {}", fn_name),
        ));
    }
    Ok(&s[open + 1..close])
}

/// Parses an assembly mnemonic line.
fn parse_assembly(
    line: &str,
    line_num: usize,
    ops: &mut Vec<IntermediateOp>,
) -> Result<(), CompileError> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.is_empty() {
        return Ok(());
    }

    match parts[0].to_ascii_uppercase().as_str() {
        "SET_HIGH" if parts.len() > 1 => {
            let p: u8 = parts[1]
                .parse()
                .map_err(|_| CompileError::new(line_num, "Invalid pin for SET_HIGH"))?;
            ops.push(IntermediateOp::SetHigh(p));
        }
        "SET_LOW" if parts.len() > 1 => {
            let p: u8 = parts[1]
                .parse()
                .map_err(|_| CompileError::new(line_num, "Invalid pin for SET_LOW"))?;
            ops.push(IntermediateOp::SetLow(p));
        }
        "DELAY" if parts.len() > 1 => {
            let d: u16 = parts[1]
                .parse()
                .map_err(|_| CompileError::new(line_num, "Invalid count for DELAY"))?;
            ops.push(IntermediateOp::Delay(d));
        }
        "JUMP" if parts.len() > 1 => {
            ops.push(IntermediateOp::Jump(parts[1].to_string()));
        }
        "PWM" if parts.len() > 2 => {
            let p: u8 = parts[1]
                .parse()
                .map_err(|_| CompileError::new(line_num, "Invalid pin for PWM"))?;
            let d: u8 = parts[2]
                .parse()
                .map_err(|_| CompileError::new(line_num, "Invalid duty for PWM"))?;
            ops.push(IntermediateOp::LedcWrite(p, d));
        }
        "PRINT" if parts.len() > 1 => {
            let text = line[parts[0].len()..].trim().trim_matches('"');
            ops.push(IntermediateOp::UartString(text.to_string()));
        }
        "WIFI" if parts.len() > 1 => {
            let ssid = line[parts[0].len()..].trim().trim_matches('"');
            ops.push(IntermediateOp::WifiConnect(ssid.to_string()));
        }
        "HALT" => {
            ops.push(IntermediateOp::Halt);
        }
        unknown => {
            return Err(CompileError::new(
                line_num,
                format!("Unrecognized instruction or syntax: '{}'", unknown),
            ));
        }
    }
    Ok(())
}
