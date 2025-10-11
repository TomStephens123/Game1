# AI Prompt: Main.rs Refactoring Architecture

## Context

You are designing the **high-level architecture** to refactor Game1's main.rs, currently a **monolithic 1,130-line function** that's nearly impossible to maintain. This is the **orchestration layer** that ties together the other system designs (Input, World, Resources, Physics, Events).

## Project Background

**Technology Stack:**
- Rust (learning-focused - teach modular design, separation of concerns)
- SDL2 for windowing, rendering, input
- Custom game systems (no engine framework)
- Fixed 60 FPS game loop

**Related System Designs:**
Before reading this, review these companion designs:
- `agents/input-system-design.md` - Input handling
- `agents/game-world-manager-design.md` - World/entity management
- `agents/resource-manager-design.md` - Asset/texture management
- `agents/physics-system-design.md` - Movement and physics
- `agents/event-system-design.md` - Event messaging

**Existing Documentation:**
- `docs/patterns/entity-pattern.md` - Entity conventions
- `docs/systems/` - Individual system designs
- `src/` modules - Current modular structure (animation, collision, etc.)

## The Problem

**main() Function is 1,130 Lines (main.rs:567-1697)**

**Current Structure:**
```rust
fn main() -> Result<(), String> {
    // Lines 568-589: SDL2 initialization (20 lines)
    // Lines 591-621: Texture loading (30 lines)
    // Lines 622-661: Entity initialization OR load game (40 lines)
    // Lines 663-727: UI/debug/state initialization (65 lines)

    // Lines 728-1694: GAME LOOP (966 lines!!!)
    'running: loop {
        // Lines 740-1211: Event handling (470 lines!)
        // Lines 1214-1223: Player update (10 lines)
        // Lines 1225-1266: Combat resolution (40 lines)
        // Lines 1268-1282: Entity updates (15 lines)
        // Lines 1284-1350: Buff/modifier logic (65 lines)
        // Lines 1353-1360: Death screen logic (8 lines)
        // Lines 1362-1366: Attack effects (5 lines)
        // Lines 1368-1447: Collision handling (80 lines)
        // Lines 1449-1503: Item pickup & static collision (55 lines)
        // Lines 1505-1662: Rendering (160 lines)
        // Lines 1664-1689: UI rendering (25 lines)
        // Line 1693: Frame delay
    }
}
```

**Why This is Unsustainable:**
1. **Impossible to test** - Can't unit test game loop phases
2. **Impossible to understand** - 1,130 lines, deep nesting, mixed concerns
3. **Impossible to modify** - Changing one thing requires understanding everything
4. **Impossible to extend** - Adding features means more lines in same function
5. **No clear ownership** - 15+ mutable variables all accessible everywhere
6. **No clear flow** - Update/collision/render phases mixed together

**Specific Nightmares:**
- 470-line event handling match statement (lines 744-1211)
- Player update scattered in 3 places (input, velocity, collision)
- Collision response duplicated for dynamic vs static (lines 1368-1413, 1487-1503)
- 8 lines of item animation setup repeated 5+ times
- Rendering logic mixed with debug visualization (lines 1505-1662)

## Your Task

Design a **modular game architecture** that:
1. **Breaks down** the 1,130-line main() into manageable components
2. **Separates** initialization, game loop, and systems
3. **Integrates** the 5 other system designs (Input, World, Resources, Physics, Events)
4. **Maintains** existing functionality while improving structure
5. **Enables** future features (pause menu, level transitions, etc.)
6. **Teaches** Rust architecture patterns (composition, encapsulation, clear interfaces)

**Target:** Reduce main() to **~150-200 lines** showing high-level flow.

## Requirements

### Must Have
- [ ] GameEngine or Game struct that owns core systems
- [ ] Clear initialization phase (setup SDL2, load assets, create world)
- [ ] Clear game loop phases (input → update → render → present)
- [ ] Integration of InputSystem (handles events)
- [ ] Integration of GameWorld (manages entities)
- [ ] Integration of ResourceManager (manages assets)
- [ ] Integration with Physics (collision response)
- [ ] Integration with Events (messaging between systems)
- [ ] Separation of game state (Playing, Menu, Dead) from game loop
- [ ] Clean error handling (no unwrap() in main loop)

