use super::inventory::Inventory;
use crate::item::{ItemRegistry, ItemStack};
use serde::{Serialize, Deserialize};

/// Player-specific inventory with hotbar
///
/// This wraps the core Inventory with player-specific functionality
/// like hotbar slot access and selected slot tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInventory {
    /// Core inventory (27 slots: 9 hotbar + 18 main)
    pub inventory: Inventory,

    /// Currently selected hotbar slot (0-8)
    pub selected_hotbar_slot: usize,
}

impl PlayerInventory {
    /// Creates a new player inventory
    ///
    /// Layout:
    /// - Slots 0-8: Hotbar
    /// - Slots 9-26: Main inventory
    pub fn new() -> Self {
        PlayerInventory {
            inventory: Inventory::new(27),
            selected_hotbar_slot: 0,
        }
    }

    /// Gets the currently selected hotbar slot
    #[allow(dead_code)]
    pub fn get_selected_hotbar(&self) -> Option<&ItemStack> {
        self.inventory.slots[self.selected_hotbar_slot].as_ref()
    }

    /// Sets the selected hotbar slot (0-8)
    #[allow(dead_code)]
    pub fn set_hotbar_slot(&mut self, slot: usize) {
        if slot < 9 {
            self.selected_hotbar_slot = slot;
        }
    }

    /// Gets a specific hotbar slot (0-8)
    #[allow(dead_code)]
    pub fn get_hotbar_slot(&self, index: usize) -> Option<&ItemStack> {
        if index < 9 {
            self.inventory.slots[index].as_ref()
        } else {
            None
        }
    }

    /// Checks if the player has at least `quantity` of an item
    #[allow(dead_code)]
    pub fn has_item(&self, item_id: &str, quantity: u32) -> bool {
        self.inventory.count_item(item_id) >= quantity
    }

    /// Quick-add to inventory (tries hotbar first, then main)
    pub fn quick_add(
        &mut self,
        item_id: &str,
        quantity: u32,
        item_registry: &ItemRegistry,
    ) -> Result<u32, super::error::InventoryError> {
        self.inventory.add_item(item_id, quantity, item_registry)
    }
}

impl Default for PlayerInventory {
    fn default() -> Self {
        Self::new()
    }
}
