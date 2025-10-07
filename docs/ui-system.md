# UI System Architecture

## Overview
This document defines Game1's two-tier UI architecture for rendering user interface elements. The system separates **world-space HUD** elements from **screen-space GUI** elements to maintain clean separation of concerns and enable independent evolution of each subsystem.

**Status**: 🏗️ **FOUNDATION** - Architecture defined, implementation in progress

## Philosophy

Game1's UI is split into two fundamentally different categories based on **coordinate space**:

1. **World-Space HUD** - UI elements attached to game entities
2. **Screen-Space GUI** - UI elements fixed to the screen

This separation prevents architectural confusion and ensures each system can evolve independently.

## System 1: World-Space HUD

### Definition
UI elements that render **in the game world**, positioned relative to entities. These elements move with entities, use world coordinates, and render in the world rendering layer.

**Module Location**: `src/ui/`

### Characteristics
- ✅ Uses world coordinates (entity x, y)
- ✅ Moves with entities (follows player/enemy position)
- ✅ Affected by camera transformations (if camera system added)
- ✅ Renders in world layer (after entities, before screen UI)
- ✅ Lightweight (procedural rendering, minimal state)
- ✅ Created on-demand (not stored in entity structs)

### Use Cases
- Health bars above entities
- Damage/healing numbers (floating text)
- Status effect icons (buffs/debuffs above head)
- Entity name tags
- Quest markers (exclamation marks above NPCs)
- Interaction prompts ("Press E to interact")
- Casting bars (spell/ability progress)

### Anti-Patterns (What NOT to Use World-Space HUD For)
- ❌ Inventory windows
- ❌ Menus (pause, settings)
- ❌ Hotbars/action bars
- ❌ Minimaps
- ❌ Score/time displays
- ❌ Dialogue boxes

### Architecture

#### Core Pattern: Component-Based Rendering

World-space HUD elements are **stateless components** that render on-demand:

```rust
// Component struct (lightweight, reusable)
pub struct HealthBar {
    style: HealthBarStyle,  // Configuration only
}

impl HealthBar {
    /// Render above an entity (does NOT store entity reference)
    pub fn render(
        &self,
        canvas: &mut Canvas<Window>,
        entity_x: i32,        // Entity's world position
        entity_y: i32,
        entity_width: u32,    // Entity's rendered size
        entity_height: u32,
        data: f32,            // Component-specific data (e.g., health %)
    ) -> Result<(), String> {
        // Calculate position relative to entity
        let ui_x = entity_x + /* centering logic */;
        let ui_y = entity_y + self.style.offset_y;

        // Procedural rendering (SDL2 primitives)
        canvas.fill_rect(/* ... */)?;

        Ok(())
    }
}
```

**Key Design Decisions**:
1. **No entity references** - Components don't store entity pointers
2. **On-demand rendering** - Created once, reused for all entities
3. **Procedural graphics** - Uses SDL2 primitives (rectangles, lines) not sprites
4. **Style configuration** - Appearance separated from rendering logic

#### Integration Pattern

**Where to create components**: `main.rs` (or game state manager)

```rust
// In main.rs, create components once
let player_health_bar = HealthBar::new();
let enemy_health_bar = HealthBar::with_style(HealthBarStyle {
    health_color: Color::RGB(150, 0, 150),  // Purple for enemies
    ..Default::default()
});

// In render loop, call render for each entity
for slime in &slimes {
    slime.render(&mut canvas)?;  // Entity renders sprite

    // Health bar renders above entity
    enemy_health_bar.render(
        &mut canvas,
        slime.x,
        slime.y,
        slime.width * 2,   // Account for sprite scaling
        slime.height * 2,
        slime.health as f32 / slime.max_health as f32,  // Data parameter
    )?;
}
```

**Why not inside entity structs?**
- ✅ **Separation of concerns** - Entities handle logic, UI handles display
- ✅ **Flexibility** - Can render same entity with different UI styles
- ✅ **Save compatibility** - UI state doesn't need serialization
- ✅ **Memory efficiency** - No per-entity UI overhead

