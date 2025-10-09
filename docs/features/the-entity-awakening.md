# The Entity - Awakening Pyramid Feature

## Overview

**The Entity** is an interactive environmental object that represents an ancient being trapped within a stone pyramid. Players can awaken it through combat, triggering a multi-stage animation sequence. The entity can fall back asleep if left alone, creating an engaging world interaction.

**Status**: 📋 **PLANNED** (October 2025)

## Feature Summary

- **4 pyramids** spawn in the default world at fixed locations
- **Dormant state**: Stone pyramid (frame 1) that blocks player movement
- **Awakening sequence**: 8-frame animation (frames 1→8) triggered by player attack
- **Active state**: Looping animation (frames 8→13) with partial collision
- **Sleep timeout**: Returns to dormant after 30 seconds of inactivity
- **Interruptible**: Hitting during sleep animation restarts awakening
- **Depth sorting**: Player can walk behind tall pyramid
- **Persistence**: State saved/loaded between sessions

## Sprite Analysis

**File**: `assets/sprites/the_entity/entity_awaken.png`
- **Dimensions**: 416×32 pixels (13 frames × 32px)
- **Frame size**: 32×32 pixels per frame
- **Frame count**: 13 total frames
- **Animation type**: Horizontal sprite sheet

### Frame Breakdown

```
Frame 1:  ◆ Dormant Pyramid (dark gray stone)
Frames 2-8: Awakening sequence (stone breaks, entity emerges)
  ├─ Frame 2-4: Cracks appear, cyan glow begins
  ├─ Frame 5-7: Entity partially emerged, glow intensifies
  └─ Frame 8: Fully emerged, transition frame
Frames 8-13: Active loop (entity floating/pulsing)
  └─ Frame 8-13: Cyan entity pulsing above pyramid base
```

### Visual Phases

```
DORMANT (1 frame):
   ◆     Frame 1: Solid stone pyramid
  ███

AWAKENING (8 frames, one-shot):
   ◆ → ◆ → ◈ → ◈ → ◊ → ◊ → ○ → ○
  ███   █▓█   ▓▒▓   ▒░▒   ░ ░   · ·   · ·   ···
  1     2     3     4     5     6     7     8

ACTIVE (6 frames, looping):
   ○ ↔ ○ ↔ ○ ↔ ○ ↔ ○ ↔ ○
  ···  ···  ···  ···  ···  ···
  8    9    10   11   12   13

SLEEPING (reverse awakening, 8 frames):
   ○ → ○ → ◊ → ◊ → ◈ → ◈ → ◆ → ◆
  ···   · ·   ░ ░   ▒░▒   ▓▒▓   █▓█   ███
  8     7     6     5     4     3     2     1
```

## State Machine Design

### States

```rust
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum EntityState {
    Dormant,       // Frame 1, full collision
    Awakening,     // Frames 1→8, transitioning
    Awake,         // Frames 8→13 loop, partial collision
    Sleeping,      // Frames current→1, reverse animation
}
```

### State Transitions

```
           ┌─────────────────────────────────┐
           │                                 │
           ▼                                 │
    [DORMANT] ──hit──> [AWAKENING] ──complete──> [AWAKE]
           ▲               ▲                      │
           │               │                      │
           │               └────hit (interrupt)───┤
           │                                      │
           └────complete────── [SLEEPING] <──timeout (30s)
```

### State Behaviors

| State | Animation | Collision | Damage | Timeout | Can Interrupt |
|-------|-----------|-----------|--------|---------|---------------|
| **Dormant** | Static frame 1 | Full (32×32) | No | None | Hit → Awakening |
| **Awakening** | Frames 1→8 (one-shot) | Full | No | None | No |
| **Awake** | Frames 8→13 (loop) | Partial (bottom 16px) | No | 30s | Timeout → Sleeping |
| **Sleeping** | Current→1 (reverse) | Partial→Full | No | None | Hit → Awakening |

## System Integration Requirements

### 1. Animation System (Existing)

