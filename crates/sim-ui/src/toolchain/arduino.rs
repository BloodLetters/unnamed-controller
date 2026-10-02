//! Arduino CLI compiler driver and sketchbook integration.

use super::library::{LibraryInfo, parse_installed_libraries, parse_search_results};
use super::traits::{FirmwareCompiler, parse_intel_hex};

/// Compiler driver that shells out to `arduino-cli`.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
pub struct ArduinoCliCompiler {
    pub cli_path: String,
    pub board_fqbn: String,
    pub sketch_name: String,
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for ArduinoCliCompiler {
    fn default() -> Self {
        Self {
            cli_path: resolve_cli_path(),
            board_fqbn: "arduino:avr:uno".to_string(),
            sketch_name: "sketch".to_string(),
        }
    }
}

/// Locates the `arduino-cli` executable, falling back to PATH lookup by name.
#[cfg(not(target_arch = "wasm32"))]
pub fn resolve_cli_path() -> String {
    #[cfg(windows)]
    {
        let mut candidates: Vec<std::path::PathBuf> = Vec::new();
        if let Ok(program_files) = std::env::var("ProgramFiles") {
            candidates.push(
                std::path::Path::new(&program_files)
                    .join("Arduino CLI")
                    .join("arduino-cli.exe"),
            );
        }
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            candidates.push(
                std::path::Path::new(&local)
                    .join("Programs")
                    .join("Arduino CLI")
                    .join("arduino-cli.exe"),
            );
            candidates.push(
                std::path::Path::new(&local)
                    .join("Microsoft")
                    .join("WinGet")
                    .join("Links")
                    .join("arduino-cli.exe"),
            );
        }
        if let Ok(profile) = std::env::var("USERPROFILE") {
            candidates.push(
                std::path::Path::new(&profile)
                    .join("scoop")
                    .join("shims")
                    .join("arduino-cli.exe"),
            );
        }
        candidates.push(std::path::PathBuf::from(
            "C:\\ProgramData\\chocolatey\\bin\\arduino-cli.exe",
        ));

        for candidate in candidates {
            if candidate.exists() {
                return candidate.to_string_lossy().into_owned();
            }
        }
    }
    "arduino-cli".to_string()
}

