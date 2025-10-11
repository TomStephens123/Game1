# AI Prompt: Game World Manager Design

## Context

You are designing a **Game World Manager** (or **Scene Manager**) for Game1, a 2.5D Rust game. Currently, the game's world state is **scattered across 15+ separate variables** in main.rs, making it nearly impossible to reason about game state, implement save/load properly, or pass data between systems cleanly.

## Project Background

**Technology Stack:**
- Rust (learning-focused - teach ownership and borrowing patterns)
- SDL2 for rendering
- Custom entity system (no ECS framework - intentionally simpler)
- Anchor-based positioning (all entities have bottom-center anchor)

**Existing Patterns:**
- Entities follow standard pattern (see `docs/patterns/entity-pattern.md`)
- Save system uses `Saveable` trait (see `docs/systems/save-system-design.md`)
- Depth sorting for 2.5D rendering (see `docs/systems/depth-sorting-render-system.md`)
- Collision system with `Collidable` trait (see `docs/systems/collision_implementation_plan.md`)

## The Problem

**Current State in main.rs (lines 622-727):**
```rust
// Game state is an explosion of separate variables:
let mut player: Player = ...;
let mut slimes: Vec<Slime> = Vec::new();
let mut entities: Vec<TheEntity> = Vec::new();
let mut dropped_items: Vec<DroppedItem> = Vec::new();
let mut world_grid: WorldGrid = ...;
let mut render_grid: RenderGrid = ...;
let mut player_inventory: PlayerInventory = ...;
let mut attack_effects: Vec<AttackEffect> = Vec::new();
let mut floating_texts: Vec<FloatingTextInstance> = Vec::new();
let mut game_state: GameState = GameState::Playing;
let mut debug_menu_state: DebugMenuState = ...;
let mut debug_config: DebugConfig = ...;
let mut save_exit_menu: SaveExitMenu = ...;
let mut death_screen: DeathScreen = ...;
let mut inventory_ui: InventoryUI = ...;
// ... and more!
```

**Pain Points:**
1. **15+ function parameters** - `save_game()` and `load_game()` take 6-7 parameters each
2. **No encapsulation** - Any code can mutate any entity at any time
3. **Hard to reason about** - What's the complete game state? Scroll through 1000 lines
4. **Difficult save/load** - Must manually list every entity type in save functions
5. **No entity lifecycle** - Can't easily add "on spawn" or "on despawn" hooks
6. **Messy collision** - Must maintain separate `all_static_collidables` vector (line 1479)
7. **Poor abstraction** - No clear boundary between "world" and "game engine"

**Code Smells:**
- main.rs:318-417 - `save_game()` function manually iterates every entity type
- main.rs:183-315 - `load_game()` function has massive match statement for entity types
- main.rs:1479-1485 - Manually building `all_static_collidables` vector every frame
- main.rs:1368-1447 - Collision handling accesses entities directly
- main.rs:1269-1276 - Entity updates scattered across game loop

## Your Task

Design a **unified GameWorld / WorldManager** that:
1. **Encapsulates** all game entities and world state in one place
2. **Simplifies** save/load by having a single "save the world" operation
3. **Provides** clean APIs for entity lifecycle (spawn, update, query, despawn)
4. **Integrates** with existing collision, rendering, and save systems
5. **Enables** future features (entity queries, spatial partitioning, etc.)
6. **Maintains** performance (no unnecessary allocations or copying)

## Requirements