**Current Capabilities:**
- ✅ Frame-based animation with timing
- ✅ Looping animations
- ✅ Multiple animation states

**Needed Enhancements:**
- 🔨 **Reverse animation playback** (for sleeping state)
- 🔨 **Animation completion callback** (to trigger state changes)
- 🔨 **Frame range animation** (1→8, 8→13, etc.)
- 🔨 **Animation interruption** (restart from current frame)

**Implementation in**: `src/animation.rs`, `src/sprite.rs`

### 2. Collision System (Existing)

**Current Capabilities:**
- ✅ AABB collision detection
- ✅ Static object collision

**Needed Enhancements:**
- 🔨 **Partial collision bounds** (bottom half only for awake state)
- 🔨 **Dynamic collision bounds** (change based on state)

**Implementation in**: `src/collision.rs`

### 3. Depth Sorting System (Planned)

**Required Capabilities:**
- 🆕 `DepthSortable` trait implementation for Entity
- 🆕 Anchor-based depth sorting (base of pyramid)
- 🆕 Render tall sprite from anchor point

**Implementation in**: `src/render.rs` (new module)

### 4. Save System (Existing)

**Current Capabilities:**
- ✅ `Saveable` trait
- ✅ Entity state serialization

**Needed Enhancements:**
- 🔨 **Save entity state** (Dormant/Awakening/Awake/Sleeping)
- 🔨 **Save current animation frame** (for mid-animation saves)
- 🔨 **Save inactivity timer** (time until sleep)

**Implementation in**: `src/save.rs`, entity module

### 5. Timer System (New)

**Required Capabilities:**
- 🆕 **Inactivity timer** (track 30 seconds since last hit)
- 🆕 **Timer serialization** (save/load timer state)
- 🆕 **Timer reset** (on player interaction)

**Implementation in**: Entity struct

### 6. Combat Integration (Existing)

**Current Capabilities:**
- ✅ Player attack detection
- ✅ Hit registration on collidable entities

**Needed Enhancements:**
- 🔨 **Non-damaging hits** (entity takes no damage but registers hit)
- 🔨 **State-dependent hit behavior** (dormant vs awake vs sleeping)

**Implementation in**: `src/combat.rs`, entity module

## Technical Specifications

### Entity Struct

```rust
pub struct TheEntity<'a> {
    // Position & Collision
    pub x: i32,
    pub y: i32,
    pub width: u32,            // 32
    pub height: u32,           // 32
    pub sprite_height: u32,    // 32 (same as collision for this object)

    // State Machine
    pub state: EntityState,
    current_frame: usize,
    last_state_change: Instant,
    inactivity_timer: f32,     // Seconds since last hit

    // Animation
    animation_controller: AnimationController<'a>,
    awakening_complete: bool,

    // Identification
    pub id: usize,             // For save/load and spawn tracking
}
```

### Collision Bounds (State-Dependent)

```rust
impl TheEntity<'_> {
    pub fn get_collision_bounds(&self) -> Rect {
        match self.state {
            EntityState::Dormant | EntityState::Awakening => {
                // Full collision (32×32)
                Rect::new(
                    self.x,
                    self.y,
                    self.width * SPRITE_SCALE,
                    self.height * SPRITE_SCALE,
                )
            }
            EntityState::Awake | EntityState::Sleeping => {
                // Partial collision (32×16, bottom half)
                let collision_height = (self.height / 2) * SPRITE_SCALE;
                let collision_y = self.y + collision_height as i32;
                Rect::new(
                    self.x,
                    collision_y,
                    self.width * SPRITE_SCALE,
                    collision_height,
                )
            }
        }
    }
}
```

### Animation Configuration

