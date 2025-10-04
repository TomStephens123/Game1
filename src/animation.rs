use crate::sprite::{Frame, SpriteSheet};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnimationMode {
    #[serde(rename = "loop")]
    Loop,
    #[serde(rename = "ping_pong")]
    PingPong,
    #[serde(rename = "once")]
    Once,
}

impl Default for AnimationMode {
    fn default() -> Self {
        AnimationMode::Loop
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlayDirection {
    Forward,
    Backward,
}

/// Animation state is now represented as a String, allowing unlimited extensibility.
/// Each entity can define its own states without modifying this core animation system.
///
/// Common pattern:
/// - "idle" - Character standing still
/// - "running" / "walking" - Character moving
/// - "attack" / "jump" / etc. - Action-specific animations
///
/// Entity-specific states:
/// - "slime_idle", "goblin_punch", "dragon_breath" - Whatever you need!
pub type AnimationState = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    South = 0,
    SouthEast = 1,
    East = 2,
    NorthEast = 3,
    North = 4,
    NorthWest = 5,
    West = 6,
    SouthWest = 7,
}

impl Direction {
    pub fn from_velocity(vel_x: i32, vel_y: i32) -> Self {
        if vel_x == 0 && vel_y == 0 {
            return Direction::South; // Default direction when not moving
        }

        // Normalize velocity to determine direction
        // SDL2 coordinate system: positive Y is down
        match (vel_x.signum(), vel_y.signum()) {
            (0, 1) => Direction::South,       // Down
            (1, 1) => Direction::SouthEast,   // Down-Right
            (1, 0) => Direction::East,        // Right
            (1, -1) => Direction::NorthEast,  // Up-Right
            (0, -1) => Direction::North,      // Up
            (-1, -1) => Direction::NorthWest, // Up-Left
            (-1, 0) => Direction::West,       // Left
            (-1, 1) => Direction::SouthWest,  // Down-Left
            _ => Direction::South,            // Fallback
        }
    }

