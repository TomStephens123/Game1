use serde::{Serialize, Deserialize};
use crate::stats::ModifierEffect;

/// Different categories of items with type-specific data
///
/// This enum enables different item types to have different behaviors
/// while sharing the core ItemDefinition structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ItemProperties {
    /// Basic material (no special properties)
    Material,

    /// Consumable item (use from hotbar)
    Consumable {
        effect: ConsumableEffect,
        use_time: f32,  // Seconds to consume
    },

    /// Equipment item (worn for stat bonuses)
    Equipment {
        slot: EquipmentSlot,
        modifiers: Vec<ModifierEffect>,
    },

    /// Tool item (interact with blocks)
    Tool {
        tool_type: ToolType,
        durability: u32,
        mining_speed: f32,
    },

    /// Block item (can be placed in world)
    Block {
        block_id: String,  // ID of block type to place
    },
}

/// Effects for consumable items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsumableEffect {
    /// Restore health
    Heal(f32),

    /// Restore stamina (future)
    RestoreStamina(f32),

    /// Apply temporary stat buff
    Buff {
        modifier: ModifierEffect,
        duration: f32,  // Seconds
    },

    /// Custom effect (for special items)
    Custom(String),  // Effect ID to look up
}

/// Equipment slots for items
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EquipmentSlot {
    Head,
    Chest,
    Legs,
    Feet,
    MainHand,   // Weapon
    OffHand,    // Shield
    Accessory1,
    Accessory2,
}

/// Tool types for mining/interaction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolType {
    Pickaxe,
    Axe,
    Shovel,
    Hoe,
    Sword,  // Weapon tool
}
