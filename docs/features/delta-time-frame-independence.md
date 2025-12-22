# Delta-Time Based Frame Independence

## Overview
Currently, the game uses a fixed sleep-based frame limiter (60 FPS) without delta-time compensation. This creates frame-rate dependent behavior where game logic is tied to how fast frames are processed rather than actual elapsed time.

## Why This Is Worth Doing

### Current Problems
1. **Inconsistent gameplay across hardware** - Faster machines that process frames quicker than 16.67ms still get slowed to 60 FPS, while slower machines that take longer will run the game in slow motion
2. **Imprecise timing** - `thread::sleep` is not guaranteed to be accurate, leading to variable actual frame rates
3. **No high-refresh support** - Users with 120Hz/144Hz/240Hz monitors cannot benefit from their hardware
4. **Frame-dependent physics** - Movement speeds and game logic are tied to frame count rather than real time

### Benefits of Delta-Time
1. **Consistent gameplay** - Character movement, enemy AI, and animations run at the same speed regardless of frame rate
2. **Future-proof** - Works correctly on any hardware, from slow to very fast
3. **Configurable frame caps** - Users can choose their preferred FPS limit (60/120/144/unlimited)
4. **Better input responsiveness** - Higher frame rates provide lower input latency even in pixel art games
5. **Professional standard** - All modern games use delta-time for frame independence

## Implementation Plan

### Phase 1: Add Time Tracking Infrastructure
- [ ] Add `std::time::Instant` to track frame timing in main game loop (src/main.rs:~980)
- [ ] Calculate delta-time (elapsed time since last frame) in seconds as `f32`
- [ ] Store `last_frame_time: Instant` in the Game struct
- [ ] Add `target_frame_time: Duration` for configurable FPS cap

### Phase 2: Update Game Loop Structure
- [ ] Measure time at start of each frame
- [ ] Calculate delta-time from previous frame
- [ ] Pass delta-time to all update methods
- [ ] Replace fixed `thread::sleep` with smart sleep that accounts for actual frame processing time:
  ```rust
  let frame_time = frame_start.elapsed();
  if frame_time < target_frame_time {
      thread::sleep(target_frame_time - frame_time);
  }
  ```

### Phase 3: Propagate Delta-Time Through Update Chain
- [ ] Update `Game::update()` signature to accept `delta_time: f32` (src/main.rs)
- [ ] Update `Player::update()` to use delta-time for movement (src/player.rs)
  - Multiply velocities by delta-time
  - Update animation timers using delta-time
- [ ] Update `World::update()` to accept and pass delta-time (src/world.rs)
- [ ] Update enemy systems to use delta-time (src/slime.rs and other enemies)
- [ ] Update projectile systems to use delta-time (src/projectile.rs)
- [ ] Update particle systems if any exist

### Phase 4: Fix Movement Calculations
Current pattern (frame-dependent):
```rust
position.x += velocity; // Moves `velocity` pixels per frame
```

New pattern (time-independent):
```rust
position.x += velocity * delta_time; // Moves `velocity` pixels per second
```

- [ ] Update all position/velocity calculations to multiply by delta_time
- [ ] Adjust velocity constants to be "per second" instead of "per frame"
  - Example: if something moves 5 pixels/frame at 60 FPS, that's 300 pixels/second
- [ ] Update animation frame timers to accumulate delta-time

### Phase 5: Add Configurable Frame Rate Cap
- [ ] Add FPS cap option to settings/config system
- [ ] Support common options: 60, 120, 144, 240, Unlimited
- [ ] Default to 60 FPS for energy efficiency but allow users to change
- [ ] Consider exposing via debug menu for easy testing

### Phase 6: Testing & Validation
- [ ] Test at various FPS caps (30, 60, 120, unlimited)
- [ ] Verify movement speeds are consistent across all frame rates
- [ ] Check that animations play at correct speeds
- [ ] Ensure physics interactions remain consistent
- [ ] Test on different hardware if possible

## Technical Notes

### Delta-Time Calculation
```rust
let current_time = Instant::now();
let delta_time = current_time.duration_since(last_frame_time).as_secs_f32();
last_frame_time = current_time;
```

### Smart Frame Limiting
```rust
let target_fps = 60;
let target_frame_time = Duration::from_secs_f32(1.0 / target_fps as f32);

let frame_start = Instant::now();
// ... update and render ...
let frame_time = frame_start.elapsed();

if frame_time < target_frame_time {
    thread::sleep(target_frame_time - frame_time);
}
```

### Velocity Adjustment Example
If a player currently moves at `5.0` pixels per frame at 60 FPS:
- Per-second velocity: `5.0 * 60 = 300.0` pixels/second
- Update code: `position.x += 300.0 * delta_time`
- At 60 FPS (delta_time ≈ 0.0167): `300.0 * 0.0167 ≈ 5.0` pixels (same as before)
- At 120 FPS (delta_time ≈ 0.0083): `300.0 * 0.0083 ≈ 2.5` pixels (but twice as many frames)

## Related Files
- `src/main.rs` - Main game loop (lines ~980-1028)
- `src/player.rs` - Player movement updates
- `src/world.rs` - World update coordination
- `src/slime.rs` - Enemy movement
- `src/projectile.rs` - Projectile movement
- Any animation systems

## Priority
**Medium-High** - Not critical for basic functionality, but important for professional quality and supporting modern hardware. Should be implemented before adding complex physics or timing-dependent features.
