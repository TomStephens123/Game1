# Camera and Viewport System

**Status**: ✅ IMPLEMENTED (Camera following complete, UI alignment fixed 2025-12-27)

## Overview

This document outlines the implementation plan for Game1's camera and viewport system, enabling:
1. **Resizable Window** - Full-screen support and arbitrary window sizes
2. **Camera Following Player** - Smooth camera movement tracking the player
3. **World-Space Rendering** - Show more world content on larger screens (not pixel scaling)
4. **UI Layer Separation** - Screen-space UI remains fixed while world moves

This transforms the game from a fixed 640x360 "screen" to a **virtual camera** viewing a larger world.

---

## Core Concept: Current vs Target System

### Current System (Fixed Screen)
```
┌─────────────────────────────┐
│   640x360 Fixed Screen      │
│                             │
│  Everything rendered at     │
│  absolute pixel positions   │
│                             │
│  Player at (320, 180)       │
└─────────────────────────────┘

Problem: Window can only scale up by integer multiples (2x, 3x)
         and just makes pixels bigger - doesn't show more world
```

### Target System (Camera + Viewport)
```
┌───────────────────────────────────────┐
│         Larger Game World             │
│                                       │
│     ┌─────────────────┐              │
│     │   Camera View   │◄──── Follows player
│     │                 │              │
│     │   Player @      │              │
│     │   world coords  │              │
│     └─────────────────┘              │
│                                       │
└───────────────────────────────────────┘
         ▼
┌─────────────────────────────┐
│   Any Size Window           │
│   (fullscreen, 1920x1080,   │
│    custom)                  │
│                             │
│   Shows camera viewport     │
│   World rendered relative   │
│   UI rendered at screen pos │
└─────────────────────────────┘

Benefit: Larger windows show MORE of the world, not bigger pixels
```

---

## Key Design Decisions

### 1. **Coordinate Systems**

We'll have **three** coordinate systems:

#### World Coordinates
- **What**: Absolute positions in the game world
- **Example**: Player at `(1234, 567)`, Enemy at `(2000, 300)`
- **Unbounded**: Can be any value
- **Used for**: Entity positions, collision detection, game logic

#### Camera Coordinates
- **What**: Position of the camera in world space
- **Example**: Camera at `(1000, 400)` means camera's center is viewing world position (1000, 400)
- **Used for**: What part of the world to render

#### Screen Coordinates
- **What**: Pixel positions on the actual window
- **Example**: UI element at `(10, 10)` is always 10 pixels from top-left corner
- **Bounded**: 0 to window width/height
- **Used for**: UI rendering, mouse input

### 2. **Rendering Pipeline**

```
┌──────────────────────────────────────────────────┐
│ 1. UPDATE PHASE (World Coordinates)              │
│    - Player moves to (1250, 580)                 │
│    - Enemy AI calculates path                    │
│    - Physics/collision in world space            │
└──────────────────────────────────────────────────┘
                      ▼
┌──────────────────────────────────────────────────┐
│ 2. CAMERA UPDATE (Camera Coordinates)            │
│    - Calculate camera target (follow player)     │
│    - Smooth camera movement (lerp)               │
│    - Apply camera bounds (keep in world)         │
│    - Camera center at (1250, 580)                │
└──────────────────────────────────────────────────┘
                      ▼
┌──────────────────────────────────────────────────┐
│ 3. WORLD RENDERING (Transform to Screen)         │
│    - For each entity:                            │
│      screen_x = entity.x - camera.x + viewport_w/2
│      screen_y = entity.y - camera.y + viewport_h/2
│    - Only render entities in camera bounds       │
│    - Use depth sorting as before                 │
└──────────────────────────────────────────────────┘
                      ▼
┌──────────────────────────────────────────────────┐
│ 4. UI RENDERING (Direct Screen Coordinates)      │
│    - Health bars at fixed screen positions       │
│    - Menus at screen center                      │
│    - No camera transformation applied            │
└──────────────────────────────────────────────────┘
```

### 3. **Reference Resolution vs Viewport**

**Reference Resolution**: `640x360`
- Minimum viewport size (for UI scaling calculations)
- UI designed for this size
- Viewport can be **larger** but not smaller

**Viewport Size**: Dynamic
- Actual visible world area
- Depends on window size
- Example: 1920x1080 window = show ~1920x1080 world pixels
- Example: Fullscreen 2560x1440 = show even more world

### 4. **Camera Behavior**

**Follow Player** (Primary Mode):
- Camera center follows player position
- Smooth movement (lerp interpolation)
- Player stays roughly centered in viewport

**Future Modes** (Not in initial implementation):
- Look-ahead (camera slightly ahead in movement direction)
- Screen-shake (for impacts)
- Zoom in/out
- Fixed position (for cutscenes)

---

## Implementation Plan

