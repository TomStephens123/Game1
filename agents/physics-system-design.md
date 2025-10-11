# AI Prompt: Physics System Design

## Context

You are designing a **Physics System** for Game1, a 2.5D Rust game. Currently, **physics logic is mixed with collision detection** throughout main.rs, making it hard to tune gameplay feel, debug movement issues, or add new physics interactions. The game needs separation between "what collides" (collision) and "how things move" (physics).

## Project Background

**Technology Stack:**
- Rust (learning-focused project)
- SDL2 for rendering (no built-in physics)
- Custom collision system (see `docs/systems/collision_implementation_plan.md`)
- 60 FPS fixed timestep
- 2.5D perspective (2D movement with depth sorting)

**Existing Patterns:**
- Entities use anchor positioning (see `docs/patterns/entity-pattern.md`)
- Collision uses AABB and Collidable trait (see `src/collision.rs`)
- Player has velocity (x_velocity, y_velocity)
- Slimes have simple jump behavior

**Current Movement Systems:**
1. **Player Movement** - Velocity-based with input (main.rs:1223)
2. **Slime Movement** - AI-controlled jumping pattern (slime.rs)
3. **Push Separation** - When entities overlap, push them apart (main.rs:1368-1413)
4. **Static Collision** - Push player away from walls (main.rs:1487-1503)

## The Problem

**Physics Logic Scattered Across Files:**

1. **Push-Apart Physics (main.rs:1368-1413)**
```rust
// When player and slime collide, push them apart
// Magic numbers: 3/10 (player), 7/10 (slime)
if overlap_x.abs() < overlap_y.abs() {
    player.apply_push(-overlap_x * 3 / 10, 0);
    slimes[slime_index].apply_push(overlap_x * 7 / 10, 0);
} else {
    player.apply_push(0, -overlap_y * 3 / 10);
    slimes[slime_index].apply_push(0, overlap_y * 7 / 10);
}
```

2. **Static Collision Push (main.rs:1487-1503)**
```rust
// When player hits wall, push 100% away
if overlap_x.abs() < overlap_y.abs() {
    player.apply_push(-overlap_x, 0);
} else {
    player.apply_push(0, -overlap_y);
}
```

3. **Player Update (player.rs)**
```rust
pub fn update(&mut self, keyboard_state: &KeyboardState) {
    // Velocity accumulation
    if keyboard_state.is_scancode_pressed(Scancode::W) {
        self.y_velocity -= self.get_speed();
    }
    // Apply velocity
    self.y += self.y_velocity as i32;
    self.x += self.x_velocity as i32;
    // Friction/decay
    self.x_velocity *= 0.8;
    self.y_velocity *= 0.8;
}
```

4. **Entity Push Method (player.rs, slime.rs)**
```rust
pub fn apply_push(&mut self, dx: i32, dy: i32) {
    self.x += dx;
    self.y += dy;
}
```

**Pain Points:**
1. **Magic numbers everywhere** - 3/10, 7/10, 0.8 friction - what do these mean?
2. **No mass concept** - Separation ratios hardcoded instead of mass-based
3. **Duplicate logic** - Push separation repeated for dynamic vs static collision
4. **Hard to tune** - Must find code in 3 different files to adjust "game feel"
5. **No forces** - Can't add knockback, wind, conveyor belts, etc.
6. **Inconsistent velocity** - Player has velocity, slimes have position-based jump
7. **No physics constants** - Friction, drag, bounce - all hardcoded

**Specific Code Smells:**
- main.rs:1377-1381 - Hardcoded 3/10 and 7/10 ratios (should be mass-based?)
- main.rs:1497-1501 - Same push logic duplicated for static collision
- player.rs:~150 - Friction constant 0.8 is magic number
- slime.rs:~80 - Jump movement is direct position modification, not velocity

## Your Task

