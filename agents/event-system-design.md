# AI Prompt: Event System Design

## Context

You are designing an **Event System** (or Message Bus) for Game1, a 2.5D Rust game. Currently, **all inter-system communication happens through direct function calls and shared mutable state**, leading to tight coupling, difficulty testing systems in isolation, and challenges implementing features like combat logs, achievement tracking, or replays.

## Project Background

**Technology Stack:**
- Rust (learning-focused project - teach message passing, enums, pattern matching)
- SDL2 for I/O
- Custom entity system (not ECS)
- Fixed 60 FPS update loop

**Existing Patterns:**
- DamageEvent struct for combat (src/combat.rs) - already event-like!
- Saveable trait for persistence
- Update methods called directly from main loop
- Collision callbacks are immediate function calls

**Current Communication Patterns:**
All communication is direct and synchronous:
1. Player attacks → main.rs checks overlap → calls slime.take_damage()
2. Slime dies → main.rs spawns DroppedItem directly
3. Player picks up item → main.rs calls inventory.quick_add()
4. Entity awakens → main.rs modifies player.active_modifiers directly

## The Problem

**Tight Coupling Everywhere:**

**Example 1: Entity Death and Loot Drop (main.rs:1415-1444)**
```rust
for slime in &mut slimes {
    if slime.is_dying() && !slime.has_dropped_loot {
        slime.has_dropped_loot = true;
        // Main loop knows how to create dropped items
        let mut item_animation_controller = animation::AnimationController::new();
        // 15 lines of item creation logic
        dropped_items.push(dropped_item);
    }
}
```
**Problem**: Main loop knows intimate details about item creation, animation setup, etc.

**Example 2: Pyramid Buff Application (main.rs:1284-1318)**
```rust
player.active_modifiers.clear();
for entity in &entities {
    if entity.state == EntityState::Awake {
        match entity.entity_type {
            EntityType::Attack => {
                player.active_modifiers.push(ModifierEffect { ... });
            }
            // ...
        }
    }
}
```
**Problem**: Main loop directly manipulates player modifiers based on entity state.

**Example 3: Combat Damage (main.rs:1228-1234)**
```rust
for slime in &mut slimes {
    let slime_bounds = slime.get_bounds();
    if collision::aabb_intersect(&attack_hitbox, &slime_bounds) {
        slime.take_damage(attack.damage as i32);
    }
}
```
**Problem**: Main loop handles combat resolution, no logging/animation hooks.

**Example 4: Player Death Item Drop (main.rs:1387-1411)**
```rust
if damage_result.is_fatal {
    // 25 lines of item drop logic embedded in collision handling
    for item_stack_option in player_inventory.inventory.slots.iter_mut() {
        // Manually spawn dropped items
    }
}
```
**Problem**: Collision response code handles death consequences directly.

**Pain Points:**
1. **No decoupling** - Systems can't be tested independently
2. **No observation** - Can't track what happened (logging, achievements, analytics)
3. **No replay** - Can't record/playback events for debugging
4. **Brittle** - Changing one system requires touching main.rs
5. **No async** - Everything happens immediately in same frame
6. **No cancellation** - Can't prevent damage, block pickup, etc.
7. **Hidden dependencies** - Not obvious what affects what

## Your Task

Design an **Event System** that:
1. **Decouples** systems (entities emit events, systems listen)
2. **Enables** observation (log all game events for debugging)
3. **Supports** gameplay features (damage numbers, kill counters, achievements)
4. **Maintains** performance (event handling in <0.5ms per frame)
5. **Integrates** with existing code gradually (incremental adoption)
6. **Teaches** Rust patterns (enums, pattern matching, interior mutability?)

## Requirements

### Must Have
- [ ] Event types as enum (DamageDealt, ItemPickedUp, EntityDied, etc.)
- [ ] Event emission (entities/systems can send events)
- [ ] Event handling (systems subscribe to event types)
- [ ] Integration with existing game loop
- [ ] Support for immediate events (this frame) vs deferred (next frame)
- [ ] Minimal allocations (use Vec, not channels if possible)
- [ ] Clear ownership model (who owns EventBus/Queue?)

### Should Have
- [ ] Event batching (collect events during update, process at end)
- [ ] Event data (carry context: who dealt damage, how much, etc.)
- [ ] Type-safe event handlers (pattern match on event enum)
- [ ] Priority/ordering (some events should process first)
- [ ] Event filtering (handlers see only relevant events)

### Nice to Have (Don't Add if Not Needed)
- [ ] Event history (ring buffer for debugging)
- [ ] Event cancellation (handler can prevent event)
- [ ] Async events (delayed effects)
- [ ] Event serialization (save replay data)

### Must NOT Have (Premature Features)
- ❌ Network synchronization (no multiplayer yet)
- ❌ Undo/redo system
- ❌ Time-travel debugging
- ❌ Complex event filtering DSL
- ❌ Multi-threaded event processing
- ❌ External event sources (mods, scripting)

## Design Constraints

