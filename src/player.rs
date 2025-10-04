use crate::animation::{AnimationController, AnimationState, Direction, determine_animation_state};
use crate::collision::{Collidable, CollisionLayer};
use sdl2::keyboard::Scancode;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::time::Instant;

pub struct Player<'a> {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub speed: i32,
    pub velocity_x: i32,
    pub velocity_y: i32,
    pub direction: Direction,
    pub is_attacking: bool,
    animation_controller: AnimationController<'a>,

    // Health and damage system
    pub health: i32,
    pub max_health: i32,
    pub is_invulnerable: bool,
    invulnerability_timer: Instant,
    invulnerability_duration: f32, // seconds

    // Collision hitbox configuration
    // Allows tuning the collision box independently from sprite rendering
    pub hitbox_offset_x: i32,
    pub hitbox_offset_y: i32,
    pub hitbox_width: u32,
    pub hitbox_height: u32,
}

impl<'a> Player<'a> {
    pub fn new(x: i32, y: i32, width: u32, height: u32, speed: i32) -> Self {
        Player {
            x,
            y,
            width,
            height,
            speed,
            velocity_x: 0,
            velocity_y: 0,
            direction: Direction::South,
            is_attacking: false,
            animation_controller: AnimationController::new(),
            health: 10,
            max_health: 10,
            is_invulnerable: false,
            invulnerability_timer: Instant::now(),
            invulnerability_duration: 1.0, // 1 second of invulnerability after taking damage

            // Default hitbox tuned to match actual sprite artwork
            hitbox_offset_x: 8,  // 8 pixels from left edge (centered)
            hitbox_offset_y: 8,  // 8 pixels from top edge
            hitbox_width: 16,    // 16x16 square hitbox
            hitbox_height: 16,
        }
    }

    pub fn set_animation_controller(&mut self, controller: AnimationController<'a>) {
        self.animation_controller = controller;
    }

    pub fn update(&mut self, keyboard_state: &sdl2::keyboard::KeyboardState) {
        self.velocity_x = 0;
        self.velocity_y = 0;

        // Always allow vertical movement
        if keyboard_state.is_scancode_pressed(Scancode::W) {
            self.velocity_y -= self.speed;
        }
        if keyboard_state.is_scancode_pressed(Scancode::S) {
            self.velocity_y += self.speed;
        }

        // Only allow horizontal movement if not attacking
        if !self.is_attacking {
            if keyboard_state.is_scancode_pressed(Scancode::A) {
                self.velocity_x -= self.speed;
            }
            if keyboard_state.is_scancode_pressed(Scancode::D) {
                self.velocity_x += self.speed;
            }
        }

        // Normalize diagonal movement to maintain consistent speed
        if self.velocity_x != 0 && self.velocity_y != 0 {
            // For diagonal movement, scale by 1/√2 ≈ 0.707 to maintain same net speed
            let diagonal_factor = 0.707; // 1.0 / sqrt(2.0)
            self.velocity_x = (self.velocity_x as f32 * diagonal_factor).round() as i32;
            self.velocity_y = (self.velocity_y as f32 * diagonal_factor).round() as i32;
        }

        self.x += self.velocity_x;
        self.y += self.velocity_y;

        // Update direction based on movement (only when moving)
        if self.velocity_x != 0 || self.velocity_y != 0 {
            self.direction = Direction::from_velocity(self.velocity_x, self.velocity_y);
        }

        // Check if attack animation is finished
        if self.is_attacking && self.animation_controller.is_animation_finished() {
            self.is_attacking = false;
        }

        // Update invulnerability timer
        if self.is_invulnerable {
            let elapsed = self.invulnerability_timer.elapsed().as_secs_f32();
            if elapsed >= self.invulnerability_duration {
                self.is_invulnerable = false;
            }
        }

        // Determine animation state based on current actions
        // Game Dev Pattern: Priority-based state selection
        // Attack takes priority over movement states
        let new_state = if self.is_attacking {
            "attack".to_string()
        } else {
            // Only consider horizontal movement for running animation
            // Vertical movement during attacks shouldn't trigger running animation
            determine_animation_state(self.velocity_x, self.velocity_y, self.speed)
        };

        self.animation_controller.set_state(new_state);
        self.animation_controller.update();
    }

