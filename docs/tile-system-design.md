# Tile-Based World System Design

## Overview
This document outlines the design for a tile-based world system using a dual grid approach for efficient sprite rendering. The system prioritizes extensibility for future features while maintaining simplicity in the initial implementation.

## Core Concepts

### Dual Grid System
The tile system uses two grids working in tandem:

1. **World Grid (Data Layer)**:
   - Stores tile type data at discrete positions
   - Size: 40 tiles wide × 24 tiles tall
   - Tile size: 32×32 pixels
   - Logical world size: 1280×768 pixels

2. **Render Grid (Visual Layer)**:
   - Offset by half a tile (16 pixels) in both X and Y from world grid
   - Determines sprite variants by checking 4 overlapping neighbors instead of 8
   - Reduces sprite combinations from 256 (2^8) to 16 (2^4)

**How It Works:**
```
World Grid:          Render Grid (offset):
[A][B][C]
                        [R1][R2]
[D][E][F]      →
                        [R3][R4]
[G][H][I]

R1 sees: A, B, D, E
R2 sees: B, C, E, F
R3 sees: D, E, G, H
R4 sees: E, F, H, I
```

Each render position checks 4 neighbors, making sprite selection straightforward with fewer variants needed.

## Architecture

### Component Structure

```
TileSystem
├── TileRegistry (tile type definitions)
├── WorldGrid (tile data storage)
├── RenderGrid (visual sprite selection)
└── TileInteraction (player modification)
```

### Tile Type Registry
A centralized system for defining and managing tile types:

```rust
struct TileType {
    id: TileId,
    name: String,
    sprite_sheet: Handle<Image>,
    // Future properties:
    // walkable: bool,
    // friction: f32,
    // footstep_sound: Option<Handle<AudioSource>>,
}

struct TileRegistry {
    tiles: HashMap<TileId, TileType>,
}
```

### World Grid (Data Layer)

Stores actual tile data:

```rust
struct WorldGrid {
    tiles: Vec<Vec<TileId>>,  // 2D array: [y][x]
    width: usize,             // 40
    height: usize,            // 24
}

impl WorldGrid {
    fn get_tile(&self, x: i32, y: i32) -> Option<TileId>
    fn set_tile(&mut self, x: i32, y: i32, tile: TileId)
    fn get_neighbors_4(&self, x: i32, y: i32) -> [Option<TileId>; 4]
}
```

### Render Grid (Visual Layer)

Handles sprite selection and rendering:

```rust
struct RenderGrid {
    // Bevy entities for each render tile
    render_tiles: Vec<Vec<Entity>>,
}

// Helper for sprite selection
fn calculate_sprite_index(neighbors: [bool; 4]) -> usize {
    // neighbors: [top_left, top_right, bottom_left, bottom_right]
    // Convert to binary: TL(8) + TR(4) + BL(2) + BR(1)
    let mut index = 0;
    if neighbors[0] { index += 8; }  // top_left
    if neighbors[1] { index += 4; }  // top_right
    if neighbors[2] { index += 2; }  // bottom_left
    if neighbors[3] { index += 1; }  // bottom_right
    index
}
```

## Sprite Sheet Organization

**File**: `assets/backgrounds/tileable/grass_tile.png`
- 4×4 grid of 32×32 pixel tiles (128×128 total)
- Each tile variant represents a different 4-neighbor combination
- Grass (green) appears where neighbors match, dirt (brown) where they differ

### Sprite Index Mapping
Based on binary encoding of 4 neighbors (TL, TR, BL, BR):

| Index | Binary | Neighbors | Grid Position | Description |
|-------|--------|-----------|---------------|-------------|
| 0 | 0000 | None | (3,0) | No grass (all dirt) |
| 1 | 0001 | BR | (3,1) | Bottom-right only |
| 2 | 0010 | BL | (0,0) | Bottom-left only |
| 3 | 0011 | BL,BR | (0,3) | Bottom-left and right |
| 4 | 0100 | TR | (2,0) | Top-right only |
| 5 | 0101 | TR,BR | (3,2) | Top-right and bottom-left (diagonal) |
| 6 | 0110 | TR,BL | (2,1) | Top-right and left |
| 7 | 0111 | TR,BL,BR | (2,2) | Top-right, left, bottom-right |
| 8 | 1000 | TL | (3,3) | Top-left only |
| 9 | 1001 | TL,BR | (1,0) | Top-left and bottom-right (diagonal) |
| 10 | 1010 | TL,BL | (2,3) | Left side (top and bottom) |
| 11 | 1011 | TL,BL,BR | (0,2) | Top-left, bottom-left and right |
| 12 | 1100 | TL,TR | (2,1) | Top side (left and right) |
| 13 | 1101 | TL,TR,BR | (1,3) | Top-right and left, bottom-left |
| 14 | 1110 | TL,TR,BL | (0,1) | Top and bottom-right |
| 15 | 1111 | All | (1,2) | Full grass |

