/// Camera system for viewport management and world-to-screen transformations
///
/// The camera defines what portion of the game world is visible on screen.
/// It tracks the player and smoothly follows them, converting world coordinates
/// to screen coordinates for rendering.
use sdl2::rect::Rect;

/// Camera configuration
#[derive(Debug, Clone)]
pub struct CameraConfig {
    /// How quickly camera follows target (0.0 = instant, 1.0 = slow)
    /// Typical values: 0.05-0.2 for smooth following
    pub smoothing: f32,

    /// Minimum viewport dimensions (reference resolution)
    pub min_viewport_width: u32,
    pub min_viewport_height: u32,

    /// World bounds (optional - camera can't see outside these)
    /// If None, camera is unbounded
    pub world_bounds: Option<Rect>,
}

impl Default for CameraConfig {
    fn default() -> Self {
        CameraConfig {
            smoothing: 0.1,  // Smooth following
            min_viewport_width: 640,
            min_viewport_height: 360,
            world_bounds: None,  // Unbounded by default
        }
    }
}

/// Camera state and viewport management
///
/// The camera tracks a target (usually the player) and smoothly follows it.
/// It provides coordinate transformations between world space and screen space.
///
/// # Coordinate Systems
///
/// - **World Coordinates**: Absolute positions in the game world (unbounded)
/// - **Screen Coordinates**: Pixel positions on the window (0 to width/height)
///
/// # Example
///
/// ```
/// let mut camera = Camera::new(320.0, 180.0, 640, 360);
/// camera.set_target(500.0, 300.0);  // Follow player at (500, 300)
/// camera.update();  // Smooth movement towards target
///
/// // Convert world position to screen position for rendering
/// let (screen_x, screen_y) = camera.world_to_screen(500, 300);
/// ```
pub struct Camera {
    /// Current camera position in world space (center of viewport)
    pub x: f32,
    pub y: f32,

    /// Target position to follow (usually player position)
    target_x: f32,
    target_y: f32,

    /// Viewport dimensions (how much of the world is visible)
    viewport_width: u32,
    viewport_height: u32,

    /// Camera configuration
    config: CameraConfig,
}

impl Camera {
    /// Create a new camera at world position (x, y)
    ///
    /// # Arguments
    ///
    /// * `x` - Initial camera X position in world space
    /// * `y` - Initial camera Y position in world space
    /// * `viewport_width` - Width of the viewport in pixels
    /// * `viewport_height` - Height of the viewport in pixels
    ///
    /// # Example
    ///
    /// ```
    /// let camera = Camera::new(320.0, 180.0, 640, 360);
    /// ```
    pub fn new(x: f32, y: f32, viewport_width: u32, viewport_height: u32) -> Self {
        Camera {
            x,
            y,
            target_x: x,
            target_y: y,
            viewport_width,
            viewport_height,
            config: CameraConfig::default(),
        }
    }

    /// Create camera with custom configuration
    ///
    /// # Arguments
    ///
    /// * `x` - Initial camera X position in world space
    /// * `y` - Initial camera Y position in world space
    /// * `viewport_width` - Width of the viewport in pixels
    /// * `viewport_height` - Height of the viewport in pixels
    /// * `config` - Custom camera configuration
    pub fn with_config(
        x: f32,
        y: f32,
        viewport_width: u32,
        viewport_height: u32,
        config: CameraConfig,
    ) -> Self {
        Camera {
            x,
            y,
            target_x: x,
            target_y: y,
            viewport_width,
            viewport_height,
            config,
        }
    }

    /// Set camera target (what it should follow)
    ///
    /// The camera will smoothly move towards this position when update() is called.
    ///
    /// # Arguments
    ///
    /// * `x` - Target X position in world space
    /// * `y` - Target Y position in world space
    pub fn set_target(&mut self, x: f32, y: f32) {
        self.target_x = x;
        self.target_y = y;
    }

