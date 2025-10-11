/// Collision detection and response system for Game1
///
/// This module provides a trait-based collision system with AABB (Axis-Aligned Bounding Box)
/// detection. It supports both dynamic entities (players, enemies) and static world objects.
///
/// # Architecture
///
/// - `Collidable` trait: Implemented by dynamic entities that can collide with each other
/// - `CollisionLayer`: Enum to categorize entities for collision filtering
/// - AABB functions: Pure functions for rectangle intersection detection
///
/// # Rust Learning Notes
///
/// This module demonstrates:
/// - **Trait-based design**: Shared behavior across different entity types
/// - **Enums for categorization**: Type-safe collision layer system
/// - **Pure functions**: Stateless collision detection logic
use crate::render::DepthSortable;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

/// Represents different categories of collidable objects.
///
/// Used for collision filtering - e.g., player attacks should only hit enemies,
/// not other players or friendly NPCs.
///
/// Note: Currently unused but included for future collision filtering features.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CollisionLayer {
    /// Player character
    Player,
    /// Enemy entities (slimes, goblins, etc.)
    Enemy,
    /// Projectiles (arrows, fireballs, etc.)
    Projectile,
    /// Static world objects (rocks, trees, buildings)
    Static,
    /// Dropped items (can be picked up by player)
    Item,
}

/// Trait for entities that participate in collision detection.
///
/// # Design Pattern: Trait-based Polymorphism
///
/// This trait allows different entity types (Player, Slime, etc.) to share
/// collision behavior without inheritance. Each entity implements this trait
/// to define:
/// - Its collision bounds
/// - What layer it belongs to
/// - How it responds to collisions
///
/// # Example
///
/// ```rust
/// impl Collidable for Player {
///     fn get_bounds(&self) -> Rect {
///         Rect::new(self.x, self.y, self.width * 3, self.height * 3)
///     }
///
///     fn get_collision_layer(&self) -> CollisionLayer {
///         CollisionLayer::Player
///     }
/// }
/// ```
pub trait Collidable {
    /// Returns the axis-aligned bounding box for this entity.
    ///
    /// The returned `Rect` should match the entity's actual position and size
    /// as rendered on screen (accounting for scale factors).
    fn get_bounds(&self) -> Rect;

    /// Returns the collision layer this entity belongs to.
    ///
    /// Used for filtering which entities can collide with each other.
    ///
    /// Note: Currently unused but included for future collision filtering features.
    #[allow(dead_code)]
    fn get_collision_layer(&self) -> CollisionLayer;
}

/// Checks if two axis-aligned bounding boxes intersect.
///
/// This is the core AABB collision detection algorithm. Two rectangles intersect
/// if they overlap on both the X and Y axes.
///
/// # Algorithm
///
/// For two rectangles to NOT intersect, one of these must be true:
/// - rect1 is completely to the left of rect2
/// - rect1 is completely to the right of rect2
/// - rect1 is completely above rect2
/// - rect1 is completely below rect2
///
/// If none of these are true, they must be intersecting.
///
/// # Example
///
/// ```rust
/// let player_bounds = Rect::new(10, 10, 32, 32);
/// let enemy_bounds = Rect::new(20, 20, 32, 32);
///
/// if aabb_intersect(&player_bounds, &enemy_bounds) {
///     println!("Collision detected!");
/// }
/// ```
///
/// # Performance
///
/// This is an O(1) operation - just a few integer comparisons.
pub fn aabb_intersect(a: &Rect, b: &Rect) -> bool {
    // Check for intersection on both axes
    let x_overlap = a.x() < b.x() + b.width() as i32 && a.x() + a.width() as i32 > b.x();
    let y_overlap = a.y() < b.y() + b.height() as i32 && a.y() + a.height() as i32 > b.y();

    x_overlap && y_overlap
}

