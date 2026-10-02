/// A single source code file/tab in the in-app code editor.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CodeTab {
    pub title: String,
    pub source: String,
}

impl CodeTab {
    /// Creates a new code tab with title and initial source code.
    pub fn new(title: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            source: source.into(),
        }
    }
}

/// Target microcontroller instance to receive compiled firmware flashes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum EditorTarget {
    #[default]
    None,
    Esp32(usize),
    Mcu(usize),
    Avr(usize),
}

/// Active section of the firmware studio window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum EditorSection {
    #[default]
    Code,
    Libraries,
}

/// State for the integrated Arduino Library Manager panel.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct LibraryPanelState {
    pub query: String,
    #[serde(default)]
    pub results: Vec<crate::toolchain::LibraryInfo>,
    #[serde(default)]
    pub installed: Vec<crate::toolchain::LibraryInfo>,
    #[serde(default)]
    pub updatable: Vec<String>,
    pub status: String,
    pub status_is_error: bool,
    pub loaded: bool,
    #[serde(skip)]
    pub is_busy: bool,
    #[serde(skip)]
    pub busy_text: String,
}

/// Active tab in the bottom monitor console panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum MonitorTab {
    #[default]
    Serial,
    Build,
}

/// Asynchronous firmware compilation job outcome.
pub struct BuildJobResult {
    pub target_index: usize,
    pub binary: Result<Vec<u8>, String>,
    pub build_output: String,
}

/// Asynchronous outcome of background Arduino library operations.
pub enum LibraryJobResult {
    InstallDone {
        name: String,
        result: Result<String, String>,
        installed: Vec<crate::toolchain::LibraryInfo>,
        updatable: Vec<String>,
    },
    SearchDone {
        results: Result<Vec<crate::toolchain::LibraryInfo>, String>,
    },
    UpdateIndexDone {
        result: Result<String, String>,
        installed: Vec<crate::toolchain::LibraryInfo>,
        updatable: Vec<String>,
    },
    RefreshDone {
        installed: Vec<crate::toolchain::LibraryInfo>,
        updatable: Vec<String>,
    },
}

/// State tracking for the embedded code editor and serial terminal.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CodeEditorState {
    pub open: bool,
    pub active_tab: usize,
    pub tabs: Vec<CodeTab>,
    pub target: EditorTarget,
    pub status_message: String,
    pub status_is_error: bool,
    pub serial_input: String,
    pub show_serial_monitor: bool,
    #[serde(default)]
    pub section: EditorSection,
    #[serde(default)]
    pub library: LibraryPanelState,
    #[serde(default)]
    pub cursor_line: usize,
    #[serde(default)]
    pub cursor_col: usize,
    #[serde(default)]
    pub monitor_tab: MonitorTab,
    #[serde(default)]
    pub build_log: String,
    #[serde(skip)]
    pub build_in_progress: bool,
}

impl Default for CodeEditorState {
    fn default() -> Self {
        let default_esp32_sketch = r#"// ESP32 Blink & Serial Demo
void setup() {
    pinMode(2, OUTPUT);
    Serial.println("ESP32 Initialized!");
    WiFi.begin("HomeNetwork");
}

void loop() {
    digitalWrite(2, HIGH);
    Serial.println("LED ON");
    delay(500);
    digitalWrite(2, LOW);
    Serial.println("LED OFF");
    delay(500);
}
"#;

        Self {
            open: false,
            active_tab: 0,
            tabs: vec![CodeTab::new("esp32_blink.ino", default_esp32_sketch)],
            target: EditorTarget::None,
            status_message: "Ready to compile and flash.".to_string(),
            status_is_error: false,
            serial_input: String::new(),
            show_serial_monitor: true,
            section: EditorSection::Code,
            library: LibraryPanelState::default(),
            cursor_line: 1,
            cursor_col: 1,
            monitor_tab: MonitorTab::Serial,
            build_log: String::new(),
            build_in_progress: false,
        }
    }
}
