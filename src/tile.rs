use sdl2::rect::Rect;
use sdl2::render::{Texture, WindowCanvas};
use std::collections::HashMap;

/// Unique identifier for tile types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TileId {
    Grass,
    Dirt,
}

/// Definition of a tile type with its properties
#[derive(Debug, Clone)]
pub struct TileType {
    #[allow(dead_code)] // Used for tile type identification
    pub id: TileId,
    #[allow(dead_code)]
    pub name: String,
    #[allow(dead_code)]
    pub tile_size: u32,
    // Future properties can be added here:
    // pub walkable: bool,
    // pub friction: f32,
    // pub footstep_sound: Option<Handle<AudioSource>>,
}

/// Registry that holds all tile type definitions
pub struct TileRegistry {
    tiles: HashMap<TileId, TileType>,
}

impl TileRegistry {
    #[allow(dead_code)] // Reserved for future tile system
    pub fn new() -> Self {
        Self {
            tiles: HashMap::new(),
        }
    }

    #[allow(dead_code)] // Reserved for future tile system
    pub fn register(&mut self, tile_type: TileType) {
        self.tiles.insert(tile_type.id, tile_type);
    }

    #[allow(dead_code)]
    pub fn get(&self, id: TileId) -> Option<&TileType> {
        self.tiles.get(&id)
    }
}

/// World grid that stores tile data
pub struct WorldGrid {
    tiles: Vec<Vec<TileId>>,
    pub width: usize,
    pub height: usize,
}

impl WorldGrid {
    pub fn new(width: usize, height: usize, default_tile: TileId) -> Self {
        let tiles = vec![vec![default_tile; width]; height];
        Self {
            tiles,
            width,
            height,
        }
    }

    pub fn get_tile(&self, x: i32, y: i32) -> Option<TileId> {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return None;
        }
        Some(self.tiles[y as usize][x as usize])
    }

    pub fn set_tile(&mut self, x: i32, y: i32, tile: TileId) -> bool {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return false;
        }
        self.tiles[y as usize][x as usize] = tile;
        true
    }

    /// Get the 4 world tiles that a render tile at (rx, ry) checks
    /// Returns [top_left, top_right, bottom_left, bottom_right]
    /// Out-of-bounds tiles default to Grass for edge blending
    pub fn get_render_neighbors(&self, rx: i32, ry: i32) -> [Option<TileId>; 4] {
        let get_tile_or_grass = |x: i32, y: i32| -> Option<TileId> {
            if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
                Some(TileId::Grass) // Out of bounds = treat as grass
            } else {
                self.get_tile(x, y)
            }
        };

        [
            get_tile_or_grass(rx - 1, ry - 1), // top_left
            get_tile_or_grass(rx, ry - 1),     // top_right
            get_tile_or_grass(rx - 1, ry),     // bottom_left
            get_tile_or_grass(rx, ry),         // bottom_right
        ]
    }
}

/// Render tile data for each position in the render grid
pub struct RenderTile {
    pub grid_x: i32,
    pub grid_y: i32,
    pub sprite_index: usize,
}

/// Render grid that manages visual tile rendering
pub struct RenderGrid {
    tiles: Vec<Vec<RenderTile>>,
    #[allow(dead_code)]
    pub width: usize,
    #[allow(dead_code)]
    pub height: usize,
}

impl RenderGrid {
    pub fn new(world_grid: &WorldGrid) -> Self {
        let width = world_grid.width + 1;
        let height = world_grid.height + 1;
        let mut tiles = Vec::new();

        for ry in 0..height {
            let mut row = Vec::new();
            for rx in 0..width {
                // Get the 4 world tiles this render tile checks
                let neighbors = world_grid.get_render_neighbors(rx as i32, ry as i32);

                // Check which neighbors are grass
                let grass_neighbors = [
                    neighbors[0] == Some(TileId::Grass), // top-left
                    neighbors[1] == Some(TileId::Grass), // top-right
                    neighbors[2] == Some(TileId::Grass), // bottom-left
                    neighbors[3] == Some(TileId::Grass), // bottom-right
                ];

                let sprite_index = calculate_sprite_index(grass_neighbors);

                row.push(RenderTile {
                    grid_x: rx as i32,
                    grid_y: ry as i32,
                    sprite_index,
                });
            }
            tiles.push(row);
        }

        Self {
            tiles,
            width,
            height,
        }
    }