Design a **Physics System** that:
1. **Separates** physics (movement, forces) from collision (detection, response)
2. **Centralizes** physics constants (friction, mass, separation ratios)
3. **Enables** tuning gameplay feel without code changes (config-driven)
4. **Supports** common physics interactions (push, knockback, drag)
5. **Integrates** with existing collision system cleanly
6. **Maintains** performance (60 FPS, no allocations in update loop)

## Requirements

### Must Have
- [ ] Separation of collision detection (collision.rs) and collision response (physics)
- [ ] Concept of "mass" or "weight" for entities (affects push separation)
- [ ] Centralized physics constants (friction, drag, separation behavior)
- [ ] Velocity-based movement (uniform across all entities)
- [ ] Push/force application system (replace direct position modification)
- [ ] Integration with existing `apply_push()` methods
- [ ] Support for different entity behaviors (player input, AI, static)

### Should Have
- [ ] Config-driven physics constants (JSON or constants file)
- [ ] Different friction values per surface type (future: ice, mud)
- [ ] Knockback/impulse forces (for combat, explosions)
- [ ] Clear separation of "physics body" from "game entity"
- [ ] Debug visualization for velocity/forces (optional but helpful)

### Nice to Have (Don't Add if Not Needed)
- [ ] Acceleration instead of direct velocity manipulation
- [ ] Gravity (if game needs falling/jumping - does it?)
- [ ] Bounce/restitution (if things should bounce off walls)
- [ ] Angular velocity/rotation (2D top-down, probably not needed)

### Must NOT Have (Premature Features)
- ❌ Full physics engine (Box2D, Rapier, etc.) - too complex
- ❌ Continuous collision detection (discrete is fine at 60 FPS)
- ❌ Rigid body dynamics (torque, angular momentum)
- ❌ Constraints/joints (no rope/chain physics)
- ❌ Soft body physics
- ❌ Fluid simulation

## Design Constraints

**Rust Learning Goals:**
This design should teach:
- Struct composition (Entity has PhysicsBody?)
- Trait-based behavior (Collidable vs Physical?)
- Constants and configuration
- Fixed-point vs floating-point math
- Separation of concerns (physics vs rendering vs logic)

**Integration Points:**
- **Collision System** - Already exists (src/collision.rs), provides overlap info
- **Entity Update** - Currently in entity.update(), move physics there?
- **Main Loop** - Currently handles collision response (main.rs:1368-1503)
- **Player Movement** - Input → velocity is special case (player.rs)

**Performance Requirements:**
- 60 FPS fixed timestep (16.6ms per frame)
- Physics update for ~50-100 entities
- No heap allocations in update loop
- Push-apart resolution in <0.1ms

**Current Entity Structure:**
```rust
// Player
pub struct Player<'a> {
    pub x: i32,           // Position
    pub y: i32,
    pub x_velocity: f32,  // Velocity (has it)
    pub y_velocity: f32,
    pub speed: i32,       // Movement speed
    // ... animation, stats, etc.
}

// Slime (no velocity currently)
pub struct Slime<'a> {
    pub x: i32,
    pub y: i32,
    // Velocity missing - jumps by modifying position directly
    // ... animation, health, etc.
}
```

## Suggested Architecture

Consider these approaches:

### Approach 1: Physics Component/Struct
```rust
pub struct PhysicsBody {
    pub mass: f32,          // For push separation ratios
    pub velocity: Vec2,     // Current velocity
    pub friction: f32,      // Deceleration rate (0.0-1.0)
    pub max_speed: f32,     // Speed cap
    pub is_static: bool,    // Immovable (walls, trees)
}

// Entities contain PhysicsBody:
pub struct Player<'a> {
    pub x: i32,
    pub y: i32,
    pub physics: PhysicsBody,
    // ...
}
```
**Pros**: Clean separation, data-driven
**Cons**: All entities need it (or Option<PhysicsBody>)