    pub fn to_row(&self) -> i32 {
        *self as i32
    }
}

// AnimationState (String) already implements Default (empty string)
// Entities should explicitly set their initial state rather than relying on a default

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationConfig {
    pub frame_width: u32,
    pub frame_height: u32,
    pub animations: HashMap<AnimationState, AnimationData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationData {
    pub frames: Vec<FrameData>,
    #[serde(default)]
    pub animation_mode: AnimationMode,
    // Keep for backward compatibility, but prefer animation_mode
    #[serde(default)]
    pub loop_animation: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameData {
    pub x: i32,
    pub y: i32,
    pub duration_ms: u64,
}

impl FrameData {
    pub fn to_frame(&self, width: u32, height: u32) -> Frame {
        Frame::new(self.x, self.y, width, height, self.duration_ms)
    }
}

/// AnimationController manages all animations for a single entity.
///
/// Game Dev Concept: Think of this as a "jukebox" for your entity's animations.
/// You load different animations (sprite sheets) into it, then tell it which one to play
/// by changing the state. The controller handles all the details of switching between
/// animations and keeping track of what's currently playing.
///
/// The 'a lifetime ensures that textures live long enough - the controller can't outlive
/// the textures it references. This is Rust's compile-time safety preventing crashes!
pub struct AnimationController<'a> {
    current_state: AnimationState,
    previous_state: AnimationState,
    sprite_sheets: HashMap<AnimationState, SpriteSheet<'a>>,
    state_changed: bool,
}

impl<'a> AnimationController<'a> {
    /// Creates a new empty AnimationController.
    /// You'll add animations to it using add_animation() or use AnimationConfig::create_controller()
    pub fn new() -> Self {
        AnimationController {
            current_state: String::new(),
            previous_state: String::new(),
            sprite_sheets: HashMap::new(),
            state_changed: false,
        }
    }

    /// Adds an animation to this controller.
    ///
    /// Game Dev Pattern: You typically load all animations during initialization,
    /// then switch between them during gameplay using set_state().
    ///
    /// Example:
    /// ```
    /// controller.add_animation("idle", idle_sprite_sheet);
    /// controller.add_animation("running", running_sprite_sheet);
    /// ```
    pub fn add_animation(&mut self, state: AnimationState, sprite_sheet: SpriteSheet<'a>) {
        self.sprite_sheets.insert(state, sprite_sheet);
    }

    /// Changes the current animation state.
    ///
    /// Game Dev Pattern: This is how you respond to gameplay events!
    /// - Player presses move key? set_state("running")
    /// - Player stops? set_state("idle")
    /// - Player attacks? set_state("attack")
    ///
    /// The state_changed flag ensures the new animation resets to frame 0
    /// when update() is next called. See the explanation in the docs for why
    /// we don't reset immediately here!
    ///
    /// # Safety Note
    /// If the state doesn't exist in the controller, a warning is printed
    /// but the controller continues using the previous state. This prevents
    /// crashes from typos while still alerting you to the problem.
    pub fn set_state(&mut self, new_state: AnimationState) {
        if new_state != self.current_state {
            // Defensive programming: check if state exists
            if !self.sprite_sheets.contains_key(&new_state) {
                eprintln!(
                    "⚠️  WARNING: Animation state '{}' not found in controller!\n\
                     Available states: {:?}\n\
                     Keeping current state: '{}'",
                    new_state,
                    self.sprite_sheets.keys().collect::<Vec<_>>(),
                    self.current_state
                );
                return; // Don't change state
            }

            self.previous_state = self.current_state.clone();
            self.current_state = new_state;
            self.state_changed = true;
        }
    }

    /// Attempts to set the state, falling back to a default if the state doesn't exist.
    ///
    /// Game Dev Pattern: Safe state transitions!
    /// This is useful when you're not sure if a state exists (e.g., optional animations).
    ///
    /// Example:
    /// ```
    /// // Try to play special attack, fall back to regular attack if it doesn't exist
    /// controller.set_state_or_fallback("special_attack".to_string(), "attack".to_string());
    /// ```
    #[allow(dead_code)] // Public API for users
    pub fn set_state_or_fallback(&mut self, new_state: AnimationState, fallback: AnimationState) {
        if self.sprite_sheets.contains_key(&new_state) {
            self.set_state(new_state);
        } else {
            eprintln!(
                "⚠️  Animation state '{}' not found, using fallback '{}'",
                new_state, fallback
            );
            self.set_state(fallback);
        }
    }

    /// Returns a list of all available animation states in this controller.
    ///
    /// Useful for debugging or validating your code.
    #[allow(dead_code)] // Public API for users
    pub fn available_states(&self) -> Vec<String> {
        self.sprite_sheets.keys().cloned().collect()
    }

    /// Updates the current animation, advancing frames based on time.
    ///
    /// Game Dev Pattern: Call this once per frame in your game loop!
    /// This is where the "state_changed flag pattern" happens:
    ///
    /// 1. If state changed, reset the new animation to frame 0 and start playing
    /// 2. Always advance the current animation's frame timer
    ///
    /// This ensures smooth transitions and proper frame timing.
    pub fn update(&mut self) {
        if self.state_changed {
            if let Some(sprite_sheet) = self.sprite_sheets.get_mut(&self.current_state) {
                sprite_sheet.reset();
                sprite_sheet.play();
            }
            self.state_changed = false;
        }

        if let Some(sprite_sheet) = self.sprite_sheets.get_mut(&self.current_state) {
            sprite_sheet.update();
        }
    }

    pub fn get_current_sprite_sheet(&self) -> Option<&SpriteSheet<'a>> {
        self.sprite_sheets.get(&self.current_state)
    }

    pub fn current_state(&self) -> &AnimationState {
        &self.current_state
    }

    pub fn is_animation_finished(&self) -> bool {
        if let Some(sprite_sheet) = self.sprite_sheets.get(&self.current_state) {
            sprite_sheet.is_finished()
        } else {
            false
        }
    }
}

impl AnimationConfig {
    /// Loads animation configuration from a JSON file.
    ///
    /// Game Dev Pattern: Data-driven design!
    /// By storing animation data in JSON files, artists and designers can tweak
    /// frame timings and add new animations without touching code.
    ///
    /// Example JSON structure:
    /// ```json
    /// {
    ///   "frame_width": 32,
    ///   "frame_height": 32,
    ///   "animations": {
    ///     "idle": {
    ///       "frames": [
    ///         { "x": 0, "y": 0, "duration_ms": 300 }
    ///       ],
    ///       "animation_mode": "loop"
    ///     }
    ///   }
    /// }
    /// ```
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: AnimationConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// Creates an AnimationController with all animations from this config.
    ///
    /// **NEW FACTORY FUNCTION!** This eliminates the repetitive boilerplate
    /// of manually creating each sprite sheet and adding it to the controller.
    ///
    /// Game Dev Pattern: Factory Method
    /// Instead of manually setting up each animation:
    /// ```
    /// let frames = config.create_frames("idle");
    /// let mut sprite_sheet = SpriteSheet::new(texture, frames);
    /// sprite_sheet.set_loop(config.should_loop("idle"));
    /// controller.add_animation("idle", sprite_sheet);
    /// // ... repeat for each state ...
    /// ```
    ///
    /// You now just do:
    /// ```
    /// let controller = config.create_controller(texture, &["idle", "running"])?;
    /// ```
    ///
    /// This is called a "factory method" - a function that creates complex objects
    /// with all their parts properly configured. Much less error-prone!
    ///
    /// # Parameters
    /// - `texture`: The sprite sheet texture to use for all animations
    /// - `states`: Which animation states to load from the config
    ///
    /// # Errors
    /// Returns an error if any requested state doesn't exist in the config
    /// Returns a list of all animation states available in this config.
    ///
    /// Game Dev Pattern: Defensive programming!
    /// Use this to validate your code references states that actually exist.
    ///
    /// Example:
    /// ```
    /// let config = AnimationConfig::load_from_file("player.json")?;
    /// println!("Available animations: {:?}", config.available_states());
    /// // Output: ["idle", "running", "attack"]
    /// ```
    pub fn available_states(&self) -> Vec<String> {
        self.animations.keys().cloned().collect()
    }