**Rust Learning Goals:**
This design should teach:
- Enums for heterogeneous data (different event types)
- Pattern matching for event dispatch
- Vec/VecDeque for event queues
- Borrowing (when to use &Event vs owned Event)
- Interior mutability (if needed - RefCell, Cell)
- Trait objects for handlers (if needed)

**Integration Points:**
- **Combat System** - Already has DamageEvent struct (see src/combat.rs)
- **Main Loop** - Where to emit/process events?
- **Entity Lifecycle** - Spawn, update, die → events
- **UI Systems** - Damage numbers, kill notifications
- **Save System** - Events might be saveable (for replays?)

**Performance Requirements:**
- 60 FPS game loop
- ~50-100 events per frame (worst case: many enemies die)
- Event processing <0.5ms per frame
- No heap allocations in event emission (if possible)

**Current Game Events (Implicit):**
From analyzing main.rs, these things "happen":
1. Player attacks (M key pressed)
2. Enemy takes damage
3. Enemy dies
4. Loot drops
5. Player picks up item
6. Item added to inventory
7. Pyramid awakens
8. Player receives buff
9. Player takes damage
10. Player dies
11. Player respawns
12. Tile edited (hoe used)
13. Slime spawned (right-click)

## Suggested Architecture

Consider these approaches:

### Approach 1: Simple Event Queue
```rust
pub enum GameEvent {
    DamageDealt { attacker: EntityId, target: EntityId, amount: i32 },
    EntityDied { entity_id: EntityId, entity_type: EntityType },
    ItemPickedUp { entity_id: EntityId, item_id: String, quantity: u32 },
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
```
**Pros**: Simple, no indirection, easy to understand
**Cons**: Single handler, no per-system subscriptions

### Approach 2: Observer Pattern (Listeners)
```rust
pub trait EventListener {
    fn handle_event(&mut self, event: &GameEvent);
}

pub struct EventBus {
    listeners: Vec<Box<dyn EventListener>>,
    events: Vec<GameEvent>,
}

impl EventBus {
    pub fn subscribe(&mut self, listener: Box<dyn EventListener>) {
        self.listeners.push(listener);
    }

    pub fn emit(&mut self, event: GameEvent) {
        self.events.push(event);
    }

    pub fn process(&mut self) {
        for event in &self.events {
            for listener in &mut self.listeners {
                listener.handle_event(event);
            }
        }
        self.events.clear();
    }
}
```
**Pros**: Decoupled, systems subscribe independently
**Cons**: Trait objects, lifetime management, borrowing issues

### Approach 3: Channel-Based (mpsc)
```rust
use std::sync::mpsc::{channel, Sender, Receiver};

pub struct EventSystem {
    sender: Sender<GameEvent>,
    receiver: Receiver<GameEvent>,
}

// Systems get clone of sender
// Main loop drains receiver each frame
```
**Pros**: Decoupled, thread-safe (future multiplayer?)
**Cons**: Heap allocations, overkill for single-threaded game

### Approach 4: Immediate + Deferred Queues
```rust
pub struct EventSystem {
    immediate: Vec<GameEvent>,  // Process this frame
    deferred: Vec<GameEvent>,   // Process next frame
}

// Useful for: damage event → death event → loot drop event
// Prevents iterator invalidation during iteration
```
**Pros**: Handles cascading events cleanly
**Cons**: More complexity

## Specific Problems to Solve

**Problem 1: Entity References in Events**
Events need to reference entities (who took damage?). Options:
```rust
// Option A: Entity IDs
DamageDealt { attacker_id: usize, target_id: usize, ... }

// Option B: Weak references (complex)
DamageDealt { attacker: Weak<RefCell<Entity>>, ... }

// Option C: Just positions/names
DamageDealt { attacker_name: String, target_pos: (i32, i32), ... }
```
Which approach? (IDs probably simplest)

**Problem 2: Event Timing**
When to emit vs process events?
```rust
// Game loop:
1. Handle input → emit events?
2. Update entities → emit events?
3. Check collisions → emit events?
4. Process events → spawn items, apply damage?
5. Render

// Or process events immediately?
player.attack() → emit DamageDealt → handler applies damage immediately
```

**Problem 3: Cascading Events**
Damage → Death → Loot Drop → Pickup (if player nearby)
How to handle chain reactions without stack overflow?

**Problem 4: Event Data Ownership**
Should events own data (String) or borrow (&str)?
```rust
pub enum GameEvent {
    ItemPickedUp { item_id: String, ... },  // Owned
    // vs
    ItemPickedUp { item_id: &'static str, ... },  // Borrowed
}
```

**Problem 5: Integration with Existing Code**
How to migrate incrementally:
- Phase 1: Add EventQueue, emit events alongside direct calls
- Phase 2: Replace direct calls with event handlers
- Phase 3: Remove direct calls entirely

**Problem 6: Mutable Borrowing**
If EventBus is part of GameWorld, and handlers need &mut GameWorld:
```rust
world.event_bus.emit(event);  // Borrows world
world.spawn_item(...);  // Can't do this if event_bus holds borrow!
```
How to structure ownership?

## Expected Deliverables

Provide a detailed design document including:

