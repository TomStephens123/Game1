# Assets Directory Structure

This document defines the standard structure for organizing game assets in the Game1 project.

## Directory Structure

```
assets/
├── sprites/
│   ├── player/
│   │   ├── idle.png
│   │   ├── walk_01.png
│   │   ├── walk_02.png
│   │   └── jump.png
│   ├── enemies/
│   │   ├── goblin/
│   │   └── skeleton/
│   ├── items/
│   │   ├── weapons/
│   │   ├── consumables/
│   │   └── collectibles/
│   └── effects/
│       ├── particles/
│       └── ui/
├── backgrounds/
│   ├── tileable/
│   │   ├── grass.png
│   │   ├── stone.png
│   │   └── dirt.png
│   └── parallax/
│       ├── sky_layer_1.png
│       └── mountains_layer_2.png
├── audio/
│   ├── sfx/
│   │   ├── player/
│   │   ├── environment/
│   │   └── ui/
│   └── music/
│       ├── ambient/
│       └── combat/
├── fonts/
└── data/
    ├── levels/
    ├── configs/
    └── saves/
```

## Naming Conventions

### Files
- Use `snake_case` for all file names (following Rust conventions)
- Use descriptive names: `player_idle_01.png` not `p1.png`
- For animation sequences, use numbers: `walk_01.png`, `walk_02.png`, etc.

### Directories
- Use `snake_case` for directory names
- Group related assets together (e.g., all player sprites in `sprites/player/`)
- Separate by functionality (e.g., `tileable` vs `parallax` backgrounds)

## Asset Guidelines

### Sprites
- **Recommended size**: 32x32 pixels for base sprites
- **Format**: PNG with transparency support
- **Organization**: Group by entity type (player, enemies, items)

### Backgrounds
- **Tileable backgrounds**: Must seamlessly wrap horizontally and vertically
- **Recommended size**: 512x512px (chunk-aligned for future chunk system)
- **Parallax layers**: Number sequentially by depth (layer_1 = furthest back)

### Audio
- **SFX**: Short sound effects, organized by source (player actions, environment, UI)
- **Music**: Longer tracks, organized by context (ambient, combat)

## Chunk System Considerations

This structure is designed to support the planned chunk loading system:
- `backgrounds/tileable/` contains assets that can be repeated across chunks
- Power-of-2 dimensions optimize GPU texture loading
- Clear separation makes asset streaming easier to implement

## Benefits for Rust Development

1. **Bevy-friendly**: Most Rust game engines expect this structure
2. **Clear separation**: Easy to locate assets during development
3. **Scalable**: Simple to add new categories as the game grows
4. **Memory efficient**: Organized for optimal loading patterns