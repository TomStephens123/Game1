use crate::animation::AnimationController;
use crate::collision::{Collidable, CollisionLayer};
use crate::render::DepthSortable;
use crate::save::{Saveable, SaveData, SaveError};
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use serde::{Serialize, Deserialize};
use std::time::Instant;

#[derive(Debug, Clone, PartialEq)]
enum SlimeBehavior {
    Idle,
    Jumping,
    TakingDamage,  // Playing damage animation
    Dying,         // Playing death animation
}

pub struct Slime<'a> {
    pub x: i32,
    pub y: i32,
    pub base_y: i32, // Original Y position for jumping reference
    pub width: u32,
    pub height: u32,
    animation_controller: AnimationController<'a>,
    behavior: SlimeBehavior,
    previous_behavior: SlimeBehavior,  // Track behavior before damage/death
    behavior_timer: Instant,
    jump_height: i32,
    jump_duration: f32, // Duration of jump animation in seconds

    // Health system
    pub health: i32,
    pub is_alive: bool,

    // Collision hitbox configuration
    pub hitbox_offset_x: i32,
    pub hitbox_offset_y: i32,
    pub hitbox_width: u32,
    pub hitbox_height: u32,
}

impl<'a> Slime<'a> {
    pub fn new(x: i32, y: i32, animation_controller: AnimationController<'a>) -> Self {
        Slime {
            x,
            y,
            base_y: y,
            width: 32,
            height: 32,
            animation_controller,
            behavior: SlimeBehavior::Idle,
            previous_behavior: SlimeBehavior::Idle,  // Start as idle
            behavior_timer: Instant::now(),
            jump_height: 20, // How high the slime bounces
            jump_duration: 0.5, // Jump lasts 0.5 seconds total (2x faster)
            health: 8, // Slimes have 8 HP (takes 3 hits of 3 damage to kill)
            is_alive: true,

            // Default hitbox for slime (smaller, rounder character)
            // Tuned values: width=16, height=12 to match actual sprite artwork
            hitbox_offset_x: 9,  // 9 pixels from left (centered with 1px adjustment)
            hitbox_offset_y: 10, // 10 pixels from top (slime sits lower in frame)
            hitbox_width: 16,    // 16 pixels wide
            hitbox_height: 12,   // 12 pixels tall
        }
    }

    pub fn set_animation_controller(&mut self, controller: AnimationController<'a>) {
        self.animation_controller = controller;
    }

    /// Returns true if the slime is currently invulnerable
    ///
    /// Slimes are invulnerable while playing their damage or death animations.
    /// This prevents stunlock and ensures visual feedback completes.
    pub fn is_invulnerable(&self) -> bool {
        matches!(self.behavior, SlimeBehavior::TakingDamage | SlimeBehavior::Dying)
    }

    pub fn update(&mut self) {
        // IMPORTANT: Update animation controller FIRST
        // This ensures animations are reset before we check is_animation_finished()
        // Otherwise, checking a "once" animation that was previously finished will
        // return true even though we just set it to play again
        self.animation_controller.update();

        let elapsed_time = self.behavior_timer.elapsed().as_secs_f32();

        // Game Dev Pattern: Simple AI State Machine
        // The slime alternates between idle and jumping based on timers
        match self.behavior {
            SlimeBehavior::Idle => {
                // Idle for 2 seconds, then switch to jumping
                if elapsed_time >= 2.0 {
                    self.previous_behavior = self.behavior.clone();
                    self.behavior = SlimeBehavior::Jumping;
                    self.behavior_timer = Instant::now();
                    self.animation_controller.set_state("jump".to_string());
                } else {
                    // Make sure we're in idle animation
                    if self.animation_controller.current_state() != "slime_idle" {
                        self.animation_controller.set_state("slime_idle".to_string());
                    }
                }
                // Stay at base position when idle
                self.y = self.base_y;
            }
            SlimeBehavior::Jumping => {
                // Jump for jump_duration, then return to idle
                if elapsed_time >= self.jump_duration {
                    self.previous_behavior = self.behavior.clone();
                    self.behavior = SlimeBehavior::Idle;
                    self.behavior_timer = Instant::now();
                    self.animation_controller.set_state("slime_idle".to_string());
                    self.y = self.base_y; // Return to ground
                } else {
                    // Calculate jump position using sine wave
                    // Game Dev Math: sin() gives smooth bounce motion (0 -> 1 -> 0)
                    let jump_progress = (elapsed_time * std::f32::consts::PI / self.jump_duration).sin();
                    let jump_offset = (jump_progress * self.jump_height as f32) as i32;
                    self.y = self.base_y - jump_offset;
                }
            }
            SlimeBehavior::TakingDamage => {
                // Play damage animation, then return to previous behavior
                if self.animation_controller.is_animation_finished() {
                    // Return to whatever we were doing before (idle or jumping)
                    self.behavior = self.previous_behavior.clone();
                    self.behavior_timer = Instant::now();

                    // Set appropriate animation based on previous behavior
                    match self.previous_behavior {
                        SlimeBehavior::Idle => self.animation_controller.set_state("slime_idle".to_string()),
                        SlimeBehavior::Jumping => self.animation_controller.set_state("jump".to_string()),
                        _ => self.animation_controller.set_state("slime_idle".to_string()),
                    }
                }
            }
            SlimeBehavior::Dying => {
                // Play death animation, then mark as dead when finished
                if self.animation_controller.is_animation_finished() {
                    self.is_alive = false;
                }
            }
        }

        // Animation controller already updated at the beginning of this function
    }

