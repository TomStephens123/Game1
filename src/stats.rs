//! Player stats and health management system
//!
//! This module provides a comprehensive stats system for player characters and potentially
//! other entities. It includes:
//! - Health management with damage and healing
//! - Base stats (movement speed, attack damage, attack speed, defense)
//! - Stat modifiers for buffs/debuffs
//! - Type-safe stat calculations
//!
//! # Design Philosophy
//!
//! This system uses f32 for all stat values to support:
//! - Percentage-based calculations (defense as 0.0-1.0)
//! - Fractional modifiers (1.5x damage, 0.8x speed)
//! - Smooth interpolation for visual effects
//!
//! # Rust Learning Notes
//!
//! This module demonstrates:
//! - **NewType Pattern**: Wrapping primitives in meaningful types (`Health`)
//! - **Enums for Type Safety**: `StatType` prevents mixing up stat categories
//! - **Struct Methods**: Encapsulating behavior with data
//! - **Option Types**: Handling cases like overkill damage

use std::time::Duration;

/// Represents a character's health points
///
/// Health is tracked separately from max health to enable:
/// - Damage calculations that reduce current health
/// - Healing that can't exceed max health
/// - Percentage-based health checks
///
/// # Example
///
/// ```rust
/// let mut health = Health::new(100.0);
/// health.take_damage(30.0);
/// assert_eq!(health.current(), 70.0);
/// assert_eq!(health.percentage(), 0.7);
/// ```
#[derive(Debug, Clone)]
pub struct Health {
    current: f32,
    #[allow(dead_code)] // Reserved for health bar display
    max: f32,
}

impl Health {
    /// Creates a new Health instance with full health
    pub fn new(max: f32) -> Self {
        Health { current: max, max }
    }

    /// Returns the current health value
    #[allow(dead_code)] // Reserved for health bar display
    pub fn current(&self) -> f32 {
        self.current
    }

    /// Returns the maximum health value
    #[allow(dead_code)] // Reserved for health bar display
    pub fn max(&self) -> f32 {
        self.max
    }

    /// Returns health as a percentage (0.0 to 1.0)
    #[allow(dead_code)] // Reserved for health bar display
    pub fn percentage(&self) -> f32 {
        if self.max <= 0.0 {
            0.0
        } else {
            self.current / self.max
        }
    }

    /// Checks if the entity is alive (health > 0)
    #[allow(dead_code)] // Already exposed via Stats
    pub fn is_alive(&self) -> bool {
        self.current > 0.0
    }

    /// Applies damage to health
    ///
    /// Returns a `DamageResult` containing:
    /// - How much damage was actually dealt
    /// - Whether the damage was fatal
    /// - How much overkill damage occurred (if fatal)
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut health = Health::new(100.0);
    /// let result = health.take_damage(150.0);
    /// assert_eq!(result.damage_dealt, 100.0);
    /// assert_eq!(result.is_fatal, true);
    /// assert_eq!(result.overkill, 50.0);
    /// ```
    pub fn take_damage(&mut self, amount: f32) -> DamageResult {
        let old_health = self.current;
        self.current = (self.current - amount).max(0.0);

        DamageResult {
            damage_dealt: old_health - self.current,
            is_fatal: self.current <= 0.0,
            overkill: if self.current <= 0.0 {
                amount - old_health
            } else {
                0.0
            },
        }
    }

    /// Heals health, capped at max health
    ///
    /// Returns the actual amount healed (which may be less than requested
    /// if already near max health)
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut health = Health::new(100.0);
    /// health.take_damage(50.0);
    /// let healed = health.heal(100.0);
    /// assert_eq!(healed, 50.0); // Only healed what was missing
    /// assert_eq!(health.current(), 100.0);
    /// ```
    #[allow(dead_code)] // Reserved for healing items/abilities
    pub fn heal(&mut self, amount: f32) -> f32 {
        let old_health = self.current;
        self.current = (self.current + amount).min(self.max);
        self.current - old_health
    }

    /// Sets the maximum health and adjusts current health if needed
    ///
    /// If new max is lower than current health, current health is capped to new max
    #[allow(dead_code)] // Reserved for level-up/stat boost features
    pub fn set_max(&mut self, new_max: f32) {
        self.max = new_max;
        if self.current > self.max {
            self.current = self.max;
        }
    }
}

