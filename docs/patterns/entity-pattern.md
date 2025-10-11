# Entity Pattern Guide

**A practical guide to adding new entities to Game1**

This document provides step-by-step instructions for adding new entities (characters, objects, items) to the game, ensuring they work correctly with rendering, collision, depth sorting, and save/load systems.

## Table of Contents

1. [Quick Reference](#quick-reference)
2. [The Standard Pattern](#the-standard-pattern)
3. [Step-by-Step: Adding a New Entity](#step-by-step-adding-a-new-entity)
4. [Collision Systems](#collision-systems)
5. [Depth Sorting](#depth-sorting)
6. [Save/Load Integration](#saveload-integration)
7. [Common Pitfalls](#common-pitfalls)
8. [Examples](#examples)

---

## Quick Reference

### Entity Checklist

When adding a new entity, implement these in order:

- [ ] Define struct with anchor-based positioning
- [ ] Implement `new()` constructor
- [ ] Implement `update()` for behavior
- [ ] Implement `render()` using anchor positioning
- [ ] Implement `DepthSortable` trait
- [ ] Implement `Collidable` trait (if needed)
- [ ] Implement `Saveable` trait (if has persistent state)
- [ ] Add to `Renderable` enum in `src/render.rs`
- [ ] Update `render_with_depth_sorting()` in `src/render.rs`
- [ ] Test in-game

---

## The Standard Pattern

**All entities in Game1 follow anchor-based positioning:**

### Position Semantics

```rust
pub struct MyEntity<'a> {
    pub x: i32,  // Anchor X position (center horizontally)
    pub y: i32,  // Anchor Y position (bottom of sprite, where it "stands")
    pub width: u32,
    pub height: u32,
    // ... other fields
}
```

**Key Concept**: `(x, y)` represents where the entity "touches the ground" in the game world.

### Visual Representation

```
        ┌─────────┐
        │  Sprite │
        │  32x32  │
        │         │
        └────•────┘
           (x,y) ← anchor point (bottom-center)
```

### Why This Pattern?

1. **Intuitive**: Position = "where it stands"
2. **Automatic depth sorting**: Just return `self.y`
3. **Consistent**: All entities work the same way
4. **Clean math**: No offset calculations needed

---

## Step-by-Step: Adding a New Entity

Let's create a **Treasure Chest** entity from scratch.

### Step 1: Create the Struct

**File**: `src/treasure_chest.rs`

```rust
use crate::animation::AnimationController;
use crate::collision::StaticCollidable;
use crate::render::DepthSortable;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

const SPRITE_SCALE: u32 = 2;

pub struct TreasureChest<'a> {
    // Position (anchor at bottom-center)
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,

    // Rendering
    animation_controller: AnimationController<'a>,

    // State
    pub is_open: bool,
}
```

**Rust Learning**: The `'a` lifetime ensures the entity doesn't outlive the texture it references.

### Step 2: Implement Constructor

```rust
impl<'a> TreasureChest<'a> {
    pub fn new(x: i32, y: i32, animation_controller: AnimationController<'a>) -> Self {
        TreasureChest {
            x,
            y,
            width: 32,
            height: 32,
            animation_controller,
            is_open: false,
        }
    }
}
```

### Step 3: Implement Update Logic

```rust
impl<'a> TreasureChest<'a> {
    pub fn update(&mut self) {
        self.animation_controller.update();
    }

    pub fn open(&mut self) {
        if !self.is_open {
            self.is_open = true;
            self.animation_controller.set_state("chest_open".to_string());
        }
    }
}
```

### Step 4: Implement Rendering (Anchor-Based)

```rust
impl<'a> TreasureChest<'a> {
    pub fn render(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        let scaled_width = self.width * SPRITE_SCALE;
        let scaled_height = self.height * SPRITE_SCALE;

        // Calculate render position FROM ANCHOR (bottom-center)
        // Render upward and centered from anchor point
        let render_x = self.x - (scaled_width / 2) as i32;
        let render_y = self.y - scaled_height as i32;

        let dest_rect = Rect::new(render_x, render_y, scaled_width, scaled_height);

        if let Some(sprite_sheet) = self.animation_controller.get_current_sprite_sheet() {
            sprite_sheet.render_flipped(canvas, dest_rect, false)
        } else {
            // Fallback if no sprite
            canvas.set_draw_color(sdl2::pixels::Color::RGB(139, 69, 19));
            canvas.fill_rect(dest_rect).map_err(|e| e.to_string())
        }
    }
}
```

**Critical**: Always calculate render position from anchor:
- `render_x = x - (width * scale / 2)` (center horizontally)
- `render_y = y - (height * scale)` (render upward)

### Step 5: Implement DepthSortable

```rust
impl DepthSortable for TreasureChest<'_> {
    fn get_depth_y(&self) -> i32 {
        // Anchor is already at the correct depth!
        self.y
    }

    fn render(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        // Delegate to existing render method
        TreasureChest::render(self, canvas)
    }
}
```

**Why so simple?** Because position is already the anchor, depth sorting is trivial!

### Step 6: Implement Collision (if needed)

For a static object like a chest:

```rust
impl StaticCollidable for TreasureChest<'_> {
    fn get_bounds(&self) -> Rect {
        // Collision box at the base (where it sits on ground)
        // Using smaller collision for better gameplay feel
        let collision_width = 24 * SPRITE_SCALE;   // Slightly smaller
        let collision_height = 16 * SPRITE_SCALE;  // Just the base

        let offset_x = -(collision_width / 2) as i32;  // Center
        let offset_y = -collision_height as i32;       // At base

        Rect::new(
            self.x + offset_x,
            self.y + offset_y,
            collision_width,
            collision_height,
        )
    }
}
```

**Design Tip**: Collision can be smaller than sprite for better feel!

### Step 7: Add to Module System

**File**: `src/main.rs` (at top)

```rust
mod treasure_chest;
use treasure_chest::TreasureChest;
```

### Step 8: Add to Renderable Enum

**File**: `src/render.rs`

```rust
pub enum Renderable<'a> {
    Player(&'a Player<'a>),
    Slime(&'a Slime<'a>),
    StaticObject(&'a StaticObject),
    TheEntity(&'a TheEntity<'a>),
    TreasureChest(&'a TreasureChest<'a>),  // Add this
}

impl<'a> Renderable<'a> {
    fn get_depth_y(&self) -> i32 {
        match self {
            Renderable::Player(p) => p.get_depth_y(),
            Renderable::Slime(s) => s.get_depth_y(),
            Renderable::StaticObject(obj) => obj.get_depth_y(),
            Renderable::TheEntity(e) => e.get_depth_y(),
            Renderable::TreasureChest(c) => c.get_depth_y(),  // Add this
        }
    }

    fn render(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        match self {
            Renderable::Player(p) => p.render(canvas),
            Renderable::Slime(s) => s.render(canvas),
            Renderable::StaticObject(obj) => obj.render(canvas),
            Renderable::TheEntity(e) => e.render(canvas),
            Renderable::TreasureChest(c) => c.render(canvas),  // Add this
        }
    }
}
```

### Step 9: Update Depth Sorting Function

**File**: `src/render.rs`

```rust
pub fn render_with_depth_sorting(
    canvas: &mut Canvas<Window>,
    player: &Player,
    slimes: &[Slime],
    static_objects: &[StaticObject],
    entities: &[TheEntity],
    treasure_chests: &[TreasureChest],  // Add parameter
) -> Result<(), String> {
    let mut renderables: Vec<(i32, Renderable)> = Vec::new();

    // Add player
    renderables.push((player.get_depth_y(), Renderable::Player(player)));

    // Add slimes
    for slime in slimes {
        renderables.push((slime.get_depth_y(), Renderable::Slime(slime)));
    }

    // Add static objects
    for obj in static_objects {
        renderables.push((obj.get_depth_y(), Renderable::StaticObject(obj)));
    }

    // Add entities
    for entity in entities {
        renderables.push((entity.get_depth_y(), Renderable::TheEntity(entity)));
    }

    // Add treasure chests
    for chest in treasure_chests {
        renderables.push((chest.get_depth_y(), Renderable::TreasureChest(chest)));
    }

    // Sort by Y (painter's algorithm)
    renderables.sort_by_key(|(y, _)| *y);

    // Render in sorted order
    for (_, renderable) in renderables {
        renderable.render(canvas)?;
    }

    Ok(())
}
```

### Step 10: Use in Main Loop

**File**: `src/main.rs`

```rust
// In main function, create some chests
let mut chests = vec![
    TreasureChest::new(400, 300, chest_animation_controller.clone()),
    TreasureChest::new(500, 400, chest_animation_controller.clone()),
];

// In game loop update
for chest in &mut chests {
    chest.update();
}

// In render section
render_with_depth_sorting(
    &mut canvas,
    &player,
    &slimes,
    &static_objects,
    &entities,
    &chests,  // Add chests
)?;
```

---

## Collision Systems

Game1 uses **two separate collision types**:

### 1. Environmental Collision

**Purpose**: Prevent walking through solid objects (walls, trees, chests)

**Implementation**: `Collidable` trait (for dynamic entities) or `StaticCollidable` (for static objects)

```rust
impl Collidable for MyEntity<'_> {
    fn get_bounds(&self) -> Rect {
        // Calculate from anchor point
        let offset_x = -8 * SPRITE_SCALE as i32;  // Relative to anchor
        let offset_y = -16 * SPRITE_SCALE as i32;
        let width = 16 * SPRITE_SCALE;
        let height = 16 * SPRITE_SCALE;

        Rect::new(
            self.x + offset_x,
            self.y + offset_y,
            width,
            height,
        )
    }

    fn get_collision_layer(&self) -> CollisionLayer {
        CollisionLayer::Enemy  // Or Player, Static, etc.
    }
}
```

**Design Tips**:
- Make collision **smaller** than sprite for responsive feel
- Center horizontally on anchor
- Keep at "feet" level (near anchor Y)

### 2. Damage Hitbox

**Purpose**: Determine if entity gets hit by attacks

**Implementation**: Custom method (see `Player::get_damage_bounds()`)

```rust
impl MyEntity<'_> {
    pub fn get_damage_bounds(&self) -> Rect {
        // Usually larger than collision box
        // Covers the visible body for fair combat
        let offset_x = -12 * SPRITE_SCALE as i32;
        let offset_y = -24 * SPRITE_SCALE as i32;
        let width = 24 * SPRITE_SCALE;
        let height = 24 * SPRITE_SCALE;

        Rect::new(
            self.x + offset_x,
            self.y + offset_y,
            width,
            height,
        )
    }
}
```

**Design Tips**:
- Make damage hitbox **larger** than collision
- Ensure it covers visual body area
- Players should feel combat is "fair"

### Collision Box Sizing Guide

```
Typical 32x32 sprite (2x scale = 64x64 pixels):

Environmental Collision:
- Width: 8-16 pixels (tight for movement)
- Height: 16-24 pixels (tall enough)
- Position: At feet/base

Damage Hitbox:
- Width: 16-24 pixels (generous)
- Height: 16-28 pixels (covers body)
- Position: Centered on character
```

---

## Depth Sorting

**How It Works**: Entities with smaller Y values render first (appear farther back).

### Implementation

1. **Implement DepthSortable trait**:
```rust
impl DepthSortable for MyEntity<'_> {
    fn get_depth_y(&self) -> i32 {
        self.y  // That's it! Anchor = depth
    }

    fn render(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        MyEntity::render(self, canvas)
    }
}
```

2. **Add to Renderable enum** (see Step 8 above)

3. **Add to render function** (see Step 9 above)

### Depth Sorting in Action

```
Y=100:  🏠 Building  (renders first)
Y=150:     🧍 Player  (renders second)
Y=200:        🌳 Tree  (renders last, appears in front)

Result: Player appears behind tree, in front of building ✓
```

### Special Cases

**Jumping/Flying Entities**: Use `base_y` instead of current `y` for consistent depth:

```rust
impl DepthSortable for JumpingEntity<'_> {
    fn get_depth_y(&self) -> i32 {
        // Use base position, not current (which changes during jump)
        self.base_y
    }
}
```

---

## Save/Load Integration

If your entity has **persistent state** (position, health, inventory, etc.), implement `Saveable`.

### Implementation

```rust
use crate::save::{Saveable, SaveData, SaveError};
use serde::{Serialize, Deserialize};

impl Saveable for TreasureChest<'_> {
    fn to_save_data(&self) -> Result<SaveData, SaveError> {
        #[derive(Serialize)]
        struct ChestData {
            x: i32,
            y: i32,
            is_open: bool,
        }

        let data = ChestData {
            x: self.x,
            y: self.y,
            is_open: self.is_open,
        };

        Ok(SaveData {
            data_type: "treasure_chest".to_string(),
            json_data: serde_json::to_string(&data)?,
        })
    }

    fn from_save_data(data: &SaveData) -> Result<Self, SaveError> {
        #[derive(Deserialize)]
        struct ChestData {
            x: i32,
            y: i32,
            is_open: bool,
        }

        if data.data_type != "treasure_chest" {
            return Err(SaveError::CorruptedData(
                format!("Expected treasure_chest, got {}", data.data_type)
            ));
        }

        let chest_data: ChestData = serde_json::from_str(&data.json_data)?;

        let mut chest = TreasureChest::new(
            chest_data.x,
            chest_data.y,
            AnimationController::new(),  // Recreate externally
        );

        chest.is_open = chest_data.is_open;

        Ok(chest)
    }
}
```

**Important**: Animation controllers can't be serialized - recreate them when loading.

### Save/Load in Main Loop

```rust
// Saving
let mut entity_saves = Vec::new();
for chest in &chests {
    entity_saves.push(chest.to_save_data()?);
}
save_manager.save_game(&save_file)?;

// Loading
let loaded_chests = save_file.entities
    .iter()
    .filter(|e| e.data_type == "treasure_chest")
    .map(|e| TreasureChest::from_save_data(e))
    .collect::<Result<Vec<_>, _>>()?;
```

---

## Common Pitfalls

### ❌ Pitfall #1: Forgetting to Calculate Render Position

```rust
// WRONG - renders from top-left
let dest_rect = Rect::new(self.x, self.y, width, height);
```

```rust
// CORRECT - calculates from anchor
let render_x = self.x - (width / 2) as i32;
let render_y = self.y - height as i32;
let dest_rect = Rect::new(render_x, render_y, width, height);
```

### ❌ Pitfall #2: Using Top-Left for Depth

```rust
// WRONG - inconsistent with position semantics
fn get_depth_y(&self) -> i32 {
    self.y + self.height  // Don't do this!
}
```

```rust
// CORRECT - anchor is already at depth
fn get_depth_y(&self) -> i32 {
    self.y  // Simple!
}
```

### ❌ Pitfall #3: Collision Box from Top-Left

```rust
// WRONG - assumes top-left positioning
fn get_bounds(&self) -> Rect {
    Rect::new(self.x + 8, self.y + 8, 16, 16)  // Doesn't work with anchor!
}
```

```rust
// CORRECT - calculates from anchor
fn get_bounds(&self) -> Rect {
    let offset_x = -8 * SPRITE_SCALE as i32;  // Negative = left of anchor
    let offset_y = -16 * SPRITE_SCALE as i32; // Negative = above anchor
    Rect::new(
        self.x + offset_x,
        self.y + offset_y,
        16 * SPRITE_SCALE,
        16 * SPRITE_SCALE,
    )
}
```

### ❌ Pitfall #4: Forgetting to Add to Renderable Enum

If you implement DepthSortable but forget to add to `Renderable` enum, your entity won't participate in depth sorting and will render in wrong order!

### ❌ Pitfall #5: Not Scaling Collision Offsets

```rust
// WRONG - offsets not scaled
let offset_x = -4;  // Will be tiny!

// CORRECT - multiply by scale
let offset_x = -4 * SPRITE_SCALE as i32;
```

---

## Examples

### Example 1: Simple Static Object (Tree)

```rust
pub struct Tree<'a> {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    sprite_sheet: SpriteSheet<'a>,
}

impl Tree<'_> {
    pub fn render(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        let render_x = self.x - (self.width * SPRITE_SCALE / 2) as i32;
        let render_y = self.y - (self.height * SPRITE_SCALE) as i32;
        // ... render sprite
    }
}

impl DepthSortable for Tree<'_> {
    fn get_depth_y(&self) -> i32 { self.y }
    fn render(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        Tree::render(self, canvas)
    }
}

impl StaticCollidable for Tree<'_> {
    fn get_bounds(&self) -> Rect {
        // Small collision at trunk
        Rect::new(
            self.x - 8,
            self.y - 16,
            16,
            16,
        )
    }
}
```

### Example 2: Animated Enemy

```rust
pub struct Goblin<'a> {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    animation_controller: AnimationController<'a>,
    pub health: i32,
    velocity_x: i32,
    velocity_y: i32,
}

impl Goblin<'_> {
    pub fn update(&mut self) {
        // AI behavior
        self.x += self.velocity_x;
        self.y += self.velocity_y;
        self.animation_controller.update();
    }

    pub fn render(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        let render_x = self.x - (self.width * SPRITE_SCALE / 2) as i32;
        let render_y = self.y - (self.height * SPRITE_SCALE) as i32;
        let dest_rect = Rect::new(render_x, render_y,
            self.width * SPRITE_SCALE, self.height * SPRITE_SCALE);

        if let Some(sprite) = self.animation_controller.get_current_sprite_sheet() {
            sprite.render_flipped(canvas, dest_rect, false)
        } else {
            Ok(())
        }
    }
}

impl DepthSortable for Goblin<'_> {
    fn get_depth_y(&self) -> i32 { self.y }
    fn render(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        Goblin::render(self, canvas)
    }
}

impl Collidable for Goblin<'_> {
    fn get_bounds(&self) -> Rect {
        Rect::new(
            self.x - 8 * SPRITE_SCALE as i32,
            self.y - 16 * SPRITE_SCALE as i32,
            16 * SPRITE_SCALE,
            16 * SPRITE_SCALE,
        )
    }

    fn get_collision_layer(&self) -> CollisionLayer {
        CollisionLayer::Enemy
    }
}
```

### Example 3: Interactive Object with State

```rust
pub struct Door<'a> {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    sprite_sheet: SpriteSheet<'a>,
    pub is_open: bool,
}

impl Door<'_> {
    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
    }

    pub fn render(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        let render_x = self.x - (self.width * SPRITE_SCALE / 2) as i32;
        let render_y = self.y - (self.height * SPRITE_SCALE) as i32;

        // Use different sprite frame based on state
        let frame = if self.is_open { 1 } else { 0 };
        // ... render with frame
        Ok(())
    }
}

impl DepthSortable for Door<'_> {
    fn get_depth_y(&self) -> i32 { self.y }
    fn render(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        Door::render(self, canvas)
    }
}

impl StaticCollidable for Door<'_> {
    fn get_bounds(&self) -> Rect {
        if self.is_open {
            // No collision when open
            Rect::new(0, 0, 0, 0)
        } else {
            // Full collision when closed
            Rect::new(
                self.x - 16,
                self.y - 32,
                32,
                32,
            )
        }
    }
}

impl Saveable for Door<'_> {
    fn to_save_data(&self) -> Result<SaveData, SaveError> {
        #[derive(Serialize)]
        struct DoorData { x: i32, y: i32, is_open: bool }

        let data = DoorData {
            x: self.x,
            y: self.y,
            is_open: self.is_open
        };

        Ok(SaveData {
            data_type: "door".to_string(),
            json_data: serde_json::to_string(&data)?,
        })
    }

    fn from_save_data(data: &SaveData) -> Result<Self, SaveError> {
        #[derive(Deserialize)]
        struct DoorData { x: i32, y: i32, is_open: bool }

        let door_data: DoorData = serde_json::from_str(&data.json_data)?;

        let mut door = Door::new(door_data.x, door_data.y, /* sprite */);
        door.is_open = door_data.is_open;
        Ok(door)
    }
}
```

---

## Quick Reference: Coordinate Math

### From Anchor to Render Position

```rust
// For a 32x32 sprite at 2x scale (64x64 rendered)
let scaled_width = 32 * 2;   // 64
let scaled_height = 32 * 2;  // 64

// Anchor at (100, 200)
let anchor_x = 100;
let anchor_y = 200;

// Calculate top-left corner for rendering
let render_x = anchor_x - (scaled_width / 2);  // 100 - 32 = 68
let render_y = anchor_y - scaled_height;        // 200 - 64 = 136

// Sprite renders from (68, 136) to (132, 200)
// Bottom-center is at (100, 200) ✓
```

### From Anchor to Collision Box

```rust
// Small collision box: 16x16 at feet
let collision_width = 16 * 2;   // 32 (scaled)
let collision_height = 16 * 2;  // 32 (scaled)

// Offsets relative to anchor (bottom-center)
let offset_x = -(collision_width / 2);  // -16 (center it)
let offset_y = -collision_height;       // -32 (at feet)

// Collision box position
let collision_x = anchor_x + offset_x;  // 100 - 16 = 84
let collision_y = anchor_y + offset_y;  // 200 - 32 = 168

// Box spans from (84, 168) to (116, 200) ✓
```

---

## Summary

**The Golden Rules**:

1. **Position = Anchor**: `(x, y)` is bottom-center, where entity "stands"
2. **Render from Anchor**: Calculate top-left as `(x - width/2, y - height)`
3. **Depth = Anchor Y**: Just return `self.y` for depth sorting
4. **Collision from Anchor**: Use offsets relative to anchor point
5. **Always Scale**: Multiply offsets by `SPRITE_SCALE`

**Follow these rules and your entities will:**
- ✅ Render in the correct position
- ✅ Sort properly with depth (walk behind/in front of objects)
- ✅ Collide accurately
- ✅ Work consistently with all other entities

---

## Related Documentation

- [Depth Sorting Render System](../systems/depth-sorting-render-system.md) - Deep dive into rendering
- [Player Anchor Rendering](../features/player-anchor-rendering.md) - Player migration example
- [Collision System](../../src/collision.rs) - Collision detection details
- [Save System Design](../systems/save-system-design.md) - Save/load architecture

## Questions?

If something doesn't work:
1. Check render position calculation (most common issue)
2. Verify you added to `Renderable` enum
3. Confirm collision offsets are scaled
4. Make sure depth sorting function includes your entity

Happy entity building! 🎮
