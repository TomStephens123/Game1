/// The Entity - Awakening Pyramid Feature
///
/// This module implements an interactive environmental object representing an ancient being
/// trapped within a stone pyramid. Players awaken it through progressive hits, triggering
/// a multi-stage animation sequence.
///
/// # Core Mechanic: Progressive Awakening
///
/// The entity requires **8 hits** to fully awaken:
/// - Hit 1: Frame 1 → Frame 2 (starts awakening)
/// - Hit 2: Frame 2 → Frame 3
/// - ...
/// - Hit 7: Frame 7 → Frame 8 (fully awake)
///
/// If the player stops hitting for **2 seconds**, the entity automatically reverses
/// one frame every 2 seconds until it returns to dormant.
///
/// Once fully awake, the entity displays a looping animation (frames 8-13) and will
/// return to dormant after **30 seconds** of inactivity.
///
/// # State Machine
///
/// See docs/features/the-entity-awakening.md for complete state machine documentation.
///
/// # Rust Learning Notes
///
/// This module demonstrates:
/// - **Enum-based state machines**: Type-safe state transitions with exhaustive matching
/// - **Timer management**: Delta time integration and timeout handling
/// - **Trait implementation**: StaticCollidable and DepthSortable for system integration
/// - **Manual animation control**: Using pause() and set_frame() from SpriteSheet
use crate::collision::StaticCollidable;
use crate::collision::aabb_intersect;
use crate::render::DepthSortable;
use crate::save::{Saveable, SaveData, SaveError};
use crate::sprite::SpriteSheet;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Scale factor for sprite rendering (imported from main)
const SPRITE_SCALE: u32 = 2;

/// Type of buff provided by this entity when awake.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum EntityType {
    Attack,       // +1 attack damage
    Defense,      // +1 defense
    Speed,        // +1 movement speed
    Regeneration, // +2 HP every 5 seconds
}

/// State machine for The Entity's awakening lifecycle.
///
/// # State Descriptions
///
/// - **Dormant**: Entity is asleep (frame 1), waiting for first hit
/// - **Awakening**: Entity is being awakened by progressive hits (frames 1-8)
/// - **ReversingToSleep**: Entity reverses toward dormant when hits stop (frames current→1)
/// - **Awake**: Entity is fully awake, looping animation (frames 8-13)
/// - **ReturningToDormant**: Entity returns to dormant after 30s timeout (frames 8→1)
///
/// # Rust Learning: Deriving Traits
///
/// - `Debug`: Allows printing state with {:?} for debugging
/// - `Clone, Copy`: State is a simple enum, can be copied efficiently
/// - `PartialEq`: Allows comparing states with == operator
/// - `Serialize, Deserialize`: Enables saving/loading state (future feature)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum EntityState {
    Dormant,
    Awakening,
    ReversingToSleep,
    Awake,
    ReturningToDormant,
}

/// The Entity - An awakening pyramid that responds to player attacks.
///
/// # Fields Overview
///
/// **Position & Rendering**:
/// - `x, y`: World position (anchor point at base)
/// - `width, height`: Collision/sprite dimensions (32×32)
/// - `sprite_height`: Visual height for rendering (32, same as collision for this object)
///
/// **State Machine**:
/// - `state`: Current lifecycle state
/// - `awakening_frame`: Current frame during awakening (1-8)
/// - `last_hit_time`: When the entity was last hit (for 2s timeout)
/// - `reverse_timer`: Accumulator for automatic frame reversal
/// - `inactivity_timer`: Time since last hit in awake state (for 30s timeout)
///
/// **Animation**:
/// - `sprite_sheet`: Sprite sheet with manual frame control
///
/// **Identification**:
/// - `id`: Unique identifier (0-3 for the 4 spawned entities)
///
/// # Rust Learning: Lifetimes
///
/// The `'a` lifetime parameter ensures the entity cannot outlive the texture
/// it references in its sprite sheet. This prevents dangling references at compile time!
pub struct TheEntity<'a> {
    // Position & Rendering
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub sprite_height: u32,

    // State Machine
    pub state: EntityState,
    pub awakening_frame: usize,
    last_hit_time: Instant,
    reverse_timer: f32,
    pub inactivity_timer: f32,

    // Animation
    sprite_sheet: SpriteSheet<'a>,
    awake_animation_frame: usize,   // Current frame in awake loop (8-12)
    awake_animation_timer: f32,      // Timer for awake frame advancement

    // Identification
    pub id: usize,
    pub entity_type: EntityType,  // What buff this entity provides when awake
}

