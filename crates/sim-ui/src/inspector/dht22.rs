//! Property inspector panel for DHT22 environmental sensor components.

use egui::{Color32, RichText, Ui, Vec2};
use sim_components::component::Dht22;

/// Renders inspector controls, live telemetry, and interactive sliders for a DHT22 sensor.
pub fn render_dht22_inspector_content(
    ui: &mut Ui,
    dht: &mut Dht22,
    engine: &sim_core::engine::Engine,
    on_delete: impl FnOnce(),
) {
    ui.label(RichText::new("DHT22 / AM2302 Sensor").strong().size(15.0));
    ui.label(
        RichText::new("Digital Relative Humidity & Temperature")
            .weak()
            .size(11.0),
    );
    ui.separator();

    render_dht22_status(ui, dht);
    ui.separator();

    render_dht22_controls(ui, dht);
    ui.separator();

    render_protocol_telemetry(ui, dht);
    ui.separator();

    render_electrical_specs(ui, engine, dht);
    ui.separator();

    if ui.button("🗑 Delete Sensor").clicked() {
        on_delete();
    }
}

/// Renders power indicator, live ambient temperature, and relative humidity telemetry.
fn render_dht22_status(ui: &mut Ui, dht: &Dht22) {
    ui.horizontal(|ui| {
        ui.label("Power:");
        let (rect, _resp) = ui.allocate_exact_size(Vec2::new(12.0, 12.0), egui::Sense::hover());
        let power_color = if dht.is_powered() {
            Color32::from_rgb(76, 175, 80)
        } else {
            Color32::from_rgb(80, 85, 95)
        };
        ui.painter().circle_filled(rect.center(), 5.0, power_color);

        if dht.is_powered() {
            ui.label(RichText::new("Powered (Active)").color(Color32::from_rgb(100, 255, 100)));
        } else {
            ui.label(RichText::new("Unpowered (<3.0V)").weak());
        }
    });

    ui.add_space(3.0);
    ui.horizontal(|ui| {
        ui.label("Temperature:");
        let temp_f = dht.temperature() * 1.8 + 32.0;
        ui.label(
            RichText::new(format!("{:.1} °C ({:.1} °F)", dht.temperature(), temp_f))
                .strong()
                .color(Color32::from_rgb(0, 229, 255)),
        );
    });

    ui.horizontal(|ui| {
        ui.label("Humidity:");
        ui.label(
            RichText::new(format!("{:.1} % RH", dht.humidity()))
                .strong()
                .color(Color32::from_rgb(76, 175, 80)),
        );
    });
}

/// Renders interactive sliders and quick environmental condition buttons.
fn render_dht22_controls(ui: &mut Ui, dht: &mut Dht22) {
    ui.label(RichText::new("Environmental Controls").strong());

    let mut temp = dht.temperature();
    if ui
        .add(egui::Slider::new(&mut temp, -40.0..=80.0).text("Temp (°C)"))
        .changed()
    {
        dht.set_temperature(temp);
    }

    let mut hum = dht.humidity();
    if ui
        .add(egui::Slider::new(&mut hum, 0.0..=100.0).text("Humidity (%)"))
        .changed()
    {
        dht.set_humidity(hum);
    }

    ui.add_space(4.0);
    ui.horizontal(|ui| {
        if ui.button("Room (25°C, 50%)").clicked() {
            dht.set_temperature(25.0);
            dht.set_humidity(50.0);
        }
        if ui.button("Hot (35°C, 80%)").clicked() {
            dht.set_temperature(35.0);
            dht.set_humidity(80.0);
        }
    });

    ui.horizontal(|ui| {
        if ui.button("Cold (-10°C, 25%)").clicked() {
            dht.set_temperature(-10.0);
            dht.set_humidity(25.0);
        }
        if ui.button("Desert (42°C, 15%)").clicked() {
            dht.set_temperature(42.0);
            dht.set_humidity(15.0);
        }
    });
}

/// Renders 1-wire protocol state and raw 40-bit frame diagnostics.
fn render_protocol_telemetry(ui: &mut Ui, dht: &Dht22) {
    ui.label(RichText::new("Single-Bus Protocol").strong());

    ui.horizontal(|ui| {
        ui.label("State:");
        ui.label(format!("{:?}", dht.protocol_state()));
    });

    ui.horizontal(|ui| {
        ui.label("Read Queries:");
        ui.label(format!("{}", dht.read_count()));
    });

    let frame = dht.raw_frame();
    ui.label(format!(
        "Raw Frame: [0x{:02X}, 0x{:02X}, 0x{:02X}, 0x{:02X}, 0x{:02X}]",
        frame[0], frame[1], frame[2], frame[3], frame[4]
    ));

    ui.horizontal(|ui| {
        ui.label("Checksum:");
        if dht.checksum_valid() {
            ui.label(RichText::new("Valid").color(Color32::from_rgb(100, 255, 100)));
        } else {
            ui.label(RichText::new("Error").color(Color32::from_rgb(255, 100, 100)));
        }
    });
}

/// Renders real-time voltage measurements on each sensor terminal pin.
fn render_electrical_specs(ui: &mut Ui, engine: &sim_core::engine::Engine, dht: &Dht22) {
    ui.label(RichText::new("Terminal Telemetry").strong());

    let v_vcc = crate::sim::get_pin_voltage(engine, dht.vcc());
    let v_data = crate::sim::get_pin_voltage(engine, dht.data());
    let v_nc = crate::sim::get_pin_voltage(engine, dht.nc());
    let v_gnd = crate::sim::get_pin_voltage(engine, dht.gnd());

    ui.label(format!("VCC Pin (1): {:?} ({:.2} V)", dht.vcc(), v_vcc));
    ui.label(format!(
        "SDA/Data Pin (2): {:?} ({:.2} V)",
        dht.data(),
        v_data
    ));
    ui.label(format!("NC Pin (3): {:?} ({:.2} V)", dht.nc(), v_nc));
    ui.label(format!("GND Pin (4): {:?} ({:.2} V)", dht.gnd(), v_gnd));
    ui.label(format!("Supply Potential: {:.2} V", v_vcc - v_gnd));
}
