//! Canvas renderer for MQ-135 air quality gas sensor modules.

use egui::{Color32, FontId, Painter, Pos2, Rect, Stroke, Vec2};
use sim_components::component::Mq135;

/// Width of the sensor PCB module body in canvas units.
pub const MQ135_BODY_WIDTH: f32 = 48.0;
/// Height of the sensor PCB module body in canvas units.
pub const MQ135_BODY_HEIGHT: f32 = 52.0;
/// Horizontal spacing between adjacent connector lead pins.
pub const MQ135_PIN_PITCH: f32 = 10.0;
/// Vertical coordinate offset from module center down to terminal pins.
pub const MQ135_PIN_OFFSET_Y: f32 = 34.0;

/// Returns relative coordinate offsets for VCC, GND, AOUT, and DOUT terminals from component center.
pub fn mq135_pin_offsets() -> [(f32, f32); 4] {
    [
        (-1.5 * MQ135_PIN_PITCH, MQ135_PIN_OFFSET_Y),
        (-0.5 * MQ135_PIN_PITCH, MQ135_PIN_OFFSET_Y),
        (0.5 * MQ135_PIN_PITCH, MQ135_PIN_OFFSET_Y),
        (1.5 * MQ135_PIN_PITCH, MQ135_PIN_OFFSET_Y),
    ]
}

/// Renders the cylindrical aluminum-cased sensing element on the upper face of the PCB.
fn draw_mq135_sensing_element(painter: &Painter, center: Pos2, zoom: f32, rot: u16) {
    let element_offset = crate::canvas::rotate_offset(Vec2::new(0.0, -10.0), rot) * zoom;
    let element_center = center + element_offset;
    let outer_radius = 13.0 * zoom;
    let inner_radius = 9.5 * zoom;

    let shell_color = Color32::from_rgb(205, 195, 140);
    let mesh_color = Color32::from_rgb(175, 160, 100);
    let core_color = Color32::from_rgb(140, 100, 60);

    painter.circle_filled(element_center, outer_radius, shell_color);
    painter.circle_stroke(
        element_center,
        outer_radius,
        Stroke::new(1.2 * zoom, Color32::from_rgb(160, 148, 88)),
    );
    painter.circle_filled(element_center, inner_radius, mesh_color);
    painter.circle_filled(element_center, 4.5 * zoom, core_color);

    for i in 0..6 {
        let angle = i as f32 * std::f32::consts::TAU / 6.0;
        let spoke_end = element_center + Vec2::new(angle.cos(), angle.sin()) * inner_radius * 0.75;
        painter.line_segment(
            [element_center, spoke_end],
            Stroke::new(0.8 * zoom, Color32::from_rgb(120, 88, 50)),
        );
    }
}

/// Renders the green FR4 PCB substrate with board outline and copper pads.
fn draw_mq135_pcb(painter: &Painter, center: Pos2, zoom: f32, rot: u16, is_selected: bool) {
    let border_color = if is_selected {
        Color32::from_rgb(0, 229, 255)
    } else {
        Color32::from_rgb(30, 100, 50)
    };
    let border_width = if is_selected { 2.0 } else { 1.2 };

    let raw_size = if rot == 90 || rot == 270 {
        Vec2::new(MQ135_BODY_HEIGHT, MQ135_BODY_WIDTH) * zoom
    } else {
        Vec2::new(MQ135_BODY_WIDTH, MQ135_BODY_HEIGHT) * zoom
    };
    let body_rect = Rect::from_center_size(center, raw_size);
    painter.rect_filled(body_rect, 3.0 * zoom, Color32::from_rgb(28, 90, 42));
    painter.rect_stroke(
        body_rect,
        3.0 * zoom,
        Stroke::new(border_width * zoom, border_color),
    );
}

