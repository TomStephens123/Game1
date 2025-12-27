# Phase 3: System Integration Roadmap

**Date**: 2025-10-12
**Objective**: Integrate proper system designs (InputSystem, GameWorld APIs, ResourceManager) to complete the main.rs refactoring
**Status**: 🚀 Ready to Begin

---

## Overview

We've successfully completed Phases 1-2 of the main.rs refactoring:
- ✅ Phase 1: Extracted functions from main() (1,130→421 lines)
- ✅ Phase 2: Created Game struct with GameWorld, Systems, UIManager
- ✅ Phase 2.5: Added run() method, refactored main() (421→197 lines - **83% total reduction**)

**Phase 3** integrates proper system designs to move from "basic structs" to "proper abstractions with clean APIs".

---

## Current State Analysis

### What We Have (Basic Structs)

```rust
// src/main.rs:134-145
pub struct GameWorld<'a> {
    pub player: Player<'a>,
    pub slimes: Vec<Slime<'a>>,
    pub entities: Vec<TheEntity<'a>>,
    pub dropped_items: Vec<DroppedItem<'a>>,
    pub world_grid: WorldGrid,
    pub render_grid: RenderGrid,
    pub player_inventory: PlayerInventory,
    pub attack_effects: Vec<AttackEffect<'a>>,
    pub floating_texts: Vec<FloatingTextInstance>,
    pub active_attack: Option<combat::AttackEvent>,
}

// src/main.rs:148-158
pub struct Systems {
    pub player_config: AnimationConfig,
    pub slime_config: AnimationConfig,
    pub punch_config: AnimationConfig,
    pub debug_config: DebugConfig,
    pub static_objects: Vec<StaticObject>,
    pub slime_spawner: SlimeSpawner,
}

// src/main.rs:162-173
pub struct UIManager<'a> {
    pub inventory_ui: InventoryUI<'a>,
    pub save_exit_menu: SaveExitMenu,
    pub death_screen: DeathScreen,
    pub debug_menu_state: DebugMenuState,
}
```

**Problem**: These are just data containers. No methods, no encapsulation, all public fields accessed directly.

### What We Need (Proper Systems with APIs)

1. **InputSystem** - Decouple 470-line event handling in handle_events()
2. **GameWorld Methods** - Add spawn(), update(), query(), cleanup() APIs
3. **ResourceManager** - Centralize texture loading and access

---

## Implementation Priority

### Priority 1: InputSystem (Highest Impact) 🔥

**Why First:**
- Biggest pain point: 470-line event match statement in handle_events()
- Most complex refactoring: touches game state, UI, world
- Teaches most Rust concepts: enums, pattern matching, ownership

**Design File**: `agents/input-system-design.md`

**Target**: Extract event handling from Game::handle_events() into InputSystem

**Success Metrics:**
- handle_events() reduced from 479 lines → <100 lines
- Input actions defined as enum
- Different input contexts (Playing, Menu, Inventory)
- Testable without SDL2 runtime

### Priority 2: GameWorld Methods (Medium Impact) 📦

**Why Second:**
- Moderate complexity: adds methods to existing struct
- Teaches ownership: who can modify entities
- Enables cleaner code: spawn_slime() instead of manual push

**Design File**: `agents/game-world-manager-design.md`

**Target**: Add methods to GameWorld for common operations

**Success Metrics:**
- spawn_slime(), spawn_item(), spawn_entity() methods
- update() method that updates all entities
- query methods: get_collidables(), get_near()
- cleanup() method: remove dead entities

### Priority 3: ResourceManager (Lower Impact) 🎨

**Why Last:**
- Lowest urgency: current texture loading works
- Mostly refactoring: move initialization code
- Bonus features: hot-reload is nice-to-have

**Design File**: `agents/resource-manager-design.md`

**Target**: Centralize texture loading in ResourceManager

**Success Metrics:**
- Single place to load textures
- Access textures by ID: resources.get("slime")
- Reduce texture parameter passing
- Graceful error handling (missing textures)

---

## Phase 3.1: InputSystem Implementation

### Step 1: Define Input Abstractions

Create new file: `src/input_system.rs`

