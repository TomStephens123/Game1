# Inventory System Design

## Overview

The Inventory System manages containers of items (slots) and the logic for adding, removing, and transferring items between inventories. This system powers player inventories, chests, and any other item storage in the game.

**Key Features**:
- **Slot-based storage** with configurable capacity
- **Automatic stacking** of identical items
- **Item transfer** between inventories
- **Type-safe operations** with clear error handling
- **Save/load support** for persistence
- **Extensible** for different inventory types

**Status**: 🏗️ **PLANNED** - December 2024

## Core Concepts

### Inventory Types

The system supports multiple inventory implementations using a shared core:

1. **Player Inventory** - 27 slots (9 hotbar + 18 main)
   - Special hotbar access methods
   - Quick-use from hotbar slots
   - Persists with player save data

2. **Container Inventory** - Variable slots (chests: 27, barrels: 18, etc.)
   - Associated with a world block position
   - Persists with world save data
   - Drops contents when destroyed

3. **Automation Inventory** - Future: multiple input/output slots
   - Furnace: input, fuel, output
   - Crafting station: 9 crafting slots + 1 result
   - Special transfer rules

### Slot Architecture

Inventories use `Option<ItemStack>` for slots:

```
Slot States:
- None             → Empty slot
- Some(ItemStack)  → Slot contains items

Slot Operations:
- Add      → Insert items (stack if possible)
- Remove   → Take items out
- Transfer → Move items between inventories
- Swap     → Exchange slot contents
```

## Architecture

### Module Structure

```
src/inventory/
├── mod.rs              # Public API, re-exports
├── inventory.rs        # Core Inventory struct
├── player.rs           # PlayerInventory wrapper
├── container.rs        # ContainerInventory for blocks
└── error.rs            # InventoryError types
```

### Component 1: Core Inventory

**File**: `src/inventory/inventory.rs`