```json
{
  "frame_width": 32,
  "frame_height": 32,
  "animations": {
    "Dormant": {
      "frames": [
        { "x": 0, "y": 0, "duration_ms": 100 }
      ],
      "loop_animation": false
    },
    "Awakening": {
      "frames": [
        { "x": 0, "y": 0, "duration_ms": 150 },
        { "x": 32, "y": 0, "duration_ms": 150 },
        { "x": 64, "y": 0, "duration_ms": 150 },
        { "x": 96, "y": 0, "duration_ms": 150 },
        { "x": 128, "y": 0, "duration_ms": 150 },
        { "x": 160, "y": 0, "duration_ms": 150 },
        { "x": 192, "y": 0, "duration_ms": 150 },
        { "x": 224, "y": 0, "duration_ms": 150 }
      ],
      "loop_animation": false
    },
    "Awake": {
      "frames": [
        { "x": 224, "y": 0, "duration_ms": 200 },
        { "x": 256, "y": 0, "duration_ms": 200 },
        { "x": 288, "y": 0, "duration_ms": 200 },
        { "x": 320, "y": 0, "duration_ms": 200 },
        { "x": 352, "y": 0, "duration_ms": 200 },
        { "x": 384, "y": 0, "duration_ms": 200 }
      ],
      "loop_animation": true
    }
  }
}
```

### Spawn Locations

4 pyramids spawn at fixed world positions:

```rust
const ENTITY_SPAWN_LOCATIONS: [(i32, i32); 4] = [
    (320, 200),   // Top-left quadrant
    (800, 200),   // Top-right quadrant
    (320, 500),   // Bottom-left quadrant
    (800, 500),   // Bottom-right quadrant
];
```

Each entity gets a unique ID (0-3) for save/load tracking.

## Implementation Phases

### Phase 1: Core Infrastructure (Foundation)

**Goal**: Set up rendering and basic entity structure

**Tasks**:
- [ ] Create `src/render.rs` module
  - [ ] `DepthSortable` trait
  - [ ] `Renderable` enum
  - [ ] `render_with_depth_sorting()` function
- [ ] Implement `DepthSortable` for Player
- [ ] Implement `DepthSortable` for Slime
- [ ] Update main render loop to use depth sorting
- [ ] Test depth sorting with existing entities

**Success Criteria**: Player/slimes render in correct depth order

**Estimated Time**: 2-3 hours

---

### Phase 2: Entity Basic Structure

**Goal**: Create The Entity with dormant state only

**Tasks**:
- [ ] Create `src/the_entity.rs` module
- [ ] Define `TheEntity` struct with basic fields
- [ ] Define `EntityState` enum
- [ ] Implement dormant state (static frame 1)
- [ ] Implement `DepthSortable` for TheEntity
- [ ] Spawn 4 entities at fixed locations
- [ ] Add full collision detection (StaticCollidable)
- [ ] Test: entities render, player can't walk through

**Success Criteria**: 4 dormant pyramids spawn, block player, depth sort correctly

**Estimated Time**: 2 hours

---

### Phase 3: Animation System Enhancements

**Goal**: Add support for reverse playback and frame ranges

**Tasks**:
- [ ] Add animation direction to `SpriteSheet` (Forward/Reverse)
- [ ] Implement reverse frame iteration
- [ ] Add animation completion detection
- [ ] Add callback system for animation events
- [ ] Create entity animation config JSON
- [ ] Load and configure entity animations
- [ ] Test: play awakening animation manually

**Success Criteria**: Can play animations forward and backward with completion events

**Estimated Time**: 3-4 hours

**Related Rust Learning**:
- Closures for callbacks
- Option<T> for optional callbacks
- Enum for animation direction

---

### Phase 4: State Machine - Awakening

**Goal**: Implement dormant → awakening → awake transition

**Tasks**:
- [ ] Implement hit detection on entity
- [ ] Add `on_hit()` method to entity
- [ ] Trigger awakening animation on hit (if dormant)
- [ ] Detect awakening animation completion
- [ ] Transition to awake state on completion
- [ ] Start awake loop animation
- [ ] Test: hit pyramid, watch full awakening sequence

**Success Criteria**: Hitting dormant pyramid triggers awakening, transitions to awake loop

**Estimated Time**: 2-3 hours

---

### Phase 5: Partial Collision

**Goal**: Implement state-dependent collision bounds