    /// Update camera position (call every frame)
    ///
    /// Uses linear interpolation (lerp) for smooth camera movement.
    /// The camera moves towards the target at a rate defined by the smoothing factor.
    ///
    /// # Rust Learning: Lerp Formula
    ///
    /// ```text
    /// lerp(current, target, t) = current + (target - current) * t
    /// ```
    ///
    /// Where `t` is the smoothing factor:
    /// - t = 0.0: No movement (stays at current)
    /// - t = 1.0: Instant snap to target
    /// - t = 0.1: Smooth gradual movement (typical)
    pub fn update(&mut self) {
        // Lerp towards target (smooth camera movement)
        let smoothing = self.config.smoothing;

        self.x += (self.target_x - self.x) * smoothing;
        self.y += (self.target_y - self.y) * smoothing;

        // Apply world bounds if configured
        if let Some(bounds) = self.config.world_bounds {
            let half_viewport_w = (self.viewport_width / 2) as f32;
            let half_viewport_h = (self.viewport_height / 2) as f32;

            // Keep camera from showing outside world bounds
            let min_x = bounds.x() as f32 + half_viewport_w;
            let max_x = (bounds.x() + bounds.width() as i32) as f32 - half_viewport_w;
            let min_y = bounds.y() as f32 + half_viewport_h;
            let max_y = (bounds.y() + bounds.height() as i32) as f32 - half_viewport_h;

            self.x = self.x.clamp(min_x, max_x);
            self.y = self.y.clamp(min_y, max_y);
        }
    }

    /// Convert world coordinates to screen coordinates
    ///
    /// This is the core transformation for rendering entities.
    /// Given a position in the game world, calculate where it should appear on screen.
    ///
    /// # Formula
    ///
    /// ```text
    /// screen_x = world_x - camera.x + viewport_width / 2
    /// screen_y = world_y - camera.y + viewport_height / 2
    /// ```
    ///
    /// # Arguments
    ///
    /// * `world_x` - X position in world space
    /// * `world_y` - Y position in world space
    ///
    /// # Returns
    ///
    /// Tuple of (screen_x, screen_y) in pixel coordinates
    ///
    /// # Example
    ///
    /// ```
    /// let camera = Camera::new(1000.0, 500.0, 640, 360);
    /// let (screen_x, screen_y) = camera.world_to_screen(1000, 500);
    /// // Result: (320, 180) - camera center maps to screen center
    /// ```
    pub fn world_to_screen(&self, world_x: i32, world_y: i32) -> (i32, i32) {
        let screen_x = world_x - self.x as i32 + (self.viewport_width / 2) as i32;
        let screen_y = world_y - self.y as i32 + (self.viewport_height / 2) as i32;
        (screen_x, screen_y)
    }

    /// Convert screen coordinates to world coordinates
    ///
    /// Inverse of world_to_screen(). Used for mouse input - convert where the user
    /// clicked on screen to a position in the game world.
    ///
    /// # Arguments
    ///
    /// * `screen_x` - X position in screen space (pixels)
    /// * `screen_y` - Y position in screen space (pixels)
    ///
    /// # Returns
    ///
    /// Tuple of (world_x, world_y) in world coordinates
    ///
    /// # Example
    ///
    /// ```
    /// let camera = Camera::new(1000.0, 500.0, 640, 360);
    /// let (world_x, world_y) = camera.screen_to_world(320, 180);
    /// // Result: (1000, 500) - screen center maps to camera center
    /// ```
    pub fn screen_to_world(&self, screen_x: i32, screen_y: i32) -> (i32, i32) {
        let world_x = screen_x + self.x as i32 - (self.viewport_width / 2) as i32;
        let world_y = screen_y + self.y as i32 - (self.viewport_height / 2) as i32;
        (world_x, world_y)
    }

    /// Check if a world position is visible in the viewport
    ///
    /// Used for culling - don't render entities that are off-screen.
    /// The margin parameter allows checking slightly outside the viewport
    /// (useful for large sprites that extend beyond their anchor point).
    ///
    /// # Arguments
    ///
    /// * `world_x` - X position in world space
    /// * `world_y` - Y position in world space
    /// * `margin` - Extra pixels to consider "visible" (can be negative)
    ///
    /// # Returns
    ///
    /// `true` if the position is visible (or within margin), `false` otherwise
    ///
    /// # Example
    ///
    /// ```
    /// let camera = Camera::new(1000.0, 500.0, 640, 360);
    ///
    /// // Check if entity is visible
    /// if camera.is_visible(entity.x, entity.y, 64) {
    ///     entity.render(canvas, &camera)?;
    /// }
    /// ```
    pub fn is_visible(&self, world_x: i32, world_y: i32, margin: i32) -> bool {
        let (screen_x, screen_y) = self.world_to_screen(world_x, world_y);

        screen_x >= -margin
            && screen_x < (self.viewport_width as i32 + margin)
            && screen_y >= -margin
            && screen_y < (self.viewport_height as i32 + margin)
    }

