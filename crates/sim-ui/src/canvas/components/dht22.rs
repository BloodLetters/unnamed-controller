//! Canvas renderer for DHT22 (AM2302) digital humidity and temperature sensors.

use egui::{Color32, FontId, Painter, Pos2, Rect, Stroke, Vec2};
use sim_components::component::Dht22;

/// Width of the plastic sensor enclosure in canvas units.
pub const DHT22_BODY_WIDTH: f32 = 44.0;
/// Height of the plastic sensor enclosure in canvas units.
pub const DHT22_BODY_HEIGHT: f32 = 56.0;
/// Horizontal spacing between adjacent connector lead pins.
pub const DHT22_PIN_PITCH: f32 = 10.0;
/// Vertical coordinate offset from sensor center down to terminal pins.
pub const DHT22_PIN_OFFSET_Y: f32 = 36.0;

/// Returns relative coordinate offsets for VCC, SDA, NC, and GND terminals from component center.
pub fn dht22_pin_offsets() -> [(f32, f32); 4] {
    [
        (-1.5 * DHT22_PIN_PITCH, DHT22_PIN_OFFSET_Y),
        (-0.5 * DHT22_PIN_PITCH, DHT22_PIN_OFFSET_Y),
        (0.5 * DHT22_PIN_PITCH, DHT22_PIN_OFFSET_Y),
        (1.5 * DHT22_PIN_PITCH, DHT22_PIN_OFFSET_Y),
    ]
}

/// Renders tin-plated metallic lead wires extending from the plastic housing to solder terminals.
fn draw_dht22_leads(painter: &Painter, center: Pos2, zoom: f32, rot: u16) {
    let lead_stroke = Stroke::new(1.8 * zoom, Color32::from_rgb(180, 188, 198));
    let offsets = dht22_pin_offsets();

    for &(ox, oy) in &offsets {
        let p_start = center + crate::canvas::rotate_offset(Vec2::new(ox, 27.0), rot) * zoom;
        let p_end = center + crate::canvas::rotate_offset(Vec2::new(ox, oy), rot) * zoom;
        painter.line_segment([p_start, p_end], lead_stroke);
    }
}

/// Renders the top mounting eyelet tab used for chassis fastening.
fn draw_dht22_mounting_tab(painter: &Painter, center: Pos2, zoom: f32, rot: u16) {
    let tab_offset = crate::canvas::rotate_offset(Vec2::new(0.0, -31.0), rot) * zoom;
    let tab_center = center + tab_offset;
    let tab_color = Color32::from_rgb(228, 233, 238);
    let border_stroke = Stroke::new(1.0 * zoom, Color32::from_rgb(180, 190, 202));
    let eyelet_color = Color32::from_rgb(32, 38, 48);

    painter.circle_filled(tab_center, 4.5 * zoom, tab_color);
    painter.circle_stroke(tab_center, 4.5 * zoom, border_stroke);
    painter.circle_filled(tab_center, 2.0 * zoom, eyelet_color);
}

/// Renders the white perforated plastic casing with ventilation grilles and silkscreen lettering.
fn draw_dht22_body(
    painter: &Painter,
    center: Pos2,
    zoom: f32,
    rot: u16,
    dht: &Dht22,
    is_selected: bool,
) {
    let body_color = Color32::from_rgb(238, 242, 246);
    let border_color = if is_selected {
        Color32::from_rgb(0, 229, 255)
    } else {
        Color32::from_rgb(175, 185, 198)
    };
    let border_width = if is_selected { 2.0 } else { 1.2 };
    let border_stroke = Stroke::new(border_width * zoom, border_color);

    let raw_size = if rot == 90 || rot == 270 {
        Vec2::new(DHT22_BODY_HEIGHT, DHT22_BODY_WIDTH) * zoom
    } else {
        Vec2::new(DHT22_BODY_WIDTH, DHT22_BODY_HEIGHT) * zoom
    };
    let body_rect = Rect::from_center_size(center, raw_size);
    painter.rect_filled(body_rect, 3.5 * zoom, body_color);
    painter.rect_stroke(body_rect, 3.5 * zoom, border_stroke);

    draw_dht22_vent_slits(painter, center, zoom, rot);
    draw_dht22_labels(painter, center, zoom, rot, dht);
}