/// Renders silkscreen product label and live analog readout on the PCB face.
fn draw_mq135_labels(painter: &Painter, center: Pos2, zoom: f32, rot: u16, sensor: &Mq135) {
    let title_pos = center + crate::canvas::rotate_offset(Vec2::new(0.0, 16.0), rot) * zoom;
    painter.text(
        title_pos,
        egui::Align2::CENTER_CENTER,
        "MQ-135",
        FontId::proportional(7.0 * zoom),
        Color32::from_rgb(220, 230, 200),
    );

    let readout_center = center + crate::canvas::rotate_offset(Vec2::new(0.0, 26.0), rot) * zoom;
    let readout_size = if rot == 90 || rot == 270 {
        Vec2::new(10.0, 38.0) * zoom
    } else {
        Vec2::new(38.0, 10.0) * zoom
    };
    let readout_rect = Rect::from_center_size(readout_center, readout_size);
    painter.rect_filled(readout_rect, 1.5 * zoom, Color32::from_rgb(16, 24, 18));

    let (ppm_text, ppm_color) = if sensor.is_powered() {
        let alert = sensor.digital_output_active();
        let color = if alert {
            Color32::from_rgb(255, 120, 60)
        } else {
            Color32::from_rgb(80, 230, 140)
        };
        (format!("{:.0}ppm", sensor.ppm()), color)
    } else {
        ("---ppm".to_string(), Color32::from_rgb(80, 92, 80))
    };

    painter.text(
        readout_center,
        egui::Align2::CENTER_CENTER,
        ppm_text,
        FontId::proportional(5.0 * zoom),
        ppm_color,
    );
}

/// Renders through-hole solder terminals with color-coded pin silkscreen identifiers.
fn draw_mq135_terminals(painter: &Painter, center: Pos2, zoom: f32, rot: u16) {
    let offsets = mq135_pin_offsets();
    let labels = ["VCC", "GND", "AO", "DO"];
    let label_colors = [
        Color32::from_rgb(248, 113, 113),
        Color32::from_rgb(130, 138, 150),
        Color32::from_rgb(251, 191, 36),
        Color32::from_rgb(96, 165, 250),
    ];

    for (i, &(ox, oy)) in offsets.iter().enumerate() {
        let terminal_pos = center + crate::canvas::rotate_offset(Vec2::new(ox, oy), rot) * zoom;
        painter.circle_filled(terminal_pos, 3.2 * zoom, Color32::from_rgb(212, 168, 68));
        painter.circle_filled(terminal_pos, 1.6 * zoom, Color32::from_rgb(12, 14, 18));

        let text_offset = crate::canvas::rotate_offset(Vec2::new(0.0, 6.8), rot) * zoom;
        painter.text(
            terminal_pos + text_offset,
            egui::Align2::CENTER_CENTER,
            labels[i],
            FontId::proportional(5.2 * zoom),
            label_colors[i],
        );
    }
}

/// Renders lead wires from the PCB edge down to solder terminal pads.
fn draw_mq135_leads(painter: &Painter, center: Pos2, zoom: f32, rot: u16) {
    let lead_stroke = Stroke::new(1.8 * zoom, Color32::from_rgb(180, 188, 198));
    let offsets = mq135_pin_offsets();

    for &(ox, oy) in &offsets {
        let p_start = center + crate::canvas::rotate_offset(Vec2::new(ox, 24.0), rot) * zoom;
        let p_end = center + crate::canvas::rotate_offset(Vec2::new(ox, oy), rot) * zoom;
        painter.line_segment([p_start, p_end], lead_stroke);
    }
}

/// Renders a single MQ-135 gas sensor module at the specified canvas position and rotation.
pub fn draw_single_mq135(
    painter: &Painter,
    center: Pos2,
    zoom: f32,
    rot: u16,
    comp: &Mq135,
    is_selected: bool,
) {
    draw_mq135_leads(painter, center, zoom, rot);
    draw_mq135_pcb(painter, center, zoom, rot, is_selected);
    draw_mq135_sensing_element(painter, center, zoom, rot);
    draw_mq135_labels(painter, center, zoom, rot, comp);
    draw_mq135_terminals(painter, center, zoom, rot);
}

/// Renders quick gas concentration preset buttons in the component context menu.
pub fn render_mq135_context_options(ui: &mut egui::Ui, comp: &mut Mq135) {
    ui.menu_button("u{1F4A8} Gas Presets", |ui| {
        for &(label, ppm) in &[
            ("Clean Air (400 ppm)", 400.0),
            ("Mild Pollution (600 ppm)", 600.0),
            ("High CO2 (800 ppm)", 800.0),
            ("Danger Level (1000 ppm)", 1000.0),
        ] {
            if ui.button(label).clicked() {
                comp.set_ppm(ppm);
                ui.close_menu();
            }
        }
    });
}
