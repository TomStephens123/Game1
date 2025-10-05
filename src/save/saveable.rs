//! Saveable trait for entities that can be saved/loaded
//!
//! This trait provides a generic interface for converting game objects to/from
//! save data. Any entity that needs to be saved should implement this trait.

use super::types::*;

/// Trait for entities that can be saved and loaded
///
/// # Design Pattern: Trait-based Serialization
///
/// This trait allows different entity types to define their own save/load logic
/// while keeping the save system generic and extensible.
///
/// # Example
///
/// ```ignore
/// impl Saveable for Player<'_> {
///     fn to_save_data(&self) -> Result<SaveData, SaveError> {
///         // Serialize player fields to JSON
///     }
///
///     fn from_save_data(data: &SaveData) -> Result<Self, SaveError> {
///         // Deserialize JSON back to Player
///     }
/// }
/// ```
pub trait Saveable {
    /// Convert the entity to saveable data
    fn to_save_data(&self) -> Result<SaveData, SaveError>;

    /// Create an entity from saved data
    ///
    /// Note: This creates a "partial" entity with only saved state.
    /// Resources like textures must be provided separately after loading.
    fn from_save_data(data: &SaveData) -> Result<Self, SaveError>
    where
        Self: Sized;
}
