//! Arduino, AVR, and ESP-IDF toolchain integrations for compiling and loading firmware.

pub mod arduino;
pub mod esp_idf;
pub mod library;
pub mod traits;

pub use arduino::{ArduinoCliCompiler, resolve_cli_path};
pub use esp_idf::{EspIdfCompiler, resolve_export_ps1, resolve_rom_elf};
pub use library::{LibraryInfo, parse_installed_libraries, parse_search_results, version_is_newer};
pub use traits::{FirmwareCompiler, parse_intel_hex};

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    #[test]
    fn test_parse_intel_hex_data_record() {
        let hex = ":10000000000102030405060708090A0B0C0D0E0F78\n:00000001FF\n";
        let memory = parse_intel_hex(hex).expect("parse");
        assert_eq!(&memory[..16], &(0u8..16).collect::<Vec<u8>>()[..]);
    }

    #[test]
    fn test_parse_intel_hex_rejects_bad_checksum() {
        let hex = ":10000000000102030405060708090A0B0C0D0E0F00\n:00000001FF\n";
        assert!(parse_intel_hex(hex).is_err());
    }

    #[test]
    fn test_resolve_cli_path_is_never_empty() {
        assert!(!resolve_cli_path().is_empty());
    }

    #[test]
    fn test_version_is_newer() {
        assert!(version_is_newer("1.2.0", "1.1.9"));
        assert!(version_is_newer("2.0", "1.9.9"));
        assert!(!version_is_newer("1.0.0", "1.0.0"));
        assert!(!version_is_newer("1.0.0", "1.0.1"));
    }

    #[test]
    fn test_parse_search_results_picks_latest_release() {
        let json = r#"{"libraries":[{"name":"Servo","releases":{"1.0.0":{"sentence":"old","version":"1.0.0"},"1.1.0":{"sentence":"new","version":"1.1.0"}}}]}"#;
        let results = parse_search_results(json);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Servo");
        assert_eq!(results[0].version, "1.1.0");
        assert_eq!(results[0].sentence, "new");
    }

    #[test]
    fn test_parse_installed_libraries_accepts_nested_and_flat() {
        let nested = r#"{"installed_libraries":[{"library":{"name":"Servo","version":"1.1.0"}}]}"#;
        let flat = r#"{"installed_libraries":[{"name":"DHT","version":"1.4.6"}]}"#;
        assert_eq!(parse_installed_libraries(nested)[0].name, "Servo");
        assert_eq!(parse_installed_libraries(nested)[0].version, "1.1.0");
        assert_eq!(parse_installed_libraries(flat)[0].name, "DHT");
    }
}
