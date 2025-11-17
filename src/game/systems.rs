// Systems configuration and helper systems
//
// This module contains the Systems struct which holds configuration data
// and helper systems that configure gameplay but aren't entities.

use crate::animation::AnimationConfig;
use crate::collision::StaticObject;
use std::time::Instant;

use super::DebugConfig;

// Constants from main.rs
const GAME_WIDTH: u32 = 640;
const GAME_HEIGHT: u32 = 360;

/// Systems holds configuration data and helper systems
/// This struct contains things that configure gameplay but aren't entities
pub struct Systems {
    pub player_config: AnimationConfig,
    pub slime_config: AnimationConfig,
    pub punch_config: AnimationConfig,
    pub debug_config: DebugConfig,
    pub static_objects: Vec<StaticObject>,
    pub regen_timer: Instant,
    pub regen_interval: f32,
    pub has_regen: bool,
}

impl Systems {
    /// Create systems with default configuration
    pub fn new(
        player_config: AnimationConfig,
        slime_config: AnimationConfig,
        punch_config: AnimationConfig,
    ) -> Self {
        let boundary_thickness = 10;
        let static_objects = vec![
            StaticObject::new(0, -(boundary_thickness as i32), GAME_WIDTH, boundary_thickness),
            StaticObject::new(-(boundary_thickness as i32), 0, boundary_thickness, GAME_HEIGHT),
            StaticObject::new(GAME_WIDTH as i32, 0, boundary_thickness, GAME_HEIGHT),
            StaticObject::new(0, GAME_HEIGHT as i32, GAME_WIDTH, boundary_thickness),
        ];

        Systems {
            player_config,
            slime_config,
            punch_config,
            debug_config: DebugConfig::new(),
            static_objects,
            regen_timer: Instant::now(),
            regen_interval: 5.0,
            has_regen: false,
        }
    }
}
