# Dropped Item Entity Design

## Overview

Dropped items are **world entities** (not UI elements) that represent physical items on the ground. They can be picked up by players, persist in save files, and use the existing animation and collision systems.

**Key Characteristics**:
- Full entity with position, sprite, and animation
- Collision detection for player pickup
- Automatic despawn after timeout
- Save/load support for persistence
- Depth sorting for proper rendering
- Optional: Magnetic attraction to player

**Status**: 🏗️ **PLANNED** - December 2024

## Core Concepts

### Why Entities, Not UI?

As defined in `docs/systems/ui-system.md`, dropped items are entities because:

1. **World Position** - They exist at (x, y) in the game world
2. **Physics** - They can fall, bounce, collide with terrain
3. **Player Interaction** - Players walk to them to pick up
4. **Persistence** - They save/load with world state
5. **Animation** - They use the animation system (idle, glow, bob)
6. **Collision** - They implement `Collidable` trait

**Rendering Layer**: Entities (layer 2), NOT UI layers

### Lifecycle

```
1. Spawn (enemy death, player drop, chest destroyed)
   ↓
2. World entity (animate, collision, render)
   ↓
3. Player collision → Pickup (add to inventory)
   ↓
4. Remove from world

Alternative:
   ↓
5. Timeout (5 minutes) → Despawn
```

## Architecture

### File Structure

```
src/dropped_item.rs     # Main entity implementation
```

(Single file, not a module - follows pattern of slime.rs, player.rs)

### Component 1: DroppedItem Entity

**File**: `src/dropped_item.rs`