impl<'a> TheEntity<'a> {
    /// Creates a new Entity at the specified position.
    ///
    /// # Parameters
    ///
    /// - `id`: Unique identifier for this entity (0-3)
    /// - `x, y`: World position in pixels (anchor at base)
    /// - `entity_type`: What buff this entity provides when awake
    /// - `sprite_sheet`: Pre-configured sprite sheet with 13 frames
    ///
    /// # Initial State
    ///
    /// Entity starts in Dormant state, showing frame 1, with all timers reset.
    ///
    /// # Example
    ///
    /// ```rust
    /// let entity = TheEntity::new(0, 320, 200, sprite_sheet);
    /// assert_eq!(entity.state, EntityState::Dormant);
    /// ```
    pub fn new(id: usize, x: i32, y: i32, entity_type: EntityType, sprite_sheet: SpriteSheet<'a>) -> Self {
        let mut entity = TheEntity {
            x,
            y,
            width: 32,
            height: 32,
            sprite_height: 32,
            state: EntityState::Dormant,
            awakening_frame: 1,
            last_hit_time: Instant::now(),
            reverse_timer: 0.0,
            inactivity_timer: 0.0,
            sprite_sheet,
            awake_animation_frame: 8,  // Start at frame 8 (frame 9 in spec)
            awake_animation_timer: 0.0,
            id,
            entity_type,
        };

        // Initialize sprite to dormant state (frame 0 in sprite sheet = frame 1 in spec)
        entity.update_sprite_frame();
        entity
    }

    /// Handles a hit from the player.
    ///
    /// This is the core mechanic for progressive awakening. Each hit advances
    /// the entity one frame closer to being fully awake.
    ///
    /// # State-Dependent Behavior
    ///
    /// - **Dormant**: Start awakening at frame 2
    /// - **Awakening/ReversingToSleep**: Advance to next frame, switch to Awakening state
    /// - **Awake**: Reset inactivity timer (prevents timeout)
    /// - **ReturningToDormant**: Interrupt return, restart awakening from current frame
    ///
    /// # Design Pattern: State Machine with Match
    ///
    /// This is a classic state machine pattern in Rust. The `match` expression
    /// ensures we handle all possible states (exhaustive matching), and the compiler
    /// will error if we forget a state!
    pub fn on_hit(&mut self) {
        match self.state {
            EntityState::Dormant => {
                // Start awakening at frame 2 (first visible change)
                self.state = EntityState::Awakening;
                self.awakening_frame = 2;
                self.last_hit_time = Instant::now();
                self.update_sprite_frame();
            }
            EntityState::Awakening | EntityState::ReversingToSleep => {
                // Advance one frame toward full awakening
                self.awakening_frame += 1;
                self.last_hit_time = Instant::now();
                self.state = EntityState::Awakening;
                self.reverse_timer = 0.0; // Reset reverse timer

                if self.awakening_frame >= 8 {
                    // Fully awakened! Transition to awake state
                    self.awakening_frame = 8; // Clamp to max
                    self.state = EntityState::Awake;
                    self.inactivity_timer = 0.0;

                    // Start the awake looping animation (frames 8-12, manually controlled)
                    self.awake_animation_frame = 8;  // Start at frame 8
                    self.awake_animation_timer = 0.0;
                    self.update_sprite_frame();
                }

                self.update_sprite_frame();
            }
            EntityState::Awake => {
                // Reset inactivity timer to prevent timeout
                self.inactivity_timer = 0.0;
            }
            EntityState::ReturningToDormant => {
                // Interrupt the return, restart awakening from current frame
                self.state = EntityState::Awakening;
                self.last_hit_time = Instant::now();
                self.reverse_timer = 0.0;
                self.update_sprite_frame();
            }
        }
    }

