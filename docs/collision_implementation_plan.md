# Collision System Implementation Plan

## Overview
This document outlines the plan for implementing a collision detection and response system for Game1. The goal is to add entity-to-entity collision detection in an idiomatic Rust manner while maintaining the existing 2.5D game architecture.

## Current State Analysis
- **Game Engine**: SDL2 with custom sprite/animation system
- **Entities**: Player (controllable, 8-directional movement) and Slime (AI-controlled, jumping behavior)
- **Coordinate System**: 2D screen coordinates with 3x rendering scale
- **Entity Dimensions**: 32x32 base size, scaled to 96x96 on screen
- **No Collision**: Currently entities can overlap freely

## Collision System Goals
1. **Detect collisions** between player and slimes (dynamic entities)
2. **Detect collisions** with static world objects (rocks, buildings, etc.)
3. **Prevent overlap** with appropriate physical response
4. **Enable gameplay mechanics** (e.g., player damage, slime knockback)
5. **Maintain performance** with simple AABB collision detection
6. **Learn Rust patterns**: Traits, ownership, and game-specific patterns

## Architecture Design

### Phase 1: Core Collision Module
**Objective**: Create a collision detection module with reusable components

**Rust Concepts to Learn**:
- Traits for shared behavior
- Generic programming
- Borrowing rules in game loops

**Tasks**:
- [ ] Create `src/collision.rs` module
- [ ] Define `Collidable` trait with methods:
  - `get_bounds() -> Rect` - Returns collision box
  - `get_collision_layer() -> CollisionLayer` - Entity type
  - `on_collision(&mut self, other: &dyn Collidable)` - Collision callback
- [ ] Implement AABB (Axis-Aligned Bounding Box) intersection function
- [ ] Define `CollisionLayer` enum (Player, Enemy, Projectile, etc.)
- [ ] Add unit tests for AABB intersection logic

**Files to Create/Modify**:
- Create: `src/collision.rs`
- Modify: `src/main.rs` (add module declaration)

**Expected Learning**:
- How to design trait-based APIs in Rust
- The difference between static and dynamic dispatch
- Writing effective unit tests for game logic

---

### Phase 2: Implement Collidable for Entities
**Objective**: Make Player and Slime entities collision-aware

**Rust Concepts to Learn**:
- Trait implementation for structs
- Reference vs mutable reference patterns
- Lifetime considerations in trait methods

**Tasks**:
- [ ] Implement `Collidable` trait for `Player`
  - Return bounds based on `x, y, width, height` (account for 3x scale)
  - Set collision layer to `CollisionLayer::Player`
  - Add placeholder collision response
- [ ] Implement `Collidable` trait for `Slime`
  - Return bounds (accounting for jump offset)
  - Set collision layer to `CollisionLayer::Enemy`
  - Add placeholder collision response
- [ ] Add helper method to convert entity coordinates to collision `Rect`

**Files to Modify**:
- `src/player.rs`
- `src/slime.rs`
- `src/collision.rs` (if helper functions needed)

**Expected Learning**:
- How trait implementations work across different modules
- Managing mutable state during collision detection
- Separation of concerns (rendering vs collision bounds)

---

### Phase 3: Collision Detection System
**Objective**: Create a central system to check and resolve collisions

**Rust Concepts to Learn**:
- Borrowing multiple mutable references (player vs slimes vector)
- Iterator patterns for efficient collision checks
- Error handling in game loops

**Tasks**:
- [ ] Create `CollisionWorld` or `CollisionSystem` struct in `collision.rs`
- [ ] Implement `check_collision_pair()` function for two collidables
- [ ] Implement `check_player_vs_slimes()` function
  - Takes `&mut Player` and `&mut Vec<Slime>`
  - Iterates through slimes checking for collision with player
  - Returns collision results (Vec of indices or collision info)
- [ ] Integrate collision checks into main game loop
  - Call collision system after entity updates
  - Before rendering

**Files to Modify**:
- `src/collision.rs`
- `src/main.rs` (game loop)

**Expected Learning**:
- How to safely borrow multiple mutable references
- Iterator methods (`iter_mut()`, `enumerate()`)
- Structuring game loop phases (update → collision → render)

---

### Phase 4: Collision Response - Push-Apart
**Objective**: Prevent entities from overlapping

