use crate::animation::{AnimationController, AnimationState, Direction, determine_animation_state};
use crate::collision::{Collidable, CollisionLayer};
use crate::combat::{AttackEvent, DamageEvent, PlayerState, calculate_damage_with_defense};
use crate::render::DepthSortable;
use crate::save::{Saveable, SaveData, SaveError};
use crate::stats::{Stats, DamageResult, ModifierEffect, StatType};
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

    // Active stat modifiers (buffs/debuffs from entities, items, etc.)
    pub active_modifiers: Vec<ModifierEffect>,

    // Player state (Alive/Dead)
    pub state: PlayerState,

    // Invulnerability system (keep existing pattern)
    pub is_invulnerable: bool,
    invulnerability_timer: Instant,
    invulnerability_duration: f32, // seconds

    // Attack cooldown system
    last_attack_time: Instant,

    // Environmental collision box configuration (for walls, static objects)
    // This is separate from the damage hitbox to allow tight movement control
    // while maintaining fair combat. All values are in unscaled sprite pixels.
    pub collision_offset_x: i32,
    pub collision_offset_y: i32,
    pub collision_width: u32,
    pub collision_height: u32,

    // Damage hitbox configuration (for getting hit by enemies)
    // This is typically larger than the collision box to ensure fair combat.
    // All values are in unscaled sprite pixels.
    pub damage_offset_x: i32,
    pub damage_offset_y: i32,
    pub damage_width: u32,
    pub damage_height: u32,
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
            active_modifiers: Vec::new(),
            state: PlayerState::Alive,
            is_invulnerable: false,
            invulnerability_timer: Instant::now(),
            invulnerability_duration: 1.0, // 1 second of invulnerability after taking damage
            last_attack_time: Instant::now(),

            // Environmental collision box (tight, at feet)
            // Centered horizontally, at base of sprite
            // Dimensions: 8px wide x 16px tall (unscaled)
            collision_offset_x: -6,   // Center 8px width (-4 to +4 from anchor)
            collision_offset_y: -16,  // 18px up from anchor (2px higher than bottom half)
            collision_width: 12,       // Narrow width for tight movement
            collision_height: 8,     // Tall height for proper collision

            // Damage hitbox (generous, covers body)
            // Centered horizontally, covers most of sprite
            damage_offset_x: -8,      // Center 16px width (-8 to +8 from anchor)
            damage_offset_y: -24,     // 24px up from anchor (covers y=8 to y=32)
            damage_width: 16,         // Square hitbox
            damage_height: 16,        // Covers upper body
        }
    }

    pub fn set_animation_controller(&mut self, controller: AnimationController<'a>) {
        self.animation_controller = controller;
    }

    pub fn update(&mut self, keyboard_state: &sdl2::keyboard::KeyboardState) {
        self.velocity_x = 0;
        self.velocity_y = 0;

        // Get effective movement speed from stats with modifiers applied
        let effective_speed = self.stats.effective_stat(StatType::MovementSpeed, &self.active_modifiers) as i32;

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

        // Calculate render position from anchor point (bottom-center)
        // The anchor is where the player "stands" in the world
        // We render the sprite upward and centered from this point
        let render_x = self.x - (scaled_width / 2) as i32;
        let render_y = self.y - scaled_height as i32;

        let dest_rect = Rect::new(render_x, render_y, scaled_width, scaled_height);

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

        // Attack cooldown based on attack_speed stat with modifiers applied
        // attack_speed is attacks per second, so cooldown = 1.0 / attack_speed
        let attack_cooldown = 1.0 / self.stats.effective_stat(StatType::AttackSpeed, &self.active_modifiers);
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

        // Attack originates from player's visual center, not anchor (feet)
        // This ensures attacks extend outward from the body, not the ground
        const SPRITE_SCALE: u32 = 2;
        let player_center_y = self.y - (self.height * SPRITE_SCALE / 2) as i32;

        Some(AttackEvent::new(
            self.stats.effective_stat(StatType::AttackDamage, &self.active_modifiers),
            (self.x, player_center_y),  // Use visual center, not anchor
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

        // Calculate final damage with defense (including modifiers)
        let effective_defense = self.stats.effective_stat(StatType::Defense, &self.active_modifiers);
        let final_damage = calculate_damage_with_defense(&damage_event, effective_defense);

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

    /// Sets custom environmental collision box parameters.
    ///
    /// Use this to adjust the collision box for walls and static objects.
    /// All values are in unscaled sprite pixels (will be multiplied by scale factor).
    /// Offsets are relative to the anchor point (bottom-center).
    ///
    /// # Parameters
    /// - `offset_x`: Horizontal offset from anchor point
    /// - `offset_y`: Vertical offset from anchor point (negative = upward)
    /// - `width`: Width of the collision box (before scaling)
    /// - `height`: Height of the collision box (before scaling)
    #[allow(dead_code)]
    pub fn set_collision_box(&mut self, offset_x: i32, offset_y: i32, width: u32, height: u32) {
        self.collision_offset_x = offset_x;
        self.collision_offset_y = offset_y;
        self.collision_width = width;
        self.collision_height = height;
    }

    /// Sets custom damage hitbox parameters.
    ///
    /// Use this to adjust the hitbox for getting hit by enemies.
    /// All values are in unscaled sprite pixels (will be multiplied by scale factor).
    /// Offsets are relative to the anchor point (bottom-center).
    ///
    /// # Parameters
    /// - `offset_x`: Horizontal offset from anchor point
    /// - `offset_y`: Vertical offset from anchor point (negative = upward)
    /// - `width`: Width of the damage hitbox (before scaling)
    /// - `height`: Height of the damage hitbox (before scaling)
    #[allow(dead_code)]
    pub fn set_damage_hitbox(&mut self, offset_x: i32, offset_y: i32, width: u32, height: u32) {
        self.damage_offset_x = offset_x;
        self.damage_offset_y = offset_y;
        self.damage_width = width;
        self.damage_height = height;
    }

    /// Gets the bounding box for damage detection (getting hit by enemies).
    ///
    /// This is separate from environmental collision bounds (`get_bounds()`) and is
    /// typically larger to ensure the player doesn't feel cheated by pixel-perfect hits.
    /// The damage hitbox is calculated from the player's anchor point (bottom-center).
    ///
    /// # Returns
    /// A `Rect` representing the damage hitbox in world coordinates
    ///
    /// # Example
    /// ```rust
    /// // Check if an enemy attack hits the player
    /// let player_damage_bounds = player.get_damage_bounds();
    /// if aabb_intersect(&enemy_attack_rect, &player_damage_bounds) {
    ///     player.take_damage(damage_event);
    /// }
    /// ```
    pub fn get_damage_bounds(&self) -> Rect {
        const SPRITE_SCALE: u32 = 2;
        let offset_x = self.damage_offset_x * SPRITE_SCALE as i32;
        let offset_y = self.damage_offset_y * SPRITE_SCALE as i32;
        let scaled_width = self.damage_width * SPRITE_SCALE;
        let scaled_height = self.damage_height * SPRITE_SCALE;

        Rect::new(
            self.x + offset_x,
            self.y + offset_y,
            scaled_width,
            scaled_height,
        )
    }
}

// Collision System Implementation
//
// This trait implementation makes Player participate in the collision system.
// The environmental collision bounds are calculated from the player's anchor point
// (bottom-center) and are intentionally tight to allow responsive movement.
impl<'a> Collidable for Player<'a> {
    fn get_bounds(&self) -> Rect {
        // Environmental collision box (for walls, static objects)
        // Calculated from anchor point at bottom-center of sprite
        const SPRITE_SCALE: u32 = 2;
        let offset_x = self.collision_offset_x * SPRITE_SCALE as i32;
        let offset_y = self.collision_offset_y * SPRITE_SCALE as i32;
        let scaled_width = self.collision_width * SPRITE_SCALE;
        let scaled_height = self.collision_height * SPRITE_SCALE;

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
// Depth Sorting Render System
// ==============================================================================

/// Implementation of depth sorting for the Player.
///
/// The player's depth is determined by their anchor point (bottom-center).
/// This ensures proper visual layering in the 2.5D game world - entities with
/// smaller Y values render first (farther back), creating the illusion of depth.
///
/// See docs/systems/depth-sorting-render-system.md for design documentation.
impl DepthSortable for Player<'_> {
    fn get_depth_y(&self) -> i32 {
        // Player's position is already at the anchor point (bottom-center)
        // This is where the player "touches the ground" in the game world
        // No calculation needed - the anchor is the depth!
        self.y
    }

    fn render(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        // Delegate to existing render implementation
        // This avoids code duplication and keeps the existing render logic intact
        Player::render(self, canvas)
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

            // Note: Collision/damage hitbox values are NOT saved
            // They are configuration constants defined in code, not player state
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
            // Note: Hitbox values not saved - using code defaults
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

        // Note: Hitbox values, animation controller, timers, and transient state are NOT saved
        // Hitbox values use code defaults (configuration, not player state)
        // Animation controller and timers are initialized to default values

        Ok(player)
    }
}
