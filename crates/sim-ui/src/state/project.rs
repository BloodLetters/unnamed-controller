use std::path::Path;

use crate::app::SimulatorApp;

/// Resets the workspace to an empty project, keeping the run switch untouched.
pub fn new_project(app: &mut SimulatorApp) {
    let power_on = app.power_on;
    *app = SimulatorApp::default();
    app.power_on = power_on;
    app.project_message = Some("New project created.".to_string());
}

/// Saves the current workspace, prompting for a path when none is known or when forced.
#[cfg(not(target_arch = "wasm32"))]
pub fn save_project(app: &mut SimulatorApp, force_prompt: bool) {
    let path = if force_prompt || app.project_path.is_none() {
        match rfd::FileDialog::new()
            .add_filter("Circuit Project", &["json"])
            .set_file_name("circuit.json")
            .save_file()
        {
            Some(path) => path,
            None => return,
        }
    } else {
        match app.project_path.clone() {
            Some(path) => path,
            None => return,
        }
    };

    let result = crate::state::save_project_json(&app.snapshot())
        .and_then(|json| std::fs::write(&path, json).map_err(|error| error.to_string()));

    match result {
        Ok(()) => {
            app.project_message = Some(format!("Saved {}", display_name(&path)));
            app.project_path = Some(path);
        }
        Err(error) => app.project_message = Some(format!("Save failed: {error}")),
    }
}

/// Opens a project file chosen by the user and restores it into the workspace.
#[cfg(not(target_arch = "wasm32"))]
pub fn open_project(app: &mut SimulatorApp) {
    let Some(path) = rfd::FileDialog::new()
        .add_filter("Circuit Project", &["json"])
        .pick_file()
    else {
        return;
    };

    let result = std::fs::read_to_string(&path)
        .map_err(|error| error.to_string())
        .and_then(|text| crate::state::load_project_json(&text));

    match result {
        Ok(data) => {
            app.restore_snapshot(data);
            app.project_message = Some(format!("Opened {}", display_name(&path)));
            app.project_path = Some(path);
        }
        Err(error) => app.project_message = Some(format!("Open failed: {error}")),
    }
}

/// Returns a path's file name for user-facing status messages.
#[cfg(not(target_arch = "wasm32"))]
fn display_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string())
}

/// File dialogs are unavailable in the browser build, so saving is a no-op there.
#[cfg(target_arch = "wasm32")]
pub fn save_project(_app: &mut SimulatorApp, _force_prompt: bool) {}

/// File dialogs are unavailable in the browser build, so opening is a no-op there.
#[cfg(target_arch = "wasm32")]
pub fn open_project(_app: &mut SimulatorApp) {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Wire;
    use sim_core::netlist::PinId;

    #[test]
    fn new_project_clears_the_workspace() {
        let mut app = SimulatorApp::default();
        app.wires.push(Wire::new(PinId(1), PinId(2)));
        assert!(!app.wires.is_empty());

        new_project(&mut app);
        assert!(app.wires.is_empty());
        assert!(app.project_path.is_none());
    }

    #[test]
    fn project_round_trips_through_a_file() {
        let mut app = SimulatorApp::default();
        app.wires.push(Wire::new(PinId(1), PinId(2)));

        let json = crate::state::save_project_json(&app.snapshot()).expect("serialize");
        let path = std::env::temp_dir().join("uc_project_roundtrip.json");
        std::fs::write(&path, json).expect("write project");

        let mut loaded = SimulatorApp::default();
        let text = std::fs::read_to_string(&path).expect("read project");
        let data = crate::state::load_project_json(&text).expect("parse project");
        loaded.restore_snapshot(data);

        assert_eq!(loaded.wires.len(), 1);
        let _ = std::fs::remove_file(path);
    }
}