### Should Have
- [ ] System trait or common interface (if it helps structure)
- [ ] Update methods that take clear parameters (delta time, input state)
- [ ] Render methods separated from update
- [ ] State machine for game states (Playing, Menu, Paused, Dead)
- [ ] Debug systems separate from game systems (optional compilation)
- [ ] Clear "frame" abstraction (what happens each frame)

### Nice to Have (Don't Add if Not Needed)
- [ ] Plugin system for optional features
- [ ] Scene/Level abstraction (if multiple levels planned)
- [ ] Hot-reload support for development
- [ ] Profiler integration (frame timing)

### Must NOT Have (Premature Features)
- ❌ Full ECS (Entity Component System) - too complex
- ❌ Scripting language integration
- ❌ Network synchronization
- ❌ Multi-threaded parallelism (single-threaded game loop is fine)
- ❌ Complex dependency injection
- ❌ Reflection/introspection

## Design Constraints

**Rust Learning Goals:**
This design should teach:
- Struct composition (Game owns World, InputSystem, etc.)
- Ownership and borrowing (who owns what, who can mutate)
- Trait-based abstraction (if needed)
- Error propagation (Result throughout)
- Separation of concerns (each system has one job)

**Integration Constraints:**
Must integrate these systems (see their design files):
1. **InputSystem** - Handles SDL2 events, provides GameActions
2. **GameWorld** - Contains all entities, provides queries
3. **ResourceManager** - Provides textures/assets
4. **PhysicsSystem** - Handles collision response and movement
5. **EventSystem** - Messaging between systems

**Performance Requirements:**
- 60 FPS (16.6ms per frame)
- No performance regression from current code
- Minimal allocations in game loop

**Backward Compatibility:**
- Must maintain existing gameplay feel
- Must preserve save/load functionality
- Must keep existing entity behaviors

## Suggested Architecture

### Approach 1: Game Struct with Systems
```rust
pub struct Game<'a> {
    // SDL2
    canvas: Canvas<Window>,
    event_pump: EventPump,

    // Core systems
    resources: ResourceManager<'a>,
    input: InputSystem,
    world: GameWorld<'a>,
    physics: PhysicsSystem,
    events: EventQueue,

    // UI systems
    ui_manager: UIManager,

    // State
    game_state: GameState,
}

impl<'a> Game<'a> {
    pub fn new() -> Result<Self, String> { ... }
    pub fn run(&mut self) -> Result<(), String> {
        loop {
            self.update()?;
            self.render()?;
            self.present();
            self.delay_frame();
        }
    }

    fn update(&mut self) -> Result<(), String> {
        self.handle_input()?;
        self.update_world()?;
        self.resolve_physics()?;
        self.process_events()?;
        Ok(())
    }

    fn render(&mut self) -> Result<(), String> { ... }
}

fn main() -> Result<(), String> {
    let mut game = Game::new()?;
    game.run()
}
```

### Approach 2: Systems as Traits
```rust
pub trait System {
    fn update(&mut self, ctx: &mut GameContext) -> Result<(), String>;
}

pub struct GameEngine {
    systems: Vec<Box<dyn System>>,
    world: GameWorld,
    // ...
}

impl GameEngine {
    pub fn run(&mut self) {
        loop {
            for system in &mut self.systems {
                system.update(&mut self.context)?;
            }
            self.render()?;
        }
    }
}
```

### Approach 3: Phased Update Functions
```rust
pub struct Game { ... }

// Separate functions for each phase
fn handle_input(
    event_pump: &mut EventPump,
    input: &mut InputSystem,
    world: &mut GameWorld,
) -> Result<(), String> { ... }

fn update_world(
    world: &mut GameWorld,
    delta_time: f32,
) -> Result<(), String> { ... }

fn resolve_collisions(
    world: &mut GameWorld,
    physics: &PhysicsSystem,
) -> Result<(), String> { ... }

fn render_game(
    canvas: &mut Canvas<Window>,
    world: &GameWorld,
    resources: &ResourceManager,
) -> Result<(), String> { ... }

// Main loop becomes:
fn main() -> Result<(), String> {
    // Initialization...

    loop {
        handle_input(&mut event_pump, &mut input, &mut world)?;
        update_world(&mut world, delta_time)?;
        resolve_collisions(&mut world, &physics)?;
        render_game(&mut canvas, &world, &resources)?;
        canvas.present();
        std::thread::sleep(FRAME_DURATION);
    }
}
```

