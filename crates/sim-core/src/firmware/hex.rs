use std::num::ParseIntError;

/// Represents an error encountered while parsing an Intel HEX file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HexError {
    InvalidStartCode,
    ParseError(String),
    ChecksumMismatch { expected: u8, calculated: u8 },
    UnsupportedRecordType(u8),
}

impl From<ParseIntError> for HexError {
    fn from(err: ParseIntError) -> Self {
        HexError::ParseError(err.to_string())
    }
}

/// Represents a single parsed record (line) from an Intel HEX file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HexRecord {
    pub byte_count: u8,
    pub address: u16,
    pub record_type: u8,
    pub data: Vec<u8>,
    pub checksum: u8,
}

impl HexRecord {
    /// Parses a single line of an Intel HEX file.
    pub fn parse(line: &str) -> Result<Self, HexError> {
        let line = line.trim();
        if line.is_empty() {
            return Err(HexError::ParseError("Empty line".into()));
        }
        if !line.starts_with(':') {
            return Err(HexError::InvalidStartCode);
        }

        if line.len() < 11 {
            return Err(HexError::ParseError("Line too short".into()));
        }

        let byte_count = u8::from_str_radix(&line[1..3], 16)?;
        let address = u16::from_str_radix(&line[3..7], 16)?;
        let record_type = u8::from_str_radix(&line[7..9], 16)?;

        let expected_len = 11 + (byte_count as usize) * 2;
        if line.len() < expected_len {
            return Err(HexError::ParseError("Data length mismatch".into()));
        }

        let mut data = Vec::with_capacity(byte_count as usize);
        for i in 0..byte_count {
            let start = 9 + (i as usize) * 2;
            let byte = u8::from_str_radix(&line[start..start + 2], 16)?;
            data.push(byte);
        }

        let checksum = u8::from_str_radix(&line[expected_len - 2..expected_len], 16)?;

        let mut sum: u32 = byte_count as u32
            + (address >> 8) as u32
            + (address & 0xFF) as u32
            + record_type as u32;
        for &b in &data {
            sum += b as u32;
        }
        let calculated = ((!sum + 1) & 0xFF) as u8;

        if checksum != calculated {
            return Err(HexError::ChecksumMismatch {
                expected: checksum,
                calculated,
            });
        }

        Ok(Self {
            byte_count,
            address,
            record_type,
            data,
            checksum,
        })
    }
}

/// Parses a full Intel HEX file into a contiguous block of memory starting at 0x0000.
pub fn parse_hex_to_memory(hex_content: &str, max_memory: usize) -> Result<Vec<u8>, HexError> {
    let mut memory = vec![0u8; max_memory];
    let mut extended_segment_address: u32 = 0;
    let mut extended_linear_address: u32 = 0;

    for line in hex_content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let record = HexRecord::parse(line)?;

        match record.record_type {
            0x00 => {
                let base_address = extended_segment_address + extended_linear_address;
                let final_address = base_address + record.address as u32;

                for (i, &byte) in record.data.iter().enumerate() {
                    let addr = final_address as usize + i;
                    if addr < memory.len() {
                        memory[addr] = byte;
                    }
                }
            }
            0x01 => {
                break;
            }
            0x02 => {
                if record.data.len() != 2 {
                    return Err(HexError::ParseError(
                        "Invalid data length for type 02".into(),
                    ));
                }
                let segment = ((record.data[0] as u16) << 8) | (record.data[1] as u16);
                extended_segment_address = (segment as u32) * 16;
            }
            0x04 => {
                if record.data.len() != 2 {
                    return Err(HexError::ParseError(
                        "Invalid data length for type 04".into(),
                    ));
                }
                let linear = ((record.data[0] as u16) << 8) | (record.data[1] as u16);
                extended_linear_address = (linear as u32) << 16;
            }
            0x03 | 0x05 => {}
            _ => {
                return Err(HexError::UnsupportedRecordType(record.record_type));
            }
        }
    }

    Ok(memory)
}
