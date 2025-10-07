# Health Bar System - Implementation Plan

## Overview
This document outlines the implementation plan for health bars - the first **world-space HUD component** in Game1. Health bars display above entities to show their current health status.

**Parent System**: See `docs/ui-system.md` for the overall UI architecture

**Status**: 📋 **PLANNED** (Not yet implemented)

## Goals
1. **Visual Clarity**: Players should immediately understand entity health status
2. **Minimal Overhead**: Health bars should be lightweight and performant
3. **Extensibility**: Easy to add health bars to new entity types
4. **Customizability**: Support different bar styles (colors, sizes, positions)
5. **Integration**: Work seamlessly with existing rendering pipeline

## System Context

Health bars are **world-space HUD components** as defined in `docs/ui-system.md`. This means they:
- ✅ Render above entities in world coordinates
- ✅ Move with entities automatically
- ✅ Use procedural rendering (SDL2 primitives)
- ✅ Are stateless components (created once, reused)

### What We're Building On

**Existing Systems**:
- ✅ **Stats System** (`docs/player-stats-system.md`) - Provides `health.percentage()` method
- ✅ **Animation System** (`docs/animation-system.md`) - Not used by health bars, but entities use it
- ✅ **Save System** (`docs/save-system-design.md`) - Health values already persisted

**New Foundation**:
- ✅ **World-Space HUD System** (`docs/ui-system.md`) - Architecture this implements

### Design Decision: Procedural vs Sprite-Based

Following the world-space HUD system architecture, health bars use **procedural rendering** (SDL2 rectangles):

**Why?**
- Dynamic width based on health percentage
- Easy color customization without new assets
- Lightweight implementation (~100 lines)
- Foundation for future HUD components (damage numbers, status icons)

**See `docs/ui-system.md`** for the full rationale and architectural guidelines.

## Implementation

This section provides the concrete implementation for the first world-space HUD component.

**Architecture Pattern**: See `docs/ui-system.md` → World-Space HUD → Component-Based Rendering

### Component 1: HealthBar Struct

**File**: `src/ui/health_bar.rs`

**Follows**: World-space HUD component pattern (stateless, procedural rendering)

