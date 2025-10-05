# Animation System: Important Implementation Notes

## Critical: Update Order for "Once" Animations

### The Problem

When using "once" animation mode (animations that play through once and stop), the order of operations in your update loop is critical:

**❌ WRONG Order (causes animations to not replay):**
```rust
pub fn update(&mut self) {
    // Check if animation finished
    if self.animation_controller.is_animation_finished() {
        // Change state
    }

    // Update animation controller AFTER checking
    self.animation_controller.update();  // Too late!
}
```

**✅ CORRECT Order:**
```rust
pub fn update(&mut self) {
    // Update animation controller FIRST
    self.animation_controller.update();

    // THEN check if animation finished
    if self.animation_controller.is_animation_finished() {
        // Change state
    }
}
```

### Why This Matters

The `AnimationController::update()` method:
1. Checks if state changed via `set_state()`
2. If yes, calls `sprite_sheet.reset()` to restart the animation
3. Then advances the animation frame

If you check `is_animation_finished()` BEFORE calling `update()`:
- The sprite sheet hasn't been reset yet
- "Once" animations from the previous play are still marked as finished
- Returns `true` even though you just set a new state
- Animation gets skipped immediately

### Implementation

See `src/slime.rs:80-85` for the correct implementation with detailed comments.

### When This Applies

- Any entity using "once" animation mode (damage, death, attack animations)
- Animations that need to replay on demand
- State machines that check `is_animation_finished()`

### Related Systems

- `src/animation.rs` - AnimationController
- `src/sprite.rs` - SpriteSheet with animation modes