#### Rendering Order

World-space HUD must render in the correct layer:

```rust
// Correct render order (in main.rs game loop)
1. World background (tiles, terrain)
2. Entities (player, enemies, items)
3. Visual effects (attack effects, particles)
4. World-Space HUD (health bars, damage numbers) ← This system
5. Screen-Space GUI (menus, inventory) ← Different system
```

### Component Lifecycle

```
Creation (once in main.rs)
    ↓
Render Loop:
    For each entity:
        1. Entity renders sprite
        2. HUD component renders UI above entity
    ↓
Repeat each frame
```

**No update phase** - World-space HUD components are pure rendering, no state to update.

### Technology: Procedural Rendering

World-space HUD uses **SDL2 primitive rendering** (not sprites):

**Supported Primitives**:
- `fill_rect()` - Filled rectangles (health bars, backgrounds)
- `draw_rect()` - Outlined rectangles (borders)
- `draw_line()` - Lines (connections, indicators)
- `draw_point()` - Points (particles, minimal indicators)

**Why not sprites?**
- Dynamic content (health bar width changes with health)
- Easy customization (color changes don't require new assets)
- Minimal memory overhead
- Simple implementation

**When to use sprites?**
- Complex graphics (icons, symbols)
- Pre-designed art (status effect icons)
- Performance-critical (many instances)

**Hybrid approach** (future):
```rust
pub enum BarRenderMode {
    Procedural,  // SDL2 rectangles (current)
    Textured {   // Sprite-based (future)
        background: Texture,
        fill: Texture,
    },
}
```

### Performance Characteristics

**Rendering Cost**:
- Per component: 3-10 SDL2 draw calls
- Per frame (10 entities): ~50 draw calls
- Impact: Negligible (SDL2 primitives are hardware-accelerated)

**Memory Cost**:
- Per component instance: ~50-100 bytes (style configuration)
- Per entity: 0 bytes (no entity storage)
- Impact: Minimal

**Optimization Strategies**:
1. **Culling** - Don't render HUD for off-screen entities
2. **Batching** - Render all bars of same style together
3. **Caching** - Reuse component instances (already done)

### Extensibility Guide

#### Adding New Component Types

**Step 1**: Create component in `src/ui/`

```rust
// src/ui/damage_number.rs
pub struct DamageNumber {
    style: DamageNumberStyle,
}

impl DamageNumber {
    pub fn render(
        &self,
        canvas: &mut Canvas<Window>,
        entity_x: i32,
        entity_y: i32,
        entity_width: u32,
        entity_height: u32,
        damage_amount: i32,  // Component-specific data
    ) -> Result<(), String> {
        // Render floating damage number above entity
        // ...
    }
}
```

**Step 2**: Export from module

```rust
// src/ui/mod.rs
pub mod health_bar;
pub mod damage_number;

pub use health_bar::{HealthBar, HealthBarStyle};
pub use damage_number::{DamageNumber, DamageNumberStyle};
```

**Step 3**: Use in main.rs

```rust
let damage_numbers = DamageNumber::new();

// When entity takes damage
if damage_dealt > 0 {
    damage_numbers.render(canvas, entity.x, entity.y, /* ... */, damage_dealt)?;
}
```

#### Creating Style Variants

```rust
// Boss health bar (larger, different position)
let boss_health_bar = HealthBar::with_style(HealthBarStyle {
    width: 200,
    height: 20,
    offset_y: 50,  // Below entity instead of above
    health_color: Color::RGB(255, 165, 0),  // Orange
    ..Default::default()
});

// Ally health bar (blue)
let ally_health_bar = HealthBar::with_style(HealthBarStyle {
    health_color: Color::RGB(0, 150, 255),  // Blue
    ..Default::default()
});
```

### Current Implementations

**Implemented**:
- _(None yet - health bars are first component)_

**Planned**:
- ✅ Health bars (see `docs/features/health-bar-system.md`)
- 🔜 Damage numbers
- 🔜 Status icons
- 🔜 Name tags

---

## System 2: Screen-Space GUI

### Definition
UI elements that render **on the screen**, independent of world entities. These elements stay fixed regardless of entity movement, use screen coordinates, and render on top of everything.

**Module Location**: `src/gui/` (future)

### Characteristics
- ✅ Uses screen coordinates (pixels from left/top edge)
- ✅ Fixed position (doesn't move with world)
- ✅ Not affected by camera (always visible)
- ✅ Renders above everything (top layer)
- ✅ Stateful (windows can be open/closed, have content)
- ✅ Complex layouts (nested elements, scrolling)

### Use Cases
- Inventory windows
- Pause/settings menus
- Character stat sheets
- Hotbars/action bars
- Dialogue boxes
- Quest logs
- Minimap
- Score/timer displays
- Crafting interfaces

### Anti-Patterns (What NOT to Use Screen-Space GUI For)
- ❌ Health bars above entities
- ❌ Damage numbers
- ❌ Entity name tags
- ❌ Status effect icons above heads

### Architecture (Not Yet Implemented)

Screen-space GUI is a **future system** that will be implemented when needed (inventory, menus, etc.).

**Recommended approach**: Use a GUI library

**Option 1: egui** (Immediate Mode GUI)
```toml
egui = "0.24"
egui_sdl2_gl = "0.2"
```

**Pros**:
- ✅ Simple API (immediate mode)
- ✅ Built-in widgets (buttons, text boxes, windows)
- ✅ Automatic layout
- ✅ Active development

**Cons**:
- ❌ Requires OpenGL (SDL2 + egui bridge)
- ❌ Different rendering paradigm

**Option 2: Custom System**
```rust
// src/gui/widget.rs
pub trait Widget {
    fn render(&self, canvas: &mut Canvas<Window>) -> Result<(), String>;
    fn handle_event(&mut self, event: &Event) -> bool;
}
```

**Pros**:
- ✅ Full control
- ✅ No external dependencies
- ✅ SDL2 native

**Cons**:
- ❌ More work to implement
- ❌ Need to handle layout, events, etc.

### Placeholder Integration

```rust
// Future: src/gui/inventory.rs
pub struct InventoryWindow {
    is_open: bool,
    items: Vec<Item>,
    position: (i32, i32),
    size: (u32, u32),
}

impl InventoryWindow {
    pub fn render(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        if !self.is_open {
            return Ok(());
        }

        // Render at fixed screen position
        let window_rect = Rect::new(
            self.position.0,  // Screen X (e.g., 100)
            self.position.1,  // Screen Y (e.g., 100)
            self.size.0,
            self.size.1,
        );

        // Draw window background, items, etc.
        // ...

        Ok(())
    }

    pub fn handle_event(&mut self, event: &Event) -> bool {
        // Handle clicks, keyboard input
        // ...
    }
}
```

### Rendering Order

```rust
// Screen-space GUI renders LAST (on top of everything)
1. World background
2. Entities
3. Visual effects
4. World-Space HUD
5. Screen-Space GUI ← Renders last
```

---

## Special Case: Dropped Items

### Are Dropped Items UI?

**No.** Dropped items are **entities**, not UI.

```rust
pub struct DroppedItem<'a> {
    pub x: i32,                            // World position
    pub y: i32,
    pub item_type: String,
    animation_controller: AnimationController<'a>,  // Uses animation system!
}
```

**Why entities?**
- They exist in the game world
- They have physics (can fall, collide)
- Players walk to them to pick up
- They use the existing animation system (idle, glow effects)
- They implement `Collidable` trait
- They implement `Saveable` trait (persist in save files)

**Rendering order**: With other entities (layer 2)

**Optional enhancement**: Add world-space HUD for item labels
```rust
// Show item name when nearby (uses world-space HUD system)
item_label.render(canvas, item.x, item.y, item.width, item.height, 1.0)?;
```

---

## System Comparison

| Aspect | World-Space HUD | Screen-Space GUI | Dropped Items |
|--------|----------------|------------------|---------------|
| **Coordinate space** | World (entity) | Screen | World (entity) |
| **Movement** | Follows entity | Fixed | Independent |
| **Module** | `src/ui/` | `src/gui/` (future) | `src/` (entity) |
| **Rendering** | Procedural (SDL2) | Widget-based | Sprite-based (animation) |
| **State** | Stateless component | Stateful widget | Full entity state |
| **Created** | Once, reused | Per-window instance | Per item spawned |
| **Saved** | No | Sometimes (window state) | Yes (world object) |
| **Examples** | Health bars, damage numbers | Inventory, menus | Sword, potion, coin |

---

## Decision Tree: Which System?

```
Does it attach to an entity in the world?
├─ Yes → Does it move with the entity?
│  ├─ Yes → Does it show entity data (health, status)?
│  │  ├─ Yes → World-Space HUD (src/ui/)
│  │  └─ No → Is it a physical object?
│  │     ├─ Yes → Entity with animation (src/)
│  │     └─ No → World-Space HUD (src/ui/)
│  └─ No → Entity (src/)
└─ No → Screen-Space GUI (src/gui/)
```

### Examples

- **Health bar** → Shows entity data (health), moves with entity → World-Space HUD
- **Damage number** → Shows entity event, moves with entity → World-Space HUD
- **Dropped sword** → Physical object, independent movement → Entity
- **Inventory window** → Fixed screen position, not attached to entity → Screen-Space GUI
- **Pause menu** → Fixed screen position, not attached to entity → Screen-Space GUI
- **Quest marker** → Attaches to NPC, moves with entity → World-Space HUD

---

## Implementation Roadmap

### Phase 1: World-Space HUD Foundation ✅ In Progress
- [x] Define architecture (this document)
- [ ] Implement health bars (see `docs/features/health-bar-system.md`)
- [ ] Validate rendering order
- [ ] Test with multiple entities

**Estimated Time**: 2-4 hours

### Phase 2: World-Space HUD Expansion 🔜 Future
- [ ] Damage numbers
- [ ] Status effect icons
- [ ] Name tags
- [ ] Interaction prompts

**Estimated Time**: 1-2 hours per component

### Phase 3: Screen-Space GUI Foundation 🔮 Future
- [ ] Choose GUI approach (egui vs custom)
- [ ] Implement basic window system
- [ ] Add inventory UI
- [ ] Add pause menu

**Estimated Time**: 8-16 hours (complex system)

### Phase 4: Polish and Integration 🔮 Future
- [ ] Camera system integration (world-space HUD follows camera)
- [ ] UI theming and styling
- [ ] Localization support
- [ ] Accessibility features

---

## Rust Learning Opportunities

### World-Space HUD System Teaches:
1. **Separation of Concerns** - UI separate from entity logic
2. **Component Pattern** - Reusable, stateless components
3. **Configuration Structs** - Style structs with `Default` trait
4. **Procedural Rendering** - SDL2 primitive drawing
5. **Module Organization** - Creating cohesive modules

### Screen-Space GUI System Will Teach (Future):
1. **Trait Objects** - `Box<dyn Widget>` for polymorphic widgets
2. **Event Handling** - Mouse/keyboard input processing
3. **State Management** - Window open/closed state, focus
4. **Layout Algorithms** - Positioning child widgets
5. **External Crates** - Integrating GUI libraries like egui

---

## Best Practices

### World-Space HUD

**DO**:
- ✅ Create components once, reuse for all entities
- ✅ Pass entity data as parameters (don't store references)
- ✅ Use procedural rendering for simple graphics
- ✅ Implement `Default` trait for style structs
- ✅ Document offset conventions (negative = above, positive = below)

**DON'T**:
- ❌ Store component instances inside entity structs
- ❌ Use sprites for dynamic content (health bars, progress bars)
- ❌ Render world-space HUD in screen-space coordinates
- ❌ Couple component rendering to specific entity types

### Screen-Space GUI (Future)

**DO**:
- ✅ Use established GUI libraries when possible
- ✅ Implement event handling consistently
- ✅ Support keyboard navigation
- ✅ Test on different screen resolutions

**DON'T**:
- ❌ Mix screen-space and world-space coordinate systems
- ❌ Render GUI below world-space HUD
- ❌ Hardcode screen positions (use relative positioning)

---

## Performance Considerations

### World-Space HUD
- **Rendering Cost**: ~5 draw calls per component per entity
- **Culling**: Don't render for off-screen entities
- **Batching**: Group similar components together
- **Target**: 60 FPS with 50+ visible entities with HUD

### Screen-Space GUI (Future)
- **Rendering Cost**: Depends on complexity (10-1000+ draw calls)
- **Dirty Rectangles**: Only redraw changed areas
- **Layout Caching**: Cache layout calculations
- **Target**: 60 FPS with complex menus open

---

## Testing Strategy

### World-Space HUD

**Unit Tests**:
```rust
#[test]
fn test_health_bar_centering() {
    let bar = HealthBar::new();
    // Verify bar centers above entity
}

#[test]
fn test_style_configuration() {
    let custom_style = HealthBarStyle { /* ... */ };
    let bar = HealthBar::with_style(custom_style);
    // Verify style applied
}
```

**Visual Tests**:
1. Spawn entities at various positions
2. Set different health values (100%, 50%, 10%, 0%)
3. Move entities around
4. Verify bars follow correctly and render at right positions

### Screen-Space GUI (Future)

**Unit Tests**:
- Window positioning
- Event handling
- Layout calculations

**Integration Tests**:
- Open/close menus
- Keyboard navigation
- Mouse interaction
- Window stacking order

---

## FAQ

### Q: Why separate world-space and screen-space systems?
**A**: They have fundamentally different requirements. World-space HUD follows entities and uses world coordinates. Screen-space GUI stays fixed and uses screen coordinates. Mixing them leads to architectural confusion.

### Q: Can I use sprites for world-space HUD?
**A**: Yes, for complex graphics like icons. But for dynamic content (health bars, progress bars), procedural rendering is more flexible.

### Q: Should dropped items use world-space HUD?
**A**: No. Dropped items are entities that use the animation system. You can optionally add HUD elements *above* them (like name labels).

### Q: When should I implement screen-space GUI?
**A**: When you need inventory, menus, or other fixed UI. Start with world-space HUD first (simpler, fewer dependencies).

### Q: Can world-space HUD work with a camera system?
**A**: Yes. When you add a camera, adjust rendering:
```rust
health_bar.render(
    canvas,
    entity.x - camera.x,  // Apply camera offset
    entity.y - camera.y,
    // ...
)?;
```

### Q: Do I need to save world-space HUD state?
**A**: No. HUD components are pure rendering. The underlying data (health, status effects) is already saved in entity structs.

---

## Summary

Game1's UI architecture consists of two independent systems:

1. **World-Space HUD** (`src/ui/`)
   - Lightweight components that render above entities
   - Procedural rendering (SDL2 primitives)
   - Stateless, reusable across entities
   - Use for: health bars, damage numbers, status icons

2. **Screen-Space GUI** (`src/gui/` - future)
   - Complex widget system for menus/windows
   - Fixed screen positioning
   - Stateful, event-driven
   - Use for: inventory, menus, hotbars

**Dropped items** are entities (not UI) that use the existing animation system.

This separation ensures clean architecture, independent evolution, and clear responsibilities for each system.

---

## Related Documentation

- **Health Bar Implementation**: `docs/features/health-bar-system.md` (first world-space HUD component)
- **Animation System**: `docs/animation-system.md` (used by entities, not UI)
- **Stats System**: `docs/player-stats-system.md` (provides health data for UI)
- **Save System**: `docs/save-system-design.md` (saves entity data, not UI state)

---

**Last Updated**: January 2025
**Status**: Foundation defined, health bars in progress
**Next Steps**: Implement health bars, validate architecture with first real component
