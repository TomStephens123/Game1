# Item System Design

## Overview

The Item System is the foundation of Game1's item/inventory architecture. It defines what items exist, their properties, and how they're referenced throughout the game. This system is designed for:

- **Centralized definitions** - Single source of truth for all item data
- **Extensibility** - Easy to add new items without modifying core code
- **Type safety** - Rust's type system prevents invalid item references
- **Performance** - Fast lookups and minimal memory overhead

**Status**: 🏗️ **PLANNED** - December 2024

## Core Concepts

### Item vs ItemStack vs DroppedItem

It's crucial to understand the distinction between these three concepts:

1. **ItemDefinition** - The "blueprint" for an item (stored in ItemRegistry)
   - Defines what the item IS (name, sprite, properties)
   - One definition per item type
   - Loaded once at game start
   - Example: The definition for "slime_ball" item

2. **ItemStack** - An instance of an item with quantity
   - References an ItemDefinition by ID
   - Has a quantity (1-64 typically)
   - Exists in inventories
   - Example: "5 slime balls in slot 3 of player inventory"

3. **DroppedItem** - A physical entity in the world
   - References an ItemDefinition by ID
   - Has a position, animation, collision
   - Can be picked up
   - Example: "Slime ball entity at (100, 200) in the world"

**Relationship**:
```
ItemDefinition (blueprint)
    ↓ referenced by
ItemStack (inventory) ←→ DroppedItem (world entity)
                  (can convert between)
```

## Architecture

### Module Structure

```
src/item/
├── mod.rs              # Public API, re-exports
├── definition.rs       # ItemDefinition struct
├── registry.rs         # ItemRegistry (central storage)
├── stack.rs            # ItemStack (instance with quantity)
└── properties.rs       # ItemProperties enum
```

### Component 1: Item Definition

**File**: `src/item/definition.rs`

```rust
use serde::{Serialize, Deserialize};

/// The blueprint for an item type
///
/// This defines the static properties of an item that are shared
/// across all instances. Think of it as the "class" and ItemStack
/// as the "instance".
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemDefinition {
    /// Unique identifier (used for lookups and saves)
    pub id: String,

    /// Display name shown in UI
    pub name: String,

    /// Description shown in tooltips
    pub description: String,

    /// Path to item sprite (16x16 recommended)
    pub sprite_path: String,

    /// Maximum stack size (1 = non-stackable, 64 = typical)
    pub max_stack_size: u32,

    /// Item-specific properties and behaviors
    pub properties: ItemProperties,
}

impl ItemDefinition {
    /// Creates a new item definition
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        sprite_path: impl Into<String>,
        max_stack_size: u32,
        properties: ItemProperties,
    ) -> Self {
        ItemDefinition {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            sprite_path: sprite_path.into(),
            max_stack_size,
            properties,
        }
    }

    /// Returns true if this item can stack with another
    pub fn can_stack_with(&self, other: &ItemDefinition) -> bool {
        // Items can only stack if they're the same type and stackable
        self.id == other.id && self.max_stack_size > 1
    }
}
```

**Design Notes**:
- **Serializable**: ItemDefinitions can be saved (for modding/custom items)
- **Cloneable**: Cheap to clone (all String data, no large assets)
- **String IDs**: Human-readable and debuggable (not integers)

### Component 2: Item Properties

**File**: `src/item/properties.rs`

```rust
use serde::{Serialize, Deserialize};
use crate::stats::ModifierEffect;

/// Different categories of items with type-specific data
///
/// This enum enables different item types to have different behaviors
/// while sharing the core ItemDefinition structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ItemProperties {
    /// Basic material (no special properties)
    Material,

    /// Consumable item (use from hotbar)
    Consumable {
        effect: ConsumableEffect,
        use_time: f32,  // Seconds to consume
    },

    /// Equipment item (worn for stat bonuses)
    Equipment {
        slot: EquipmentSlot,
        modifiers: Vec<ModifierEffect>,
    },

    /// Tool item (interact with blocks)
    Tool {
        tool_type: ToolType,
        durability: u32,
        mining_speed: f32,
    },

    /// Block item (can be placed in world)
    Block {
        block_id: String,  // ID of block type to place
    },
}

/// Effects for consumable items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsumableEffect {
    /// Restore health
    Heal(f32),

    /// Restore stamina (future)
    RestoreStamina(f32),

    /// Apply temporary stat buff
    Buff {
        modifier: ModifierEffect,
        duration: f32,  // Seconds
    },

    /// Custom effect (for special items)
    Custom(String),  // Effect ID to look up
}

/// Equipment slots for items
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EquipmentSlot {
    Head,
    Chest,
    Legs,
    Feet,
    MainHand,   // Weapon
    OffHand,    // Shield
    Accessory1,
    Accessory2,
}

/// Tool types for mining/interaction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolType {
    Pickaxe,
    Axe,
    Shovel,
    Hoe,
    Sword,  // Weapon tool
}
```