```rust
/// Actions the player can perform in the game
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameAction {
    // Movement (handled via keyboard state, not events)

    // Combat
    Attack,

    // UI
    OpenInventory,
    CloseInventory,
    OpenDebugMenu,
    CloseDebugMenu,
    OpenExitMenu,
    CloseExitMenu,

    // Inventory actions
    UseItem(usize),  // slot index
    DropItem(usize),

    // Debug
    SaveGame,
    LoadGame,
    ToggleCollisionBoxes,
    ToggleGridOverlay,
    SpawnSlime(i32, i32),  // x, y

    // Tile editing
    UseHoe(i32, i32),  // x, y

    // System
    Quit,
}

/// Input context determines which actions are available
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputContext {
    Playing,
    Inventory,
    ExitMenu,
    DeathScreen,
    DebugMenu,
}

/// InputSystem processes SDL2 events and produces GameActions
pub struct InputSystem {
    pub context: InputContext,
}

impl InputSystem {
    pub fn new() -> Self {
        InputSystem {
            context: InputContext::Playing,
        }
    }

    /// Process SDL2 events and return list of actions to handle
    pub fn poll_events(
        &mut self,
        event_pump: &mut EventPump,
        ui_state: &UIState,
    ) -> Result<Vec<GameAction>, String> {
        let mut actions = Vec::new();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => actions.push(GameAction::Quit),
                Event::KeyDown { keycode: Some(key), .. } => {
                    self.handle_keydown(key, ui_state, &mut actions)?;
                }
                Event::MouseButtonDown { x, y, .. } => {
                    self.handle_click(x, y, ui_state, &mut actions)?;
                }
                _ => {}
            }
        }

        Ok(actions)
    }

    fn handle_keydown(
        &self,
        key: Keycode,
        ui_state: &UIState,
        actions: &mut Vec<GameAction>,
    ) -> Result<(), String> {
        // Context-specific input handling
        match self.context {
            InputContext::Playing => {
                // Gameplay keys
                match key {
                    Keycode::M => actions.push(GameAction::Attack),
                    Keycode::I => actions.push(GameAction::OpenInventory),
                    Keycode::Escape => actions.push(GameAction::OpenExitMenu),
                    Keycode::F3 => actions.push(GameAction::OpenDebugMenu),
                    Keycode::F5 => actions.push(GameAction::SaveGame),
                    Keycode::F9 => actions.push(GameAction::LoadGame),
                    Keycode::B => actions.push(GameAction::ToggleCollisionBoxes),
                    Keycode::G => actions.push(GameAction::ToggleGridOverlay),
                    _ => {}
                }
            }
            InputContext::Inventory => {
                // Inventory keys
                match key {
                    Keycode::I | Keycode::Escape => {
                        actions.push(GameAction::CloseInventory);
                    }
                    Keycode::Num1 => actions.push(GameAction::UseItem(0)),
                    Keycode::Num2 => actions.push(GameAction::UseItem(1)),
                    // ... more slots
                    _ => {}
                }
            }
            InputContext::ExitMenu => {
                // Exit menu navigation handled by UI
            }
            InputContext::DeathScreen => {
                // Death screen handled by UI
            }
            InputContext::DebugMenu => {
                match key {
                    Keycode::F3 | Keycode::Escape => {
                        actions.push(GameAction::CloseDebugMenu);
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    fn handle_click(
        &self,
        x: i32,
        y: i32,
        ui_state: &UIState,
        actions: &mut Vec<GameAction>,
    ) -> Result<(), String> {
        // Click handling based on context
        match self.context {
            InputContext::Playing => {
                // Right-click to spawn slime (debug feature)
                // Left-click to use hoe tool if equipped
                // ... etc
            }
            _ => {
                // UI handles its own clicks
            }
        }

        Ok(())
    }
}

/// Helper struct to pass UI state without full Game borrow
pub struct UIState {
    pub inventory_open: bool,
    pub debug_menu_open: bool,
    pub exit_menu_open: bool,
    pub death_screen_active: bool,
}
```

### Step 2: Integrate InputSystem into Game

Modify `src/main.rs`:

```rust
pub struct Game<'a> {
    pub world: GameWorld<'a>,
    pub systems: Systems,
    pub ui: UIManager<'a>,
    pub game_state: GameState,
    pub canvas: sdl2::render::Canvas<sdl2::video::Window>,
    pub event_pump: sdl2::EventPump,
    pub texture_creator: &'a sdl2::render::TextureCreator<sdl2::video::WindowContext>,
    pub textures: GameTextures<'a>,
    pub item_registry: ItemRegistry,
    pub save_manager: SaveManager,
    pub input_system: InputSystem,  // NEW
}
```

