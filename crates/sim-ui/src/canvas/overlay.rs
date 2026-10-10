//! Floating canvas overlay widgets including zoom controls and component spawning banners.

use egui::{Color32, Rect, RichText, Stroke, Vec2};

use crate::app::SimulatorApp;
use crate::types::SpawningComponent;

/// Renders the floating bottom-right zoom control widget.
pub fn render_floating_zoom_control(ui: &mut egui::Ui, rect: Rect, app: &mut SimulatorApp) {
    let zoom_rect = Rect::from_min_size(rect.max - Vec2::new(140.0, 36.0), Vec2::new(130.0, 26.0));
    let painter = ui.painter();
    painter.rect_filled(
        zoom_rect,
        4.0,
        Color32::from_rgba_premultiplied(24, 26, 32, 230),
    );
    painter.rect_stroke(
        zoom_rect,
        4.0,
        Stroke::new(1.0_f32, Color32::from_rgb(45, 48, 56)),
    );

    let mut zoom_ui = ui.child_ui(zoom_rect, egui::Layout::left_to_right(egui::Align::Center));
    zoom_ui.add_space(4.0);
    if zoom_ui.button(RichText::new("-").strong()).clicked() {
        app.zoom = (app.zoom / 1.15).max(0.2);
    }
    zoom_ui.label(RichText::new(format!("{:.0}%", app.zoom * 100.0)).size(10.5));
    if zoom_ui.button(RichText::new("+").strong()).clicked() {
        app.zoom = (app.zoom * 1.15).min(5.0);
    }
}

/// Renders a top-centered placement banner when a component is armed for placement.
pub fn render_spawning_banner(ui: &mut egui::Ui, rect: Rect, app: &mut SimulatorApp) {
    if app.spawning == SpawningComponent::None {
        return;
    }
    let banner_rect = Rect::from_center_size(
        Rect::from_min_size(rect.min, Vec2::new(rect.width(), 32.0)).center(),
        Vec2::new(320.0, 26.0),
    );
    let painter = ui.painter();
    painter.rect_filled(banner_rect, 4.0, Color32::from_rgb(13, 153, 255));
    painter.text(
        banner_rect.center(),
        egui::Align2::CENTER_CENTER,
        "Click canvas to place component (Esc to cancel)",
        egui::FontId::proportional(11.0),
        Color32::WHITE,
    );
}
