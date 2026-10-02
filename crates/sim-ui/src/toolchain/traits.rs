//! Common traits and format decoders for microcontroller firmware.

/// Compiles Arduino/C++ source into a raw flash image.
pub trait FirmwareCompiler {
    /// Compiles the given sketch source, returning raw flash bytes.
    fn compile(&self, source: &str) -> Result<Vec<u8>, String>;
}

/// Parses an Intel HEX image into raw flash bytes.
pub fn parse_intel_hex(hex: &str) -> Result<Vec<u8>, String> {
    sim_core::firmware::hex::parse_hex_to_memory(hex, 0x8000)
        .map_err(|error| format!("Intel HEX error: {error:?}"))
}
