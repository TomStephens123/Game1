# Resolution Scaling Plan

## Current State Analysis

### Window & World
- **Current resolution**: 1028 x 1028 pixels (1:1 square aspect ratio)
- **Target resolution**: 640 x 360 pixels (16:9 aspect ratio)
- **World boundaries**:
  - Border walls at edges (32px thick)
  - Central obstacle at (450, 400) with size 128x128

### Sprite Sizes (Before Scaling)
- **Player sprite**: 32 x 32 pixels per frame
- **Slime sprite**: 32 x 32 pixels per frame
- Both use **3x render scale** → displayed as 96 x 96 pixels

### Entity Sizes & Hitboxes

#### Player
- **Sprite size**: 32 x 32 (unscaled)
- **Rendered size**: 96 x 96 (3x scale)
- **Hitbox** (unscaled): 16 x 16
  - Offset: (8, 8) from sprite origin
  - Scaled hitbox: 48 x 48
- **Speed**: 3 pixels/frame
- **Starting position**: (300, 200)

#### Slime
- **Sprite size**: 32 x 32 (unscaled)
- **Rendered size**: 96 x 96 (3x scale)
- **Hitbox** (unscaled): 16 x 12
  - Offset: (9, 10) from sprite origin
  - Scaled hitbox: 48 x 36
- **Jump height**: 20 pixels
- **Spawned at**: mouse click position

### Static Objects (Current)
```rust
// All coordinates relative to 1028x1028 window
Top border:    (0, 0, 1028, 32)
Left border:   (0, 0, 32, 1028)
Right border:  (996, 0, 32, 1028)  // 1028 - 32
Bottom border: (0, 996, 1028, 32)  // 1028 - 32
Central obstacle: (450, 400, 128, 128)
```

## Target Resolution: 640 x 360

### Design Goals
1. **16:9 aspect ratio** - Standard for modern monitors
2. **Pixel-perfect scaling** - Maintain crisp pixel art
3. **Adaptive window size** - Scale based on monitor resolution
4. **Preserve gameplay** - Same playable area, just different aspect ratio

### Why 640 x 360?
- **2x scale** → 1280 x 720 (720p, common for laptops)
- **3x scale** → 1920 x 1080 (1080p, most common desktop)
- **4x scale** → 2560 x 1440 (1440p, high-end monitors)
- **5x scale** → 3200 x 1800 (some 4K displays)
- **6x scale** → 3840 x 2160 (4K)

### Coordinate System Architecture

We'll use a **two-layer coordinate system**:

1. **Game coordinates** (internal, fixed)
   - Fixed 640 x 360 virtual canvas
   - All game logic uses these coordinates
   - Sprites, entities, and hitboxes defined here

2. **Display coordinates** (scaled)
   - Actual window size (e.g., 1920 x 1080)
   - SDL2 handles scaling automatically via logical size
   - No code changes needed for rendering

## Implementation Plan

### Step 1: Convert World to 640 x 360

#### Border Walls (Proportional Scaling)
Current 1028x1028 → New 640x360

```rust
// Old: 32px borders on 1028x1028 (3.1% of dimension)
// New: Keep 32px borders on 640x360 (5% of width, 8.9% of height)
// OR proportional: 20px borders (3.1% of 640)

Option A (Keep 32px borders - more visible):
Top border:    (0, 0, 640, 32)
Left border:   (0, 0, 32, 360)
Right border:  (608, 0, 32, 360)  // 640 - 32
Bottom border: (0, 328, 640, 32)  // 360 - 32

Option B (Proportional 20px borders):
Top border:    (0, 0, 640, 20)
Left border:   (0, 0, 20, 360)
Right border:  (620, 0, 20, 360)  // 640 - 20
Bottom border: (0, 340, 640, 20)  // 360 - 20
```

**Recommendation**: Use Option A (32px) - easier to see boundaries, more forgiving gameplay.

#### Central Obstacle
```rust
// Old: 128x128 at (450, 400) on 1028x1028
// Relative position: (43.8%, 38.9%)
// Scale: 12.5% of window width

// New position (centered horizontally, similar vertical %):
// Width: 640 * 0.125 = 80 pixels
// Position: (280, 140) - approximately centered

Central obstacle: (280, 140, 80, 80)
```

#### Playable Area
```rust
// With 32px borders:
Playable area: (32, 32) to (608, 328)
Playable dimensions: 576 x 296 pixels
```

### Step 2: Adjust Entity Scales

Current entities are rendered at **3x scale** on a 1028x1028 window (96x96 pixels).

