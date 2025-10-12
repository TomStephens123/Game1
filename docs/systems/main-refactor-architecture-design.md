# Main.rs Refactoring Architecture Design

**Status:** Design Phase
**Created:** 2025-01-12
**Purpose:** Architecture to reduce main.rs from 1,130 lines to ~150-200 lines with clear system separation

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Architecture Overview](#architecture-overview)
3. [Core Structs and Types](#core-structs-and-types)
4. [Game Loop Design](#game-loop-design)
5. [System Integration](#system-integration)
6. [Code Examples (Before/After)](#code-examples-beforeafter)
7. [Migration Strategy](#migration-strategy)
8. [Error Handling Strategy](#error-handling-strategy)
9. [Rust Patterns Explained](#rust-patterns-explained)
10. [Answers to Key Questions](#answers-to-key-questions)
11. [Trade-offs and Future Work](#trade-offs-and-future-work)

---

## Executive Summary

### The Problem
Current `main.rs` is **1,130 lines** (567-1697) with:
- 470-line event handling match statement
- 15+ mutable variables in main loop
- Mixed concerns (input/update/collision/render all interleaved)
- Impossible to test, understand, or modify safely

### The Solution
**Phased approach using "Approach 3: Phased Update Functions" with eventual migration to "Approach 1: Game Struct"**

**Why this hybrid approach:**
1. **Phase 1 (Low Risk):** Extract functions from main() - no structural changes
2. **Phase 2 (Medium Risk):** Group related data into structs (GameWorld, Systems)
3. **Phase 3 (Higher Risk):** Create Game struct to own everything
4. **Phase 4 (Polish):** Integrate specialized systems (Input, Events, Physics)

This allows **incremental migration** while maintaining functionality at each step.

### Key Decisions
- ✅ Start with free functions, migrate to Game struct later
- ✅ GameWorld owns all entities (player, slimes, items, etc.)
- ✅ Systems struct groups related functionality
- ✅ Fixed timestep (60 FPS) with frame limiting
- ✅ Save/load becomes methods on Game
- ✅ UI systems separate from GameWorld (UIManager)
- ❌ No trait-based System interface (YAGNI - premature abstraction)
- ❌ No complex state machine yet (current GameState enum is sufficient)

---

## Architecture Overview

### High-Level Structure

```
┌─────────────────────────────────────────────┐
│              main()                         │
│  - SDL2 initialization                      │
│  - Create Game struct                       │
│  - Run game loop                            │
└─────────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────────┐
│              Game<'a>                       │
│  - Owns all systems and world state         │
│  - Coordinates game loop phases             │
│  - Manages state transitions                │
└─────────────────────────────────────────────┘
         │         │         │         │
         ▼         ▼         ▼         ▼
    ┌───────┐ ┌───────┐ ┌───────┐ ┌───────┐
    │ Game  │ │Systems│ │  UI   │ │ SDL2  │
    │ World │ │       │ │Manager│ │Context│
    └───────┘ └───────┘ └───────┘ └───────┘
         │
         ├─── Player
         ├─── Slimes (Vec)
         ├─── Entities (Vec)
         ├─── DroppedItems (Vec)
         ├─── WorldGrid/RenderGrid
         └─── Visual Effects
```

### Module Organization

**Existing modules stay the same**, add new coordination layer:

```
src/
├── main.rs                 (150-200 lines - orchestration only)
├── game.rs                 (NEW - Game struct and game loop)
├── game_world.rs          (NEW - GameWorld struct, entity management)
├── systems.rs             (NEW - Systems struct grouping helpers)
├── animation.rs           (existing)
├── collision.rs           (existing)
├── player.rs              (existing)
├── slime.rs               (existing)
└── ... (other existing modules)
```

### Ownership Model

```rust
main()
  └─ owns Game<'a>
       ├─ owns Canvas<Window>           (SDL2 rendering)
       ├─ owns EventPump                (SDL2 input)
       ├─ owns TextureCreator<'a>       (SDL2 textures)
       ├─ owns GameWorld<'a>            (all entities, world state)
       │    ├─ owns Player<'a>
       │    ├─ owns Vec<Slime<'a>>
       │    ├─ owns Vec<TheEntity<'a>>
       │    ├─ owns Vec<DroppedItem<'a>>
       │    ├─ owns Vec<AttackEffect<'a>>
       │    ├─ owns Vec<FloatingTextInstance>
       │    ├─ owns WorldGrid
       │    ├─ owns RenderGrid
       │    └─ owns PlayerInventory
       ├─ owns Systems                  (helpers, configs, managers)
       │    ├─ owns AnimationConfig (player, slime, punch)
       │    ├─ owns ItemRegistry
       │    ├─ owns SaveManager
       │    └─ owns StaticObjects (walls)
       ├─ owns UIManager                (UI state, not game state)
       │    ├─ owns SaveExitMenu
       │    ├─ owns DeathScreen
       │    ├─ owns InventoryUI
       │    ├─ owns HealthBar (player, enemy)
       │    ├─ owns FloatingText renderer
       │    └─ owns BuffDisplay
       ├─ owns DebugState              (debug menu, flags)
       └─ game_state: GameState        (Playing/ExitMenu/Dead)
```

**Key Insight:** Textures (`&'a Texture`) borrow from `TextureCreator`, which lives in `Game`. Lifetime `'a` propagates to all structs holding texture references.

### Data Flow

```
Frame N:
  1. INPUT:     event_pump → handle_input() → actions
  2. UPDATE:    actions + world → update_world() → modified world
  3. COLLISION: world → resolve_collisions() → pushed entities
  4. RENDER:    world → render_game() → pixels on screen
  5. PRESENT:   canvas.present()
  6. TIMING:    sleep until frame end
```

**Sequential phases prevent borrow conflicts:**
- Input mutates nothing (just reads events, produces actions)
- Update mutates world (has exclusive &mut access)
- Collision mutates world positions (has exclusive &mut access)
- Render only reads world (has &World access)

---

## Core Structs and Types

### 1. Game Struct (Primary Orchestrator)

```rust
/// Main game structure owning all systems and state
///
/// Lifetime 'a tied to TextureCreator (SDL2 requirement)
pub struct Game<'a> {
    // === SDL2 Core ===
    canvas: Canvas<Window>,
    event_pump: EventPump,
    texture_creator: &'a TextureCreator<WindowContext>,

    // === Game State ===
    world: GameWorld<'a>,
    systems: Systems,
    ui: UIManager<'a>,
    debug: DebugState,
    game_state: GameState,

    // === Timing ===
    frame_duration: Duration,
    last_frame_time: Instant,
}

impl<'a> Game<'a> {
    /// Create new game instance
    pub fn new(
        canvas: Canvas<Window>,
        event_pump: EventPump,
        texture_creator: &'a TextureCreator<WindowContext>,
    ) -> Result<Self, String> {
        // Load configs, create world, initialize systems
    }

    /// Main game loop (called from main())
    pub fn run(&mut self) -> Result<(), String> {
        'running: loop {
            // 1. Input
            if self.handle_input()? {
                break 'running; // Quit requested
            }

            // 2. Update (only if playing)
            if self.game_state == GameState::Playing {
                self.update()?;
            }

            // 3. Render (always, even in menus)
            self.render()?;

            // 4. Frame timing
            self.limit_frame_rate();
        }
        Ok(())
    }

    /// Handle input events and state transitions
    fn handle_input(&mut self) -> Result<bool, String> {
        // Returns true if quit requested
        // Handles: keyboard, mouse, window events
        // Updates: game_state transitions
    }

    /// Update game world (only when Playing)
    fn update(&mut self) -> Result<(), String> {
        // Player update, enemy AI, physics, collisions
    }

    /// Render everything
    fn render(&mut self) -> Result<(), String> {
        // World rendering, UI overlay, debug visualization
    }

    /// Save game to disk
    pub fn save(&mut self) -> Result<(), String> {
        self.systems.save_manager.save_game(
            &self.world.player,
            &self.world.slimes,
            &self.world.grid,
            &self.world.entities,
            &self.world.inventory,
            &self.world.dropped_items,
        )
    }

    /// Load game from disk
    pub fn load(&mut self) -> Result<(), String> {
        let (player, slimes, grid, entities, inventory, dropped_items) =
            self.systems.save_manager.load_game(
                &self.systems.player_config,
                &self.systems.slime_config,
                self.texture_creator,
                &self.systems.item_textures,
            )?;

        self.world = GameWorld::from_loaded(
            player, slimes, grid, entities, inventory, dropped_items
        );
        Ok(())
    }
}
```

### 2. GameWorld Struct (All Entities)

```rust
/// Contains all game entities and world state
///
/// This is the "game state" that gets saved/loaded
pub struct GameWorld<'a> {
    // === Core Entities ===
    pub player: Player<'a>,
    pub slimes: Vec<Slime<'a>>,
    pub entities: Vec<TheEntity<'a>>,         // Pyramids
    pub dropped_items: Vec<DroppedItem<'a>>,

    // === World Data ===
    pub grid: WorldGrid,
    pub render_grid: RenderGrid,
    pub inventory: PlayerInventory,

    // === Visual Effects (transient, not saved) ===
    pub attack_effects: Vec<AttackEffect<'a>>,
    pub floating_texts: Vec<FloatingTextInstance>,

    // === Gameplay State ===
    pub active_attack: Option<combat::AttackEvent>,
    pub has_regen: bool,
    pub regen_timer: Instant,
}

impl<'a> GameWorld<'a> {
    /// Create new world with default entities
    pub fn new(/* textures, configs */) -> Result<Self, String> {
        // Spawn player at center
        // Create default world grid
        // Spawn initial entities (pyramids)
    }

    /// Create world from loaded save data
    pub fn from_loaded(
        player: Player<'a>,
        slimes: Vec<Slime<'a>>,
        grid: WorldGrid,
        entities: Vec<TheEntity<'a>>,
        inventory: PlayerInventory,
        dropped_items: Vec<DroppedItem<'a>>,
    ) -> Self {
        // Reconstruct world from save data
    }

    /// Update all entities (called from Game::update)
    pub fn update(&mut self, delta_time: f32) {
        // Update player, slimes, entities, effects
        // Apply buffs from pyramids
        // Handle regeneration
        // Cleanup dead entities
    }

    /// Get all collidable entities (for collision system)
    pub fn get_collidables(&self) -> Vec<&dyn Collidable> {
        // Return player, slimes, entities
    }

    /// Get all collidables mutably (for push separation)
    pub fn get_collidables_mut(&mut self) -> Vec<&mut dyn Collidable> {
        // Mutable version for collision response
    }
}
```

### 3. Systems Struct (Helper Systems)

```rust
/// Container for helper systems and configurations
///
/// These are systems that don't fit cleanly into GameWorld
/// but are needed for game operation
pub struct Systems {
    // === Animation Configs ===
    pub player_config: AnimationConfig,
    pub slime_config: AnimationConfig,
    pub punch_config: AnimationConfig,

    // === Item System ===
    pub item_registry: ItemRegistry,
    pub item_textures: HashMap<String, Texture<'a>>,

    // === Save/Load ===
    pub save_manager: SaveManager,

    // === Collision ===
    pub static_objects: Vec<StaticObject>,  // Walls

    // === Debug ===
    pub debug_config: DebugConfig,
}

impl Systems {
    pub fn new(texture_creator: &TextureCreator) -> Result<Self, String> {
        // Load all configs
        // Initialize item registry and textures
        // Create save manager
        // Setup static collision boundaries
    }
}
```

### 4. UIManager Struct (UI State)

```rust
/// Manages UI state and rendering
///
/// Separate from GameWorld because UI state isn't "game state"
/// (doesn't affect gameplay, not saved)
pub struct UIManager<'a> {
    // === Menus ===
    pub save_exit_menu: SaveExitMenu,
    pub death_screen: DeathScreen,
    pub inventory_ui: InventoryUI<'a>,

    // === HUD Elements ===
    pub player_health_bar: HealthBar,
    pub enemy_health_bar: HealthBar,
    pub floating_text_renderer: FloatingText,
    pub buff_display: BuffDisplay<'a>,

    // === Input State ===
    pub mouse_x: i32,
    pub mouse_y: i32,
    pub is_tilling: bool,
    pub last_tilled_tile: Option<(i32, i32)>,
}

impl<'a> UIManager<'a> {
    pub fn new(texture_creator: &'a TextureCreator, item_textures: &'a HashMap<String, Texture<'a>>, item_registry: &ItemRegistry) -> Result<Self, String> {
        // Initialize all UI components
    }

    pub fn render_hud(
        &self,
        canvas: &mut Canvas<Window>,
        world: &GameWorld,
        game_state: GameState,
    ) -> Result<(), String> {
        // Render health bars, buffs, floating text
        // Render hotbar/inventory if open
    }

    pub fn render_menu(
        &self,
        canvas: &mut Canvas<Window>,
        game_state: GameState,
    ) -> Result<(), String> {
        // Render save/exit menu or death screen
    }
}
```

### 5. DebugState Struct (Debug Features)

```rust
/// Debug state and configuration
///
/// In release builds, could be compiled out with cfg(debug_assertions)
pub struct DebugState {
    pub menu_state: DebugMenuState,
    pub show_collision_boxes: bool,
    pub show_tile_grid: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DebugMenuState {
    Closed,
    Open { selected_index: usize },
}
```

### Lifetime Strategy

**Lifetime `'a`** tied to `TextureCreator`:
- `TextureCreator` lives in `main()` (created from canvas)
- All `Texture<'a>` references borrow from it
- All entities holding textures have lifetime `'a`
- This propagates: `GameWorld<'a>`, `Game<'a>`, etc.

**Why this works:**
- `TextureCreator` outlives everything (owned by main)
- No circular references (one-way ownership tree)
- Clear that textures must outlive entities

---

## Game Loop Design

### Initialization Phase

```rust
fn main() -> Result<(), String> {
    // === SDL2 Setup (~20 lines) ===
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let _image_context = sdl2::image::init(InitFlag::PNG)?;

    let window_scale = calculate_window_scale(&video_subsystem);
    let window = video_subsystem
        .window("Game 1", GAME_WIDTH * window_scale, GAME_HEIGHT * window_scale)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    canvas.set_logical_size(GAME_WIDTH, GAME_HEIGHT)?;

    let texture_creator = canvas.texture_creator();
    let event_pump = sdl_context.event_pump()?;

    // === Game Creation (~10 lines) ===
    let mut game = Game::new(canvas, event_pump, &texture_creator)?;

    // Try to load existing save, or create new world
    if let Err(_) = game.load() {
        println!("No save found, starting new game");
    }

    // === Run Game Loop (~5 lines) ===
    game.run()?;

    Ok(())
}
```

**Total: ~35 lines**

### Main Loop Phases

```rust
impl<'a> Game<'a> {
    pub fn run(&mut self) -> Result<(), String> {
        'running: loop {
            // === PHASE 1: INPUT ===
            // Process SDL2 events, handle state transitions
            // Returns true if quit requested
            if self.handle_input()? {
                break 'running;
            }

            // === PHASE 2: UPDATE ===
            // Only update gameplay if Playing (not in menu/dead)
            if self.game_state == GameState::Playing
               && !self.ui.inventory_ui.is_open
               && self.debug.menu_state == DebugMenuState::Closed
            {
                self.update()?;
            }

            // === PHASE 3: STATE UPDATES ===
            // Update menus/death screens regardless of game state
            if self.game_state == GameState::Dead {
                if self.ui.death_screen.should_respawn() {
                    self.world.player.respawn(
                        GAME_WIDTH as i32 / 2,
                        GAME_HEIGHT as i32 / 2
                    );
                    self.ui.death_screen.reset();
                    self.game_state = GameState::Playing;
                }
            }

            // === PHASE 4: RENDER ===
            // Always render (menus, game, everything)
            self.render()?;

            // === PHASE 5: FRAME TIMING ===
            self.limit_frame_rate();
        }

        Ok(())
    }
}
```

### Frame Timing (Fixed Timestep)

```rust
impl<'a> Game<'a> {
    fn limit_frame_rate(&mut self) {
        let frame_time = self.last_frame_time.elapsed();
        if frame_time < self.frame_duration {
            std::thread::sleep(self.frame_duration - frame_time);
        }
        self.last_frame_time = Instant::now();
    }
}
```

**Current approach:** Fixed 60 FPS (16.6ms per frame)
- Simple, deterministic physics
- Works well for 2D games
- No need for complex timestep accumulator yet

**Future improvement (if needed):**
- Variable delta time with maximum cap
- Timestep accumulator for consistent physics
- Only add if frame drops become an issue

### State Transitions

```rust
// Current GameState enum (keep as-is)
#[derive(Debug, Clone, PartialEq)]
pub enum GameState {
    Playing,    // Normal gameplay
    ExitMenu,   // Save/Exit menu open
    Dead,       // Death screen showing
}
```

**State transition logic:**

```rust
// In handle_input():
match event {
    Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
        match self.game_state {
            GameState::Playing => {
                // Close other UIs first
                if self.ui.inventory_ui.is_open {
                    self.ui.inventory_ui.is_open = false;
                } else if self.debug.menu_state != DebugMenuState::Closed {
                    self.debug.menu_state = DebugMenuState::Closed;
                } else {
                    self.game_state = GameState::ExitMenu;
                }
            }
            GameState::ExitMenu => {
                self.game_state = GameState::Playing;
            }
            GameState::Dead => {
                self.game_state = GameState::ExitMenu;
                self.ui.death_screen.reset();
            }
        }
    }
    // ...
}
```

**No complex state machine needed yet.** Current enum is sufficient. Future consideration: If adding pause menu, level transitions, etc., consider state stack pattern.

---

## System Integration

### InputSystem Integration (Future Phase)

**Current phase:** Input handling stays in `Game::handle_input()` method.

**Future phase (after Game struct stabilizes):**
```rust
pub struct InputSystem {
    input_mapper: InputMapper,
}

pub enum GameAction {
    Move(Direction),
    Attack,
    OpenInventory,
    QuickSave,
    QuickLoad,
    ToggleDebug,
    Quit,
    // ...
}

impl InputSystem {
    pub fn process_events(
        &mut self,
        event_pump: &mut EventPump,
        context: InputContext,
    ) -> Vec<GameAction> {
        // Returns list of actions this frame
    }
}

// In Game::handle_input():
let actions = self.input_system.process_events(&mut self.event_pump, context);
for action in actions {
    self.handle_action(action)?;
}
```

**Migration benefit:** 470-line match becomes ~50-line action handler.

### GameWorld Integration (Current Phase)

**Already designed above.** GameWorld owns all entities and provides:

```rust
impl<'a> GameWorld<'a> {
    // Update all entities
    pub fn update(&mut self, delta_time: f32);

    // Query collidables
    pub fn get_collidables(&self) -> Vec<&dyn Collidable>;
    pub fn get_collidables_mut(&mut self) -> Vec<&mut dyn Collidable>;

    // Spawn entities
    pub fn spawn_slime(&mut self, x: i32, y: i32, /* config, texture */);
    pub fn spawn_dropped_item(&mut self, x: i32, y: i32, item_id: String, quantity: u32);

    // Cleanup
    pub fn cleanup_dead_entities(&mut self);
}
```

### ResourceManager Integration (Future Phase)

**Current phase:** Textures in `Systems` struct as `HashMap<String, Texture>`.

**Future phase:**
```rust
pub struct ResourceManager<'a> {
    texture_creator: &'a TextureCreator<WindowContext>,
    textures: HashMap<String, Texture<'a>>,
    animation_configs: HashMap<String, AnimationConfig>,
}

impl<'a> ResourceManager<'a> {
    pub fn get_texture(&self, name: &str) -> Result<&Texture<'a>, String>;
    pub fn get_animation_config(&self, name: &str) -> Result<&AnimationConfig, String>;

    // Pre-built animation controllers
    pub fn create_player_controller(&self) -> Result<AnimationController<'a>, String>;
    pub fn create_slime_controller(&self) -> Result<AnimationController<'a>, String>;
}
```

**Migration benefit:** Eliminates repetitive texture loading code.

### PhysicsSystem Integration (Future Phase)

**Current phase:** Collision response in `Game::resolve_collisions()` method.

**Future phase:**
```rust
pub struct PhysicsConfig {
    pub player_mass: f32,
    pub slime_mass: f32,
    pub friction: f32,
    pub push_force_multiplier: f32,
}

pub struct PhysicsSystem {
    config: PhysicsConfig,
}

impl PhysicsSystem {
    pub fn resolve_collision(
        &self,
        entity1: &mut dyn Collidable,
        entity2: &mut dyn Collidable,
        overlap: (i32, i32),
    ) {
        // Mass-based push separation
        let total_mass = entity1.mass() + entity2.mass();
        let ratio1 = entity2.mass() / total_mass;
        let ratio2 = entity1.mass() / total_mass;

        entity1.apply_push(-overlap.0 * ratio1, -overlap.1 * ratio1);
        entity2.apply_push(overlap.0 * ratio2, overlap.1 * ratio2);
    }
}
```

**Migration benefit:** Replaces hardcoded 3/10, 7/10 ratios with mass-based physics.

### EventSystem Integration (Future Phase)

**Current phase:** Direct function calls (entity dies → spawn item in same code).

**Future phase:**
```rust
pub enum GameEvent {
    EntityDied { entity_id: usize, entity_type: EntityType, position: (i32, i32) },
    DamageDealt { amount: i32, target_id: usize },
    ItemPickedUp { item_id: String, quantity: u32 },
    // ...
}

pub struct EventQueue {
    events: Vec<GameEvent>,
}

impl EventQueue {
    pub fn emit(&mut self, event: GameEvent) {
        self.events.push(event);
    }

    pub fn process<F>(&mut self, mut handler: F)
    where F: FnMut(&GameEvent) {
        for event in &self.events {
            handler(event);
        }
        self.events.clear();
    }
}

// In Game::update():
self.event_queue.process(|event| {
    match event {
        GameEvent::EntityDied { position, .. } => {
            self.world.spawn_dropped_item(*position, "slime_ball", 1);
        }
        // ...
    }
});
```

**Migration benefit:** Decouples systems, enables logging/achievements/analytics.

---

## Code Examples (Before/After)

### Before: Current main() Structure

```rust
fn main() -> Result<(), String> {
    // SDL2 init (20 lines)
    let sdl_context = sdl2::init()?;
    // ...

    // Texture loading (30 lines)
    let character_texture = load_texture(&texture_creator, "...")?;
    let slime_texture = load_texture(&texture_creator, "...")?;
    // ... 10 more textures

    // Entity initialization (40 lines)
    let mut player = Player::new(...);
    let mut slimes = Vec::new();
    let mut entities = Vec::new();
    // ... 15 more variables

    // UI/debug initialization (65 lines)
    let mut save_exit_menu = SaveExitMenu::new();
    let mut debug_menu_state = DebugMenuState::Closed;
    // ... 10 more UI components

    // GAME LOOP (966 lines!!!)
    'running: loop {
        // Event handling (470 lines!)
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    match game_state {
                        GameState::Playing => {
                            if inventory_ui.is_open {
                                // ...
                            } else if matches!(debug_menu_state, ...) {
                                // ...
                            } else {
                                // ...
                            }
                        }
                        // ... 400 more lines
                    }
                }
                // ... 50 more event types
            }
        }

        // Player update (10 lines)
        let keyboard_state = event_pump.keyboard_state();
        if game_state == GameState::Playing && !is_ui_active {
            player.update(&keyboard_state);
        }

        // Combat resolution (40 lines)
        if let Some(ref attack) = active_attack {
            for slime in &mut slimes {
                if collision::aabb_intersect(...) {
                    slime.take_damage(...);
                }
            }
            // ... 30 more lines
        }

        // ... 800 more lines of game loop
    }

    Ok(())
}
```

**Problems:**
- 1,130 lines, impossible to navigate
- 15+ mutable variables, all accessible everywhere
- Mixed concerns (input handling next to rendering)
- Can't test individual phases

### After: New main() Structure (Target)

```rust
fn main() -> Result<(), String> {
    // === SDL2 Setup (20 lines) ===
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let _image_context = sdl2::image::init(InitFlag::PNG)?;

    let window_scale = calculate_window_scale(&video_subsystem);
    let window = video_subsystem
        .window("Game 1", GAME_WIDTH * window_scale, GAME_HEIGHT * window_scale)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    canvas.set_logical_size(GAME_WIDTH, GAME_HEIGHT)?;

    let texture_creator = canvas.texture_creator();
    let event_pump = sdl_context.event_pump()?;

    println!("Controls:");
    println!("WASD - Move player");
    println!("M Key - Attack");
    println!("F3 - Debug Menu");
    println!("F5 - Quick Save");
    println!("F9 - Load Game");
    println!("ESC - Menu");

    // === Game Creation (10 lines) ===
    let mut game = Game::new(canvas, event_pump, &texture_creator)?;

    if let Err(_) = game.load() {
        println!("No save found, starting new game");
    }

    // === Run Game (1 line!) ===
    game.run()
}
```

**Total: ~35 lines** (vs 1,130 before)

### Example: Input Handling (Before/After)

**Before (in main loop):**
```rust
// 470 lines of match statement
for event in event_pump.poll_iter() {
    match event {
        Event::Quit { .. } => break 'running,
        Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
            match game_state {
                GameState::Playing => {
                    if inventory_ui.is_open {
                        inventory_ui.is_open = false;
                    } else if matches!(debug_menu_state, DebugMenuState::Open { .. }) {
                        debug_menu_state = DebugMenuState::Closed;
                    } else {
                        game_state = GameState::ExitMenu;
                    }
                }
                GameState::ExitMenu => {
                    game_state = GameState::Playing;
                }
                GameState::Dead => {
                    game_state = GameState::ExitMenu;
                    death_screen.reset();
                }
            }
        }
        // ... 450 more lines
    }
}
```

**After (in Game::handle_input):**
```rust
fn handle_input(&mut self) -> Result<bool, String> {
    for event in self.event_pump.poll_iter() {
        // Quit check
        if matches!(event, Event::Quit { .. }) {
            return Ok(true); // Signal quit
        }

        // Route to specialized handlers
        match self.game_state {
            GameState::Playing => self.handle_playing_input(&event)?,
            GameState::ExitMenu => self.handle_menu_input(&event)?,
            GameState::Dead => self.handle_dead_input(&event)?,
        }
    }

    Ok(false) // Continue running
}

fn handle_playing_input(&mut self, event: &Event) -> Result<(), String> {
    // Input when playing (movement, attack, inventory, debug)
    // Delegates to specialized handlers:
    // - self.handle_gameplay_keys(event)
    // - self.handle_debug_keys(event)
    // - self.handle_mouse_input(event)
    // Each ~50 lines instead of 470
}
```

**Benefit:** 470 lines → 3 functions of ~50-80 lines each = more readable, testable.

### Example: Collision Handling (Before/After)

**Before (in main loop):**
```rust
// Dynamic collision (player vs slimes)
let colliding_slime_indices = check_collisions_with_collection(&player, &slimes);
for slime_index in colliding_slime_indices {
    let player_bounds = player.get_bounds();
    let slime_bounds = slimes[slime_index].get_bounds();
    let (overlap_x, overlap_y) = calculate_overlap(&player_bounds, &slime_bounds);

    // Hardcoded ratios!
    if overlap_x.abs() < overlap_y.abs() {
        player.apply_push(-overlap_x * 3 / 10, 0);
        slimes[slime_index].apply_push(overlap_x * 7 / 10, 0);
    } else {
        player.apply_push(0, -overlap_y * 3 / 10);
        slimes[slime_index].apply_push(0, overlap_y * 7 / 10);
    }

    // Damage on contact
    if !player.is_attacking && !slimes[slime_index].is_invulnerable() {
        // ... 20 more lines
    }
}

// Static collision (player vs walls) - DUPLICATE LOGIC
let static_collisions = check_static_collisions(&player, &all_static_collidables);
for obj_index in static_collisions {
    let player_bounds = player.get_bounds();
    let obj_bounds = all_static_collidables[obj_index].get_bounds();
    let (overlap_x, overlap_y) = calculate_overlap(&player_bounds, &obj_bounds);

    // Same logic, different ratios
    if overlap_x.abs() < overlap_y.abs() {
        player.apply_push(-overlap_x, 0);
    } else {
        player.apply_push(0, -overlap_y);
    }
}
```

**After (in Game::resolve_collisions):**
```rust
fn resolve_collisions(&mut self) -> Result<(), String> {
    // Dynamic collisions
    self.resolve_dynamic_collisions()?;

    // Static collisions
    self.resolve_static_collisions()?;

    Ok(())
}

fn resolve_dynamic_collisions(&mut self) -> Result<(), String> {
    let colliding_pairs = check_collisions_with_collection(
        &self.world.player,
        &self.world.slimes,
    );

    for slime_index in colliding_pairs {
        // Get collision info
        let (player_bounds, slime_bounds) = (
            self.world.player.get_bounds(),
            self.world.slimes[slime_index].get_bounds(),
        );
        let overlap = calculate_overlap(&player_bounds, &slime_bounds);

        // Apply push (unified logic for both axes)
        apply_push_separation(
            &mut self.world.player,
            &mut self.world.slimes[slime_index],
            overlap,
            PLAYER_MASS,
            SLIME_MASS,
        );

        // Handle damage
        self.handle_contact_damage(slime_index)?;
    }

    Ok(())
}

// Helper function (reusable!)
fn apply_push_separation(
    entity1: &mut impl Collidable,
    entity2: &mut impl Collidable,
    overlap: (i32, i32),
    mass1: f32,
    mass2: f32,
) {
    let total_mass = mass1 + mass2;
    let ratio1 = mass2 / total_mass;
    let ratio2 = mass1 / total_mass;

    let (overlap_x, overlap_y) = overlap;
    if overlap_x.abs() < overlap_y.abs() {
        entity1.apply_push((-overlap_x as f32 * ratio1) as i32, 0);
        entity2.apply_push((overlap_x as f32 * ratio2) as i32, 0);
    } else {
        entity1.apply_push(0, (-overlap_y as f32 * ratio1) as i32);
        entity2.apply_push(0, (overlap_y as f32 * ratio2) as i32);
    }
}
```

**Benefits:**
- Duplicate logic eliminated (one `apply_push_separation` function)
- Mass-based ratios instead of magic numbers
- Testable (can test collision logic without full game)
- Clearer (obvious what each function does)

---

## Migration Strategy

### Overview

**Phased approach to avoid breaking the game:**

1. **Phase 1 (Low Risk):** Extract functions from main() - no structural changes
2. **Phase 2 (Medium Risk):** Create structs (GameWorld, Systems, UIManager)
3. **Phase 3 (Medium Risk):** Create Game struct, move functions to methods
4. **Phase 4 (Higher Risk):** Integrate specialized systems (Input, Events, Physics)

Each phase maintains a working game. Test thoroughly before next phase.

### Phase 1: Extract Functions (Week 1)

**Goal:** Reduce main() from 1,130 lines to ~400 lines by extracting functions.

**Tasks:**

1. **Extract event handling:**
   ```rust
   // Before: 470 lines in main loop
   for event in event_pump.poll_iter() {
       match event { /* 470 lines */ }
   }

   // After: Function call
   if handle_events(
       &mut event_pump,
       &mut game_state,
       &mut player,
       &mut slimes,
       /* ... 15 parameters */
   )? {
       break 'running; // Quit requested
   }
   ```

2. **Extract rendering:**
   ```rust
   fn render_game(
       canvas: &mut Canvas<Window>,
       world_grid: &WorldGrid,
       render_grid: &RenderGrid,
       player: &Player,
       slimes: &[Slime],
       /* ... 10 parameters */
   ) -> Result<(), String> {
       // All rendering logic (160 lines)
   }
   ```

3. **Extract collision resolution:**
   ```rust
   fn resolve_collisions(
       player: &mut Player,
       slimes: &mut [Slime],
       static_objects: &[StaticObject],
       /* ... */
   ) -> Result<(), String> {
       // Collision detection and response (80 lines)
   }
   ```

4. **Extract update logic:**
   ```rust
   fn update_world(
       player: &mut Player,
       slimes: &mut [Slime],
       entities: &mut [TheEntity],
       /* ... */
       delta_time: f32,
   ) -> Result<(), String> {
       // Entity updates, AI, physics (100 lines)
   }
   ```

**Success Criteria:**
- main() reduced to ~400 lines
- Game still works identically
- All tests pass
- No borrow checker errors

**Estimated Time:** 2-3 days

### Phase 2: Create Structs (Week 2)

**Goal:** Group related data into structs, reduce parameter passing.

**Tasks:**

1. **Create GameWorld struct:**
   ```rust
   pub struct GameWorld<'a> {
       pub player: Player<'a>,
       pub slimes: Vec<Slime<'a>>,
       pub entities: Vec<TheEntity<'a>>,
       pub dropped_items: Vec<DroppedItem<'a>>,
       pub grid: WorldGrid,
       pub render_grid: RenderGrid,
       pub inventory: PlayerInventory,
       pub attack_effects: Vec<AttackEffect<'a>>,
       pub floating_texts: Vec<FloatingTextInstance>,
       pub active_attack: Option<combat::AttackEvent>,
   }
   ```

   **Benefit:** Functions now take `&mut GameWorld` instead of 15 parameters!

2. **Create Systems struct:**
   ```rust
   pub struct Systems {
       pub player_config: AnimationConfig,
       pub slime_config: AnimationConfig,
       pub punch_config: AnimationConfig,
       pub item_registry: ItemRegistry,
       pub item_textures: HashMap<String, Texture<'a>>,
       pub save_manager: SaveManager,
       pub static_objects: Vec<StaticObject>,
       pub debug_config: DebugConfig,
   }
   ```

3. **Create UIManager struct:**
   ```rust
   pub struct UIManager<'a> {
       pub save_exit_menu: SaveExitMenu,
       pub death_screen: DeathScreen,
       pub inventory_ui: InventoryUI<'a>,
       pub player_health_bar: HealthBar,
       pub enemy_health_bar: HealthBar,
       pub floating_text_renderer: FloatingText,
       pub buff_display: BuffDisplay<'a>,
       pub mouse_x: i32,
       pub mouse_y: i32,
   }
   ```

4. **Update function signatures:**
   ```rust
   // Before:
   fn update_world(player: &mut Player, slimes: &mut [Slime], /* 12 params */) { }

   // After:
   fn update_world(world: &mut GameWorld) { }
   ```

**Success Criteria:**
- main() reduced to ~300 lines
- Functions take 1-3 parameters instead of 15
- Game still works identically
- Save/load still works

**Estimated Time:** 3-4 days

### Phase 3: Create Game Struct (Week 3)

**Goal:** Encapsulate everything in Game struct, move functions to methods.

**Tasks:**

1. **Create Game struct:**
   ```rust
   pub struct Game<'a> {
       canvas: Canvas<Window>,
       event_pump: EventPump,
       texture_creator: &'a TextureCreator<WindowContext>,
       world: GameWorld<'a>,
       systems: Systems,
       ui: UIManager<'a>,
       debug: DebugState,
       game_state: GameState,
       frame_duration: Duration,
       last_frame_time: Instant,
   }
   ```

2. **Move functions to methods:**
   ```rust
   impl<'a> Game<'a> {
       fn handle_input(&mut self) -> Result<bool, String> {
           // Was: handle_events(/* 15 params */)
       }

       fn update(&mut self) -> Result<(), String> {
           // Was: update_world(/* params */)
       }

       fn render(&mut self) -> Result<(), String> {
           // Was: render_game(/* params */)
       }
   }
   ```

3. **Implement Game::new() and Game::run():**
   ```rust
   impl<'a> Game<'a> {
       pub fn new(/* SDL2 components */) -> Result<Self, String> {
           // Initialize all systems
       }

       pub fn run(&mut self) -> Result<(), String> {
           'running: loop {
               if self.handle_input()? { break; }
               if self.game_state == GameState::Playing {
                   self.update()?;
               }
               self.render()?;
               self.limit_frame_rate();
           }
           Ok(())
       }
   }
   ```

4. **Simplify main():**
   ```rust
   fn main() -> Result<(), String> {
       // SDL2 init (~20 lines)
       let mut game = Game::new(canvas, event_pump, &texture_creator)?;
       if let Err(_) = game.load() { /* new game */ }
       game.run()
   }
   ```

**Success Criteria:**
- main() reduced to ~150-200 lines
- Clear ownership model (Game owns everything)
- Game loop is Game::run() method
- No borrow checker fights

**Estimated Time:** 4-5 days

### Phase 4: Integrate Specialized Systems (Week 4+)

**Goal:** Integrate InputSystem, PhysicsSystem, EventSystem (optional, future work).

**This phase is OPTIONAL and can be done incrementally as needed.**

**Tasks:**

1. **InputSystem (if needed):**
   - Design action enum
   - Create input mapper
   - Replace event handling with action processing

2. **PhysicsSystem (if needed):**
   - Extract physics config
   - Mass-based collision response
   - Unified push separation

3. **EventSystem (if needed):**
   - Design event enum
   - Create event queue
   - Refactor direct calls to events

4. **ResourceManager (if needed):**
   - Centralize texture management
   - Pre-built animation controllers
   - Asset manifest system

**Each system is independent and can be added when pain points emerge.**

**Estimated Time:** 1-2 weeks per system (as needed)

### Migration Checklist

- [ ] **Phase 1: Extract Functions**
  - [ ] Extract handle_events() function
  - [ ] Extract render_game() function
  - [ ] Extract resolve_collisions() function
  - [ ] Extract update_world() function
  - [ ] Test: Game works identically
  - [ ] Test: Save/load works
  - [ ] Commit: "refactor: extract functions from main loop"

- [ ] **Phase 2: Create Structs**
  - [ ] Create GameWorld struct
  - [ ] Create Systems struct
  - [ ] Create UIManager struct
  - [ ] Update function signatures
  - [ ] Test: Game works identically
  - [ ] Test: Save/load still works
  - [ ] Commit: "refactor: group data into structs"

- [ ] **Phase 3: Create Game Struct**
  - [ ] Create Game struct
  - [ ] Implement Game::new()
  - [ ] Move functions to methods
  - [ ] Implement Game::run()
  - [ ] Simplify main()
  - [ ] Test: Game works identically
  - [ ] Test: All features work (debug menu, inventory, etc.)
  - [ ] Commit: "refactor: create Game struct"

- [ ] **Phase 4: Specialized Systems (Optional)**
  - [ ] Design and implement as needed
  - [ ] Test each system independently
  - [ ] Commit: "feat: add [SystemName]"

---

## Error Handling Strategy

### Current Approach (Keep As-Is)

```rust
fn main() -> Result<(), String> {
    // Functions return Result<(), String>
    // Use ? operator for propagation
}
```

**Why String errors:**
- Simple, no custom error types needed
- Good enough for learning project
- Easy to add context: `map_err(|e| format!("Failed to load: {}", e))`

**Future improvement (if needed):**
- Custom error enum for different error types
- Only add if error handling becomes complex

### Panic vs Result

**Panic (program terminates):**
- Truly unrecoverable errors (SDL2 init failure)
- Programming bugs (unwrap() on None should never happen)
- Debugging (assertions in dev builds)

**Result (can recover):**
- File not found (use default, show message)
- Missing texture (use placeholder)
- Save/load errors (show error, continue game)

### Error Propagation

```rust
impl<'a> Game<'a> {
    fn handle_input(&mut self) -> Result<bool, String> {
        // Use ? to propagate errors up
        self.handle_gameplay_keys()?;
        self.handle_debug_keys()?;
        self.handle_mouse_input()?;
        Ok(false)
    }

    fn handle_gameplay_keys(&mut self) -> Result<(), String> {
        // Errors automatically propagate to caller
        self.ui.inventory_ui.handle_key(key)?;
        Ok(())
    }
}
```

**Benefit:** Errors bubble up to main(), which logs and exits gracefully.

### User-Facing Errors

```rust
// Good: Informative message
return Err(format!("Failed to load save slot 1: {}", e));

// Bad: Cryptic message
return Err(e.to_string());
```

**In release builds, could show error dialog instead of console message.**

### Debug vs Release

```rust
// Development: Detailed errors
#[cfg(debug_assertions)]
fn load_texture(&self, path: &str) -> Result<Texture, String> {
    self.texture_creator
        .load_texture(path)
        .map_err(|e| format!("Failed to load texture '{}': {}\nPath: {}",
                             path, e, std::env::current_dir().unwrap().display()))
}

// Release: Simple errors
#[cfg(not(debug_assertions))]
fn load_texture(&self, path: &str) -> Result<Texture, String> {
    self.texture_creator
        .load_texture(path)
        .map_err(|_| format!("Failed to load texture"))
}
```

---

## Rust Patterns Explained

### 1. Struct Composition (Ownership Tree)

**Pattern:**
```rust
pub struct Game<'a> {
    world: GameWorld<'a>,  // Game owns GameWorld
    systems: Systems,      // Game owns Systems
    ui: UIManager<'a>,     // Game owns UIManager
}

pub struct GameWorld<'a> {
    player: Player<'a>,         // World owns Player
    slimes: Vec<Slime<'a>>,     // World owns Slimes
}
```

**What it teaches:**
- **Ownership hierarchy:** Clear tree structure (Game → World → Entities)
- **Single owner:** Each piece of data has exactly one owner
- **No shared references:** No Rc/Arc needed (simpler reasoning)
- **Automatic cleanup:** When Game drops, everything drops (RAII)

**Why it's good:**
- Compiler enforces ownership rules
- No manual memory management
- Clear who's responsible for what

### 2. Borrowing (Shared vs Exclusive Access)

**Pattern:**
```rust
impl<'a> Game<'a> {
    fn update(&mut self) -> Result<(), String> {
        // Exclusive access to world
        self.world.update();
        self.resolve_collisions();  // Also needs &mut world
    }

    fn render(&mut self) -> Result<(), String> {
        // Read-only access to world
        render_game(&mut self.canvas, &self.world, &self.systems)?;
    }
}
```

**What it teaches:**
- **&mut** = exclusive access (can modify)
- **&** = shared access (read-only, multiple allowed)
- **Can't mix:** Can't have &mut and & at same time
- **Sequential phases:** Update has &mut, then Render has &

**Why it's good:**
- Prevents data races at compile time
- Forces clear phase separation
- No locks/mutexes needed

### 3. Lifetime 'a (Texture Lifetime)

**Pattern:**
```rust
pub struct Game<'a> {
    texture_creator: &'a TextureCreator<WindowContext>,
    world: GameWorld<'a>,  // Borrows from texture_creator
}

pub struct GameWorld<'a> {
    player: Player<'a>,    // Player has textures from creator
}

pub struct Player<'a> {
    animation: AnimationController<'a>,  // Has Texture<'a> references
}
```

**What it teaches:**
- **Lifetime parameter:** 'a says "lives as long as something else"
- **Borrow checker:** Compiler ensures textures outlive entities
- **No dangling references:** Can't have entity after texture is freed
- **Propagation:** Lifetime bubbles up through ownership tree

**Why it's good:**
- Memory safety without runtime cost
- No use-after-free bugs
- Clear dependency (entities depend on textures)

**SDL2 requirement:** TextureCreator must outlive all Textures it creates. Lifetime 'a enforces this at compile time.

### 4. Result and ? Operator (Error Propagation)

**Pattern:**
```rust
fn main() -> Result<(), String> {
    let game = Game::new(canvas, event_pump, &texture_creator)?;
    //                                                        ^ propagates error up
    game.run()?;
    Ok(())
}

impl Game {
    fn handle_input(&mut self) -> Result<bool, String> {
        self.handle_gameplay_keys()?;  // If error, return early
        self.handle_debug_keys()?;     // Only runs if previous succeeds
        Ok(false)
    }
}
```

**What it teaches:**
- **Result<T, E>:** Function can succeed (T) or fail (E)
- **? operator:** Unwraps Ok, returns Err early
- **Explicit errors:** Compiler forces you to handle failures
- **Composable:** Errors flow up call stack automatically

**Why it's good:**
- No hidden exceptions
- Can't forget error handling (compiler error if unhandled)
- Easy to add context with `map_err`

### 5. Enum for State (Type-Safe States)

**Pattern:**
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum GameState {
    Playing,
    ExitMenu,
    Dead,
}

// Compiler ensures all states handled
match self.game_state {
    GameState::Playing => self.handle_playing_input(event)?,
    GameState::ExitMenu => self.handle_menu_input(event)?,
    GameState::Dead => self.handle_dead_input(event)?,
    // If we add new state, compiler forces us to handle it here!
}
```

**What it teaches:**
- **Exhaustive matching:** Compiler ensures all cases handled
- **Type safety:** Can't accidentally use invalid state
- **Refactoring safety:** Adding state = compiler finds all places to update
- **Self-documenting:** All states visible in enum

**Why it's good:**
- No magic strings ("playing" vs "PLAYING" vs "Playing")
- Can't typo a state
- Adding feature = compiler shows all places to integrate it

### 6. Module System (Code Organization)

**Pattern:**
```rust
// main.rs
mod game;         // src/game.rs
mod game_world;   // src/game_world.rs
mod systems;      // src/systems.rs

use game::Game;
use game_world::GameWorld;

// game.rs
pub struct Game<'a> { /* ... */ }
impl<'a> Game<'a> {
    pub fn new() -> Result<Self, String> { /* ... */ }
    pub fn run(&mut self) -> Result<(), String> { /* ... */ }
}
```

**What it teaches:**
- **Modules = files:** Each module is a separate file
- **pub keyword:** Explicit public API
- **Encapsulation:** Internal details stay private
- **Dependency management:** `use` shows what depends on what

**Why it's good:**
- Clear boundaries between systems
- Can't accidentally use internal details
- Easy to find code (module = file)

---

## Answers to Key Questions

### 1. Should Game be a struct or just organized functions?

**Answer: Start with functions (Phase 1), migrate to Game struct (Phase 3).**

**Rationale:**
- Functions are lowest-risk refactor (no ownership changes)
- Once functions stabilize, wrapping in struct is mechanical
- Struct gives better encapsulation and state management
- Struct enables methods (cleaner than passing 15 parameters)

**Final form:** Game struct with methods (Approach 1).

### 2. Who owns SDL2 Canvas?

**Answer: Game struct owns Canvas.**

```rust
pub struct Game<'a> {
    canvas: Canvas<Window>,  // Owned
    // ...
}
```

**Rationale:**
- Canvas needed for rendering (happens in Game::render)
- TextureCreator borrows from Canvas (lifetime dependency)
- Ownership tree: main() → Game → Canvas

**Alternative considered:** Separate RenderSystem owns Canvas.
**Why rejected:** Over-engineering. Canvas is simple, Game can own it directly.

### 3. How to avoid borrow checker issues with systems?

**Answer: Sequential phases (input → update → render) with clear &mut vs & access.**

**Pattern:**
```rust
fn run(&mut self) -> Result<(), String> {
    loop {
        // Phase 1: &mut self
        self.handle_input()?;

        // Phase 2: &mut self (input done, can mutate world)
        self.update()?;

        // Phase 3: &mut self.canvas, & self.world (world immutable during render)
        self.render()?;

        // Phases don't overlap, no borrow conflicts
    }
}
```

**Key insight:** Update modifies world, render reads world. They never happen simultaneously, so borrow checker is happy.

**If needed:** Split borrows (borrow individual fields, not whole struct):
```rust
// Instead of: self.foo(&mut self.bar)  // Can't borrow self twice!
// Do this:
let bar = &mut self.bar;
let baz = &self.baz;
standalone_function(bar, baz);
```

### 4. Should there be a GameContext struct passed to systems?

**Answer: No, not yet. YAGNI (You Ain't Gonna Need It).**

**Rationale:**
- Current systems don't need shared context
- Would add indirection without clear benefit
- Can add later if pattern emerges (e.g., 5 systems all need same 3 things)

**Future consideration:** If we add many systems that all need (delta_time, event_queue, config), then consider:
```rust
pub struct GameContext<'a> {
    delta_time: f32,
    events: &'a mut EventQueue,
    config: &'a GameConfig,
}
```

But wait until need is clear.

### 5. How to structure initialization?

**Answer: Game::new() constructor.**

```rust
impl<'a> Game<'a> {
    pub fn new(
        canvas: Canvas<Window>,
        event_pump: EventPump,
        texture_creator: &'a TextureCreator<WindowContext>,
    ) -> Result<Self, String> {
        // Load configs
        let systems = Systems::new(texture_creator)?;

        // Create world (new game)
        let world = GameWorld::new(
            &systems.player_config,
            &systems.slime_config,
            texture_creator,
            &systems.item_textures,
        )?;

        // Initialize UI
        let ui = UIManager::new(
            texture_creator,
            &systems.item_textures,
            &systems.item_registry,
        )?;

        Ok(Self {
            canvas,
            event_pump,
            texture_creator,
            world,
            systems,
            ui,
            game_state: GameState::Playing,
            debug: DebugState::new(),
            frame_duration: Duration::from_millis(16), // 60 FPS
            last_frame_time: Instant::now(),
        })
    }
}
```

**Builder pattern considered, but rejected:** Overkill for game with simple initialization. Constructor is sufficient.

### 6. Should systems have traits/common interface?

**Answer: No, not yet.**

```rust
// DON'T do this (yet):
pub trait System {
    fn update(&mut self, ctx: &mut GameContext);
}

// Vec<Box<dyn System>> adds complexity for no clear benefit
```

**Rationale:**
- Systems have different update signatures (collision needs world, input needs events)
- Trait objects add runtime cost (dynamic dispatch)
- Current approach (direct method calls) is simpler and faster
- Can add traits later if clear pattern emerges

**When to add:** If we have 10+ systems that all follow same pattern. Not needed now.

### 7. How to handle frame timing?

**Answer: Fixed timestep (60 FPS) with sleep.**

```rust
const FRAME_DURATION: Duration = Duration::from_millis(16); // ~60 FPS

impl Game {
    fn limit_frame_rate(&mut self) {
        let elapsed = self.last_frame_time.elapsed();
        if elapsed < self.frame_duration {
            std::thread::sleep(self.frame_duration - elapsed);
        }
        self.last_frame_time = Instant::now();
    }
}
```

**Why this approach:**
- Simple, deterministic
- Works well for 2D games
- No complex timestep accumulator needed

**Future improvement (if frame drops occur):**
```rust
// Variable delta time with cap
let delta_time = self.last_frame_time.elapsed().as_secs_f32().min(0.1);
```

But only add if needed. Current approach is fine.

### 8. Where does save/load functionality live?

**Answer: Methods on Game struct.**

```rust
impl<'a> Game<'a> {
    pub fn save(&mut self) -> Result<(), String> {
        self.systems.save_manager.save_game(
            &self.world.player,
            &self.world.slimes,
            // ...
        )
    }

    pub fn load(&mut self) -> Result<(), String> {
        let loaded_data = self.systems.save_manager.load_game(/* ... */)?;
        self.world = GameWorld::from_loaded(loaded_data);
        Ok(())
    }
}
```

**Rationale:**
- Save/load needs access to entire world state
- Game owns world, so Game is natural place for these operations
- SaveManager (in Systems) does file I/O, Game coordinates what to save

**Alternative considered:** Free functions `save_game()` and `load_game()`.
**Why rejected:** Would need to pass world state in/out, messier than method.

### 9. How to transition between game states?

**Answer: GameState enum with explicit transitions in handle_input.**

```rust
// In handle_input:
match event {
    Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
        match self.game_state {
            GameState::Playing => {
                // Close UIs first, then menu
                if self.ui.inventory_ui.is_open {
                    self.ui.inventory_ui.is_open = false;
                } else {
                    self.game_state = GameState::ExitMenu;
                }
            }
            GameState::ExitMenu => {
                self.game_state = GameState::Playing;
            }
            GameState::Dead => {
                // From death screen, ESC opens menu
                self.game_state = GameState::ExitMenu;
                self.ui.death_screen.reset();
            }
        }
    }
}
```

**Future consideration:** If states become complex (nested menus, level transitions), consider state stack:
```rust
pub struct StateStack {
    states: Vec<GameState>,
}
// Push/pop states instead of direct assignment
```

But not needed yet.

### 10. How to integrate debug features?

**Answer: DebugState struct with cfg gates.**

```rust
pub struct DebugState {
    pub menu_state: DebugMenuState,
    pub show_collision_boxes: bool,
    pub show_tile_grid: bool,
}

// In Game::render:
#[cfg(debug_assertions)]
if self.debug.show_collision_boxes {
    render_collision_boxes(&mut self.canvas, &self.world)?;
}
```

**Future improvement:**
- Compile out debug code in release builds
- Separate debug module: `src/debug.rs`
- Debug console for runtime commands

But current approach is sufficient for now.

---

## Trade-offs and Future Work

### Design Trade-offs

| Decision | Trade-off | Rationale |
|----------|-----------|-----------|
| **Phased migration** | Slower to reach "ideal" architecture | Lower risk, game stays working |
| **No trait-based System** | Less generic, more repetitive | Simpler, faster, YAGNI |
| **Game owns everything** | Large struct, many fields | Clear ownership, no indirection |
| **Fixed timestep** | May stutter on slow machines | Simple, deterministic physics |
| **String errors** | Less structured error handling | Good enough for learning project |
| **No InputSystem yet** | Event handling still verbose | Wait until handle_input stabilizes |

### Future Improvements

**Phase 5 (Optional, as needed):**

1. **InputSystem**
   - When: handle_input() stabilizes and is too large
   - Benefit: 100-line action handler instead of 300-line event handler

2. **PhysicsSystem**
   - When: Physics tuning becomes pain point
   - Benefit: Config-driven physics, mass-based separation

3. **EventSystem**
   - When: Need logging, achievements, or system decoupling
   - Benefit: Observability, loosely coupled systems

4. **ResourceManager**
   - When: Asset loading becomes complex
   - Benefit: Hot-reload, lazy loading, cleaner entity spawning

5. **State Stack**
   - When: Need nested states (pause during inventory, etc.)
   - Benefit: Cleaner state management, push/pop paradigm

6. **Testing**
   - Unit tests for collision logic
   - Integration tests for save/load
   - Fuzzing for robust error handling

**None of these are needed immediately. Add when pain points emerge.**

### Open Questions

1. **Should WorldGrid and RenderGrid be separate?**
   - Current: RenderGrid wraps WorldGrid, caches rendering
   - Alternative: Merge into single GridSystem
   - Decision: Keep separate for now (clear separation of concerns)

2. **Should PlayerInventory be part of Player or GameWorld?**
   - Current: In GameWorld (alongside player)
   - Alternative: Field in Player struct
   - Decision: Keep in GameWorld (easier save/load, player is simpler)

3. **How to handle entity IDs/references?**
   - Current: Vec indices (slimes[index])
   - Issue: Indices invalidate when entity removed
   - Future: Generational indexes or entity handles
   - Decision: Vec indices sufficient for now (few entities, simple logic)

4. **Should there be a Camera abstraction?**
   - Current: No camera (viewport is static 640x360)
   - Future: If scrolling world, camera that tracks player
   - Decision: Not needed yet (single-screen game)

---

## Summary

### Key Architectural Decisions

1. **Hybrid Approach:** Start with functions (Phase 1), migrate to Game struct (Phase 3)
2. **GameWorld Struct:** Owns all entities, world state, visual effects
3. **Systems Struct:** Owns configs, managers, helpers
4. **UIManager Struct:** Owns UI state (menus, HUD, input tracking)
5. **Clear Ownership:** Game → World → Entities (tree structure)
6. **Sequential Phases:** Input → Update → Render (prevents borrow conflicts)
7. **Fixed Timestep:** 60 FPS with sleep (simple, deterministic)
8. **String Errors:** Result<(), String> throughout (good enough)
9. **No Premature Systems:** Wait for InputSystem, PhysicsSystem, EventSystem until needed

### Migration Path

- **Week 1:** Extract functions (1,130 → 400 lines)
- **Week 2:** Create structs (400 → 300 lines)
- **Week 3:** Create Game struct (300 → 150 lines)
- **Week 4+:** Optional specialized systems (as needed)

### Success Metrics

- ✅ main() < 200 lines
- ✅ Clear phase separation (input/update/render)
- ✅ Testable systems (each phase can be unit tested)
- ✅ Maintainable (adding feature touches < 3 files)
- ✅ Rust-idiomatic (ownership, borrowing, error handling)
- ✅ No performance regression (still 60 FPS)

### What This Teaches (Rust Learning)

1. **Struct composition** - Building complex systems from simple parts
2. **Ownership tree** - Clear hierarchy, single owners
3. **Borrowing** - &mut vs &, sequential phases
4. **Lifetimes** - 'a for texture lifetime, propagation
5. **Result and ?** - Explicit error handling, propagation
6. **Enums** - Type-safe states, exhaustive matching
7. **Modules** - Code organization, encapsulation

---

**This design provides a clear, incremental path from the current 1,130-line monolith to a clean, maintainable 150-200 line orchestration layer, while teaching core Rust patterns along the way.**

**Next Steps:** Begin Phase 1 (extract functions) and iterate from there.
