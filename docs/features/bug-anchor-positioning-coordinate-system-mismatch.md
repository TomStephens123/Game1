# Bug Report: Anchor Positioning Coordinate System Mismatch

**Date**: 2025-10-11
**Status**: ✅ RESOLVED
**Severity**: High (gameplay-impacting)
**Systems Affected**: Slime entity, combat system, item drops, health bars

---

## Summary

After implementing anchor-based positioning for the Player entity, we attempted to migrate Slimes to use the same system. This revealed multiple coordinate system mismatches where different parts of the codebase made incompatible assumptions about what entity positions represent.

**Core Issue**: Mixing position reference points (anchor vs top-left vs collision box center) without proper coordinate transforms led to visual misalignment, incorrect collision detection, and confusing item drop locations.

---

## Symptoms Observed

### 1. **Collision Box Misalignment**
- ❌ Red debug collision box was offset from visible slime sprite
- ❌ Required manual hitbox offset adjustments to make collision box align
- ❌ Suggested underlying coordinate system confusion

### 2. **Attack Detection Issues**
- ❌ Player could hit slimes by attacking "above" them
- ❌ Player could hit TheEntity when above it, even without visual overlap
- ❌ Attack hitbox (green) didn't need to overlap collision box (red) to register hits

### 3. **Visual Element Misplacement**
- ❌ Health bars appeared offset below slimes
- ❌ Item drops spawned far below slime death location
- ❌ All visuals assumed different coordinate origins

### 4. **Inconsistent Behavior Across Entities**
- ✅ Player attacks worked correctly
- ✅ TheEntity collision worked correctly
- ❌ Slime collision/attacks/drops failed
- **Why?** TheEntity already used anchor positioning; Slime was half-migrated

---

## Root Causes

### Cause 1: Attack Origin at Player's Feet

**File**: `src/player.rs:247`

```rust
// BEFORE (WRONG):
Some(AttackEvent::new(
    damage,
    (self.x, self.y),  // ← Anchor is at FEET, not body!
    self.direction,
    32,
))
```

**Problem**:
- Player position `(x, y)` represents anchor point at **bottom-center of sprite** (feet)
- Attacks originating from feet meant the attack box extended **upward from the ground**
- When attacking North, half the attack box overlapped player's own body
- This allowed hitting enemies "above" the visual punch effect

**Why TheEntity worked**: TheEntity's collision box is large and positioned such that even a feet-origin attack would hit it when visually aligned.

**Why Slime failed**: Slime's tight collision box required precise body-level attacks.

---

### Cause 2: Saved Hitbox Values from Pre-Migration

**File**: `src/slime.rs:295-370` (save/load implementation)

**Problem**:
- Hitbox offsets were saved with each slime entity
- Old saves had top-left offsets: `(9, 10)`
- New code expected anchor offsets: `(-8, -12)`
- Loading old slimes created 17-22 pixel misalignment

**Why this matters**:
- Hitbox dimensions are **configuration constants**, not entity state
- Saving them treats them as mutable per-entity data
- Code changes to hitboxes didn't take effect on existing saves

---

### Cause 3: Missing Coordinate Transforms

Multiple systems expected **top-left coordinates** but received **anchor coordinates**:

#### Health Bar Rendering
**File**: `src/main.rs:1519`

```rust
// BEFORE (WRONG):
enemy_health_bar.render(
    &mut canvas,
    slime.x,  // ← Anchor (bottom-center), not top-left!
    slime.y,
    ...
);
```

Health bars expect top-left to draw above entities. Passing anchor coordinates rendered bars at wrong position.

#### Item Drops
**File**: `src/main.rs:1437`

```rust
// BEFORE (WRONG):
let dropped_item = DroppedItem::new(
    slime.x + 32,  // ← Adding offset to anchor doesn't make sense
    slime.y + 32,
    ...
);
```

Old code assumed `(x, y)` was top-left and added 32 to reach "center". After anchor migration, this placed items 32 pixels below feet.

---

### Cause 4: Sprite Sheet Padding (Red Herring!)

**Not actually a bug**, but caused confusion:

- Slime sprite sheet is 32×32 pixels
- Actual slime artwork is ~16×12 pixels, centered in frame
- Collision box correctly positioned on visible artwork
- But yellow debug box (full 32×32 frame) didn't match sprite

**Key Insight**: This is **normal game dev practice**. Sprite sheets have padding for animation consistency. Collision boxes should target the visible artwork, not the frame.

---

## Fixes Applied

### Fix 1: Attack Origin from Visual Center ✅

**File**: `src/player.rs:242-252`

