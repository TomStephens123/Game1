# Manual Frame Control - Quick Reference

## Overview

The manual frame control API allows precise control over animation frames, enabling frame-by-frame progression independent of timers. This is essential for animations driven by game events rather than time.

**File:** `/Users/tomstephens/Documents/GitHub/Game1/src/sprite.rs`

## API Reference

### Control Methods

#### `pause()`
Pauses animation playback. Frames will not automatically advance.

```rust
sprite_sheet.pause();
```

**Use when:**
- Taking manual control of frame progression
- Freezing animation for game effects
- Implementing cutscenes or dialogue

---

#### `play()`
Resumes automatic animation playback based on frame timers.

```rust
sprite_sheet.play();
```

**Use when:**
- Returning to timer-based animation
- Restarting a paused animation
- Initial animation start

---

#### `set_frame(frame_index: usize)`
Manually jumps to a specific frame. Includes bounds checking.

```rust
// Jump to frame 5
sprite_sheet.set_frame(5);

// Out of bounds? Clamped to last frame automatically
sprite_sheet.set_frame(999); // Safe - clamps to last frame
```

**Behavior:**
- If `frame_index >= frame_count()`, clamps to last frame
- Resets frame timer to prevent immediate auto-advance
- Does not change playing/paused state

**Use when:**
- Manual frame-by-frame progression
- Syncing animations to external events
- Implementing custom animation logic

---

### Query Methods

#### `get_current_frame() -> usize`
Returns the zero-based index of the currently displayed frame.

```rust
let current = sprite_sheet.get_current_frame();
println!("On frame {}", current);
```

**Use when:**
- Checking animation progress
- Synchronizing multiple animations
- Saving animation state

---

#### `frame_count() -> usize`
Returns the total number of frames in the animation.

```rust
let total = sprite_sheet.frame_count();

// Example: Progress percentage
let progress = (sprite_sheet.get_current_frame() as f32 / total as f32) * 100.0;
```

**Use when:**
- Bounds checking before `set_frame()`
- Calculating animation progress
- Looping through all frames manually

---

#### `is_playing() -> bool`
Returns whether the animation is currently playing (true) or paused (false).

```rust
if sprite_sheet.is_playing() {
    println!("Animation is running");
} else {
    println!("Animation is paused");
}
```

**Use when:**
- Checking if pause is needed
- Conditional logic based on playback state
- UI indicators

---

## Usage Patterns

### Pattern 1: Manual Frame Progression

Advance frames based on game events rather than time.

```rust
// Initialize
sprite_sheet.pause();
sprite_sheet.set_frame(0);

// In game update loop:
if player_pressed_button {
    let current = sprite_sheet.get_current_frame();
    let next = (current + 1) % sprite_sheet.frame_count();
    sprite_sheet.set_frame(next);
}
```

**Example Use Case:** The Entity awakening - advance frames as player approaches.

---

### Pattern 2: Animation Scrubbing

Preview animations by scrubbing through frames.

```rust
sprite_sheet.pause();

// User drags slider from 0.0 to 1.0
let frame_index = (slider_value * (sprite_sheet.frame_count() - 1) as f32) as usize;
sprite_sheet.set_frame(frame_index);
```

**Example Use Case:** Animation editor or debug tool.

---

### Pattern 3: Synchronized Animations

Keep multiple sprites in perfect sync.

```rust
let target_frame = master_sprite.get_current_frame();

for sprite in &mut synchronized_sprites {
    sprite.pause();
    sprite.set_frame(target_frame);
}
```

**Example Use Case:** Multiple entities performing synchronized attack.

---

### Pattern 4: Event-Driven Animation

Trigger specific frames on game events.

```rust
match game_event {
    GameEvent::PowerUp => sprite_sheet.set_frame(5), // Jump to "powered up" frame
    GameEvent::Damaged => sprite_sheet.set_frame(2), // Jump to "hurt" frame
    GameEvent::Normal => sprite_sheet.play(),        // Resume normal animation
}
```

**Example Use Case:** Character state changes with instant visual feedback.

---

### Pattern 5: Save/Load Animation State

Persist and restore exact animation frame.

```rust
// Saving
let saved_frame = sprite_sheet.get_current_frame();
let saved_playing = sprite_sheet.is_playing();
save_data.animation_frame = saved_frame;
save_data.animation_playing = saved_playing;

// Loading
sprite_sheet.set_frame(save_data.animation_frame);
if save_data.animation_playing {
    sprite_sheet.play();
} else {
    sprite_sheet.pause();
}
```

**Example Use Case:** Game save files that preserve animation state.

---

## Safety & Best Practices

### Bounds Checking
`set_frame()` automatically clamps out-of-bounds indices:

```rust
let sprite = SpriteSheet::new(texture, vec![frame1, frame2, frame3]); // 3 frames

sprite.set_frame(0);   // ✅ Valid: frame 0
sprite.set_frame(2);   // ✅ Valid: frame 2
sprite.set_frame(5);   // ✅ Safe: clamped to frame 2 (last frame)
sprite.set_frame(999); // ✅ Safe: clamped to frame 2 (last frame)
```