**Extensibility Pattern**:
When you need a new item type:
1. Add variant to `ItemProperties` enum
2. Define type-specific data inline
3. Pattern match in item use logic
4. No changes to ItemDefinition or ItemRegistry!

### Component 3: Item Registry

**File**: `src/item/registry.rs`

```rust
use std::collections::HashMap;
use super::definition::ItemDefinition;
use super::properties::*;

/// Central registry of all item definitions
///
/// This is the single source of truth for what items exist in the game.
/// All item references (in inventories, drops, saves) use IDs that
/// look up definitions in this registry.
pub struct ItemRegistry {
    items: HashMap<String, ItemDefinition>,
}

impl ItemRegistry {
    /// Creates a new empty registry
    pub fn new() -> Self {
        ItemRegistry {
            items: HashMap::new(),
        }
    }

    /// Creates a registry with all game items pre-registered
    ///
    /// This is called once at game startup to populate the registry
    /// with all built-in items.
    pub fn create_default() -> Self {
        let mut registry = Self::new();

        // Register all base game items
        registry.register_base_items();

        registry
    }

    /// Registers a new item definition
    ///
    /// Returns error if an item with this ID already exists.
    pub fn register(&mut self, item: ItemDefinition) -> Result<(), String> {
        if self.items.contains_key(&item.id) {
            return Err(format!("Item '{}' already registered", item.id));
        }

        self.items.insert(item.id.clone(), item);
        Ok(())
    }

    /// Gets an item definition by ID
    ///
    /// Returns None if no item with this ID exists.
    pub fn get(&self, id: &str) -> Option<&ItemDefinition> {
        self.items.get(id)
    }

    /// Gets a mutable reference to an item definition
    ///
    /// Useful for modding/runtime item modification.
    #[allow(dead_code)]
    pub fn get_mut(&mut self, id: &str) -> Option<&mut ItemDefinition> {
        self.items.get_mut(id)
    }

    /// Returns true if an item with this ID exists
    pub fn exists(&self, id: &str) -> bool {
        self.items.contains_key(id)
    }

    /// Returns all registered item IDs
    #[allow(dead_code)]
    pub fn all_ids(&self) -> Vec<&String> {
        self.items.keys().collect()
    }

    /// Returns all item definitions
    #[allow(dead_code)]
    pub fn all_items(&self) -> Vec<&ItemDefinition> {
        self.items.values().collect()
    }

    // ======================================================================
    // Item Registration - Base Game Items
    // ======================================================================

    /// Registers all base game items
    ///
    /// This is where all built-in items are defined. Add new items here.
    fn register_base_items(&mut self) {
        // Slime Ball (basic material, dropped by slimes)
        self.register(ItemDefinition::new(
            "slime_ball",
            "Slime Ball",
            "A bouncy ball of slime. Used in crafting.",
            "assets/items/slime_ball.png",
            64,  // Max stack size
            ItemProperties::Material,
        )).expect("Failed to register slime_ball");

        // Add more items here as they're created
        // Example: Health Potion
        // self.register(ItemDefinition::new(
        //     "health_potion",
        //     "Health Potion",
        //     "Restores 50 health when consumed.",
        //     "assets/items/health_potion.png",
        //     16,
        //     ItemProperties::Consumable {
        //         effect: ConsumableEffect::Heal(50.0),
        //         use_time: 1.0,
        //     },
        // )).expect("Failed to register health_potion");
    }
}

impl Default for ItemRegistry {
    fn default() -> Self {
        Self::create_default()
    }
}
```

**Design Decisions**:
1. **HashMap storage**: O(1) lookups by ID
2. **String keys**: Debuggable, modding-friendly (vs integers)
3. **Centralized registration**: One place to add new items
4. **Immutable access**: Get references, not ownership (prevents accidental modification)

### Component 4: Item Stack

**File**: `src/item/stack.rs`

