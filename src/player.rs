use crate::animation::{AnimationController, AnimationState, Direction, determine_animation_state};
use crate::collision::{Collidable, CollisionLayer};
use crate::combat::{AttackEvent, DamageEvent, PlayerState, calculate_damage_with_defense};
use crate::save::{Saveable, SaveData, SaveError};
use crate::stats::{Stats, DamageResult};
use sdl2::keyboard::Scancode;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use serde::{Serialize, Deserialize};
use std::time::Instant;

pub struct Player<'a> {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub velocity_x: i32,
    pub velocity_y: i32,
    pub direction: Direction,
    pub is_attacking: bool,
    pub is_taking_damage: bool,  // Track if damage animation is playing
    animation_controller: AnimationController<'a>,

    // New comprehensive stats system
    pub stats: Stats,

    // Player state (Alive/Dead)
    pub state: PlayerState,

    // Invulnerability system (keep existing pattern)
    pub is_invulnerable: bool,
    invulnerability_timer: Instant,
    invulnerability_duration: f32, // seconds

    // Attack cooldown system
    last_attack_time: Instant,

    // Collision hitbox configuration
    // Allows tuning the collision box independently from sprite rendering
    pub hitbox_offset_x: i32,
    pub hitbox_offset_y: i32,
    pub hitbox_width: u32,
    pub hitbox_height: u32,
}

