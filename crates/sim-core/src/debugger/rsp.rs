use crate::debugger::types::CpuRegisters;

/// Represents a parsed GDB Remote Serial Protocol command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RspCommand {
    QuerySupported,
    HaltReason,
    ReadRegisters,
    WriteRegisters(Vec<u32>),
    ReadMemory { addr: u32, len: usize },
    WriteMemory { addr: u32, data: Vec<u8> },
    InsertBreakpoint { addr: u32 },
    RemoveBreakpoint { addr: u32 },
    InsertWatchpoint { addr: u32, is_write: bool },
    RemoveWatchpoint { addr: u32, is_write: bool },
    Step,
    Continue,
    Detach,
    Kill,
    Unknown(String),
}

/// Encapsulates a payload into a framed GDB RSP packet with checksum (`$<payload>#<checksum>`).
pub fn encode_packet(payload: &str) -> String {
    let sum = payload
        .as_bytes()
        .iter()
        .fold(0u8, |acc, &b| acc.wrapping_add(b));
    format!("${}#{:02x}", payload, sum)
}

/// Decodes and validates the checksum of a framed GDB RSP packet.
pub fn decode_packet(raw: &str) -> Option<&str> {
    let trimmed = raw.trim();
    if !trimmed.starts_with('$') {
        return None;
    }

    let hash_idx = trimmed.rfind('#')?;
    let payload = &trimmed[1..hash_idx];
    let expected_csum_str = &trimmed[hash_idx + 1..];

    let calculated = payload
        .as_bytes()
        .iter()
        .fold(0u8, |acc, &b| acc.wrapping_add(b));
    let expected = u8::from_str_radix(expected_csum_str, 16).ok()?;

    if calculated == expected {
        Some(payload)
    } else {
        None
    }
}

/// Parses an un-framed GDB RSP packet payload into a strongly-typed command.
pub fn parse_command(payload: &str) -> RspCommand {
    if payload.starts_with("qSupported") {
        return RspCommand::QuerySupported;
    }
    if payload == "?" {
        return RspCommand::HaltReason;
    }
    if payload == "g" {
        return RspCommand::ReadRegisters;
    }
    if payload == "s" {
        return RspCommand::Step;
    }
    if payload == "c" {
        return RspCommand::Continue;
    }
    if payload == "D" {
        return RspCommand::Detach;
    }
    if payload == "k" {
        return RspCommand::Kill;
    }

    if let Some(rest) = payload.strip_prefix('m') {
        let parts: Vec<&str> = rest.split(',').collect();
        if parts.len() == 2
            && let Ok(addr) = u32::from_str_radix(parts[0], 16)
            && let Ok(len) = usize::from_str_radix(parts[1], 16)
        {
            return RspCommand::ReadMemory { addr, len };
        }
    }

    if let Some(rest) = payload.strip_prefix('M') {
        let colon_parts: Vec<&str> = rest.split(':').collect();
        if colon_parts.len() == 2 {
            let addr_parts: Vec<&str> = colon_parts[0].split(',').collect();
            if addr_parts.len() == 2
                && let Ok(addr) = u32::from_str_radix(addr_parts[0], 16)
            {
                let hex_str = colon_parts[1];
                let mut data = Vec::new();
                for i in (0..hex_str.len()).step_by(2) {
                    if i + 2 <= hex_str.len()
                        && let Ok(b) = u8::from_str_radix(&hex_str[i..i + 2], 16)
                    {
                        data.push(b);
                    }
                }
                return RspCommand::WriteMemory { addr, data };
            }
        }
    }

    if payload.starts_with('Z') || payload.starts_with('z') {
        let is_insert = payload.starts_with('Z');
        let rest = &payload[1..];
        let parts: Vec<&str> = rest.split(',').collect();
        if parts.len() >= 2
            && let Ok(addr) = u32::from_str_radix(parts[1], 16)
        {
            let type_char = parts[0];
            return match (is_insert, type_char) {
                (true, "0") | (true, "1") => RspCommand::InsertBreakpoint { addr },
                (false, "0") | (false, "1") => RspCommand::RemoveBreakpoint { addr },
                (true, "2") => RspCommand::InsertWatchpoint {
                    addr,
                    is_write: true,
                },
                (false, "2") => RspCommand::RemoveWatchpoint {
                    addr,
                    is_write: true,
                },
                (true, "3") => RspCommand::InsertWatchpoint {
                    addr,
                    is_write: false,
                },
                (false, "3") => RspCommand::RemoveWatchpoint {
                    addr,
                    is_write: false,
                },
                _ => RspCommand::Unknown(payload.to_string()),
            };
        }
    }

    RspCommand::Unknown(payload.to_string())
}

/// Serializes CPU registers into GDB hex byte stream format.
pub fn format_registers(regs: &CpuRegisters) -> String {
    let mut out = String::new();
    for r in &regs.r {
        for b in r.to_le_bytes() {
            out.push_str(&format!("{:02x}", b));
        }
    }
    for b in regs.pc.to_le_bytes() {
        out.push_str(&format!("{:02x}", b));
    }
    for b in regs.sp.to_le_bytes() {
        out.push_str(&format!("{:02x}", b));
    }
    out.push_str(&format!("{:02x}", regs.flags));
    out
}

/// Serializes memory byte data into GDB hex string representation.
pub fn format_memory_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rsp_encode_decode_checksum() {
        let encoded = encode_packet("OK");
        assert_eq!(encoded, "$OK#9a");
        let decoded = decode_packet(&encoded);
        assert_eq!(decoded, Some("OK"));
    }

    #[test]
    fn test_rsp_parse_commands() {
        assert_eq!(parse_command("?"), RspCommand::HaltReason);
        assert_eq!(parse_command("s"), RspCommand::Step);
        assert_eq!(parse_command("c"), RspCommand::Continue);
        assert_eq!(
            parse_command("m100,4"),
            RspCommand::ReadMemory {
                addr: 0x100,
                len: 4
            }
        );
        assert_eq!(
            parse_command("Z0,200,2"),
            RspCommand::InsertBreakpoint { addr: 0x200 }
        );
        assert_eq!(
            parse_command("z2,300,4"),
            RspCommand::RemoveWatchpoint {
                addr: 0x300,
                is_write: true
            }
        );
    }
}