```rust
use crate::animation::AnimationController;
use crate::collision::{Collidable, CollisionLayer};
use crate::render::DepthSortable;
use crate::save::{Saveable, SaveData, SaveError};
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use serde::{Serialize, Deserialize};
use std::time::Instant;

/// A dropped item entity in the world
///
/// Represents a physical item that can be picked up by the player.
/// Uses the animation system for visual effects (bob, glow, spin).
pub struct DroppedItem<'a> {
    /// World position (center of item sprite)
    pub x: i32,
    pub y: i32,

    /// Item ID (references ItemRegistry)
    pub item_id: String,

    /// Quantity of items in this drop
    pub quantity: u32,

    /// Sprite size (typically 16x16 or 32x32)
    pub width: u32,
    pub height: u32,

    /// Animation controller for item visuals
    animation_controller: AnimationController<'a>,

    /// Time when item was spawned
    spawn_time: Instant,

    /// Despawn delay in seconds (5 minutes default)
    despawn_delay: f32,

    /// Pickup cooldown (prevents instant re-pickup after drop)
    pub can_pickup: bool,
    pickup_cooldown: Instant,
    pickup_cooldown_duration: f32,

    /// Collision bounds (for pickup detection)
    pub pickup_radius: u32,
}

impl<'a> DroppedItem<'a> {
    /// Creates a new dropped item at a position
    pub fn new(
        x: i32,
        y: i32,
        item_id: String,
        quantity: u32,
        animation_controller: AnimationController<'a>,
    ) -> Self {
        DroppedItem {
            x,
            y,
            item_id,
            quantity,
            width: 16,   // Item sprites are 16x16
            height: 16,
            animation_controller,
            spawn_time: Instant::now(),
            despawn_delay: 300.0,  // 5 minutes
            can_pickup: false,
            pickup_cooldown: Instant::now(),
            pickup_cooldown_duration: 0.5,  // 0.5 second cooldown
            pickup_radius: 24,  // Pickup collision radius
        }
    }

    /// Sets the animation controller for this dropped item
    pub fn set_animation_controller(&mut self, controller: AnimationController<'a>) {
        self.animation_controller = controller;
    }

    /// Updates the dropped item state
    ///
    /// Returns true if the item should be despawned (timeout reached)
    pub fn update(&mut self) -> bool {
        // Update animation (bobbing, glowing)
        self.animation_controller.update();

        // Enable pickup after cooldown
        if !self.can_pickup {
            if self.pickup_cooldown.elapsed().as_secs_f32() >= self.pickup_cooldown_duration {
                self.can_pickup = true;
            }
        }

        // Check despawn timeout
        let age = self.spawn_time.elapsed().as_secs_f32();
        if age >= self.despawn_delay {
            return true;  // Signal to despawn
        }

        false
    }

    /// Renders the dropped item
    pub fn render(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        const SPRITE_SCALE: u32 = 2;
        let scaled_width = self.width * SPRITE_SCALE;
        let scaled_height = self.height * SPRITE_SCALE;

        // Center the sprite on (x, y)
        let render_x = self.x - (scaled_width / 2) as i32;
        let render_y = self.y - (scaled_height / 2) as i32;

        let dest_rect = Rect::new(render_x, render_y, scaled_width, scaled_height);

        if let Some(sprite_sheet) = self.animation_controller.get_current_sprite_sheet() {
            sprite_sheet.render_flipped(canvas, dest_rect, false)
        } else {
            // Fallback: colored square
            canvas.set_draw_color(sdl2::pixels::Color::RGB(255, 215, 0));  // Gold
            canvas.fill_rect(dest_rect).map_err(|e| e.to_string())
        }
    }

    /// Returns the remaining time before despawn (in seconds)
    pub fn time_until_despawn(&self) -> f32 {
        let age = self.spawn_time.elapsed().as_secs_f32();
        (self.despawn_delay - age).max(0.0)
    }

    /// Returns true if the item is about to despawn (< 10 seconds)
    pub fn is_despawning_soon(&self) -> bool {
        self.time_until_despawn() < 10.0
    }

    /// Merges another dropped item into this one (combines quantities)
    ///
    /// Returns true if the merge was successful (same item type)
    pub fn try_merge(&mut self, other: &DroppedItem, max_stack_size: u32) -> bool {
        if self.item_id != other.item_id {
            return false;  // Different items can't merge
        }

        let total = self.quantity + other.quantity;
        self.quantity = total.min(max_stack_size);

        true
    }

    /// Creates a dropped item from an inventory ItemStack
    pub fn from_item_stack(
        x: i32,
        y: i32,
        stack: &crate::item::ItemStack,
        animation_controller: AnimationController<'a>,
    ) -> Self {
        DroppedItem::new(x, y, stack.item_id.clone(), stack.quantity, animation_controller)
    }
}

// ==============================================================================
// Depth Sorting Render System
// ==============================================================================

impl DepthSortable for DroppedItem<'_> {
    fn get_depth_y(&self) -> i32 {
        // Dropped items sort by their center Y position
        // This ensures they render in correct order when on the ground
        self.y
    }

    fn render(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        DroppedItem::render(self, canvas)
    }
}

// ==============================================================================
// Collision System Implementation
// ==============================================================================

impl Collidable for DroppedItem<'_> {
    fn get_bounds(&self) -> Rect {
        // Circular pickup area (represented as square for AABB collision)
        let radius = self.pickup_radius as i32;

        Rect::new(
            self.x - radius,
            self.y - radius,
            self.pickup_radius * 2,
            self.pickup_radius * 2,
        )
    }

    fn get_collision_layer(&self) -> CollisionLayer {
        CollisionLayer::Item  // New collision layer for items
    }
}

// ==============================================================================
// Save/Load Implementation
// ==============================================================================

impl Saveable for DroppedItem<'_> {
    fn to_save_data(&self) -> Result<SaveData, SaveError> {
        #[derive(Serialize)]
        struct DroppedItemData {
            x: i32,
            y: i32,
            item_id: String,
            quantity: u32,
            // Note: Don't save timers (age, cooldown) - reset on load
            // This prevents issues with system time changes
        }

        let data = DroppedItemData {
            x: self.x,
            y: self.y,
            item_id: self.item_id.clone(),
            quantity: self.quantity,
        };

        Ok(SaveData {
            data_type: "dropped_item".to_string(),
            json_data: serde_json::to_string(&data)?,
        })
    }

    fn from_save_data(data: &SaveData) -> Result<Self, SaveError> {
        #[derive(Deserialize)]
        struct DroppedItemData {
            x: i32,
            y: i32,
            item_id: String,
            quantity: u32,
        }

        if data.data_type != "dropped_item" {
            return Err(SaveError::CorruptedData(format!(
                "Expected dropped_item, got {}",
                data.data_type
            )));
        }

        let item_data: DroppedItemData = serde_json::from_str(&data.json_data)?;

        // Create with animation controller placeholder
        let mut item = DroppedItem::new(
            item_data.x,
            item_data.y,
            item_data.item_id,
            item_data.quantity,
            AnimationController::new(),
        );

        // Reset timers on load (start fresh despawn timer)
        // This is intentional - loaded items get a fresh 5 minutes
        item.can_pickup = true;  // Can pickup immediately on load

        Ok(item)
    }
}
```