impl<'a> Player<'a> {
    pub fn new(x: i32, y: i32, width: u32, height: u32, speed: i32) -> Self {
        let mut stats = Stats::new();
        // Set movement speed from parameter (for backward compatibility)
        stats.movement_speed = speed as f32;

        Player {
            x,
            y,
            width,
            height,
            velocity_x: 0,
            velocity_y: 0,
            direction: Direction::South,
            is_attacking: false,
            is_taking_damage: false,  // Not taking damage initially
            animation_controller: AnimationController::new(),
            stats,
            state: PlayerState::Alive,
            is_invulnerable: false,
            invulnerability_timer: Instant::now(),
            invulnerability_duration: 1.0, // 1 second of invulnerability after taking damage
            last_attack_time: Instant::now(),

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

        // Get effective movement speed from stats (will support modifiers later)
        let effective_speed = self.stats.movement_speed as i32;

        // Only allow movement if not attacking or taking damage
        if !self.is_attacking && !self.is_taking_damage {
            // Vertical movement
            if keyboard_state.is_scancode_pressed(Scancode::W) {
                self.velocity_y -= effective_speed;
            }
            if keyboard_state.is_scancode_pressed(Scancode::S) {
                self.velocity_y += effective_speed;
            }

            // Horizontal movement
            if keyboard_state.is_scancode_pressed(Scancode::A) {
                self.velocity_x -= effective_speed;
            }
            if keyboard_state.is_scancode_pressed(Scancode::D) {
                self.velocity_x += effective_speed;
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

        // Check if damage animation is finished
        if self.is_taking_damage && self.animation_controller.is_animation_finished() {
            self.is_taking_damage = false;
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
        // Death > Damage > Attack > Movement > Idle
        let new_state = if !self.state.is_alive() {
            "death".to_string()
        } else if self.is_taking_damage {
            "damage".to_string()
        } else if self.is_attacking {
            "attack".to_string()
        } else {
            // Only consider horizontal movement for running animation
            // Vertical movement during attacks shouldn't trigger running animation
            determine_animation_state(self.velocity_x, self.velocity_y, effective_speed)
        };

        self.animation_controller.set_state(new_state);
        self.animation_controller.update();
    }

    pub fn render(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        const SPRITE_SCALE: u32 = 2;
        let scaled_width = self.width * SPRITE_SCALE;
        let scaled_height = self.height * SPRITE_SCALE;
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

    /// Checks if the player can attack (not attacking, alive, and cooldown ready)
    pub fn can_attack(&self) -> bool {
        if !self.state.is_alive() || self.is_attacking {
            return false;
        }

        // Attack cooldown based on attack_speed stat
        // attack_speed is attacks per second, so cooldown = 1.0 / attack_speed
        let attack_cooldown = 1.0 / self.stats.attack_speed;
        self.last_attack_time.elapsed().as_secs_f32() >= attack_cooldown
    }

    /// Attempts to start an attack
    ///
    /// Returns Some(AttackEvent) if attack was successful, None if on cooldown
    pub fn start_attack(&mut self) -> Option<AttackEvent> {
        if !self.can_attack() {
            return None;
        }

        self.is_attacking = true;
        self.last_attack_time = Instant::now();

        // Calculate attack position from player's hitbox center
        const SPRITE_SCALE: i32 = 2;
        let hitbox_center_x = self.x + (self.hitbox_offset_x * SPRITE_SCALE) + (self.hitbox_width as i32 * SPRITE_SCALE / 2);
        let hitbox_center_y = self.y + (self.hitbox_offset_y * SPRITE_SCALE) + (self.hitbox_height as i32 * SPRITE_SCALE / 2);

        Some(AttackEvent::new(
            self.stats.attack_damage,
            (hitbox_center_x, hitbox_center_y),
            self.direction,
            32, // Attack range in pixels (balanced for close-range combat)
        ))
    }

    /// Applies a push force to the player (used for collision response).
    ///
    /// This is called when the player collides with something and needs to be
    /// pushed away to prevent overlap.
    pub fn apply_push(&mut self, push_x: i32, push_y: i32) {
        self.x += push_x;
        self.y += push_y;
    }

    /// Deals damage to the player using a DamageEvent
    ///
    /// This applies defense calculations based on damage type and returns detailed results
    pub fn take_damage(&mut self, damage_event: DamageEvent) -> DamageResult {
        if self.is_invulnerable || !self.state.is_alive() {
            return DamageResult::no_damage();
        }

        // Calculate final damage with defense
        let final_damage = calculate_damage_with_defense(&damage_event, self.stats.defense);

        let result = self.stats.health.take_damage(final_damage);

        // Activate invulnerability after taking damage
        self.is_invulnerable = true;
        self.invulnerability_timer = Instant::now();

        if result.is_fatal {
            self.die();
        } else {
            // Play damage animation (only if not dead)
            self.is_taking_damage = true;
        }

        result
    }

    /// Handles player death
    fn die(&mut self) {
        self.state = PlayerState::Dead {
            death_time: Instant::now(),
        };
    }

    /// Respawns the player at a specific position with full health
    ///
    /// This method is called after the death screen timer expires. It:
    /// - Restores full health
    /// - Resets player state to Alive
    /// - Clears combat state (attacking, taking damage)
    /// - Resets invulnerability
    /// - Moves player to the respawn position
    /// - Stops player movement
    ///
    /// # Parameters
    /// - `x`, `y`: The respawn position in world coordinates
    ///
    /// # Example
    /// ```rust
    /// // Respawn at world center after death
    /// player.respawn(GAME_WIDTH as i32 / 2, GAME_HEIGHT as i32 / 2);
    /// ```
    pub fn respawn(&mut self, x: i32, y: i32) {
        // Restore health to full
        let max_health = self.stats.max_health;
        self.stats.health.heal(max_health);

        // Reset state
        self.state = PlayerState::Alive;

        // Clear combat state
        self.is_attacking = false;
        self.is_taking_damage = false;

        // Reset invulnerability
        self.is_invulnerable = false;
        self.invulnerability_timer = Instant::now();

        // Reset position
        self.x = x;
        self.y = y;
        self.velocity_x = 0;
        self.velocity_y = 0;

        println!("Player respawned at ({}, {}) with full health", x, y);
    }

    /// Heals the player
    ///
    /// Returns the actual amount healed (may be less than requested if near max health)
    #[allow(dead_code)] // Reserved for healing items/abilities
    pub fn heal(&mut self, amount: f32) -> f32 {
        if !self.is_alive() {
            return 0.0;
        }

        self.stats.health.heal(amount)
    }

    /// Returns true if the player is alive.
    #[allow(dead_code)] // Already exposed via state.is_alive()
    pub fn is_alive(&self) -> bool {
        self.stats.health.is_alive()
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
// The collision bounds match the player's rendered size (accounting for 2x scale).
impl<'a> Collidable for Player<'a> {
    fn get_bounds(&self) -> Rect {
        // Use configurable hitbox instead of full sprite size
        // This allows fine-tuning collision to match actual sprite artwork
        const SPRITE_SCALE: u32 = 2;
        let offset_x = self.hitbox_offset_x * SPRITE_SCALE as i32;
        let offset_y = self.hitbox_offset_y * SPRITE_SCALE as i32;
        let scaled_width = self.hitbox_width * SPRITE_SCALE;
        let scaled_height = self.hitbox_height * SPRITE_SCALE;

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

// ==============================================================================
// Save/Load Implementation
// ==============================================================================

impl Saveable for Player<'_> {
    fn to_save_data(&self) -> Result<SaveData, SaveError> {
        #[derive(Serialize)]
        struct PlayerData {
            // Position and movement
            x: i32,
            y: i32,
            direction: String,

            // Stats (health, movement speed, attack damage, etc.)
            health_current: f32,
            health_max: f32,
            movement_speed: f32,
            attack_damage: f32,
            attack_speed: f32,
            defense: f32,
            max_health: f32,

            // State
            is_alive: bool,

            // Hitbox configuration
            hitbox_offset_x: i32,
            hitbox_offset_y: i32,
            hitbox_width: u32,
            hitbox_height: u32,
        }

        // Determine if player is alive
        let is_alive = self.state.is_alive();

        let player_data = PlayerData {
            x: self.x,
            y: self.y,
            direction: format!("{:?}", self.direction),
            health_current: self.stats.health.current(),
            health_max: self.stats.health.max(),
            movement_speed: self.stats.movement_speed,
            attack_damage: self.stats.attack_damage,
            attack_speed: self.stats.attack_speed,
            defense: self.stats.defense,
            max_health: self.stats.max_health,
            is_alive,
            hitbox_offset_x: self.hitbox_offset_x,
            hitbox_offset_y: self.hitbox_offset_y,
            hitbox_width: self.hitbox_width,
            hitbox_height: self.hitbox_height,
        };

        Ok(SaveData {
            data_type: "player".to_string(),
            json_data: serde_json::to_string(&player_data)?,
        })
    }

    fn from_save_data(data: &SaveData) -> Result<Self, SaveError> {
        #[derive(Deserialize)]
        struct PlayerData {
            x: i32,
            y: i32,
            direction: String,
            health_current: f32,
            health_max: f32,
            movement_speed: f32,
            attack_damage: f32,
            attack_speed: f32,
            defense: f32,
            max_health: f32,
            is_alive: bool,
            hitbox_offset_x: i32,
            hitbox_offset_y: i32,
            hitbox_width: u32,
            hitbox_height: u32,
        }

        if data.data_type != "player" {
            return Err(SaveError::CorruptedData(format!(
                "Expected player data, got {}",
                data.data_type
            )));
        }

        let player_data: PlayerData = serde_json::from_str(&data.json_data)?;

        // Create player with position and initial speed
        let mut player = Player::new(
            player_data.x,
            player_data.y,
            32, // width
            32, // height
            player_data.movement_speed as i32,
        );

        // Restore stats
        player.stats.health = crate::stats::Health::new(player_data.health_max);
        // Set current health (take damage to reduce from max to current)
        let damage_taken = player_data.health_max - player_data.health_current;
        if damage_taken > 0.0 {
            player.stats.health.take_damage(damage_taken);
        }

        player.stats.movement_speed = player_data.movement_speed;
        player.stats.attack_damage = player_data.attack_damage;
        player.stats.attack_speed = player_data.attack_speed;
        player.stats.defense = player_data.defense;
        player.stats.max_health = player_data.max_health;

        // Restore direction
        player.direction = match player_data.direction.as_str() {
            "South" => Direction::South,
            "SouthEast" => Direction::SouthEast,
            "East" => Direction::East,
            "NorthEast" => Direction::NorthEast,
            "North" => Direction::North,
            "NorthWest" => Direction::NorthWest,
            "West" => Direction::West,
            "SouthWest" => Direction::SouthWest,
            _ => Direction::South, // Default fallback
        };

        // Restore player state (Alive/Dead)
        if !player_data.is_alive {
            player.state = PlayerState::Dead {
                death_time: Instant::now(), // Reset death time to now
            };
        }

        // Restore hitbox configuration
        player.hitbox_offset_x = player_data.hitbox_offset_x;
        player.hitbox_offset_y = player_data.hitbox_offset_y;
        player.hitbox_width = player_data.hitbox_width;
        player.hitbox_height = player_data.hitbox_height;

        // Note: Animation controller, timers, and transient state are NOT saved
        // They will be initialized to default values and set up externally

        Ok(player)
    }
}