### Approach 4: State Machine
```rust
pub enum GameState {
    MainMenu(MainMenuState),
    Playing(PlayingState),
    Paused(PausedState),
    Dead(DeadState),
}

pub trait State {
    fn update(&mut self, engine: &mut Engine) -> Transition;
    fn render(&self, canvas: &mut Canvas) -> Result<(), String>;
}

pub struct Engine {
    state: GameState,
    // systems...
}

impl Engine {
    pub fn run(&mut self) {
        loop {
            let transition = self.state.update(self);
            match transition {
                Transition::None => {},
                Transition::Push(new_state) => { ... },
                Transition::Pop => { ... },
                Transition::Quit => break,
            }
            self.state.render(&mut self.canvas)?;
        }
    }
}
```

## Specific Problems to Solve

**Problem 1: Ownership of Systems**
Who owns what?
```rust
// Option A: Game owns everything
struct Game<'a> {
    world: GameWorld<'a>,
    input: InputSystem,
    // ...
}

// Option B: Systems own their data, Game coordinates
struct Game {
    systems: Systems,  // Systems struct with all systems
    // ...
}

// Option C: Flat in main, pass to functions
fn main() {
    let mut world = GameWorld::new();
    let mut input = InputSystem::new();
    loop {
        update(&mut world, &mut input);
    }
}
```

**Problem 2: Borrowing During Update**
```rust
// Problem: Multiple systems need &mut world
world.update();        // Needs &mut world
physics.update(&world); // Needs &world
events.process(&mut world); // Needs &mut world

// Can't borrow mutably and immutably at same time!
// Solution: ???
```

**Problem 3: Delta Time**
Current code uses fixed 60 FPS (1.0/60.0). How to handle:
- Variable delta time (for smoother gameplay)?
- Fixed timestep (for deterministic physics)?
- Time accumulator pattern?

**Problem 4: Game State Transitions**
Currently uses `GameState` enum (Playing, ExitMenu, Dead). How to handle:
- State-specific update logic (no player update when dead)
- State transitions (Playing → Dead → Playing)
- Nested states (Playing + Inventory open)

**Problem 5: System Dependencies**
Systems depend on each other:
- Input → World (spawn slime on right-click)
- World → Events (entity dies → emit event)
- Events → World (loot drop event → spawn item)
- Physics → World (collision response → modify positions)

How to structure to avoid circular dependencies?

**Problem 6: UI vs World**
UI systems (inventory, menus) currently separate from world. Should they:
- Be part of GameWorld?
- Be separate systems?
- Be part of a UIManager?

**Problem 7: Save/Load Integration**
Currently save/load are free functions (lines 183-417). Should they:
- Be methods on Game?
- Be part of SaveManager?
- Stay as free functions?

## Expected Deliverables

Provide a detailed design document including:

1. **Architecture Overview**
   - High-level structure diagram (Game → Systems → World → Entities)
   - Module organization (new files? restructure src/?)
   - Ownership model (who owns what)
   - Data flow (Input → Update → Events → Render)