```rust
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::pixels::Color;

/// Configuration for health bar appearance
#[derive(Debug, Clone)]
pub struct HealthBarStyle {
    pub width: u32,          // Bar width in pixels
    pub height: u32,         // Bar height in pixels
    pub offset_y: i32,       // Offset above entity (negative = above)
    pub background_color: Color,  // Background bar color (dark gray)
    pub health_color: Color,      // Filled portion color (green)
    pub low_health_color: Color,  // Color when health < 30% (red)
    pub border_color: Color,      // Border color (black)
    pub border_thickness: u32,    // Border thickness in pixels
    pub show_when_full: bool,     // Show bar even at 100% health?
}

impl Default for HealthBarStyle {
    fn default() -> Self {
        HealthBarStyle {
            width: 32,
            height: 4,
            offset_y: -8,  // 8 pixels above entity
            background_color: Color::RGB(50, 50, 50),     // Dark gray
            health_color: Color::RGB(0, 200, 0),          // Green
            low_health_color: Color::RGB(200, 0, 0),      // Red
            border_color: Color::RGB(0, 0, 0),            // Black
            border_thickness: 1,
            show_when_full: false,  // Hide when at full health
        }
    }
}

/// A health bar component that can be attached to entities
pub struct HealthBar {
    style: HealthBarStyle,
}

impl HealthBar {
    /// Creates a new health bar with default styling
    pub fn new() -> Self {
        HealthBar {
            style: HealthBarStyle::default(),
        }
    }

    /// Creates a health bar with custom styling
    pub fn with_style(style: HealthBarStyle) -> Self {
        HealthBar { style }
    }

    /// Renders the health bar above an entity
    ///
    /// # Parameters
    /// - `canvas`: SDL2 canvas to render to
    /// - `entity_x`, `entity_y`: Entity's world position
    /// - `entity_width`, `entity_height`: Entity's rendered size (after scaling)
    /// - `health_percentage`: Current health as 0.0-1.0 (from stats.health.percentage())
    ///
    /// # Example
    /// ```rust
    /// let health_bar = HealthBar::new();
    /// health_bar.render(
    ///     &mut canvas,
    ///     player.x,
    ///     player.y,
    ///     player.width * 2,  // Account for 2x sprite scaling
    ///     player.height * 2,
    ///     player.stats.health.percentage()
    /// )?;
    /// ```
    pub fn render(
        &self,
        canvas: &mut Canvas<Window>,
        entity_x: i32,
        entity_y: i32,
        entity_width: u32,
        entity_height: u32,
        health_percentage: f32,
    ) -> Result<(), String> {
        // Don't render if health is full and show_when_full is false
        if !self.style.show_when_full && health_percentage >= 1.0 {
            return Ok(());
        }

        // Calculate bar position (centered above entity)
        let bar_x = entity_x + (entity_width as i32 / 2) - (self.style.width as i32 / 2);
        let bar_y = entity_y + self.style.offset_y;

        // Background bar (full width)
        let background_rect = Rect::new(
            bar_x,
            bar_y,
            self.style.width,
            self.style.height,
        );
        canvas.set_draw_color(self.style.background_color);
        canvas.fill_rect(background_rect)?;

        // Health bar (filled portion)
        let health_width = (self.style.width as f32 * health_percentage.clamp(0.0, 1.0)) as u32;
        if health_width > 0 {
            let health_rect = Rect::new(
                bar_x,
                bar_y,
                health_width,
                self.style.height,
            );

            // Use red color if health is low (<30%), otherwise green
            let fill_color = if health_percentage < 0.3 {
                self.style.low_health_color
            } else {
                self.style.health_color
            };

            canvas.set_draw_color(fill_color);
            canvas.fill_rect(health_rect)?;
        }

        // Border (optional)
        if self.style.border_thickness > 0 {
            let border_rect = Rect::new(
                bar_x,
                bar_y,
                self.style.width,
                self.style.height,
            );
            canvas.set_draw_color(self.style.border_color);
            canvas.draw_rect(border_rect)?;
        }

        Ok(())
    }

    /// Updates the health bar's style
    pub fn set_style(&mut self, style: HealthBarStyle) {
        self.style = style;
    }

    /// Gets a reference to the current style
    pub fn style(&self) -> &HealthBarStyle {
        &self.style
    }
}
```

**Design Decisions Explained**:

1. **Procedural Rendering (SDL2 Primitives)**: Uses `fill_rect()` and `draw_rect()` instead of sprite textures
   - **Pro**: Dynamic width calculations, easy color changes, no texture assets needed
   - **Con**: Limited visual complexity (no gradients, no textures)
   - **Extensibility**: Could add texture support later for fancy bars

2. **Style Configuration Struct**: Separates appearance from logic
   - **Pro**: Easy to create variants (enemy bars, boss bars, ally bars)
   - **Con**: Slightly more verbose
   - **Extensibility**: Add new style fields without changing render logic

3. **Health Percentage Parameter**: Takes 0.0-1.0 instead of current/max
   - **Pro**: Caller handles stat lookups, bar doesn't need to know about Stats struct
   - **Con**: Caller must remember to call `.percentage()`
   - **Extensibility**: Works with any system that tracks percentages

4. **Position Relative to Entity**: Bar positions itself based on entity bounds
   - **Pro**: Automatic centering, moves with entity
   - **Con**: Caller must pass entity dimensions
   - **Extensibility**: Works for any entity size

### Component 2: UI Module Structure

**File**: `src/ui/mod.rs`

**Important**: This creates the foundation for the world-space HUD system defined in `docs/ui-system.md`.

```rust
//! World-Space HUD Components
//!
//! This module provides UI elements that render above entities in world coordinates.
//! These components follow the stateless, procedural rendering pattern.
//!
//! # Architecture
//! See `docs/ui-system.md` for the full world-space HUD architecture.
//!
//! # Available Components
//! - `HealthBar` - Displays health above entities
//!
//! # Future Components (see ui-system.md)
//! - `DamageNumber` - Floating damage/heal numbers
//! - `StatusIcon` - Buff/debuff indicators
//! - `NameTag` - Entity name labels