**Tasks**:
- [ ] Refactor `StaticCollidable` to support dynamic bounds
- [ ] Implement `get_collision_bounds()` for entity
- [ ] Return full bounds for Dormant/Awakening
- [ ] Return bottom-half bounds for Awake/Sleeping
- [ ] Update collision detection to use dynamic bounds
- [ ] Test: player can walk through top half when awake

**Success Criteria**: Awake entity only collides on bottom half

**Estimated Time**: 2 hours

**Related Rust Learning**:
- Pattern matching for state-dependent behavior
- Rect manipulation

---

### Phase 6: Inactivity Timer & Sleeping

**Goal**: Implement 30-second timeout and sleep animation

**Tasks**:
- [ ] Add inactivity timer to entity (f32 seconds)
- [ ] Update timer in entity update() method (delta time)
- [ ] Reset timer on hit
- [ ] Trigger sleeping state when timer reaches 30s
- [ ] Play reverse animation (awake → dormant)
- [ ] Restore full collision during sleep
- [ ] Transition to dormant when sleep completes
- [ ] Test: leave entity alone for 30s, watch it sleep

**Success Criteria**: Entity sleeps after 30s, returns to dormant state

**Estimated Time**: 2-3 hours

**Related Rust Learning**:
- Time/duration handling
- Delta time integration

---

### Phase 7: Interrupt Sleeping

**Goal**: Allow hitting during sleep to restart awakening

**Tasks**:
- [ ] Detect hits during sleeping state
- [ ] Store current frame when interrupted
- [ ] Transition back to awakening state
- [ ] Resume forward animation from current frame
- [ ] Test: hit sleeping entity, watch it re-awaken

**Success Criteria**: Hitting sleeping entity restarts awakening animation

**Estimated Time**: 1-2 hours

---

### Phase 8: Save/Load Integration

**Goal**: Persist entity state between sessions

**Tasks**:
- [ ] Implement `Saveable` trait for TheEntity
- [ ] Serialize: state, current_frame, timer, id
- [ ] Deserialize: restore state and resume animation
- [ ] Add entities to save_game() function
- [ ] Add entities to load_game() function
- [ ] Handle texture/animation recreation on load
- [ ] Test: save in each state, reload, verify state persists

**Success Criteria**: Entity state saves/loads correctly in all states

**Estimated Time**: 2-3 hours

**Related Rust Learning**:
- Serde serialization
- Trait implementation for complex types
- State restoration patterns

---

### Phase 9: Polish & Effects

**Goal**: Add visual/audio feedback and refinement

**Tasks**:
- [ ] Add particle effects on awakening (optional)
- [ ] Add sound effects for state transitions
- [ ] Add visual feedback on hit (flash, shake)
- [ ] Tune animation timing for feel
- [ ] Add debug visualization for collision bounds
- [ ] Test all edge cases and transitions

**Success Criteria**: Feature feels polished and responsive

**Estimated Time**: 2-3 hours

---

### Phase 10: Documentation & Testing

**Goal**: Ensure quality and maintainability

**Tasks**:
- [ ] Run `cargo clippy` and fix warnings
- [ ] Run `cargo fmt`
- [ ] Write unit tests for state machine
- [ ] Write integration tests for save/load
- [ ] Document code with comments
- [ ] Update main CLAUDE.md with feature notes
- [ ] Playtest all interactions thoroughly

**Success Criteria**: Clean code, no warnings, all tests pass

**Estimated Time**: 2 hours

---

## Total Estimated Time

**24-30 hours** of development time across 10 phases

Can be broken into smaller work sessions:
- **Weekend 1**: Phases 1-3 (foundation + animation)
- **Weekend 2**: Phases 4-6 (state machine + timer)
- **Weekend 3**: Phases 7-9 (sleep interrupt + save + polish)
- **Weekday**: Phase 10 (testing + documentation)

## Edge Cases & Considerations

### 1. Multiple Players Hitting Simultaneously
**Behavior**: Last hit resets timer (in single-player, not an issue)

### 2. Save During Mid-Animation
**Solution**: Save current frame index, resume from exact frame on load

