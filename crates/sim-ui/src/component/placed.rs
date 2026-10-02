//! Spatial and oriented entity wrapper for components placed on the canvas.

use egui::{Painter, Pos2, Vec2};
use sim_core::engine::Engine;
use sim_core::netlist::PinId;

use crate::component::instance::ComponentInstance;

/// Represents a positioned, oriented component entity placed on the circuit schematic.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PlacedComponent {
    pub pos: Pos2,
    pub rotation: u16,
    pub instance: ComponentInstance,
}

impl PlacedComponent {
    /// Creates a new placed component at the specified canvas position and rotation.
    pub fn new(pos: Pos2, rotation: u16, instance: ComponentInstance) -> Self {
        Self {
            pos,
            rotation,
            instance,
        }
    }

    /// List of all terminal pins owned by this component.
    pub fn pins(&self) -> Vec<PinId> {
        self.instance.pins()
    }

    /// Returns rotated canvas positions for each terminal pin of this placed component.
    pub fn pin_positions(&self) -> Vec<(PinId, Pos2)> {
        self.instance
            .pin_offsets()
            .into_iter()
            .map(|(pin, offset)| {
                let rot_offset = crate::canvas::rotate_offset(offset, self.rotation);
                (
                    pin,
                    Pos2::new(self.pos.x + rot_offset.x, self.pos.y + rot_offset.y),
                )
            })
            .collect()
    }

    /// Effective bounding box taking into account current rotation (swaps on 90/270).
    pub fn effective_bounds(&self) -> Vec2 {
        let b = self.instance.bounds();
        if self.rotation == 90 || self.rotation == 270 {
            Vec2::new(b.y, b.x)
        } else {
            b
        }
    }

    /// Rotates the component by a signed delta angle in degrees (normalized to 0..360).
    pub fn rotate_by(&mut self, delta: i16) {
        self.rotation = ((self.rotation as i32 + delta as i32).rem_euclid(360)) as u16;
    }

    /// Renders the placed component at screen coordinates accounting for zoom and pan.
    pub fn draw(&self, painter: &Painter, origin: Pos2, pan: Vec2, zoom: f32, is_selected: bool) {
        let screen_pos = origin + pan + self.pos.to_vec2() * zoom;
        self.instance
            .draw(painter, screen_pos, zoom, self.rotation, is_selected);
    }

    /// Evaluates electrical voltages and updates component state.
    pub fn update_electrical(&mut self, engine: &Engine, sim_time_us: u64, dt_seconds: f32) {
        self.instance
            .update_electrical(engine, sim_time_us, dt_seconds);
    }

    /// Dispatches an incoming I2C write transaction.
    pub fn handle_i2c_write(&mut self, address: u8, bytes: &[u8]) {
        self.instance.handle_i2c_write(address, bytes);
    }

    /// Duplicates this placed component with new pins allocated by the provider.
    pub fn duplicate(&self, next_pin: &mut dyn FnMut() -> PinId) -> Self {
        Self {
            pos: self.pos,
            rotation: self.rotation,
            instance: self.instance.duplicate(next_pin),
        }
    }
}