### Phase 1: Core Camera System

#### Step 1.1: Create Camera Module

**File**: `src/camera.rs`

```rust
/// Camera system for viewport management and world-to-screen transformations
use sdl2::rect::Rect;

/// Camera configuration
#[derive(Debug, Clone)]
pub struct CameraConfig {
    /// How quickly camera follows target (0.0 = instant, 1.0 = slow)
    pub smoothing: f32,

    /// Minimum viewport dimensions (reference resolution)
    pub min_viewport_width: u32,
    pub min_viewport_height: u32,

    /// World bounds (optional - camera can't see outside these)
    pub world_bounds: Option<Rect>,
}

impl Default for CameraConfig {
    fn default() -> Self {
        CameraConfig {
            smoothing: 0.1,  // Smooth following
            min_viewport_width: 640,
            min_viewport_height: 360,
            world_bounds: None,  // Unbounded by default
        }
    }
}

/// Camera state and viewport management
pub struct Camera {
    /// Current camera position in world space (center of viewport)
    pub x: f32,
    pub y: f32,

    /// Target position to follow (usually player position)
    target_x: f32,
    target_y: f32,

    /// Viewport dimensions (how much of the world is visible)
    viewport_width: u32,
    viewport_height: u32,

    /// Camera configuration
    config: CameraConfig,
}

impl Camera {
    /// Create a new camera at world position (x, y)
    pub fn new(x: f32, y: f32, viewport_width: u32, viewport_height: u32) -> Self {
        Camera {
            x,
            y,
            target_x: x,
            target_y: y,
            viewport_width,
            viewport_height,
            config: CameraConfig::default(),
        }
    }

    /// Create camera with custom configuration
    pub fn with_config(
        x: f32,
        y: f32,
        viewport_width: u32,
        viewport_height: u32,
        config: CameraConfig,
    ) -> Self {
        Camera {
            x,
            y,
            target_x: x,
            target_y: y,
            viewport_width,
            viewport_height,
            config,
        }
    }

    /// Set camera target (what it should follow)
    pub fn set_target(&mut self, x: f32, y: f32) {
        self.target_x = x;
        self.target_y = y;
    }

    /// Update camera position (call every frame)
    /// Uses lerp interpolation for smooth following
    pub fn update(&mut self) {
        // Lerp towards target (smooth camera movement)
        // lerp(a, b, t) = a + (b - a) * t
        let smoothing = self.config.smoothing;

        self.x += (self.target_x - self.x) * smoothing;
        self.y += (self.target_y - self.y) * smoothing;

        // Apply world bounds if configured
        if let Some(bounds) = self.config.world_bounds {
            let half_viewport_w = (self.viewport_width / 2) as f32;
            let half_viewport_h = (self.viewport_height / 2) as f32;

            // Keep camera from showing outside world bounds
            let min_x = bounds.x() as f32 + half_viewport_w;
            let max_x = (bounds.x() + bounds.width() as i32) as f32 - half_viewport_w;
            let min_y = bounds.y() as f32 + half_viewport_h;
            let max_y = (bounds.y() + bounds.height() as i32) as f32 - half_viewport_h;

            self.x = self.x.clamp(min_x, max_x);
            self.y = self.y.clamp(min_y, max_y);
        }
    }

    /// Convert world coordinates to screen coordinates
    /// Returns None if position is outside viewport
    pub fn world_to_screen(&self, world_x: i32, world_y: i32) -> (i32, i32) {
        let screen_x = world_x - self.x as i32 + (self.viewport_width / 2) as i32;
        let screen_y = world_y - self.y as i32 + (self.viewport_height / 2) as i32;
        (screen_x, screen_y)
    }

    /// Convert screen coordinates to world coordinates
    /// Useful for mouse input (click on world position)
    pub fn screen_to_world(&self, screen_x: i32, screen_y: i32) -> (i32, i32) {
        let world_x = screen_x + self.x as i32 - (self.viewport_width / 2) as i32;
        let world_y = screen_y + self.y as i32 - (self.viewport_height / 2) as i32;
        (world_x, world_y)
    }

    /// Check if a world position is visible in the viewport
    /// Useful for culling (don't render off-screen entities)
    pub fn is_visible(&self, world_x: i32, world_y: i32, margin: i32) -> bool {
        let (screen_x, screen_y) = self.world_to_screen(world_x, world_y);

        screen_x >= -margin
            && screen_x < (self.viewport_width as i32 + margin)
            && screen_y >= -margin
            && screen_y < (self.viewport_height as i32 + margin)
    }

    /// Get camera bounds in world space (what rectangle is visible)
    pub fn get_world_bounds(&self) -> Rect {
        let half_w = (self.viewport_width / 2) as i32;
        let half_h = (self.viewport_height / 2) as i32;

        Rect::new(
            self.x as i32 - half_w,
            self.y as i32 - half_h,
            self.viewport_width,
            self.viewport_height,
        )
    }

    /// Update viewport size (when window resizes)
    pub fn set_viewport_size(&mut self, width: u32, height: u32) {
        self.viewport_width = width.max(self.config.min_viewport_width);
        self.viewport_height = height.max(self.config.min_viewport_height);
    }

    /// Instantly move camera to position (no smoothing)
    /// Useful for: spawning, teleporting, loading game
    pub fn snap_to(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
        self.target_x = x;
        self.target_y = y;
    }

    /// Get viewport dimensions
    pub fn viewport_size(&self) -> (u32, u32) {
        (self.viewport_width, self.viewport_height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_to_screen_center() {
        let camera = Camera::new(1000.0, 500.0, 640, 360);

        // Camera center (1000, 500) should map to screen center
        let (screen_x, screen_y) = camera.world_to_screen(1000, 500);
        assert_eq!(screen_x, 320);  // 640 / 2
        assert_eq!(screen_y, 180);  // 360 / 2
    }

    #[test]
    fn test_screen_to_world() {
        let camera = Camera::new(1000.0, 500.0, 640, 360);

        // Screen center should map to camera center
        let (world_x, world_y) = camera.screen_to_world(320, 180);
        assert_eq!(world_x, 1000);
        assert_eq!(world_y, 500);
    }

    #[test]
    fn test_is_visible() {
        let camera = Camera::new(1000.0, 500.0, 640, 360);

        // Camera center should be visible
        assert!(camera.is_visible(1000, 500, 0));

        // Far outside should not be visible
        assert!(!camera.is_visible(5000, 5000, 0));

        // Just outside with margin should be visible
        assert!(camera.is_visible(-1000, -1000, 2000));
    }

    #[test]
    fn test_camera_smoothing() {
        let mut camera = Camera::new(0.0, 0.0, 640, 360);
        camera.set_target(1000.0, 500.0);

        // Camera shouldn't instantly jump to target
        camera.update();
        assert!(camera.x < 1000.0);
        assert!(camera.y < 500.0);

        // After many updates, should approach target
        for _ in 0..100 {
            camera.update();
        }

        assert!((camera.x - 1000.0).abs() < 0.1);
        assert!((camera.y - 500.0).abs() < 0.1);
    }
}
```

