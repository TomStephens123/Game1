# Player Stats and Information System Design

## Overview
This document outlines the design for a comprehensive player stats system that supports health, damage, death, healing, and stat augmentation. The system is designed to be extensible, type-safe, and idiomatic Rust.

**Note**: This is a refactor/enhancement of existing basic systems. The current game has:
- ✅ Basic health/damage (`i32` health values)
- ✅ Collision system (trait-based `Collidable`)
- ✅ Simple invulnerability timer
- ✅ Death detection (health <= 0)

We'll replace these with a more robust, extensible system.

## Goals
1. **Extensibility**: Easy to add new stats and modifiers
2. **Type Safety**: Leverage Rust's type system to prevent invalid states
3. **Performance**: Minimal overhead for stat calculations
4. **Modularity**: Clean separation between stats, modifiers, and effects
5. **Game Integration**: Seamless integration with existing collision and animation systems

## Architecture

### Core Components

#### 1. **Stats Module** (`src/stats.rs`)
Central module containing all stat-related types and logic.

```rust
// Core stat types
pub struct Health {
    current: f32,
    max: f32,
}

pub struct Stats {
    pub health: Health,
    pub movement_speed: f32,
    pub attack_damage: f32,
    pub attack_speed: f32,      // Attacks per second
    pub defense: f32,            // Damage reduction percentage (0.0-1.0)
    pub max_health: f32,         // Base max health
}

// Stat modifiers (for temporary buffs/debuffs)
pub enum StatModifier {
    Flat(f32),           // Add/subtract flat value
    Percentage(f32),     // Multiply by percentage (1.0 = +100%)
    Override(f32),       // Completely override the value
}

pub struct ModifierEffect {
    pub stat_type: StatType,
    pub modifier: StatModifier,
    pub duration: Option<Duration>, // None = permanent
    pub source: String,              // What applied this modifier
}
```

#### 2. **Combat Module** (`src/combat.rs`)
Handles damage, healing, and death logic.

```rust
pub struct DamageEvent {
    pub amount: f32,
    pub damage_type: DamageType,
    pub source: DamageSource,
}

pub enum DamageType {
    Physical,
    Magical,
    True,  // Ignores defense
}

pub enum DamageSource {
    Enemy(EntityId),
    Environment,
    SelfInflicted,
}

pub struct CombatSystem {
    // Handles damage calculation, defense application, etc.
}
```

#### 3. **Effect System** (`src/effects.rs`)
Manages temporary and permanent stat modifications.

```rust
pub struct EffectManager {
    active_effects: Vec<ActiveEffect>,
}

pub struct ActiveEffect {
    pub effect: ModifierEffect,
    pub applied_at: Instant,
}
```

### Data Structures

#### Health System
```rust
impl Health {
    pub fn new(max: f32) -> Self {
        Health {
            current: max,
            max,
        }
    }

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

    pub fn heal(&mut self, amount: f32) -> f32 {
        let old_health = self.current;
        self.current = (self.current + amount).min(self.max);
        self.current - old_health // Return actual healing done
    }

    pub fn is_alive(&self) -> bool {
        self.current > 0.0
    }

    pub fn health_percentage(&self) -> f32 {
        self.current / self.max
    }
}
```

#### Stats Implementation
```rust
impl Stats {
    pub fn new() -> Self {
        Stats {
            health: Health::new(100.0),
            movement_speed: 3.0,
            attack_damage: 10.0,
            attack_speed: 1.0,
            defense: 0.0,
            max_health: 100.0,
        }
    }

    // Apply modifiers to get effective stat value
    pub fn effective_stat(&self, stat_type: StatType, modifiers: &[ModifierEffect]) -> f32 {
        let base_value = self.base_stat(stat_type);

        // Apply modifiers in order: Override -> Flat -> Percentage
        let mut value = base_value;
        let mut flat_bonus = 0.0;
        let mut percentage_multiplier = 1.0;

        for modifier in modifiers {
            if modifier.stat_type != stat_type {
                continue;
            }

            match modifier.modifier {
                StatModifier::Override(v) => return v,
                StatModifier::Flat(v) => flat_bonus += v,
                StatModifier::Percentage(v) => percentage_multiplier += v,
            }
        }

        (value + flat_bonus) * percentage_multiplier
    }

    fn base_stat(&self, stat_type: StatType) -> f32 {
        match stat_type {
            StatType::MovementSpeed => self.movement_speed,
            StatType::AttackDamage => self.attack_damage,
            StatType::AttackSpeed => self.attack_speed,
            StatType::Defense => self.defense,
            StatType::MaxHealth => self.max_health,
        }
    }
}
```