```rust
use crate::item::ItemStack;
use crate::item::ItemRegistry;
use super::error::InventoryError;

/// Generic inventory container with slots
///
/// This is the core storage structure used by all inventory types.
/// It handles slot management, stacking, and basic operations.
pub struct Inventory {
    /// Slots that can hold item stacks (None = empty)
    pub slots: Vec<Option<ItemStack>>,

    /// Maximum number of slots
    pub capacity: usize,
}

impl Inventory {
    /// Creates a new empty inventory with the specified capacity
    pub fn new(capacity: usize) -> Self {
        Inventory {
            slots: vec![None; capacity],
            capacity,
        }
    }

    /// Returns true if the inventory has no items
    pub fn is_empty(&self) -> bool {
        self.slots.iter().all(|slot| slot.is_none())
    }

    /// Returns true if all slots are occupied
    pub fn is_full(&self) -> bool {
        self.slots.iter().all(|slot| slot.is_some())
    }

    /// Counts how many of a specific item are in the inventory
    pub fn count_item(&self, item_id: &str) -> u32 {
        self.slots
            .iter()
            .filter_map(|slot| slot.as_ref())
            .filter(|stack| stack.item_id == item_id)
            .map(|stack| stack.quantity)
            .sum()
    }

    /// Finds the first empty slot index
    pub fn find_empty_slot(&self) -> Option<usize> {
        self.slots
            .iter()
            .position(|slot| slot.is_none())
    }

    /// Finds the first slot containing a specific item with room to stack
    pub fn find_stackable_slot(
        &self,
        item_id: &str,
        item_registry: &ItemRegistry,
    ) -> Option<usize> {
        let max_stack = item_registry.get(item_id)?.max_stack_size;

        self.slots
            .iter()
            .position(|slot| {
                if let Some(stack) = slot {
                    stack.item_id == item_id && stack.quantity < max_stack
                } else {
                    false
                }
            })
    }

    /// Adds items to the inventory
    ///
    /// Returns the number of items that couldn't fit (overflow).
    /// Returns 0 if all items were added successfully.
    ///
    /// # Strategy
    /// 1. Try to stack with existing items first
    /// 2. Create new stacks in empty slots
    /// 3. Return overflow if inventory is full
    pub fn add_item(
        &mut self,
        item_id: &str,
        quantity: u32,
        item_registry: &ItemRegistry,
    ) -> Result<u32, InventoryError> {
        if quantity == 0 {
            return Ok(0);
        }

        // Validate item exists
        let item_def = item_registry.get(item_id)
            .ok_or(InventoryError::InvalidItem(item_id.to_string()))?;

        let max_stack_size = item_def.max_stack_size;
        let mut remaining = quantity;

        // Phase 1: Try to add to existing stacks
        for slot in self.slots.iter_mut() {
            if remaining == 0 {
                break;
            }

            if let Some(stack) = slot {
                if stack.item_id == item_id && stack.quantity < max_stack_size {
                    let added = stack.add(remaining, max_stack_size);
                    remaining -= (remaining - added);
                }
            }
        }

        // Phase 2: Create new stacks in empty slots
        while remaining > 0 {
            if let Some(empty_index) = self.find_empty_slot() {
                let stack_size = remaining.min(max_stack_size);
                self.slots[empty_index] = Some(ItemStack::new(item_id, stack_size));
                remaining -= stack_size;
            } else {
                // No more empty slots, return overflow
                break;
            }
        }

        Ok(remaining)  // Return how many items didn't fit
    }

    /// Removes items from the inventory
    ///
    /// Returns the number of items actually removed (might be less than requested).
    ///
    /// # Strategy
    /// 1. Scan all slots with matching item
    /// 2. Take items until quantity is met
    /// 3. Clear empty slots
    pub fn remove_item(&mut self, item_id: &str, quantity: u32) -> u32 {
        let mut remaining = quantity;
        let mut removed_total = 0;

        for slot in self.slots.iter_mut() {
            if remaining == 0 {
                break;
            }

            if let Some(stack) = slot {
                if stack.item_id == item_id {
                    let to_take = remaining.min(stack.quantity);
                    stack.quantity -= to_take;
                    remaining -= to_take;
                    removed_total += to_take;

                    // Clear slot if empty
                    if stack.quantity == 0 {
                        *slot = None;
                    }
                }
            }
        }

        removed_total
    }

    /// Removes a specific quantity from a specific slot
    ///
    /// Returns the removed ItemStack, or None if slot is empty.
    pub fn take_from_slot(&mut self, slot_index: usize, quantity: u32) -> Option<ItemStack> {
        if slot_index >= self.capacity {
            return None;
        }

        if let Some(stack) = &mut self.slots[slot_index] {
            if quantity >= stack.quantity {
                // Take entire stack
                self.slots[slot_index].take()
            } else {
                // Split stack
                stack.split(quantity)
            }
        } else {
            None
        }
    }

    /// Places items in a specific slot
    ///
    /// If the slot is empty, places the item.
    /// If the slot has the same item, tries to stack.
    /// Returns overflow items that didn't fit.
    pub fn place_in_slot(
        &mut self,
        slot_index: usize,
        item_stack: ItemStack,
        item_registry: &ItemRegistry,
    ) -> Result<Option<ItemStack>, InventoryError> {
        if slot_index >= self.capacity {
            return Err(InventoryError::InvalidSlot(slot_index));
        }

        let max_stack = item_registry.get(&item_stack.item_id)
            .ok_or(InventoryError::InvalidItem(item_stack.item_id.clone()))?
            .max_stack_size;

        match &mut self.slots[slot_index] {
            None => {
                // Empty slot, place item
                self.slots[slot_index] = Some(item_stack);
                Ok(None)
            }
            Some(existing_stack) => {
                if existing_stack.item_id == item_stack.item_id {
                    // Same item, try to stack
                    let overflow = existing_stack.add(item_stack.quantity, max_stack);

                    if overflow > 0 {
                        Ok(Some(ItemStack::new(&item_stack.item_id, overflow)))
                    } else {
                        Ok(None)
                    }
                } else {
                    // Different item, can't stack
                    Err(InventoryError::SlotOccupied(slot_index))
                }
            }
        }
    }

    /// Swaps the contents of two slots
    pub fn swap_slots(&mut self, slot_a: usize, slot_b: usize) -> Result<(), InventoryError> {
        if slot_a >= self.capacity {
            return Err(InventoryError::InvalidSlot(slot_a));
        }
        if slot_b >= self.capacity {
            return Err(InventoryError::InvalidSlot(slot_b));
        }

        self.slots.swap(slot_a, slot_b);
        Ok(())
    }

    /// Transfers items from one inventory to another
    ///
    /// Returns the number of items successfully transferred.
    pub fn transfer_to(
        &mut self,
        other: &mut Inventory,
        item_id: &str,
        quantity: u32,
        item_registry: &ItemRegistry,
    ) -> Result<u32, InventoryError> {
        // Remove from source
        let removed = self.remove_item(item_id, quantity);

        if removed == 0 {
            return Ok(0);
        }

        // Add to destination
        let overflow = other.add_item(item_id, removed, item_registry)?;

        // Put overflow back in source
        if overflow > 0 {
            self.add_item(item_id, overflow, item_registry)?;
        }

        Ok(removed - overflow)
    }

    /// Transfers items from a specific slot to another inventory
    pub fn transfer_slot_to(
        &mut self,
        slot_index: usize,
        other: &mut Inventory,
        item_registry: &ItemRegistry,
    ) -> Result<(), InventoryError> {
        if slot_index >= self.capacity {
            return Err(InventoryError::InvalidSlot(slot_index));
        }

        if let Some(stack) = self.slots[slot_index].take() {
            let overflow = other.add_item(&stack.item_id, stack.quantity, item_registry)?;

            // Put overflow back in original slot
            if overflow > 0 {
                self.slots[slot_index] = Some(ItemStack::new(&stack.item_id, overflow));
            }
        }

        Ok(())
    }

    /// Clears all items from the inventory
    pub fn clear(&mut self) {
        self.slots.fill(None);
    }

    /// Returns an iterator over all non-empty item stacks
    pub fn iter_items(&self) -> impl Iterator<Item = &ItemStack> {
        self.slots.iter().filter_map(|slot| slot.as_ref())
    }
}
```

