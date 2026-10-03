//! Data models and category classifications for the Add Component catalog.

use crate::types::SpawningComponent;

/// High-level component category used to filter the Add Component catalog.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Default, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum CategoryKind {
    #[default]
    All,
    Boards,
    Displays,
    Output,
    Sensors,
    Input,
    Passive,
}

impl CategoryKind {
    /// Human-readable label for the category.
    pub fn label(self) -> &'static str {
        match self {
            Self::All => "All Components",
            Self::Boards => "Boards",
            Self::Displays => "Displays",
            Self::Output => "Output",
            Self::Sensors => "Sensors",
            Self::Input => "Input",
            Self::Passive => "Passive",
        }
    }

    /// All categories in standard sidebar display order.
    pub const ALL: [Self; 7] = [
        Self::All,
        Self::Boards,
        Self::Displays,
        Self::Output,
        Self::Sensors,
        Self::Input,
        Self::Passive,
    ];
}

/// Metadata description for one catalog item presented in the modal grid.
#[derive(Debug, Clone)]
pub struct ComponentItem {
    pub id: &'static str,
    pub name: &'static str,
    pub category: CategoryKind,
    pub specs: &'static str,
    pub badge: Option<&'static str>,
    pub is_pro: bool,
    pub is_ready: bool,
    pub spawn: SpawningComponent,
}