1. **Architecture Overview**
   - Module structure (src/events.rs? src/event_system.rs?)
   - Core enum (GameEvent with all variants)
   - Queue/Bus structure
   - Data flow diagram (System → EventQueue → Handlers → Systems)

2. **Event Catalog**
   - Complete list of current game events (from analysis above)
   - Event data for each type (what info does event carry?)
   - Grouping/categories (combat, items, entities, world)

3. **API Design**
   - How to emit events
   - How to handle events
   - How to subscribe (if using listeners)
   - Code examples showing before/after

4. **Integration Strategy**
   - Where EventQueue lives (in GameWorld? Separate?)
   - When events are emitted (during update, collision, etc.)
   - When events are processed (end of frame, immediately?)
   - How to avoid borrow checker issues

5. **Migration Plan**
   - Phase 1: Add EventQueue, emit alongside direct calls (parallel)
   - Phase 2: Add handlers, verify behavior matches
   - Phase 3: Remove direct calls
   - Which code to migrate first (lowest risk, highest value)

6. **Use Case Examples**
   Show before/after for:
   - Enemy death → loot drop
   - Player takes damage → floating damage number
   - Pyramid awakens → player buff applied
   - Item pickup → inventory update

7. **Rust Patterns Explained**
   - Enum design (large variants, Box if needed?)
   - Ownership model (who owns events?)
   - Borrowing strategy (avoid RefCell if possible)
   - Performance considerations (allocations, cloning)

## Success Criteria

Your design is successful if:
- ✅ Main loop code reduced by 20%+ (logic moved to event handlers)
- ✅ Systems can be tested in isolation (emit fake events)
- ✅ Game events are observable (can log all events for debugging)
- ✅ Adding new event type requires < 5 lines in 2 places
- ✅ No borrow checker fights (clean ownership model)
- ✅ No performance regression (event processing <0.5ms)
- ✅ Code is more maintainable (clear cause and effect)
- ✅ Future features easier (achievements, replays, combat log)

## Anti-Patterns to Avoid

- ❌ Don't use channels for single-threaded game (overkill)
- ❌ Don't make events too granular (PlayerMovedOnePixel? No.)
- ❌ Don't use dynamic dispatch unless needed (trait objects have cost)
- ❌ Don't put entire game state in events (just IDs/minimal data)
- ❌ Don't process events recursively (stack overflow risk)
- ❌ Don't make event system god-object (focus on messaging only)

## References

Study these files for context:
- `src/main.rs:1228-1234` - Combat damage handling (target for events)
- `src/main.rs:1415-1444` - Loot drop on death (target for events)
- `src/main.rs:1284-1318` - Buff application (target for events)
- `src/main.rs:1387-1411` - Player death item drop (target for events)
- `src/combat.rs` - DamageEvent struct already exists!
- `docs/patterns/entity-pattern.md` - Entity structure
- `docs/systems/save-system-design.md` - How systems are structured

## Questions to Answer

As you design, explicitly address:
1. Should events be processed immediately or batched?
2. How to handle cascading events (event causes event)?
3. Where does EventQueue/Bus live in code structure?
4. How to avoid borrow checker issues (EventBus in World?)?
5. Should events be Copy/Clone or owned?
6. How to identify entities in events (ID, reference, name)?
7. Can event handlers cancel/modify events?
8. Should there be event history for debugging?
9. How to test event-driven code?

## Example Use Cases

Show how your design handles:

**Use Case 1: Slime Dies, Drops Loot**
```rust
// Current (main.rs:1415-1444): 30 lines in main loop
if slime.is_dying() && !slime.has_dropped_loot {
    slime.has_dropped_loot = true;
    // Manually create dropped item...
}

// Your design: ???
// Emit EntityDied event → LootDropHandler spawns item?
```

**Use Case 2: Player Takes Damage, Show Damage Number**
```rust
// Current: damage applied, no UI feedback
let damage_result = player.take_damage(damage);

// Your design: ???
// Emit DamageDealt event → FloatingTextHandler shows number?
```

**Use Case 3: Item Picked Up, Add to Inventory**
```rust
// Current (main.rs:1450-1475): Direct call in main loop
if player_bounds.has_intersection(item.get_bounds()) {
    match player_inventory.quick_add(...) {
        Ok(overflow) => { ... }
    }
}

// Your design: ???
// Emit ItemPickedUp event → InventoryHandler adds item?
```

**Use Case 4: Achievement System (Future)**
```rust
// Future: Track "kill 100 slimes" achievement
// How would your event system enable this?
// Achievement tracker subscribes to EntityDied events?
```

## Final Note

Remember: This is about **reducing coupling** and **enabling observation**, not building a complex framework. Your design should:
- **Simplify** main loop (move logic into handlers)
- **Enable** testing (mock events, test handlers independently)
- **Support** future features (logging, achievements, replays)
- **Integrate** gradually (coexist with direct calls during migration)

Prioritize **clarity** over **cleverness**. The event flow should be obvious, not magic.

If you're choosing between a simple Vec<Event> or a complex publish-subscribe system, start with the Vec. Add complexity only when needed.

Good luck! 📡✨