/// Result of a damage operation
///
/// Provides detailed information about damage dealt, useful for:
/// - Visual feedback (damage numbers)
/// - Game logic (death handling)
/// - Statistics tracking
#[derive(Debug, Clone)]
pub struct DamageResult {
    /// Actual damage dealt (may be less than requested if target had less health)
    #[allow(dead_code)] // Reserved for damage number display
    pub damage_dealt: f32,
    /// Whether this damage killed the target
    pub is_fatal: bool,
    /// Excess damage beyond what was needed to kill (0.0 if not fatal)
    #[allow(dead_code)] // Reserved for damage number display
    pub overkill: f32,
}

impl DamageResult {
    /// Creates a result representing no damage dealt
    pub fn no_damage() -> Self {
        DamageResult {
            damage_dealt: 0.0,
            is_fatal: false,
            overkill: 0.0,
        }
    }
}

/// Categories of stats that can be modified
///
/// Using an enum ensures we can't accidentally mix up different stat types
/// when applying modifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(dead_code)] // Reserved for future stat modifier system
pub enum StatType {
    /// Movement speed in pixels per frame
    MovementSpeed,
    /// Base attack damage
    AttackDamage,
    /// Attacks per second
    AttackSpeed,
    /// Damage reduction (0.0 = no reduction, 1.0 = invulnerable)
    Defense,
    /// Maximum health points
    MaxHealth,
}

/// Types of stat modifications
///
/// Modifiers are applied in a specific order to ensure consistent behavior:
/// 1. Override - replaces the value completely
/// 2. Flat - adds/subtracts a fixed amount
/// 3. Percentage - multiplies by a factor (0.5 = +50%)
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)] // Reserved for future stat modifier system
pub enum StatModifier {
    /// Completely replace the stat value
    Override(f32),
    /// Add or subtract a flat amount
    Flat(f32),
    /// Multiply by (1.0 + value). So 0.5 means +50%, -0.2 means -20%
    Percentage(f32),
}

/// A stat modification effect
///
/// Represents a buff, debuff, or permanent stat change
#[derive(Debug, Clone)]
#[allow(dead_code)] // Reserved for future stat modifier system
pub struct ModifierEffect {
    /// Which stat this modifies
    pub stat_type: StatType,
    /// The type and value of modification
    pub modifier: StatModifier,
    /// How long the effect lasts (None = permanent)
    pub duration: Option<Duration>,
    /// What applied this effect (for debugging/UI)
    pub source: String,
}

/// Container for all base stats
///
/// This struct holds the "base" values before any modifiers are applied.
/// Use the `effective_stat()` method to get the actual value after modifiers.
#[derive(Debug, Clone)]
pub struct Stats {
    pub health: Health,
    pub movement_speed: f32,
    pub attack_damage: f32,
    pub attack_speed: f32,
    pub defense: f32,
    pub max_health: f32,
}

impl Stats {
    /// Creates a new Stats instance with default values
    ///
    /// Default values are balanced for a starting player character
    pub fn new() -> Self {
        Stats {
            health: Health::new(10.0),
            movement_speed: 3.0,
            attack_damage: 3.0,  // 3 damage per hit (slimes have 8 HP, so 3 hits to kill)
            attack_speed: 3.0,
            defense: 0.0,
            max_health: 10.0,
        }
    }

    /// Gets the base value of a stat (before modifiers)
    #[allow(dead_code)] // Reserved for future stat modifier system
    fn base_stat(&self, stat_type: StatType) -> f32 {
        match stat_type {
            StatType::MovementSpeed => self.movement_speed,
            StatType::AttackDamage => self.attack_damage,
            StatType::AttackSpeed => self.attack_speed,
            StatType::Defense => self.defense,
            StatType::MaxHealth => self.max_health,
        }
    }

    /// Calculates the effective value of a stat after applying modifiers
    ///
    /// Modifiers are applied in order:
    /// 1. Check for Override - if found, return that value immediately
    /// 2. Sum all Flat modifiers
    /// 3. Sum all Percentage modifiers
    /// 4. Apply: (base + flat) * (1.0 + percentage)
    ///
    /// # Example
    ///
    /// ```rust
    /// let stats = Stats::new();
    /// let modifiers = vec![
    ///     ModifierEffect {
    ///         stat_type: StatType::AttackDamage,
    ///         modifier: StatModifier::Flat(5.0),
    ///         duration: None,
    ///         source: "Sword".to_string(),
    ///     },
    ///     ModifierEffect {
    ///         stat_type: StatType::AttackDamage,
    ///         modifier: StatModifier::Percentage(0.5),  // +50%
    ///         duration: None,
    ///         source: "Rage".to_string(),
    ///     },
    /// ];
    ///
    /// // Base (10) + Flat (5) = 15, then * 1.5 = 22.5
    /// let effective = stats.effective_stat(StatType::AttackDamage, &modifiers);
    /// assert_eq!(effective, 22.5);
    /// ```
    #[allow(dead_code)] // Reserved for future stat modifier system
    pub fn effective_stat(&self, stat_type: StatType, modifiers: &[ModifierEffect]) -> f32 {
        let base_value = self.base_stat(stat_type);

        // Check for override first
        for modifier in modifiers {
            if modifier.stat_type == stat_type {
                if let StatModifier::Override(value) = modifier.modifier {
                    return value;
                }
            }
        }

        // Sum flat and percentage modifiers
        let mut flat_bonus = 0.0;
        let mut percentage_multiplier = 1.0;

        for modifier in modifiers {
            if modifier.stat_type != stat_type {
                continue;
            }

            match modifier.modifier {
                StatModifier::Override(_) => {} // Already handled above
                StatModifier::Flat(value) => flat_bonus += value,
                StatModifier::Percentage(value) => percentage_multiplier += value,
            }
        }

        // Apply formula: (base + flat) * percentage_multiplier
        (base_value + flat_bonus) * percentage_multiplier
    }
}

