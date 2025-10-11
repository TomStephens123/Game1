use serde::{Serialize, Deserialize};
use super::properties::ItemProperties;

/// The blueprint for an item type
///
/// This defines the static properties of an item that are shared
/// across all instances. Think of it as the "class" and ItemStack
/// as the "instance".
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemDefinition {
    /// Unique identifier (used for lookups and saves)
    pub id: String,

    /// Display name shown in UI
    pub name: String,

    /// Description shown in tooltips
    pub description: String,

    /// Path to item sprite (16x16 recommended)
    pub sprite_path: String,

    /// Maximum stack size (1 = non-stackable, 64 = typical)
    pub max_stack_size: u32,

    /// Item-specific properties and behaviors
    pub properties: ItemProperties,
}

impl ItemDefinition {
    /// Creates a new item definition
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        sprite_path: impl Into<String>,
        max_stack_size: u32,
        properties: ItemProperties,
    ) -> Self {
        ItemDefinition {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            sprite_path: sprite_path.into(),
            max_stack_size,
            properties,
        }
    }

    /// Returns true if this item can stack with another
    #[allow(dead_code)]  // Reserved for future item comparison features
    pub fn can_stack_with(&self, other: &ItemDefinition) -> bool {
        // Items can only stack if they're the same type and stackable
        self.id == other.id && self.max_stack_size > 1
    }
}
