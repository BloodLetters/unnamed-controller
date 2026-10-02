//! ESP-IDF compiler driver for ESP32-S3 firmware generation.

/// Compiler driver that builds an ESP-IDF project for the ESP32-S3 via `idf.py`.
///
/// ESP-IDF v6.x requires its Python environment and toolchain PATH to be active, so builds are
/// run through `export.ps1` rather than calling `idf.py` directly.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
pub struct EspIdfCompiler {
    pub export_ps1: std::path::PathBuf,
    pub board_target: String,
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for EspIdfCompiler {
    fn default() -> Self {
        Self {
            export_ps1: resolve_export_ps1().unwrap_or_default(),
            board_target: "esp32s3".to_string(),
        }
    }
}

/// Locates ESP-IDF's `export.ps1`, the script that activates the Python environment and toolchain.
#[cfg(not(target_arch = "wasm32"))]
pub fn resolve_export_ps1() -> Option<std::path::PathBuf> {
    let candidates = [
        std::env::var("ESP_IDF_EXPORT_PS1")
            .ok()
            .map(std::path::PathBuf::from),
        std::env::var("IDF_PATH")
            .ok()
            .map(|path| std::path::Path::new(&path).join("export.ps1")),
        Some(std::path::PathBuf::from(
            "C:\\esp\\v6.1\\esp-idf\\export.ps1",
        )),
        find_idf_py().map(|idf_py| {
            idf_py
                .parent()
                .and_then(|tools| tools.parent())
                .map(|root| root.join("export.ps1"))
                .unwrap_or_default()
        }),
    ];
    candidates
        .into_iter()
        .flatten()
        .find(|candidate| candidate.exists())
}

/// Finds `idf.py` on PATH, which also reveals the ESP-IDF root.
#[cfg(not(target_arch = "wasm32"))]
fn find_idf_py() -> Option<std::path::PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|dir| dir.join("idf.py"))
        .find(|candidate| candidate.exists())
}

#[cfg(not(target_arch = "wasm32"))]
impl EspIdfCompiler {
    /// Returns true when ESP-IDF's activating script was found.
    pub fn is_available(&self) -> bool {
        self.export_ps1.exists()
    }

    /// Generates a minimal ESP-IDF project around `source` and returns the merged flash image.
    pub fn build(&self, source: &str) -> Result<Vec<u8>, String> {
        if !self.is_available() {
            return Err(
                "export.ps1 ESP-IDF tidak ditemukan. Pasang ESP-IDF lalu pastikan idf.py ada di PATH, \
                 atau set IDF_PATH / ESP_IDF_EXPORT_PS1."
                    .to_string(),
            );
        }

        let base = std::env::temp_dir().join(format!("uc_idf_{}", std::process::id()));
        let main_dir = base.join("main");
        std::fs::create_dir_all(&main_dir).map_err(|error| error.to_string())?;

        std::fs::write(
            base.join("CMakeLists.txt"),
            "cmake_minimum_required(VERSION 3.16)\n\
             include($ENV{IDF_PATH}/tools/cmake/project.cmake)\n\
             project(uc_sketch)\n",
        )
        .map_err(|error| error.to_string())?;
        std::fs::write(
            main_dir.join("CMakeLists.txt"),
            "idf_component_register(SRCS \"main.c\")\n",
        )
        .map_err(|error| error.to_string())?;
        std::fs::write(main_dir.join("main.c"), source).map_err(|error| error.to_string())?;

        self.run(&base, &["set-target", &self.board_target])?;
        self.run(&base, &["build"])?;
        self.run(&base, &["merge-bin", "-o", "merged.bin"])?;

        let merged = base.join("build").join("merged.bin");
        std::fs::read(&merged).map_err(|error| format!("merged.bin tidak ditemukan: {error}"))
    }

    /// Activates ESP-IDF via profile/export script and runs `idf.py` inside the generated project.
    fn run(&self, dir: &std::path::Path, args: &[&str]) -> Result<(), String> {
        let profile =
            std::path::Path::new("C:\\Espressif\\tools\\Microsoft.v6.1.PowerShell_profile.ps1");
        let profile_init = if profile.exists() {
            format!("& '{}' | Out-Null; ", profile.display())
        } else {
            String::new()
        };
        let script = format!(
            "$ErrorActionPreference='Stop'; {}& '{}' | Out-Null; & idf.py {}",
            profile_init,
            self.export_ps1.display(),
            args.join(" ")
        );
        let output = std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                &script,
            ])
            .current_dir(dir)
            .output()
            .map_err(|error| format!("powershell tidak dapat dijalankan: {error}"))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let detail = if stderr.trim().is_empty() {
                stdout
            } else {
                stderr
            };
            return Err(detail.trim().to_string());
        }
        Ok(())
    }
}

/// Locates the ESP32-S3 mask ROM ELF (from `espressif/esp-rom-elfs`) required for a cold boot.
#[cfg(not(target_arch = "wasm32"))]
pub fn resolve_rom_elf() -> Option<std::path::PathBuf> {
    if let Ok(path) = std::env::var("ESP32SIM_ROM_ELF") {
        let candidate = std::path::PathBuf::from(path);
        if candidate.exists() {
            return Some(candidate);
        }
    }
    let direct_eim = std::path::PathBuf::from(
        "C:\\Espressif\\tools\\esp-rom-elfs\\20241011\\esp32s3_rev0_rom.elf",
    );
    if direct_eim.exists() {
        return Some(direct_eim);
    }
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .ok()?;
    let root = std::path::Path::new(&home)
        .join(".espressif")
        .join("tools")
        .join("esp-rom-elfs");
    for entry in std::fs::read_dir(&root).ok()?.flatten() {
        let candidate = entry.path().join("esp32s3_rev0_rom.elf");
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

/// WebAssembly placeholder: ESP-IDF is unavailable in the browser.
#[cfg(target_arch = "wasm32")]
#[derive(Debug, Clone, Default)]
pub struct EspIdfCompiler {
    pub export_ps1: std::path::PathBuf,
    pub board_target: String,
}

#[cfg(target_arch = "wasm32")]
impl EspIdfCompiler {
    /// Always false in the browser; precompile the firmware to a merged `.bin` instead.
    pub fn is_available(&self) -> bool {
        false
    }

    /// Unavailable in the browser.
    pub fn build(&self, _source: &str) -> Result<Vec<u8>, String> {
        Err("Kompilasi ESP-IDF tidak tersedia di target WebAssembly.".to_string())
    }
}

/// Mask ROM ELF lookup; unavailable in the browser.
#[cfg(target_arch = "wasm32")]
pub fn resolve_rom_elf() -> Option<std::path::PathBuf> {
    None
}