**Key Design Points**:
- **Smooth following**: Uses lerp for cinematic camera movement
- **Coordinate conversion**: Easy world ↔ screen transformation
- **Visibility culling**: Don't render off-screen entities (performance)
- **Configurable**: Can adjust smoothing, bounds, viewport size

#### Step 1.2: Integrate Camera into Game Structure

**File**: `src/main.rs` (modifications)

```rust
mod camera;
use camera::{Camera, CameraConfig};

// In Game struct
pub struct Game<'a> {
    // ... existing fields ...

    // NEW: Camera system
    pub camera: Camera,
}

// In Game::new() or Game::load()
let camera = Camera::new(
    GAME_WIDTH as f32 / 2.0,      // Start at world center X
    GAME_HEIGHT as f32 / 2.0,     // Start at world center Y
    GAME_WIDTH,                    // Initial viewport = reference resolution
    GAME_HEIGHT,
);

// In main() - update camera when window resizes (future work)
// For now, camera viewport matches logical size
```

#### Step 1.3: Update Camera Each Frame

**File**: `src/game/update.rs` or `src/main.rs` (in game loop)

```rust
// In Game::update() or main game loop
impl<'a> Game<'a> {
    pub fn update(&mut self) {
        // ... existing update logic ...

        // Update camera to follow player
        self.camera.set_target(
            self.world.player.x as f32,
            self.world.player.y as f32,
        );
        self.camera.update();
    }
}
```

---

### Phase 2: Update Rendering System

#### Step 2.1: Add Camera Transform to Entity Rendering

**Concept**: All entities need to render relative to camera position, not absolute.

**File**: Modify all entity render methods (Player, Slime, StaticObject, etc.)

**Current Pattern** (example from `player.rs`):
```rust
pub fn render(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
    let scaled_width = self.width * SPRITE_SCALE;
    let scaled_height = self.height * SPRITE_SCALE;

    // CURRENT: Absolute screen position
    let render_x = self.x - (scaled_width / 2) as i32;
    let render_y = self.y - scaled_height as i32;

    let dest_rect = Rect::new(render_x, render_y, scaled_width, scaled_height);
    // ... render sprite ...
}
```