### Component 2: Inventory Errors

**File**: `src/inventory/error.rs`

```rust
use std::fmt;

/// Errors that can occur during inventory operations
#[derive(Debug, Clone)]
pub enum InventoryError {
    /// Slot index out of bounds
    InvalidSlot(usize),

    /// Item ID doesn't exist in registry
    InvalidItem(String),

    /// Slot is occupied (can't place different item)
    SlotOccupied(usize),

    /// Inventory is full (can't add more items)
    InventoryFull,

    /// Tried to remove more items than exist
    InsufficientItems {
        requested: u32,
        available: u32,
    },
}

impl fmt::Display for InventoryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InventoryError::InvalidSlot(index) => {
                write!(f, "Invalid slot index: {}", index)
            }
            InventoryError::InvalidItem(id) => {
                write!(f, "Invalid item ID: {}", id)
            }
            InventoryError::SlotOccupied(index) => {
                write!(f, "Slot {} is occupied", index)
            }
            InventoryError::InventoryFull => {
                write!(f, "Inventory is full")
            }
            InventoryError::InsufficientItems { requested, available } => {
                write!(f, "Insufficient items (requested: {}, available: {})", requested, available)
            }
        }
    }
}

impl std::error::Error for InventoryError {}
```

### Component 3: Player Inventory

**File**: `src/inventory/player.rs`