    pub fn render(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        const SPRITE_SCALE: u32 = 2;
        let scaled_width = self.width * SPRITE_SCALE;
        let scaled_height = self.height * SPRITE_SCALE;
        let dest_rect = Rect::new(self.x, self.y, scaled_width, scaled_height);

        if let Some(sprite_sheet) = self.animation_controller.get_current_sprite_sheet() {
            sprite_sheet.render_flipped(canvas, dest_rect, false)
        } else {
            // Fallback red square if no sprite sheet
            canvas.set_draw_color(sdl2::pixels::Color::RGB(255, 0, 0));
            canvas.fill_rect(dest_rect).map_err(|e| e.to_string())
        }
    }

    /// Applies a push force to the slime (used for collision response).
    ///
    /// This is called when the slime collides with something and needs to be
    /// pushed away to prevent overlap.
    ///
    /// Note: We update both x and base_y so the slime stays pushed even after jumping
    pub fn apply_push(&mut self, push_x: i32, push_y: i32) {
        self.x += push_x;
        self.y += push_y;
        self.base_y += push_y; // Keep base_y in sync so jump behavior works correctly
    }

    /// Deals damage to the slime.
    ///
    /// Returns true if the slime died from this damage.
    ///
    /// Slimes are invulnerable while playing damage or death animations,
    /// preventing stunlock and ensuring visual feedback completes.
    pub fn take_damage(&mut self, damage: i32) -> bool {
        // Check invulnerability (state-based: invulnerable during damage/death animations)
        if self.is_invulnerable() {
            return false;
        }

        self.health -= damage;

        if self.health <= 0 {
            // Start death animation (don't set is_alive = false until animation finishes)
            // Slime becomes invulnerable while dying
            self.previous_behavior = self.behavior.clone();
            self.behavior = SlimeBehavior::Dying;
            self.behavior_timer = Instant::now();
            self.animation_controller.set_state("slime_death".to_string());
            return true;
        }

        // Take damage but still alive - play damage animation
        // Slime becomes invulnerable while taking damage (animation lasts 300ms)
        self.previous_behavior = self.behavior.clone();
        self.behavior = SlimeBehavior::TakingDamage;
        self.behavior_timer = Instant::now();
        self.animation_controller.set_state("slime_damage".to_string());
        false
    }

    /// Sets custom hitbox parameters for fine-tuning collision detection.
    ///
    /// All values are in unscaled sprite pixels (will be multiplied by scale factor).
    #[allow(dead_code)]
    pub fn set_hitbox(&mut self, offset_x: i32, offset_y: i32, width: u32, height: u32) {
        self.hitbox_offset_x = offset_x;
        self.hitbox_offset_y = offset_y;
        self.hitbox_width = width;
        self.hitbox_height = height;
    }
}

// ==============================================================================
// Depth Sorting Render System
// ==============================================================================