### Approach 2: Physics Trait
```rust
pub trait Physical {
    fn get_velocity(&self) -> (f32, f32);
    fn set_velocity(&mut self, vx: f32, vy: f32);
    fn get_mass(&self) -> f32;
    fn apply_force(&mut self, fx: f32, fy: f32);
}

impl Physical for Player { ... }
impl Physical for Slime { ... }
```
**Pros**: Flexible, entities control their own physics
**Cons**: Trait object overhead if storing `Vec<Box<dyn Physical>>`

### Approach 3: Centralized Physics System
```rust
pub struct PhysicsSystem {
    bodies: Vec<PhysicsBody>,  // Separate from entities
    config: PhysicsConfig,     // Constants
}

// Entities reference their physics by index?
pub struct Player {
    physics_id: usize,
    // ...
}
```
**Pros**: ECS-style separation
**Cons**: Complexity, indirection, lifetime management

### Approach 4: Keep Simple, Just Config
```rust
// No new structs, just extract constants
pub struct PhysicsConfig {
    pub player_mass: f32,
    pub slime_mass: f32,
    pub friction: f32,
    pub push_force_multiplier: f32,
}

// Use in collision response:
let ratio = entity1.mass / (entity1.mass + entity2.mass);
entity1.apply_push(overlap * ratio);
entity2.apply_push(overlap * (1.0 - ratio));
```
**Pros**: Minimal change, easy to migrate
**Cons**: Less structured, still hardcoded in logic

## Specific Problems to Solve

**Problem 1: Mass-Based Push Separation**
Currently uses hardcoded 3/10 (player) and 7/10 (slime). Should be:
```rust
// Mass-based ratio
let total_mass = player.mass + slime.mass;
let player_ratio = slime.mass / total_mass;  // Heavier = pushed less
let slime_ratio = player.mass / total_mass;

player.apply_push(overlap * player_ratio);
slime.apply_push(overlap * slime_ratio);
```
Where do mass values live? PhysicsBody struct? Constants?

**Problem 2: Velocity vs Direct Position**
Player uses velocity, slimes modify position directly. Should unify to:
- All entities have velocity
- Update loop: position += velocity * delta_time
- Input/AI modifies velocity, not position

**Problem 3: Static vs Dynamic Collision Response**
Currently two separate code paths (main.rs:1368-1413 vs 1487-1503). Should be:
- Static entities have infinite mass (or mass = 0.0 to indicate immovable?)
- Same push separation algorithm handles both cases

**Problem 4: Physics Configuration**
Where do these live:
- Friction (currently 0.8 in player.rs)
- Mass values (currently hardcoded ratios)
- Max speeds (currently in entity stats)
- Push force multipliers

Options:
- Hardcoded constants in physics.rs
- JSON config file (assets/config/physics.json)
- Per-entity in their constructors

**Problem 5: Forces vs Direct Velocity**
Currently both exist:
- Player input directly sets velocity
- apply_push() directly modifies position
Should there be:
- apply_force() that modifies velocity
- update() that applies velocity to position
- Separation between instantaneous (push) and continuous (forces)?

## Expected Deliverables

Provide a detailed design document including:

1. **Architecture Overview**
   - Module structure (src/physics.rs? Part of collision.rs?)
   - Core structs (PhysicsBody, PhysicsConfig, etc.)
   - How physics integrates with entities
   - Data flow diagram (Input → Physics → Position)

2. **Physics Data Model**
   - What data does each entity need (velocity, mass, friction)?
   - Where does this data live (struct field, component, trait)?
   - Who owns it (entity, system, separate vector)?

3. **API Design**
   - Public interface for physics operations
   - How to apply forces/impulses
   - How to update physics (per-entity or batch?)
   - Code examples showing usage

4. **Collision Response Integration**
   - How collision system calls physics system
   - Push-apart algorithm with mass-based ratios
   - Static vs dynamic collision handling
   - Example: player collides with slime → what happens?