### 3. Entity Destroyed by Player
**Future Feature**: Could add health system, entity "dies" after many hits
**Current**: Entity is invulnerable (no damage taken)

### 4. Collision While Awakening
**Design Decision**: Full collision during awakening (safe, simple)
**Alternative**: No collision during awakening (player can walk through)

### 5. Timer During Sleep Animation
**Behavior**: Timer paused during sleep (only runs in awake state)

### 6. Spawning in Unreachable Locations
**Solution**: Verify spawn locations are accessible, not in walls

## Testing Plan

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_state_transitions() {
        let mut entity = TheEntity::new(0, 100, 100);
        assert_eq!(entity.state, EntityState::Dormant);

        entity.on_hit();
        assert_eq!(entity.state, EntityState::Awakening);

        // Simulate animation completion
        entity.complete_awakening();
        assert_eq!(entity.state, EntityState::Awake);
    }

    #[test]
    fn test_collision_bounds() {
        let entity = TheEntity::new(0, 0, 0);

        entity.state = EntityState::Dormant;
        let bounds = entity.get_collision_bounds();
        assert_eq!(bounds.height(), 64); // Full height

        entity.state = EntityState::Awake;
        let bounds = entity.get_collision_bounds();
        assert_eq!(bounds.height(), 32); // Half height
    }

    #[test]
    fn test_inactivity_timer() {
        let mut entity = TheEntity::new(0, 0, 0);
        entity.state = EntityState::Awake;

        entity.update(1.0); // 1 second
        assert_eq!(entity.inactivity_timer, 1.0);

        entity.update(29.0); // 30 seconds total
        assert_eq!(entity.state, EntityState::Sleeping);
    }
}
```

### Integration Tests

1. **Full Lifecycle Test**:
   - Start game
   - Hit dormant pyramid
   - Wait for awakening
   - Verify awake loop
   - Wait 30 seconds
   - Verify sleep animation
   - Verify return to dormant

2. **Save/Load Test**:
   - Awaken pyramid
   - F5 save
   - F9 reload
   - Verify still awake with correct animation

3. **Interrupt Test**:
   - Start sleep animation
   - Hit at various frames
   - Verify awakening restarts correctly

### Manual Testing Checklist

- [ ] Spawn locations are accessible
- [ ] Collision works in all states
- [ ] Depth sorting works (walk behind pyramid)
- [ ] Animation timing feels good
- [ ] State transitions are smooth
- [ ] Timer reset works on hit
- [ ] Sleep interrupt works reliably
- [ ] Save/load preserves all state
- [ ] No crashes or panics
- [ ] Performance is acceptable (60 FPS)

## Performance Considerations

### Rendering
- **Cost**: 4 additional sprites (negligible)
- **Depth sorting**: 4 more entities in sort (insignificant)

### Update Loop
- **Cost per entity**: ~0.01ms
  - State check
  - Timer update
  - Animation update
- **Total for 4 entities**: ~0.04ms (negligible at 60 FPS)

### Memory
- **Per entity**: ~200 bytes
- **Total for 4 entities**: ~800 bytes (insignificant)

### Collision
- **Cost**: 4 additional AABB checks per frame
- **Impact**: Negligible (AABB is very fast)

**Conclusion**: Feature has minimal performance impact

## Future Enhancements

### Potential Expansions

1. **Health System**: Entity can be "killed" after enough hits
2. **Loot Drop**: Entity drops items when destroyed
3. **Quest Integration**: Awakening all 4 triggers event
4. **Procedural Spawn**: Random locations instead of fixed
5. **Voice Lines**: Entity speaks when awakened
6. **Multiple Variants**: Different colored entities with unique behaviors
7. **Animation Events**: Particles at specific frames
8. **Difficulty Scaling**: Longer timeout at higher levels

## Rust Learning Opportunities

This feature demonstrates:

### 1. State Machines (Enums + Match)
```rust
match self.state {
    EntityState::Dormant => self.handle_dormant(),
    EntityState::Awakening => self.handle_awakening(),
    EntityState::Awake => self.handle_awake(delta_time),
    EntityState::Sleeping => self.handle_sleeping(),
}
```

### 2. Trait Implementation
- `DepthSortable` for rendering
- `StaticCollidable` for collision
- `Saveable` for persistence

### 3. Time Handling
- Delta time integration
- Timer management
- Instant vs Duration

### 4. Animation State Management
- Tracking current frame
- Detecting completion
- Reversing playback

### 5. Serialization
- Serde derive macros
- Custom serialize/deserialize
- State restoration

## References

- **State Machine Pattern**: Game Programming Patterns by Robert Nystrom
- **Animation Systems**: Game Engine Architecture by Jason Gregory
- **Rust Timers**: std::time module documentation
- **Collision Detection**: Real-Time Collision Detection by Christer Ericson

---

## Summary

The Entity awakening feature is a rich, multi-system interaction that will:

✅ Demonstrate depth sorting and rendering
✅ Showcase state machine design in Rust
✅ Integrate animation, collision, and save systems
✅ Create engaging environmental storytelling
✅ Provide excellent Rust learning opportunities

**Ready to implement!** Start with Phase 1 (depth sorting infrastructure) and work through each phase systematically.

---

## Implementation Notes

### Clarification: Progressive Awakening Mechanic

**IMPORTANT DESIGN CHANGE**: The entity requires **multiple hits** to fully awaken, not a single hit.

#### Awakening Progression

**Original Design**: One hit → full awakening animation → awake state

**Updated Design**: Multiple hits required, each advancing one frame:

```
Frame 1 (Dormant) --hit 1--> Frame 2 --hit 2--> Frame 3 --hit 3--> ... --hit 7--> Frame 8 (Awake)
       ◆                        ◆                 ◈                              ○