```rust
use super::inventory::Inventory;
use crate::item::ItemRegistry;

/// Player-specific inventory with hotbar
///
/// This wraps the core Inventory with player-specific functionality
/// like hotbar slot access and selected slot tracking.
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
    pub fn get_selected_hotbar(&self) -> Option<&crate::item::ItemStack> {
        self.inventory.slots[self.selected_hotbar_slot].as_ref()
    }

    /// Sets the selected hotbar slot (0-8)
    pub fn set_hotbar_slot(&mut self, slot: usize) {
        if slot < 9 {
            self.selected_hotbar_slot = slot;
        }
    }

    /// Gets a specific hotbar slot (0-8)
    pub fn get_hotbar_slot(&self, index: usize) -> Option<&crate::item::ItemStack> {
        if index < 9 {
            self.inventory.slots[index].as_ref()
        } else {
            None
        }
    }

    /// Checks if the player has at least `quantity` of an item
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
```

### Component 4: Container Inventory

**File**: `src/inventory/container.rs`

```rust
use super::inventory::Inventory;

/// Container inventory for blocks (chests, barrels, etc.)
///
/// This wraps the core Inventory with container-specific data
/// like world position and open state.
pub struct ContainerInventory {
    /// Core inventory storage
    pub inventory: Inventory,

    /// World position of the container block
    pub world_x: i32,
    pub world_y: i32,

    /// True if a player has this container open
    pub is_open: bool,

    /// Type of container (for UI rendering)
    pub container_type: ContainerType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerType {
    Chest,      // 27 slots
    Barrel,     // 18 slots
    Dispenser,  // 9 slots
    // Future: Furnace, CraftingTable, etc.
}

impl ContainerInventory {
    /// Creates a new container inventory at a world position
    pub fn new(container_type: ContainerType, world_x: i32, world_y: i32) -> Self {
        let capacity = match container_type {
            ContainerType::Chest => 27,
            ContainerType::Barrel => 18,
            ContainerType::Dispenser => 9,
        };

        ContainerInventory {
            inventory: Inventory::new(capacity),
            world_x,
            world_y,
            is_open: false,
            container_type,
        }
    }

    /// Opens the container (sets open flag)
    pub fn open(&mut self) {
        self.is_open = true;
    }

    /// Closes the container
    pub fn close(&mut self) {
        self.is_open = false;
    }

    /// Drops all items as entities when container is destroyed
    ///
    /// Returns a list of (item_id, quantity) to spawn as dropped items
    pub fn drop_contents(&mut self) -> Vec<(String, u32)> {
        let mut drops = Vec::new();

        for slot in &mut self.inventory.slots {
            if let Some(stack) = slot.take() {
                drops.push((stack.item_id, stack.quantity));
            }
        }

        drops
    }
}
```

### Save System Integration

**File**: `src/inventory/inventory.rs` (continued)

