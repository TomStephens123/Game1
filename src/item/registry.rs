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