### Step 3: Refactor handle_events() to use InputSystem

Replace 470-line match statement with action dispatch:

```rust
impl<'a> Game<'a> {
    pub fn handle_events(&mut self) -> Result<bool, String> {
        // Get UI state for input system
        let ui_state = UIState {
            inventory_open: self.ui.inventory_ui.is_open,
            debug_menu_open: matches!(self.ui.debug_menu_state, DebugMenuState::Open { .. }),
            exit_menu_open: self.game_state == GameState::ExitMenu,
            death_screen_active: self.game_state == GameState::Dead,
        };

        // Update input context
        self.input_system.context = if self.ui.inventory_ui.is_open {
            InputContext::Inventory
        } else if self.game_state == GameState::ExitMenu {
            InputContext::ExitMenu
        } else if self.game_state == GameState::Dead {
            InputContext::DeathScreen
        } else {
            InputContext::Playing
        };

        // Poll events and get actions
        let actions = self.input_system.poll_events(&mut self.event_pump, &ui_state)?;

        // Handle actions
        for action in actions {
            if self.handle_action(action)? {
                return Ok(true);  // Quit
            }
        }

        Ok(false)
    }

    fn handle_action(&mut self, action: GameAction) -> Result<bool, String> {
        match action {
            GameAction::Quit => return Ok(true),

            GameAction::Attack => {
                // Attack logic
            }

            GameAction::OpenInventory => {
                self.ui.inventory_ui.is_open = true;
            }

            GameAction::CloseInventory => {
                self.ui.inventory_ui.is_open = false;
            }

            // ... other actions

            _ => {}
        }

        Ok(false)
    }
}
```

### Step 4: Testing Strategy

Create `src/input_system.rs` with tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_switches() {
        let mut input = InputSystem::new();
        assert_eq!(input.context, InputContext::Playing);

        input.context = InputContext::Inventory;
        assert_eq!(input.context, InputContext::Inventory);
    }

    // Mock SDL2 events for testing
    // Test action generation
    // Test context-specific input
}
```

**Migration Plan:**
1. Create `src/input_system.rs` with structs and empty methods
2. Add `mod input_system;` to `src/main.rs`
3. Add `input_system: InputSystem::new()` to Game construction
4. Extract event handling logic incrementally (one action type at a time)
5. Test after each extraction
6. Remove old event handling code once all actions work

---

## Phase 3.2: GameWorld Methods

### Step 1: Add Spawning Methods

Add to `src/main.rs` GameWorld impl:

```rust
impl<'a> GameWorld<'a> {
    /// Spawn a new slime at the given position
    pub fn spawn_slime(
        &mut self,
        x: i32,
        y: i32,
        texture: &'a sdl2::render::Texture<'a>,
        config: &AnimationConfig,
    ) {
        let slime = Slime::new(
            x, y, texture, config,
            self.slimes.len() as u32 + 1  // Simple ID
        );
        self.slimes.push(slime);
        println!("Slime spawned at ({}, {})", x, y);
    }

    /// Spawn a dropped item
    pub fn spawn_dropped_item(
        &mut self,
        item_id: &str,
        x: i32,
        y: i32,
        texture: &'a sdl2::render::Texture<'a>,
    ) {
        let item = DroppedItem::new(item_id, x, y, texture);
        self.dropped_items.push(item);
    }

    /// Spawn attack effect
    pub fn spawn_attack_effect(
        &mut self,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        texture: &'a sdl2::render::Texture<'a>,
        config: &AnimationConfig,
    ) {
        let effect = AttackEffect::new(x, y, width, height, texture, config);
        self.attack_effects.push(effect);
    }

