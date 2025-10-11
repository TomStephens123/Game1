# Item & Inventory System - Quick Reference

## For Developers: Common Tasks

This guide provides quick examples for common item/inventory operations. For detailed design documentation, see the system docs.

---

## Adding a New Item

### Step 1: Register Item Definition

**File**: `src/item/registry.rs` (in `register_base_items()`)

```rust
// Basic material
self.register(ItemDefinition::new(
    "slime_ball",                              // Item ID
    "Slime Ball",                              // Display name
    "A bouncy ball of slime.",                 // Description
    "assets/items/slime_ball.png",             // Sprite path
    64,                                        // Max stack size
    ItemProperties::Material,                  // Type
)).expect("Failed to register slime_ball");

// Consumable (health potion)
self.register(ItemDefinition::new(
    "health_potion",
    "Health Potion",
    "Restores 50 health.",
    "assets/items/health_potion.png",
    16,
    ItemProperties::Consumable {
        effect: ConsumableEffect::Heal(50.0),
        use_time: 1.0,
    },
)).expect("Failed to register health_potion");

// Equipment (sword)
self.register(ItemDefinition::new(
    "iron_sword",
    "Iron Sword",
    "A sturdy blade.",
    "assets/items/iron_sword.png",
    1,  // Non-stackable
    ItemProperties::Equipment {
        slot: EquipmentSlot::MainHand,
        modifiers: vec![
            ModifierEffect::new(StatType::AttackDamage, 5.0, ModifierType::Flat),
        ],
    },
)).expect("Failed to register iron_sword");
```

### Step 2: Create Item Sprite

- **Location**: `assets/items/{item_id}.png`
- **Size**: 16x16 pixels (scales to 32x32 in-game)
- **Format**: PNG with transparency
- **Example**: `assets/items/slime_ball.png`

---

## Spawning Dropped Items

### When Enemy Dies

**File**: `src/main.rs` (in game loop)

```rust
// Check for dead enemies
for slime in &slimes {
    if !slime.is_alive && slime.animation_controller.is_animation_finished() {
        // Spawn dropped item
        spawn_dropped_item(
            &mut dropped_items,
            slime.x,
            slime.y,
            "slime_ball",  // Item ID
            1,             // Quantity
            &item_registry,
            &texture_creator,
        ).ok();
    }
}

// Remove dead enemies after spawning drops
slimes.retain(|s| s.is_alive || !s.animation_controller.is_animation_finished());
```

### When Player Drops Item

```rust
// Drop item from hotbar (Q key)
if keyboard_state.is_scancode_pressed(Scancode::Q) {
    if let Some(stack) = player_inventory.get_selected_hotbar() {
        let drop_x = player.x + 32;  // In front of player
        let drop_y = player.y;

        spawn_dropped_item(
            &mut dropped_items,
            drop_x,
            drop_y,
            &stack.item_id,
            1,  // Drop 1 item
            &item_registry,
            &texture_creator,
        ).ok();

        // Remove from inventory
        player_inventory.inventory.take_from_slot(
            player_inventory.selected_hotbar_slot,
            1,
        );
    }
}
```

### When Container Breaks

```rust
// Drop all items from chest
if let Some(chest) = get_container_at(block_x, block_y) {
    for (item_id, quantity) in chest.drop_contents() {
        spawn_dropped_item(
            &mut dropped_items,
            block_x * TILE_SIZE,
            block_y * TILE_SIZE,
            &item_id,
            quantity,
            &item_registry,
            &texture_creator,
        ).ok();
    }

    // Remove chest block
    world.remove_block(block_x, block_y);
}
```

---

## Inventory Operations

### Add Item to Player Inventory

```rust
use crate::inventory::error::InventoryError;

// Add 10 slime balls
match player_inventory.quick_add("slime_ball", 10, &item_registry) {
    Ok(overflow) => {
        if overflow == 0 {
            println!("Added 10 slime balls");
        } else {
            println!("Added {} slime balls, {} didn't fit", 10 - overflow, overflow);
        }
    }
    Err(e) => {
        eprintln!("Failed to add item: {}", e);
    }
}
```

### Remove Item from Inventory