#### Player State Management
```rust
pub enum PlayerState {
    Alive,
    Dead { death_time: Instant },
}

impl PlayerState {
    pub fn is_alive(&self) -> bool {
        !matches!(self, PlayerState::Dead { .. })
    }
}
```

**Note**: Invulnerability is handled separately via `is_invulnerable` bool + timer, not as a state.
This is simpler since you can be invulnerable while alive but not while dead.

## Integration with Existing Player

### Updated Player Structure

**Current Structure** (simplified):
```rust
pub struct Player<'a> {
    // Position & rendering
    pub x: i32,
    pub y: i32,
    pub width: u32, pub height: u32,
    pub speed: i32,
    pub velocity_x: i32, pub velocity_y: i32,
    pub direction: Direction,
    pub is_attacking: bool,
    animation_controller: AnimationController<'a>,

    // Current basic stats (to be replaced)
    pub health: i32,
    pub max_health: i32,
    pub is_invulnerable: bool,
    invulnerability_timer: Instant,
    invulnerability_duration: f32,

    // Collision hitbox
    pub hitbox_offset_x: i32, pub hitbox_offset_y: i32,
    pub hitbox_width: u32, pub hitbox_height: u32,
}
```

**Proposed New Structure**:
```rust
pub struct Player<'a> {
    // Position & rendering (unchanged)
    pub x: i32,
    pub y: i32,
    pub width: u32, pub height: u32,
    pub velocity_x: i32, pub velocity_y: i32,
    pub direction: Direction,
    pub is_attacking: bool,
    animation_controller: AnimationController<'a>,

    // New comprehensive stat system (replaces simple health)
    pub stats: Stats,
    pub state: PlayerState,
    effect_manager: EffectManager,

    // Invulnerability (keep current pattern)
    pub is_invulnerable: bool,
    invulnerability_timer: Instant,
    invulnerability_duration: f32,

    // Combat timing
    last_attack_time: Instant,
    damage_flash_timer: Option<Instant>, // For visual feedback when hit

    // Collision hitbox (unchanged)
    pub hitbox_offset_x: i32, pub hitbox_offset_y: i32,
    pub hitbox_width: u32, pub hitbox_height: u32,
}
```

**Key Changes**:
- Replace `health: i32` and `max_health: i32` with `stats: Stats`
- Replace `pub fn is_alive() -> bool` with `state: PlayerState`
- Add `effect_manager: EffectManager` for buffs/debuffs
- Keep existing invulnerability pattern (it works well)
- Speed becomes part of stats system (currently hardcoded `speed: i32`)

### Key Methods