    /// Spawn floating text
    pub fn spawn_floating_text(
        &mut self,
        text: String,
        x: i32,
        y: i32,
        style: FloatingTextStyle,
    ) {
        let float_text = FloatingTextInstance::new(text, x, y, style);
        self.floating_texts.push(float_text);
    }
}
```

### Step 2: Add Update Methods

```rust
impl<'a> GameWorld<'a> {
    /// Update all entities in the world
    pub fn update(&mut self, delta_time: f32) {
        // Update slimes
        for slime in &mut self.slimes {
            slime.update(&self.player);
        }

        // Update entities
        for entity in &mut self.entities {
            entity.update();
        }

        // Update effects
        for effect in &mut self.attack_effects {
            effect.update();
        }

        // Update floating texts
        for text in &mut self.floating_texts {
            text.update(delta_time);
        }
    }

    /// Remove dead/finished entities
    pub fn cleanup(&mut self) {
        self.slimes.retain(|s| s.is_alive);
        self.attack_effects.retain(|e| !e.is_finished());
        self.floating_texts.retain(|t| !t.is_expired());
    }
}
```

### Step 3: Add Query Methods

```rust
impl<'a> GameWorld<'a> {
    /// Get all collidable objects (for collision detection)
    pub fn get_all_collidables(&self) -> Vec<&dyn collision::Collidable> {
        let mut collidables: Vec<&dyn collision::Collidable> = Vec::new();

        collidables.push(&self.player as &dyn collision::Collidable);

        for slime in &self.slimes {
            collidables.push(slime as &dyn collision::Collidable);
        }

        for entity in &self.entities {
            collidables.push(entity as &dyn collision::Collidable);
        }

        collidables
    }

    /// Get mutable reference to player
    pub fn get_player_mut(&mut self) -> &mut Player<'a> {
        &mut self.player
    }

    /// Get player position
    pub fn get_player_pos(&self) -> (i32, i32) {
        (self.player.x, self.player.y)
    }

    /// Check if position is valid for spawning
    pub fn is_position_valid(&self, x: i32, y: i32) -> bool {
        // Check bounds
        if x < 0 || y < 0 {
            return false;
        }

        // Check collision with terrain
        // ... implementation

        true
    }
}
```

---

## Phase 3.3: ResourceManager (Optional)

**Note**: This is lower priority. Only implement if time allows or if texture management becomes a problem.

### Defer to Later

ResourceManager is a **nice-to-have** but not critical for Phase 3. Current texture loading works fine. Consider implementing later if:
- Adding many new asset types
- Need hot-reload for development
- Texture loading becomes performance bottleneck

---

## Success Criteria

Phase 3 is complete when:

- ✅ InputSystem extracts event handling (handle_events <100 lines)
- ✅ GameAction enum defines all possible actions
- ✅ Input contexts separate Playing/Inventory/Menu logic
- ✅ GameWorld has spawn methods (spawn_slime, spawn_item, etc.)
- ✅ GameWorld has update() and cleanup() methods
- ✅ GameWorld has query methods (get_collidables, etc.)
- ✅ All 39 tests still pass
- ✅ Zero compilation warnings
- ✅ Game plays identically to before refactoring

---

## Risk Assessment

### High Risk: InputSystem

**Risk**: Breaking existing input handling (attacks, inventory, debug keys)
**Mitigation**: Incremental extraction, test each action type
**Rollback Plan**: Keep old handle_events() commented out until fully validated

### Medium Risk: GameWorld Methods

**Risk**: Lifetime/borrowing issues with methods
**Mitigation**: Start with simple methods (spawn), add complex later
**Rollback Plan**: Methods can be added without breaking existing code

### Low Risk: ResourceManager

**Risk**: Minimal (we're deferring this)
**Mitigation**: N/A
**Rollback Plan**: N/A

---

## Timeline Estimate

Based on complexity and integration points:

- **Phase 3.1: InputSystem** - 3-4 hours
  - Define structs/enums: 30min
  - Extract event handling: 2 hours
  - Testing and validation: 1 hour
  - Documentation: 30min

- **Phase 3.2: GameWorld Methods** - 1-2 hours
  - Spawning methods: 30min
  - Update/cleanup methods: 30min
  - Query methods: 30min
  - Testing: 30min

**Total: 4-6 hours of focused work**

---

## Next Steps

1. ✅ Read this roadmap
2. Begin Phase 3.1: InputSystem implementation
3. Create `src/input_system.rs`
4. Define GameAction enum
5. Extract first action (Quit) as proof of concept
6. Incrementally extract remaining actions
7. Test and validate
8. Move to Phase 3.2

Let's start with InputSystem! 🚀