5. **Configuration Strategy**
   - What constants exist (list them all)
   - Where they're defined (code, JSON, TOML?)
   - How to tune without recompiling
   - Default values and reasoning

6. **Migration Strategy**
   - Phase 1: Extract constants, no structure change
   - Phase 2: Add PhysicsBody/trait to entities
   - Phase 3: Unify velocity-based movement
   - Phase 4: Refactor collision response in main.rs
   - Which parts to change first (lowest risk)

7. **Rust Patterns Explained**
   - Struct composition vs trait abstraction (which and why?)
   - f32 vs i32 for physics math
   - When to use fixed-point math (if at all)
   - Performance considerations

## Success Criteria

Your design is successful if:
- ✅ Physics constants are in one place (not scattered)
- ✅ Push separation is mass-based (not hardcoded ratios)
- ✅ All entities use velocity-based movement
- ✅ Collision response code reduced by 50%+ lines
- ✅ Adding new physics behavior requires < 10 lines
- ✅ Gameplay feel is tunable without code changes
- ✅ No performance regression (still 60 FPS)
- ✅ Code is more understandable (clear separation of concerns)

## Anti-Patterns to Avoid

- ❌ Don't build a full physics engine (Box2D, Rapier) - too complex
- ❌ Don't abstract too much (trait soup, generic over everything)
- ❌ Don't add features the game doesn't need (gravity, rotation)
- ❌ Don't break existing gameplay feel (player movement must stay responsive)
- ❌ Don't introduce frame-rate dependence (fixed timestep already exists)
- ❌ Don't use f64 if f32 is sufficient (2D game, not physics simulation)

## References

Study these files for context:
- `src/main.rs:1368-1413` - Dynamic collision response (push-apart)
- `src/main.rs:1487-1503` - Static collision response
- `src/player.rs` - Player movement and velocity system
- `src/slime.rs` - Slime jumping behavior (no velocity)
- `src/collision.rs` - Collision detection (already implemented)
- `docs/systems/collision_implementation_plan.md` - Collision system design
- `docs/patterns/entity-pattern.md` - Entity structure conventions

## Questions to Answer

As you design, explicitly address:
1. Should all entities have velocity, even static ones?
2. How to represent "immovable" (infinite mass, special flag, trait)?
3. Should physics be data (PhysicsBody struct) or behavior (Physical trait)?
4. Where do physics constants live (code, config file)?
5. How to handle instant position changes (teleport, spawn)?
6. Should there be separate "impulse" (instant) vs "force" (continuous)?
7. How to debug physics (visualize velocity, forces)?
8. What unit system (pixels/second, tiles/second)?

## Example Use Cases

Show how your design handles:

**Use Case 1: Player Collides with Slime**
```rust
// Current: Hardcoded ratios
player.apply_push(-overlap_x * 3 / 10, 0);
slime.apply_push(overlap_x * 7 / 10, 0);

// Your design: ???
```

**Use Case 2: Player Hits Wall**
```rust
// Current: 100% push on player
player.apply_push(-overlap_x, 0);

// Your design: ???
```

**Use Case 3: Knockback from Attack**
```rust
// Future feature: Slime hit by player attack → knocked back
// How would this work with your physics system?
```

**Use Case 4: Adding New Entity Type**
```rust
// Adding "Rock" entity (heavy, pushable by player)
// What physics data does Rock need?
// How does it integrate with collision response?
```

## Final Note

Remember: This is about **gameplay feel**, not physics simulation accuracy. Your design should:
- **Feel good** - Player movement stays responsive
- **Be tunable** - Easy to adjust without code changes
- **Be simple** - Don't over-engineer for unused features
- **Integrate cleanly** - Work with existing collision and entity systems

The goal is **separation of concerns** (what collides vs how things move) and **configurability** (tune without recompile).

Prioritize **gameplay** over **realism**. If realistic physics feels bad, the design should allow easy tweaking.

Good luck! 🎮⚙️