```rust
impl<'a> Player<'a> {
    pub fn take_damage(&mut self, damage_event: DamageEvent) -> DamageResult {
        // Check invulnerability (existing pattern)
        if self.is_invulnerable || !self.state.is_alive() {
            return DamageResult::no_damage();
        }

        // Apply defense to physical damage
        let final_damage = match damage_event.damage_type {
            DamageType::Physical => {
                let defense = self.effective_defense();
                damage_event.amount * (1.0 - defense)
            }
            DamageType::Magical => damage_event.amount * 0.8, // Example: 20% magic resistance
            DamageType::True => damage_event.amount,
        };

        let result = self.stats.health.take_damage(final_damage);

        if result.is_fatal {
            self.die();
        } else {
            // Visual feedback
            self.damage_flash_timer = Some(Instant::now());

            // Activate invulnerability
            self.is_invulnerable = true;
            self.invulnerability_timer = Instant::now();
        }

        result
    }

    pub fn heal(&mut self, amount: f32) -> f32 {
        if !self.state.is_alive() {
            return 0.0;
        }
        self.stats.health.heal(amount)
    }

    pub fn die(&mut self) {
        self.state = PlayerState::Dead {
            death_time: Instant::now(),
        };
        // Trigger death animation, sound effects, etc.
    }

    pub fn can_attack(&self) -> bool {
        if !self.state.is_alive() || self.is_attacking {
            return false;
        }

        let attack_cooldown = 1.0 / self.effective_attack_speed();
        self.last_attack_time.elapsed().as_secs_f32() >= attack_cooldown
    }

    pub fn perform_attack(&mut self) -> Option<AttackEvent> {
        if !self.can_attack() {
            return None;
        }

        self.last_attack_time = Instant::now();
        self.is_attacking = true;

        Some(AttackEvent {
            damage: self.effective_attack_damage(),
            position: (self.x, self.y),
            direction: self.direction,
            range: 50, // Attack reach
        })
    }

    // Effective stat calculations (with modifiers)
    pub fn effective_movement_speed(&self) -> f32 {
        self.stats.effective_stat(
            StatType::MovementSpeed,
            &self.effect_manager.active_modifiers(),
        )
    }

    pub fn effective_attack_damage(&self) -> f32 {
        self.stats.effective_stat(
            StatType::AttackDamage,
            &self.effect_manager.active_modifiers(),
        )
    }

    pub fn effective_attack_speed(&self) -> f32 {
        self.stats.effective_stat(
            StatType::AttackSpeed,
            &self.effect_manager.active_modifiers(),
        )
    }

    pub fn effective_defense(&self) -> f32 {
        self.stats.effective_stat(
            StatType::Defense,
            &self.effect_manager.active_modifiers(),
        )
    }

    // Apply temporary or permanent effect
    pub fn apply_effect(&mut self, effect: ModifierEffect) {
        self.effect_manager.add_effect(effect);
    }

    // Update must handle effect expiration and stats
    pub fn update(&mut self, keyboard_state: &sdl2::keyboard::KeyboardState) {
        // Update effects (remove expired ones)
        // Note: We'll need to track delta time for this
        self.effect_manager.update();

        // Update damage flash
        if let Some(flash_time) = self.damage_flash_timer {
            if flash_time.elapsed().as_millis() > 200 {
                self.damage_flash_timer = None;
            }
        }

        // Update invulnerability timer (existing pattern)
        if self.is_invulnerable {
            let elapsed = self.invulnerability_timer.elapsed().as_secs_f32();
            if elapsed >= self.invulnerability_duration {
                self.is_invulnerable = false;
            }
        }

        // Use effective movement speed from stats
        let effective_speed = self.effective_movement_speed() as i32;

        // Movement logic (adapted from current implementation)
        self.velocity_x = 0;
        self.velocity_y = 0;

        // Only allow movement if not attacking (current behavior)
        if !self.is_attacking {
            if keyboard_state.is_scancode_pressed(Scancode::W) {
                self.velocity_y -= effective_speed;
            }
            if keyboard_state.is_scancode_pressed(Scancode::S) {
                self.velocity_y += effective_speed;
            }
            if keyboard_state.is_scancode_pressed(Scancode::A) {
                self.velocity_x -= effective_speed;
            }
            if keyboard_state.is_scancode_pressed(Scancode::D) {
                self.velocity_x += effective_speed;
            }
        }

        // Diagonal normalization
        if self.velocity_x != 0 && self.velocity_y != 0 {
            let diagonal_factor = 0.707;
            self.velocity_x = (self.velocity_x as f32 * diagonal_factor).round() as i32;
            self.velocity_y = (self.velocity_y as f32 * diagonal_factor).round() as i32;
        }

        self.x += self.velocity_x;
        self.y += self.velocity_y;

        // Update direction
        if self.velocity_x != 0 || self.velocity_y != 0 {
            self.direction = Direction::from_velocity(self.velocity_x, self.velocity_y);
        }

        // Check attack animation
        if self.is_attacking && self.animation_controller.is_animation_finished() {
            self.is_attacking = false;
        }

        // Update animation state
        let new_state = if self.is_attacking {
            "attack".to_string()
        } else {
            determine_animation_state(self.velocity_x, self.velocity_y, effective_speed)
        };

        self.animation_controller.set_state(new_state);
        self.animation_controller.update();
    }
}
```

## Rust Learning Opportunities

### 1. **Enums and Pattern Matching**
- `StatType`, `DamageType`, and `PlayerState` demonstrate Rust's powerful enum system
- Pattern matching in `can_take_damage()` and state transitions

### 2. **Option and Result Types**
- `Option<Instant>` for damage flash timer
- `Option<AttackEvent>` for conditional attack execution
- Proper error handling with `Result<DamageResult, CombatError>`

### 3. **Ownership and Borrowing**
- Effect manager owns effects but provides references
- Careful lifetime management with borrowed stat modifiers
- Avoiding unnecessary clones while maintaining safety

### 4. **Type Safety**
- Separate types for health, damage, and healing prevent mixing concerns
- NewType pattern for stats prevents accidental misuse
- Zero-cost abstractions through inline functions

