//! Add Component modal window and device catalog.

pub mod card;
pub mod catalog;
pub mod dialog;
pub mod types;

pub use dialog::render_add_component_dialog;
pub use types::{CategoryKind, ComponentItem};