    #[allow(dead_code)]
    pub fn get_tile(&self, x: usize, y: usize) -> Option<&RenderTile> {
        if x >= self.width || y >= self.height {
            return None;
        }
        Some(&self.tiles[y][x])
    }

    /// Update render tiles affected by a world tile change
    /// When world tile (wx, wy) changes, we need to update render tiles that check it
    ///
    /// A render tile checks 4 world tiles:
    ///   Render(rx, ry) checks World(rx-1, ry-1), World(rx, ry-1), World(rx-1, ry), World(rx, ry)
    ///
    /// So when World(wx, wy) changes, we update these render tiles:
    ///   - Render(wx, wy)     - checks world(wx,wy) as bottom-right
    ///   - Render(wx+1, wy)   - checks world(wx,wy) as bottom-left
    ///   - Render(wx, wy+1)   - checks world(wx,wy) as top-right
    ///   - Render(wx+1, wy+1) - checks world(wx,wy) as top-left
    pub fn update_tile_and_neighbors(&mut self, world_grid: &WorldGrid, wx: i32, wy: i32) {
        let render_positions = [
            (wx, wy),           // world(wx,wy) is bottom-right neighbor
            (wx + 1, wy),       // world(wx,wy) is bottom-left neighbor
            (wx, wy + 1),       // world(wx,wy) is top-right neighbor
            (wx + 1, wy + 1),   // world(wx,wy) is top-left neighbor
        ];

        for (rx, ry) in render_positions {
            if rx >= 0 && ry >= 0 && (rx as usize) < self.width && (ry as usize) < self.height {
                // Get the 4 world tiles this render tile checks
                let neighbors = world_grid.get_render_neighbors(rx, ry);

                // Check which neighbors are grass
                let grass_neighbors = [
                    neighbors[0] == Some(TileId::Grass), // top-left
                    neighbors[1] == Some(TileId::Grass), // top-right
                    neighbors[2] == Some(TileId::Grass), // bottom-left
                    neighbors[3] == Some(TileId::Grass), // bottom-right
                ];

                // Calculate sprite index based on which neighbors are grass
                let sprite_index = calculate_sprite_index(grass_neighbors);
                self.tiles[ry as usize][rx as usize].sprite_index = sprite_index;
            }
        }
    }

    pub fn render(&self, canvas: &mut WindowCanvas, texture: &Texture) -> Result<(), String> {
        let tile_size = 32;  // World tile size
        let sprite_tile_size = 16;  // Sprite sheet tile size (64x64 / 4 = 16)

        for row in &self.tiles {
            for render_tile in row {
                // Calculate world position (offset by half tile)
                let x = (render_tile.grid_x * tile_size) - 16;
                let y = (render_tile.grid_y * tile_size) - 16;

                // Calculate sprite sheet position
                let (sx, sy) = index_to_sprite_coords(render_tile.sprite_index);

                let src_rect = Rect::new(
                    (sx * sprite_tile_size as usize) as i32,
                    (sy * sprite_tile_size as usize) as i32,
                    sprite_tile_size as u32,
                    sprite_tile_size as u32,
                );

                let dst_rect = Rect::new(x, y, tile_size as u32, tile_size as u32);

                canvas.copy(texture, Some(src_rect), Some(dst_rect))
                    .map_err(|e| format!("Tile render error: {}", e))?;
            }
        }

        Ok(())
    }
}