### Coordinate Conversion
```rust
fn index_to_sprite_coords(index: usize) -> (usize, usize) {
    match index {
        0 => (3, 0),   1 => (3, 1),   2 => (0, 0),   3 => (0, 3),
        4 => (2, 0),   5 => (3, 2),   6 => (2, 1),   7 => (2, 2),
        8 => (3, 3),   9 => (1, 0),  10 => (2, 3),  11 => (0, 2),
       12 => (2, 1),  13 => (1, 3),  14 => (0, 1),  15 => (1, 2),
        _ => (1, 2),  // Default to full grass
    }
}
```

## Tile Replacement System

### Player Interaction Flow
1. Player selects a tile type to place (UI or hotkey)
2. Player aims at a tile position (cursor or direction)
3. On confirm input:
   - Trigger replacement animation on target tile
   - Update world grid data
   - Recalculate affected render tiles (target + neighbors)
   - Update sprite indices

### Replacement Animation
```rust
#[derive(Component)]
struct TileReplacementAnimation {
    timer: Timer,
    from_tile: TileId,
    to_tile: TileId,
    phase: AnimationPhase,
}

enum AnimationPhase {
    FadeOut,    // 0.15s - tile fades/scales down
    Switch,     // Instant - change tile type
    FadeIn,     // 0.15s - new tile fades/scales up
}
```

### Rendering Updates
When a tile is replaced, we must update:
- The changed tile's render sprite
- All 4 render tiles that consider it as a neighbor (offset positions)

```rust
fn update_tile_and_neighbors(
    world_grid: &WorldGrid,
    render_grid: &mut RenderGrid,
    x: i32,
    y: i32
) {
    // Update the 4 render positions affected by this world tile change
    let render_positions = [
        (x, y),           // bottom-left of this tile
        (x + 1, y),       // bottom-right of this tile
        (x, y + 1),       // top-left of this tile
        (x + 1, y + 1),   // top-right of this tile
    ];

    for (rx, ry) in render_positions {
        update_render_tile_sprite(world_grid, render_grid, rx, ry);
    }
}
```

## Bevy ECS Integration

### Resources
```rust
#[derive(Resource)]
struct TileRegistry { /* ... */ }

#[derive(Resource)]
struct WorldGrid { /* ... */ }
```

### Components
```rust
#[derive(Component)]
struct RenderTile {
    grid_x: i32,
    grid_y: i32,
}

#[derive(Component)]
struct TileReplacementAnimation { /* ... */ }
```

### Systems
```rust
fn setup_tile_world(
    mut commands: Commands,
    tile_registry: Res<TileRegistry>,
) {
    // Initialize world grid with default tiles (grass)
    // Spawn render grid entities with sprites
}

fn handle_tile_placement_input(
    input: Res<ButtonInput<MouseButton>>,
    // ... player state, cursor position
) {
    // Detect tile placement input
    // Trigger tile replacement
}

fn animate_tile_replacement(
    time: Res<Time>,
    mut query: Query<(&mut TileReplacementAnimation, &mut Transform, &mut TextureAtlas)>,
) {
    // Handle fade out/in animation
    // Switch sprite when needed
}

fn update_render_tiles(
    world_grid: Res<WorldGrid>,
    mut query: Query<(&RenderTile, &mut TextureAtlas)>,
    // triggered when tiles change
) {
    // Recalculate sprite indices based on neighbors
}
```

## Initial Implementation Plan

### Phase 1: Core Grid System ✅ COMPLETE
- [x] Sprite sheet added to assets
- [x] Create `TileId` enum and `TileType` struct
- [x] Implement `TileRegistry` resource
- [x] Implement `WorldGrid` with basic operations
- [x] Create render grid spawn system
- [x] Implement sprite index calculation logic
- [x] Test rendering of static grass tile world

### Phase 2: Tile Replacement ✅ COMPLETE
- [x] Add player tile selection state (1=Grass, 2=Dirt)
- [x] Implement tile placement input handling (left-click to place)
- [x] Add render tile update logic for changes
- [x] Test tile replacement with grass ↔ dirt
- [x] Fix sprite coordinate mapping and neighbor logic
- [ ] Create tile replacement animation component (future polish)
- [ ] Implement animation system (future polish)