    /// Get camera bounds in world space (what rectangle is visible)
    ///
    /// Returns a Rect representing the area of the world currently visible
    /// in the viewport. Useful for tile culling and spatial queries.
    ///
    /// # Returns
    ///
    /// Rect with (x, y) at top-left of visible world area and (width, height)
    /// matching the viewport dimensions
    ///
    /// # Example
    ///
    /// ```
    /// let camera = Camera::new(1000.0, 500.0, 640, 360);
    /// let bounds = camera.get_world_bounds();
    /// // bounds covers world area from (680, 320) to (1320, 680)
    /// ```
    pub fn get_world_bounds(&self) -> Rect {
        let half_w = (self.viewport_width / 2) as i32;
        let half_h = (self.viewport_height / 2) as i32;

        Rect::new(
            self.x as i32 - half_w,
            self.y as i32 - half_h,
            self.viewport_width,
            self.viewport_height,
        )
    }

    /// Update viewport size (when window resizes)
    ///
    /// Respects minimum viewport dimensions from config.
    ///
    /// # Arguments
    ///
    /// * `width` - New viewport width in pixels
    /// * `height` - New viewport height in pixels
    pub fn set_viewport_size(&mut self, width: u32, height: u32) {
        self.viewport_width = width.max(self.config.min_viewport_width);
        self.viewport_height = height.max(self.config.min_viewport_height);
    }

    /// Instantly move camera to position (no smoothing)
    ///
    /// Useful for:
    /// - Spawning player (camera starts at player position)
    /// - Teleporting (instant camera jump)
    /// - Loading game (restore camera position)
    ///
    /// # Arguments
    ///
    /// * `x` - Target X position in world space
    /// * `y` - Target Y position in world space
    pub fn snap_to(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
        self.target_x = x;
        self.target_y = y;
    }

