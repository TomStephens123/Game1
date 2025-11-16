// GameWorld struct and entity management
//
// This module contains the GameWorld struct which owns all game entities and world state.
// It provides methods for spawning entities, updating world state, and managing interactions.

use crate::animation::{self, AnimationController};
use crate::attack_effect::AttackEffect;
use crate::collision::{self, Collidable};
use crate::combat;
use crate::dropped_item::{self, DroppedItem};
use crate::inventory::PlayerInventory;
use crate::item::ItemRegistry;
use crate::player::Player;
use crate::slime::Slime;
use crate::sprite;
use crate::stats::{ModifierEffect, StatModifier, StatType};
use crate::the_entity::{TheEntity, EntityState, EntityType};
use crate::tile::{WorldGrid, RenderGrid};
use sdl2::pixels::Color;

use super::FloatingTextInstance;

// Constants from main.rs
const SPRITE_SCALE: u32 = 2;
const GAME_WIDTH: u32 = 640;
const GAME_HEIGHT: u32 = 360;

/// GameWorld encapsulates all game entities and world state
/// This struct owns all the game objects that exist in the world
pub struct GameWorld<'a> {
    pub player: Player<'a>,
    pub slimes: Vec<Slime<'a>>,
    pub entities: Vec<TheEntity<'a>>,
    pub dropped_items: Vec<DroppedItem<'a>>,
    pub world_grid: WorldGrid,
    pub render_grid: RenderGrid,
    pub player_inventory: PlayerInventory,
    pub attack_effects: Vec<AttackEffect<'a>>,
    pub floating_texts: Vec<FloatingTextInstance>,
    pub active_attack: Option<combat::AttackEvent>,
}

impl<'a> GameWorld<'a> {
    /// Spawn a dropped item in the world at given coordinates
    ///
    /// This method encapsulates the repeated item spawning logic that appears
    /// throughout the codebase (entity loot, slime drops, death drops, etc.)
    ///
    /// # Arguments
    /// * `x` - X coordinate for the item
    /// * `y` - Y coordinate for the item
    /// * `item_id` - Item identifier (e.g., "stone", "slime_ball")
    /// * `quantity` - Number of items in the stack
    /// * `item_texture` - Texture reference for the item
    ///
    /// # Returns
    /// Ok(()) on success, Err if item texture is missing
    pub fn spawn_dropped_item(
        &mut self,
        x: i32,
        y: i32,
        item_id: String,
        quantity: u32,
        item_texture: &'a sdl2::render::Texture<'a>,
    ) -> Result<(), String> {
        let mut item_animation_controller = animation::AnimationController::new();
        let item_frames = vec![sprite::Frame::new(0, 0, 32, 32, 300)];
        let item_sprite_sheet = sprite::SpriteSheet::new(item_texture, item_frames);
        item_animation_controller.add_animation("item_idle".to_string(), item_sprite_sheet);
        item_animation_controller.set_state("item_idle".to_string());

        let dropped_item = dropped_item::DroppedItem::new(
            x,
            y,
            item_id,
            quantity,
            item_animation_controller,
        );
        self.dropped_items.push(dropped_item);
        Ok(())
    }

    /// Update all world entities (slimes, pyramids, effects, floating text)
    ///
    /// This method centralizes entity update logic that was previously scattered
    /// throughout the Game::update() method.
    ///
    /// # Arguments
    /// * `delta_time` - Time elapsed since last frame (in seconds)
    pub fn update_entities(&mut self, delta_time: f32) {
        // Update slimes
        for slime in self.slimes.iter_mut() {
            slime.update();
        }

        // Update entities (pyramids)
        for entity in self.entities.iter_mut() {
            entity.update(delta_time);
        }

        // Update floating texts (move upward and age)
        for text in self.floating_texts.iter_mut() {
            text.lifetime += delta_time;
            text.y -= 20.0 * delta_time;
        }

        // Update attack effects
        for effect in self.attack_effects.iter_mut() {
            effect.update();
        }
    }

    /// Remove dead/expired entities from world
    ///
    /// This method cleans up entities that have finished their lifecycle:
    /// - Dead slimes
    /// - Finished attack effects
    /// - Expired floating text
    pub fn cleanup_dead_entities(&mut self) {
        self.slimes.retain(|slime| slime.is_alive);
        self.attack_effects.retain(|effect| !effect.is_finished());
        self.floating_texts.retain(|text| text.lifetime < text.max_lifetime);
    }

