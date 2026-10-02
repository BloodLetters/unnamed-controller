use egui::{Pos2, Vec2};
use sim_components::board::Board;
use sim_core::netlist::PinId;

use crate::app::SimulatorApp;
use crate::types::{SelectedItem, SpawningComponent};

/// Metadata and index-based operations for one component storage bucket.
pub struct Spec {
    /// Builds the selectable identity for an index in this bucket.
    pub make: fn(usize) -> SelectedItem,
    /// Extracts the bucket index when the selection refers to this component kind.
    pub index: fn(&SelectedItem) -> Option<usize>,
    /// Number of instances currently stored.
    pub count: fn(&SimulatorApp) -> usize,
    /// Canvas position of an instance.
    pub pos: fn(&SimulatorApp, usize) -> Option<Pos2>,
    /// Bounding size used for hit testing and box selection.
    pub bounds: fn(&SimulatorApp, usize) -> Vec2,
    /// All pin terminals of an instance.
    pub pins: fn(&SimulatorApp, usize) -> Vec<PinId>,
    /// Returns the rotation angle in degrees (0, 90, 180, 270).
    pub rotation: fn(&SimulatorApp, usize) -> u16,
    /// Rotates the instance by a signed delta angle in degrees.
    pub rotate: fn(&mut SimulatorApp, usize, i16),
    /// Removes an instance and reports its pins as detached.
    pub remove: fn(&mut SimulatorApp, usize, &mut Vec<PinId>),
    /// Displaces an instance by a canvas delta.
    pub move_by: fn(&mut SimulatorApp, usize, Vec2),
}

/// The single source of truth for per-component selection, hit testing, deletion, and moving.
pub const CATALOG: &[Spec] = &[
    Spec {
        make: SelectedItem::Esp32S3,
        index: |item| match item {
            SelectedItem::Esp32S3(i) => Some(*i),
            _ => None,
        },
        count: |app| app.esp32_s3_boards.len(),
        pos: |app, i| app.esp32_s3_boards.get(i).map(|(pos, _)| *pos),
        bounds: |app, i| {
            let rot = app.esp32_s3_rotations.get(i).copied().unwrap_or(0);
            if rot == 90 || rot == 270 {
                Vec2::new(300.0, 140.0)
            } else {
                Vec2::new(140.0, 300.0)
            }
        },
        pins: |app, i| {
            app.esp32_s3_boards
                .get(i)
                .map(|(_, b)| b.pins())
                .unwrap_or_default()
        },
        rotation: |app, i| app.esp32_s3_rotations.get(i).copied().unwrap_or(0),
        rotate: |app, i, delta| {
            if let Some(r) = app.esp32_s3_rotations.get_mut(i) {
                *r = ((*r as i32 + delta as i32).rem_euclid(360)) as u16;
            }
        },
        remove: |app, i, pins| {
            crate::state::remove_indexed_component(&mut app.esp32_s3_boards, i, pins);
            if i < app.esp32_s3_rotations.len() {
                app.esp32_s3_rotations.remove(i);
            }
        },
        move_by: |app, i, delta| {
            if let Some(item) = app.esp32_s3_boards.get_mut(i) {
                item.0 += delta;
            }
        },
    },
    Spec {
        make: SelectedItem::Component,
        index: |item| match item {
            SelectedItem::Component(i) => Some(*i),
            _ => None,
        },
        count: |app| app.components.len(),
        pos: |app, i| app.components.get(i).map(|c| c.pos),
        bounds: |app, i| {
            app.components
                .get(i)
                .map(|c| c.effective_bounds())
                .unwrap_or(Vec2::ZERO)
        },
        pins: |app, i| app.components.get(i).map(|c| c.pins()).unwrap_or_default(),
        rotation: |app, i| app.components.get(i).map(|c| c.rotation).unwrap_or(0),
        rotate: |app, i, delta| {
            if let Some(c) = app.components.get_mut(i) {
                c.rotate_by(delta);
            }
        },
        remove: |app, i, pins| {
            if i < app.components.len() {
                let comp = app.components.remove(i);
                pins.extend(comp.pins());
            }
        },
        move_by: |app, i, delta| {
            if let Some(c) = app.components.get_mut(i) {
                c.pos += delta;
            }
        },
    },
];

/// Returns the effective visual bounding box accounting for rotation.
pub fn item_bounds(spec: &Spec, app: &SimulatorApp, index: usize) -> Vec2 {
    (spec.bounds)(app, index)
}

/// Resolves the catalog entry and index that a selection refers to, if it is a component.
pub fn find(item: &SelectedItem) -> Option<(&'static Spec, usize)> {
    CATALOG
        .iter()
        .find_map(|spec| (spec.index)(item).map(|index| (spec, index)))
}

/// A palette button that places a specific component variant.
pub struct PaletteEntry {
    pub label: &'static str,
    pub category: Category,
    pub spawn: SpawningComponent,
}

/// Palette grouping for the component and board library sidebar.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Boards,
    Components,
}

impl Category {
    /// Human-readable heading for the palette group.
    pub fn title(self) -> &'static str {
        match self {
            Category::Boards => "Development Boards",
            Category::Components => "Components",
        }
    }

    /// All categories in display order.
    pub const ALL: [Category; 2] = [Category::Boards, Category::Components];
}

/// Every placeable component variant shown in the sidebar.
pub const PALETTE: &[PaletteEntry] = &[
    PaletteEntry {
        label: "ESP32-S3 DevKitC-1",
        category: Category::Boards,
        spawn: SpawningComponent::Esp32S3,
    },
    PaletteEntry {
        label: "LED (5mm)",
        category: Category::Components,
        spawn: SpawningComponent::LED,
    },
    PaletteEntry {
        label: "SSD1306 OLED (128x64)",
        category: Category::Components,
        spawn: SpawningComponent::SSD1306,
    },
    PaletteEntry {
        label: "LCD 1602 (I2C)",
        category: Category::Components,
        spawn: SpawningComponent::LCD1602,
    },
    PaletteEntry {
        label: "SG90 Servo",
        category: Category::Components,
        spawn: SpawningComponent::SERVO,
    },
    PaletteEntry {
        label: "DHT22 Sensor",
        category: Category::Components,
        spawn: SpawningComponent::DHT22,
    },
];