```rust
// Remove 5 slime balls
let removed = player_inventory.inventory.remove_item("slime_ball", 5);

if removed == 5 {
    println!("Removed 5 slime balls");
} else {
    println!("Only removed {} slime balls (not enough in inventory)", removed);
}
```

### Check if Player Has Item

```rust
// Check if player has at least 10 slime balls
if player_inventory.has_item("slime_ball", 10) {
    println!("Player has enough slime balls for crafting!");
} else {
    println!("Not enough slime balls");
}
```

### Transfer Item Between Inventories

```rust
// Transfer 20 slime balls from player to chest
match player_inventory.inventory.transfer_to(
    &mut chest.inventory,
    "slime_ball",
    20,
    &item_registry,
) {
    Ok(transferred) => {
        println!("Transferred {} slime balls to chest", transferred);
    }
    Err(e) => {
        eprintln!("Transfer failed: {}", e);
    }
}
```

### Quick Transfer Slot (Shift+Click)

```rust
// Transfer entire slot from player to chest
if let Err(e) = player_inventory.inventory.transfer_slot_to(
    slot_index,
    &mut chest.inventory,
    &item_registry,
) {
    eprintln!("Quick transfer failed: {}", e);
}
```

---

## Item Pickup Logic

### Automatic Pickup (Collision-Based)

**File**: `src/main.rs` (in game loop)

```rust
// Handle item pickup
let player_bounds = player.get_bounds();
let mut picked_up_indices = Vec::new();

for (index, item) in dropped_items.iter().enumerate() {
    if !item.can_pickup {
        continue;  // Skip cooldown items
    }

    if player_bounds.has_intersection(item.get_bounds()) {
        // Try to add to inventory
        match player_inventory.quick_add(&item.item_id, item.quantity, &item_registry) {
            Ok(overflow) => {
                if overflow == 0 {
                    // Full pickup
                    picked_up_indices.push(index);
                    println!("Picked up {} x{}", item.item_id, item.quantity);

                    // Optional: Play pickup sound
                    // audio.play_sound("pickup");
                }
                // If overflow > 0, item stays on ground with reduced quantity
                // (Requires mutable access to item, handle separately)
            }
            Err(e) => {
                eprintln!("Pickup failed: {}", e);
            }
        }
    }
}

// Remove picked up items
for &index in picked_up_indices.iter().rev() {
    dropped_items.remove(index);
}
```

---

## Using Items

### Consumable Items (Health Potion)

```rust
// Use item from hotbar
if keyboard_state.is_scancode_pressed(Scancode::E) {  // Use key
    if let Some(stack) = player_inventory.get_selected_hotbar() {
        if let Some(item_def) = item_registry.get(&stack.item_id) {
            match &item_def.properties {
                ItemProperties::Consumable { effect, use_time } => {
                    // Apply effect
                    match effect {
                        ConsumableEffect::Heal(amount) => {
                            player.stats.heal(*amount);
                            println!("Healed {} HP", amount);
                        }
                        ConsumableEffect::Buff { modifier, duration } => {
                            player.active_modifiers.push(modifier.clone());
                            // TODO: Add timed buff removal
                        }
                        _ => {}
                    }

                    // Remove 1 item
                    player_inventory.inventory.take_from_slot(
                        player_inventory.selected_hotbar_slot,
                        1,
                    );
                }
                _ => {
                    println!("Item cannot be used");
                }
            }
        }
    }
}
```

### Equipment Items (Sword)

```rust
// Equip from inventory (future: equipment slots)
if let ItemProperties::Equipment { slot, modifiers } = &item_def.properties {
    // Remove from inventory
    player_inventory.inventory.take_from_slot(slot_index, 1);

    // Apply stat modifiers
    for modifier in modifiers {
        player.active_modifiers.push(modifier.clone());
    }

    // Store in equipment slot (future)
    // player.equipment.set_slot(*slot, item_stack);
}
```

---

## Container Inventory

### Create Chest

```rust
use crate::inventory::container::{ContainerInventory, ContainerType};

// Create chest at world position
let chest = ContainerInventory::new(
    ContainerType::Chest,  // 27 slots
    block_x,
    block_y,
);

// Store in world container list
world_containers.push(chest);
```

### Open/Close Container