### 5. **Traits**
- `Damageable` trait for any entity that can take damage (Player, Enemy)
- `Healable` trait for entities that can be healed
- Custom `Debug` implementations for better debugging

## Example Usage

```rust
// In main game loop
let mut player = Player::new(100, 100, 32, 32);

// Player takes damage from enemy
let damage = DamageEvent {
    amount: 25.0,
    damage_type: DamageType::Physical,
    source: DamageSource::Enemy(slime_id),
};
let result = player.take_damage(damage);
println!("Player took {} damage! HP: {}/{}",
    result.damage_dealt,
    player.stats.health.current,
    player.stats.health.max
);

// Heal player
let healing = player.heal(15.0);
println!("Player healed for {} HP!", healing);

// Apply temporary speed boost
player.apply_effect(ModifierEffect {
    stat_type: StatType::MovementSpeed,
    modifier: StatModifier::Percentage(0.5), // +50% speed
    duration: Some(Duration::from_secs(5)),
    source: "Speed Potion".to_string(),
});

// Apply permanent attack damage increase
player.apply_effect(ModifierEffect {
    stat_type: StatType::AttackDamage,
    modifier: StatModifier::Flat(5.0), // +5 damage
    duration: None, // Permanent
    source: "Damage Upgrade".to_string(),
});
```

## Current State vs. Plan

### Already Implemented ✅
- **Collision System**: Full trait-based collision with `Collidable` trait, AABB detection
- **Basic Health**: Simple `i32` health/max_health on Player and Slime
- **Damage/Death**: `take_damage()` methods, death detection (health <= 0)
- **Invulnerability**: Timer-based invulnerability after taking damage
- **Attack System**: Basic attack triggering, animation states
- **Visual Scale**: Game uses 2x sprite scaling (recently changed from 3x)

### To Be Implemented 🔨
- **Stats System**: Replace simple health with `Stats` struct
- **Effect Manager**: Buffs/debuffs with duration tracking
- **Defense/Resistance**: Damage mitigation calculations
- **Attack Speed**: Rate-limiting for attacks (cooldown)
- **Stat Modifiers**: Temporary and permanent stat changes
- **Death State**: Proper state machine for death handling
- **Visual Feedback**: Damage numbers, health bars, stat indicators

## Implementation Phases

### Phase 1: Core Stats (Foundation) ✅ COMPLETED
- [x] Create `stats.rs` module
- [x] Implement `Health` struct with damage/healing (replace `i32` health)
- [x] Implement `Stats` struct with base stats
- [x] Add `StatType` enum (MovementSpeed, AttackDamage, etc.)
- [x] Implement `DamageResult` struct for detailed damage feedback
- [x] Integrate basic stats into `Player` (replace existing fields)
- [x] All unit tests passing (9/9 tests pass)

### Phase 2: Combat System ✅ COMPLETED
- [x] Create `combat.rs` module
- [x] Implement `DamageEvent`, `DamageType`, `DamageSource` enums
- [x] Add defense calculations to damage formula
- [x] Implement `PlayerState` enum (Alive/Dead)
- [x] Add attack cooldown system using `attack_speed` stat
- [x] Update damage system to use `DamageEvent`
- [x] All unit tests passing (8/8 combat tests pass)

### Phase 3: Effect System
- [ ] Create `effects.rs` module
- [ ] Implement `ModifierEffect` and `StatModifier` enums
- [ ] Create `EffectManager` for tracking active effects
- [ ] Add effect expiration logic (duration tracking)
- [ ] Integrate modifiers into stat calculations
- [ ] Add `apply_effect()` method to Player

### Phase 4: Visual Feedback
- [ ] Damage flash effect on hit (partially done, enhance it)
- [ ] Create health bar UI overlay
- [ ] Add death animation state
- [ ] Show stat modifiers visually (buff/debuff icons)
- [ ] Add damage numbers popup system (optional)

### Phase 5: Testing and Refinement
- [ ] Write unit tests for stat calculations
- [ ] Test damage/healing edge cases (overkill, overheal)
- [ ] Test modifier stacking and expiration
- [ ] Verify integration with collision system
- [ ] Performance profiling
- [ ] Documentation and examples

## Future Enhancements