/// Calculates the overlap between two intersecting rectangles.
///
/// Returns a tuple `(overlap_x, overlap_y)` representing how much the rectangles
/// overlap on each axis. This is used to determine the minimum push-apart vector.
///
/// # Returns
///
/// - `overlap_x`: Positive if `a` overlaps to the right of `b`, negative if left
/// - `overlap_y`: Positive if `a` overlaps below `b`, negative if above
///
/// # Note
///
/// This function assumes the rectangles are intersecting. If they don't intersect,
/// the returned values are meaningless.
///
/// # Example
///
/// ```rust
/// let (overlap_x, overlap_y) = calculate_overlap(&rect_a, &rect_b);
///
/// // Use the smaller overlap to determine push direction
/// if overlap_x.abs() < overlap_y.abs() {
///     // Push apart on X axis
///     player.x += overlap_x;
/// } else {
///     // Push apart on Y axis
///     player.y += overlap_y;
/// }
/// ```
pub fn calculate_overlap(a: &Rect, b: &Rect) -> (i32, i32) {
    // Calculate overlap on X axis
    let a_right = a.x() + a.width() as i32;
    let b_right = b.x() + b.width() as i32;

    let overlap_x = if a.x() <= b.x() {
        // a is to the left or aligned, overlap is positive (push a left, b right)
        a_right - b.x()
    } else {
        // a is to the right, overlap is negative (push a right, b left)
        a.x() - b_right
    };

    // Calculate overlap on Y axis
    let a_bottom = a.y() + a.height() as i32;
    let b_bottom = b.y() + b.height() as i32;

    let overlap_y = if a.y() <= b.y() {
        // a is above or aligned, overlap is positive (push a up, b down)
        a_bottom - b.y()
    } else {
        // a is below, overlap is negative (push a down, b up)
        a.y() - b_bottom
    };

    (overlap_x, overlap_y)
}

/// Checks collision between a single collidable entity and a collection of other entities.
///
/// This is a generic helper function that works with any type implementing `Collidable`.
/// It returns a vector of indices where collisions occurred.
///
/// # Type Parameters
///
/// - `T`: The type of entities in the collection (must implement `Collidable`)
///
/// # Returns
///
/// A vector of indices into `entities` where collisions with `entity` were detected.
///
/// # Example
///
/// ```rust
/// let colliding_indices = check_collisions_with_collection(&player, &slimes);
/// for index in colliding_indices {
///     println!("Player collided with slime {}", index);
/// }
/// ```
pub fn check_collisions_with_collection<T: Collidable>(
    entity: &impl Collidable,
    entities: &[T],
) -> Vec<usize> {
    let entity_bounds = entity.get_bounds();
    let mut collisions = Vec::new();

    for (index, other) in entities.iter().enumerate() {
        let other_bounds = other.get_bounds();

        if aabb_intersect(&entity_bounds, &other_bounds) {
            collisions.push(index);
        }
    }

    collisions
}

/// Trait for static (non-moving) world objects that participate in collision.
///
/// This is separate from `Collidable` because static objects have different behavior:
/// - They never move during collision response
/// - They don't collide with each other
/// - Collision checks are one-way (only moving objects check against static)
///
/// # Example
///
/// ```rust
/// struct Rock {
///     x: i32,
///     y: i32,
///     width: u32,
///     height: u32,
/// }
///
/// impl StaticCollidable for Rock {
///     fn get_bounds(&self) -> Rect {
///         Rect::new(self.x, self.y, self.width, self.height)
///     }
/// }
/// ```
pub trait StaticCollidable {
    /// Returns the axis-aligned bounding box for this static object.
    fn get_bounds(&self) -> Rect;
}

/// A generic static world object (rock, tree, building, etc.).
///
/// This is a simple struct that implements `StaticCollidable` and can be
/// used for any immovable obstacle in the game world.
pub struct StaticObject {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl StaticObject {
    /// Creates a new static object at the given position with the given size.
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        StaticObject {
            x,
            y,
            width,
            height,
        }
    }
}

impl StaticCollidable for StaticObject {
    fn get_bounds(&self) -> Rect {
        Rect::new(self.x, self.y, self.width, self.height)
    }
}

/// Implementation of depth sorting for StaticObject.
///
/// Static objects use their Y-coordinate as the anchor point for depth sorting.
/// This is a simple implementation for basic static obstacles (boundary walls, etc.).
///
/// Note: For more complex static objects like trees or buildings with visual sprites,
/// you would want a more sophisticated StaticObject struct with sprite_height and
/// render methods. See docs/systems/depth-sorting-render-system.md for details.
impl DepthSortable for StaticObject {
    fn get_depth_y(&self) -> i32 {
        // For simple static objects, the anchor is at the Y position
        // More complex objects would calculate based on sprite_height
        self.y
    }