```rust
// Right-click to open chest
if keyboard_state.is_scancode_pressed(Scancode::E) {
    if let Some(chest) = get_container_at_player_position(&world_containers, &player) {
        chest.open();
        ui_state.open_container = Some(chest.clone());  // Show UI
    }
}

// Close container (ESC key)
if keyboard_state.is_scancode_pressed(Scancode::Escape) {
    if let Some(chest_id) = ui_state.open_container {
        if let Some(chest) = get_container_by_id(&mut world_containers, chest_id) {
            chest.close();
        }
        ui_state.open_container = None;
    }
}
```

---

## Save/Load Integration

### Saving Player Inventory

**File**: `src/main.rs` (in `save_game()`)

```rust
// Save player inventory as part of player data
#[derive(Serialize)]
struct PlayerSaveData {
    // ... other player fields ...
    inventory: SaveData,
}

let player_data = PlayerSaveData {
    // ... other fields ...
    inventory: player_inventory.to_save_data()?,
};
```

### Saving Dropped Items

```rust
// Save all dropped items as entities
for (i, item) in dropped_items.iter().enumerate() {
    let item_save_data = item.to_save_data()
        .map_err(|e| format!("Failed to save dropped item {}: {}", i, e))?;

    entities.push(EntitySaveData {
        entity_id: (next_id + i) as u64,
        entity_type: "dropped_item".to_string(),
        position: (item.x, item.y),
        data: item_save_data.json_data,
    });
}
```

### Loading Player Inventory

**File**: `src/main.rs` (in `load_game()`)

```rust
// Load player inventory
let player_inv_data = /* ... extract from save file ... */;
let mut player_inventory = PlayerInventory::from_save_data(&player_inv_data)?;

// Validate items exist in registry
for slot in &mut player_inventory.inventory.slots {
    if let Some(stack) = slot {
        if !item_registry.exists(&stack.item_id) {
            eprintln!("Warning: Removed unknown item '{}' from inventory", stack.item_id);
            *slot = None;
        }
    }
}
```

### Loading Dropped Items

```rust
// Load dropped items from entities
"dropped_item" => {
    let mut loaded_item = DroppedItem::from_save_data(&save_data)
        .map_err(|e| format!("Failed to load dropped item: {}", e))?;

    // Validate item exists
    if !item_registry.exists(&loaded_item.item_id) {
        eprintln!("Warning: Skipped unknown item '{}'", loaded_item.item_id);
        continue;
    }

    // Set up animation controller
    if let Some(item_def) = item_registry.get(&loaded_item.item_id) {
        let texture = texture_creator.load_texture(&item_def.sprite_path)?;
        let animation_controller = create_item_animation_controller(texture);
        loaded_item.set_animation_controller(animation_controller);
    }

    dropped_items.push(loaded_item);
}
```

---

## UI Rendering (Future)

### Render Inventory Grid

**File**: `src/gui/inventory_ui.rs`

```rust
pub fn render_inventory(
    canvas: &mut Canvas<Window>,
    inventory: &Inventory,
    item_registry: &ItemRegistry,
) -> Result<(), String> {
    const SLOT_SIZE: u32 = 40;
    const GRID_START_X: i32 = 100;
    const GRID_START_Y: i32 = 100;

    // Render each slot
    for (index, slot) in inventory.slots.iter().enumerate() {
        let row = index / 9;
        let col = index % 9;
        let x = GRID_START_X + (col as i32 * SLOT_SIZE as i32);
        let y = GRID_START_Y + (row as i32 * SLOT_SIZE as i32);

        // Draw slot background
        canvas.set_draw_color(Color::RGB(50, 50, 50));
        canvas.fill_rect(Rect::new(x, y, SLOT_SIZE, SLOT_SIZE))?;

        // Draw item if slot occupied
        if let Some(stack) = slot {
            if let Some(item_def) = item_registry.get(&stack.item_id) {
                // Load and render item sprite
                // ... sprite rendering code ...

                // Render quantity text
                render_text(canvas, &format!("{}", stack.quantity), x + 2, y + 22)?;
            }
        }
    }

    Ok(())
}
```

---

## Common Patterns

### Item Count Check Pattern