On 640x360:
- **2x scale** → 64x64 pixels (10% of width) - **RECOMMENDED**
- **3x scale** → 96x96 pixels (15% of width) - too large
- **1.5x scale** → 48x48 pixels (7.5% of width) - too small, loses pixel art clarity

**Recommendation**: Change sprite render scale from 3x to 2x.

#### Updated Entity Sizes

```rust
Player:
  Sprite: 32 x 32 (unscaled)
  Rendered: 64 x 64 (2x scale)
  Hitbox (unscaled): 16 x 16
  Hitbox (scaled): 32 x 32
  Speed: Keep at 3px/frame (or adjust to 2px for tighter gameplay)

Slime:
  Sprite: 32 x 32 (unscaled)
  Rendered: 64 x 64 (2x scale)
  Hitbox (unscaled): 16 x 12
  Hitbox (scaled): 32 x 24
  Jump height: Keep at 20px (or adjust to 15px)
```

### Step 3: Window Scaling Logic

Use SDL2's **logical rendering size** to handle all scaling automatically:

```rust
// Set internal game resolution (fixed)
canvas.set_logical_size(640, 360)?;

// Detect monitor size and choose window scale
let display_mode = video_subsystem.desktop_display_mode(0)?;
let monitor_width = display_mode.w;
let monitor_height = display_mode.h;

// Calculate best integer scale that fits on monitor
let scale = calculate_best_scale(monitor_width, monitor_height, 640, 360);

// Create window at scaled resolution
let window_width = 640 * scale;
let window_height = 360 * scale;

let window = video_subsystem
    .window("Game 1", window_width, window_height)
    .position_centered()
    .build()?;
```

#### Scale Selection Logic

```rust
fn calculate_best_scale(monitor_w: i32, monitor_h: i32, game_w: u32, game_h: u32) -> u32 {
    // Leave 10% margin for taskbars/decorations
    let usable_w = (monitor_w as f32 * 0.9) as i32;
    let usable_h = (monitor_h as f32 * 0.9) as i32;

    let max_scale_w = usable_w / game_w as i32;
    let max_scale_h = usable_h / game_h as i32;

    // Use smaller scale to ensure both dimensions fit
    let scale = max_scale_w.min(max_scale_h);

    // Clamp to reasonable range (2x minimum, 6x maximum)
    scale.max(2).min(6) as u32
}
```

#### Expected Window Sizes by Monitor

| Monitor Resolution | Scale | Window Size | Notes |
|-------------------|-------|-------------|-------|
| 1280 x 720 (720p) | 2x | 1280 x 720 | Full screen (90%) |
| 1366 x 768 (laptop) | 2x | 1280 x 720 | Fits comfortably |
| 1920 x 1080 (1080p) | 3x | 1920 x 1080 | Full screen (90%) |
| 2560 x 1440 (1440p) | 4x | 2560 x 1440 | Full screen (90%) |
| 3840 x 2160 (4K) | 6x | 3840 x 2160 | Full screen (90%) |

### Step 4: Update All Hardcoded Values

#### Files to Modify

1. **src/main.rs**
   - Change window creation from `1028, 1028` to use dynamic scaling
   - Add `canvas.set_logical_size(640, 360)`
   - Update `static_objects` coordinates
   - Update `player.keep_in_bounds()` call to use `640, 360`

2. **src/player.rs**
   - Change render scale from `3` to `2` (line 154)
   - Change `keep_in_bounds` scale from `3` to `2` (line 135)
   - Change `get_bounds` scale from `3` to `2` (line 251)
   - Consider adjusting speed from `3` to `2` for tighter gameplay

3. **src/slime.rs**
   - Change render scale from `3` to `2` (line 105)
   - Change `get_bounds` scale from `3` to `2` (line 166)
   - Consider adjusting jump_height from `20` to `15`

4. **src/collision.rs** (if it exists as separate file)
   - Verify no hardcoded scales

## Code Changes Summary

### Before (Current)
```rust
// main.rs
let window = video_subsystem
    .window("Game 1", 1028, 1028)  // Fixed square
    .build()?;

player.keep_in_bounds(1028, 1028);

static_objects = vec![
    StaticObject::new(0, 0, 1028, 32),  // Top
    StaticObject::new(0, 0, 32, 1028),  // Left
    // ... etc
];
```

```rust
// player.rs
let scale = 3;  // Hardcoded in multiple places
```