### Phase 3: Polish & Foundation for Future
- [ ] Add multiple tile type support
- [ ] Create tile selection UI/hotkeys
- [ ] Optimize render updates (only changed areas)
- [ ] Add world bounds checking
- [ ] Document sprite sheet format for new tiles

## Extensibility Roadmap

### Near Future: Object Layer
```rust
#[derive(Component)]
struct WorldObject {
    tile_x: i32,
    tile_y: i32,
    object_type: ObjectType,
}
```
- Objects occupy world grid positions but render on top
- Separate collision/interaction from base tiles

### Medium Term: Tile Properties
```rust
struct TileType {
    // ... existing fields
    walkable: bool,
    friction: f32,
    footstep_sound: Handle<AudioSource>,
    particle_effect: Option<ParticleType>,
}
```
- Affects player movement and physics
- Audio feedback for different surfaces
- Visual effects (dust, splashes, etc.)

### Long Term: Chunk System
```rust
struct ChunkCoord { x: i32, y: i32 }
struct Chunk {
    tiles: Vec<Vec<TileId>>,  // 16×16 or similar
    objects: Vec<WorldObject>,
}
struct ChunkedWorld {
    chunks: HashMap<ChunkCoord, Chunk>,
    loaded_chunks: HashSet<ChunkCoord>,
}
```
- Dynamic loading/unloading based on player position
- Infinite world generation
- Save/load individual chunks
- Maintains dual grid logic within each chunk

### Save/Load System
```rust
struct WorldSave {
    tiles: Vec<Vec<TileId>>,
    modified_tiles: HashMap<(i32, i32), TileId>,
    objects: Vec<ObjectSaveData>,
}
```
- Serialize world state to file
- Delta saving (only changed tiles)
- Version compatibility handling

## Performance Considerations

### Current Scale (40×24)
- Total tiles: 960 world tiles, ~961 render tiles
- All tiles can be active simultaneously
- No chunking needed at this scale
- Sprite updates only on tile changes (not per frame)

### Optimization Strategies
1. **Dirty Flagging**: Only update render tiles when world changes
2. **Batched Rendering**: Use Bevy's sprite batching for same texture
3. **Future Culling**: When adding camera movement, cull off-screen tiles
4. **Sparse Updates**: Only recalculate affected neighbors on changes

## Technical Details

### Coordinate Systems
```rust
// World space: pixels from origin
let world_pos = Vec2::new(x * 32.0, y * 32.0);

// World grid: tile indices
let grid_pos = (x, y);  // integers 0-39, 0-23

// Render grid: offset by half tile
let render_world_pos = Vec2::new(
    (x as f32 * 32.0) + 16.0,
    (y as f32 * 32.0) + 16.0
);
```

### Neighbor Checking Logic
```rust
fn get_render_tile_neighbors(world_grid: &WorldGrid, rx: i32, ry: i32, tile_type: TileId) -> [bool; 4] {
    // Render tile at (rx, ry) checks these world tiles:
    let top_left = world_grid.get_tile(rx - 1, ry);
    let top_right = world_grid.get_tile(rx, ry);
    let bottom_left = world_grid.get_tile(rx - 1, ry - 1);
    let bottom_right = world_grid.get_tile(rx, ry - 1);

    [
        top_left == Some(tile_type),
        top_right == Some(tile_type),
        bottom_left == Some(tile_type),
        bottom_right == Some(tile_type),
    ]
}
```

## Questions & Decisions

### Resolved
- ✅ World size: 40×24 tiles (1280×768 pixels)
- ✅ Tile size: 32×32 pixels
- ✅ Initial tile types: Grass and dirt
- ✅ Replacement mechanic: Direct replacement with animation
- ✅ Sprite organization: 4×4 grid for 16 variants

### To Determine
- Input method for tile selection (number keys, UI wheel, etc.)
- Camera behavior (fixed, follow player, free move?)
- Edge behavior (world wrap, walls, void?)
- Default world fill (all grass, mixed, generated?)

## References & Learning Notes

### Rust Concepts Applied
- **Enums for Tile Types**: Type-safe tile identification
- **2D Vec Storage**: `Vec<Vec<T>>` for grid data
- **HashMap for Registry**: O(1) tile type lookup
- **Bevy ECS Patterns**: Resources for global state, Components for per-entity data
- **Bitwise Operations**: Efficient sprite index calculation

### Design Patterns
- **Separation of Concerns**: Data (world grid) separate from presentation (render grid)
- **Data-Driven Design**: Tile properties in registry, not hardcoded
- **Event-Driven Updates**: Only recalculate when changes occur
- **Extensibility via Composition**: Add features through new components/systems

---

*This design prioritizes learning Rust while building a solid foundation for future game features. Each phase can be implemented, tested, and understood before moving to the next.*