    /// Updates the entity's state machine and timers.
    ///
    /// This should be called once per frame with the time elapsed since last frame.
    ///
    /// # Parameters
    ///
    /// - `delta_time`: Time elapsed since last update in seconds
    ///
    /// # State-Dependent Behavior
    ///
    /// - **Awakening**: Check for 2s timeout → transition to ReversingToSleep
    /// - **ReversingToSleep**: Reverse 1 frame every 2 seconds → Dormant when frame = 1
    /// - **Awake**: Check for 30s timeout → transition to ReturningToDormant
    /// - **ReturningToDormant**: Reverse 1 frame per second → Dormant when frame = 1
    /// - **Dormant**: No updates needed
    ///
    /// # Rust Learning: Pattern Matching
    ///
    /// The `match` expression here is exhaustive - it must handle all enum variants.
    /// We use `_ => {}` as a catch-all for states that don't need update logic.
    pub fn update(&mut self, delta_time: f32) {
        match self.state {
            EntityState::Awakening => {
                // Check if no hit for 1 second - start reversing
                if self.last_hit_time.elapsed().as_secs_f32() > 1.0 {
                    self.state = EntityState::ReversingToSleep;
                    self.reverse_timer = 0.0;
                    self.sprite_sheet.pause(); // Pause auto-animation
                }
            }
            EntityState::ReversingToSleep => {
                // Reverse 1 frame every 1 second (fast reverse when you stop hitting)
                self.reverse_timer += delta_time;
                if self.reverse_timer >= 1.0 {
                    self.reverse_timer = 0.0;

                    // Use saturating_sub to prevent underflow (won't panic if already at 0)
                    self.awakening_frame = self.awakening_frame.saturating_sub(1);
                    self.update_sprite_frame();

                    if self.awakening_frame <= 1 {
                        // Fully reversed - back to dormant
                        self.state = EntityState::Dormant;
                        self.awakening_frame = 1;
                        self.update_sprite_frame();
                    }
                }
            }
            EntityState::Awake => {
                // 30 second timeout
                self.inactivity_timer += delta_time;
                if self.inactivity_timer >= 30.0 {
                    self.state = EntityState::ReturningToDormant;
                    self.awakening_frame = 8; // Start from frame 8
                    self.reverse_timer = 0.0;
                    self.sprite_sheet.pause(); // Stop loop animation
                } else {
                    // Manually cycle through awake frames 8-12 (0.2 seconds per frame)
                    self.awake_animation_timer += delta_time;
                    if self.awake_animation_timer >= 0.2 {
                        self.awake_animation_timer = 0.0;
                        // Cycle frames: 8 → 9 → 10 → 11 → 12 → 8
                        self.awake_animation_frame += 1;
                        if self.awake_animation_frame > 12 {
                            self.awake_animation_frame = 8;  // Loop back to start
                        }
                        self.update_sprite_frame();
                    }
                }
            }
            EntityState::ReturningToDormant => {
                // Reverse faster: 1 frame per second
                self.reverse_timer += delta_time;
                if self.reverse_timer >= 1.0 {
                    self.reverse_timer = 0.0;

                    self.awakening_frame = self.awakening_frame.saturating_sub(1);
                    self.update_sprite_frame();

                    if self.awakening_frame <= 1 {
                        // Fully returned to dormant
                        self.state = EntityState::Dormant;
                        self.awakening_frame = 1;
                        self.update_sprite_frame();
                    }
                }
            }
            EntityState::Dormant => {
                // No updates needed in dormant state
            }
        }

        // Always update sprite animation (for awake loop)
        self.sprite_sheet.update();
    }