**Updated Pattern**:
```rust
pub fn render(&self, canvas: &mut Canvas<Window>, camera: &Camera) -> Result<(), String> {
    let scaled_width = self.width * SPRITE_SCALE;
    let scaled_height = self.height * SPRITE_SCALE;

    // NEW: Transform world position to screen position
    let (screen_x, screen_y) = camera.world_to_screen(self.x, self.y);

    // Calculate render position from screen position (anchor still applies)
    let render_x = screen_x - (scaled_width / 2) as i32;
    let render_y = screen_y - scaled_height as i32;

    // Optional: Cull if off-screen
    if !camera.is_visible(self.x, self.y, scaled_width as i32) {
        return Ok(()); // Don't render off-screen entities
    }

    let dest_rect = Rect::new(render_x, render_y, scaled_width, scaled_height);
    // ... render sprite ...
}
```

**Apply to All Entities**:
- ✅ `player.rs` - Player::render()
- ✅ `slime.rs` - Slime::render()
- ✅ `collision.rs` - StaticObject::render()
- ✅ `the_entity.rs` - TheEntity::render()
- ✅ `dropped_item.rs` - DroppedItem::render()
- ✅ `attack_effect.rs` - AttackEffect::render() (if applicable)

#### Step 2.2: Update DepthSortable Trait

**File**: `src/render.rs`

```rust
pub trait DepthSortable {
    fn get_depth_y(&self) -> i32;

    // NEW: Add camera parameter to render signature
    fn render(&self, canvas: &mut Canvas<Window>, camera: &Camera) -> Result<(), String>;
}

// Update Renderable enum render method
impl<'a> Renderable<'a> {
    fn render(&self, canvas: &mut Canvas<Window>, camera: &Camera) -> Result<(), String> {
        match self {
            Renderable::Player(p) => p.render(canvas, camera),
            Renderable::Slime(s) => s.render(canvas, camera),
            Renderable::StaticObject(obj) => obj.render(canvas, camera),
            Renderable::TheEntity(e) => e.render(canvas, camera),
            Renderable::DroppedItem(item) => item.render(canvas, camera),
        }
    }
}

// Update main render function
pub fn render_with_depth_sorting(
    canvas: &mut Canvas<Window>,
    camera: &Camera,  // NEW parameter
    player: &Player,
    slimes: &[Slime],
    static_objects: &[StaticObject],
    entities: &[TheEntity],
    dropped_items: &[DroppedItem],
) -> Result<(), String> {
    // ... collect renderables (same as before) ...

    // Sort by depth (same as before)
    renderables.sort_by_key(|(y, _)| *y);

    // Render with camera transform
    for (_, renderable) in renderables {
        renderable.render(canvas, camera)?;  // Pass camera
    }

    Ok(())
}
```

#### Step 2.3: Update Tile Rendering

**File**: `src/tile.rs` (RenderGrid implementation)

Tiles need camera transformation too, plus aggressive culling for performance.

```rust
impl RenderGrid {
    pub fn render(&self, canvas: &mut Canvas<Window>, camera: &Camera) -> Result<(), String> {
        // Calculate visible tile range (culling optimization)
        let world_bounds = camera.get_world_bounds();

        let start_tile_x = (world_bounds.x() / TILE_SIZE as i32).max(0) as usize;
        let start_tile_y = (world_bounds.y() / TILE_SIZE as i32).max(0) as usize;
        let end_tile_x = ((world_bounds.x() + world_bounds.width() as i32) / TILE_SIZE as i32 + 1)
            .min(self.width as i32) as usize;
        let end_tile_y = ((world_bounds.y() + world_bounds.height() as i32) / TILE_SIZE as i32 + 1)
            .min(self.height as i32) as usize;

        // Only render visible tiles
        for tile_y in start_tile_y..end_tile_y {
            for tile_x in start_tile_x..end_tile_x {
                if let Some(tile) = self.get_tile(tile_x, tile_y) {
                    let world_x = (tile_x * TILE_SIZE as usize) as i32;
                    let world_y = (tile_y * TILE_SIZE as usize) as i32;

                    // Transform to screen coordinates
                    let (screen_x, screen_y) = camera.world_to_screen(world_x, world_y);

                    let dest_rect = Rect::new(screen_x, screen_y, TILE_SIZE, TILE_SIZE);
                    tile.render(canvas, dest_rect)?;
                }
            }
        }

        Ok(())
    }
}
```

**Performance Note**: Culling tiles is critical! Rendering a 1000x1000 tile world every frame would be ~1 million draw calls. Culling reduces this to ~100-200 visible tiles.

#### Step 2.4: Keep UI in Screen Space

**Important**: UI elements (health bars, menus, debug text) should **NOT** use camera transform.

**File**: `src/ui/health_bar.rs`, `src/gui/*.rs`, etc.