pub mod health_bar;

pub use health_bar::{HealthBar, HealthBarStyle};
```

**Note**: Screen-space GUI (inventory, menus) will be in a separate `src/gui/` module (see `docs/ui-system.md`).

### Component 3: Integration with Entities

Health bars follow the **world-space HUD integration pattern** from `docs/ui-system.md`.

**Key principle**: Components are **not stored inside entity structs**. They're created once in main.rs and reused.

**Why this pattern?** (See `docs/ui-system.md` for full explanation)
1. **Separation of concerns**: Entities handle game logic, UI handles display
2. **Flexibility**: Can render same entity with different UI styles
3. **Save compatibility**: UI state doesn't need serialization
4. **Memory efficiency**: No per-entity UI overhead
5. **Extensibility**: Add new HUD components without modifying entities

**Integration Pattern**:

```rust
// In Player struct (src/player.rs)
impl<'a> Player<'a> {
    pub fn render(&self, canvas: &mut Canvas<Window>, health_bar: &HealthBar) -> Result<(), String> {
        const SPRITE_SCALE: u32 = 2;
        let scaled_width = self.width * SPRITE_SCALE;
        let scaled_height = self.height * SPRITE_SCALE;
        let dest_rect = Rect::new(self.x, self.y, scaled_width, scaled_height);

        // Render entity sprite (existing code)
        if let Some(sprite_sheet) = self.animation_controller.get_current_sprite_sheet() {
            sprite_sheet.render_directional(canvas, dest_rect, false, self.direction)?;
        }

        // Render health bar above entity (NEW!)
        if self.state.is_alive() {
            health_bar.render(
                canvas,
                self.x,
                self.y,
                scaled_width,
                scaled_height,
                self.stats.health.percentage(),
            )?;
        }

        Ok(())
    }
}
```

**Alternative Pattern** (for main.rs):

```rust
// Create health bars once in main()
let player_health_bar = HealthBar::new();
let enemy_health_bar = HealthBar::with_style(HealthBarStyle {
    health_color: Color::RGB(150, 0, 150),  // Purple for enemies
    ..Default::default()
});

// In render loop
player.render(&mut canvas)?;  // Renders sprite only
player_health_bar.render(
    &mut canvas,
    player.x,
    player.y,
    player.width * 2,
    player.height * 2,
    player.stats.health.percentage(),
)?;

for slime in &slimes {
    slime.render(&mut canvas)?;
    if slime.is_alive {
        enemy_health_bar.render(
            &mut canvas,
            slime.x,
            slime.y,
            slime.width * 2,
            slime.height * 2,
            slime.health as f32 / 8.0,  // Slimes have max 8 HP
        )?;
    }
}
```

**Recommendation**: Use the alternative pattern (main.rs manages bars). This keeps entity code unchanged and centralizes UI rendering.

## Rendering Order

Health bars must render in the **world-space HUD layer** as defined in `docs/ui-system.md`:

```rust
// Correct render order (in main.rs game loop)
1. World/background tiles
2. Entities (player, slimes)
3. Attack effects (punch effect)
4. World-Space HUD (health bars) ← This component renders here
5. Screen-Space GUI (menus, inventory - future) ← Different system
```

**Why this layer?** (See `docs/ui-system.md` for full layering explanation)
- Renders on top of entities so bars aren't occluded
- Renders below screen-space GUI so menus cover bars
- Maintains visual hierarchy

## Extensibility

Health bars follow the **world-space HUD extensibility model** from `docs/ui-system.md`.

### Easy: Adding Health Bars to New Entities

**Step 1**: Entity already exposes position and health (existing requirement for all entities)

**Step 2**: Render bar in main loop using the world-space HUD pattern:
```rust
// For a new Goblin entity
for goblin in &goblins {
    goblin.render(&mut canvas)?;
    enemy_health_bar.render(
        &mut canvas,
        goblin.x,
        goblin.y,
        goblin.width * 2,
        goblin.height * 2,
        goblin.health_percentage(),  // Assumes goblin has this method
    )?;
}
```

**That's it!** This is the power of the component pattern - no changes to HealthBar code, no changes to Goblin struct.

### Medium: Creating Custom Bar Styles

```rust
// Boss health bar (larger, positioned at top of screen)
let boss_health_bar = HealthBar::with_style(HealthBarStyle {
    width: 200,
    height: 20,
    offset_y: 50,  // Below entity instead of above
    health_color: Color::RGB(255, 165, 0),  // Orange
    border_thickness: 2,
    show_when_full: true,  // Always visible
    ..Default::default()
});

