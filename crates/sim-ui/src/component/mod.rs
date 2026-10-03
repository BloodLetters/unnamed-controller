//! Unified component container, abstraction, and lifecycle for canvas parts.

pub mod instance;
pub mod kind;
pub mod placed;
pub mod rendering;

pub use instance::ComponentInstance;
pub use kind::ComponentKind;
pub use placed::PlacedComponent;