```rust
// AFTER (CORRECT):
const SPRITE_SCALE: u32 = 2;
let player_center_y = self.y - (self.height * SPRITE_SCALE / 2) as i32;

Some(AttackEvent::new(
    damage,
    (self.x, player_center_y),  // ← Now at body/center, not feet!
    self.direction,
    32,
))
```

**Result**: Attacks now extend from player's body. Visually aligns with punch effect. Precise hit detection.

---

### Fix 2: Remove Hitbox from Save/Load ✅

**File**: `src/slime.rs:295-357`

**Changes**:
1. Removed `hitbox_offset_x`, `hitbox_offset_y`, `hitbox_width`, `hitbox_height` from `SlimeData` structs
2. Added comments: "Hitbox values are NOT saved - they are configuration constants"
3. Deleted old save file to clear stale hitbox values

**Result**: Hitbox changes now take effect immediately. No save compatibility issues when tuning collision boxes.

---

### Fix 3: Add Coordinate Transforms ✅

#### Health Bar Transform
**File**: `src/main.rs:1521-1522`

```rust
// Convert anchor to top-left for health bar rendering
let slime_top_left_x = slime.x - ((slime.width * SPRITE_SCALE) / 2) as i32;
let slime_top_left_y = slime.y - (slime.height * SPRITE_SCALE) as i32;

enemy_health_bar.render(&mut canvas, slime_top_left_x, slime_top_left_y, ...);
```

#### Item Drop at Collision Center
**File**: `src/main.rs:1436-1437`

```rust
// Drop item at center of slime's collision box (visible body center)
let drop_x = slime.x + (slime.hitbox_offset_x * SPRITE_SCALE as i32)
                     + (slime.hitbox_width * SPRITE_SCALE / 2) as i32;
let drop_y = slime.y + (slime.hitbox_offset_y * SPRITE_SCALE as i32)
                     + (slime.hitbox_height * SPRITE_SCALE / 2) as i32;
```

**Result**: All visual elements now render at correct positions.

---

### Fix 4: Intuitive Spawn Position ✅

**File**: `src/main.rs:1129-1131`

**Problem**: When spawning procedurally, forgetting that `new(x, y)` uses anchor could place entities incorrectly.

**Solution**: Calculate anchor from desired collision box center:

```rust
// Spawn slime so that click position = collision box center
let temp_slime = Slime::new(0, 0, AnimationController::new());
let anchor_x = x - (temp_slime.hitbox_offset_x * SPRITE_SCALE as i32)
                 - (temp_slime.hitbox_width * SPRITE_SCALE / 2) as i32;
let anchor_y = y - (temp_slime.hitbox_offset_y * SPRITE_SCALE as i32)
                 - (temp_slime.hitbox_height * SPRITE_SCALE / 2) as i32;

let new_slime = Slime::new(anchor_x, anchor_y, animation_controller);
```

**Result**: Clicking spawns slime with collision box centered on cursor. Intuitive for testing and procedural generation.

---

## Debug Visualization Added

**Keybind**: Press `B` to toggle collision box visualization