    fn render(&self, _canvas: &mut Canvas<Window>) -> Result<(), String> {
        // Simple static objects (like boundary walls) don't render visually
        // They only exist for collision detection
        // More complex static objects (trees, rocks) would have sprite rendering here

        // For now, return Ok without rendering anything
        // This prevents invisible boundary walls from appearing in the depth-sorted render
        Ok(())
    }
}

/// Checks for collisions between a moving entity and static world objects.
///
/// Returns indices of all static objects the entity is colliding with.
///
/// # Example
///
/// ```rust
/// let static_objects = vec![
///     StaticObject::new(100, 100, 64, 64),
///     StaticObject::new(300, 200, 96, 96),
/// ];
///
/// let colliding_indices = check_static_collisions(&player, &static_objects);
/// for index in colliding_indices {
///     // Resolve collision with static_objects[index]
/// }
/// ```
pub fn check_static_collisions(
    entity: &impl Collidable,
    static_objects: &[&dyn StaticCollidable],
) -> Vec<usize> {
    let entity_bounds = entity.get_bounds();
    let mut collisions = Vec::new();

    for (index, static_obj) in static_objects.iter().enumerate() {
        let obj_bounds = static_obj.get_bounds();

        if aabb_intersect(&entity_bounds, &obj_bounds) {
            collisions.push(index);
        }
    }

    collisions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aabb_intersect_overlapping() {
        let rect_a = Rect::new(0, 0, 32, 32);
        let rect_b = Rect::new(16, 16, 32, 32);

        assert!(aabb_intersect(&rect_a, &rect_b));
        assert!(aabb_intersect(&rect_b, &rect_a)); // Symmetric
    }

    #[test]
    fn test_aabb_intersect_touching_edges() {
        // Rectangles touching at edges should NOT intersect (boundary case)
        let rect_a = Rect::new(0, 0, 32, 32);
        let rect_b = Rect::new(32, 0, 32, 32); // Touching right edge

        // SDL2 Rect uses exclusive upper bounds, so touching edges don't intersect
        assert!(!aabb_intersect(&rect_a, &rect_b));
    }

    #[test]
    fn test_aabb_intersect_separated() {
        let rect_a = Rect::new(0, 0, 32, 32);
        let rect_b = Rect::new(100, 100, 32, 32);

        assert!(!aabb_intersect(&rect_a, &rect_b));
    }

    #[test]
    fn test_aabb_intersect_contained() {
        // Small rectangle completely inside larger one
        let large = Rect::new(0, 0, 100, 100);
        let small = Rect::new(25, 25, 50, 50);

        assert!(aabb_intersect(&large, &small));
        assert!(aabb_intersect(&small, &large));
    }

    #[test]
    fn test_calculate_overlap_horizontal() {
        // rect_a overlapping rect_b from the left
        let rect_a = Rect::new(0, 0, 32, 32);
        let rect_b = Rect::new(20, 0, 32, 32);

        let (overlap_x, overlap_y) = calculate_overlap(&rect_a, &rect_b);

        // rect_a extends 12 pixels into rect_b on X axis
        assert_eq!(overlap_x, 12);
        // No Y overlap (aligned)
        assert_eq!(overlap_y, 32);
    }

    #[test]
    fn test_calculate_overlap_vertical() {
        // rect_a overlapping rect_b from above
        let rect_a = Rect::new(0, 0, 32, 32);
        let rect_b = Rect::new(0, 20, 32, 32);

        let (overlap_x, overlap_y) = calculate_overlap(&rect_a, &rect_b);

        // No X overlap (aligned)
        assert_eq!(overlap_x, 32);
        // rect_a extends 12 pixels into rect_b on Y axis
        assert_eq!(overlap_y, 12);
    }

    #[test]
    fn test_calculate_overlap_diagonal() {
        // Diagonal overlap
        let rect_a = Rect::new(0, 0, 32, 32);
        let rect_b = Rect::new(16, 16, 32, 32);

        let (overlap_x, overlap_y) = calculate_overlap(&rect_a, &rect_b);

        // Both axes have 16 pixels of overlap
        assert_eq!(overlap_x, 16);
        assert_eq!(overlap_y, 16);
    }
}
