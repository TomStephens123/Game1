# Player Anchor-Based Rendering System

## Overview
This document describes the upgrade to the player's rendering and collision system to use anchor-based positioning, matching the pattern used by other entities in the game. This enables proper depth sorting and allows the player to walk in front of and behind objects naturally.

**Status**: ✅ **IMPLEMENTED** (January 2025)

## Problem Statement

### Current System
Currently, the player uses **top-left positioning**:
- Position `(x, y)` represents the top-left corner of the sprite
- Sprite renders directly from `(x, y)`
- Depth sorting calculates anchor at bottom: `y + (height * SPRITE_SCALE)`
- **Issue**: The visual position doesn't intuitively match the depth anchor

### Desired System
The player should use **anchor-based positioning** (like Slime and TheEntity):
- Position `(x, y)` represents the **anchor point** (bottom-center of sprite, where "feet" touch ground)
- Sprite renders **upward** from anchor point
- Depth sorting uses the same anchor point
- **Benefit**: Consistent with other entities, intuitive positioning, proper depth layering

## Current Implementation Analysis

### Player Structure (src/player.rs)
```rust
pub struct Player<'a> {
    pub x: i32,              // Currently: top-left X
    pub y: i32,              // Currently: top-left Y
    pub width: u32,          // Sprite width (32)
    pub height: u32,         // Sprite height (32)

    // Collision/hitbox configuration
    pub hitbox_offset_x: i32,     // Default: 8
    pub hitbox_offset_y: i32,     // Default: 8
    pub hitbox_width: u32,        // Default: 16
    pub hitbox_height: u32,       // Default: 16
}
```

### Current Collision System
The player has **one collision box** used for:
1. Environmental collision (walls, static objects)
2. Damage detection (getting hit by enemies)

```rust
impl Collidable for Player<'_> {
    fn get_bounds(&self) -> Rect {
        const SPRITE_SCALE: u32 = 2;
        let offset_x = self.hitbox_offset_x * SPRITE_SCALE as i32;
        let offset_y = self.hitbox_offset_y * SPRITE_SCALE as i32;
        Rect::new(
            self.x + offset_x,      // Top-left based
            self.y + offset_y,
            self.hitbox_width * SPRITE_SCALE,
            self.hitbox_height * SPRITE_SCALE,
        )
    }
}
```

### Current Rendering
```rust
pub fn render(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
    const SPRITE_SCALE: u32 = 2;
    let dest_rect = Rect::new(
        self.x,                        // Render from top-left
        self.y,
        self.width * SPRITE_SCALE,
        self.height * SPRITE_SCALE,
    );
    // ... render sprite
}
```

### Current Depth Sorting
```rust
impl DepthSortable for Player<'_> {
    fn get_depth_y(&self) -> i32 {
        // Depth anchor at bottom of sprite
        self.y + (self.height * SPRITE_SCALE) as i32
    }
}
```

## Proposed Solution

### 1. Anchor-Based Position System

**Change position semantics:**
- `(x, y)` will represent the **anchor point** at the bottom-center of the sprite
- This matches the conceptual "where the player stands" in the game world

**Visual Representation:**
```
Before (Top-Left):          After (Anchor at Bottom-Center):
┌─────────┐                      ┌─────────┐
│  (x,y)  │                      │         │
│         │                      │         │
│  Player │                      │  Player │
│         │                      │         │
└─────────┘                      └────•────┘
                                     (x,y) ← anchor
```

### 2. Separate Collision Systems

Create **two distinct collision boxes**:

#### A. Environmental Collision Box
Used for: Colliding with walls, static objects, terrain
- **Purpose**: Prevents walking through solid objects
- **Size**: Smaller footprint at the base (where player "stands")
- **Configuration**: Keep existing `hitbox_*` fields, rename for clarity

#### B. Damage Hitbox
Used for: Getting hit by enemy attacks
- **Purpose**: Determines if player takes damage
- **Size**: Larger, covering the player's body
- **Configuration**: New fields for damage hitbox

