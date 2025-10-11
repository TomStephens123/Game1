# Tool System - Quick Reference

## Overview

Tools are a type of item that allow players to interact with the world (breaking blocks, tilling soil, etc.). This guide shows how to add new tools and implement tool-based interactions.

---

## Tool Types

Tools are defined in `src/item/properties.rs` as part of the `ItemProperties` enum:

```rust
ItemProperties::Tool {
    tool_type: ToolType,
    durability: u32,
    mining_speed: f32,
}
```

### Available Tool Types

Defined in `src/item/properties.rs`:

```rust
pub enum ToolType {
    Pickaxe,  // Mining stone/ore
    Axe,      // Chopping wood
    Shovel,   // Digging dirt/sand
    Hoe,      // Tilling soil
    Sword,    // Weapon tool
}
```

---

## Adding a New Tool

### Step 1: Register Tool in Item Registry

**File**: `src/item/registry.rs` (in `register_base_items()`)

```rust
// Hoe (farming tool)
self.register(ItemDefinition::new(
    "hoe",                                    // Item ID
    "Hoe",                                    // Display name
    "A simple farming tool for tilling soil.", // Description
    "assets/items/hoe.png",                   // Sprite path
    1,                                        // Max stack (tools don't stack)
    ItemProperties::Tool {
        tool_type: ToolType::Hoe,
        durability: 100,                      // Uses before breaking
        mining_speed: 1.0,                    // Speed multiplier
    },
)).expect("Failed to register hoe");
```

**Notes:**
- Tools are typically non-stackable (max_stack_size = 1)
- Durability determines how many uses before the tool breaks
- Mining speed affects how quickly blocks are broken/modified

### Step 2: Create Tool Sprite

- **Location**: `assets/items/{tool_id}.png`
- **Size**: 16x16 pixels (scales to 32x32 in-game)
- **Format**: PNG with transparency
- **Example**: `assets/items/hoe.png`

---

## Implementing Tool Actions

### Pattern: Check for Tool in Hotbar

Tools are typically used via mouse clicks when selected in the hotbar. Here's the standard pattern:

**File**: `src/main.rs` (in event loop, MouseButtonDown handler)

```rust
// Import tool types at the top of the file
use item::{ItemRegistry, ItemProperties, ToolType};

// In the MouseButtonDown event handler:
if !is_ui_active {
    // Check if player has the required tool selected
    if let Some(selected_item) = player_inventory.get_selected_hotbar() {
        if let Some(item_def) = item_registry.get(&selected_item.item_id) {
            // Match the specific tool type
            if let ItemProperties::Tool { tool_type: ToolType::Hoe, .. } = item_def.properties {
                // Perform tool-specific action
                let tile_x = x / 32;
                let tile_y = y / 32;

                // Only allow grass -> dirt conversion
                if world_grid.get_tile(tile_x, tile_y) == Some(TileId::Grass) {
                    if world_grid.set_tile(tile_x, tile_y, TileId::Dirt) {
                        render_grid.update_tile_and_neighbors(&world_grid, tile_x, tile_y);
                        // Optional: Reduce tool durability here
                    }
                }
            }
        }
    }
}
```

### Pattern: Continuous Tool Use (Drag to Use)

For tools that should work while dragging (like the hoe for tilling multiple tiles):

```rust
// Track tool use state
let mut is_using_tool = false;
let mut last_used_tile: Option<(i32, i32)> = None;

// In MouseButtonDown event:
if let ItemProperties::Tool { tool_type: ToolType::Hoe, .. } = item_def.properties {
    is_using_tool = true;
    // Perform action...
}

// In MouseMotion event:
if is_using_tool && !is_ui_active {
    let tile_x = x / 32;
    let tile_y = y / 32;

    // Only use tool on new tile
    if last_used_tile != Some((tile_x, tile_y)) {
        // Perform tool action...
        last_used_tile = Some((tile_x, tile_y));
    }
}

// In MouseButtonUp event:
is_using_tool = false;
last_used_tile = None;
```

---

## Tool Durability (Future Enhancement)

Currently, tools have a durability property but it's not yet implemented. Here's the planned pattern:

```rust
// When tool is used:
if let ItemProperties::Tool { tool_type, durability, .. } = item_def.properties {
    // Perform tool action...

    // Reduce durability
    let current_durability = player_inventory.get_item_durability(selected_slot);
    if current_durability > 1 {
        player_inventory.reduce_item_durability(selected_slot, 1);
    } else {
        // Tool breaks
        player_inventory.inventory.slots[selected_slot] = None;
        println!("Your {} broke!", item_def.name);
    }
}
```