```rust
// UI rendering - NO camera transform needed
impl HealthBar {
    pub fn render(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        // Render at ABSOLUTE screen positions
        // (UI doesn't move with camera)
        let dest_rect = Rect::new(self.screen_x, self.screen_y, self.width, self.height);
        // ... render UI ...
        Ok(())
    }
}

// World-space health bars (above entities) - USE camera transform
impl EntityHealthBar {
    pub fn render(&self, canvas: &mut Canvas<Window>, camera: &Camera, entity_x: i32, entity_y: i32) -> Result<(), String> {
        // Transform entity position to screen
        let (screen_x, screen_y) = camera.world_to_screen(entity_x, entity_y);

        // Render health bar above entity
        let bar_x = screen_x - (self.width / 2) as i32;
        let bar_y = screen_y - 40;  // Above entity

        let dest_rect = Rect::new(bar_x, bar_y, self.width, self.height);
        // ... render health bar ...
        Ok(())
    }
}
```

**Categorization**:
- **Screen-space UI** (no camera): Menus, HUD, debug overlays
- **World-space UI** (with camera): Health bars above enemies, floating damage numbers

---

### Phase 3: Input System Updates

#### Step 3.1: Mouse Input Transformation

**Problem**: Mouse clicks are in screen coordinates, but we need world coordinates for gameplay (place tiles, spawn enemies, etc.)

**File**: `src/input_system.rs` or `src/main.rs`

```rust
// In event handling
Event::MouseButtonDown { x, y, mouse_btn, .. } => {
    // Convert screen coordinates to world coordinates
    let (world_x, world_y) = self.camera.screen_to_world(x, y);

    match mouse_btn {
        MouseButton::Left => {
            // Place tile at world position
            let tile_x = (world_x / TILE_SIZE as i32).max(0) as usize;
            let tile_y = (world_y / TILE_SIZE as i32).max(0) as usize;
            // ... place tile logic ...
        }
        MouseButton::Right => {
            // Spawn slime at world position
            self.world.slimes.push(Slime::new(world_x, world_y, /* ... */));
        }
        _ => {}
    }
}
```

**Important**: All mouse interactions need `screen_to_world` conversion!

#### Step 3.2: Keyboard Input (No Changes Needed)

**Good News**: Keyboard input for player movement doesn't need changes!
- Player movement updates `player.x`, `player.y` in **world coordinates**
- Camera automatically follows player
- Rendering handles transformation

---

### Phase 4: Window Resizing and Fullscreen

#### Step 4.1: Remove Logical Size Constraint

**Current Problem**: `canvas.set_logical_size(640, 360)` forces pixel scaling.

**File**: `src/main.rs`

```rust
// BEFORE (current):
let mut canvas = window.into_canvas().build()?;
canvas.set_logical_size(GAME_WIDTH, GAME_HEIGHT)?;  // ❌ Forces scaling

// AFTER (new):
let mut canvas = window.into_canvas().build()?;
// ✅ No logical size - use actual window dimensions

// Update camera to match window size
let (window_w, window_h) = canvas.output_size()?;
self.camera.set_viewport_size(window_w, window_h);
```

#### Step 4.2: Handle Window Resize Events

```rust
// In event loop
Event::Window { win_event: WindowEvent::Resized(width, height), .. } => {
    // Update camera viewport to new window size
    self.camera.set_viewport_size(width as u32, height as u32);
    println!("Window resized to {}x{}", width, height);
}
```

#### Step 4.3: Add Fullscreen Toggle

```rust
// In event loop
Event::KeyDown { keycode: Some(Keycode::F11), .. } => {
    // Toggle fullscreen
    let window = self.canvas.window_mut();
    let is_fullscreen = window.fullscreen_state() != sdl2::video::FullscreenType::Off;

    if is_fullscreen {
        window.set_fullscreen(sdl2::video::FullscreenType::Off)?;
    } else {
        window.set_fullscreen(sdl2::video::FullscreenType::Desktop)?;
    }

    // Update camera viewport
    let (new_w, new_h) = self.canvas.output_size()?;
    self.camera.set_viewport_size(new_w, new_h);
}
```

#### Step 4.4: Remove Fixed Window Scale Calculation

**File**: `src/main.rs`

```rust
// BEFORE (remove this):
fn calculate_window_scale(video_subsystem: &sdl2::VideoSubsystem) -> u32 {
    // ... complicated scaling logic ...
}

let window_scale = calculate_window_scale(&video_subsystem);
let window_width = GAME_WIDTH * window_scale;  // ❌ Fixed multiples
let window_height = GAME_HEIGHT * window_scale;

// AFTER (new):
// Start with reference resolution, allow resizing
let window = video_subsystem
    .window("Game 1", 1280, 720)  // Reasonable default (720p)
    .position_centered()
    .resizable()  // ✅ Allow resizing
    .build()?;
```

---

### Phase 5: World Bounds and Camera Constraints

#### Step 5.1: Define World Size

Currently, the world is implicitly bounded by `GAME_WIDTH x GAME_HEIGHT`, but with a camera, we want a larger world.

