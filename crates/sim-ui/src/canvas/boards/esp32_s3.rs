use egui::{Color32, FontId, Painter, Pos2, Rect, Stroke, Vec2};
use sim_components::board::{Board, HeaderSide};

use super::elements::{
    draw_header_terminal, draw_mounting_holes, draw_pcb_substrate, draw_rgb_led, draw_status_led,
    draw_tactile_button, draw_usb_port,
};
use crate::app::SimulatorApp;
use crate::pins::{HEADER_PIN_OFFSET_X, HEADER_PIN_PITCH, HEADER_PIN_START_Y};
use crate::types::SelectedItem;

/// Renders all ESP32-S3 development boards placed on the circuit canvas.
pub fn draw_esp32_s3_boards(painter: &Painter, origin: Pos2, app: &SimulatorApp) {
    for (idx, (board_pos, board)) in app.esp32_s3_boards.iter().enumerate() {
        let is_selected = match app.selected {
            SelectedItem::Esp32S3(selected_idx) => selected_idx == idx,
            SelectedItem::Group(ref items) => items.contains(&SelectedItem::Esp32S3(idx)),
            _ => false,
        };

        let center = app.to_screen(*board_pos, origin);
        let zoom = app.zoom;
        let rot = app.esp32_s3_rotations.get(idx).copied().unwrap_or(0);

        draw_board_pcb(painter, center, zoom, rot, is_selected);
        draw_antenna_and_shield(painter, center, zoom, rot);
        draw_onboard_indicators(painter, center, zoom, rot, board);
        draw_onboard_buttons(painter, center, zoom, rot, board);
        draw_usb_ports(painter, center, zoom, rot);
        draw_header_terminals(painter, center, zoom, rot, board);
    }
}

/// Renders the matte black PCB substrate and four corner mounting holes.
fn draw_board_pcb(painter: &Painter, center: Pos2, zoom: f32, rot: u16, is_selected: bool) {
    let pcb_size = if rot == 90 || rot == 270 {
        Vec2::new(300.0, 140.0)
    } else {
        Vec2::new(140.0, 300.0)
    };
    draw_pcb_substrate(painter, center, pcb_size, 6.0, is_selected, zoom);

    let hole_offsets = [
        Vec2::new(-58.0, -138.0),
        Vec2::new(58.0, -138.0),
        Vec2::new(-58.0, 138.0),
        Vec2::new(58.0, 138.0),
    ];
    let rotated_holes: Vec<Vec2> = hole_offsets
        .iter()
        .map(|&o| crate::canvas::rotate_offset(o, rot))
        .collect();
    draw_mounting_holes(painter, center, &rotated_holes, zoom);
}

/// Renders the metallic ESP32-S3 WROOM module and PCB antenna at the top.
fn draw_antenna_and_shield(painter: &Painter, center: Pos2, zoom: f32, rot: u16) {
    let ant_size = if rot == 90 || rot == 270 {
        Vec2::new(22.0, 46.0) * zoom
    } else {
        Vec2::new(46.0, 22.0) * zoom
    };
    let ant_center = center + crate::canvas::rotate_offset(Vec2::new(0.0, -112.0), rot) * zoom;
    let antenna_rect = Rect::from_center_size(ant_center, ant_size);
    painter.rect_filled(antenna_rect, 2.0 * zoom, Color32::from_rgb(15, 20, 25));

    let shield_size = if rot == 90 || rot == 270 {
        Vec2::new(56.0, 50.0) * zoom
    } else {
        Vec2::new(50.0, 56.0) * zoom
    };
    let shield_center = center + crate::canvas::rotate_offset(Vec2::new(0.0, -68.0), rot) * zoom;
    let shield_rect = Rect::from_center_size(shield_center, shield_size);
    painter.rect_filled(shield_rect, 3.0 * zoom, Color32::from_rgb(175, 180, 188));
    painter.rect_stroke(
        shield_rect,
        3.0 * zoom,
        Stroke::new(1.0 * zoom, Color32::from_rgb(210, 215, 222)),
    );

    let text_pos = center + crate::canvas::rotate_offset(Vec2::new(0.0, -70.0), rot) * zoom;
    painter.text(
        text_pos,
        egui::Align2::CENTER_CENTER,
        "ESP32-S3",
        FontId::proportional(8.5 * zoom),
        Color32::from_rgb(40, 45, 52),
    );
}

