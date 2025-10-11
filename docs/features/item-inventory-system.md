# Item & Inventory System - Feature Plan

## Overview

A comprehensive item and inventory system for Game1 that supports:
- **Player inventory** with hotbar (9 slots) and main inventory (18 slots)
- **Container inventories** (chests, storage blocks)
- **Automation inventories** (future: crafting stations, furnaces, etc.)
- **Dropped items** as pickable world entities
- **Item definitions** with extensible properties
- **UI rendering** using the existing screen-space GUI system
- **Full save/load support** for all items and inventories

**Status**: 🏗️ **PLANNED** - December 2024

## Philosophy

The item/inventory system is designed with three core principles:

1. **Separation of Concerns**
   - Items are data definitions (properties, ID, metadata)
   - Inventories are containers (slots, stacks, transfer logic)
   - Dropped items are entities (physics, collision, rendering)
   - UI is separate (displays inventory state, doesn't own it)

2. **Extensibility**
   - Easy to add new item types without modifying core systems
   - Item registry pattern for centralized definitions
   - Trait-based system for special item behaviors

3. **Integration**
   - Seamless save/load using existing `Saveable` trait
   - Items can provide stat modifiers to the existing stats system
   - Uses UI system architecture (screen-space GUI for inventory windows)
   - Dropped items use animation and collision systems

## System Architecture

### High-Level Component Diagram

```
┌─────────────────┐
│  Item Registry  │ ← Central item definitions
└────────┬────────┘
         │
         ↓
┌─────────────────┐     ┌──────────────────┐
│  Item Instance  │ ←── │ Inventory Slots  │
└────────┬────────┘     └────────┬─────────┘
         │                       │
         ↓                       ↓
┌─────────────────┐     ┌──────────────────┐
│ Dropped Item    │     │ Player/Container │
│ (Entity)        │     │ Inventory        │
└─────────────────┘     └──────────────────┘
         │                       │
         ↓                       ↓
    [Animation]           [Screen-Space GUI]
    [Collision]           [Save System]
```

### Core Components

1. **Item System** (`src/item/mod.rs`)
   - Item definitions and registry
   - Item IDs and metadata
   - Item properties (stackable, consumable, equipment, etc.)
   - See: `docs/systems/item-system-design.md`

2. **Inventory System** (`src/inventory/mod.rs`)
   - Generic inventory container
   - Slot management and stacking
   - Item transfer logic
   - Player/block inventory implementations
   - See: `docs/systems/inventory-system-design.md`

3. **Dropped Item Entity** (`src/dropped_item.rs`)
   - World entity for items on the ground
   - Pickup collision detection
   - Animation and rendering
   - See: `docs/systems/dropped-item-entity.md`

4. **Inventory UI** (`src/gui/inventory_ui.rs`)
   - Screen-space GUI windows
   - Player inventory display (hotbar + main)
   - Container inventory display (chests)
   - Item tooltips and interaction
   - Uses: Screen-space GUI system (`docs/systems/ui-system.md`)

## User-Facing Features

### Phase 1: Core Functionality ✅ Foundation

#### Player Inventory
- **Hotbar**: 9 slots for quick access items
- **Main Inventory**: 18 additional slots (2 rows of 9)
- **Total Capacity**: 27 item slots
- **Controls**:
  - `I` key to open/close inventory
  - `1-9` keys to select hotbar slot (future: quick-use items)
  - Mouse click to pick up/place items
  - Shift+Click for quick transfer (future)

#### Item Picking
- Walk over dropped items to automatically pick up
- Items add to existing stacks if possible
- Full inventory prevents pickup (item stays on ground)
- Visual/audio feedback on pickup

#### Item Dropping
- Drop item from inventory (future: `Q` key or drag out)
- Item spawns as entity in world
- Dropped items persist in save files
- Items despawn after 5 minutes (configurable)

#### Basic Items
- **Slime Ball**: Dropped by slimes on death
  - Stackable (max 64)
  - Sprite: `assets/items/slime_ball.png`
  - No special properties (crafting material)

### Phase 2: Containers 🔜 Next

#### Chest Blocks
- Place chests in world (future: crafting)
- Right-click to open chest inventory
- 27 slots (same size as player inventory)
- Shared inventory UI code
- Visual indicator when chest is open

#### Container Interaction
- Open multiple containers simultaneously
- Transfer items between player and container
- Containers save their contents
- Containers drop their items when destroyed

### Phase 3: Advanced Features 🔮 Future

#### Item Types
- **Consumables**: Potions, food (use from hotbar)
- **Equipment**: Weapons, armor (equip from inventory)
- **Tools**: Pickaxe, axe (interact with blocks)
- **Crafting Materials**: Base items for recipes

#### Automation Blocks
- **Crafting Station**: Recipe-based item creation
- **Furnace**: Smelting with fuel and input/output
- **Hopper**: Auto-transfer items between containers

#### Item Properties
- **Durability**: Tools break after X uses
- **Stack Sizes**: Different limits per item type
- **Rarity**: Common, Rare, Epic, Legendary
- **Stat Modifiers**: Items provide buffs (attack damage, speed, etc.)

## Implementation Roadmap

### Phase 1: Foundation (Est. 8-12 hours)

1. **Item System Core** (3-4 hours)
   - [ ] Create `src/item/` module
   - [ ] Implement `ItemDefinition` struct
   - [ ] Create `ItemRegistry` for centralized definitions
   - [ ] Add `ItemStack` for quantity management
   - [ ] Implement `Saveable` for items
   - **Deliverable**: Can create item definitions and instances

2. **Dropped Item Entity** (2-3 hours)
   - [ ] Create `src/dropped_item.rs`
   - [ ] Implement entity with animation system
   - [ ] Add collision detection for pickup
   - [ ] Implement `Saveable` for persistence
   - [ ] Add pickup logic (collision with player)
   - **Deliverable**: Slime balls drop on slime death and can be picked up

3. **Inventory Container** (2-3 hours)
   - [ ] Create `src/inventory/` module
   - [ ] Implement `Inventory` struct with slots
   - [ ] Add item stacking logic
   - [ ] Implement add/remove/transfer methods
   - [ ] Implement `Saveable` for inventory
   - **Deliverable**: Player has working inventory storage

4. **Basic UI** (1-2 hours)
   - [ ] Create `src/gui/inventory_ui.rs`
   - [ ] Render inventory grid (9x3 layout)
   - [ ] Show item sprites in slots
   - [ ] Display stack quantities
   - [ ] Add open/close keybinding (`I`)
   - **Deliverable**: Can view inventory contents

**Phase 1 Success Criteria**:
- ✅ Kill a slime → slime ball drops
- ✅ Walk over slime ball → automatically picked up
- ✅ Press `I` → inventory opens showing slime ball
- ✅ Save game → reload → slime ball still in inventory

### Phase 2: Containers & Interaction (Est. 6-8 hours)

5. **Container Blocks** (3-4 hours)
   - [ ] Create `Chest` block type
   - [ ] Add chest inventory (27 slots)
   - [ ] Implement block placement/destruction
   - [ ] Add "open chest" interaction (right-click)
   - [ ] Container UI rendering

6. **Item Transfer** (2-3 hours)
   - [ ] Click to pick up/place items
   - [ ] Split stacks (right-click)
   - [ ] Quick transfer (shift+click)
   - [ ] Drag and drop items

7. **Polish** (1 hour)
   - [ ] Item tooltips (hover for name/description)
   - [ ] Pickup sound effects
   - [ ] Inventory full indicator
   - [ ] Visual feedback for actions

**Phase 2 Success Criteria**:
- ✅ Place chest in world
- ✅ Right-click chest → opens inventory
- ✅ Transfer slime balls to chest
- ✅ Close chest → reopen → items still there
- ✅ Destroy chest → items drop as entities

### Phase 3: Advanced Items (Est. 10-15 hours)

8. **Item Types & Properties** (4-5 hours)
   - [ ] Consumable items (health potions)
   - [ ] Equipment system (sword, armor)
   - [ ] Tool system (pickaxe durability)
   - [ ] Stat modifiers from items

9. **Automation Blocks** (4-6 hours)
   - [ ] Crafting station with recipes
   - [ ] Furnace with smelting
   - [ ] Hopper for auto-transfer

10. **Recipes & Crafting** (2-4 hours)
    - [ ] Recipe definition system
    - [ ] Crafting UI
    - [ ] Item combination logic

## Technical Design Highlights

### Item Definition System

Items are defined centrally in a registry:

```rust
pub struct ItemDefinition {
    pub id: String,              // "slime_ball"
    pub name: String,            // "Slime Ball"
    pub sprite_path: String,     // Path to item sprite
    pub max_stack_size: u32,     // 64 for slime ball
    pub properties: ItemProperties,
}

pub enum ItemProperties {
    Material,                    // Basic crafting material
    Consumable { effect: Effect },
    Equipment { slot: EquipSlot, stats: StatModifiers },
    Tool { durability: u32, tool_type: ToolType },
}
```

### Inventory Slot System

Inventories use a slot-based system with stacking:

```rust
pub struct Inventory {
    pub slots: Vec<Option<ItemStack>>,
    pub capacity: usize,
}

pub struct ItemStack {
    pub item_id: String,         // Reference to ItemRegistry
    pub quantity: u32,
}

impl Inventory {
    pub fn add_item(&mut self, item_id: &str, quantity: u32) -> Result<u32, InventoryError>;
    pub fn remove_item(&mut self, slot: usize, quantity: u32) -> Option<ItemStack>;
    pub fn transfer_to(&mut self, other: &mut Inventory, slot: usize) -> Result<(), String>;
}
```

### Dropped Item Entity Pattern

Dropped items are full entities (not UI):

```rust
pub struct DroppedItem<'a> {
    pub x: i32,
    pub y: i32,
    pub item_id: String,
    pub quantity: u32,
    animation_controller: AnimationController<'a>,
    spawn_time: Instant,
    pub can_pickup: bool,       // Cooldown before pickup
}

impl Collidable for DroppedItem<'_> {
    fn get_bounds(&self) -> Rect { /* ... */ }
    fn get_collision_layer(&self) -> CollisionLayer {
        CollisionLayer::Item  // New layer for item pickup
    }
}
```

### Save System Integration

All components implement `Saveable`:

```rust
// Items save their ID and quantity
impl Saveable for ItemStack {
    fn to_save_data(&self) -> Result<SaveData, SaveError> {
        // Serialize item_id and quantity
    }
}

// Inventories save all non-empty slots
impl Saveable for Inventory {
    fn to_save_data(&self) -> Result<SaveData, SaveError> {
        // Serialize slots as array of ItemStack
    }
}

// Dropped items save position and item data
impl Saveable for DroppedItem<'_> {
    fn to_save_data(&self) -> Result<SaveData, SaveError> {
        // Serialize position, item_id, quantity
    }
}
```

## UI Integration

### Screen-Space GUI Usage

Per the UI system architecture (`docs/systems/ui-system.md`):

- **Inventory windows** are screen-space GUI (fixed screen position)
- **Dropped items** are entities (use animation system, not UI)
- **Item labels** (optional) are world-space HUD (hover text above item)

```rust
// src/gui/inventory_ui.rs
pub struct InventoryWindow {
    pub is_open: bool,
    position: (i32, i32),        // Screen coordinates
    size: (u32, u32),
    slot_size: u32,              // Size of each slot
}

impl InventoryWindow {
    pub fn render(
        &self,
        canvas: &mut Canvas<Window>,
        inventory: &Inventory,
        item_registry: &ItemRegistry,
    ) -> Result<(), String> {
        // Render at fixed screen position
        // Draw grid, item sprites, quantities
    }

    pub fn handle_click(&mut self, mouse_x: i32, mouse_y: i32) -> Option<usize> {
        // Return clicked slot index
    }
}
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_inventory_add_item() {
        let mut inv = Inventory::new(27);
        inv.add_item("slime_ball", 10).unwrap();
        assert_eq!(inv.count_item("slime_ball"), 10);
    }

    #[test]
    fn test_item_stacking() {
        let mut inv = Inventory::new(1);
        inv.add_item("slime_ball", 30).unwrap();
        inv.add_item("slime_ball", 30).unwrap();
        // Should stack in same slot
        assert_eq!(inv.slots[0].as_ref().unwrap().quantity, 60);
    }

    #[test]
    fn test_inventory_full() {
        let mut inv = Inventory::new(1);
        inv.add_item("slime_ball", 64).unwrap(); // Fill slot
        let result = inv.add_item("slime_ball", 1);
        assert!(result.is_err()); // Should fail (inventory full)
    }
}
```

### Integration Tests

**Test Case 1: Slime Ball Drop & Pickup**
1. Spawn slime
2. Kill slime (reduce health to 0)
3. Verify `DroppedItem` entity spawns
4. Move player to item position
5. Verify collision triggers pickup
6. Verify item added to player inventory
7. Verify dropped item removed from world

**Test Case 2: Inventory Persistence**
1. Add 10 slime balls to player inventory
2. Save game (F5)
3. Modify inventory (add/remove items)
4. Load game (F9)
5. Verify inventory restored to save state

**Test Case 3: Chest Storage**
1. Place chest block
2. Add items to chest
3. Close and reopen chest
4. Verify items persist
5. Save and reload game
6. Verify chest contents restored

### Playtesting Checklist

Phase 1 (Slime Ball Test):
- [ ] Kill 5 slimes → each drops slime ball
- [ ] Pick up all slime balls → inventory shows 5
- [ ] Press `I` → inventory opens correctly
- [ ] Save and reload → slime balls persist
- [ ] Drop 1 slime ball → appears in world
- [ ] Walk away and return → item still there
- [ ] Pick it back up → inventory shows 5 again

Phase 2 (Chest Test):
- [ ] Place chest in world
- [ ] Transfer all 5 slime balls to chest
- [ ] Close chest → reopen → items still there
- [ ] Break chest → items drop as entities
- [ ] Pick up items → back in player inventory
- [ ] Save/load → chest contents persist

## Performance Considerations

### Item Registry Optimization
- **Lazy loading**: Only load item definitions when first accessed
- **String interning**: Use `ItemId` enum instead of String for lookups
- **Asset caching**: Load item sprites once, share between instances

### Dropped Item Management
- **Spatial partitioning**: Only check pickup collisions for nearby items
- **Despawn timer**: Remove items after 5 minutes (configurable)
- **Merge nearby items**: Combine dropped stacks within 1 tile radius
- **Limit cap**: Max 1000 dropped items in world (oldest despawn first)

### Inventory Operations
- **Slot indexing**: O(1) access to specific slots
- **Stack search**: O(n) scan for matching items (fast for small inventories)
- **Batch transfers**: Single operation for shift+click transfers
- **UI updates**: Only redraw inventory when contents change

## Rust Learning Opportunities

This system teaches:

1. **Module Organization** (Chapter 7)
   - Creating multi-file modules (`src/item/mod.rs`, `src/inventory/mod.rs`)
   - Re-exporting types for clean public API
   - Internal vs public types

2. **Enums and Pattern Matching** (Chapter 6)
   - `ItemProperties` enum for different item types
   - `Option<ItemStack>` for empty/full slots
   - Match expressions for item behavior

3. **Trait Objects** (Chapter 17)
   - `Box<dyn ItemBehavior>` for custom item actions
   - Trait-based item effects
   - Dynamic dispatch for item use

4. **Collections** (Chapter 8)
   - `Vec<Option<ItemStack>>` for inventory slots
   - `HashMap<String, ItemDefinition>` for item registry
   - Efficient searching and filtering

5. **Error Handling** (Chapter 9)
   - Custom `InventoryError` type
   - `Result<T, E>` for fallible operations
   - Error propagation with `?`

## Security & Validation

### Item Duplication Prevention
- **Server authority**: Future multiplayer validates all item operations
- **Save validation**: Check item IDs exist in registry on load
- **Quantity limits**: Enforce max stack size in all operations

### Invalid State Handling
- **Missing items**: Gracefully handle deleted item definitions
- **Corrupted saves**: Remove invalid items, log warnings
- **Desync protection**: Validate inventory state before save

## Future Enhancements

### Quality of Life
- [ ] Item sorting (by type, name, quantity)
- [ ] Search/filter in large inventories
- [ ] Favorite items (pin to top)
- [ ] Quick stack (move all matching items)

### Advanced Features
- [ ] Item durability with repair
- [ ] Enchantments/upgrades
- [ ] Item sets with bonuses
- [ ] Cursed items (can't drop)
- [ ] Unique/legendary items

### Multiplayer Preparation
- [ ] Inventory sync protocol
- [ ] Trade system (player-to-player)
- [ ] Item drop on death
- [ ] Loot tables for enemies

## Related Documentation

- **Item System Design**: `docs/systems/item-system-design.md` (core item definitions)
- **Inventory System Design**: `docs/systems/inventory-system-design.md` (container logic)
- **Dropped Item Entity**: `docs/systems/dropped-item-entity.md` (world item entities)
- **Quick Reference**: `docs/patterns/item-inventory-quick-reference.md` (developer guide)
- **UI System**: `docs/systems/ui-system.md` (screen-space GUI architecture)
- **Save System**: `docs/systems/save-system-design.md` (persistence integration)

## Summary

The Item & Inventory System provides:

✅ **Extensible** - Easy to add new items and inventory types
✅ **Integrated** - Works with save, UI, collision, and stats systems
✅ **Type-Safe** - Rust's type system prevents common inventory bugs
✅ **Performant** - Optimized for 1000s of items and fast operations
✅ **Testable** - Comprehensive unit and integration tests
✅ **Learnable** - Clear patterns for adding items and features

**Next Steps**:
1. Review this plan and system designs
2. Begin Phase 1 implementation (Item System Core)
3. Test with slime ball drops
4. Iterate based on playtesting feedback

---

**Last Updated**: December 2024
**Status**: 🏗️ Planned - Ready for implementation
**Implementation Time**: 24-35 hours (all 3 phases)
