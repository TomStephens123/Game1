# Sprite Animation System Design

## Overview

This document describes the animated sprite system implemented for Game1, a 2.5D Rust game project using SDL2. The system provides frame-based sprite animation with configurable timing and state management.

## Architecture

### Core Components

#### 1. Frame (`src/sprite.rs`)
Represents a single animation frame with position and timing data.

```rust
pub struct Frame {
    pub x: i32,        // X position in sprite sheet
    pub y: i32,        // Y position in sprite sheet
    pub width: u32,    // Frame width
    pub height: u32,   // Frame height
    pub duration: Duration, // How long to display this frame
}
```

**Key Methods:**
- `new()` - Creates a frame with millisecond duration
- `to_rect()` - Converts to SDL2 Rect for rendering

#### 2. SpriteSheet (`src/sprite.rs`)
Manages texture and frame sequence animation.

```rust
pub struct SpriteSheet<'a> {
    texture: &'a Texture<'a>,
    frames: Vec<Frame>,
    current_frame: usize,
    last_frame_time: Instant,
    is_playing: bool,
    loop_animation: bool,
}
```

**Key Methods:**
- `update()` - Advances frame based on elapsed time
- `render()` / `render_flipped()` - Draws current frame to canvas
- `play()` / `pause()` / `reset()` - Animation control

**Rust Learning Points:**
- **Lifetimes**: The `<'a>` lifetime parameter ensures the SpriteSheet doesn't outlive the texture it references
- **Borrowing**: Uses immutable reference to texture to avoid ownership issues
- **Time Management**: Uses `std::time::Instant` for precise frame timing

#### 3. AnimationController (`src/animation.rs`)
Manages multiple sprite sheets and handles state transitions.

```rust
pub struct AnimationController<'a> {
    current_state: AnimationState,
    previous_state: AnimationState,
    sprite_sheets: HashMap<AnimationState, SpriteSheet<'a>>,
    state_changed: bool,
}
```

**Key Methods:**
- `set_state()` - Changes animation state
- `update()` - Updates current animation and handles transitions
- `get_current_sprite_sheet()` - Returns active sprite sheet

**Rust Learning Points:**
- **HashMap**: Efficiently maps animation states to sprite sheets
- **Enums**: `AnimationState` provides type-safe state management
- **Pattern Matching**: Used throughout for state handling

#### 4. Player (`src/player.rs`)
Integrates movement with animation system.

```rust
pub struct Player<'a> {
    pub x: i32, y: i32,           // Position
    pub width: u32, height: u32,   // Size
    pub speed: i32,                // Movement speed
    pub velocity_x: i32, velocity_y: i32, // Current velocity
    pub facing_left: bool,         // Direction
    animation_controller: AnimationController<'a>,
}
```

**Key Methods:**
- `update()` - Updates position and determines animation state
- `render()` - Draws player with current animation
- `keep_in_bounds()` - Constrains movement to window

**Animation State Logic:**
```rust
pub fn determine_animation_state(velocity_x: i32, velocity_y: i32, speed_threshold: i32) -> AnimationState {
    let total_velocity = (velocity_x.abs() + velocity_y.abs()) as i32;

    if total_velocity == 0 {
        AnimationState::Idle
    } else if total_velocity <= speed_threshold {
        AnimationState::Walking
    } else {
        AnimationState::Running
    }
}
```

## Animation States

The system currently supports three animation states:

1. **Idle**: When player is not moving (blue frames in demo)
2. **Walking**: Slow movement (green frames in demo)
3. **Running**: Fast movement (red frames in demo)

States are determined by total velocity magnitude compared to the player's base speed.

## Configuration System

### JSON Format (`assets/config/player_animations.json`)

```json
{
  "frame_width": 32,
  "frame_height": 32,
  "animations": {
    "Idle": {
      "frames": [
        { "x": 0, "y": 0, "duration_ms": 500 },
        { "x": 32, "y": 0, "duration_ms": 500 }
      ],
      "loop_animation": true
    }
  }
}
```

**Rust Learning Points:**
- **Serde**: Automatic JSON serialization/deserialization
- **Error Handling**: Proper `Result<T, E>` usage for file loading
- **Traits**: `Serialize`, `Deserialize`, `Hash`, `PartialEq` implementation

## Asset Organization

```
assets/
├── sprites/
│   └── player/          # Player sprite sheets (.png files)
├── config/
│   └── player_animations.json  # Animation definitions
```

## Usage Example

```rust
// Load configuration
let config = AnimationConfig::load_from_file("assets/config/player_animations.json")?;

// Create texture (placeholder or loaded from file)
let texture = create_placeholder_texture(&texture_creator)?;

// Setup animations
let controller = setup_player_animations(&texture, &config)?;

// Create player with animations
let mut player = Player::new(x, y, width, height, speed);
player.set_animation_controller(controller);

// In game loop:
player.update(&keyboard_state);  // Updates position and animation
player.render(&mut canvas)?;     // Draws animated player
```

## Technical Considerations

### Performance
- **Frame Timing**: Uses `Instant::elapsed()` for precise timing
- **Memory**: Minimal allocation after setup; reuses frame vectors
- **Rendering**: Single texture copy per frame using SDL2's hardware acceleration

### Flexibility
- **Configurable**: JSON-driven animation definitions
- **Extensible**: Easy to add new animation states
- **Modular**: Separate concerns for sprites, animations, and player logic

### Rust Ownership Model
- **Lifetimes**: Ensures texture references remain valid
- **Borrowing**: Prevents data races through compile-time checks
- **Move Semantics**: Efficient transfer of large structs

## Future Enhancements

1. **Real Sprite Assets**: Replace placeholder texture with actual 16-bit pixel art
2. **Animation Blending**: Smooth transitions between states
3. **Sound Integration**: Audio cues for animation events
4. **Multiple Characters**: Support for different character types
5. **Animation Events**: Trigger gameplay events at specific frames
6. **Asset Caching**: Resource manager for efficient texture loading

## Learning Achievements

This implementation demonstrates several key Rust concepts:

- ✅ **Ownership and Borrowing**: Safe memory management without garbage collection
- ✅ **Lifetimes**: Ensuring references remain valid
- ✅ **Pattern Matching**: Type-safe state handling with enums
- ✅ **Error Handling**: Proper use of `Result<T, E>` and `?` operator
- ✅ **Traits**: Code reuse and abstraction
- ✅ **Modules**: Code organization and encapsulation
- ✅ **External Crates**: Integration with SDL2 and Serde
- ✅ **Time Management**: Precise timing for smooth animations
- ✅ **Data Structures**: HashMap for efficient lookups

This sprite system provides a solid foundation for 2.5D game development while showcasing idiomatic Rust programming patterns.