use crate::item::{ItemStack, ItemRegistry};
use super::error::InventoryError;
use serde::{Serialize, Deserialize};

/// Generic inventory container with slots
///
/// This is the core storage structure used by all inventory types.
/// It handles slot management, stacking, and basic operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.slots.iter().all(|slot| slot.is_none())
    }

    /// Returns true if all slots are occupied
    #[allow(dead_code)]
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
                    let overflow = stack.add(remaining, max_stack_size);
                    remaining = overflow;
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.slots.fill(None);
    }

    /// Returns an iterator over all non-empty item stacks
    #[allow(dead_code)]
    pub fn iter_items(&self) -> impl Iterator<Item = &ItemStack> {
        self.slots.iter().filter_map(|slot| slot.as_ref())
    }
}