```rust
use serde::{Serialize, Deserialize};

/// An instance of an item with quantity
///
/// This represents a specific amount of an item type. It's stored
/// in inventory slots and can be split/merged with other stacks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemStack {
    /// ID of the item definition in ItemRegistry
    pub item_id: String,

    /// How many of this item (1 to max_stack_size)
    pub quantity: u32,
}

impl ItemStack {
    /// Creates a new item stack
    pub fn new(item_id: impl Into<String>, quantity: u32) -> Self {
        ItemStack {
            item_id: item_id.into(),
            quantity,
        }
    }

    /// Returns true if this stack can merge with another
    ///
    /// Stacks can merge if they're the same item type
    pub fn can_merge_with(&self, other: &ItemStack) -> bool {
        self.item_id == other.item_id
    }

    /// Merges another stack into this one
    ///
    /// Returns how many items couldn't fit (overflow)
    ///
    /// # Example
    /// ```
    /// let mut stack1 = ItemStack::new("slime_ball", 50);
    /// let stack2 = ItemStack::new("slime_ball", 20);
    /// let overflow = stack1.merge(stack2, 64);
    /// assert_eq!(stack1.quantity, 64);
    /// assert_eq!(overflow, 6);
    /// ```
    pub fn merge(&mut self, mut other: ItemStack, max_stack_size: u32) -> u32 {
        if !self.can_merge_with(&other) {
            return other.quantity;  // Can't merge, return all as overflow
        }

        let total = self.quantity + other.quantity;

        if total <= max_stack_size {
            // All items fit in this stack
            self.quantity = total;
            0  // No overflow
        } else {
            // Stack is full, some items overflow
            self.quantity = max_stack_size;
            total - max_stack_size  // Return overflow amount
        }
    }

    /// Splits this stack into two
    ///
    /// Takes `amount` items from this stack and returns them as a new stack.
    /// Returns None if there aren't enough items to split.
    ///
    /// # Example
    /// ```
    /// let mut stack = ItemStack::new("slime_ball", 10);
    /// let split = stack.split(3).unwrap();
    /// assert_eq!(stack.quantity, 7);
    /// assert_eq!(split.quantity, 3);
    /// ```
    pub fn split(&mut self, amount: u32) -> Option<ItemStack> {
        if amount == 0 || amount >= self.quantity {
            return None;  // Can't split 0 or entire stack
        }

        self.quantity -= amount;

        Some(ItemStack {
            item_id: self.item_id.clone(),
            quantity: amount,
        })
    }

    /// Takes up to `amount` items from this stack
    ///
    /// Returns how many items were actually taken (might be less if stack is small)
    ///
    /// # Example
    /// ```
    /// let mut stack = ItemStack::new("slime_ball", 5);
    /// let taken = stack.take(10);
    /// assert_eq!(taken, 5);  // Only had 5 to take
    /// assert_eq!(stack.quantity, 0);
    /// ```
    pub fn take(&mut self, amount: u32) -> u32 {
        let taken = amount.min(self.quantity);
        self.quantity -= taken;
        taken
    }

    /// Adds items to this stack
    ///
    /// Returns how many items couldn't fit (overflow)
    pub fn add(&mut self, amount: u32, max_stack_size: u32) -> u32 {
        let total = self.quantity + amount;

        if total <= max_stack_size {
            self.quantity = total;
            0
        } else {
            self.quantity = max_stack_size;
            total - max_stack_size
        }
    }

    /// Returns true if this stack is empty
    pub fn is_empty(&self) -> bool {
        self.quantity == 0
    }
}
```

**Usage Pattern**:
```rust
// In inventory system
let mut slot: Option<ItemStack> = Some(ItemStack::new("slime_ball", 50));

// Add more items
if let Some(stack) = &mut slot {
    let overflow = stack.add(20, 64);  // Try to add 20 more
    if overflow > 0 {
        // Find another slot for overflow...
    }
}

// Take items
if let Some(stack) = &mut slot {
    let taken = stack.take(10);
    println!("Took {} items", taken);

    if stack.is_empty() {
        slot = None;  // Clear slot
    }
}
```

### Save System Integration

Items are saved by ID and quantity only (definitions are not saved):

```rust
use crate::save::{Saveable, SaveData, SaveError};

impl Saveable for ItemStack {
    fn to_save_data(&self) -> Result<SaveData, SaveError> {
        #[derive(Serialize)]
        struct ItemStackData {
            item_id: String,
            quantity: u32,
        }

        let data = ItemStackData {
            item_id: self.item_id.clone(),
            quantity: self.quantity,
        };

        Ok(SaveData {
            data_type: "item_stack".to_string(),
            json_data: serde_json::to_string(&data)?,
        })
    }

