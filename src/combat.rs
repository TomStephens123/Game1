//! Combat system for damage calculation and death handling
//!
//! This module provides the combat layer on top of the stats system, including:
//! - Damage types (Physical, Magical, True)
//! - Damage sources (Enemy, Environment, etc.)
//! - Player state management (Alive/Dead)
//! - Defense calculations
//!
//! # Rust Learning Notes
//!
//! This module demonstrates:
//! - **Enums with data**: `PlayerState::Dead { death_time }`
//! - **Pattern matching**: Using `matches!()` for state checks
//! - **Type safety**: Different damage types are handled differently

use std::time::Instant;

/// Types of damage that can be dealt
///
/// Different damage types may be affected by different resistances/defenses
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageType {
    /// Physical damage (affected by defense stat)
    Physical,
    /// Magical damage (could have separate resistance in future)
    Magical,
    /// True damage (ignores all defenses)
    True,
}

/// Source of damage for tracking and game logic
///
/// Useful for:
/// - Achievement tracking ("killed by X")
/// - AI behavior (retaliate against attacker)
/// - Visual feedback (different effects per source)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageSource {
    /// Damage from an enemy entity
    /// In the future, this could store an entity ID
    Enemy,
    /// Environmental damage (spikes, lava, etc.)
    Environment,
    /// Self-inflicted damage (fall damage, self-destruct, etc.)
    SelfInflicted,
}

/// A complete damage event with type and source information
///
/// This provides rich context for damage calculations and visual feedback
///
/// # Example
///
/// ```rust
/// let damage = DamageEvent {
///     amount: 25.0,
///     damage_type: DamageType::Physical,
///     source: DamageSource::Enemy,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct DamageEvent {
    /// Base damage amount (before defense/resistance)
    pub amount: f32,
    /// Type of damage
    pub damage_type: DamageType,
    /// What caused this damage
    pub source: DamageSource,
}

impl DamageEvent {
    /// Creates a new physical damage event
    pub fn physical(amount: f32, source: DamageSource) -> Self {
        DamageEvent {
            amount,
            damage_type: DamageType::Physical,
            source,
        }
    }

    /// Creates a new magical damage event
    pub fn magical(amount: f32, source: DamageSource) -> Self {
        DamageEvent {
            amount,
            damage_type: DamageType::Magical,
            source,
        }
    }

    /// Creates a new true damage event (ignores defenses)
    pub fn true_damage(amount: f32, source: DamageSource) -> Self {
        DamageEvent {
            amount,
            damage_type: DamageType::True,
            source,
        }
    }
}

/// Player state for life/death management
///
/// # Rust Learning: Enums with Associated Data
///
/// This enum demonstrates Rust's powerful enum system. Unlike simple enums in other languages,
/// Rust enums can carry data with each variant. The `Dead` variant stores when the player died,
/// which could be used for:
/// - Death animations (how long has player been dead?)
/// - Respawn timers
/// - Game over screens
///
/// # Example
///
/// ```rust
/// let mut state = PlayerState::Alive;
///
/// if damage_was_fatal {
///     state = PlayerState::Dead { death_time: Instant::now() };
/// }
///
/// // Pattern matching with data extraction
/// match state {
///     PlayerState::Alive => println!("Still fighting!"),
///     PlayerState::Dead { death_time } => {
///         let elapsed = death_time.elapsed().as_secs();
///         println!("Died {} seconds ago", elapsed);
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub enum PlayerState {
    /// Player is alive and can take actions
    Alive,
    /// Player is dead (stores when death occurred)
    Dead { death_time: Instant },
}

impl PlayerState {
    /// Checks if the player is alive
    ///
    /// # Rust Learning: Pattern Matching with matches!()
    ///
    /// The `matches!()` macro is a concise way to check if a value matches a pattern.
    /// It's equivalent to:
    /// ```rust
    /// match self {
    ///     PlayerState::Alive => true,
    ///     _ => false,
    /// }
    /// ```
    pub fn is_alive(&self) -> bool {
        matches!(self, PlayerState::Alive)
    }

    /// Checks if the player is dead
    pub fn is_dead(&self) -> bool {
        matches!(self, PlayerState::Dead { .. })
    }

    /// Gets the time of death if player is dead
    ///
    /// Returns `None` if player is alive
    pub fn death_time(&self) -> Option<Instant> {
        match self {
            PlayerState::Alive => None,
            PlayerState::Dead { death_time } => Some(*death_time),
        }
    }
}

impl Default for PlayerState {
    fn default() -> Self {
        PlayerState::Alive
    }
}

/// Attack event information
///
/// Created when a player/entity performs an attack
#[derive(Debug, Clone)]
pub struct AttackEvent {
    /// Base damage of the attack
    pub damage: f32,
    /// Position of the attacker
    pub position: (i32, i32),
    /// Direction the attack is facing
    pub direction: crate::animation::Direction,
    /// Range of the attack in pixels
    pub range: i32,
}

impl AttackEvent {
    /// Creates a new attack event
    pub fn new(damage: f32, position: (i32, i32), direction: crate::animation::Direction, range: i32) -> Self {
        AttackEvent {
            damage,
            position,
            direction,
            range,
        }
    }

    /// Converts this attack to a damage event
    pub fn to_damage_event(&self, damage_type: DamageType, source: DamageSource) -> DamageEvent {
        DamageEvent {
            amount: self.damage,
            damage_type,
            source,
        }
    }