```

**Mechanics**:
- **8 hits required** to fully awaken (one per awakening frame)
- Each hit advances the animation by 1 frame
- If you **stop hitting**, entity begins reversing back to dormant
- Reverse rate: ~1 frame per 2 seconds (slower than awakening)
- Once fully awake (frame 8), enters normal awake loop with 30s timeout

#### State Machine Update

```rust
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum EntityState {
    Dormant,           // Frame 1, waiting for hits
    Awakening,         // Frames 1-8, progressing with hits
    ReversingToSleep,  // Frames current→1, reversing when hits stop
    Awake,             // Frames 8-13 loop, fully awakened
    ReturningToDormant // Frames 8→1, after 30s timeout in awake
}
```

#### Updated Behavior Table

| State | Hit Behavior | No Hit Behavior | Animation | Collision |
|-------|--------------|-----------------|-----------|-----------|
| **Dormant** | → Awakening (frame 2) | Stay dormant | Static frame 1 | Full |
| **Awakening** | Advance +1 frame | → ReversingToSleep | Manual frame control | Full |
| **ReversingToSleep** | → Awakening (continue from frame) | Reverse -1 frame/2s | Reverse | Full |
| **Awake** | Reset timer | After 30s → ReturningToDormant | Loop 8-13 | Partial |
| **ReturningToDormant** | → Awakening (interrupt) | Reverse to frame 1 | Reverse | Partial→Full |

#### Implementation Details

```rust
pub struct TheEntity<'a> {
    // ... existing fields ...

    // Awakening progress
    awakening_frame: usize,        // Current frame during awakening (1-8)
    last_hit_time: Instant,        // When last hit occurred
    reverse_timer: f32,            // Timer for automatic frame reversal
}