### Must Have
- [ ] Single struct containing all world entities (`GameWorld` or `World`)
- [ ] Player management (there's always exactly 1 player)
- [ ] Enemy management (slimes, future enemy types)
- [ ] Item management (dropped items in world)
- [ ] Static entity management (TheEntity pyramid things)
- [ ] World grid (tiles/terrain)
- [ ] Visual effects (attack effects, floating text)
- [ ] Integration with existing `Saveable` trait
- [ ] Methods: `update(delta_time)`, `query_entities()`, `get_player()`, etc.
- [ ] Support for existing anchor-based positioning system

### Should Have
- [ ] Entity spawning API (`spawn_slime()`, `spawn_dropped_item()`, etc.)
- [ ] Entity despawning API (mark for removal, cleanup at end of frame)
- [ ] Query API (get all collidables, get entities near point, etc.)
- [ ] Clear ownership model (who can access what)
- [ ] Separation of "persistent world state" vs "transient effects"

### Nice to Have (Don't Add if Not Needed)
- [ ] Entity IDs/handles (for references without lifetimes)
- [ ] Spatial partitioning (quadtree/grid) for collision queries
- [ ] Component groups (all Collidable, all Renderable)
- [ ] Event system integration (entity spawned → broadcast event)

### Must NOT Have (Premature Optimization)
- ❌ Full ECS (Entity Component System) - too complex for this project
- ❌ Generational indexes/handles - only add if lifetime issues arise
- ❌ Serialization framework - work with existing Saveable trait
- ❌ Scripting system for entity behavior
- ❌ Multiple worlds/scenes at once

## Design Constraints

**Rust Learning Goals:**
This design should teach:
- Struct composition and encapsulation
- Borrowing (how to pass `&mut World` around safely)
- Trait objects (if needed for heterogeneous collections)
- The `retain()` pattern for removing dead entities
- Interior mutability (if needed - but prefer to avoid)

**Integration Points:**
- **Saving**: Must implement or work with `Saveable` trait (see `docs/systems/save-system-design.md`)
- **Rendering**: Must provide entities to `render_with_depth_sorting()` (see `src/render.rs`)
- **Collision**: Must provide entities to collision system (see `src/collision.rs`)
- **UI Systems**: Inventory, menus, etc. currently live outside world (should they stay there?)

**Current Entity Types:**
From analyzing main.rs, here are all the "things" in the world:
1. **Player** - Exactly 1, lives throughout game (respawns on death)
2. **Slimes** - Vec<Slime>, enemies that spawn/die
3. **TheEntity** - Vec<TheEntity>, static pyramid objects (4 types: Attack/Defense/Speed/Regen)
4. **DroppedItems** - Vec<DroppedItem>, items on ground
5. **AttackEffects** - Vec<AttackEffect>, temporary visual effects
6. **FloatingText** - Vec<FloatingTextInstance>, damage numbers, etc.
7. **WorldGrid** - Tile data (grass, dirt)
8. **RenderGrid** - Cached rendering data for tiles

**Special Cases:**
- Player inventory is part of Player or separate? (currently separate in main.rs)
- UI state (debug menu, death screen) - part of world or separate?
- Static boundaries (collision walls) - hardcoded in main.rs:681-687

## Suggested Architecture

Consider these approaches (choose or combine):

### Approach 1: Flat Collections
```rust
pub struct GameWorld<'a> {
    pub player: Player<'a>,
    pub slimes: Vec<Slime<'a>>,
    pub entities: Vec<TheEntity<'a>>,
    pub dropped_items: Vec<DroppedItem<'a>>,
    pub attack_effects: Vec<AttackEffect<'a>>,
    pub floating_texts: Vec<FloatingTextInstance>,
    pub world_grid: WorldGrid,
    pub render_grid: RenderGrid,
}
```
**Pros**: Simple, clear ownership, easy to migrate to
**Cons**: Still need to update each Vec separately

### Approach 2: Layered (Persistent + Transient)
```rust
pub struct GameWorld<'a> {
    pub persistent: PersistentState<'a>,  // Things that save
    pub transient: TransientState<'a>,    // Visual effects, etc.
}

pub struct PersistentState<'a> {
    pub player: Player<'a>,
    pub enemies: Vec<Box<dyn Enemy + 'a>>,  // Trait object for multiple enemy types
    pub world_entities: Vec<TheEntity<'a>>,
    pub dropped_items: Vec<DroppedItem<'a>>,
    pub terrain: WorldGrid,
}
```
**Pros**: Clear what saves, easier to implement save/load
**Cons**: More abstraction, trait objects have runtime cost

### Approach 3: Grouped by Trait
```rust
pub struct GameWorld<'a> {
    pub player: Player<'a>,
    pub collidables: Vec<Box<dyn Collidable + 'a>>,
    pub renderables: Vec<Box<dyn Renderable + 'a>>,
    pub updateables: Vec<Box<dyn Updateable + 'a>>,
    pub world_grid: WorldGrid,
}
```
**Pros**: Systems-oriented, easy to iterate by capability
**Cons**: Complex ownership (entity in multiple vecs?), harder to spawn/despawn

## Specific Problems to Solve

**Problem 1: Entity Lifetimes**
How do entities get cleaned up? Current approach:
```rust
slimes.retain(|slime| slime.is_alive);  // main.rs:1447
```
Should GameWorld handle this? When?

**Problem 2: Multiple Borrows**
If rendering needs `&World` and collision needs `&mut World`, how to handle? Consider:
- Separate update vs render phases
- Interior mutability (RefCell/Cell)
- Split borrows (borrow individual fields)

**Problem 3: Heterogeneous Collections**
Currently have separate Vec for each entity type. Alternatives:
- Keep separate (simple, but verbose)
- Use trait objects `Vec<Box<dyn Entity>>` (dynamic dispatch)
- Use enum `Vec<EntityType>` (no heap allocation)

**Problem 4: Save/Load Integration**
Current save/load manually constructs entity vectors:
```rust
for (i, slime) in slimes.iter().enumerate() {
    entities_vec.push(EntitySaveData { ... });
}
```
How can GameWorld make this cleaner?

**Problem 5: Player Special Case**
Player is unique (exactly 1, never despawns, just respawns). Should it:
- Live in `Option<Player>` (can be None when dead)?
- Live as `Player` with internal is_dead state (current approach)?
- Live separate from other entities (current approach)?

## Expected Deliverables

Provide a detailed design document including:

1. **Architecture Overview**
   - Module structure (`src/world.rs`? `src/world/mod.rs` with submodules?)
   - Core struct definition (GameWorld or equivalent)
   - Ownership model diagram
   - Lifetime strategy (<'a> for textures, 'static where possible)

2. **API Design**
   - Public methods on GameWorld
   - How to spawn entities
   - How to query entities (get all collidables, get enemies in radius, etc.)
   - How to update entities
   - Code examples showing usage from main.rs

3. **Migration Strategy**
   - How to incrementally move from current loose variables
   - Phase 1: Wrap existing Vecs in struct (no logic change)
   - Phase 2: Add methods, move logic into World
   - Phase 3: Improve APIs based on usage
   - Specific line ranges in main.rs to refactor first

4. **Save/Load Integration**
   - How does `Saveable` trait work with GameWorld?
   - Should GameWorld implement Saveable?
   - Show before/after of save_game() function

5. **System Integration**
   - How collision system queries world
   - How render system gets renderables
   - How input system spawns entities (e.g., right-click spawn slime)
   - How UI systems access inventory/player state

6. **Rust Patterns Explained**
   - Why these specific ownership choices?
   - When to use `&World` vs `&mut World`?
   - Lifetime management strategy
   - Error handling (Result vs unwrap)

## Success Criteria

Your design is successful if:
- ✅ `save_game()` and `load_game()` functions reduced by 50%+ lines
- ✅ Adding a new entity type requires changes in < 5 places
- ✅ Main.rs game loop becomes more readable (clear phases)
- ✅ No performance regression (entity updates still < 1ms)
- ✅ Clearer ownership (obvious who owns what)
- ✅ Future features easier (spatial queries, entity events, etc.)
- ✅ Code is more maintainable (can understand world state at a glance)

## Anti-Patterns to Avoid

- ❌ Don't create a god-object (World that does everything)
- ❌ Don't prematurely optimize (no cache-friendly memory layouts yet)
- ❌ Don't over-abstract (Vec<Slime> is fine, don't need Vec<Box<dyn Enemy>>)
- ❌ Don't fight Rust's borrow checker (if design requires RefCell everywhere, rethink)
- ❌ Don't break existing systems (collision, rendering, save must still work)
- ❌ Don't force ECS patterns (this isn't Bevy)