    /// Checks if a specific animation state exists in this config.
    ///
    /// Game Dev Pattern: Validate before you use!
    /// Call this before setting a state to avoid runtime errors.
    ///
    /// Example:
    /// ```
    /// if config.has_state("special_attack") {
    ///     controller.set_state("special_attack".to_string());
    /// } else {
    ///     controller.set_state("attack".to_string()); // Fallback
    /// }
    /// ```
    #[allow(dead_code)] // Public API for users
    pub fn has_state(&self, state: &str) -> bool {
        self.animations.contains_key(state)
    }

    pub fn create_controller<'a>(
        &self,
        texture: &'a sdl2::render::Texture<'a>,
        states: &[&str],
    ) -> Result<AnimationController<'a>, String> {
        let mut controller = AnimationController::new();

        // Validate ALL states before creating any sprite sheets
        // This ensures we fail fast with a helpful error message
        for state_name in states {
            if !self.animations.contains_key(*state_name) {
                // Generate a helpful error with suggestions
                let available_states: Vec<String> = self.animations.keys()
                    .map(|s| format!("\"{}\"", s))
                    .collect();

                return Err(format!(
                    "Animation state '{}' not found in config!\n\
                     Available states: [{}]\n\
                     Hint: Check for typos - state names are case-sensitive.",
                    state_name,
                    available_states.join(", ")
                ));
            }
        }

        // All states validated, now create sprite sheets
        for state_name in states {
            let state = state_name.to_string();
            let frames = self.create_frames(&state);

            if frames.is_empty() {
                return Err(format!(
                    "No frames found for animation state '{}' in config",
                    state_name
                ));
            }

            let mut sprite_sheet = SpriteSheet::new(texture, frames);
            sprite_sheet.set_loop(self.should_loop(&state));
            sprite_sheet.set_animation_mode(self.get_animation_mode(&state));
            controller.add_animation(state, sprite_sheet);
        }

        Ok(controller)
    }

    pub fn create_frames(&self, state: &AnimationState) -> Vec<Frame> {
        if let Some(animation_data) = self.animations.get(state) {
            animation_data
                .frames
                .iter()
                .map(|frame_data| frame_data.to_frame(self.frame_width, self.frame_height))
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn should_loop(&self, state: &AnimationState) -> bool {
        if let Some(animation_data) = self.animations.get(state) {
            // Check if we have legacy loop_animation setting first for backward compatibility
            if let Some(legacy_loop) = animation_data.loop_animation {
                return legacy_loop;
            }
            // Otherwise use the new animation_mode
            match animation_data.animation_mode {
                AnimationMode::Loop | AnimationMode::PingPong => true,
                AnimationMode::Once => false,
            }
        } else {
            true // Default to looping
        }
    }

    pub fn get_animation_mode(&self, state: &AnimationState) -> AnimationMode {
        self.animations
            .get(state)
            .map(|data| {
                // Handle backward compatibility
                if let Some(legacy_loop) = data.loop_animation {
                    if legacy_loop {
                        AnimationMode::Loop
                    } else {
                        AnimationMode::Once
                    }
                } else {
                    data.animation_mode.clone()
                }
            })
            .unwrap_or(AnimationMode::Loop)
    }
}

/// Helper function to determine animation state from velocity.
///
/// Game Dev Pattern: State determination logic!
/// This is a common pattern - looking at an entity's physics (velocity)
/// and determining what animation should play.
///
/// In a more complex game, you might have:
/// - velocity magnitude determines walk vs run vs sprint
/// - direction affects which animation variant
/// - additional state like "is_grounded" or "is_attacking"
///
/// For now, simple: moving = "running", not moving = "idle"
pub fn determine_animation_state(
    velocity_x: i32,
    velocity_y: i32,
    _speed_threshold: i32,
) -> AnimationState {
    let total_velocity = (velocity_x.abs() + velocity_y.abs()) as i32;

    if total_velocity == 0 {
        "idle".to_string()
    } else {
        "running".to_string()
    }
}