### Component 2: Collision Layer Extension

**File**: `src/collision.rs` (add new layer)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionLayer {
    Player,
    Enemy,
    Environment,
    Item,        // ← New layer for dropped items
}

impl CollisionLayer {
    /// Returns true if this layer should collide with another
    pub fn collides_with(&self, other: CollisionLayer) -> bool {
        match (self, other) {
            // Player collides with items (for pickup)
            (CollisionLayer::Player, CollisionLayer::Item) => true,
            (CollisionLayer::Item, CollisionLayer::Player) => true,

            // Items don't collide with enemies or environment
            (CollisionLayer::Item, _) => false,
            (_, CollisionLayer::Item) => false,

            // ... existing collision rules
        }
    }
}
```

### Component 3: Item Spawning

**In main.rs or appropriate system:**

```rust
/// Spawns a dropped item in the world
pub fn spawn_dropped_item(
    dropped_items: &mut Vec<DroppedItem>,
    x: i32,
    y: i32,
    item_id: &str,
    quantity: u32,
    item_registry: &ItemRegistry,
    texture_creator: &TextureCreator<WindowContext>,
) -> Result<(), String> {
    // Load item sprite from registry
    let item_def = item_registry.get(item_id)
        .ok_or(format!("Unknown item: {}", item_id))?;

    // Load texture for item
    let texture = texture_creator.load_texture(&item_def.sprite_path)?;

    // Create animation controller with "idle" animation
    let mut animation_controller = AnimationController::new();
    let sprite_config = SpriteSheetConfig::new(vec![
        AnimationConfig::new("item_idle", 0, 1, 0.2, AnimationMode::Looping),
    ]);
    let sprite_sheet = SpriteSheet::new(texture, 16, 16, sprite_config);
    animation_controller.add_sprite_sheet("item_idle".to_string(), sprite_sheet);
    animation_controller.set_state("item_idle".to_string());

    // Create dropped item entity
    let dropped_item = DroppedItem::new(
        x,
        y,
        item_id.to_string(),
        quantity,
        animation_controller,
    );

    dropped_items.push(dropped_item);

    Ok(())
}
```

### Component 4: Pickup Logic

**In main.rs game loop:**

```rust
/// Handles player collision with dropped items
pub fn handle_item_pickup(
    player: &Player,
    player_inventory: &mut PlayerInventory,
    dropped_items: &mut Vec<DroppedItem>,
    item_registry: &ItemRegistry,
) {
    // Find items colliding with player
    let player_bounds = player.get_bounds();
    let mut pickup_indices = Vec::new();

    for (index, item) in dropped_items.iter().enumerate() {
        if !item.can_pickup {
            continue;  // Skip items in cooldown
        }

        if player_bounds.has_intersection(item.get_bounds()) {
            // Try to add to player inventory
            match player_inventory.quick_add(&item.item_id, item.quantity, item_registry) {
                Ok(overflow) => {
                    if overflow == 0 {
                        // All items picked up
                        pickup_indices.push(index);
                        println!("Picked up {} {}", item.quantity, item.item_id);
                    } else {
                        // Partial pickup (inventory full)
                        // Update item quantity
                        // (This requires mutable access, handle separately)
                    }
                }
                Err(e) => {
                    eprintln!("Failed to pickup item: {}", e);
                }
            }
        }
    }

    // Remove picked up items (iterate in reverse to avoid index shifting)
    for &index in pickup_indices.iter().rev() {
        dropped_items.remove(index);
    }
}
```

### Component 5: Automatic Item Merging

**Optimization for nearby drops:**

```rust
/// Merges nearby dropped items of the same type
///
/// This prevents clutter when many items drop in the same area
pub fn merge_nearby_items(
    dropped_items: &mut Vec<DroppedItem>,
    item_registry: &ItemRegistry,
) {
    const MERGE_DISTANCE: i32 = 32;  // Items within 32 pixels merge

    let mut merged_indices = Vec::new();

    for i in 0..dropped_items.len() {
        if merged_indices.contains(&i) {
            continue;
        }

        for j in (i + 1)..dropped_items.len() {
            if merged_indices.contains(&j) {
                continue;
            }

            let item_a = &dropped_items[i];
            let item_b = &dropped_items[j];

            // Check if same item and close enough
            if item_a.item_id == item_b.item_id {
                let dx = item_a.x - item_b.x;
                let dy = item_a.y - item_b.y;
                let distance = ((dx * dx + dy * dy) as f32).sqrt() as i32;

                if distance <= MERGE_DISTANCE {
                    // Get max stack size from registry
                    if let Some(item_def) = item_registry.get(&item_a.item_id) {
                        // Try to merge j into i
                        if dropped_items[i].try_merge(&item_b, item_def.max_stack_size) {
                            merged_indices.push(j);
                        }
                    }
                }
            }
        }
    }

    // Remove merged items
    for &index in merged_indices.iter().rev() {
        dropped_items.remove(index);
    }
}
```

## Integration Examples

### Example 1: Slime Drops Slime Ball on Death

**File**: `src/slime.rs` (in death logic)

```rust
// In Slime::update() when health <= 0
if self.health <= 0 && !self.has_dropped_loot {
    self.has_dropped_loot = true;  // Prevent multiple drops

    // Signal to spawn dropped item
    // (Done in main.rs to avoid circular dependencies)
}
```

**File**: `src/main.rs` (in game loop)

```rust
// Check for dead slimes and spawn drops
for slime in &mut slimes {
    if !slime.is_alive && slime.animation_controller.is_animation_finished() {
        // Spawn slime ball at slime position
        spawn_dropped_item(
            &mut dropped_items,
            slime.x,
            slime.y,
            "slime_ball",
            1,  // Drop 1 slime ball
            &item_registry,
            &texture_creator,
        ).ok();
    }
}