## References

Study these files for context:
- `src/main.rs:622-727` - Current entity initialization
- `src/main.rs:183-315` - load_game() function (target for simplification)
- `src/main.rs:318-417` - save_game() function (target for simplification)
- `src/main.rs:1216-1351` - Game update loop (logic to move into World)
- `src/render.rs` - render_with_depth_sorting() needs entity access
- `src/collision.rs` - Collision system queries entities
- `docs/patterns/entity-pattern.md` - Standard entity structure
- `docs/systems/save-system-design.md` - Saveable trait documentation
- `docs/systems/depth-sorting-render-system.md` - Rendering requirements

## Questions to Answer

As you design, explicitly address:
1. Should UI state (menus, inventory UI) be part of GameWorld or separate?
2. How to handle player inventory - part of Player or part of World?
3. Should there be a clear "persistent" vs "transient" split?
4. How to handle entity spawning mid-update (e.g., slime dies → drop item)?
5. What's the API for systems that need to query world state?
6. Should GameWorld own the texture references, or just entity data?
7. How to avoid the "everything is pub" trap?

## Final Note

Remember: This is a **refactoring for clarity**, not a rewrite. Your design should:
- **Simplify** the current codebase without breaking it
- **Reduce** the number of parameters passed around
- **Clarify** ownership and entity lifecycle
- **Enable** future features without over-engineering

The goal is **obvious correctness** - someone reading main.rs should immediately understand what state exists and how to access it.

Think of this as building a "game state container" not a "game engine framework". Keep it focused on this project's needs.

Good luck! 🌍