/// Implementation of depth sorting for Slime.
///
/// The slime's depth is determined by its base Y-coordinate (where it touches ground).
/// We use base_y rather than y to ensure consistent depth even when jumping.
///
/// See docs/systems/depth-sorting-render-system.md for design documentation.
impl DepthSortable for Slime<'_> {
    fn get_depth_y(&self) -> i32 {
        // Slime's anchor point is at the base (bottom of sprite when on ground)
        // Use base_y to ensure consistent depth during jump animation
        // The visual Y position (self.y) changes during jumps, but the slime's
        // depth in the scene should remain constant
        const SPRITE_SCALE: u32 = 2;
        self.base_y + (self.height * SPRITE_SCALE) as i32
    }

    fn render(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        // Delegate to existing render implementation
        // This avoids code duplication and keeps the existing render logic intact
        Slime::render(self, canvas)
    }
}

// ==============================================================================
// Collision System Implementation
// ==============================================================================

// This trait implementation makes Slime participate in the collision system.
// Important: Collision bounds use the slime's current Y position (which changes during jumps)
// rather than base_y, so collision detection works correctly mid-jump.
impl<'a> Collidable for Slime<'a> {
    fn get_bounds(&self) -> Rect {
        // Use configurable hitbox instead of full sprite size
        const SPRITE_SCALE: u32 = 2;
        let offset_x = self.hitbox_offset_x * SPRITE_SCALE as i32;
        let offset_y = self.hitbox_offset_y * SPRITE_SCALE as i32;
        let scaled_width = self.hitbox_width * SPRITE_SCALE;
        let scaled_height = self.hitbox_height * SPRITE_SCALE;

        // Use current Y position (self.y), not base_y
        // This ensures collision detection works when slime is jumping
        Rect::new(
            self.x + offset_x,
            self.y + offset_y,
            scaled_width,
            scaled_height,
        )
    }

    fn get_collision_layer(&self) -> CollisionLayer {
        CollisionLayer::Enemy
    }
}
// ==============================================================================
// Save/Load Implementation
// ==============================================================================

impl Saveable for Slime<'_> {
    fn to_save_data(&self) -> Result<SaveData, SaveError> {
        #[derive(Serialize)]
        struct SlimeData {
            x: i32,
            y: i32,
            base_y: i32,
            health: i32,
            is_alive: bool,
            hitbox_offset_x: i32,
            hitbox_offset_y: i32,
            hitbox_width: u32,
            hitbox_height: u32,
        }

        let slime_data = SlimeData {
            x: self.x,
            y: self.y,
            base_y: self.base_y,
            health: self.health,
            is_alive: self.is_alive,
            hitbox_offset_x: self.hitbox_offset_x,
            hitbox_offset_y: self.hitbox_offset_y,
            hitbox_width: self.hitbox_width,
            hitbox_height: self.hitbox_height,
        };

        Ok(SaveData {
            data_type: "slime".to_string(),
            json_data: serde_json::to_string(&slime_data)?,
        })
    }

    fn from_save_data(data: &SaveData) -> Result<Self, SaveError> {
        #[derive(Deserialize)]
        struct SlimeData {
            x: i32,
            y: i32,
            base_y: i32,
            health: i32,
            is_alive: bool,
            hitbox_offset_x: i32,
            hitbox_offset_y: i32,
            hitbox_width: u32,
            hitbox_height: u32,
        }

        if data.data_type != "slime" {
            return Err(SaveError::CorruptedData(format!(
                "Expected slime data, got {}",
                data.data_type
            )));
        }

        let slime_data: SlimeData = serde_json::from_str(&data.json_data)?;

        // Create slime with animation controller placeholder
        // The actual animation controller will be set externally
        let mut slime = Slime::new(
            slime_data.x,
            slime_data.y,
            AnimationController::new(),
        );

        // Restore state
        slime.base_y = slime_data.base_y;
        slime.health = slime_data.health;
        slime.is_alive = slime_data.is_alive;
        slime.hitbox_offset_x = slime_data.hitbox_offset_x;
        slime.hitbox_offset_y = slime_data.hitbox_offset_y;
        slime.hitbox_width = slime_data.hitbox_width;
        slime.hitbox_height = slime_data.hitbox_height;

        // Note: Behavior state and timers are NOT saved
        // Slimes will start in Idle state with reset timers
        // Invulnerability is derived from behavior state (not saved separately)

        Ok(slime)
    }
}