// Remove dead slimes after drop
slimes.retain(|s| s.is_alive || !s.animation_controller.is_animation_finished());
```

### Example 2: Chest Drops Contents

```rust
// When chest is destroyed
if let Some(chest) = get_container_at(block_x, block_y) {
    let drops = chest.drop_contents();

    for (item_id, quantity) in drops {
        // Spawn each item stack as a dropped item
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
}
```

### Example 3: Player Drops Item

```rust
// When player presses 'Q' key
if keyboard_state.is_scancode_pressed(Scancode::Q) {
    if let Some(stack) = player_inventory.get_selected_hotbar() {
        // Drop the item in front of player
        let drop_x = player.x + player.direction.offset_x() * 32;
        let drop_y = player.y + player.direction.offset_y() * 32;

        spawn_dropped_item(
            &mut dropped_items,
            drop_x,
            drop_y,
            &stack.item_id,
            1,  // Drop 1 item (or full stack with shift+Q)
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

## Rendering & Animation

### Animation States for Items

Items can have multiple animation states:

```rust
// In item sprite sheet config
SpriteSheetConfig::new(vec![
    AnimationConfig::new("item_idle", 0, 1, 0.2, AnimationMode::Looping),
    AnimationConfig::new("item_glow", 0, 4, 0.1, AnimationMode::Looping),  // Pulse effect
    AnimationConfig::new("item_despawn", 0, 8, 0.05, AnimationMode::Once), // Fade out
])
```

### Visual Effects

**Bobbing Motion** (optional):
```rust
impl DroppedItem<'_> {
    pub fn update(&mut self) {
        // ...existing update code...

        // Add vertical bobbing
        let age = self.spawn_time.elapsed().as_secs_f32();
        let bob_offset = (age * 2.0).sin() * 3.0;  // Bob 3 pixels up/down
        self.render_y_offset = bob_offset as i32;
    }
}
```

**Despawn Warning** (flash when < 10 seconds):
```rust
pub fn render(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
    // Flash when about to despawn
    if self.is_despawning_soon() {
        let flash = (self.time_until_despawn() * 2.0).sin() > 0.0;
        if !flash {
            return Ok(());  // Skip rendering (flashing effect)
        }
    }

    // ... normal rendering ...
}
```

## Performance Optimization

### Item Count Limit

Prevent performance issues from too many drops:

```rust
const MAX_DROPPED_ITEMS: usize = 1000;

pub fn spawn_dropped_item(...) {
    if dropped_items.len() >= MAX_DROPPED_ITEMS {
        // Despawn oldest item
        dropped_items.remove(0);
    }

    // ... spawn new item ...
}
```

### Spatial Partitioning

For large worlds, only check nearby items for pickup:

```rust
// Only check items within 100 pixels of player
for item in dropped_items.iter().filter(|item| {
    let dx = (item.x - player.x).abs();
    let dy = (item.y - player.y).abs();
    dx < 100 && dy < 100
}) {
    // ... pickup logic ...
}
```

### Auto-Merge Timer

Run merge logic periodically, not every frame:

```rust
// In game loop
if frame_count % 60 == 0 {  // Every second at 60 FPS
    merge_nearby_items(&mut dropped_items, &item_registry);
}
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_merge() {
        let mut item1 = DroppedItem::new(100, 100, "slime_ball".to_string(), 30, AnimationController::new());
        let item2 = DroppedItem::new(105, 105, "slime_ball".to_string(), 20, AnimationController::new());

        assert!(item1.try_merge(&item2, 64));
        assert_eq!(item1.quantity, 50);
    }

    #[test]
    fn test_despawn_timer() {
        let item = DroppedItem::new(0, 0, "test".to_string(), 1, AnimationController::new());

        assert!(item.time_until_despawn() > 299.0);  // Almost 300 seconds
        assert!(!item.is_despawning_soon());
    }

    #[test]
    fn test_pickup_cooldown() {
        let mut item = DroppedItem::new(0, 0, "test".to_string(), 1, AnimationController::new());

        assert!(!item.can_pickup);  // Can't pickup immediately

        // Simulate time passage (in real code, wait 0.5 seconds)
        item.can_pickup = true;
        assert!(item.can_pickup);
    }
}
```

### Integration Tests

```rust
#[test]
fn test_save_load_dropped_item() {
    let item = DroppedItem::new(100, 200, "slime_ball".to_string(), 5, AnimationController::new());

    let save_data = item.to_save_data().unwrap();
    let loaded = DroppedItem::from_save_data(&save_data).unwrap();

    assert_eq!(loaded.x, 100);
    assert_eq!(loaded.y, 200);
    assert_eq!(loaded.item_id, "slime_ball");
    assert_eq!(loaded.quantity, 5);
}
```

## Rust Learning Opportunities

This system teaches:

1. **Entity Pattern** (Game Dev)
   - Full entity implementation
   - Component integration (animation, collision, rendering)
   - Entity lifecycle management

2. **Trait Implementation** (Chapter 10)
   - Implementing multiple traits (Collidable, DepthSortable, Saveable)
   - Trait requirements and constraints
   - Default trait implementations

3. **Time and Duration** (std::time)
   - Instant for timers
   - Duration calculations
   - Timeout logic

4. **Vector Operations** (Chapter 8)
   - Adding/removing items
   - Filtering and finding
   - Reverse iteration for removal

5. **Error Handling** (Chapter 9)
   - Result returns for spawning
   - Graceful failure handling
   - Error propagation

## Summary

Dropped Item Entity provides:

✅ **Physical Presence** - Real entities in the game world
✅ **Collision Detection** - Player pickup via collision system
✅ **Animation Support** - Visual effects using animation system
✅ **Auto-Despawn** - Prevents world clutter (5 min timeout)
✅ **Item Merging** - Combines nearby drops automatically
✅ **Save/Load** - Full persistence support
✅ **Performance** - Optimized for many items

**Next Steps**:
1. Implement `src/dropped_item.rs`
2. Add `Item` collision layer to collision system
3. Test slime ball drops from slime deaths
4. Integrate pickup with player collision
5. Add to save/load in main.rs

---

**Related Documentation**:
- **Feature Plan**: `docs/features/item-inventory-system.md`
- **Item System**: `docs/systems/item-system-design.md`
- **Inventory System**: `docs/systems/inventory-system-design.md`
- **UI System**: `docs/systems/ui-system.md` (why items aren't UI)
- **Collision System**: `docs/systems/collision_implementation_plan.md`
- **Quick Reference**: `docs/patterns/item-inventory-quick-reference.md`

---

**Last Updated**: December 2024
**Status**: 🏗️ Planned - Ready for implementation