**Rust Concepts to Learn**:
- Mutable state modification based on game state
- Vector math for physics responses
- Floating-point vs integer coordinate handling

**Tasks**:
- [ ] Implement simple push-apart algorithm
  - Calculate overlap on X and Y axes
  - Determine minimum push-apart vector
  - Move entities apart along shortest axis
- [ ] Decide collision resolution priority
  - Option A: Player is immovable, push slimes away
  - Option B: Split the push (both move half the distance)
- [ ] Update entity positions in collision callbacks
- [ ] Test collision response with multiple slimes

**Files to Modify**:
- `src/collision.rs` (push-apart logic)
- `src/player.rs` (collision response)
- `src/slime.rs` (collision response)

**Expected Learning**:
- Basic game physics programming
- Managing coordinate precision (f32 vs i32)
- Handling edge cases (multiple simultaneous collisions)

---

### Phase 5: Gameplay Integration
**Objective**: Add game mechanics based on collisions

**Rust Concepts to Learn**:
- State machines in Rust (for player damage states)
- Enums for representing game events
- Timer/cooldown patterns

**Tasks**:
- [ ] Add collision effects to Player
  - Knockback when hit by slime
  - Damage/invulnerability system
  - Visual feedback (flashing, animation state)
- [ ] Add collision effects to Slime
  - Knockback when attacked by player
  - Death on collision with player attack
  - Optional: Slime-to-slime collision avoidance
- [ ] Add collision filtering
  - Player attacks only collide with enemies
  - Slimes don't damage each other (or implement bouncing)
- [ ] Visual/audio feedback (optional)
  - Screen shake on hit
  - Sound effects

**Files to Modify**:
- `src/player.rs` (health/damage system)
- `src/slime.rs` (death handling)
- `src/collision.rs` (filtering logic)
- `src/main.rs` (remove dead slimes, effects)

**Expected Learning**:
- Game state management patterns
- Cooldown/timer implementation
- Remove-while-iterating patterns (retain, swap_remove)

---

### Phase 6: Optimization & Polish
**Objective**: Ensure performance and code quality

**Rust Concepts to Learn**:
- Profiling Rust game code
- Optimization patterns (spatial partitioning intro)
- Clippy and best practices

**Tasks**:
- [ ] Run `cargo clippy` and fix any warnings
- [ ] Add comprehensive unit tests for collision system
- [ ] Profile collision detection performance (if needed)
  - Consider spatial partitioning if >100 entities
  - Simple grid or quadtree (optional advanced feature)
- [ ] Document the collision system in code comments
- [ ] Update main README with collision features

**Files to Modify**:
- All collision-related files (add docs)
- `src/collision.rs` (add tests module)
- Top-level README or CLAUDE.md

**Expected Learning**:
- Rust documentation best practices
- Performance profiling tools
- When to optimize (measure first!)

---

### Phase 7: Static World Object Collision
**Objective**: Add collision with non-moving world objects (rocks, buildings, trees, etc.)

**Rust Concepts to Learn**:
- Trait specialization patterns
- Separation of dynamic vs static collision logic
- Collection management for game world objects
- One-way collision response (only player moves)

**Tasks**:
- [ ] Define `StaticCollidable` trait in `collision.rs`
  - `get_bounds() -> Rect` - Returns collision box
  - `get_collision_type() -> StaticCollisionType` - Object category
  - Optional: `is_solid() -> bool` - Some static objects might be passable
- [ ] Create `StaticObject` struct (or specific types like `Rock`, `Tree`, `Building`)
  - Basic fields: `x, y, width, height`
  - Optional: `collision_type` enum (Solid, Trigger, Passable)
  - Implement `StaticCollidable` trait
- [ ] Add static object collection to main game state
  - `Vec<StaticObject>` or `Vec<Box<dyn StaticCollidable>>`
  - Load from level data (hardcoded first, JSON later)
- [ ] Implement player vs static object collision checks
  - Add `check_player_vs_static()` function
  - One-way push: only player gets moved, static objects stay put
  - Integrate into game loop after entity updates
- [ ] Add helper methods for placing static objects
  - `place_rock(x, y)`, `place_building(x, y, width, height)`, etc.
  - Consider factory pattern or builder for complex objects
- [ ] Test collision with various static object layouts
  - Create test level with walls/obstacles
  - Verify player cannot pass through solid objects
  - Test corner cases (wedged between objects)

