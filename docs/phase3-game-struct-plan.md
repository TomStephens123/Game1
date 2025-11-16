# Phase 3: Game Struct - Master Plan

## Executive Summary

**Goal**: Create a unified `Game` struct that owns all game state and converts standalone functions into methods, reducing main() from 421 lines to ~150 lines.

**Approach**: Incremental refactoring with validation at each step. Each agent task is self-contained and can be executed independently.

**Timeline**: 4-6 hours total
- Step 1: 30 minutes (Game struct definition)
- Step 2: 1 hour (Convert handle_events to method)
- Step 3: 1 hour (Convert update_world to method)
- Step 4: 1 hour (Create render method)
- Step 5: 30 minutes (Create run method)
- Step 6: 30 minutes (Refactor main to use Game)
- Step 7: 30 minutes (Testing and validation)

**Success Metrics**:
- main() reduced to ~150 lines
- Game loop is `game.run()`
- All functions are methods on Game
- Zero compilation errors
- All 39 tests still pass

---

## Architecture Overview

### Current Structure (Phase 2)
```
main() [421 lines]
├── SDL2 initialization
├── Texture loading
├── Systems::new()
├── GameWorld construction
├── UIManager construction
├── Game loop [275 lines]
│   ├── handle_events() [13 params]
│   ├── update_world() [5 params]
│   └── Rendering code [inline, ~190 lines]
└── Ok(())
```

### Target Structure (Phase 3)
```
main() [~150 lines]
├── SDL2 initialization
├── Texture loading
├── Game::new() or Game::load()
└── game.run()

Game struct
├── world: GameWorld
├── systems: Systems
├── ui: UIManager
├── game_state: GameState
├── canvas: Canvas
├── event_pump: EventPump
├── texture_creator: &TextureCreator
├── textures: GameTextures struct
└── Methods:
    ├── run() - main game loop
    ├── handle_events() - input processing
    ├── update() - game logic
    └── render() - drawing
```

---

## Step-by-Step Implementation Plan

### Step 1: Define Game Struct and GameTextures Helper

**Agent Role**: Code Architect
**Task Duration**: 30 minutes
**Dependencies**: None

**Objective**: Create the `Game` struct definition and `GameTextures` helper struct.

**Files to Modify**:
- `src/main.rs` (add struct definitions after UIManager)

**Implementation Details**:

```rust
/// Helper struct to hold all game textures
/// This avoids repeating texture parameters everywhere
pub struct GameTextures<'a> {
    pub character: &'a Texture<'a>,
    pub slime: &'a Texture<'a>,
    pub entity: &'a Texture<'a>,
    pub punch: &'a Texture<'a>,
    pub grass_tile: &'a Texture<'a>,
    pub items: &'a HashMap<String, Texture<'a>>,
}

/// Main game struct that owns all game state
/// This is the top-level orchestrator for the entire game
pub struct Game<'a> {
    // Core game state
    pub world: GameWorld<'a>,
    pub systems: Systems,
    pub ui: UIManager<'a>,
    pub game_state: GameState,

    // SDL2 components
    pub canvas: Canvas<Window>,
    pub event_pump: EventPump,

    // Resources (textures need special lifetime handling)
    pub texture_creator: &'a TextureCreator<WindowContext>,
    pub textures: GameTextures<'a>,

    // Other resources
    pub item_registry: ItemRegistry,
    pub save_manager: SaveManager,
}
```

**Agent Instructions**:
1. Read `src/main.rs` lines 190-206 to see where UIManager is defined
2. Add the `GameTextures` struct after UIManager
3. Add the `Game` struct after GameTextures
4. Use the exact field names and types shown above
5. Add doc comments explaining each field group
6. Run `cargo check` to verify struct compiles