impl Default for Stats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_damage() {
        let mut health = Health::new(100.0);
        let result = health.take_damage(30.0);

        assert_eq!(result.damage_dealt, 30.0);
        assert_eq!(health.current(), 70.0);
        assert!(!result.is_fatal);
        assert_eq!(result.overkill, 0.0);
    }

    #[test]
    fn test_health_fatal_damage() {
        let mut health = Health::new(100.0);
        let result = health.take_damage(150.0);

        assert_eq!(result.damage_dealt, 100.0);
        assert_eq!(health.current(), 0.0);
        assert!(result.is_fatal);
        assert_eq!(result.overkill, 50.0);
    }

    #[test]
    fn test_health_healing() {
        let mut health = Health::new(100.0);
        health.take_damage(50.0);

        let healed = health.heal(30.0);
        assert_eq!(healed, 30.0);
        assert_eq!(health.current(), 80.0);
    }

    #[test]
    fn test_health_overheal_caps() {
        let mut health = Health::new(100.0);
        health.take_damage(50.0);

        let healed = health.heal(100.0);
        assert_eq!(healed, 50.0); // Only healed what was missing
        assert_eq!(health.current(), 100.0);
    }

    #[test]
    fn test_health_percentage() {
        let mut health = Health::new(100.0);
        health.take_damage(25.0);

        assert_eq!(health.percentage(), 0.75);
    }

    #[test]
    fn test_stat_modifiers_flat() {
        let stats = Stats::new();
        let modifiers = vec![ModifierEffect {
            stat_type: StatType::AttackDamage,
            modifier: StatModifier::Flat(5.0),
            duration: None,
            source: "Test".to_string(),
        }];

        let effective = stats.effective_stat(StatType::AttackDamage, &modifiers);
        assert_eq!(effective, 15.0); // Base 10 + 5
    }

    #[test]
    fn test_stat_modifiers_percentage() {
        let stats = Stats::new();
        let modifiers = vec![ModifierEffect {
            stat_type: StatType::AttackDamage,
            modifier: StatModifier::Percentage(0.5), // +50%
            duration: None,
            source: "Test".to_string(),
        }];

        let effective = stats.effective_stat(StatType::AttackDamage, &modifiers);
        assert_eq!(effective, 15.0); // Base 10 * 1.5
    }

    #[test]
    fn test_stat_modifiers_stacking() {
        let stats = Stats::new();
        let modifiers = vec![
            ModifierEffect {
                stat_type: StatType::AttackDamage,
                modifier: StatModifier::Flat(5.0),
                duration: None,
                source: "Weapon".to_string(),
            },
            ModifierEffect {
                stat_type: StatType::AttackDamage,
                modifier: StatModifier::Percentage(0.5), // +50%
                duration: None,
                source: "Buff".to_string(),
            },
        ];

        // (10 + 5) * 1.5 = 22.5
        let effective = stats.effective_stat(StatType::AttackDamage, &modifiers);
        assert_eq!(effective, 22.5);
    }

    #[test]
    fn test_stat_modifiers_override() {
        let stats = Stats::new();
        let modifiers = vec![
            ModifierEffect {
                stat_type: StatType::AttackDamage,
                modifier: StatModifier::Flat(5.0),
                duration: None,
                source: "Weapon".to_string(),
            },
            ModifierEffect {
                stat_type: StatType::AttackDamage,
                modifier: StatModifier::Override(100.0),
                duration: None,
                source: "God Mode".to_string(),
            },
        ];

        // Override ignores all other modifiers
        let effective = stats.effective_stat(StatType::AttackDamage, &modifiers);
        assert_eq!(effective, 100.0);
    }
}