    /// Gets the attack hitbox as a Rect based on position, direction, and range
    ///
    /// This creates a rectangular hitbox in front of the attacker based on their facing direction
    pub fn get_hitbox(&self) -> sdl2::rect::Rect {
        use crate::animation::Direction;

        // Attack hitbox is a rectangle in front of the player
        let hitbox_size = self.range;

        // Calculate offset based on direction (all calculations in i32)
        let (offset_x, offset_y) = match self.direction {
            Direction::North => (-(hitbox_size / 2), -hitbox_size),
            Direction::NorthEast => (0, -hitbox_size),
            Direction::East => (0, -(hitbox_size / 2)),
            Direction::SouthEast => (0, 0),
            Direction::South => (-(hitbox_size / 2), 0),
            Direction::SouthWest => (-hitbox_size, 0),
            Direction::West => (-hitbox_size, -(hitbox_size / 2)),
            Direction::NorthWest => (-hitbox_size, -hitbox_size),
        };

        sdl2::rect::Rect::new(
            self.position.0 + offset_x,
            self.position.1 + offset_y,
            hitbox_size as u32,
            hitbox_size as u32,
        )
    }
}

/// Calculates final damage after applying defense
///
/// # Formula
///
/// - **Physical damage**: `damage * (1.0 - defense)`
///   - defense is clamped to 0.0-1.0 (0% to 100% reduction)
///   - Example: 100 damage with 0.25 defense = 75 damage
///
/// - **Magical damage**: Currently reduced by 20% flat (could add magic resistance later)
///
/// - **True damage**: Ignores all defenses
///
/// # Example
///
/// ```rust
/// let event = DamageEvent::physical(100.0, DamageSource::Enemy);
/// let defense = 0.25; // 25% damage reduction
/// let final_damage = calculate_damage_with_defense(&event, defense);
/// assert_eq!(final_damage, 75.0);
/// ```
pub fn calculate_damage_with_defense(event: &DamageEvent, defense: f32) -> f32 {
    // Clamp defense to valid range (0.0 = no reduction, 1.0 = full immunity)
    let clamped_defense = defense.clamp(0.0, 1.0);

    match event.damage_type {
        DamageType::Physical => {
            // Physical damage reduced by defense percentage
            event.amount * (1.0 - clamped_defense)
        }
        DamageType::Magical => {
            // Magical damage: 20% flat reduction for now
            // In future, could add separate magic_resistance stat
            event.amount * 0.8
        }
        DamageType::True => {
            // True damage ignores all defenses
            event.amount
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_damage_event_constructors() {
        let physical = DamageEvent::physical(10.0, DamageSource::Enemy);
        assert_eq!(physical.damage_type, DamageType::Physical);
        assert_eq!(physical.amount, 10.0);

        let magical = DamageEvent::magical(15.0, DamageSource::Environment);
        assert_eq!(magical.damage_type, DamageType::Magical);

        let true_dmg = DamageEvent::true_damage(20.0, DamageSource::SelfInflicted);
        assert_eq!(true_dmg.damage_type, DamageType::True);
    }

    #[test]
    fn test_player_state_alive() {
        let state = PlayerState::Alive;
        assert!(state.is_alive());
        assert!(!state.is_dead());
        assert!(state.death_time().is_none());
    }

    #[test]
    fn test_player_state_dead() {
        let death_time = Instant::now();
        let state = PlayerState::Dead { death_time };

        assert!(!state.is_alive());
        assert!(state.is_dead());
        assert_eq!(state.death_time(), Some(death_time));
    }

    #[test]
    fn test_physical_damage_with_defense() {
        let event = DamageEvent::physical(100.0, DamageSource::Enemy);

        // 25% defense = 25% damage reduction
        let damage = calculate_damage_with_defense(&event, 0.25);
        assert_eq!(damage, 75.0);

        // 50% defense = 50% damage reduction
        let damage = calculate_damage_with_defense(&event, 0.5);
        assert_eq!(damage, 50.0);

        // 100% defense = 100% damage reduction (immunity)
        let damage = calculate_damage_with_defense(&event, 1.0);
        assert_eq!(damage, 0.0);
    }

    #[test]
    fn test_defense_clamping() {
        let event = DamageEvent::physical(100.0, DamageSource::Enemy);

        // Defense > 1.0 should be clamped to 1.0
        let damage = calculate_damage_with_defense(&event, 2.0);
        assert_eq!(damage, 0.0);

        // Negative defense should be clamped to 0.0
        let damage = calculate_damage_with_defense(&event, -0.5);
        assert_eq!(damage, 100.0);
    }

    #[test]
    fn test_magical_damage() {
        let event = DamageEvent::magical(100.0, DamageSource::Environment);
        let defense = 0.5; // Defense doesn't affect magical damage currently

        let damage = calculate_damage_with_defense(&event, defense);
        assert_eq!(damage, 80.0); // 20% flat reduction
    }

    #[test]
    fn test_true_damage_ignores_defense() {
        let event = DamageEvent::true_damage(100.0, DamageSource::SelfInflicted);

        // True damage ignores defense completely
        let damage = calculate_damage_with_defense(&event, 1.0);
        assert_eq!(damage, 100.0);
    }

    #[test]
    fn test_attack_event_conversion() {
        use crate::animation::Direction;

        let attack = AttackEvent::new(50.0, (100, 100), Direction::East, 32);
        let damage_event = attack.to_damage_event(DamageType::Physical, DamageSource::Enemy);

        assert_eq!(damage_event.amount, 50.0);
        assert_eq!(damage_event.damage_type, DamageType::Physical);
        assert_eq!(damage_event.source, DamageSource::Enemy);
    }
}