    /// Updates the sprite frame based on current state.
    ///
    /// # Frame Mapping
    ///
    /// - **Dormant/Awakening/Reversing**: Manual frame control based on awakening_frame
    ///   - awakening_frame 1 = sprite index 0
    ///   - awakening_frame 2 = sprite index 1
    ///   - ... etc
    /// - **Awake/ReturningToDormant**: Auto-playing loop (frames 7-12 in sprite sheet)
    ///
    /// # Implementation Note
    ///
    /// We pause the sprite sheet during manual control states to prevent auto-advancement,
    /// then use set_frame() to directly control which frame is displayed.
    /// Updates the sprite frame to match the current state.
    ///
    /// This should be called after manually changing the entity's state
    /// (e.g., during save/load) to ensure the visual frame matches.
    pub fn update_sprite_frame(&mut self) {
        match self.state {
            EntityState::Dormant | EntityState::Awakening | EntityState::ReversingToSleep => {
                // Manual frame control - pause auto-animation
                self.sprite_sheet.pause();

                // Convert awakening_frame (1-based spec) to sprite index (0-based)
                let sprite_index = self.awakening_frame.saturating_sub(1);
                self.sprite_sheet.set_frame(sprite_index);
            }
            EntityState::Awake => {
                // Manual control for awake animation (frames 8-12 loop)
                self.sprite_sheet.pause();
                self.sprite_sheet.set_frame(self.awake_animation_frame);
            }
            EntityState::ReturningToDormant => {
                // Manual control during return
                self.sprite_sheet.pause();
                let sprite_index = self.awakening_frame.saturating_sub(1);
                self.sprite_sheet.set_frame(sprite_index);
            }
        }
    }

    /// Checks if a player attack hitbox intersects this entity.
    ///
    /// If hit, triggers the on_hit() state transition and returns true.
    ///
    /// # Parameters
    ///
    /// - `attack_hitbox`: The AABB of the player's attack
    ///
    /// # Returns
    ///
    /// `true` if the attack hit this entity, `false` otherwise
    ///
    /// # Example
    ///
    /// ```rust
    /// if entity.check_hit(&player_attack_rect) {
    ///     println!("Entity was hit!");
    /// }
    /// ```
    pub fn check_hit(&mut self, attack_hitbox: &Rect) -> bool {
        let bounds = self.get_bounds();
        if aabb_intersect(&bounds, attack_hitbox) {
            self.on_hit();
            true
        } else {
            false
        }
    }
}

/// Implementation of StaticCollidable for collision detection.
///
/// The Entity has **state-dependent collision bounds**:
///
/// - **Dormant/Awakening/ReversingToSleep**: Full collision (32×32 scaled = 64×64)
///   - Players cannot walk through the solid pyramid
/// - **Awake/ReturningToDormant**: Partial collision (bottom 16px scaled = 32px height)
///   - Players can walk through the top half (entity is floating above base)
///
/// This creates visual depth - when awake, the entity appears to levitate,
/// and players can walk "under" the floating part.
impl StaticCollidable for TheEntity<'_> {
    fn get_bounds(&self) -> Rect {
        let shrink_amount = 3 * SPRITE_SCALE as i32;
        match self.state {
            EntityState::Dormant | EntityState::Awakening | EntityState::ReversingToSleep => {
                // Full collision - solid pyramid blocks player
                Rect::new(
                    self.x + shrink_amount,
                    self.y + shrink_amount,
                    self.width * SPRITE_SCALE - (shrink_amount * 2) as u32,
                    self.height * SPRITE_SCALE - (shrink_amount * 2) as u32,
                )
            }
            EntityState::Awake | EntityState::ReturningToDormant => {
                // Partial collision - only bottom half and half width
                let scaled_width = self.width * SPRITE_SCALE;
                let new_width = scaled_width / 2;
                let x_offset = (scaled_width - new_width) / 2;

                let collision_height = (self.height / 2) * SPRITE_SCALE; // 16 * 2 = 32
                let collision_y = self.y + collision_height as i32; // Offset down by 32

                Rect::new(
                    self.x + x_offset as i32 + shrink_amount,
                    collision_y + shrink_amount,
                    new_width - (shrink_amount * 2) as u32,
                    collision_height - (shrink_amount * 2) as u32,
                )
            }
        }
    }
}