**File**: `src/game/world.rs`

```rust
// NEW: Define actual world dimensions
pub const WORLD_WIDTH: u32 = 3200;   // 5x reference width
pub const WORLD_HEIGHT: u32 = 1800;  // 5x reference height

// Keep reference resolution for UI scaling
pub const REFERENCE_WIDTH: u32 = 640;
pub const REFERENCE_HEIGHT: u32 = 360;
```

#### Step 5.2: Update Tile Grid Size

```rust
// Increase world grid to match larger world
impl WorldGrid {
    pub fn new_default() -> Self {
        WorldGrid::new(
            WORLD_WIDTH / TILE_SIZE,   // More tiles
            WORLD_HEIGHT / TILE_SIZE,
        )
    }
}
```

#### Step 5.3: Apply Camera Bounds

```rust
// In main() or Game::new()
let camera_config = CameraConfig {
    smoothing: 0.1,
    min_viewport_width: REFERENCE_WIDTH,
    min_viewport_height: REFERENCE_HEIGHT,
    world_bounds: Some(Rect::new(0, 0, WORLD_WIDTH, WORLD_HEIGHT)),  // ✅ Constrain camera
};

let camera = Camera::with_config(
    WORLD_WIDTH as f32 / 2.0,
    WORLD_HEIGHT as f32 / 2.0,
    REFERENCE_WIDTH,
    REFERENCE_HEIGHT,
    camera_config,
);
```

#### Step 5.4: Update Boundary Walls

**File**: `src/game/systems.rs`

```rust
// OLD: Walls at 640x360
const GAME_WIDTH: u32 = 640;
const GAME_HEIGHT: u32 = 360;

// NEW: Walls at world bounds
use crate::game::world::{WORLD_WIDTH, WORLD_HEIGHT};

impl Systems {
    pub fn new() -> Self {
        let boundary_thickness = 50;

        let boundary_walls = vec![
            // Top wall
            StaticObject::new(0, -(boundary_thickness as i32), WORLD_WIDTH, boundary_thickness),
            // Left wall
            StaticObject::new(-(boundary_thickness as i32), 0, boundary_thickness, WORLD_HEIGHT),
            // Right wall
            StaticObject::new(WORLD_WIDTH as i32, 0, boundary_thickness, WORLD_HEIGHT),
            // Bottom wall
            StaticObject::new(0, WORLD_HEIGHT as i32, WORLD_WIDTH, boundary_thickness),
        ];

        // ... rest of initialization ...
    }
}
```

---

## Testing Plan

### Phase 1 Testing (Camera System)

**Manual Tests**:
1. ✅ Run game → Camera follows player smoothly (not jerky)
2. ✅ Move player to edge of world → Camera stops at world bounds
3. ✅ Spawn slime → Slime renders at correct position relative to camera
4. ✅ Check debug logs → Camera position updates match player position

**Expected Behavior**:
- Player stays roughly centered on screen
- World scrolls smoothly as player moves
- Entities render at correct relative positions

### Phase 2 Testing (Rendering Updates)