/// Calculate sprite index from 4 neighbors
/// neighbors: [top_left, top_right, bottom_left, bottom_right] where true = grass
/// Maps to sprite indices 0-15 based on which neighbors are grass
pub fn calculate_sprite_index(neighbors: [bool; 4]) -> usize {
    let tl = neighbors[0]; // top_left
    let tr = neighbors[1]; // top_right
    let bl = neighbors[2]; // bottom_left
    let br = neighbors[3]; // bottom_right

    match (tl, tr, bl, br) {
        (false, false, true, false) => 0,  // grass bottom left
        (false, true, false, true) => 1,   // grass top and bottom right
        (true, false, true, true) => 2,    // grass all but top right
        (false, false, true, true) => 3,   // grass bottom left and right
        (true, false, false, true) => 4,   // grass top left and bottom right
        (false, true, true, true) => 5,    // grass all but top left
        (true, true, true, true) => 6,     // grass everywhere (full grass)
        (true, true, true, false) => 7,    // grass all but bottom right
        (false, true, false, false) => 8,  // grass top right
        (true, true, false, false) => 9,   // grass top left and right
        (true, true, false, true) => 10,   // grass all but bottom left
        (true, false, true, false) => 11,  // grass top and bottom left
        (false, false, false, false) => 12, // no grass (all dirt)
        (false, false, false, true) => 13,  // grass bottom right
        (false, true, true, false) => 14,  // grass top right and bottom left
        (true, false, false, false) => 15,  // grass top left
    }
}

/// Convert sprite index to coordinates in the 4x4 sprite sheet
/// Direct mapping: sprite 0 = (0,0), sprite 1 = (1,0), sprite 4 = (0,1), etc.
/// Sprites are numbered left-to-right, top-to-bottom
pub fn index_to_sprite_coords(index: usize) -> (usize, usize) {
    match index {
        0 => (0, 0),  // grass bottom left
        1 => (1, 0),  // grass top and bottom right
        2 => (2, 0),  // grass all but top right
        3 => (3, 0),  // grass bottom left and right
        4 => (0, 1),  // grass top left and bottom right
        5 => (1, 1),  // grass all but top left
        6 => (2, 1),  // grass everywhere (default/full grass)
        7 => (3, 1),  // grass all but bottom right
        8 => (0, 2),  // grass top right
        9 => (1, 2),  // grass top left and right
        10 => (2, 2), // grass all but bottom left
        11 => (3, 2), // grass top and bottom left
        12 => (0, 3), // no grass (all dirt)
        13 => (1, 3), // grass bottom right
        14 => (2, 3), // grass top right and bottom left
        15 => (3, 3), // grass top left
        _ => (2, 1),  // Default to full grass (sprite 6)
    }
}

/// Get render tile neighbors that match the given tile type
#[allow(dead_code)]
pub fn get_render_tile_neighbors(
    world_grid: &WorldGrid,
    rx: i32,
    ry: i32,
    tile_type: TileId,
) -> [bool; 4] {
    let neighbors = world_grid.get_render_neighbors(rx, ry);
    [
        neighbors[0] == Some(tile_type),
        neighbors[1] == Some(tile_type),
        neighbors[2] == Some(tile_type),
        neighbors[3] == Some(tile_type),
    ]
}

// ==============================================================================
// Save/Load Support for WorldGrid
// ==============================================================================

impl TileId {
    /// Convert TileId to string for serialization
    pub fn to_string(&self) -> String {
        match self {
            TileId::Grass => "grass".to_string(),
            TileId::Dirt => "dirt".to_string(),
        }
    }

    /// Convert string back to TileId
    pub fn from_string(s: &str) -> Option<TileId> {
        match s {
            "grass" => Some(TileId::Grass),
            "dirt" => Some(TileId::Dirt),
            _ => None,
        }
    }
}

impl WorldGrid {
    /// Convert the grid to a save-friendly format (2D array of strings)
    pub fn to_save_data(&self) -> Vec<Vec<String>> {
        self.tiles
            .iter()
            .map(|row| row.iter().map(|tile| tile.to_string()).collect())
            .collect()
    }

    /// Create a WorldGrid from saved data
    pub fn from_save_data(width: usize, height: usize, tiles: Vec<Vec<String>>) -> Option<Self> {
        // Validate dimensions
        if tiles.len() != height {
            return None;
        }

        let mut grid_tiles = Vec::new();

        for row in tiles {
            if row.len() != width {
                return None;
            }

            let tile_row: Option<Vec<TileId>> = row
                .iter()
                .map(|s| TileId::from_string(s))
                .collect();

            grid_tiles.push(tile_row?);
        }

        Some(WorldGrid {
            tiles: grid_tiles,
            width,
            height,
        })
    }
}