**Rationale for Separation:**
- **Environmental collision** should be tight (allows squeezing through gaps, feels responsive)
- **Damage hitbox** should be generous (player shouldn't feel cheated by pixel-perfect hits)
- Other games commonly use this pattern (e.g., Dark Souls, Zelda)

### 3. Implementation Plan

#### Step 1: Add Damage Hitbox Fields

```rust
pub struct Player<'a> {
    // ... existing fields ...

    // Rename existing hitbox fields for clarity
    pub collision_offset_x: i32,   // Renamed from hitbox_offset_x
    pub collision_offset_y: i32,   // Renamed from hitbox_offset_y
    pub collision_width: u32,      // Renamed from hitbox_width
    pub collision_height: u32,     // Renamed from hitbox_height

    // New: Damage hitbox (for getting hit by enemies)
    pub damage_hitbox_offset_x: i32,
    pub damage_hitbox_offset_y: i32,
    pub damage_hitbox_width: u32,
    pub damage_hitbox_height: u32,
}
```

**Default Values:**
```rust
// Environmental collision (tight, at feet)
collision_offset_x: -8,      // 8 pixels left of anchor
collision_offset_y: -16,     // 16 pixels up from anchor
collision_width: 16,         // 16px wide
collision_height: 16,        // 16px tall

// Damage hitbox (generous, covers body)
damage_hitbox_offset_x: -12, // 12 pixels left of anchor
damage_hitbox_offset_y: -28, // 28 pixels up from anchor
damage_hitbox_width: 24,     // 24px wide
damage_hitbox_height: 28,    // 28px tall
```

#### Step 2: Update Rendering to Use Anchor

```rust
pub fn render(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
    const SPRITE_SCALE: u32 = 2;
    let scaled_width = self.width * SPRITE_SCALE;
    let scaled_height = self.height * SPRITE_SCALE;

    // Calculate render position from anchor
    // Anchor is at bottom-center, so we render upward and centered
    let render_x = self.x - (scaled_width / 2) as i32;
    let render_y = self.y - scaled_height as i32;

    let dest_rect = Rect::new(render_x, render_y, scaled_width, scaled_height);

    // ... existing render logic ...
}
```

#### Step 3: Update Collision Bounds (Environmental)

```rust
impl Collidable for Player<'_> {
    fn get_bounds(&self) -> Rect {
        const SPRITE_SCALE: u32 = 2;

        // Calculate from anchor point (bottom-center)
        let offset_x = self.collision_offset_x * SPRITE_SCALE as i32;
        let offset_y = self.collision_offset_y * SPRITE_SCALE as i32;
        let scaled_width = self.collision_width * SPRITE_SCALE;
        let scaled_height = self.collision_height * SPRITE_SCALE;

        Rect::new(
            self.x + offset_x,
            self.y + offset_y,
            scaled_width,
            scaled_height,
        )
    }
}
```

#### Step 4: Add Damage Hitbox Method

```rust
impl Player<'_> {
    /// Gets the bounding box for damage detection (getting hit by enemies).
    ///
    /// This is separate from environmental collision bounds and is typically
    /// larger to ensure the player doesn't feel cheated by pixel-perfect hits.
    pub fn get_damage_bounds(&self) -> Rect {
        const SPRITE_SCALE: u32 = 2;

        let offset_x = self.damage_hitbox_offset_x * SPRITE_SCALE as i32;
        let offset_y = self.damage_hitbox_offset_y * SPRITE_SCALE as i32;
        let scaled_width = self.damage_hitbox_width * SPRITE_SCALE;
        let scaled_height = self.damage_hitbox_height * SPRITE_SCALE;

        Rect::new(
            self.x + offset_x,
            self.y + offset_y,
            scaled_width,
            scaled_height,
        )
    }
}
```

#### Step 5: Update Depth Sorting

```rust
impl DepthSortable for Player<'_> {
    fn get_depth_y(&self) -> i32 {
        // Anchor point is already at the base
        // This is where the player "touches the ground"
        self.y
    }

    fn render(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        Player::render(self, canvas)
    }
}
```

#### Step 6: Update Attack Position Calculation

The player's attack currently calculates from hitbox center. Update to use anchor:

```rust
pub fn start_attack(&mut self) -> Option<AttackEvent> {
    // ... existing checks ...

    // Attack originates from player's anchor point
    // We can add direction-based offset if needed
    Some(AttackEvent::new(
        self.stats.effective_stat(StatType::AttackDamage, &self.active_modifiers),
        (self.x, self.y),  // Use anchor point directly
        self.direction,
        32, // Attack range
    ))
}
```

#### Step 7: Migration of Existing Code

Files that reference player position need review:
- ✅ `src/player.rs` - Primary changes
- ⚠️ `src/main.rs` - Player initialization, camera centering
- ⚠️ `src/inventory/player.rs` - Might reference position
- ⚠️ Any respawn logic

**Migration Strategy:**
1. Update player struct and methods
2. Find all `player.x` and `player.y` references
3. Determine if they need adjustment for new anchor semantics
4. Test thoroughly to catch any visual misalignments

### 4. Visual Comparison

```
OLD SYSTEM (Top-Left):

  (x, y) ┌─────────┐
         │ Sprite  │
         │ 32x32   │
         │         │
         └─────────┘

  Collision Box:
  (x+8, y+8) to (x+24, y+24)
  [16x16 box inside sprite]

  Depth: y + 64


NEW SYSTEM (Anchor):

         ┌─────────┐
         │ Sprite  │  Damage Hitbox:
         │ 32x32   │  24x28 covering body
         │         │  (generous for fairness)
         └────•────┘
            (x,y)     Collision Box:
                      16x16 at feet
                      (tight for movement)

  Depth: y (anchor already at base)
```

## Benefits

### 1. Consistency
- Matches pattern used by Slime, TheEntity, and StaticObject
- Position semantics are intuitive: "where the entity stands"

### 2. Proper Depth Sorting
- Visual position matches depth anchor
- Player naturally layers in front/behind objects based on Y position

### 3. Better Collision Feel
- Tight environmental collision = responsive movement
- Generous damage hitbox = fair combat

### 4. Easier Level Design
- Designer places player at "foot position" rather than calculating top-left offset
- More intuitive to position relative to terrain

## Testing Strategy

### Unit Tests
```rust
#[test]
fn test_anchor_rendering_position() {
    let player = Player::new(100, 100, 32, 32, 5);
    // Anchor at (100, 100)
    // Should render at (100 - 32, 100 - 64) = (68, 36)
    // Verify render_x and render_y calculations
}

#[test]
fn test_collision_bounds_from_anchor() {
    let player = Player::new(100, 100, 32, 32, 5);
    let bounds = player.get_bounds();
    // Verify collision box is positioned relative to anchor
}

#[test]
fn test_damage_bounds_separate() {
    let player = Player::new(100, 100, 32, 32, 5);
    let collision = player.get_bounds();
    let damage = player.get_damage_bounds();
    // Verify they are different sizes
    assert!(damage.width() > collision.width());
    assert!(damage.height() > collision.height());
}
```

### Manual Testing
1. **Visual alignment**:
   - Place player at known coordinates
   - Verify sprite appears at correct position relative to anchor

2. **Depth sorting**:
   - Walk player behind and in front of objects
   - Verify correct layering based on Y position

3. **Environmental collision**:
   - Walk against walls, static objects
   - Verify tight collision feels responsive
   - Verify player can squeeze through appropriate gaps

4. **Damage detection**:
   - Get hit by slime attacks
   - Verify damage hitbox feels fair (not too strict)
   - Verify it covers the player's body visually

5. **Combat**:
   - Attack in all directions
   - Verify attack originates from correct position

## Rust Learning Opportunities

### 1. Semantic Field Renaming
- Renaming `hitbox_*` to `collision_*` makes code intent clearer
- Rust's compiler will catch all references during refactor

### 2. Coordinate Systems
- Understanding different coordinate spaces (world, screen, sprite-local)
- Anchor points as a coordinate transform

### 3. Game Design Patterns
- Separation of collision types (environmental vs damage)
- Anchor-based rendering for consistent depth sorting

### 4. Migration Safety
- Compiler catches all `hitbox_*` references after rename
- Type system ensures correct coordinate calculations

## Migration Checklist

- [ ] Add new damage hitbox fields to Player struct
- [ ] Rename existing hitbox fields to collision fields
- [ ] Update Player::new() with default values
- [ ] Update render() method to calculate from anchor
- [ ] Update Collidable::get_bounds() for environmental collision
- [ ] Add get_damage_bounds() method
- [ ] Update DepthSortable::get_depth_y()
- [ ] Update attack position calculation
- [ ] Update save/load to include new fields
- [ ] Search for all player position references in main.rs
- [ ] Update camera centering logic
- [ ] Update respawn positioning
- [ ] Add unit tests
- [ ] Manual testing pass
- [ ] Update this documentation with results

## Open Questions

1. **Should existing save files still load?**
   - Option A: Add migration logic (complex)
   - Option B: Breaking change, require new game (simpler)
   - **Recommendation**: Option B for simplicity during development

2. **Should we add visual debug rendering?**
   - Draw collision box in green
   - Draw damage hitbox in red
   - **Recommendation**: Yes, very helpful for tuning

3. **Direction-based attack offset?**
   - Attack forward from player position
   - Add small offset in facing direction
   - **Recommendation**: Test both, see what feels better

## Related Documentation

- [Depth Sorting Render System](../systems/depth-sorting-render-system.md) - Anchor-based rendering pattern
- [Collision System](../../src/collision.rs) - AABB collision detection
- [Player Stats System](../systems/player-stats-system.md) - Player damage mechanics

## References

- **The Legend of Zelda**: Separate hurtbox/hitbox system
- **Dark Souls**: Generous player hurtbox for fairness
- **Game Programming Patterns**: Spatial positioning chapter

## Implementation Summary

### Changes Made

**Player Struct** (src/player.rs):
- ✅ Renamed `hitbox_*` fields to `collision_*` (environmental collision)
- ✅ Added `damage_*` fields (damage hitbox for getting hit)
- ✅ Updated default values:
  - Collision box: 8×16 (narrow, at feet)
  - Damage hitbox: 16×16 (square, covers body)

**Rendering** (src/player.rs):
- ✅ Updated `render()` to calculate from anchor (bottom-center)
- ✅ Render position: `(x - width*scale/2, y - height*scale)`

**Collision** (src/player.rs):
- ✅ Updated `Collidable::get_bounds()` for environmental collision
- ✅ Added `get_damage_bounds()` method for damage detection
- ✅ Added `set_collision_box()` and `set_damage_hitbox()` methods

**Depth Sorting** (src/player.rs):
- ✅ Simplified `get_depth_y()` to return `self.y` directly

**Combat** (src/player.rs):
- ✅ Updated `start_attack()` to use anchor point directly

**Save/Load** (src/player.rs):
- ✅ Updated serialization to save both collision and damage hitbox fields
- ⚠️ **Breaking change**: Old save files incompatible

**Main Game Loop** (src/main.rs):
- ✅ Updated floating text positioning for regen
- ✅ Updated health bar rendering to convert anchor to top-left
- ✅ Verified attack effects, dropped items work with anchor positioning

**Documentation**:
- ✅ Updated depth-sorting-render-system.md with anchor pattern guide
- ✅ Added "Standard Entity Pattern" section with examples

### Final Configuration

**Player Anchor Point**: Bottom-center
- X: Center of sprite horizontally
- Y: Bottom of sprite vertically (where player "stands")

**Collision Boxes**:
```
Environmental Collision (8×16):
- Offset: (-4, -16) from anchor
- Centered, at feet
- Allows tight movement

Damage Hitbox (16×16):
- Offset: (-8, -24) from anchor
- Centered, covers upper body
- Fair combat detection
```

### Testing Checklist

Before marking as fully complete, test:
- [ ] Player renders correctly at spawn position
- [ ] Player moves smoothly, collision feels responsive
- [ ] Depth sorting works (walk behind/in front of objects)
- [ ] Getting hit by slimes works correctly
- [ ] Attack positioning works in all directions
- [ ] Dropped items appear at player's feet
- [ ] Health bar appears above player
- [ ] Floating text appears above player
- [ ] Respawn positioning works
- [ ] Save/load works (new saves only)

### Known Issues

**Save File Compatibility**:
- Old save files will **not** load correctly
- Field name changes: `hitbox_*` → `collision_*`
- New fields added: `damage_*`
- **Solution**: Start new game, or implement migration logic

**Potential Visual Adjustments**:
- Player spawn position (300, 200) may need tuning
- Damage hitbox size may need adjustment based on playtesting
- Collision box may need fine-tuning for feel

### Future Enhancements

- **Debug visualization**: Render collision and damage boxes
- **Hot-reloadable hitboxes**: Adjust in-game for tuning
- **Different hitboxes per animation**: Smaller during dodge, etc.

## Revision History

- 2025-01-11: Initial document created
- 2025-01-11: Implementation completed
- 2025-01-11: Documentation finalized