```rust
use crate::save::{Saveable, SaveData, SaveError};
use serde::{Serialize, Deserialize};

impl Saveable for Inventory {
    fn to_save_data(&self) -> Result<SaveData, SaveError> {
        #[derive(Serialize)]
        struct InventoryData {
            capacity: usize,
            // Only save non-empty slots (saves space)
            items: Vec<(usize, ItemStack)>,  // (slot_index, stack)
        }

        let mut items = Vec::new();
        for (index, slot) in self.slots.iter().enumerate() {
            if let Some(stack) = slot {
                items.push((index, stack.clone()));
            }
        }

        let data = InventoryData {
            capacity: self.capacity,
            items,
        };

        Ok(SaveData {
            data_type: "inventory".to_string(),
            json_data: serde_json::to_string(&data)?,
        })
    }

    fn from_save_data(data: &SaveData) -> Result<Self, SaveError> {
        #[derive(Deserialize)]
        struct InventoryData {
            capacity: usize,
            items: Vec<(usize, ItemStack)>,
        }

        if data.data_type != "inventory" {
            return Err(SaveError::CorruptedData(format!(
                "Expected inventory, got {}",
                data.data_type
            )));
        }

        let inv_data: InventoryData = serde_json::from_str(&data.json_data)?;

        let mut inventory = Inventory::new(inv_data.capacity);

        // Restore items to their slots
        for (index, stack) in inv_data.items {
            if index < inventory.capacity {
                inventory.slots[index] = Some(stack);
            } else {
                eprintln!("Warning: Skipped item in invalid slot {}", index);
            }
        }

        Ok(inventory)
    }
}

// PlayerInventory saves the same way (delegates to Inventory)
impl Saveable for PlayerInventory {
    fn to_save_data(&self) -> Result<SaveData, SaveError> {
        #[derive(Serialize)]
        struct PlayerInventoryData {
            inventory: SaveData,
            selected_hotbar_slot: usize,
        }

        let data = PlayerInventoryData {
            inventory: self.inventory.to_save_data()?,
            selected_hotbar_slot: self.selected_hotbar_slot,
        };

        Ok(SaveData {
            data_type: "player_inventory".to_string(),
            json_data: serde_json::to_string(&data)?,
        })
    }

    fn from_save_data(data: &SaveData) -> Result<Self, SaveError> {
        #[derive(Deserialize)]
        struct PlayerInventoryData {
            inventory: SaveData,
            selected_hotbar_slot: usize,
        }

        if data.data_type != "player_inventory" {
            return Err(SaveError::CorruptedData(format!(
                "Expected player_inventory, got {}",
                data.data_type
            )));
        }

        let inv_data: PlayerInventoryData = serde_json::from_str(&data.json_data)?;

        Ok(PlayerInventory {
            inventory: Inventory::from_save_data(&inv_data.inventory)?,
            selected_hotbar_slot: inv_data.selected_hotbar_slot,
        })
    }
}
```

## Usage Examples

### Example 1: Player Picking Up Item

```rust
// When player collides with dropped item
fn pickup_item(
    player_inventory: &mut PlayerInventory,
    dropped_item: &DroppedItem,
    item_registry: &ItemRegistry,
) -> bool {
    match player_inventory.quick_add(
        &dropped_item.item_id,
        dropped_item.quantity,
        item_registry,
    ) {
        Ok(overflow) => {
            if overflow == 0 {
                // All items picked up
                true
            } else {
                // Some items couldn't fit, update dropped item quantity
                // (not implemented in this example)
                false
            }
        }
        Err(e) => {
            eprintln!("Failed to pickup item: {}", e);
            false
        }
    }
}
```

### Example 2: Transferring to Chest

```rust
// Shift+click to quick transfer
fn quick_transfer_to_chest(
    player_inv: &mut PlayerInventory,
    chest: &mut ContainerInventory,
    slot_index: usize,
    item_registry: &ItemRegistry,
) -> Result<(), InventoryError> {
    player_inv.inventory.transfer_slot_to(
        slot_index,
        &mut chest.inventory,
        item_registry,
    )
}
```

### Example 3: Using Consumable Item

```rust
// Use item from hotbar
fn use_hotbar_item(
    player: &mut Player,
    player_inv: &mut PlayerInventory,
    item_registry: &ItemRegistry,
) -> Result<(), String> {
    if let Some(stack) = player_inv.get_selected_hotbar() {
        if let Some(item_def) = item_registry.get(&stack.item_id) {
            // Check if consumable
            if let ItemProperties::Consumable { effect, .. } = &item_def.properties {
                // Apply effect
                match effect {
                    ConsumableEffect::Heal(amount) => {
                        player.stats.heal(*amount);
                    }
                    // ... other effects
                }

                // Remove 1 item from hotbar slot
                player_inv.inventory.take_from_slot(
                    player_inv.selected_hotbar_slot,
                    1,
                );

                return Ok(());
            }
        }
    }

    Err("No item to use".to_string())
}
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inventory_add_and_count() {
        let mut inv = Inventory::new(27);
        let registry = ItemRegistry::create_default();

        inv.add_item("slime_ball", 10, &registry).unwrap();
        assert_eq!(inv.count_item("slime_ball"), 10);
    }

    #[test]
    fn test_inventory_stacking() {
        let mut inv = Inventory::new(1);
        let registry = ItemRegistry::create_default();

        inv.add_item("slime_ball", 30, &registry).unwrap();
        inv.add_item("slime_ball", 30, &registry).unwrap();

        // Should stack in same slot
        assert_eq!(inv.slots[0].as_ref().unwrap().quantity, 60);
    }

    #[test]
    fn test_inventory_overflow() {
        let mut inv = Inventory::new(1);
        let registry = ItemRegistry::create_default();

        inv.add_item("slime_ball", 64, &registry).unwrap();
        let overflow = inv.add_item("slime_ball", 10, &registry).unwrap();

        assert_eq!(overflow, 10);  // Couldn't fit 10 items
    }

    #[test]
    fn test_transfer_between_inventories() {
        let mut inv1 = Inventory::new(27);
        let mut inv2 = Inventory::new(27);
        let registry = ItemRegistry::create_default();

        inv1.add_item("slime_ball", 50, &registry).unwrap();

        let transferred = inv1.transfer_to(
            &mut inv2,
            "slime_ball",
            30,
            &registry,
        ).unwrap();

        assert_eq!(transferred, 30);
        assert_eq!(inv1.count_item("slime_ball"), 20);
        assert_eq!(inv2.count_item("slime_ball"), 30);
    }
}
```