### After (Proposed)
```rust
// main.rs
const GAME_WIDTH: u32 = 640;
const GAME_HEIGHT: u32 = 360;
const SPRITE_SCALE: u32 = 2;

let (window_width, window_height) = calculate_window_size(&video_subsystem)?;

let window = video_subsystem
    .window("Game 1", window_width, window_height)
    .position_centered()
    .build()?;

let mut canvas = window.into_canvas().build()?;
canvas.set_logical_size(GAME_WIDTH, GAME_HEIGHT)?;

player.keep_in_bounds(GAME_WIDTH, GAME_HEIGHT);

static_objects = vec![
    StaticObject::new(0, 0, GAME_WIDTH, 32),     // Top
    StaticObject::new(0, 0, 32, GAME_HEIGHT),    // Left
    StaticObject::new(GAME_WIDTH - 32, 0, 32, GAME_HEIGHT),  // Right
    StaticObject::new(0, GAME_HEIGHT - 32, GAME_WIDTH, 32),  // Bottom
    StaticObject::new(280, 140, 80, 80),         // Central obstacle
];
```

```rust
// player.rs - replace all scale = 3 with constant
use crate::SPRITE_SCALE;  // Import from main

let scaled_width = self.width * SPRITE_SCALE;
```

## Testing Checklist

- [ ] Game runs at 640x360 internal resolution
- [ ] Window scales correctly on different monitor sizes
- [ ] Sprites are crisp (pixel-perfect scaling)
- [ ] Player collision feels same as before
- [ ] Slime collision feels same as before
- [ ] Player can't escape borders
- [ ] Central obstacle blocks movement correctly
- [ ] Player speed feels appropriate (may need tuning)
- [ ] Slime jump height looks good (may need tuning)
- [ ] Mouse click spawning works correctly (coordinates may need adjustment)
- [ ] All animations play correctly at new scale

## Migration Strategy

### Phase 1: Add Constants
1. Define `GAME_WIDTH`, `GAME_HEIGHT`, `SPRITE_SCALE` constants
2. Run game - should work identically (no behavior change)

### Phase 2: Enable Logical Sizing
1. Add `canvas.set_logical_size(640, 360)`
2. Keep window at 1028x1028 temporarily
3. Test that rendering still works (will be stretched/letterboxed)

### Phase 3: Update Static Objects
1. Change border wall coordinates to 640x360
2. Update central obstacle position and size
3. Test collision boundaries

### Phase 4: Update Sprite Scale
1. Change all `scale = 3` to `scale = 2`
2. Test rendering, collision, and gameplay feel
3. Adjust player speed and slime jump if needed

### Phase 5: Dynamic Window Sizing
1. Implement monitor detection
2. Add scale calculation function
3. Test on multiple monitor sizes (if available)
4. Add fallback to 2x scale if detection fails

### Phase 6: Polish
1. Add window resize handling (optional)
2. Add fullscreen toggle (optional)
3. Add settings to override auto-scale (optional)

## Potential Issues & Solutions

### Issue 1: Sprites Look Blurry
**Cause**: SDL2 using linear texture filtering
**Solution**: Set texture filtering to nearest-neighbor
```rust
canvas.set_scale_quality(sdl2::hint::ScaleQuality::Nearest)?;
```

### Issue 2: Window Too Large/Small
**Cause**: Monitor detection fails or unusual aspect ratio
**Solution**: Add command-line argument to override scale
```rust
// Allow: ./game1 --scale 3
let scale = parse_args().unwrap_or_else(|| auto_detect_scale());
```

### Issue 3: Mouse Coordinates Wrong
**Cause**: Mouse events give display coordinates, need game coordinates
**Solution**: SDL2 handles this automatically with logical size, but verify:
```rust
// Mouse coordinates are automatically converted to logical coordinates
// when using set_logical_size()
```

### Issue 4: Gameplay Feels Different
**Cause**: Different playable area proportions (was square, now 16:9)
**Solution**:
- Adjust player speed if needed
- Adjust slime spawn logic if needed
- May need to rebalance obstacle placement

## Future Enhancements

1. **Fullscreen support**: Allow toggling fullscreen mode
2. **Custom scaling**: Let players choose scale manually
3. **Aspect ratio options**: Support ultrawide (21:9) or other ratios
4. **Resolution settings**: Save preferred window size to config file
5. **UI scaling**: If you add UI elements, make them scale-aware

## Recommended Execution Order

1. Create constants in main.rs
2. Update static object coordinates
3. Change sprite scale from 3 to 2
4. Add logical size setting
5. Implement dynamic window sizing
6. Test on target monitor
7. Fine-tune gameplay values (speed, jump height)