// Friendly NPC health bar (blue)
let ally_health_bar = HealthBar::with_style(HealthBarStyle {
    health_color: Color::RGB(0, 150, 255),  // Blue
    show_when_full: false,
    ..Default::default()
});
```

### Advanced: Adding Visual Effects

**Smooth Bar Transitions** (health doesn't snap instantly):

```rust
pub struct AnimatedHealthBar {
    health_bar: HealthBar,
    current_percentage: f32,  // Interpolated value
    target_percentage: f32,   // Actual health value
    interpolation_speed: f32, // How fast bar catches up
}

impl AnimatedHealthBar {
    pub fn update(&mut self, dt: f32, target_health: f32) {
        self.target_percentage = target_health;

        // Smoothly interpolate current towards target
        let diff = self.target_percentage - self.current_percentage;
        self.current_percentage += diff * self.interpolation_speed * dt;
    }

    pub fn render(&self, /* ... */) -> Result<(), String> {
        self.health_bar.render(/* ... */, self.current_percentage)
    }
}
```

**Damage Flash Effect**:

```rust
pub struct FlashingHealthBar {
    health_bar: HealthBar,
    flash_timer: Option<Instant>,
    flash_duration: f32,
}

impl FlashingHealthBar {
    pub fn trigger_flash(&mut self) {
        self.flash_timer = Some(Instant::now());
    }