    fn from_save_data(data: &SaveData) -> Result<Self, SaveError> {
        #[derive(Deserialize)]
        struct ItemStackData {
            item_id: String,
            quantity: u32,
        }

        if data.data_type != "item_stack" {
            return Err(SaveError::CorruptedData(format!(
                "Expected item_stack, got {}",
                data.data_type
            )));
        }

        let stack_data: ItemStackData = serde_json::from_str(&data.json_data)?;

        // Note: We don't validate if item_id exists in registry here
        // That's done by the inventory system on load
        Ok(ItemStack {
            item_id: stack_data.item_id,
            quantity: stack_data.quantity,
        })
    }
}
```

**Important**: ItemRegistry is NOT saved. It's rebuilt on every game start by calling `ItemRegistry::create_default()`. This ensures:
- All players have the same items (no modded items in vanilla saves)
- Item definitions can be updated between versions
- Save files are smaller (don't duplicate item data)

### Validation & Error Handling

When loading items from saves:

```rust
impl Inventory {
    /// Validates and loads items from save data
    ///
    /// Invalid items (missing from registry) are removed with warning
    pub fn load_from_save(
        &mut self,
        saved_slots: Vec<Option<ItemStack>>,
        item_registry: &ItemRegistry,
    ) {
        for (i, slot) in saved_slots.into_iter().enumerate() {
            if let Some(stack) = slot {
                // Validate item exists in registry
                if item_registry.exists(&stack.item_id) {
                    self.slots[i] = Some(stack);
                } else {
                    // Item was removed from game, log warning
                    eprintln!(
                        "Warning: Removed unknown item '{}' from inventory slot {}",
                        stack.item_id, i
                    );
                }
            }
        }
    }
}
```

## Usage Examples

### Example 1: Registering a New Item

```rust
// In ItemRegistry::register_base_items()

// Health Potion (consumable)
self.register(ItemDefinition::new(
    "health_potion",
    "Health Potion",
    "Restores 50 health.",
    "assets/items/health_potion.png",
    16,  // Stack up to 16
    ItemProperties::Consumable {
        effect: ConsumableEffect::Heal(50.0),
        use_time: 1.0,
    },
)).expect("Failed to register health_potion");

// Iron Sword (equipment)
self.register(ItemDefinition::new(
    "iron_sword",
    "Iron Sword",
    "A sturdy iron blade.",
    "assets/items/iron_sword.png",
    1,  // Not stackable
    ItemProperties::Equipment {
        slot: EquipmentSlot::MainHand,
        modifiers: vec![
            ModifierEffect::new(StatType::AttackDamage, 5.0, ModifierType::Flat),
        ],
    },
)).expect("Failed to register iron_sword");
```

### Example 2: Item Lookup

```rust
// In main.rs or game state
let item_registry = ItemRegistry::create_default();

// Get item definition
if let Some(item) = item_registry.get("slime_ball") {
    println!("Item: {}", item.name);
    println!("Max stack: {}", item.max_stack_size);
}

// Check if item exists (validation)
if !item_registry.exists("invalid_item") {
    eprintln!("Item not found!");
}
```

### Example 3: Creating Item Stacks

```rust
// Create a stack of 10 slime balls
let stack = ItemStack::new("slime_ball", 10);

// Merge two stacks
let mut stack1 = ItemStack::new("slime_ball", 50);
let stack2 = ItemStack::new("slime_ball", 20);
let overflow = stack1.merge(stack2, 64);  // max_stack_size from registry

if overflow > 0 {
    println!("{} items couldn't fit", overflow);
}

// Split a stack
let mut big_stack = ItemStack::new("slime_ball", 64);
if let Some(small_stack) = big_stack.split(10) {
    println!("Split off {} items", small_stack.quantity);
}
```

## Integration with Game Systems

### With Inventory System
```rust
// Inventory queries item registry for max stack size
impl Inventory {
    pub fn add_item(
        &mut self,
        item_id: &str,
        quantity: u32,
        item_registry: &ItemRegistry,
    ) -> Result<u32, String> {
        // Get max stack size from registry
        let max_stack = item_registry.get(item_id)
            .ok_or("Item not found")?
            .max_stack_size;

        // Try to add to existing stacks first...
        // Then create new stacks...
    }
}
```

### With Dropped Items
```rust
// DroppedItem references item by ID
pub struct DroppedItem<'a> {
    pub item_id: String,  // Look up in registry for sprite
    pub quantity: u32,
    // ...
}

impl DroppedItem<'_> {
    pub fn render(&self, registry: &ItemRegistry, canvas: &mut Canvas<Window>) {
        if let Some(item_def) = registry.get(&self.item_id) {
            // Load sprite from item_def.sprite_path
            // Render item with animation
        }
    }
}
```

### With Stats System (Equipment)
```rust
// When equipping an item, apply its modifiers
if let ItemProperties::Equipment { modifiers, .. } = &item.properties {
    for modifier in modifiers {
        player.active_modifiers.push(modifier.clone());
    }
}
```

## Performance Optimization

### Item ID Interning (Future)

For even faster lookups, consider using interned item IDs:

```rust
use string_interner::{StringInterner, DefaultSymbol};