**Note**: Item durability tracking requires extending the `ItemStack` struct to include a durability field. This is not yet implemented.

---

## Adding a New Tool Type

### Step 1: Add to ToolType Enum

**File**: `src/item/properties.rs`

```rust
pub enum ToolType {
    Pickaxe,
    Axe,
    Shovel,
    Hoe,
    Sword,
    Hammer,  // New tool type
}
```

### Step 2: Register Tool Item

Add the tool to the item registry (see "Adding a New Tool" above).

### Step 3: Implement Tool Action

Add tool-specific logic in the event loop (see "Implementing Tool Actions" above).

**Example: Hammer for Breaking Blocks**

```rust
if let ItemProperties::Tool { tool_type: ToolType::Hammer, .. } = item_def.properties {
    let tile_x = x / 32;
    let tile_y = y / 32;

    // Break block at this position
    if let Some(block) = world.get_block_at(tile_x, tile_y) {
        // Drop block as item
        spawn_dropped_item(&mut dropped_items, tile_x * 32, tile_y * 32, &block.item_id, 1, &item_registry, &texture_creator)?;

        // Remove block from world
        world.remove_block(tile_x, tile_y);
    }
}
```

---

## Common Tool Patterns

### 1. World Modification Tool (Hoe, Shovel)

```rust
// Check tile state, modify if valid
if world_grid.get_tile(tile_x, tile_y) == Some(TileId::Grass) {
    world_grid.set_tile(tile_x, tile_y, TileId::Dirt);
    render_grid.update_tile_and_neighbors(&world_grid, tile_x, tile_y);
}
```

### 2. Block Breaking Tool (Pickaxe, Axe)

```rust
// Break block and drop items
if let Some(block) = world.get_block_at(tile_x, tile_y) {
    // Drop block item
    spawn_dropped_item(...);

    // Remove block
    world.remove_block(tile_x, tile_y);
}
```

### 3. Combat Tool (Sword)

```rust
// Deal damage to entities
if let Some(target) = find_entity_at(x, y) {
    let damage = calculate_damage(&item_def, &player);
    target.take_damage(damage);
}
```

### 4. Placement Tool (Watering Can, Seeds)

```rust
// Place or spawn entities
if world_grid.get_tile(tile_x, tile_y) == Some(TileId::Dirt) {
    // Spawn crop entity
    let crop = Crop::new(tile_x * 32, tile_y * 32, "wheat");
    crops.push(crop);
}
```

---

## Exporting Tool Types

Make sure `ToolType` is exported from the item module for use in main.rs:

**File**: `src/item/mod.rs`

```rust
pub use properties::{ItemProperties, ToolType};
```

---

## Testing Tools

### Debug Command for Giving Tools

Add a debug key to give yourself tools for testing:

**File**: `src/main.rs`

```rust
Event::KeyDown {
    keycode: Some(Keycode::H),
    ..
} if inventory_ui.is_open => {
    // Give a hoe to the player (debug command)
    match player_inventory.quick_add("hoe", 1, &item_registry) {
        Ok(0) => println!("Added hoe to inventory."),
        Ok(_) => println!("Inventory full. Hoe could not be added."),
        Err(e) => eprintln!("Error adding hoe: {}", e),
    }
}
```

### Testing Checklist

- [ ] Tool appears in inventory with correct sprite
- [ ] Tool can be selected in hotbar
- [ ] Left-click with tool performs expected action
- [ ] Tool only works on valid targets (e.g., hoe only tills grass)
- [ ] Dragging with tool works (if applicable)
- [ ] Tool action stops when mouse is released
- [ ] Tool doesn't interfere with UI interactions

---

## Architecture Notes

### Why Tools Are Items, Not Equipment

Tools could have been implemented as `Equipment` items, but we use the `Tool` variant because:

1. **Different behavior**: Tools modify the world; equipment provides passive stat bonuses
2. **Usage pattern**: Tools are used via clicks; equipment is worn
3. **Durability**: Tools degrade with use; equipment typically doesn't
4. **Future flexibility**: Allows for tool-specific mechanics (mining levels, speed bonuses, etc.)

### Tool vs. Item Interaction

The key difference:
- **Items**: Added to inventory, can be consumed/equipped
- **Tools**: A type of item with special interaction capabilities

Tools bridge the gap between inventory items and world interaction, providing a clean pattern for player-driven world modification.

---

## Related Documentation

- **Item System**: `docs/patterns/item-inventory-quick-reference.md`
- **Item Properties**: `src/item/properties.rs`
- **Item Registry**: `src/item/registry.rs`

---

**Last Updated**: January 2025
**Quick Reference Version**: 1.0
