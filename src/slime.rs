use crate::animation::AnimationController;
use crate::collision::{Collidable, CollisionLayer};
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::time::Instant;

#[derive(Debug, Clone, PartialEq)]
enum SlimeBehavior {
    Idle,
    Jumping,
}

pub struct Slime<'a> {
    pub x: i32,
    pub y: i32,
    pub base_y: i32, // Original Y position for jumping reference
    pub width: u32,
    pub height: u32,
    animation_controller: AnimationController<'a>,
    behavior: SlimeBehavior,
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
            behavior_timer: Instant::now(),
            jump_height: 20, // How high the slime bounces
            jump_duration: 0.5, // Jump lasts 0.5 seconds total (2x faster)
            health: 3, // Slimes take 3 hits to kill
            is_alive: true,

            // Default hitbox for slime (smaller, rounder character)
            // Tuned values: width=16, height=12 to match actual sprite artwork
            hitbox_offset_x: 9,  // 9 pixels from left (centered with 1px adjustment)
            hitbox_offset_y: 10, // 10 pixels from top (slime sits lower in frame)
            hitbox_width: 16,    // 16 pixels wide
            hitbox_height: 12,   // 12 pixels tall
        }
    }

    pub fn update(&mut self) {
        let elapsed_time = self.behavior_timer.elapsed().as_secs_f32();

        // Game Dev Pattern: Simple AI State Machine
        // The slime alternates between idle and jumping based on timers
        match self.behavior {
            SlimeBehavior::Idle => {
                // Idle for 2 seconds, then switch to jumping
                if elapsed_time >= 2.0 {
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
        }

        // Always update animation controller
        self.animation_controller.update();
    }

    pub fn render(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        let scale = 3; // 3x zoom scale
        let scaled_width = self.width * scale;
        let scaled_height = self.height * scale;
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
    pub fn take_damage(&mut self, damage: i32) -> bool {
        self.health -= damage;

        if self.health <= 0 {
            self.is_alive = false;
            println!("Slime defeated!");
            return true;
        }

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

// Collision System Implementation
//
// This trait implementation makes Slime participate in the collision system.
// Important: Collision bounds use the slime's current Y position (which changes during jumps)
// rather than base_y, so collision detection works correctly mid-jump.
impl<'a> Collidable for Slime<'a> {
    fn get_bounds(&self) -> Rect {
        // Use configurable hitbox instead of full sprite size
        let scale = 3;
        let offset_x = self.hitbox_offset_x * scale as i32;
        let offset_y = self.hitbox_offset_y * scale as i32;
        let scaled_width = self.hitbox_width * scale;
        let scaled_height = self.hitbox_height * scale;

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