**No crashes, no panics!**

---

### Timer Reset Behavior
When you manually set a frame, the frame timer resets:

```rust
sprite_sheet.play();
// ... time passes, about to advance to next frame ...

sprite_sheet.set_frame(0); // Timer resets!
// Frame will stay at 0 for full duration before auto-advancing
```

**Why:** Prevents the frame from immediately advancing after you set it manually.

---

### Mixing Manual and Automatic Playback

Manual control and automatic playback coexist safely:

```rust
// Start with automatic playback
sprite_sheet.play();

// Take manual control
sprite_sheet.pause();
sprite_sheet.set_frame(5);
// ... do manual things ...

// Return to automatic playback
sprite_sheet.play(); // Resumes from frame 5
```

**Best Practice:** Use `pause()` before manual frame manipulation to prevent conflicts.

---

## Common Pitfalls

### ❌ Forgetting to Pause

```rust
// BAD: Manual control while animation is playing
sprite_sheet.set_frame(3);
// Timer might immediately advance to frame 4!
```

```rust
// GOOD: Pause first
sprite_sheet.pause();
sprite_sheet.set_frame(3);
// Frame stays at 3 until you call play()
```

---

### ❌ Manual Looping Without Frame Count Check

```rust
// BAD: Can exceed frame count
let next = sprite_sheet.get_current_frame() + 1;
sprite_sheet.set_frame(next); // What if next >= frame_count()?
```

```rust
// GOOD: Use modulo for looping
let next = (sprite_sheet.get_current_frame() + 1) % sprite_sheet.frame_count();
sprite_sheet.set_frame(next); // Always valid
```

---

### ❌ Not Checking Frame Count Before Loops

```rust
// BAD: Assumes frame count
for i in 0..10 {
    sprite_sheet.set_frame(i); // What if only 5 frames?
}
```

```rust
// GOOD: Use actual frame count
for i in 0..sprite_sheet.frame_count() {
    sprite_sheet.set_frame(i); // Always valid
}
```

---

## AnimationController Integration

These methods work on `SpriteSheet`. To use with `AnimationController`:

```rust
// Get mutable access to current sprite sheet
if let Some(sprite_sheet) = animation_controller.get_current_sprite_sheet_mut() {
    sprite_sheet.pause();
    sprite_sheet.set_frame(5);
}
```

**Note:** `get_current_sprite_sheet_mut()` doesn't exist in the current API. You'll need to access sprite sheets directly or add this method to `AnimationController`.

**Workaround:** Store sprite sheets separately if you need manual control:

```rust
pub struct Entity<'a> {
    sprite_sheet: SpriteSheet<'a>, // Direct access
    // OR
    animation_controller: AnimationController<'a>, // For complex state management
}
```

---

## Future Enhancement: Reverse Playback

Currently planned but not implemented. See TODO in `/Users/tomstephens/Documents/GitHub/Game1/src/sprite.rs`:

```rust
// TODO: Future Enhancement - Reverse Playback
// Add support for reverse playback direction independent of PingPong mode.
// This would allow animations to play backwards on demand, useful for:
// - Rewinding animations
// - "Undoing" visual effects
// - Symmetrical enter/exit animations
//
// Proposed API:
// pub fn set_direction(&mut self, direction: AnimationDirection)
// pub fn reverse(&mut self) - toggle current direction
//
// where AnimationDirection is Forward | Reverse
// This differs from PlayDirection (used by PingPong) which auto-switches.
```

**Current Workaround:** Manually reverse frames:

```rust
sprite_sheet.pause();
for i in (0..sprite_sheet.frame_count()).rev() {
    sprite_sheet.set_frame(i);
    // Render...
}
```

---

## Testing

Unit tests verify the new functionality:

**File:** `/Users/tomstephens/Documents/GitHub/Game1/src/sprite.rs` (lines 341-403)

```bash
# Run sprite tests
cargo test sprite::tests

# Expected output:
# test sprite::tests::test_frame_bounds_checking ... ok
# test sprite::tests::test_frame_creation ... ok
# test sprite::tests::test_pause_stops_playback ... ok
```

**Note:** Full integration tests require SDL2 rendering context and will be tested through gameplay.

---

## Summary

The manual frame control API provides:

✅ **Flexibility** - Mix automatic and manual control
✅ **Safety** - Bounds checking prevents crashes
✅ **Simplicity** - Clear, well-documented methods
✅ **Power** - Enable complex animation logic

**Key Methods:**
- `pause()` / `play()` - Control playback
- `set_frame()` - Jump to specific frame
- `get_current_frame()` - Query current frame
- `frame_count()` - Get total frames
- `is_playing()` - Check playback state

**When to Use:**
- Game event-driven animations (Entity awakening)
- Frame-perfect gameplay mechanics
- Animation synchronization
- Custom animation logic

This feature enables the Entity awakening mechanic and opens possibilities for many other frame-precise animations!