impl TheEntity<'_> {
    pub fn on_hit(&mut self) {
        match self.state {
            EntityState::Dormant => {
                self.state = EntityState::Awakening;
                self.awakening_frame = 2; // Start at frame 2
                self.last_hit_time = Instant::now();
            }
            EntityState::Awakening | EntityState::ReversingToSleep => {
                // Advance awakening
                self.awakening_frame += 1;
                self.last_hit_time = Instant::now();
                self.state = EntityState::Awakening;

                if self.awakening_frame >= 8 {
                    // Fully awakened!
                    self.state = EntityState::Awake;
                    self.start_awake_loop();
                }
            }
            EntityState::Awake => {
                // Reset inactivity timer
                self.inactivity_timer = 0.0;
            }
            EntityState::ReturningToDormant => {
                // Interrupt return, restart awakening from current frame
                self.state = EntityState::Awakening;
                self.last_hit_time = Instant::now();
            }
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        match self.state {
            EntityState::Awakening => {
                // If no hit for 2 seconds, start reversing
                if self.last_hit_time.elapsed().as_secs_f32() > 2.0 {
                    self.state = EntityState::ReversingToSleep;
                    self.reverse_timer = 0.0;
                }
            }
            EntityState::ReversingToSleep => {
                self.reverse_timer += delta_time;
                if self.reverse_timer >= 2.0 {
                    self.reverse_timer = 0.0;
                    self.awakening_frame = self.awakening_frame.saturating_sub(1);

                    if self.awakening_frame <= 1 {
                        // Back to dormant
                        self.state = EntityState::Dormant;
                    }
                }
            }
            EntityState::Awake => {
                // Normal 30s timeout logic
                self.inactivity_timer += delta_time;
                if self.inactivity_timer >= 30.0 {
                    self.state = EntityState::ReturningToDormant;
                }
            }
            EntityState::ReturningToDormant => {
                // Auto-reverse animation logic
                // ... similar to ReversingToSleep but faster
            }
            _ => {}
        }
    }
}
```

#### Tuning Parameters

These values can be adjusted for feel:

```rust
const AWAKENING_HIT_REQUIREMENT: usize = 8;  // Hits to fully awaken
const REVERSE_DELAY: f32 = 2.0;              // Seconds before reversing
const REVERSE_FRAME_DURATION: f32 = 2.0;     // Seconds per frame when reversing
const AWAKE_TIMEOUT: f32 = 30.0;             // Seconds before sleeping
const RETURN_FRAME_DURATION: f32 = 1.0;      // Faster return after timeout
```

#### Gameplay Implications

**Challenge**: Player must commit to awakening the entity with sustained attacks
**Reward**: Fully awakened entity serves as a landmark/checkpoint
**Risk**: Stopping mid-awakening wastes effort as entity reverses

**Example Scenario**:
1. Player hits dormant pyramid → Frame 2
2. Player hits 4 more times → Frame 6
3. Player leaves to fight slimes
4. Entity reverses: Frame 6 → Frame 5 → Frame 4 (over 6 seconds)
5. Player returns and hits again → Frame 5 → Frame 6 → Frame 7 → Frame 8 (awake!)

#### Visual Feedback

To make the mechanic clear to players:

- **Progress indicator**: Small dots above entity showing awakening progress (●●●○○○○○)
- **Reverse warning**: Entity flashes or pulses when starting to reverse
- **Hit feedback**: Screen shake or particle burst on each hit
- **Frame counter**: Debug display showing "Frame 4/8"

#### Testing Considerations

- Test hitting rapidly (all 8 in quick succession)
- Test hitting slowly (verify reversing triggers)
- Test interrupting reverse at each frame
- Test hitting awake entity (timer reset)
- Verify save/load preserves awakening_frame
- Edge case: What if player hits during frame transition?

---

### Development Log

**Phase Status**: Starting Phase 1 - Depth Sorting Infrastructure

**Implementation Order**:
1. Use subagent to create depth sorting system (Phase 1)
2. Use subagent to enhance animation system for manual frame control (Phase 3 partial)
3. Create TheEntity struct with progressive awakening logic (Phase 2 + 4 combined)
4. Implement partial collision (Phase 5)
5. Add state machine logic (Phases 6-7)
6. Integrate save/load (Phase 8)
7. Polish and test (Phases 9-10)

**Subagent Tasks**:
- Task 1: Implement depth sorting render system (render.rs module)
- Task 2: Enhance animation system with manual frame control and reverse playback
- Task 3: Implement TheEntity with full state machine

**Next Steps**: Launch subagent for depth sorting implementation.