#[cfg(not(target_arch = "wasm32"))]
impl ArduinoCliCompiler {
    /// Returns true when the CLI is installed and responds to `version`.
    pub fn is_available(&self) -> bool {
        std::process::Command::new(&self.cli_path)
            .arg("version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Constructs a compiler instance targeting the ESP32-S3 board FQBN.
    pub fn for_esp32_s3() -> Self {
        Self {
            cli_path: resolve_cli_path(),
            board_fqbn: "esp32:esp32:esp32s3".to_string(),
            sketch_name: "sketch".to_string(),
        }
    }

    /// Runs the CLI and returns its stdout, or the trimmed stderr as an error.
    fn run(&self, args: &[&str]) -> Result<String, String> {
        let output = std::process::Command::new(&self.cli_path)
            .args(args)
            .output()
            .map_err(|error| format!("arduino-cli tidak dapat dijalankan: {error}"))?;
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
        }
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }

    /// Searches the Library Manager for libraries matching `query`.
    pub fn search_libraries(&self, query: &str) -> Result<Vec<LibraryInfo>, String> {
        let json = self.run(&["lib", "search", query, "--format", "json"])?;
        Ok(parse_search_results(&json))
    }

    /// Lists libraries already installed in the sketchbook (auto-detected on build).
    pub fn installed_libraries(&self) -> Result<Vec<LibraryInfo>, String> {
        let json = self.run(&["lib", "list", "--format", "json"])?;
        Ok(parse_installed_libraries(&json))
    }

    /// Lists names of installed libraries that have a newer release available.
    pub fn updatable_libraries(&self) -> Result<Vec<String>, String> {
        let json = self.run(&["lib", "list", "--updatable", "--format", "json"])?;
        Ok(parse_installed_libraries(&json)
            .into_iter()
            .map(|library| library.name)
            .collect())
    }

    /// Installs a library (optionally pinned to a version) into the sketchbook.
    pub fn install_library(&self, name: &str, version: Option<&str>) -> Result<String, String> {
        let spec = match version {
            Some(version) if !version.is_empty() => format!("{name}@{version}"),
            _ => name.to_string(),
        };
        self.run(&["lib", "install", &spec])
    }

    /// Refreshes the local Library Manager index.
    pub fn update_library_index(&self) -> Result<String, String> {
        self.run(&["lib", "update-index"])
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl FirmwareCompiler for ArduinoCliCompiler {
    fn compile(&self, source: &str) -> Result<Vec<u8>, String> {
        use std::process::Command;

        let base = std::env::temp_dir().join(format!("uc_sketch_{}", std::process::id()));
        let sketch_dir = base.join(&self.sketch_name);
        let build_dir = base.join("build");
        std::fs::create_dir_all(&sketch_dir).map_err(|error| error.to_string())?;
        std::fs::create_dir_all(&build_dir).map_err(|error| error.to_string())?;

        let ino_path = sketch_dir.join(format!("{}.ino", self.sketch_name));
        std::fs::write(&ino_path, source).map_err(|error| error.to_string())?;

        let output = Command::new(&self.cli_path)
            .arg("compile")
            .arg("--fqbn")
            .arg(&self.board_fqbn)
            .arg("--output-dir")
            .arg(&build_dir)
            .arg(&sketch_dir)
            .output()
            .map_err(|error| {
                format!(
                    "arduino-cli tidak dapat dijalankan: {error}. Pasang arduino-cli dan pastikan ada di PATH."
                )
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Kompilasi gagal:\n{}", stderr.trim()));
        }

        let bin_path = build_dir.join(format!("{}.ino.bin", self.sketch_name));
        if bin_path.exists() {
            return std::fs::read(&bin_path).map_err(|error| {
                format!("BIN tidak dapat dibaca di {}: {error}", bin_path.display())
            });
        }

        let hex_path = build_dir.join(format!("{}.ino.hex", self.sketch_name));
        let hex = std::fs::read_to_string(&hex_path).map_err(|error| {
            format!("HEX/BIN tidak ditemukan di {}: {error}", hex_path.display())
        })?;
        parse_intel_hex(&hex)
    }
}

/// WebAssembly placeholder: external toolchain integration is unavailable.
#[cfg(target_arch = "wasm32")]
#[derive(Debug, Clone)]
pub struct ArduinoCliCompiler {
    pub cli_path: String,
    pub board_fqbn: String,
    pub sketch_name: String,
}

#[cfg(target_arch = "wasm32")]
impl Default for ArduinoCliCompiler {
    fn default() -> Self {
        Self {
            cli_path: String::new(),
            board_fqbn: "arduino:avr:uno".to_string(),
            sketch_name: "sketch".to_string(),
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl ArduinoCliCompiler {
    /// Always false in the browser; precompile sketches to HEX instead.
    pub fn is_available(&self) -> bool {
        false
    }

    /// Constructs an ESP32-S3 compiler placeholder for WebAssembly.
    pub fn for_esp32_s3() -> Self {
        Self {
            cli_path: String::new(),
            board_fqbn: "esp32:esp32:esp32s3".to_string(),
            sketch_name: "sketch".to_string(),
        }
    }

    /// Reports that the Library Manager is unavailable in the browser.
    fn unavailable<T>() -> Result<T, String> {
        Err("Library Manager tidak tersedia di target WebAssembly.".to_string())
    }

    /// Searches the Library Manager; unavailable in the browser.
    pub fn search_libraries(&self, _query: &str) -> Result<Vec<LibraryInfo>, String> {
        Self::unavailable()
    }

    /// Lists installed libraries; unavailable in the browser.
    pub fn installed_libraries(&self) -> Result<Vec<LibraryInfo>, String> {
        Self::unavailable()
    }

    /// Lists updatable libraries; unavailable in the browser.
    pub fn updatable_libraries(&self) -> Result<Vec<String>, String> {
        Self::unavailable()
    }

    /// Installs a library; unavailable in the browser.
    pub fn install_library(&self, _name: &str, _version: Option<&str>) -> Result<String, String> {
        Self::unavailable()
    }

    /// Refreshes the Library Manager index; unavailable in the browser.
    pub fn update_library_index(&self) -> Result<String, String> {
        Self::unavailable()
    }
}

#[cfg(target_arch = "wasm32")]
impl FirmwareCompiler for ArduinoCliCompiler {
    fn compile(&self, _source: &str) -> Result<Vec<u8>, String> {
        Err(
            "Kompilasi Arduino tidak tersedia di target WebAssembly. Kompilasi ke .hex lebih dulu."
                .to_string(),
        )
    }
}
