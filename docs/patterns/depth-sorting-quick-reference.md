# Depth-Sorting Quick Reference

A quick guide for adding new entities to the depth-sorted rendering system.

## Overview

**Goal**: Make entities render in correct order based on Y-position (entities farther back render first)

**How it works**:
1. Entities implement `DepthSortable` trait
2. All entities collected into `Vec<Renderable>`
3. Sort by `get_depth_y()`
4. Render in order (back to front)

See full design doc: [depth-sorting-render-system.md](../systems/depth-sorting-render-system.md)

## Adding a New Entity Type

### Step 1: Implement DepthSortable

```rust
use crate::render::DepthSortable;
use sdl2::render::{Canvas, WindowCanvas};

impl DepthSortable for YourEntity<'_> {
    fn get_depth_y(&self) -> i32 {
        // Return the base/anchor Y coordinate
        // Usually: y + (height * SPRITE_SCALE) for bottom of sprite
        self.y + (self.height * crate::SPRITE_SCALE) as i32
    }

    fn render(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        // Your existing render logic
        self.sprite_sheet.render(canvas, self.x, self.y, crate::SPRITE_SCALE)
    }
}
```

### Step 2: Add to Renderable Enum

In `src/render.rs`:

```rust
pub enum Renderable<'a> {
    Player(&'a Player<'a>),
    Slime(&'a Slime<'a>),
    StaticObject(&'a StaticObject<'a>),
    YourEntity(&'a YourEntity<'a>),  // Add this line
}

impl<'a> Renderable<'a> {
    pub fn get_depth_y(&self) -> i32 {
        match self {
            Renderable::Player(p) => p.get_depth_y(),
            Renderable::Slime(s) => s.get_depth_y(),
            Renderable::StaticObject(obj) => obj.get_depth_y(),
            Renderable::YourEntity(e) => e.get_depth_y(),  // Add this line
        }
    }

    pub fn render(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        match self {
            Renderable::Player(p) => p.render(canvas),
            Renderable::Slime(s) => s.render(canvas),
            Renderable::StaticObject(obj) => obj.render(canvas),
            Renderable::YourEntity(e) => e.render(canvas),  // Add this line
        }
    }
}
```

### Step 3: Add to Main Render Function

In `src/render.rs`:

```rust
pub fn render_with_depth_sorting(
    canvas: &mut Canvas<Window>,
    player: &Player,
    slimes: &[Slime],
    static_objects: &[StaticObject],
    your_entities: &[YourEntity],  // Add parameter
) -> Result<(), String> {
    let mut renderables: Vec<Renderable> = Vec::new();

    renderables.push(Renderable::Player(player));

    for slime in slimes {
        renderables.push(Renderable::Slime(slime));
    }

    for obj in static_objects {
        renderables.push(Renderable::StaticObject(obj));
    }

    // Add your entities
    for entity in your_entities {
        renderables.push(Renderable::YourEntity(entity));
    }

    // Sort and render
    renderables.sort_by_key(|r| r.get_depth_y());
    for renderable in renderables {
        renderable.render(canvas)?;
    }

    Ok(())
}
```

### Step 4: Update Main Game Loop

In `src/main.rs`:

```rust
// In render section
render_with_depth_sorting(
    &mut canvas,
    &player,
    &slimes,
    &static_objects,
    &your_entities,  // Add here
)?;
```

## Common Patterns

### For Entities with Feet on Ground
```rust
fn get_depth_y(&self) -> i32 {
    // Bottom of sprite
    self.y + (self.height * SPRITE_SCALE) as i32
}
```

### For Entities with Custom Anchor
```rust
fn get_depth_y(&self) -> i32 {
    // Custom position (e.g., center)
    self.y + (self.height * SPRITE_SCALE / 2) as i32
}
```

### For Tall Objects (Trees, Buildings)
```rust
fn get_depth_y(&self) -> i32 {
    // Base Y (where collision happens)
    self.y  // Don't add height - anchor at base
}

fn render(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
    // Render upward from anchor
    let render_y = self.y - (self.sprite_height * SPRITE_SCALE) as i32;
    self.sprite_sheet.render(canvas, self.x, render_y, SPRITE_SCALE)
}
```

### For Flying/Jumping Entities
```rust
fn get_depth_y(&self) -> i32 {
    // Use shadow position (where they'd land)
    self.shadow_y
}
```

## Debugging Tips

### Visualize Depth Values
```rust
// In debug mode, render Y value above entity
draw_simple_text(
    canvas,
    font,
    &format!("Y:{}", self.get_depth_y()),
    self.x,
    self.y - 20,
    Color::WHITE,
)?;
```

### Check Render Order
```rust
// Add logging in render function
for renderable in &renderables {
    println!("Rendering at Y: {}", renderable.get_depth_y());
}
```

### Wrong Layering?
- Check `get_depth_y()` returns correct value (bottom of sprite, not top)
- Ensure Y increases downward in your coordinate system
- Verify sorting happens before rendering

## Performance Notes

- Sorting is O(n log n) but very fast for <1000 entities
- Use `sort_unstable_by_key` for slight speed boost (if order of equal Y doesn't matter)
- Only sort visible entities if you have >1000 entities

## Example: Adding a Treasure Chest

```rust
// src/treasure_chest.rs
pub struct TreasureChest<'a> {
    pub x: i32,
    pub y: i32,  // Base position
    pub width: u32,
    pub height: u32,
    pub is_open: bool,
    sprite_sheet: SpriteSheet<'a>,
}

impl DepthSortable for TreasureChest<'_> {
    fn get_depth_y(&self) -> i32 {
        self.y  // Anchor at base
    }

    fn render(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        // Render chest upward from base
        let render_y = self.y - (self.height * crate::SPRITE_SCALE) as i32;
        self.sprite_sheet.render(canvas, self.x, render_y, crate::SPRITE_SCALE)
    }
}

// Then add to Renderable enum and render function as shown above
```

That's it! The chest will now layer correctly with all other entities.