## Performance Considerations

### Slot Search Optimization

For large inventories, use early exit:

```rust
// Good: stops when found
pub fn find_empty_slot(&self) -> Option<usize> {
    self.slots.iter().position(|slot| slot.is_none())
}

// Bad: checks all slots
pub fn find_empty_slot_slow(&self) -> Option<usize> {
    for (i, slot) in self.slots.iter().enumerate() {
        if slot.is_none() {
            return Some(i);
        }
    }
    None
}
```

### Stack Operations

Minimize allocations when stacking:

```rust
// ItemStack methods modify in-place (no new allocations)
stack.add(10, 64);  // ✅ Good
// vs
// stack = ItemStack::new(id, stack.quantity + 10);  // ❌ Allocates
```

### Save Format

Save only non-empty slots to reduce file size:

```rust
// Sparse format: only save occupied slots
items: Vec<(usize, ItemStack)>  // ✅ Small saves

// vs Dense format: save all slots
items: Vec<Option<ItemStack>>  // ❌ Large saves
```

## Rust Learning Opportunities

This system teaches:

1. **Vec and Option** (Chapters 4, 6)
   - `Vec<Option<T>>` for nullable collections
   - Pattern matching on Options
   - Iterators over Option types

2. **Error Handling** (Chapter 9)
   - Custom error types with Display
   - Result propagation
   - Recoverable vs unrecoverable errors

3. **Borrowing** (Chapter 4)
   - Mutable references for inventory operations
   - Borrowing rules in transfer operations
   - Lifetime considerations

4. **Iterators** (Chapter 13)
   - `filter_map` for non-empty slots
   - `position` for finding slots
   - Custom iterator methods

5. **Composition** (Chapter 17)
   - PlayerInventory wraps Inventory
   - Delegation pattern
   - Type-specific extensions

## Summary

The Inventory System provides:

✅ **Flexible Storage** - Configurable slot-based containers
✅ **Automatic Stacking** - Smart item merging
✅ **Type-Safe Operations** - Rust prevents invalid states
✅ **Transfer Logic** - Move items between inventories
✅ **Save Support** - Full persistence integration
✅ **Extensible** - Easy to add new inventory types

**Next Steps**:
1. Implement `src/inventory/` module
2. Add PlayerInventory to Player struct
3. Test item adding and stacking
4. Integrate with UI for display

---

**Related Documentation**:
- **Feature Plan**: `docs/features/item-inventory-system.md`
- **Item System**: `docs/systems/item-system-design.md`
- **Dropped Items**: `docs/systems/dropped-item-entity.md`
- **Quick Reference**: `docs/patterns/item-inventory-quick-reference.md`

---

**Last Updated**: December 2024
**Status**: 🏗️ Planned - Ready for implementation