    /// Get viewport dimensions
    ///
    /// # Returns
    ///
    /// Tuple of (width, height) in pixels
    pub fn viewport_size(&self) -> (u32, u32) {
        (self.viewport_width, self.viewport_height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_to_screen_center() {
        let camera = Camera::new(1000.0, 500.0, 640, 360);

        // Camera center (1000, 500) should map to screen center
        let (screen_x, screen_y) = camera.world_to_screen(1000, 500);
        assert_eq!(screen_x, 320);  // 640 / 2
        assert_eq!(screen_y, 180);  // 360 / 2
    }

    #[test]
    fn test_world_to_screen_offset() {
        let camera = Camera::new(1000.0, 500.0, 640, 360);

        // Position 100 pixels right and down from camera center
        let (screen_x, screen_y) = camera.world_to_screen(1100, 600);
        assert_eq!(screen_x, 420);  // 320 + 100
        assert_eq!(screen_y, 280);  // 180 + 100
    }

    #[test]
    fn test_screen_to_world() {
        let camera = Camera::new(1000.0, 500.0, 640, 360);

        // Screen center should map to camera center
        let (world_x, world_y) = camera.screen_to_world(320, 180);
        assert_eq!(world_x, 1000);
        assert_eq!(world_y, 500);
    }

    #[test]
    fn test_screen_to_world_offset() {
        let camera = Camera::new(1000.0, 500.0, 640, 360);

        // Screen position (0, 0) should map to top-left of viewport in world
        let (world_x, world_y) = camera.screen_to_world(0, 0);
        assert_eq!(world_x, 680);   // 1000 - 320
        assert_eq!(world_y, 320);   // 500 - 180
    }

    #[test]
    fn test_coordinate_round_trip() {
        let camera = Camera::new(1234.5, 567.8, 640, 360);

        // World -> Screen -> World should be identity (within rounding)
        let original_world = (1500, 750);
        let (screen_x, screen_y) = camera.world_to_screen(original_world.0, original_world.1);
        let (world_x, world_y) = camera.screen_to_world(screen_x, screen_y);

        assert_eq!(world_x, original_world.0);
        assert_eq!(world_y, original_world.1);
    }

    #[test]
    fn test_is_visible_center() {
        let camera = Camera::new(1000.0, 500.0, 640, 360);

        // Camera center should be visible
        assert!(camera.is_visible(1000, 500, 0));
    }

    #[test]
    fn test_is_visible_edges() {
        let camera = Camera::new(1000.0, 500.0, 640, 360);

        // Just inside viewport edges should be visible
        assert!(camera.is_visible(680, 320, 0));   // Top-left
        assert!(camera.is_visible(1319, 679, 0));  // Bottom-right

        // Just outside should not be visible
        assert!(!camera.is_visible(679, 320, 0));   // Left of viewport
        assert!(!camera.is_visible(1320, 680, 0));  // Right of viewport
    }

    #[test]
    fn test_is_visible_with_margin() {
        let camera = Camera::new(1000.0, 500.0, 640, 360);

        // Outside viewport but within margin
        assert!(camera.is_visible(600, 300, 100));  // 80 pixels left, within 100px margin
        assert!(!camera.is_visible(500, 200, 50));  // Too far outside even with margin
    }

    #[test]
    fn test_far_outside_not_visible() {
        let camera = Camera::new(1000.0, 500.0, 640, 360);

        // Very far from viewport
        assert!(!camera.is_visible(5000, 5000, 0));
        assert!(!camera.is_visible(-5000, -5000, 0));
    }

    #[test]
    fn test_camera_smoothing() {
        let mut camera = Camera::new(0.0, 0.0, 640, 360);
        camera.set_target(1000.0, 500.0);

        // Camera shouldn't instantly jump to target
        camera.update();
        assert!(camera.x > 0.0 && camera.x < 1000.0);
        assert!(camera.y > 0.0 && camera.y < 500.0);

        // After many updates, should approach target
        for _ in 0..100 {
            camera.update();
        }

        assert!((camera.x - 1000.0).abs() < 0.1);
        assert!((camera.y - 500.0).abs() < 0.1);
    }

    #[test]
    fn test_camera_snap_to() {
        let mut camera = Camera::new(0.0, 0.0, 640, 360);
        camera.set_target(1000.0, 500.0);

        // Snap should instantly move camera
        camera.snap_to(1000.0, 500.0);

        assert_eq!(camera.x, 1000.0);
        assert_eq!(camera.y, 500.0);
        assert_eq!(camera.target_x, 1000.0);
        assert_eq!(camera.target_y, 500.0);
    }

    #[test]
    fn test_get_world_bounds() {
        let camera = Camera::new(1000.0, 500.0, 640, 360);
        let bounds = camera.get_world_bounds();

        assert_eq!(bounds.x(), 680);      // 1000 - 320
        assert_eq!(bounds.y(), 320);      // 500 - 180
        assert_eq!(bounds.width(), 640);
        assert_eq!(bounds.height(), 360);
    }

    #[test]
    fn test_viewport_resize() {
        let mut camera = Camera::new(1000.0, 500.0, 640, 360);

        camera.set_viewport_size(1280, 720);

        assert_eq!(camera.viewport_width, 1280);
        assert_eq!(camera.viewport_height, 720);

        // World bounds should reflect new viewport size
        let bounds = camera.get_world_bounds();
        assert_eq!(bounds.width(), 1280);
        assert_eq!(bounds.height(), 720);
    }

    #[test]
    fn test_viewport_respects_minimum() {
        let config = CameraConfig {
            min_viewport_width: 640,
            min_viewport_height: 360,
            ..Default::default()
        };

        let mut camera = Camera::with_config(1000.0, 500.0, 640, 360, config);

        // Try to set smaller than minimum
        camera.set_viewport_size(320, 180);

        // Should clamp to minimum
        assert_eq!(camera.viewport_width, 640);
        assert_eq!(camera.viewport_height, 360);
    }

    #[test]
    fn test_world_bounds_clamping() {
        let config = CameraConfig {
            world_bounds: Some(Rect::new(0, 0, 2000, 1000)),
            ..Default::default()
        };

        let mut camera = Camera::with_config(1000.0, 500.0, 640, 360, config);

        // Try to move camera outside world bounds
        camera.set_target(3000.0, 2000.0);

        // Update multiple times to reach target
        for _ in 0..100 {
            camera.update();
        }

        // Camera should be clamped to world bounds
        // Max X = 2000 - 320 = 1680
        // Max Y = 1000 - 180 = 820
        assert!(camera.x <= 1680.0);
        assert!(camera.y <= 820.0);
    }

    #[test]
    fn test_world_bounds_minimum() {
        let config = CameraConfig {
            world_bounds: Some(Rect::new(0, 0, 2000, 1000)),
            ..Default::default()
        };

        let mut camera = Camera::with_config(1000.0, 500.0, 640, 360, config);

        // Try to move camera outside world bounds (negative)
        camera.set_target(-1000.0, -1000.0);

        // Update multiple times
        for _ in 0..100 {
            camera.update();
        }

        // Camera should be clamped to minimum bounds
        // Min X = 0 + 320 = 320
        // Min Y = 0 + 180 = 180
        assert!(camera.x >= 320.0);
        assert!(camera.y >= 180.0);
    }
}