### Debug Colors (with B pressed):
- 🔴 **Red**: Environmental collision boxes (for push physics, walls)
- 🔵 **Blue**: Damage hitboxes (where entities can be hit)
- 🟢 **Green**: Attack hitboxes (when attacking)
- 🟨 **Yellow**: Full sprite frame boundaries (32×32 box)
- ⬜ **White dot**: Anchor point (entity's logical position)

**Implementation**: `src/main.rs:1571-1607`

This visualization was **critical for debugging** and should remain permanently for development.

---

## Lessons Learned

### 1. **Coordinate Systems Must Be Explicit**

**Problem**: Different systems assumed different position meanings without documentation.

**Solution**: Every position field should document its reference point:

```rust
/// Entity position (anchor point at bottom-center of sprite)
/// For rendering, convert to top-left: (x - width/2, y - height)
pub x: i32,
pub y: i32,
```

### 2. **Anchor ≠ Visual Center ≠ Collision Center**

Three different reference points serve different purposes:

| Reference Point | Purpose | Example Use |
|----------------|---------|-------------|
| **Anchor** | Logical position, depth sorting | Entity "stands" here |
| **Visual Center** | Player perception, gameplay feel | Attacks, effects originate here |
| **Collision Center** | Hitbox tuning, precise detection | Item drops, spawn positions |

**Takeaway**: Don't conflate these! Transform between them as needed.

### 3. **Save Configuration Separately from State**

**Configuration** (should NOT be saved):
- Hitbox offsets and sizes
- Sprite dimensions
- Attack ranges
- Animation frame counts

**State** (should be saved):
- Position (x, y)
- Health
- Inventory contents
- Current behavior/animation

**Why**: Configuration changes should take effect immediately without breaking saves.

### 4. **Test Cross-System Interactions**

This bug was discovered because we tested:
- ✅ Player attacks on Slimes
- ✅ Player attacks on TheEntity (comparison)
- ✅ Item drops
- ✅ Health bars

**If we only tested one system**, we might have blamed the wrong component.

### 5. **Debug Visualization is Not Optional**

Adding the B-key hitbox visualization was **the turning point**. Without seeing:
- Where the attack box actually was (green)
- Where the collision box was (red)
- Where the anchor was (white dot)
- Where the sprite frame was (yellow)

...we would have kept guessing at the root cause.

**Recommendation**: Add debug visualization **early** for all spatial systems.

---

## How to Avoid This in the Future

### Checklist: Adding New Entities

When adding a new entity that uses anchor-based positioning:

#### ✅ 1. Define Anchor Point Clearly

```rust
/// MyEntity position represents the anchor point at [DESCRIBE LOCATION]
/// For a character: bottom-center (where feet touch ground)
/// For a tree: bottom-center (trunk base)
/// For a flying enemy: could be center-center
pub x: i32,
pub y: i32,
```

#### ✅ 2. Implement Rendering

```rust
pub fn render(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
    const SPRITE_SCALE: u32 = 2;

    // Calculate render position from anchor
    let render_x = self.x - (self.width * SPRITE_SCALE / 2) as i32;
    let render_y = self.y - (self.height * SPRITE_SCALE) as i32;

    let dest_rect = Rect::new(render_x, render_y, ...);
    // ... render sprite
}
```

#### ✅ 3. Implement Collision Box

```rust
impl Collidable for MyEntity {
    fn get_bounds(&self) -> Rect {
        const SPRITE_SCALE: u32 = 2;

        // Offsets are RELATIVE TO ANCHOR
        let offset_x = self.collision_offset_x * SPRITE_SCALE as i32;
        let offset_y = self.collision_offset_y * SPRITE_SCALE as i32;

        Rect::new(
            self.x + offset_x,
            self.y + offset_y,
            self.collision_width * SPRITE_SCALE,
            self.collision_height * SPRITE_SCALE,
        )
    }
}
```

#### ✅ 4. Implement Depth Sorting

```rust
impl DepthSortable for MyEntity {
    fn get_depth_y(&self) -> i32 {
        // Return anchor Y directly (it's already at the depth point!)
        self.y
    }

    fn render(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        // Delegate to existing render method
        MyEntity::render(self, canvas)
    }
}
```

#### ✅ 5. Coordinate Transforms for External Systems

When passing position to systems expecting **top-left**:

```rust
// Health bars
let top_left_x = entity.x - ((entity.width * SPRITE_SCALE) / 2) as i32;
let top_left_y = entity.y - (entity.height * SPRITE_SCALE) as i32;
health_bar.render(canvas, top_left_x, top_left_y, ...);

// Floating text
let text_x = entity.x;  // Already centered
let text_y = entity.y - (entity.height * SPRITE_SCALE) as i32;  // Above sprite
```

When calculating **visual center** for attacks/effects:

```rust
let center_x = entity.x;  // Anchor is already centered horizontally
let center_y = entity.y - (entity.height * SPRITE_SCALE / 2) as i32;
```

When calculating **collision box center** for spawning/drops:

```rust
let collision_center_x = entity.x
    + (entity.collision_offset_x * SPRITE_SCALE as i32)
    + (entity.collision_width * SPRITE_SCALE / 2) as i32;
let collision_center_y = entity.y
    + (entity.collision_offset_y * SPRITE_SCALE as i32)
    + (entity.collision_height * SPRITE_SCALE / 2) as i32;
```

#### ✅ 6. Test with Debug Visualization

Press `B` and verify:
- 🔴 Red box aligns with visible sprite body
- 🟨 Yellow box shows full sprite frame (may have padding - OK!)
- ⬜ White dot is at intended anchor location
- Attacking from all directions works correctly
- Item drops appear at sensible locations

#### ✅ 7. Don't Save Configuration

```rust
impl Saveable for MyEntity {
    fn to_save_data(&self) -> Result<SaveData, SaveError> {
        struct EntityData {
            x: i32,
            y: i32,
            health: i32,
            // ❌ DON'T save: collision_offset_x, collision_width, etc.
        }
        // ...
    }
}
```

---

## Reference: Coordinate Transform Formulas

### Given: Entity with Anchor Positioning

```rust
// Entity anchor point (bottom-center for ground entities)
entity.x: i32
entity.y: i32

// Sprite dimensions (unscaled)
entity.width: u32
entity.height: u32

// Collision box config (unscaled, relative to anchor)
entity.collision_offset_x: i32
entity.collision_offset_y: i32
entity.collision_width: u32
entity.collision_height: u32

const SPRITE_SCALE: u32 = 2;
```

### Transform: Anchor → Top-Left (for rendering)

```rust
let top_left_x = entity.x - (entity.width * SPRITE_SCALE / 2) as i32;
let top_left_y = entity.y - (entity.height * SPRITE_SCALE) as i32;
```

### Transform: Anchor → Visual Center

```rust
let center_x = entity.x;  // Already centered horizontally
let center_y = entity.y - (entity.height * SPRITE_SCALE / 2) as i32;
```

### Transform: Anchor → Collision Box Bounds

```rust
let collision_x = entity.x + (entity.collision_offset_x * SPRITE_SCALE as i32);
let collision_y = entity.y + (entity.collision_offset_y * SPRITE_SCALE as i32);
let collision_width = entity.collision_width * SPRITE_SCALE;
let collision_height = entity.collision_height * SPRITE_SCALE;

let collision_rect = Rect::new(collision_x, collision_y, collision_width, collision_height);
```

### Transform: Anchor → Collision Box Center

```rust
let collision_center_x = entity.x
    + (entity.collision_offset_x * SPRITE_SCALE as i32)
    + (entity.collision_width * SPRITE_SCALE / 2) as i32;
let collision_center_y = entity.y
    + (entity.collision_offset_y * SPRITE_SCALE as i32)
    + (entity.collision_height * SPRITE_SCALE / 2) as i32;
```

### Reverse Transform: Collision Center → Anchor (for spawning)

```rust
// When you want to spawn an entity with collision box at specific position:
let anchor_x = desired_collision_center_x
    - (entity.collision_offset_x * SPRITE_SCALE as i32)
    - (entity.collision_width * SPRITE_SCALE / 2) as i32;
let anchor_y = desired_collision_center_y
    - (entity.collision_offset_y * SPRITE_SCALE as i32)
    - (entity.collision_height * SPRITE_SCALE / 2) as i32;

let entity = MyEntity::new(anchor_x, anchor_y, ...);
```

---

## Testing This Fix

### Manual Test Plan

1. **Start game** (fresh save, no old coordinate data)
2. **Press B** to show debug visualization
3. **Right-click** to spawn slime
   - ✅ Slime appears with red box centered on cursor
   - ✅ White anchor dot at slime's bottom-center
   - ✅ Yellow frame shows full 32×32 sprite bounds (larger than slime artwork - OK!)

4. **Walk up to slime and attack (M key)**
   - ✅ Green attack box appears at player's body level (not feet)
   - ✅ Attack only connects when green overlaps red
   - ✅ Punch effect aligns with attack hitbox

5. **Attack from all 8 directions**
   - ✅ North: attack extends upward from body
   - ✅ South: attack extends downward
   - ✅ East/West: attack extends horizontally at body level
   - ✅ Diagonals: attack extends at 45° from body

6. **Kill slime**
   - ✅ Slime_ball drops inside red collision box
   - ✅ Item is pickupable (not stuck in ground/wall)

7. **Health bar positioning**
   - ✅ Green bar appears above slime sprite
   - ✅ Bar stays above slime when jumping
   - ✅ Player health bar also correct (comparison test)

8. **Test TheEntity (comparison)**
   - ✅ Still works correctly (already used anchors)
   - ✅ Attacks only hit when overlapping

### Regression Testing

- ✅ Player movement still smooth
- ✅ Collision with walls works
- ✅ Depth sorting correct (can walk in front/behind static objects)
- ✅ Save/load works without crashing
- ✅ Existing inventory/hotbar functionality unaffected

---

## Related Documentation

- [Entity Pattern Guide](../patterns/entity-pattern.md) - How to add new entities correctly
- [Depth Sorting System](../systems/depth-sorting-render-system.md) - Painter's algorithm implementation
- [Player Anchor Rendering](./player-anchor-rendering.md) - Original anchor migration for Player
- [Save System Design](../save-system-design.md) - What to save vs configure

---

## Conclusion

This bug revealed a fundamental truth about game development: **coordinate systems are implicit contracts**. When different parts of a codebase make incompatible assumptions about what positions mean, subtle bugs cascade through every system.

The fix required:
1. Making coordinate reference points **explicit** (anchor vs top-left vs center)
2. Adding **transforms** where systems meet
3. **Visualizing** the coordinate systems to verify correctness
4. Separating **configuration** from **state** in saves

**For future development**: Treat coordinate systems like API contracts. Document them clearly, transform between them explicitly, and visualize them during testing.

---

**Resolution Date**: 2025-10-11
**Total Time**: ~2 hours of debugging and fixes
**Lines Changed**: ~50 across 3 files
**Impact**: Complete fix with no known regressions
