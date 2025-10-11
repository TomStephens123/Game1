/// Depth-sorting render system for 2.5D games
///
/// This module provides a trait-based system for rendering entities in the correct
/// visual order based on their Y-position in the game world (painter's algorithm).
///
/// # Architecture
///
/// - `DepthSortable` trait: Implemented by entities that need depth sorting
/// - `Renderable` enum: Wraps different entity types for unified rendering
/// - `render_with_depth_sorting()`: Main rendering function
///
/// # Usage Example
///
/// ```rust
/// render_with_depth_sorting(
///     &mut canvas,
///     &player,
///     &slimes,
///     &static_objects,
/// )?;
/// ```
///
/// See docs/systems/depth-sorting-render-system.md for detailed design documentation.
use crate::player::Player;
use crate::slime::Slime;
use crate::collision::StaticObject;
use crate::the_entity::TheEntity;
use crate::dropped_item::DroppedItem;
use sdl2::render::Canvas;
use sdl2::video::Window;

/// Trait for entities that participate in depth-sorted rendering.
///
/// Entities implement this trait to define:
/// 1. Their depth (Y-coordinate for sorting)
/// 2. How they render themselves
///
/// # Design Philosophy
///
/// The depth is typically the Y-coordinate of the entity's anchor point (base/bottom).
/// Entities with smaller Y-values render first (farther back in the scene).
pub trait DepthSortable {
    /// Get the Y-coordinate used for depth sorting.
    ///
    /// This should return the Y-coordinate of the entity's anchor point,
    /// typically the bottom/base of the entity where it touches the ground.
    fn get_depth_y(&self) -> i32;

    /// Render the entity to the canvas.
    ///
    /// This method is responsible for drawing the entity at its current position.
    fn render(&self, canvas: &mut Canvas<Window>) -> Result<(), String>;
}

/// Wrapper enum for different renderable entity types.
///
/// This enum allows us to treat different entity types uniformly during rendering
/// while maintaining type safety and avoiding dynamic dispatch overhead.
///
/// # Why an Enum?
///
/// - **Type Safety**: Compile-time guarantee we handle all entity types
/// - **Performance**: No vtable lookups (zero-cost abstraction)
/// - **Pattern Matching**: Exhaustive matching prevents bugs
///
/// # Rust Learning: Zero-Cost Abstractions
///
/// This enum wrapping has no runtime overhead compared to calling methods directly.
/// The compiler optimizes away the enum wrapper during compilation.
pub enum Renderable<'a> {
    Player(&'a Player<'a>),
    Slime(&'a Slime<'a>),
    StaticObject(&'a StaticObject),
    TheEntity(&'a TheEntity<'a>),
    DroppedItem(&'a DroppedItem<'a>),
}

impl<'a> Renderable<'a> {
    /// Get the depth Y-coordinate for this renderable.
    ///
    /// Delegates to the underlying entity's `get_depth_y()` implementation.
    // fn get_depth_y(&self) -> i32 {
    //     match self {
    //         Renderable::Player(p) => p.get_depth_y(),
    //         Renderable::Slime(s) => s.get_depth_y(),
    //         Renderable::StaticObject(obj) => obj.get_depth_y(),
    //         Renderable::TheEntity(e) => e.get_depth_y(),
    //     }
    // }
    /// Render this entity to the canvas.
    ///
    /// Delegates to the underlying entity's `render()` implementation.
    fn render(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        match self {
            Renderable::Player(p) => p.render(canvas),
            Renderable::Slime(s) => s.render(canvas),
            Renderable::StaticObject(obj) => obj.render(canvas),
            Renderable::TheEntity(e) => e.render(canvas),
            Renderable::DroppedItem(item) => item.render(canvas),
        }
    }
}

/// Renders all entities with depth sorting (painter's algorithm).
///
/// This function collects all entities, sorts them by Y-coordinate (depth),
/// and renders them back-to-front to create correct visual layering in 2.5D.
///
/// # Algorithm: Painter's Algorithm
///
/// 1. Collect all entities with their depth (Y-coordinate)
/// 2. Sort by depth ascending (smaller Y = farther back)
/// 3. Render in sorted order (back to front)
///
/// # Performance
///
/// - Time complexity: O(n log n) for sorting
/// - Space complexity: O(n) for renderable vector
/// - Typical overhead: ~0.1ms for 100 entities
///
/// # Parameters
///
/// - `canvas`: SDL2 canvas to render to
/// - `player`: The player entity
/// - `slimes`: Slice of slime enemies
/// - `static_objects`: Slice of static world objects
///
/// # Example
///
/// ```rust
/// render_with_depth_sorting(
///     &mut canvas,
///     &player,
///     &slimes,
///     &static_objects,
/// )?;
/// ```
pub fn render_with_depth_sorting(
    canvas: &mut Canvas<Window>,
    player: &Player,
    slimes: &[Slime],
    static_objects: &[StaticObject],
    entities: &[TheEntity],
    dropped_items: &[DroppedItem],
) -> Result<(), String> {
    // Collect all renderables with their depth
    // Rust Learning: Vec::with_capacity() pre-allocates to avoid reallocation
    let mut renderables: Vec<(i32, Renderable)> = Vec::with_capacity(
        1 + slimes.len() + static_objects.len() + entities.len() + dropped_items.len()
    );

    // Add player
    renderables.push((player.get_depth_y(), Renderable::Player(player)));

    // Add slimes
    for slime in slimes {
        renderables.push((slime.get_depth_y(), Renderable::Slime(slime)));
    }

    // Add static objects
    for obj in static_objects {
        renderables.push((obj.get_depth_y(), Renderable::StaticObject(obj)));
    }

    // Add entities
    for entity in entities {
        renderables.push((entity.get_depth_y(), Renderable::TheEntity(entity)));
    }

    // Add dropped items
    for item in dropped_items {
        renderables.push((item.get_depth_y(), Renderable::DroppedItem(item)));
    }

    // Sort by Y-coordinate (painter's algorithm)
    // Entities with smaller Y render first (farther back in scene)
    // Rust Learning: sort_by_key() is a stable sort (maintains order of equal elements)
    renderables.sort_by_key(|(y, _)| *y);

    // Render in sorted order (back to front)
    for (_, renderable) in renderables {
        renderable.render(canvas)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    // Note: Full integration tests require SDL2 context, which is complex to set up
    // in unit tests. These tests verify the depth sorting logic conceptually.
    // Real testing should be done manually in the game or with visual regression tests.

    #[test]
    fn test_depth_sorting_order() {
        // Create depth values as if from entities
        let mut depths = vec![
            (150, "slime"),     // Y=150
            (100, "player"),    // Y=100
            (120, "tree"),      // Y=120
        ];

        // Sort by depth (same algorithm as render function)
        depths.sort_by_key(|(y, _)| *y);

        // Assert correct order: player (100) < tree (120) < slime (150)
        assert_eq!(depths[0], (100, "player"));
        assert_eq!(depths[1], (120, "tree"));
        assert_eq!(depths[2], (150, "slime"));
    }

    #[test]
    fn test_equal_depth_stable_sort() {
        // Test stable sort behavior with equal depths
        let mut depths = vec![
            (100, "first"),
            (100, "second"),
            (100, "third"),
        ];

        depths.sort_by_key(|(y, _)| *y);

        // Stable sort maintains original order for equal keys
        assert_eq!(depths[0], (100, "first"));
        assert_eq!(depths[1], (100, "second"));
        assert_eq!(depths[2], (100, "third"));
    }
}