pub struct ItemRegistry {
    interner: StringInterner,
    items: HashMap<DefaultSymbol, ItemDefinition>,
}

// Benefits:
// - Symbol is Copy (no cloning strings)
// - Faster comparisons (integer vs string)
// - Lower memory (strings stored once)
```

### Lazy Sprite Loading

Don't load all item sprites at startup:

```rust
pub struct ItemDefinition {
    pub sprite_path: String,
    sprite_texture: Option<Texture>,  // Loaded on demand
}

impl ItemDefinition {
    pub fn get_or_load_sprite<'a>(
        &mut self,
        texture_creator: &'a TextureCreator<WindowContext>,
    ) -> Result<&Texture, String> {
        if self.sprite_texture.is_none() {
            // Load sprite on first access
            let texture = texture_creator.load_texture(&self.sprite_path)?;
            self.sprite_texture = Some(texture);
        }

        Ok(self.sprite_texture.as_ref().unwrap())
    }
}
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_registration() {
        let mut registry = ItemRegistry::new();
        let item = ItemDefinition::new(
            "test_item",
            "Test Item",
            "Description",
            "path.png",
            64,
            ItemProperties::Material,
        );

        registry.register(item).unwrap();
        assert!(registry.exists("test_item"));
    }

    #[test]
    fn test_duplicate_registration() {
        let mut registry = ItemRegistry::new();
        let item = ItemDefinition::new("test", "Test", "", "", 1, ItemProperties::Material);

        registry.register(item.clone()).unwrap();
        let result = registry.register(item);

        assert!(result.is_err());
    }

    #[test]
    fn test_stack_merge() {
        let mut stack1 = ItemStack::new("slime_ball", 30);
        let stack2 = ItemStack::new("slime_ball", 40);

        let overflow = stack1.merge(stack2, 64);

        assert_eq!(stack1.quantity, 64);
        assert_eq!(overflow, 6);
    }

    #[test]
    fn test_stack_split() {
        let mut stack = ItemStack::new("slime_ball", 20);
        let split = stack.split(5).unwrap();

        assert_eq!(stack.quantity, 15);
        assert_eq!(split.quantity, 5);
    }
}
```

### Integration Tests

Test item system with other systems:

```rust
#[test]
fn test_save_load_item_stack() {
    let stack = ItemStack::new("slime_ball", 10);
    let save_data = stack.to_save_data().unwrap();
    let loaded = ItemStack::from_save_data(&save_data).unwrap();

    assert_eq!(stack.item_id, loaded.item_id);
    assert_eq!(stack.quantity, loaded.quantity);
}
```

## Rust Learning Opportunities

This system teaches:

1. **HashMap Collections** (Chapter 8)
   - Using HashMap for fast lookups
   - String keys vs numeric keys tradeoffs
   - Borrowing from HashMap

2. **Enums with Data** (Chapter 6)
   - ItemProperties variants hold different data
   - Pattern matching on enum variants
   - Type-safe item behaviors

3. **Clone and Copy** (Chapter 4)
   - When to implement Clone (ItemDefinition)
   - Why Copy doesn't work for String data
   - Smart cloning strategies

4. **Error Handling** (Chapter 9)
   - Result<T, E> for registration
   - Option<T> for lookups
   - Custom error messages

5. **Module Organization** (Chapter 7)
   - Multi-file modules (mod.rs pattern)
   - Public vs private functions
   - Re-exporting types

## Summary

The Item System provides:

✅ **Centralized Definitions** - Single source of truth for all items
✅ **Type Safety** - Rust prevents invalid item references
✅ **Extensibility** - Easy to add new item types
✅ **Performance** - Fast O(1) lookups by ID
✅ **Save Compatible** - Integrates with save system
✅ **Stackable** - Smart stack merging and splitting

**Next Steps**:
1. Implement `src/item/` module structure
2. Create ItemRegistry with slime_ball definition
3. Test item lookup and stacking
4. Integrate with inventory system

---

**Related Documentation**:
- **Feature Plan**: `docs/features/item-inventory-system.md`
- **Inventory System**: `docs/systems/inventory-system-design.md`
- **Dropped Items**: `docs/systems/dropped-item-entity.md`
- **Quick Reference**: `docs/patterns/item-inventory-quick-reference.md`

---

**Last Updated**: December 2024
**Status**: 🏗️ Planned - Ready for implementation