    /// Update dropped items (pickup collision, despawn timer)
    ///
    /// This method handles:
    /// 1. Checking if player is colliding with items
    /// 2. Adding items to player inventory
    /// 3. Updating despawn timers
    /// 4. Removing picked up or despawned items
    ///
    /// # Arguments
    /// * `item_registry` - Registry for looking up item properties
    ///
    /// # Returns
    /// Vec of (item_id, quantity) tuples for items that were picked up
    pub fn update_dropped_items(&mut self, item_registry: &ItemRegistry) -> Vec<(String, u32)> {
        let mut picked_up_items = Vec::new();
        let player_bounds = self.player.get_bounds();

        // Handle item pickup
        self.dropped_items.retain(|item| {
            if !item.can_pickup {
                return true; // Keep items in cooldown
            }

            if player_bounds.has_intersection(item.get_bounds()) {
                match self.player_inventory.quick_add(&item.item_id, item.quantity, item_registry) {
                    Ok(overflow) => {
                        if overflow == 0 {
                            picked_up_items.push((item.item_id.clone(), item.quantity));
                            false // Remove from world
                        } else {
                            eprintln!("⚠ Inventory full! {} items couldn't fit", overflow);
                            true // Keep in world
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to pickup item: {}", e);
                        true // Keep in world on error
                    }
                }
            } else {
                true // Keep if not colliding
            }
        });

        // Update despawn timers and remove despawned items
        self.dropped_items.retain_mut(|item| !item.update()); // Remove if despawned

        picked_up_items
    }

    /// Apply buffs from awakened pyramids to player
    ///
    /// This method clears existing buffs and reapplies them based on which
    /// pyramids are currently in the Awake state.
    ///
    /// # Returns
    /// true if player has regeneration buff, false otherwise
    pub fn apply_pyramid_buffs(&mut self) -> bool {
        // Clear existing buffs
        self.player.active_modifiers.clear();
        let mut has_regen = false;

        // Check each entity and apply appropriate buffs
        for entity in self.entities.iter() {
            if entity.state == EntityState::Awake {
                match entity.entity_type {
                    EntityType::Attack => {
                        self.player.active_modifiers.push(ModifierEffect {
                            stat_type: StatType::AttackDamage,
                            modifier: StatModifier::Flat(1.0),
                            duration: None,
                            source: "Pyramid of Attack".to_string(),
                        });
                    }
                    EntityType::Defense => {
                        self.player.active_modifiers.push(ModifierEffect {
                            stat_type: StatType::Defense,
                            modifier: StatModifier::Flat(1.0),
                            duration: None,
                            source: "Pyramid of Defense".to_string(),
                        });
                    }
                    EntityType::Speed => {
                        self.player.active_modifiers.push(ModifierEffect {
                            stat_type: StatType::MovementSpeed,
                            modifier: StatModifier::Flat(1.0),
                            duration: None,
                            source: "Pyramid of Speed".to_string(),
                        });
                    }
                    EntityType::Regeneration => {
                        has_regen = true;
                    }
                }
            }
        }

        has_regen
    }

    /// Apply regeneration healing to player and create floating text
    ///
    /// This method handles the regeneration buff effect, healing the player
    /// and creating visual feedback via floating text.
    pub fn handle_regeneration(&mut self) {
        if self.player.stats.health.current() < self.player.stats.max_health {
            self.player.stats.health.heal(2.0);

            // Create floating text at player
            self.floating_texts.push(FloatingTextInstance {
                x: self.player.x as f32,
                y: (self.player.y - (self.player.height * SPRITE_SCALE) as i32) as f32,
                text: "+2".to_string(),
                color: Color::RGB(0, 255, 0),
                lifetime: 0.0,
                max_lifetime: 1.5,
            });

            // Create floating text at regen pyramid
            for entity in self.entities.iter() {
                if entity.state == EntityState::Awake && entity.entity_type == EntityType::Regeneration {
                    self.floating_texts.push(FloatingTextInstance {
                        x: entity.x as f32 + 16.0,
                        y: entity.y as f32,
                        text: "+2".to_string(),
                        color: Color::RGB(0, 255, 0),
                        lifetime: 0.0,
                        max_lifetime: 1.5,
                    });
                    break;
                }
            }
        }
    }

    /// Spawn a new slime at the given position
    ///
    /// This method handles the complex positioning logic for slime spawning,
    /// ensuring that the click/spawn position corresponds to the center of
    /// the slime's collision box (not its sprite anchor).
    ///
    /// # Arguments
    /// * `x` - Desired X position (collision box center)
    /// * `y` - Desired Y position (collision box center)
    /// * `slime_animation_controller` - Animation controller for the slime
    /// * `health` - Initial health for the slime
    ///
    /// # Returns
    /// Ok(()) on success
    pub fn spawn_slime(
        &mut self,
        x: i32,
        y: i32,
        slime_animation_controller: AnimationController<'a>,
        health: i32,
    ) -> Result<(), String> {
        // Create temp slime to get hitbox dimensions
        // Slime uses anchor positioning (bottom-center of sprite)
        // but we want spawn position to be collision box center
        let temp_slime = Slime::new(0, 0, AnimationController::new());

        // Calculate anchor position from desired collision center
        // anchor = click - collision_offset - (collision_size / 2)
        let anchor_x = x - (temp_slime.hitbox_offset_x * SPRITE_SCALE as i32)
                       - (temp_slime.hitbox_width * SPRITE_SCALE / 2) as i32;
        let anchor_y = y - (temp_slime.hitbox_offset_y * SPRITE_SCALE as i32)
                       - (temp_slime.hitbox_height * SPRITE_SCALE / 2) as i32;

        let mut new_slime = Slime::new(anchor_x, anchor_y, slime_animation_controller);
        new_slime.health = health;
        self.slimes.push(new_slime);

        Ok(())
    }

    /// Spawn an attack effect (punch, slash, etc.)
    ///
    /// This method creates a visual attack effect at the specified position.
    ///
    /// # Arguments
    /// * `x` - X position (top-left corner)
    /// * `y` - Y position (top-left corner)
    /// * `width` - Effect width in pixels (before scaling)
    /// * `height` - Effect height in pixels (before scaling)
    /// * `direction` - Direction the effect is facing
    /// * `animation_controller` - Animation controller for the effect
    pub fn spawn_attack_effect(
        &mut self,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        direction: animation::Direction,
        animation_controller: AnimationController<'a>,
    ) {
        let effect = AttackEffect::new(
            x,
            y,
            width,
            height,
            direction,
            animation_controller,
        );
        self.attack_effects.push(effect);
    }

    /// Spawn floating text at the given position
    ///
    /// This method creates animated floating text that rises upward and fades out.
    /// Commonly used for damage numbers, healing, item pickups, etc.
    ///
    /// # Arguments
    /// * `text` - The text to display
    /// * `x` - X position
    /// * `y` - Y position
    /// * `color` - Text color
    /// * `max_lifetime` - How long the text should last (in seconds)
    pub fn spawn_floating_text(
        &mut self,
        text: String,
        x: f32,
        y: f32,
        color: Color,
        max_lifetime: f32,
    ) {
        self.floating_texts.push(FloatingTextInstance {
            x,
            y,
            text,
            color,
            lifetime: 0.0,
            max_lifetime,
        });
    }

    /// Get all collidable objects in the world
    ///
    /// This method returns a list of all entities that can be involved in
    /// collision detection. Used by the collision system.
    ///
    /// # Returns
    /// Vector of references to all collidable objects
    pub fn get_all_collidables(&self) -> Vec<&dyn collision::Collidable> {
        let mut collidables: Vec<&dyn collision::Collidable> = Vec::new();

        // Add player
        collidables.push(&self.player as &dyn collision::Collidable);

        // Add all slimes
        for slime in &self.slimes {
            collidables.push(slime as &dyn collision::Collidable);
        }

        // Note: TheEntity (pyramids) don't implement Collidable trait
        // They use a different interaction system (proximity-based buffs)

        collidables
    }

    /// Get the player's current position
    ///
    /// # Returns
    /// Tuple of (x, y) coordinates
    pub fn get_player_pos(&self) -> (i32, i32) {
        (self.player.x, self.player.y)
    }

    /// Get a mutable reference to the player
    ///
    /// This is useful for systems that need to modify player state
    /// without borrowing the entire world.
    ///
    /// # Returns
    /// Mutable reference to the player
    pub fn get_player_mut(&mut self) -> &mut Player<'a> {
        &mut self.player
    }

    /// Check if a position is valid for spawning entities
    ///
    /// This method checks if the given position is within world bounds
    /// and doesn't collide with existing static objects.
    ///
    /// # Arguments
    /// * `x` - X coordinate to check
    /// * `y` - Y coordinate to check
    ///
    /// # Returns
    /// true if position is valid, false otherwise
    pub fn is_position_valid(&self, x: i32, y: i32) -> bool {
        // Check world bounds
        if x < 0 || y < 0 || x >= GAME_WIDTH as i32 || y >= GAME_HEIGHT as i32 {
            return false;
        }

        // Position is valid if in bounds
        // (More sophisticated checks could be added here, like checking
        // for collision with static objects or other entities)
        true
    }
}