**Files to Create/Modify**:
- Modify: `src/collision.rs` (add `StaticCollidable` trait)
- Create: `src/world_object.rs` or add to existing module
- Modify: `src/main.rs` (static object collection, collision checks)

**Expected Learning**:
- Trait-based polymorphism for different object types
- One-way collision response implementation
- Managing separate collections (dynamic vs static)
- Level design data structures

**Design Considerations**:

**Static vs Dynamic Collision Response**:
```rust
// Dynamic entity collision (both objects can move)
fn resolve_dynamic_collision(a: &mut Player, b: &mut Slime) {
    // Both entities get pushed apart
    let push_vector = calculate_push_apart(a.get_bounds(), b.get_bounds());
    a.apply_push(push_vector * 0.5);
    b.apply_push(push_vector * -0.5);
}

// Static object collision (only player moves)
fn resolve_static_collision(player: &mut Player, object: &StaticObject) {
    // Only player gets pushed, object stays put
    let push_vector = calculate_push_apart(player.get_bounds(), object.get_bounds());
    player.apply_push(push_vector);
}
```

**Object Type Flexibility**:
- Start simple: Single `StaticObject` struct
- Later: Specialize with `Rock`, `Tree`, `Building` structs if needed
- Use enum or trait objects for heterogeneous collections

**Performance Note**:
- Static objects don't need to check against each other
- Only check player (and potentially projectiles) vs static objects
- Consider spatial partitioning if you have 100+ static objects

---

## Implementation Order Summary
1. **Phase 1**: Core collision module (traits, AABB)
2. **Phase 2**: Trait implementation for Player/Slime
3. **Phase 3**: Detection system integrated into game loop
4. **Phase 4**: Push-apart collision response
5. **Phase 5**: Gameplay mechanics (damage, knockback)
6. **Phase 6**: Testing, optimization, documentation
7. **Phase 7**: Static world object collision

## Key Rust Learning Checkpoints
- ✓ Trait definition and implementation
- ✓ Borrowing rules with multiple mutable entities
- ✓ Enum-based state machines
- ✓ Iterator patterns for collections
- ✓ Unit testing game logic
- ✓ Error handling in game loops
- ✓ Code organization across modules

## Design Decisions & Tradeoffs

### Why AABB over Circle Collision?
- **Simpler math**: Rectangle intersection is straightforward
- **Sprite alignment**: Matches rectangular sprite rendering
- **Performance**: Slightly faster than circle checks
- **Tradeoff**: Less accurate for circular sprites, but good enough for this game

### Why Trait-Based Design?
- **Extensibility**: Easy to add new entity types (projectiles, items)
- **Rust idioms**: Traits are the Rust way to define shared behavior
- **Learning**: Great introduction to polymorphism in Rust
- **Tradeoff**: Slightly more complex than direct struct methods

### Push-Apart vs Physics Engine?
- **Simplicity**: Push-apart is easy to understand and implement
- **Control**: Full control over collision response behavior
- **Learning**: Builds understanding before using libraries
- **Tradeoff**: Won't scale to complex physics (but not needed here)

## Future Enhancements (Post-MVP)
- Tilemap collision (walls, obstacles)
- Trigger zones (doors, level transitions)
- Projectile collision
- Advanced physics (bouncing, friction)
- Spatial partitioning (quadtree/grid) for performance

## Testing Strategy
1. **Unit tests**: AABB intersection logic, collision detection
2. **Manual testing**: Spawn many slimes, test edge cases
3. **Visual debugging**: Optional debug rendering of collision boxes

## Success Criteria
**Dynamic Entity Collision (Phases 1-6)**:
- ✓ Player cannot overlap with slimes
- ✓ Slimes push away when player moves through them
- ✓ Combat mechanics work (damage, knockback)
- ✓ Performance: 60 FPS with 50+ slimes

**Static World Collision (Phase 7)**:
- ✓ Player cannot pass through solid objects
- ✓ Smooth collision response (no getting stuck)
- ✓ Performance: 60 FPS with 100+ static objects

**Code Quality (All Phases)**:
- ✓ No runtime panics or crashes
- ✓ Clean `cargo clippy` output
- ✓ Documented code with examples
- ✓ Comprehensive unit tests
