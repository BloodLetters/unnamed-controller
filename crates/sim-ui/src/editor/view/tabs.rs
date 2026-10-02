use egui::Ui;

use crate::app::SimulatorApp;
use crate::editor::types::CodeTab;

/// Renders the tab selection bar for open files.
pub fn render_tabs_bar(ui: &mut Ui, app: &mut SimulatorApp) {
    ui.horizontal(|ui| {
        let active = app.code_editor.active_tab;
        for i in 0..app.code_editor.tabs.len() {
            let title = app.code_editor.tabs[i].title.clone();
            let is_selected = active == i;
            if ui.selectable_label(is_selected, &title).clicked() {
                app.code_editor.active_tab = i;
            }
        }

        if ui.button("+ New Tab").clicked() {
            let next_id = app.code_editor.tabs.len() + 1;
            app.code_editor.tabs.push(CodeTab::new(
                format!("sketch{}.ino", next_id),
                "void loop() {\n    // Code here\n}\n",
            ));
            app.code_editor.active_tab = app.code_editor.tabs.len() - 1;
        }
    });
}

/// Renders the editable source code area with VS Code style syntax highlighting.
pub fn render_code_area(ui: &mut Ui, app: &mut SimulatorApp) {
    let active = app.code_editor.active_tab;

    let mut layouter = |ui: &Ui, text: &str, wrap_width: f32| {
        let mut job = crate::editor::highlight::highlight(text, ui.style());
        job.wrap.max_width = wrap_width;
        ui.fonts(|fonts| fonts.layout_job(job))
    };

    let viewport_height = ui.available_height();
    let row_height = ui.text_style_height(&egui::TextStyle::Monospace).max(1.0);
    let rows = ((viewport_height / row_height).floor() as usize).max(1);

    let mut cursor: Option<(usize, usize)> = None;
    if let Some(tab) = app.code_editor.tabs.get_mut(active) {
        egui::ScrollArea::both()
            .id_source("code_editor_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let output = egui::TextEdit::multiline(&mut tab.source)
                    .font(egui::TextStyle::Monospace)
                    .code_editor()
                    .desired_rows(rows)
                    .desired_width(f32::INFINITY)
                    .lock_focus(true)
                    .layouter(&mut layouter)
                    .show(ui);
                if let Some(range) = output.cursor_range {
                    cursor = Some(line_and_column(&tab.source, range.primary.ccursor.index));
                }
            });
    }

    if let Some((line, column)) = cursor {
        app.code_editor.cursor_line = line;
        app.code_editor.cursor_col = column;
    }
}

/// Converts a character index into a 1-based line and column pair.
fn line_and_column(text: &str, char_index: usize) -> (usize, usize) {
    let mut line = 1;
    let mut column = 1;
    for (count, ch) in text.chars().enumerate() {
        if count >= char_index {
            break;
        }
        if ch == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    (line, column)
}
