### Inventory Improvements - Feature Plan

#### Overview

This plan details the implementation of several key inventory features to improve usability and testability. The goal is to create a robust foundation for future inventory-based mechanics.

The features to be added are:
1.  **Item Spawning for Testing:** A debug command to give the player items, and a temporary reduction of the slime ball stack size to facilitate testing full inventories.
2.  **Drag-and-Drop:** The ability to pick up, move, and swap items within and between inventory grids.
3.  **Shift-Click Transfers:** Quickly move items between the player's inventory and an open container.
4.  **Drag-to-Drop:** Drop items into the world by dragging them out of the inventory window.

---

#### Phase 1: Item Spawning & Test Preparation

This phase focuses on setting up the necessary conditions for effectively testing inventory functionality.

1.  **Reduce Slime Ball Stack Size:**
    *   **File:** `src/item/registry.rs`
    *   **Change:** Modify the `ItemDefinition` for `slime_ball` to set `max_stack_size` to `16`. This will make it easier to fill inventory slots for testing.

2.  **Implement Debug Item Spawning:**
    *   **File:** `src/main.rs`
    *   **Logic:** When the inventory window is open, pressing a debug key (e.g., `P`) will add a full stack of slime balls to the player's inventory.
    *   This will use the existing `player_inventory.quick_add()` method.

---

#### Phase 2: Drag-and-Drop Functionality

This phase implements the core mechanics for rearranging items.

1.  **Introduce "Held Item" State:**
    *   **File:** `src/gui/inventory_ui.rs` (or a new UI state struct)
    *   **Logic:** Create a state to hold an `Option<ItemStack>` that represents the item stack currently attached to the mouse cursor.

2.  **Implement Mouse Handling:**
    *   **File:** `src/main.rs` (event loop) & `src/gui/inventory_ui.rs`
    *   **On `MouseButtonDown`:**
        *   If the cursor is over an inventory slot containing an item and no item is currently held, pick it up (move it from the slot to the `held_item` state).
        *   If an item is held and the slot is empty, place it in the slot.
        *   If an item is held and the slot is occupied, swap the held item with the item in the slot.
    *   This will require functions to map mouse coordinates to inventory slot indices.

3.  **Render Held Item:**
    *   **File:** `src/gui/inventory_ui.rs`
    *   **Logic:** If an item is being held, render its texture centered on the mouse cursor during the `render` phase.

---

#### Phase 3: Advanced Interactions

This phase builds on the drag-and-drop system to add more complex actions.

1.  **Implement Shift-Click Transfer:**
    *   **File:** `src/main.rs` (event loop)
    *   **Logic:** On a `MouseButtonDown` event, if the `Shift` key is also held:
        *   Identify the clicked slot.
        *   Use the `transfer_slot_to` method (from `src/inventory/inventory.rs`) to move the item to the other inventory (player to container, or container to player).
        *   This will require tracking which container is currently open.

2.  **Implement Drag-to-Drop:**
    *   **File:** `src/main.rs` (event loop)
    *   **Logic:** If a `MouseButtonUp` event occurs while an item is held, and the cursor is outside the bounds of any open inventory window:
        *   Spawn a `DroppedItem` entity at the player's position.
        *   Clear the `held_item` state.
        *   This will use the `spawn_dropped_item` function.

---

#### Testing Strategy

Once implemented, the features will be tested manually with the following checks:

1.  **Item Spawning:**
    *   ✅ Open inventory, press `P`. A stack of 16 slime balls appears.
    *   ✅ Repeat until inventory is full. Verify no more items can be added.

2.  **Drag-and-Drop:**
    *   ✅ Click an item stack to pick it up. Verify it renders at the cursor.
    *   ✅ Click an empty slot to place it.
    *   ✅ Click an occupied slot to swap it with the held item.
    *   ✅ Verify that items can be moved between the hotbar and the main inventory grid.

3.  **Shift-Click:**
    *   ✅ With a container open, shift-click an item in the player inventory. Verify it moves to the container.
    *   ✅ Shift-click an item in the container. Verify it moves to the player inventory.

4.  **Drag-to-Drop:**
    *   ✅ Drag an item from the inventory window and release it outside the window bounds.
    *   ✅ Verify the item disappears from the cursor and a `DroppedItem` entity appears in the world.