    pub fn render(&self, canvas: &mut Canvas<Window>, /* ... */) -> Result<(), String> {
        // Modify style temporarily if flashing
        let style = if let Some(timer) = self.flash_timer {
            if timer.elapsed().as_secs_f32() < self.flash_duration {
                // Flash white
                HealthBarStyle {
                    health_color: Color::RGB(255, 255, 255),
                    ..self.health_bar.style().clone()
                }
            } else {
                self.health_bar.style().clone()
            }
        } else {
            self.health_bar.style().clone()
        };

        // Render with modified style
        // ...
    }
}
```

### Future Extensions

**Texture-Based Bars** (for fancy visuals):

```rust
pub enum BarRenderMode {
    Procedural,  // Current: SDL2 rectangles
    Textured {   // Future: Sprite-based bars
        background_texture: Texture,
        fill_texture: Texture,
        border_texture: Option<Texture>,
    },
}
```

**Multi-Segment Bars** (for large health pools):

```rust
pub struct SegmentedHealthBar {
    segments: Vec<HealthBarSegment>,  // Multiple bars for 100+ HP
    segment_size: f32,                // Health per segment (e.g., 20 HP)
}
```

## Implementation Plan

### Phase 1: Core Health Bar ✅ Ready to Implement
- [x] Design health bar system (this document)
- [ ] Create `src/ui/` module
- [ ] Implement `HealthBar` struct with procedural rendering
- [ ] Implement `HealthBarStyle` configuration
- [ ] Add basic unit tests

**Estimated Time**: 1-2 hours

### Phase 2: Entity Integration
- [ ] Add health bar rendering to Player in main.rs
- [ ] Add health bar rendering to Slime in main.rs
- [ ] Test with different health values
- [ ] Verify bar positions at different entity sizes

**Estimated Time**: 30-60 minutes

### Phase 3: Polish and Customization
- [ ] Create enemy-specific bar style (different color)
- [ ] Implement "hide when full" logic
- [ ] Add low-health color transition (red below 30%)
- [ ] Test with multiple entities on screen

**Estimated Time**: 30-45 minutes

### Phase 4: (Optional) Advanced Features
- [ ] Implement smooth bar transitions (AnimatedHealthBar)
- [ ] Add damage flash effect (FlashingHealthBar)
- [ ] Add damage number popups (new component)
- [ ] Create boss health bar variant

**Estimated Time**: 2-4 hours (only if desired)

## Performance Considerations

### Rendering Cost
- **Per bar**: ~3-5 SDL2 draw calls (background, fill, border)
- **Per frame**: ~10-20 draw calls for 5 entities
- **Impact**: Negligible (SDL2 rectangles are extremely fast)

### Memory Overhead
- **HealthBar struct**: ~80 bytes (style configuration)
- **Per entity**: 0 bytes (bars created on-demand)
- **Impact**: Minimal

### Optimization Opportunities
1. **Culling**: Don't render bars for off-screen entities
2. **Batching**: Render all bars in one pass (current design already does this)
3. **Caching**: Reuse bar instances instead of creating new ones each frame (current design already does this)

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_bar_centering() {
        let bar = HealthBar::new();
        let entity_x = 100;
        let entity_width = 32;
        let bar_width = 32;

        // Bar should center above entity
        let expected_x = 100 + (32 / 2) - (32 / 2);
        // (Calculation done in render, verify manually)
    }

    #[test]
    fn test_hide_when_full() {
        let bar = HealthBar::new();
        // Verify render returns early when health is 1.0
    }

    #[test]
    fn test_low_health_color() {
        // Verify color changes below 30% threshold
    }
}
```

### Integration Tests

1. **Visual Test**: Spawn entities with different health values (100%, 50%, 25%, 0%)
2. **Movement Test**: Move entity around, verify bar follows correctly
3. **Damage Test**: Take damage, verify bar width updates immediately
4. **Multi-Entity Test**: Spawn 10+ entities, verify all bars render correctly

## Save System Compatibility

**Do health bars need to be saved?**

**No.** Health bars are purely visual UI elements. The underlying health data (in `Stats` struct) is already saved by the save system.

On load:
1. Entities restore their health values (already implemented)
2. Health bars are recreated in main.rs (no state to restore)
3. Bars automatically display correct health percentage

**No changes needed to save system.**

## Comparison with Alternatives

### Alternative 1: Sprite-Based Health Bars

**Approach**: Pre-render bar graphics as sprite sheets

**Pros**:
- Can have fancy textures/gradients
- Consistent with existing rendering pipeline

**Cons**:
- Requires creating 100+ sprites for different fill amounts
- Inflexible (changing colors requires new sprites)
- More memory overhead

**Verdict**: ❌ Too inflexible for first implementation

### Alternative 2: Shader-Based Health Bars

**Approach**: Use OpenGL/Vulkan shaders for fancy effects

**Pros**:
- Can do gradients, glows, animations
- GPU-accelerated

**Cons**:
- SDL2 has limited shader support
- Way overkill for simple bars
- Steep learning curve

**Verdict**: ❌ Not appropriate for this project

### Alternative 3: Immediate Mode GUI (imgui)

**Approach**: Use imgui-rs for all UI

**Pros**:
- Professional UI toolkit
- Built-in layout, styling, widgets

**Cons**:
- Heavy dependency
- Designed for editor UI, not in-game HUD
- Adds complexity

**Verdict**: ❌ Too heavyweight, designed for different use case