1. **Status Effects**: Poison, burning, stun, slow, etc.
2. **Stat Caps**: Max/min values for stats
3. **Stat Regeneration**: Health/mana regeneration over time
4. **Critical Hits**: Damage variance and critical strike chance
5. **Resistances**: Element-specific damage reduction
6. **Stat Persistence**: Save/load player stats
7. **Level System**: Experience and stat growth
8. **Equipment System**: Items that modify stats

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_damage() {
        let mut health = Health::new(100.0);
        let result = health.take_damage(30.0);
        assert_eq!(result.damage_dealt, 30.0);
        assert_eq!(health.current, 70.0);
        assert!(!result.is_fatal);
    }

    #[test]
    fn test_health_fatal_damage() {
        let mut health = Health::new(100.0);
        let result = health.take_damage(150.0);
        assert_eq!(result.damage_dealt, 100.0);
        assert_eq!(health.current, 0.0);
        assert!(result.is_fatal);
        assert_eq!(result.overkill, 50.0);
    }

    #[test]
    fn test_healing_caps_at_max() {
        let mut health = Health::new(100.0);
        health.take_damage(50.0);
        let healed = health.heal(100.0);
        assert_eq!(healed, 50.0); // Only healed what was missing
        assert_eq!(health.current, 100.0);
    }

    #[test]
    fn test_stat_modifiers_stack() {
        let stats = Stats::new();
        let modifiers = vec![
            ModifierEffect {
                stat_type: StatType::AttackDamage,
                modifier: StatModifier::Flat(5.0),
                duration: None,
                source: "Item 1".to_string(),
            },
            ModifierEffect {
                stat_type: StatType::AttackDamage,
                modifier: StatModifier::Percentage(0.5),
                duration: None,
                source: "Item 2".to_string(),
            },
        ];

        // Base (10) + Flat (5) = 15, then * 1.5 = 22.5
        let effective = stats.effective_stat(StatType::AttackDamage, &modifiers);
        assert_eq!(effective, 22.5);
    }
}
```

## Notes on Rust Best Practices

1. **Prefer Composition Over Inheritance**: Stats, Combat, and Effects are separate modules that compose together
2. **Use NewType Pattern**: Wrap primitives in meaningful types (e.g., `Health` instead of raw `f32`)
3. **Leverage Type System**: Use enums for state machines and discriminated unions
4. **Avoid Premature Optimization**: Start with simple calculations, profile before optimizing
5. **Document Public APIs**: All public methods should have doc comments
6. **Use Builder Pattern**: For complex stat initialization with many optional fields
7. **Error Handling**: Use `Result` for operations that can fail, `Option` for optional values
8. **Zero-Cost Abstractions**: Most stat calculations can be inlined and optimized away

## Questions to Consider

1. Should stats use `f32` or `i32`? (f32 allows for percentages and fractional values)
2. How should we handle negative stats? (Clamp to 0 or allow?)
3. Should modifiers stack additively or multiplicatively?
4. How do we want to visualize stat changes to the player?
5. Should we implement a damage number popup system?

## Migration Strategy

### Replacing Existing Code

**Current Player fields to remove**:
```rust
pub health: i32,
pub max_health: i32,
pub speed: i32,  // Will become part of stats.movement_speed
```

**Current Player methods to update**:
```rust
pub fn take_damage(&mut self, damage: i32) -> bool
// Becomes:
pub fn take_damage(&mut self, damage_event: DamageEvent) -> DamageResult

pub fn is_alive(&self) -> bool
// Becomes:
self.state.is_alive()  // Access through state enum
```

**Main.rs changes needed**:
- Update collision damage from `player.take_damage(1)` to use `DamageEvent`
- Update slime damage to use new system
- Add stat modifier demos (speed boost pickup, etc.)

### Compatibility Notes

1. **Collision system**: No changes needed - `Collidable` trait works as-is
2. **Animation system**: No changes needed - uses `String` state names
3. **Invulnerability**: Keep existing timer pattern, works perfectly
4. **Sprite scaling**: Currently 2x, account for this in hitbox calculations

## Conclusion

This system provides a solid foundation for player stats that is:
- **Extensible**: Easy to add new stats and modifiers
- **Type-safe**: Leverages Rust's type system
- **Performant**: Minimal overhead through zero-cost abstractions
- **Testable**: Clear separation of concerns enables thorough testing
- **Maintainable**: Well-documented with clear patterns
- **Compatible**: Works with existing collision and animation systems

The phased implementation approach allows you to build incrementally while learning Rust concepts along the way. The migration from the current simple system to this comprehensive one will be straightforward since we're following similar patterns (just more robust).
