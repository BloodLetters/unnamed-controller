//! Unified component container, abstraction, and lifecycle for canvas parts.

pub mod instance;
pub mod kind;
pub mod placed;

pub use instance::ComponentInstance;
pub use kind::ComponentKind;
pub use placed::PlacedComponent;