    pub fn keep_in_bounds(&mut self, window_width: u32, window_height: u32) {
        let scale = 3; // Match rendering scale
        let scaled_width = self.width * scale;
        let scaled_height = self.height * scale;

        if self.x < 0 {
            self.x = 0;
        }
        if self.y < 0 {
            self.y = 0;
        }
        if self.x > (window_width as i32) - (scaled_width as i32) {
            self.x = (window_width as i32) - (scaled_width as i32);
        }
        if self.y > (window_height as i32) - (scaled_height as i32) {
            self.y = (window_height as i32) - (scaled_height as i32);
        }
    }

    pub fn render(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        let scale = 3; // 3x zoom scale
        let scaled_width = self.width * scale;
        let scaled_height = self.height * scale;
        let dest_rect = Rect::new(self.x, self.y, scaled_width, scaled_height);

        if let Some(sprite_sheet) = self.animation_controller.get_current_sprite_sheet() {
            sprite_sheet.render_directional(canvas, dest_rect, false, self.direction)
        } else {
            canvas.set_draw_color(sdl2::pixels::Color::RGB(255, 0, 0));
            canvas.fill_rect(dest_rect).map_err(|e| e.to_string())
        }
    }

    pub fn current_animation_state(&self) -> &AnimationState {
        self.animation_controller.current_state()
    }


    pub fn position(&self) -> (i32, i32) {
        (self.x, self.y)
    }

    pub fn velocity(&self) -> (i32, i32) {
        (self.velocity_x, self.velocity_y)
    }

    pub fn start_attack(&mut self) {
        if !self.is_attacking {
            self.is_attacking = true;
        }
    }

    /// Applies a push force to the player (used for collision response).
    ///
    /// This is called when the player collides with something and needs to be
    /// pushed away to prevent overlap.
    pub fn apply_push(&mut self, push_x: i32, push_y: i32) {
        self.x += push_x;
        self.y += push_y;
    }

    /// Deals damage to the player if not invulnerable.
    ///
    /// Returns true if damage was taken, false if player was invulnerable.
    pub fn take_damage(&mut self, damage: i32) -> bool {
        if self.is_invulnerable {
            return false;
        }

        self.health -= damage;
        self.is_invulnerable = true;
        self.invulnerability_timer = Instant::now();

        println!("Player took {} damage! Health: {}/{}", damage, self.health, self.max_health);

        if self.health <= 0 {
            println!("Player died!");
        }

        true
    }

    /// Returns true if the player is alive.
    ///
    /// Note: Currently unused but included for future game over detection.
    #[allow(dead_code)]
    pub fn is_alive(&self) -> bool {
        self.health > 0
    }

    /// Sets custom hitbox parameters for fine-tuning collision detection.
    ///
    /// Use this to adjust the collision box to match the actual sprite artwork.
    /// All values are in unscaled sprite pixels (will be multiplied by scale factor).
    ///
    /// # Parameters
    /// - `offset_x`: Horizontal offset from sprite position
    /// - `offset_y`: Vertical offset from sprite position
    /// - `width`: Width of the hitbox (before scaling)
    /// - `height`: Height of the hitbox (before scaling)
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
// This trait implementation makes Player participate in the collision system.
// The collision bounds match the player's rendered size (accounting for 3x scale).
impl<'a> Collidable for Player<'a> {
    fn get_bounds(&self) -> Rect {
        // Use configurable hitbox instead of full sprite size
        // This allows fine-tuning collision to match actual sprite artwork
        let scale = 3;
        let offset_x = self.hitbox_offset_x * scale as i32;
        let offset_y = self.hitbox_offset_y * scale as i32;
        let scaled_width = self.hitbox_width * scale;
        let scaled_height = self.hitbox_height * scale;

        Rect::new(
            self.x + offset_x,
            self.y + offset_y,
            scaled_width,
            scaled_height,
        )
    }

    fn get_collision_layer(&self) -> CollisionLayer {
        CollisionLayer::Player
    }
}