```rust
// Pattern: Check before removing
if player_inventory.has_item("slime_ball", 5) {
    player_inventory.inventory.remove_item("slime_ball", 5);
    // Craft something...
} else {
    println!("Not enough materials!");
}
```

### Safe Transfer Pattern

```rust
// Pattern: Transfer with overflow handling
let removed = source_inv.remove_item("slime_ball", 20);

if removed > 0 {
    let overflow = dest_inv.add_item("slime_ball", removed, &item_registry)?;

    if overflow > 0 {
        // Put overflow back in source
        source_inv.add_item("slime_ball", overflow, &item_registry)?;
    }
}
```

### Item Validation Pattern

```rust
// Pattern: Always validate item IDs from save/network
if item_registry.exists(&item_id) {
    // Safe to use item
    inventory.add_item(&item_id, quantity, &item_registry)?;
} else {
    eprintln!("Warning: Unknown item ID '{}', skipping", item_id);
}
```

---

## Debugging Tips

### Check Inventory Contents

```rust
// Print all items in inventory
for (index, slot) in inventory.slots.iter().enumerate() {
    if let Some(stack) = slot {
        println!("Slot {}: {} x{}", index, stack.item_id, stack.quantity);
    }
}
```

### Verify Item Registry

```rust
// List all registered items
for id in item_registry.all_ids() {
    if let Some(item) = item_registry.get(id) {
        println!("Item: {} ({})", item.name, id);
    }
}
```

### Debug Dropped Items

```rust
// Print dropped item info
for item in &dropped_items {
    println!(
        "DroppedItem: {} x{} at ({}, {}) - despawn in {:.1}s",
        item.item_id,
        item.quantity,
        item.x,
        item.y,
        item.time_until_despawn()
    );
}
```

---

## Performance Tips

### Batch Operations

```rust
// Good: Single add operation
inventory.add_item("slime_ball", 100, &item_registry)?;

// Bad: Many small operations
for _ in 0..100 {
    inventory.add_item("slime_ball", 1, &item_registry)?;
}
```

### Item Merging

```rust
// Merge dropped items periodically (not every frame)
if frame_count % 60 == 0 {  // Once per second
    merge_nearby_items(&mut dropped_items, &item_registry);
}
```

### Lazy Sprite Loading

```rust
// Load item sprites on first use, not all at startup
if let Some(item_def) = item_registry.get(&item_id) {
    let sprite = sprite_cache.get_or_load(&item_def.sprite_path)?;
    // Use sprite...
}
```

---

## Error Handling

### Handle Inventory Errors

```rust
use crate::inventory::error::InventoryError;

match inventory.add_item("slime_ball", 10, &item_registry) {
    Ok(overflow) => {
        if overflow > 0 {
            println!("Warning: {} items didn't fit", overflow);
        }
    }
    Err(InventoryError::InvalidItem(id)) => {
        eprintln!("Error: Unknown item '{}'", id);
    }
    Err(InventoryError::InventoryFull) => {
        println!("Inventory is full!");
    }
    Err(e) => {
        eprintln!("Inventory error: {}", e);
    }
}
```

---

## Testing Checklist

### Item System Tests
- [ ] Register new item in registry
- [ ] Lookup item by ID
- [ ] Verify item properties (name, sprite, stack size)
- [ ] Create ItemStack with quantity
- [ ] Merge two ItemStacks
- [ ] Split an ItemStack

### Inventory Tests
- [ ] Add items to empty inventory
- [ ] Stack identical items
- [ ] Handle inventory overflow
- [ ] Remove items from inventory
- [ ] Transfer between inventories
- [ ] Save and load inventory

### Dropped Item Tests
- [ ] Spawn dropped item in world
- [ ] Player collision triggers pickup
- [ ] Item adds to player inventory
- [ ] Item despawns after timeout
- [ ] Nearby items merge automatically
- [ ] Dropped items save/load correctly

---

## Related Documentation

- **Feature Plan**: `docs/features/item-inventory-system.md`
- **Item System**: `docs/systems/item-system-design.md`
- **Inventory System**: `docs/systems/inventory-system-design.md`
- **Dropped Items**: `docs/systems/dropped-item-entity.md`
- **Save System**: `docs/patterns/save-system-quick-reference.md`

---

**Last Updated**: December 2024
**Quick Reference Version**: 1.0