/// Renders the 3.3V Power indicator LED and onboard WS2812 addressable RGB LED.
fn draw_onboard_indicators(
    painter: &Painter,
    center: Pos2,
    zoom: f32,
    rot: u16,
    board: &sim_components::board::Esp32S3DevKit,
) {
    let pwr_offset = crate::canvas::rotate_offset(Vec2::new(14.0, 6.0), rot) * zoom;
    draw_status_led(
        painter,
        center + pwr_offset,
        zoom,
        "PWR",
        board.is_power_led_on(),
        Color32::from_rgb(244, 67, 54),
        Color32::from_rgb(60, 30, 30),
    );

    let rgb_offset = crate::canvas::rotate_offset(Vec2::new(-14.0, 6.0), rot) * zoom;
    draw_rgb_led(painter, center + rgb_offset, zoom, "RGB", board.rgb_led());
}

/// Renders the tactile Boot and Reset pushbuttons.
fn draw_onboard_buttons(
    painter: &Painter,
    center: Pos2,
    zoom: f32,
    rot: u16,
    board: &sim_components::board::Esp32S3DevKit,
) {
    let btn_configs = [
        (
            Vec2::new(-15.0, 50.0),
            "BOOT",
            board.is_boot_pressed(),
            Color32::from_rgb(212, 168, 68),
        ),
        (
            Vec2::new(15.0, 50.0),
            "RST",
            board.is_reset_pressed(),
            Color32::from_rgb(230, 80, 70),
        ),
    ];

    for (offset, label, is_pressed, plunger_color) in btn_configs {
        let btn_pos = center + crate::canvas::rotate_offset(offset, rot) * zoom;
        draw_tactile_button(painter, btn_pos, zoom, label, is_pressed, plunger_color);
    }
}

/// Renders the dual USB-C / UART ports at the bottom edge.
fn draw_usb_ports(painter: &Painter, center: Pos2, zoom: f32, rot: u16) {
    let ports = [
        (Vec2::new(-18.0, 134.0), "USB"),
        (Vec2::new(18.0, 134.0), "UART"),
    ];

    for (offset, label) in ports {
        let pos = center + crate::canvas::rotate_offset(offset, rot) * zoom;
        draw_usb_port(painter, pos, zoom, label);
    }
}

/// Renders the 44 header terminals and their silkscreen labels.
fn draw_header_terminals(
    painter: &Painter,
    center: Pos2,
    zoom: f32,
    rot: u16,
    board: &sim_components::board::Esp32S3DevKit,
) {
    for pin in board.header_pins() {
        let (offset_x, text_x_offset, text_align) = match pin.side {
            HeaderSide::Left => (
                -HEADER_PIN_OFFSET_X,
                -HEADER_PIN_OFFSET_X + 7.5,
                if rot == 0 {
                    egui::Align2::LEFT_CENTER
                } else {
                    egui::Align2::CENTER_CENTER
                },
            ),
            HeaderSide::Right => (
                HEADER_PIN_OFFSET_X,
                HEADER_PIN_OFFSET_X - 7.5,
                if rot == 0 {
                    egui::Align2::RIGHT_CENTER
                } else {
                    egui::Align2::CENTER_CENTER
                },
            ),
            _ => (0.0, 0.0, egui::Align2::CENTER_CENTER),
        };
        let offset_y = HEADER_PIN_START_Y + pin.index as f32 * HEADER_PIN_PITCH;

        let pin_center =
            center + crate::canvas::rotate_offset(Vec2::new(offset_x, offset_y), rot) * zoom;
        let label_pos =
            center + crate::canvas::rotate_offset(Vec2::new(text_x_offset, offset_y), rot) * zoom;

        draw_header_terminal(painter, pin_center, zoom, label_pos, text_align, pin);
    }
}