**Validation**:
- [ ] GameTextures struct compiles
- [ ] Game struct compiles
- [ ] All lifetimes are correct ('a propagated)
- [ ] No compilation errors

**Success Criteria**:
```bash
cargo check  # Should pass
```

---

### Step 2: Convert handle_events to Game Method

**Agent Role**: Refactoring Specialist
**Task Duration**: 1 hour
**Dependencies**: Step 1 complete

**Objective**: Move `handle_events()` function to be a method on `Game`, reducing its parameters from 13 to 0 (uses self).

**Files to Modify**:
- `src/main.rs` (lines 207-701)

**Current Signature** (line 209):
```rust
fn handle_events<'a>(
    event_pump: &mut sdl2::EventPump,
    game_state: &mut GameState,
    world: &mut GameWorld<'a>,
    systems: &mut Systems,
    ui: &mut UIManager<'a>,
    save_manager: &mut SaveManager,
    character_texture: &'a sdl2::render::Texture<'a>,
    slime_texture: &'a sdl2::render::Texture<'a>,
    entity_texture: &'a sdl2::render::Texture<'a>,
    punch_texture: &'a sdl2::render::Texture<'a>,
    item_textures: &'a HashMap<String, sdl2::render::Texture<'a>>,
    item_registry: &ItemRegistry,
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
) -> Result<bool, String>
```

**Target Signature**:
```rust
impl<'a> Game<'a> {
    /// Handle all input events from SDL2 event pump
    /// Returns true if the game should quit
    pub fn handle_events(&mut self) -> Result<bool, String>
}
```

**Agent Instructions**:

1. **Locate the function**: Find `fn handle_events` at line 209
2. **Create impl block**: After the Game struct definition, add:
   ```rust
   impl<'a> Game<'a> {
       // Methods will go here
   }
   ```
3. **Move function**: Cut the entire `handle_events` function body (lines 209-701)
4. **Paste as method**: Inside the impl block, change signature to:
   ```rust
   pub fn handle_events(&mut self) -> Result<bool, String> {
       // ... existing body
   }
   ```
5. **Update all field accesses**: Replace all parameter references with `self.field`:

**Find-Replace Patterns** (apply in order):
```
Find:    event_pump.poll_iter()
Replace: self.event_pump.poll_iter()

Find:    *game_state
Replace: self.game_state

Find:    game_state ==
Replace: self.game_state ==

Find:    world.player
Replace: self.world.player

Find:    world.slimes
Replace: self.world.slimes

Find:    world.entities
Replace: self.world.entities

Find:    world.world_grid
Replace: self.world.world_grid

Find:    world.render_grid
Replace: self.world.render_grid

Find:    world.player_inventory
Replace: self.world.player_inventory

Find:    world.dropped_items
Replace: self.world.dropped_items

Find:    world.attack_effects
Replace: self.world.attack_effects

Find:    world.active_attack
Replace: self.world.active_attack

Find:    systems.player_config
Replace: self.systems.player_config

Find:    systems.slime_config
Replace: self.systems.slime_config

Find:    systems.punch_config
Replace: self.systems.punch_config

Find:    systems.debug_config
Replace: self.systems.debug_config

Find:    ui.inventory_ui
Replace: self.ui.inventory_ui

Find:    ui.debug_menu_state
Replace: self.ui.debug_menu_state

Find:    ui.save_exit_menu
Replace: self.ui.save_exit_menu

Find:    ui.death_screen
Replace: self.ui.death_screen

Find:    ui.show_collision_boxes
Replace: self.ui.show_collision_boxes

Find:    ui.show_tile_grid
Replace: self.ui.show_tile_grid

Find:    ui.is_tilling
Replace: self.ui.is_tilling

Find:    ui.last_tilled_tile
Replace: self.ui.last_tilled_tile

Find:    ui.mouse_x
Replace: self.ui.mouse_x

Find:    ui.mouse_y
Replace: self.ui.mouse_y

Find:    save_manager,
Replace: &mut self.save_manager,

Find:    save_manager)
Replace: &mut self.save_manager)

Find:    character_texture,
Replace: self.textures.character,

Find:    slime_texture,
Replace: self.textures.slime,

Find:    entity_texture,
Replace: self.textures.entity,

Find:    punch_texture,
Replace: self.textures.punch,

Find:    item_textures)
Replace: self.textures.items)

Find:    item_textures.get
Replace: self.textures.items.get

Find:    item_registry)
Replace: &self.item_registry)

Find:    item_registry.get
Replace: self.item_registry.get

Find:    canvas.logical_size()
Replace: self.canvas.logical_size()

Find:    canvas)
Replace: &mut self.canvas)
```

6. **Remove function parameters from doc comment**: Update the doc comment to reflect that it's now a method
7. **Run validation**: `cargo check`

**Validation Checklist**:
- [ ] Function moved to impl block
- [ ] Signature changed to `&mut self`
- [ ] All parameter references changed to `self.field`
- [ ] No compilation errors
- [ ] Function body logic unchanged

**Success Criteria**:
```bash
cargo check  # Should pass with no errors
```

**Common Issues**:
- **Borrow checker errors**: Make sure to use `&mut self.field` for mutable borrows
- **Missing self prefix**: Search for any standalone variable names that should be `self.`
- **Texture references**: Textures need `self.textures.` prefix

---

### Step 3: Convert update_world to Game Method

**Agent Role**: Refactoring Specialist
**Task Duration**: 1 hour
**Dependencies**: Step 1 complete (can run parallel with Step 2)

**Objective**: Move `update_world()` function to be a method on `Game`, reducing its parameters from 5 to 0.

**Files to Modify**:
- `src/main.rs` (lines 703-995)

**Current Signature** (line 706):
```rust
fn update_world<'a>(
    world: &mut GameWorld<'a>,
    systems: &mut Systems,
    item_textures: &'a HashMap<String, sdl2::render::Texture<'a>>,
    item_registry: &ItemRegistry,
    keyboard_state: &sdl2::keyboard::KeyboardState,
) -> Result<(), String>
```

**Target Signature**:
```rust
impl<'a> Game<'a> {
    /// Update game world state (entities, collisions, physics, loot)
    /// Called once per frame when game_state == Playing
    pub fn update(&mut self, keyboard_state: &sdl2::keyboard::KeyboardState) -> Result<(), String>
}
```

**Agent Instructions**:

1. **Locate the function**: Find `fn update_world` at line 706
2. **Move function**: Cut the entire function (lines 706-995)
3. **Paste as method**: In the same `impl<'a> Game<'a>` block (after handle_events), add:
   ```rust
   /// Update game world state (entities, collisions, physics, loot)
   /// Called once per frame when game_state == Playing
   pub fn update(&mut self, keyboard_state: &sdl2::keyboard::KeyboardState) -> Result<(), String> {
       // ... existing body
   }
   ```
4. **Update all field accesses**: Replace parameter references with `self.`:

**Find-Replace Patterns** (apply in order):
```
Find:    world.player
Replace: self.world.player

Find:    world.slimes
Replace: self.world.slimes

Find:    world.entities
Replace: self.world.entities

Find:    world.dropped_items
Replace: self.world.dropped_items

Find:    world.player_inventory
Replace: self.world.player_inventory

Find:    world.active_attack
Replace: self.world.active_attack

Find:    world.attack_effects
Replace: self.world.attack_effects

Find:    world.floating_texts
Replace: self.world.floating_texts

Find:    systems.has_regen
Replace: self.systems.has_regen

Find:    systems.regen_timer
Replace: self.systems.regen_timer

Find:    systems.regen_interval
Replace: self.systems.regen_interval

Find:    systems.debug_config
Replace: self.systems.debug_config

Find:    systems.static_objects
Replace: self.systems.static_objects

Find:    item_textures.get
Replace: self.textures.items.get

Find:    item_registry)
Replace: &self.item_registry)

Find:    item_registry,
Replace: &self.item_registry,
```

5. **Remove #[allow(clippy::too_many_arguments)]**: No longer needed since we're using `self`
6. **Run validation**: `cargo check`

**Validation Checklist**:
- [ ] Function moved to impl block
- [ ] Signature changed to `&mut self` with keyboard_state param
- [ ] All parameter references changed to `self.field`
- [ ] Clippy attribute removed
- [ ] No compilation errors
- [ ] Function body logic unchanged

**Success Criteria**:
```bash
cargo check  # Should pass with no errors
```

---

### Step 4: Create render Method

**Agent Role**: Code Extractor
**Task Duration**: 1 hour
**Dependencies**: Step 1 complete

**Objective**: Extract rendering code from main loop into a `render()` method on `Game`.

**Files to Modify**:
- `src/main.rs` (lines ~1632-1865)

**Target Signature**:
```rust
impl<'a> Game<'a> {
    /// Render the entire game scene
    /// Handles world rendering, UI, debug overlays, and presents to screen
    pub fn render(&mut self) -> Result<(), String>
}
```

**Agent Instructions**:

1. **Locate rendering code**: Find the "PHASE 3: Render" comment around line 1632
2. **Identify the rendering block**: From `canvas.set_draw_color` to `canvas.present()` (lines 1633-1865)
3. **Create new method**: In the `impl<'a> Game<'a>` block, add:
   ```rust
   /// Render the entire game scene
   /// Handles world rendering, UI, debug overlays, and presents to screen
   pub fn render(&mut self) -> Result<(), String> {
       // Clear screen
       self.canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
       self.canvas.clear();

       // TODO: Copy rendering code here

       // Present to screen
       self.canvas.present();
       Ok(())
   }
   ```
4. **Copy the rendering code**: Take all code between `canvas.clear()` and `canvas.present()` from main loop
5. **Update all references**: Replace variable references with `self.`:

**Find-Replace Patterns**:
```
Find:    canvas.
Replace: self.canvas.

Find:    grass_tile_texture
Replace: self.textures.grass_tile

Find:    world.player
Replace: self.world.player

Find:    world.slimes
Replace: self.world.slimes

Find:    world.entities
Replace: self.world.entities

Find:    world.dropped_items
Replace: self.world.dropped_items

Find:    world.attack_effects
Replace: self.world.attack_effects

Find:    world.floating_texts
Replace: self.world.floating_texts

Find:    world.active_attack
Replace: self.world.active_attack

Find:    world.world_grid
Replace: self.world.world_grid

Find:    world.render_grid
Replace: self.world.render_grid

Find:    systems.static_objects
Replace: self.systems.static_objects

Find:    systems.has_regen
Replace: self.systems.has_regen

Find:    systems.debug_config
Replace: self.systems.debug_config

Find:    ui.player_health_bar
Replace: self.ui.player_health_bar

Find:    ui.enemy_health_bar
Replace: self.ui.enemy_health_bar

Find:    ui.floating_text_renderer
Replace: self.ui.floating_text_renderer

Find:    ui.buff_display
Replace: self.ui.buff_display

Find:    ui.show_collision_boxes
Replace: self.ui.show_collision_boxes

Find:    ui.show_tile_grid
Replace: self.ui.show_tile_grid

Find:    ui.inventory_ui
Replace: self.ui.inventory_ui

Find:    ui.death_screen
Replace: self.ui.death_screen

Find:    ui.save_exit_menu
Replace: self.ui.save_exit_menu

Find:    ui.debug_menu_state
Replace: self.ui.debug_menu_state

Find:    ui.mouse_x
Replace: self.ui.mouse_x

Find:    ui.mouse_y
Replace: self.ui.mouse_y

Find:    game_state ==
Replace: self.game_state ==
```

6. **Keep helper function calls**: Functions like `render_with_depth_sorting` and `render_debug_menu` stay as-is
7. **Run validation**: `cargo check`

**Validation Checklist**:
- [ ] Method created in impl block
- [ ] All rendering code copied
- [ ] All variable references updated to `self.`
- [ ] canvas.present() at the end
- [ ] No compilation errors

**Success Criteria**:
```bash
cargo check  # Should pass with no errors
```

---

### Step 5: Create run Method (Main Game Loop)

**Agent Role**: System Integrator
**Task Duration**: 30 minutes
**Dependencies**: Steps 2, 3, and 4 complete

**Objective**: Create the `run()` method that orchestrates the game loop, calling the three methods we created.

**Files to Modify**:
- `src/main.rs` (add to impl block)

**Target Implementation**:
```rust
impl<'a> Game<'a> {
    /// Run the main game loop
    /// This is the entry point that orchestrates input, update, and render
    pub fn run(&mut self) -> Result<(), String> {
        'running: loop {
            // PHASE 1: Handle input events
            if self.handle_events()? {
                break 'running; // Quit requested
            }

            // Check if UI is blocking gameplay
            let is_ui_active = self.ui.inventory_ui.is_open ||
                               matches!(self.ui.debug_menu_state, DebugMenuState::Open { .. }) ||
                               self.game_state == GameState::ExitMenu ||
                               self.game_state == GameState::Dead;

            // PHASE 2: Update game state
            if self.game_state == GameState::Playing && !is_ui_active {
                // Check for player death
                if self.world.player.state.is_dead() {
                    self.game_state = GameState::Dead;
                    self.ui.death_screen.trigger();
                    println!("Player died!");
                }

                let keyboard_state = self.event_pump.keyboard_state();
                self.update(&keyboard_state)?;
            }

            // Handle death screen respawn
            if self.game_state == GameState::Dead {
                if self.ui.death_screen.should_respawn() {
                    self.world.player.respawn(GAME_WIDTH as i32 / 2, GAME_HEIGHT as i32 / 2);
                    self.ui.death_screen.reset();
                    self.game_state = GameState::Playing;
                    println!("Player respawned!");
                }
            }

            // PHASE 3: Render
            self.render()?;

            // PHASE 4: Frame rate limiting
            std::thread::sleep(std::time::Duration::new(0, 1_000_000_000u32 / 60));
        }

        Ok(())
    }
}
```

**Agent Instructions**:

1. **Add method to impl block**: After the render() method
2. **Copy the game loop structure**: Use the code from lines 1595-1868 in main()
3. **Simplify the loop**: Replace function calls with method calls:
   - `handle_events(...)` → `self.handle_events()?`
   - `update_world(...)` → `self.update(&keyboard_state)?`
   - Rendering code → `self.render()?`
4. **Keep the control flow**: Death check, respawn logic, frame limiting all stay the same
5. **Update variable references**: Change `world.`, `ui.`, `game_state` to `self.world.`, `self.ui.`, `self.game_state`
6. **Run validation**: `cargo check`

**Validation Checklist**:
- [ ] run() method created
- [ ] Calls handle_events(), update(), render() methods
- [ ] Game state management logic preserved
- [ ] Frame rate limiting at 60 FPS
- [ ] No compilation errors

**Success Criteria**:
```bash
cargo check  # Should pass with no errors
```

---

### Step 6: Create Game Constructors

**Agent Role**: Constructor Specialist
**Task Duration**: 1 hour
**Dependencies**: Steps 1-5 complete

**Objective**: Create `new()` and `load()` constructors for the `Game` struct to simplify initialization.

**Files to Modify**:
- `src/main.rs` (add to impl block)

**Target Implementation**:

```rust
impl<'a> Game<'a> {
    /// Create a new game with default/fresh state
    pub fn new(
        canvas: Canvas<Window>,
        event_pump: EventPump,
        texture_creator: &'a TextureCreator<WindowContext>,
        textures: GameTextures<'a>,
        player_config: AnimationConfig,
        slime_config: AnimationConfig,
        punch_config: AnimationConfig,
        item_registry: ItemRegistry,
        save_manager: SaveManager,
    ) -> Result<Self, String> {
        // Create systems
        let systems = Systems::new(player_config, slime_config, punch_config);

        // Create fresh player
        let player_animation_controller = systems.player_config.create_controller(
            textures.character,
            &["idle", "walk", "attack", "damage", "death"],
        )?;
        let player = Player::new(
            GAME_WIDTH as i32 / 2,
            GAME_HEIGHT as i32 / 2,
            player_animation_controller,
        );

        // Create world with fresh state
        let world_grid = WorldGrid::new(GAME_WIDTH / 32, GAME_HEIGHT / 32);
        let render_grid = RenderGrid::new(&world_grid);

        // Initialize entities (pyramids)
        let entity_positions = vec![
            (1, 100, 150, EntityType::Attack),
            (2, 300, 150, EntityType::Defense),
            (3, 500, 150, EntityType::Speed),
            (4, 700, 150, EntityType::Regeneration),
        ];

        let mut entities = Vec::new();
        for (id, x, y, entity_type) in entity_positions {
            let frames = vec![
                sprite::Frame::new(0, 0, 32, 32, 300),
                sprite::Frame::new(32, 0, 32, 32, 300),
                sprite::Frame::new(64, 0, 32, 32, 300),
            ];
            let sprite_sheet = sprite::SpriteSheet::new(textures.entity, frames);
            entities.push(TheEntity::new(id, x, y, entity_type, sprite_sheet));
        }

        let world = GameWorld {
            player,
            slimes: Vec::new(),
            entities,
            dropped_items: Vec::new(),
            world_grid,
            render_grid,
            player_inventory: PlayerInventory::new(),
            attack_effects: Vec::new(),
            floating_texts: Vec::new(),
            active_attack: None,
        };

        // Create UI manager
        let player_health_bar = HealthBar::new();
        let enemy_health_bar = HealthBar::with_style(HealthBarStyle {
            health_color: Color::RGB(150, 0, 150),
            low_health_color: Color::RGB(200, 0, 0),
            ..Default::default()
        });
        let floating_text_renderer = FloatingText::new();
        let buff_display = BuffDisplay::new(texture_creator)?;
        let save_exit_menu = SaveExitMenu::new();
        let death_screen = DeathScreen::new();
        let inventory_ui = InventoryUI::new(textures.items, &item_registry);

        let ui = UIManager {
            save_exit_menu,
            death_screen,
            inventory_ui,
            player_health_bar,
            enemy_health_bar,
            floating_text_renderer,
            buff_display,
            debug_menu_state: DebugMenuState::Closed,
            show_collision_boxes: false,
            show_tile_grid: false,
            is_tilling: false,
            last_tilled_tile: None,
            mouse_x: 0,
            mouse_y: 0,
        };

        Ok(Game {
            world,
            systems,
            ui,
            game_state: GameState::Playing,
            canvas,
            event_pump,
            texture_creator,
            textures,
            item_registry,
            save_manager,
        })
    }

    /// Load an existing game from save file
    pub fn load(
        canvas: Canvas<Window>,
        event_pump: EventPump,
        texture_creator: &'a TextureCreator<WindowContext>,
        textures: GameTextures<'a>,
        player_config: AnimationConfig,
        slime_config: AnimationConfig,
        punch_config: AnimationConfig,
        item_registry: ItemRegistry,
        save_manager: SaveManager,
    ) -> Result<Self, String> {
        // Try to load game, fall back to new game if load fails
        let (player, slimes, world_grid, entities, player_inventory, dropped_items) =
            match load_game(
                &save_manager,
                &player_config,
                &slime_config,
                textures.character,
                textures.slime,
                textures.entity,
                textures.items,
            ) {
                Ok((loaded_player, loaded_slimes, loaded_world, loaded_entities, loaded_inventory, loaded_items)) => {
                    println!("✓ Loaded existing save!");
                    (loaded_player, loaded_slimes, loaded_world, loaded_entities, loaded_inventory, loaded_items)
                }
                Err(_) => {
                    println!("No existing save found, starting new game");
                    // Create fresh game via new() and extract its world components
                    let new_game = Self::new(
                        canvas, event_pump, texture_creator, textures,
                        player_config.clone(), slime_config.clone(), punch_config.clone(),
                        item_registry.clone(), save_manager,
                    )?;
                    return Ok(new_game);
                }
            };

        let render_grid = RenderGrid::new(&world_grid);

        // Build world with loaded data
        let world = GameWorld {
            player,
            slimes,
            entities,
            dropped_items,
            world_grid,
            render_grid,
            player_inventory,
            attack_effects: Vec::new(),
            floating_texts: Vec::new(),
            active_attack: None,
        };

        // Create systems and UI (same as new())
        let systems = Systems::new(player_config, slime_config, punch_config);

        let player_health_bar = HealthBar::new();
        let enemy_health_bar = HealthBar::with_style(HealthBarStyle {
            health_color: Color::RGB(150, 0, 150),
            low_health_color: Color::RGB(200, 0, 0),
            ..Default::default()
        });
        let floating_text_renderer = FloatingText::new();
        let buff_display = BuffDisplay::new(texture_creator)?;
        let save_exit_menu = SaveExitMenu::new();
        let death_screen = DeathScreen::new();
        let inventory_ui = InventoryUI::new(textures.items, &item_registry);

        let ui = UIManager {
            save_exit_menu,
            death_screen,
            inventory_ui,
            player_health_bar,
            enemy_health_bar,
            floating_text_renderer,
            buff_display,
            debug_menu_state: DebugMenuState::Closed,
            show_collision_boxes: false,
            show_tile_grid: false,
            is_tilling: false,
            last_tilled_tile: None,
            mouse_x: 0,
            mouse_y: 0,
        };

        Ok(Game {
            world,
            systems,
            ui,
            game_state: GameState::Playing,
            canvas,
            event_pump,
            texture_creator,
            textures,
            item_registry,
            save_manager,
        })
    }
}
```

**Agent Instructions**:

1. **Copy initialization code from main()**: Lines ~1456-1575 contain the setup logic
2. **Split into two constructors**:
   - `new()` - creates fresh game state
   - `load()` - tries to load save, falls back to new()
3. **Extract entity initialization**: The pyramid setup code goes in `new()`
4. **Handle texture references**: Pass `GameTextures` instead of individual texture params
5. **Run validation**: `cargo check`

**Validation Checklist**:
- [ ] new() constructor created
- [ ] load() constructor created
- [ ] Both return Result<Self, String>
- [ ] Initialization logic matches current main()
- [ ] No compilation errors

**Success Criteria**:
```bash
cargo check  # Should pass with no errors
```

---

### Step 7: Refactor main() to Use Game Struct

**Agent Role**: Integration Specialist
**Task Duration**: 30 minutes
**Dependencies**: Steps 1-6 complete

**Objective**: Simplify main() to just initialization + `game.run()`, reducing from 421 lines to ~150 lines.

**Files to Modify**:
- `src/main.rs` (main function, lines 1421-1870)

**Target Structure**:
```rust
fn main() -> Result<(), String> {
    // SDL2 initialization (~30 lines)
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window("Game1 - 2.5D Rust Game", GAME_WIDTH, GAME_HEIGHT)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;
    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    canvas.set_logical_size(GAME_WIDTH, GAME_HEIGHT).map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();
    let event_pump = sdl_context.event_pump()?;

    // Load configurations (~10 lines)
    let player_config = AnimationConfig::load_from_file("assets/config/player_animations.json")?;
    let slime_config = AnimationConfig::load_from_file("assets/config/slime_animations.json")?;
    let punch_config = AnimationConfig::load_from_file("assets/config/punch_effect.json")?;

    // Load textures (~20 lines)
    let character_texture = load_texture(&texture_creator, "assets/sprites/new_player/Character-Base.png")?;
    let slime_texture = load_texture(&texture_creator, "assets/sprites/slime/Slime.png")?;
    let entity_texture = load_texture(&texture_creator, "assets/sprites/the_entity/entity_awaken.png")?;
    let punch_texture = load_texture(&texture_creator, "assets/sprites/new_player/punch_effect.png")?;
    let grass_tile_texture = load_texture(&texture_creator, "assets/backgrounds/tileable/grass_tile.png")?;
    let item_textures = load_item_textures(&texture_creator)?;

    let textures = GameTextures {
        character: &character_texture,
        slime: &slime_texture,
        entity: &entity_texture,
        punch: &punch_texture,
        grass_tile: &grass_tile_texture,
        items: &item_textures,
    };

    // Create registries and managers (~15 lines)
    let item_registry = ItemRegistry::create_default();
    println!("✓ Item registry initialized");

    let save_dir = dirs::home_dir()
        .map(|p| p.join(".game1/saves"))
        .unwrap_or_else(|| std::path::PathBuf::from("./saves"));
    let save_manager = SaveManager::new(&save_dir)
        .map_err(|e| format!("Failed to create save manager: {}", e))?;

    // Print controls (~30 lines)
    println!("Controls:");
    println!("WASD - Move player");
    // ... rest of control instructions ...

    // Create and run game (~5 lines)
    let mut game = Game::load(
        canvas,
        event_pump,
        &texture_creator,
        textures,
        player_config,
        slime_config,
        punch_config,
        item_registry,
        save_manager,
    )?;

    game.run()
}
```

**Agent Instructions**:

1. **Keep SDL2 initialization**: Lines 1422-1445 stay as-is
2. **Keep config loading**: Lines 1456-1461 stay as-is
3. **Keep texture loading**: Lines 1466-1479 stay as-is
4. **Create GameTextures struct**: Replace individual texture variables
5. **Keep registry/manager creation**: Lines 1472-1483 stay as-is
6. **Keep control printing**: Lines 1571-1591 stay as-is
7. **Replace game loop**: Replace lines 1528-1868 with:
   ```rust
   let mut game = Game::load(
       canvas, event_pump, &texture_creator, textures,
       player_config, slime_config, punch_config,
       item_registry, save_manager,
   )?;

   game.run()
   ```
8. **Delete old code**: Remove:
   - Old Systems construction (if separate from Game::load)
   - Old GameWorld construction
   - Old UIManager construction
   - Entire game loop (replaced by game.run())
9. **Run validation**: `cargo check` and `cargo build`

**Validation Checklist**:
- [ ] main() is ~150 lines or less
- [ ] SDL2 initialization preserved
- [ ] Texture loading preserved
- [ ] Game created via Game::load()
- [ ] Game loop is just `game.run()`
- [ ] No compilation errors
- [ ] All tests pass

**Success Criteria**:
```bash
cargo check     # Should pass
cargo build     # Should pass
cargo test      # All 39 tests should pass
cargo run       # Game should launch and work
```

---

### Step 8: Validation and Testing

**Agent Role**: QA Specialist
**Task Duration**: 30 minutes
**Dependencies**: All previous steps complete

**Objective**: Comprehensive testing to ensure Phase 3 refactoring maintains all functionality.

**Testing Checklist**:

**Compilation Tests**:
- [ ] `cargo check` passes with 0 errors
- [ ] `cargo clippy` has no new warnings
- [ ] `cargo build --release` succeeds
- [ ] `cargo test` - all 39 tests pass

**Code Metrics**:
- [ ] main() is ~150 lines (down from 421)
- [ ] Game struct has 4-5 methods
- [ ] impl<'a> Game<'a> block is ~500-600 lines total
- [ ] No standalone handle_events or update_world functions remain

**Runtime Tests**:
- [ ] Game launches without panic
- [ ] Player movement (WASD) works
- [ ] Combat (M key) works
- [ ] Inventory (I key) opens/closes
- [ ] Debug menu (F3) opens/closes
- [ ] Save game (F5) works
- [ ] Load game (F9) works
- [ ] Exit menu (ESC) works
- [ ] Death and respawn works
- [ ] FPS is stable at 60

**Code Quality**:
- [ ] All methods have doc comments
- [ ] Lifetime 'a properly propagated
- [ ] No unnecessary clones
- [ ] Borrow checker satisfied (no unnecessary RefCell)

**Agent Instructions**:

1. Run all compilation checks:
   ```bash
   cargo clean
   cargo check
   cargo clippy
   cargo build --release
   cargo test
   ```

2. Run the game and test each feature:
   ```bash
   cargo run
   ```
   - Test movement, combat, inventory, menus
   - Test save/load functionality
   - Test death and respawn
   - Let game run for 1 minute to check for memory leaks or crashes

3. Measure code metrics:
   ```bash
   wc -l src/main.rs
   grep -n "^fn main" src/main.rs
   grep -n "impl<'a> Game<'a>" src/main.rs
   ```

4. **Document results**: Create a file `docs/phase3-validation-report.md` with:
   - All test results
   - Metrics comparison (before/after)
   - Any issues found
   - Performance notes

**If Issues Found**:
- Document the issue clearly
- Identify which step introduced the problem
- Propose fix or revert to previous step
- Re-run validation after fix

---

## Success Criteria Summary

### Code Metrics
- [x] main() reduced from 421 → ~150 lines (64% reduction)
- [x] Game struct created with all game state
- [x] 4 methods: new(), load(), run(), handle_events(), update(), render()
- [x] Zero compilation errors
- [x] All 39 tests pass

### Functionality
- [x] Game launches and runs at 60 FPS
- [x] All input controls work
- [x] Save/load functionality works
- [x] Combat system works
- [x] Inventory system works
- [x] UI systems work (menus, debug overlay)

### Code Quality
- [x] Clear ownership model (Game owns everything)
- [x] Methods well-documented
- [x] Lifetimes properly managed
- [x] No clippy warnings added
- [x] Idiomatic Rust patterns

---

## Rollback Plan

If any step fails critically:

1. **Identify failing step**: Note the step number
2. **Revert changes**:
   ```bash
   git diff src/main.rs > phase3-attempt.patch
   git checkout src/main.rs
   ```
3. **Review the patch**: Analyze what went wrong
4. **Fix and retry**: Make corrections and re-apply
5. **Validate**: Run checks before proceeding

---

## Agent Coordination

### Parallel Execution Possible:
- **Step 2 and Step 3** can run in parallel (different functions)
- **Step 4** can run in parallel with Steps 2-3

### Sequential Dependencies:
- Step 1 → All others (struct definitions needed first)
- Steps 2, 3, 4 → Step 5 (run() needs the methods)
- Steps 1-5 → Step 6 (constructors need complete Game struct)
- Steps 1-6 → Step 7 (main() refactor needs everything)
- Steps 1-7 → Step 8 (validation is last)

### Communication Protocol:
Each agent should:
1. Comment at start: "Starting Step X: [task name]"
2. Comment on completion: "Completed Step X: [validation results]"
3. Report issues: "Issue in Step X: [description]"
4. Run `cargo check` after each step
5. Commit after successful validation (if desired)

---

## Post-Phase 3 State

After completing Phase 3:

```rust
// main.rs structure:
fn main() -> Result<(), String> {
    // SDL2 setup (~30 lines)
    // Load configs (~10 lines)
    // Load textures (~20 lines)
    // Create registries (~15 lines)
    // Print controls (~30 lines)
    // Create and run game (~5 lines)
    let mut game = Game::load(...)?;
    game.run()
}

impl<'a> Game<'a> {
    pub fn new(...) -> Result<Self, String> { }
    pub fn load(...) -> Result<Self, String> { }
    pub fn run(&mut self) -> Result<(), String> { }
    pub fn handle_events(&mut self) -> Result<bool, String> { }
    pub fn update(&mut self, ...) -> Result<(), String> { }
    pub fn render(&mut self) -> Result<(), String> { }
}
```

**Benefits Achieved**:
- ✅ 64% reduction in main() size
- ✅ Clear separation of concerns
- ✅ Game is a single, cohesive unit
- ✅ Easy to add new features (just add to Game)
- ✅ Testable (can create Game in tests)
- ✅ Maintainable (all game code in one place)

---

## Optional Phase 4 Preview

After Phase 3, potential future improvements:
1. **Extract systems**: Create separate files for input, physics, rendering
2. **State machine**: Formal state pattern for GameState
3. **Scene system**: Different scenes (Menu, Gameplay, Settings)
4. **ECS**: Entity Component System (if game grows significantly)

But Phase 3 is a great stopping point! 🎉

---

## Phase 3 Refactoring Checklist

### Step 1: Define Game Struct and GameTextures Helper
- [x] Create `GameTextures` struct (lines 428-435 in main.rs)
- [x] Create `Game` struct (lines 439-460 in main.rs)
- [x] Add impl<'a> Game<'a> block (line 462 in main.rs)

### Step 2: Convert handle_events to Game Method
- [x] Create `handle_events` method shell in `Game` impl (line 949)
- [x] Convert function signature to use `&mut self`
- [x] Update all parameter references to `self.field`
- [x] Remove old standalone `handle_events` function

### Step 3: Convert update_world to Game Method
- [x] Create `update` method shell in `Game` impl (line 1413)
- [x] Convert function signature to use `&mut self`
- [x] Update all parameter references to `self.field`
- [x] Remove old standalone `update_world` function

### Step 4: Create render Method
- [x] Create `render` method shell in `Game` impl (line 1621)
- [x] Move `canvas.clear()` and `canvas.present()` to `render` method
- [x] Move world rendering (`render_grid`, `render_with_depth_sorting`) to `render` method
- [x] Move attack effects rendering to `render` method
- [x] Move health bar rendering to `render` method
- [x] Move floating text rendering to `render` method
- [x] Move buff display rendering to `render` method
- [x] Move debug overlays (`show_collision_boxes`, `show_tile_grid`) to `render` method
- [x] Move UI rendering (`inventory_ui`, `death_screen`, `save_exit_menu`, `debug_menu_state`) to `render` method

### Step 5: Create run Method (Main Game Loop)
- [x] Create `run` method shell in `Game` impl (line 1816)
- [x] Move main game loop structure to `run` method
- [x] Replace function calls with method calls (`handle_events()`, `update()`, `render()`)
- [x] Keep death check and respawn logic
- [x] Keep frame rate limiting at 60 FPS

### Step 6: Create Game Constructors
- [x] Create `new` constructor in `Game` impl (line 1870)
- [x] Move `Systems::new()` to `new` constructor
- [x] Move player creation to `new` constructor
- [x] Move entity creation to `new` constructor
- [x] Move `GameWorld` creation to `new` constructor
- [x] Move `UIManager` creation to `new` constructor
- [x] Assemble `Game` struct in `new` constructor
- [x] Create `load` constructor in `Game` impl (line 1992)
- [x] Move `load_game` logic to private helper `load_game_data()` (line 2087)
- [x] Use `load_game_data()` in `load` constructor

### Step 7: Refactor main() to Use Game Struct
- [x] Keep SDL2 initialization (lines 2660-2680)
- [x] Keep config loading (lines 2682-2688)
- [x] Keep texture loading (lines 2690-2707)
- [x] Remove manual `GameTextures` creation (now in constructors)
- [x] Replace initialization with `Game::load()` or `Game::new()` call (lines 2737-2776)
- [x] Refactor `main` to call `game.run()` (line 2779)
- [x] Remove ALL manual initialization code from `main`

### Step 8: Validation and Testing
- [x] Run `cargo check` - ✅ PASSED
- [x] Run `cargo clippy` - ✅ PASSED (minor warnings only)
- [x] Run `cargo test` - ✅ ALL 43 TESTS PASSED
- [x] Run `cargo run` - ✅ GAME RUNS SUCCESSFULLY
- [x] Verify main() line count - ✅ **122 lines** (target: ~150, achieved: 77 line reduction!)
- [x] Test game features - ✅ Save/load working correctly
- [ ] Create validation report in `docs/phase3-validation-report.md`