### Alternative 4: Procedural Rendering (Chosen Approach)

**Approach**: SDL2 primitive rendering (rectangles)

**Pros**:
- ✅ Simple, minimal code
- ✅ No external dependencies
- ✅ Flexible (easy color/size changes)
- ✅ Performant (hardware accelerated)

**Cons**:
- Limited visual complexity (no textures)

**Verdict**: ✅ **Best choice for first implementation**

## Rust Learning Opportunities

This health bar system teaches:

1. **Separation of Concerns**: UI logic separate from entity logic
2. **Struct Configuration**: Style structs for customization
3. **Result Error Handling**: SDL2 rendering returns `Result<(), String>`
4. **Default Trait**: Implementing sensible defaults for configuration
5. **Module Organization**: Creating a new `ui` module
6. **Color Manipulation**: Working with SDL2's Color type
7. **Rectangle Math**: Calculating positions and centering

## Questions to Consider

1. **Should health bars have borders?**
   - Recommendation: Yes, optional via `border_thickness` (default 1px)

2. **Should bars hide at full health?**
   - Recommendation: Yes by default (`show_when_full: false`) for cleaner visuals

3. **What color scheme?**
   - Recommendation: Green (healthy), red (low health), dark gray background

4. **Should bars scale with zoom?**
   - Recommendation: Not in first version (bars are UI, not world objects)

5. **Should player have a different bar style than enemies?**
   - Recommendation: Yes, create separate HealthBar instances with different styles

## Summary

### System Overview

✅ **Does NOT require new systems** - Uses SDL2 primitives
✅ **Does NOT modify existing systems** - Pure addition
✅ **Highly extensible** - Easy to add to any entity
✅ **Minimal complexity** - ~100 lines of code for full implementation
✅ **Save compatible** - No state to persist

### Key Design Decisions

1. **Procedural rendering** over sprite-based for flexibility
2. **Style configuration** for easy customization per entity type
3. **Render in main.rs** instead of inside entity structs
4. **Health percentage parameter** for loose coupling with Stats system

### Implementation Checklist

- [ ] Create `src/ui/mod.rs` and `src/ui/health_bar.rs`
- [ ] Implement `HealthBar` and `HealthBarStyle` structs
- [ ] Add health bar rendering to main.rs render loop (after entities)
- [ ] Create default bar for player (green)
- [ ] Create enemy bar variant (purple/red)
- [ ] Test with various health values
- [ ] Add documentation and examples

**Estimated Total Time**: 2-4 hours for complete implementation with polish

## Related Documentation

### Core Systems
- **UI System Architecture**: `docs/ui-system.md` ← **Read this first** for full architectural context
  - Explains world-space HUD vs screen-space GUI
  - Defines component patterns used here
  - Covers extensibility to other UI types

### Supporting Systems
- **Stats System**: `docs/player-stats-system.md` - Provides `health.percentage()` method
- **Animation System**: `docs/animation-system.md` - Used by entities (not health bars)
- **Save System**: `docs/save-system-design.md` - Health values are saved, UI is not

### Future Components (Same System)
Once health bars are implemented, these can follow the same pattern:
- **Damage Numbers** - Floating text showing damage/healing
- **Status Icons** - Buff/debuff indicators
- **Name Tags** - Entity name labels
- **Interaction Prompts** - "Press E" text above interactables

All use the world-space HUD architecture defined in `docs/ui-system.md`.

## Next Steps

1. **Understand architecture** - Review `docs/ui-system.md` if you haven't
2. **Review this plan** - Any concerns or suggestions?
3. **Implementation** - Follow Phase 1-3 in implementation plan
4. **Testing** - Visual testing with various health values
5. **Polish** - Tweak colors, sizes, positions based on feel
6. **Foundation complete** - This validates the world-space HUD system for future components

**This is the first world-space HUD component!** Success here proves the architecture works and enables rapid development of damage numbers, status icons, etc.