/// Renders horizontal airflow ventilation slits on the upper half of the sensor housing.
fn draw_dht22_vent_slits(painter: &Painter, center: Pos2, zoom: f32, rot: u16) {
    let slit_color = Color32::from_rgb(192, 200, 210);
    let slit_stroke = Stroke::new(1.4 * zoom, slit_color);

    for idx in 0..5 {
        let slit_y = -20.0 + idx as f32 * 5.0;
        let p_start = center + crate::canvas::rotate_offset(Vec2::new(-14.0, slit_y), rot) * zoom;
        let p_end = center + crate::canvas::rotate_offset(Vec2::new(14.0, slit_y), rot) * zoom;
        painter.line_segment([p_start, p_end], slit_stroke);
    }
}

/// Renders silkscreen product branding and active telemetry readout on the sensor face.
fn draw_dht22_labels(painter: &Painter, center: Pos2, zoom: f32, rot: u16, dht: &Dht22) {
    let title_pos = center + crate::canvas::rotate_offset(Vec2::new(0.0, 7.0), rot) * zoom;
    painter.text(
        title_pos,
        egui::Align2::CENTER_CENTER,
        "DHT22",
        FontId::proportional(7.5 * zoom),
        Color32::from_rgb(35, 55, 80),
    );

    let readout_rect_center =
        center + crate::canvas::rotate_offset(Vec2::new(0.0, 19.0), rot) * zoom;
    let readout_size = if rot == 90 || rot == 270 {
        Vec2::new(12.0, 36.0) * zoom
    } else {
        Vec2::new(36.0, 12.0) * zoom
    };
    let readout_rect = Rect::from_center_size(readout_rect_center, readout_size);
    painter.rect_filled(readout_rect, 2.0 * zoom, Color32::from_rgb(22, 28, 38));

    let (temp_text, hum_text, temp_color, hum_color) = if dht.is_powered() {
        (
            format!("{:.1}°", dht.temperature()),
            format!("{:.0}%", dht.humidity()),
            Color32::from_rgb(80, 225, 160),
            Color32::from_rgb(90, 195, 255),
        )
    } else {
        (
            "--.-°".to_string(),
            "--%".to_string(),
            Color32::from_rgb(90, 95, 105),
            Color32::from_rgb(90, 95, 105),
        )
    };

    let temp_pos =
        readout_rect_center + crate::canvas::rotate_offset(Vec2::new(-9.0, 0.0), rot) * zoom;
    let hum_pos =
        readout_rect_center + crate::canvas::rotate_offset(Vec2::new(9.0, 0.0), rot) * zoom;

    painter.text(
        temp_pos,
        egui::Align2::CENTER_CENTER,
        temp_text,
        FontId::proportional(5.2 * zoom),
        temp_color,
    );
    painter.text(
        hum_pos,
        egui::Align2::CENTER_CENTER,
        hum_text,
        FontId::proportional(5.2 * zoom),
        hum_color,
    );
}

/// Renders the 4 through-hole solder terminals with colored silkscreen pin identifiers.
fn draw_dht22_terminals(painter: &Painter, center: Pos2, zoom: f32, rot: u16) {
    let offsets = dht22_pin_offsets();
    let labels = ["VCC", "SDA", "NC", "GND"];
    let label_colors = [
        Color32::from_rgb(248, 113, 113),
        Color32::from_rgb(56, 189, 248),
        Color32::from_rgb(148, 156, 168),
        Color32::from_rgb(130, 138, 150),
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

/// Renders a single DHT22 sensor module at the specified canvas coordinates and rotation.
pub fn draw_single_dht22(
    painter: &Painter,
    center: Pos2,
    zoom: f32,
    rot: u16,
    comp: &Dht22,
    is_selected: bool,
) {
    draw_dht22_leads(painter, center, zoom, rot);
    draw_dht22_mounting_tab(painter, center, zoom, rot);
    draw_dht22_body(painter, center, zoom, rot, comp, is_selected);
    draw_dht22_terminals(painter, center, zoom, rot);
}

/// Renders quick environmental preset options in the context menu for a DHT22 sensor.
pub fn render_dht22_context_options(ui: &mut egui::Ui, comp: &mut Dht22) {
    ui.menu_button("🌡 Environment Presets", |ui| {
        for &(label, temp, hum) in &[
            ("Room (25°C, 50% RH)", 25.0, 50.0),
            ("Hot & Humid (35°C, 80% RH)", 35.0, 80.0),
            ("Freezing (-10°C, 25% RH)", -10.0, 25.0),
            ("Desert (42°C, 15% RH)", 42.0, 15.0),
        ] {
            if ui.button(label).clicked() {
                comp.set_temperature(temp);
                comp.set_humidity(hum);
                ui.close_menu();
            }
        }
    });
}