2. **Core Structs/Types**
   - Game/GameEngine struct definition
   - How systems are organized
   - How state is managed (GameState enum?)
   - Lifetime strategy (<'a> propagation)

3. **Game Loop Design**
   - Initialization phase (SDL2 setup, asset loading, world creation)
   - Main loop phases (input, update, render)
   - Frame timing (fixed timestep, variable delta time?)
   - State transitions (Playing ↔ Menu ↔ Dead)

4. **System Integration**
   Show how each system integrates:
   - **InputSystem**: How it's called, what it returns
   - **GameWorld**: When it's updated, how it's queried
   - **ResourceManager**: How textures are accessed
   - **PhysicsSystem**: When collision response happens
   - **EventSystem**: When events are emitted/processed

5. **Code Examples**
   Show before/after:
   - **Before**: Current 1,130-line main()
   - **After**: New ~150-200 line main() with clear structure
   - Show a few key phases in detail (input handling, collision response)

6. **Migration Strategy**
   - Phase 1: Extract functions from main() (no new structs yet)
     - Extract event handling → `handle_events()` function
     - Extract rendering → `render_game()` function
     - Extract collision → `resolve_collisions()` function
   - Phase 2: Create Game struct, move functions to methods
   - Phase 3: Integrate new systems (InputSystem, GameWorld, etc.)
   - Phase 4: Remove old code, verify tests pass
   - Which parts to extract first (lowest risk, highest value)

7. **Error Handling Strategy**
   - How errors propagate (Result<(), String> everywhere?)
   - When to panic vs return error
   - User-facing error messages
   - Debug error messages

8. **Rust Patterns Explained**
   - Struct composition (Game owns systems)
   - Borrowing patterns (avoid RefCell if possible)
   - Trait usage (if any)
   - Error propagation (?)
   - Why these choices teach good Rust

## Success Criteria

Your design is successful if:
- ✅ main() function reduced to < 200 lines
- ✅ Each system has single responsibility
- ✅ Game loop phases are clear (input → update → render)
- ✅ No borrow checker fights (clean ownership)
- ✅ Systems can be tested independently
- ✅ Adding new feature requires touching < 3 files
- ✅ Code is self-documenting (obvious what each part does)
- ✅ No performance regression (still 60 FPS)
- ✅ Existing gameplay preserved (save/load works, game feels same)

## Anti-Patterns to Avoid

- ❌ Don't create god-objects (Game that does everything)
- ❌ Don't over-abstract (trait soup, generic over everything)
- ❌ Don't use RefCell unless absolutely necessary (indicates design flaw)
- ❌ Don't break the game during refactor (incremental migration)
- ❌ Don't prematurely optimize (keep it simple)
- ❌ Don't copy other engines' architecture (design for this game's needs)

## References

Study these files for context:
- `src/main.rs` - The entire 1,697-line file (the problem)
- `agents/input-system-design.md` - Input handling design
- `agents/game-world-manager-design.md` - World management design
- `agents/resource-manager-design.md` - Asset management design
- `agents/physics-system-design.md` - Physics/collision design
- `agents/event-system-design.md` - Event messaging design
- `docs/patterns/entity-pattern.md` - Entity conventions
- `docs/systems/` - Existing system documentation

## Questions to Answer

As you design, explicitly address:
1. Should Game be a struct or just organized functions?
2. Who owns SDL2 Canvas (Game, separate RenderSystem, main)?
3. How to avoid borrow checker issues with systems depending on each other?
4. Should there be a GameContext struct passed to systems?
5. How to structure initialization (builder pattern, constructor)?
6. Should systems have traits/common interface, or just similar methods?
7. How to handle frame timing (fixed timestep, variable delta time)?
8. Where does save/load functionality live?
9. How to transition between game states (Playing → Dead → Playing)?
10. How to integrate debug features (F3 menu, collision boxes)?

## Example: Target main() Structure

Show what the ideal main() would look like:

```rust
fn main() -> Result<(), String> {
    // ~20 lines: Initialization
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let window = create_window(&video_subsystem)?;
    let canvas = create_canvas(window)?;

    // ~30 lines: Create game systems
    let resources = ResourceManager::new(canvas.texture_creator())?;
    resources.load_all_assets()?;

    let input_system = InputSystem::new();
    let mut world = GameWorld::load_or_create(&resources)?;
    let physics = PhysicsSystem::new();
    let mut events = EventQueue::new();

    let mut game = Game {
        canvas,
        resources,
        input_system,
        world,
        physics,
        events,
    };

    // ~80-100 lines: Game loop
    let mut event_pump = sdl_context.event_pump()?;
    let mut last_frame = Instant::now();

    'running: loop {
        let delta_time = calculate_delta_time(&mut last_frame);

        // Input phase
        let actions = game.input_system.poll(&event_pump)?;
        if actions.contains(&GameAction::Quit) {
            break 'running;
        }

        // Update phase
        game.handle_actions(actions)?;
        game.world.update(delta_time)?;
        game.physics.resolve_collisions(&mut game.world)?;
        game.events.process(&mut game.world)?;

        // Render phase
        game.render()?;
        game.canvas.present();

        // Frame timing
        std::thread::sleep(FRAME_DURATION);
    }

    Ok(())
}
```

## Final Note

Remember: This is **the orchestration layer** that makes all other systems work together. Your design should:
- **Simplify** the current mess into understandable pieces
- **Integrate** the 5 other system designs cohesively
- **Enable** incremental migration (don't rewrite everything at once)
- **Teach** good Rust architecture (composition, clear ownership, separation of concerns)

The goal is **obvious correctness** - someone reading main() should immediately understand:
1. What systems exist
2. What each system does
3. How they interact
4. What happens each frame

Prioritize **clarity** and **maintainability** over cleverness. This is the foundation for all future development.

Think of this as building the "skeleton" that holds the game together. It should be strong, flexible, and obvious.

Good luck! 🏗️🎮