**Manual Tests**:
1. ✅ All entities render correctly (player, slimes, tiles, items)
2. ✅ Depth sorting still works (entities pass behind/in front correctly)
3. ✅ UI elements stay at fixed screen positions (health bars don't move with camera)
4. ✅ Menus render at screen center (not world center)

**Performance Test**:
- Monitor FPS with large world
- Verify tile culling works (only visible tiles rendered)
- Check entity culling reduces render calls

### Phase 3 Testing (Input)

**Manual Tests**:
1. ✅ Left click → Tile placed at cursor position in world
2. ✅ Right click → Slime spawns at cursor position in world
3. ✅ Move camera, then click → Still places at correct world position
4. ✅ UI clicks work (menus, buttons) without world transform

### Phase 4 Testing (Window Resizing)

**Manual Tests**:
1. ✅ Resize window → Viewport shows more/less world
2. ✅ Press F11 → Fullscreen works, shows maximum world area
3. ✅ F11 again → Returns to windowed mode correctly
4. ✅ Verify UI scales/positions correctly on different window sizes

**Aspect Ratio Test**:
- Try 16:9 (1920x1080), 16:10 (1920x1200), 4:3 (1024x768)
- UI should look reasonable on all aspect ratios

### Phase 5 Testing (World Bounds)

**Manual Tests**:
1. ✅ Move player to world edge → Can't move further
2. ✅ Camera stops at world edge (doesn't show black void)
3. ✅ Larger world has more space to explore
4. ✅ Tile grid covers entire world

---

## Implementation Checklist

### Camera System Core
- [ ] Create `src/camera.rs` with Camera struct
- [ ] Implement camera smoothing and following
- [ ] Add world_to_screen and screen_to_world conversions
- [ ] Write camera tests
- [ ] Integrate camera into Game struct

### Rendering Updates
- [ ] Add camera parameter to DepthSortable trait
- [ ] Update Player::render() with camera transform
- [ ] Update Slime::render() with camera transform
- [ ] Update StaticObject::render() with camera transform
- [ ] Update TheEntity::render() with camera transform
- [ ] Update DroppedItem::render() with camera transform
- [ ] Update tile rendering with camera culling
- [ ] Update render_with_depth_sorting() signature
- [ ] Verify UI rendering stays in screen space

### Input System
- [ ] Add screen_to_world for mouse clicks
- [ ] Update tile placement with world coordinates
- [ ] Update enemy spawning with world coordinates
- [ ] Test mouse input at different camera positions

### Window Management
- [ ] Remove set_logical_size() constraint
- [ ] Add window resize event handling
- [ ] Implement fullscreen toggle (F11)
- [ ] Update camera viewport on resize
- [ ] Remove calculate_window_scale() function
- [ ] Set resizable window flag

### World Expansion
- [ ] Define WORLD_WIDTH and WORLD_HEIGHT constants
- [ ] Increase tile grid size
- [ ] Update boundary walls to world size
- [ ] Apply camera world bounds
- [ ] Test movement in expanded world

### Polish
- [ ] Add camera configuration options
- [ ] Tune camera smoothing value
- [ ] Test various window sizes
- [ ] Verify performance with large world
- [ ] Update documentation

---

## Expected Outcomes

### Before Implementation
```
Window Size: 1280x720 (2x scale of 640x360)
Result: Game renders at 640x360, scaled up 2x (bigger pixels)
World Visible: 640x360 pixels of world
```

### After Implementation
```
Window Size: 1280x720
Result: Game renders at native 1280x720
World Visible: 1280x720 pixels of world (2x more content!)

Window Size: 1920x1080 (fullscreen)
Result: Game renders at native 1920x1080
World Visible: 1920x1080 pixels of world (3x more content!)
```

**Key Benefit**: Players with larger monitors see MORE of the game world, not just bigger pixels.

---

## Performance Considerations

### Rendering Optimizations

**Tile Culling** (Critical):
- Before: Render all tiles (~10,000 for 100x100 grid)
- After: Render only visible tiles (~100-200)
- Performance Gain: 50-100x reduction in draw calls

**Entity Culling**:
- Use `camera.is_visible()` to skip off-screen entities
- Especially important for particle effects and projectiles
- Performance Gain: Scales with world size

**Depth Sorting** (No Change):
- Still O(n log n) for visible entities
- But n is smaller due to culling
- Performance: Minimal impact

### Expected Performance
- Target: 60 FPS at 1920x1080 with 100+ entities
- Bottleneck: Tile rendering (optimized with culling)
- Future: Consider sprite batching for further optimization

---

## Future Enhancements

### Camera Features
- **Zoom**: Scale viewport (see more/less world)
- **Screen shake**: Camera offset on impact
- **Look-ahead**: Camera leads player in movement direction
- **Cinematic mode**: Fixed camera for cutscenes
- **Bounds padding**: Keep player away from screen edges

### Window Features
- **Borderless fullscreen**: Better alt-tab experience
- **Resolution selection**: Settings menu for window size
- **Aspect ratio lock**: Optional letterboxing for 16:9
- **VSync toggle**: Settings for screen tearing

### World Features
- **Infinite world**: Procedural generation, no bounds
- **Minimap**: Show full world with camera position
- **Fog of war**: Hide unexplored areas
- **Multiple cameras**: Split-screen multiplayer

---

## Related Documentation

- **Entity Pattern**: `docs/patterns/entity-pattern.md` - Anchor positioning still applies
- **Depth Sorting**: `docs/systems/depth-sorting-render-system.md` - Rendering architecture
- **Screen-Space UI**: `docs/features/screen-space-menus.md` - UI rendering (no camera transform)
- **Coordinate Systems**: This document defines the new coordinate transform system

---

## Rust Learning Notes

### Concepts Demonstrated

**1. Coordinate Transformations**:
```rust
// Linear transformation: world → screen
let screen_x = world_x - camera.x + viewport_width / 2;

// Inverse transformation: screen → world
let world_x = screen_x + camera.x - viewport_width / 2;
```

**2. Lerp (Linear Interpolation)**:
```rust
// Smooth camera movement
// lerp(current, target, t) = current + (target - current) * t
// t = 0.0 → no movement, t = 1.0 → instant snap
self.x += (self.target_x - self.x) * smoothing;
```

**3. Culling with Spatial Queries**:
```rust
// Only process entities in viewport bounds
if camera.is_visible(entity.x, entity.y, margin) {
    entity.render(canvas, camera)?;
}
```

**4. Clamping**:
```rust
// Keep camera inside world bounds
self.x = self.x.clamp(min_x, max_x);
```

**5. Trait Updates (Breaking Changes)**:
```rust
// Adding camera parameter to trait is a breaking change
// All implementations must update their signatures
trait DepthSortable {
    fn render(&self, canvas: &mut Canvas<Window>, camera: &Camera) -> Result<(), String>;
}
```

---

## Implementation Notes

### Order of Implementation

**Recommended Order**:
1. **Phase 1** (Camera system) - Get basic camera following working
2. **Phase 2** (Rendering) - Update all render methods, test with fixed viewport
3. **Phase 5** (World bounds) - Expand world before testing resizing
4. **Phase 3** (Input) - Fix mouse input with camera transform
5. **Phase 4** (Resizing) - Enable dynamic viewport last

**Why This Order?**:
- Test camera with fixed viewport first (simpler debugging)
- Expand world to see camera benefits
- Resizing is final step (relies on everything else working)

### Common Pitfalls

**❌ Pitfall 1: Forgetting Camera Parameter**
```rust
// WRONG - Missing camera parameter
entity.render(canvas)?;

// CORRECT
entity.render(canvas, camera)?;
```

**❌ Pitfall 2: Confusing Screen-Space and World-Space UI**

There are TWO types of UI - know which one you're building:

```rust
// SCREEN-SPACE UI - Fixed position on screen (inventory, menus, HUD)
// WRONG - Applying camera transform to screen-space UI
let (screen_x, screen_y) = camera.world_to_screen(menu.x, menu.y);

// CORRECT - Screen-space UI uses direct screen coordinates
canvas.fill_rect(Rect::new(menu.x, menu.y, menu.width, menu.height))?;
```

```rust
// WORLD-SPACE UI - Follows entities in the world (health bars, floating text)
// WRONG - Forgetting camera transform for world-space UI
canvas.fill_rect(Rect::new(entity.x, entity.y, width, height))?;  // Will be misaligned!

// CORRECT - World-space UI needs camera transform
let (screen_x, screen_y) = camera.world_to_screen(entity.x, entity.y);
canvas.fill_rect(Rect::new(screen_x, screen_y, width, height))?;
```

**Decision Tree**: Does this UI element follow something in the world?
- **YES** (health bars, damage numbers, entity labels) → Use `camera.world_to_screen()`
- **NO** (menus, HUD, debug overlays) → Use direct screen coordinates

**❌ Pitfall 3: Forgetting Camera Transform in Rendering Loop**

**Real Bug Example** (Fixed 2025-12-27): Health bars and floating text were misaligned after implementing camera following because the render loop forgot to transform coordinates.

```rust
// WRONG - Health bars appear in wrong location when camera moves
for text in &floating_texts {
    renderer.render(canvas, text.x as i32, text.y as i32, &text.text)?;
    // Bug: text.x and text.y are world coordinates, but renderer expects screen coordinates!
}

// CORRECT - Transform world coordinates before rendering
for text in &floating_texts {
    let (screen_x, screen_y) = camera.world_to_screen(text.x as i32, text.y as i32);
    renderer.render(canvas, screen_x, screen_y, &text.text)?;
}
```

**Checklist for World-Space UI**:
- [ ] Calculate world position (entity.x, entity.y)
- [ ] Call `camera.world_to_screen(world_x, world_y)`
- [ ] Pass screen coordinates to renderer
- [ ] Test by moving player around world - UI should follow entity

See `src/main.rs` lines 836-890 for the correct implementation.

**❌ Pitfall 4: Using Player Position for World Bounds**
```rust
// WRONG - Player can't reach world edges
if player.x < WORLD_WIDTH { ... }

// CORRECT - Player anchor can be at world edge
if player.x <= WORLD_WIDTH { ... }
```

**❌ Pitfall 5: Not Culling Tiles**
```rust
// WRONG - Renders entire world every frame (kills performance)
for y in 0..world.height {
    for x in 0..world.width {
        render_tile(x, y);
    }
}

// CORRECT - Only render visible tiles
let visible_rect = camera.get_world_bounds();
for y in visible_y_start..visible_y_end {
    for x in visible_x_start..visible_x_end {
        render_tile(x, y);
    }
}
```

---

## Summary

This camera and viewport system transforms Game1 from a fixed 640x360 screen to a **virtual camera** viewing a larger world. Key benefits:

✅ **Larger worlds**: Expand beyond reference resolution
✅ **Better scaling**: Fullscreen shows more content, not bigger pixels
✅ **Smooth camera**: Cinematic following with lerp interpolation
✅ **Performance**: Culling prevents rendering off-screen content
✅ **Flexible UI**: Screen-space UI stays fixed while world scrolls

**Next Steps**: Begin Phase 1 implementation by creating the `Camera` module and integrating it into the game structure.