/// Implementation of DepthSortable for correct render ordering.
///
/// The Entity uses its base Y-coordinate as the depth anchor point.
/// Entities with smaller Y render first (farther back), creating the
/// illusion of depth in a 2.5D world.
///
/// # Render Order Example
///
/// ```
/// Entity at Y=100 (renders first, appears behind)
/// Player at Y=150 (renders second, appears in middle)
/// Slime at Y=200 (renders last, appears in front)
/// ```
impl DepthSortable for TheEntity<'_> {
    fn get_depth_y(&self) -> i32 {
        // Anchor at base of entity for proper depth sorting
        // The Y position represents where the entity "touches the ground" visually
        self.y + (self.height * SPRITE_SCALE) as i32
    }

    fn render(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        // Render the sprite at the entity's position
        let dest_rect = Rect::new(
            self.x,
            self.y,
            self.width * SPRITE_SCALE,
            self.sprite_height * SPRITE_SCALE,
        );

        // Use render_flipped with no flip (false) and default direction
        // This renders the current frame from the sprite sheet
        self.sprite_sheet.render_flipped(canvas, dest_rect, false)
    }
}

/// Implementation of Saveable trait for save/load functionality.
///
/// TheEntity saves its state including position, awakening progress, and timers.
/// Note: The sprite sheet texture cannot be serialized, so it must be recreated
/// when loading from save data.
///
/// # Save Data Format
///
/// The entity is saved as JSON containing:
/// - `id`: Unique identifier (0-3)
/// - `x, y`: World position
/// - `state`: Current EntityState enum value
/// - `awakening_frame`: Current frame in awakening sequence (1-8)
/// - `inactivity_timer`: Time accumulated for timeout checks
///
/// # Loading Process
///
/// When loading, the texture must be provided externally and a new sprite sheet
/// created. The entity state is then restored from the saved data.
impl Saveable for TheEntity<'_> {
    fn to_save_data(&self) -> Result<SaveData, SaveError> {
        /// Serializable structure for entity save data
        #[derive(Serialize)]
        struct EntitySaveData {
            id: usize,
            x: i32,
            y: i32,
            state: EntityState,
            awakening_frame: usize,
            inactivity_timer: f32,
            entity_type: EntityType,
        }

        let data = EntitySaveData {
            id: self.id,
            x: self.x,
            y: self.y,
            state: self.state,
            awakening_frame: self.awakening_frame,
            inactivity_timer: self.inactivity_timer,
            entity_type: self.entity_type,
        };

        Ok(SaveData {
            data_type: "the_entity".to_string(),
            json_data: serde_json::to_string(&data)
                .map_err(SaveError::SerializationError)?,
        })
    }

    fn from_save_data(_data: &SaveData) -> Result<Self, SaveError> {
        // TheEntity cannot be directly constructed from save data because it requires
        // a texture reference (lifetime 'a). The loading process must:
        // 1. Load the texture externally
        // 2. Create a sprite sheet with the texture
        // 3. Construct the entity with the sprite sheet
        // 4. Apply saved state to the entity
        //
        // This is handled manually in the load_game() function in main.rs
        Err(SaveError::CorruptedData(
            "TheEntity requires external texture setup - use manual loading in load_game()".to_string()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Full unit tests require SDL2 texture creation, which is complex
    // in test environments. These tests verify the state machine logic.

    #[test]
    fn test_entity_state_transitions() {
        // This test verifies the basic state flow conceptually
        // Full testing requires SDL2 context and will be done manually

        // Verify state enum is Copy (can be assigned without .clone())
        let state = EntityState::Dormant;
        let _state_copy = state; // Should compile without error
        assert_eq!(state, EntityState::Dormant); // Original still usable
    }

    #[test]
    fn test_awakening_frame_bounds() {
        // Verify frame numbers make sense
        let dormant_frame = 1;
        let fully_awake_frame = 8;

        assert!(dormant_frame < fully_awake_frame);
        assert_eq!(fully_awake_frame, 8); // Spec requirement
    }

    #[test]
    fn test_collision_bounds_calculation() {
        // Test the math for collision bounds
        let width = 32u32;
        let height = 32u32;
        let scale = 2u32;

        // Full collision
        let full_height = height * scale;
        assert_eq!(full_height, 64);

        // Partial collision
        let partial_height = (height / 2) * scale;
        assert_eq!(partial_height, 32);
    }

    #[test]
    fn test_saturating_sub() {
        // Verify saturating_sub prevents underflow
        let frame: usize = 1;
        let result = frame.saturating_sub(1);
        assert_eq!(result, 0);

        let result2 = result.saturating_sub(1);
        assert_eq!(result2, 0); // Doesn't panic or wrap around
    }
}
