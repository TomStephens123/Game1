# AI Prompt: Resource Manager Design

## Context

You are designing a **Resource Manager** for Game1, a 2.5D Rust game using SDL2. Currently, texture loading is **ad-hoc and manual**, with assets scattered across main.rs initialization code, no caching strategy, and no error recovery. As the game grows, this will become a significant bottleneck and source of bugs.

## Project Background

**Technology Stack:**
- Rust (learning-focused project)
- SDL2 for rendering and texture loading
- PNG images (via sdl2_image crate)
- JSON configs for animations (AnimationConfig pattern)

**Existing Patterns:**
- AnimationConfig loads JSON, creates AnimationController (see `docs/systems/animation-system.md`)
- Entities receive texture references with lifetime `<'a>` tied to TextureCreator
- Items use sprite paths defined in ItemRegistry (see `docs/systems/item-system-design.md`)
- Assets organized in `assets/` directory (see `docs/systems/assets-structure.md`)

## The Problem

**Current State in main.rs (lines 588-621):**
```rust
// Texture loading is repetitive and error-prone:
let texture_creator = canvas.texture_creator();

let character_texture = load_texture(&texture_creator, "assets/sprites/new_player/Character-Base.png")?;
let slime_texture = load_texture(&texture_creator, "assets/sprites/slime/Slime.png")?;
let punch_texture = load_texture(&texture_creator, "assets/sprites/new_player/punch_effect.png")?;
let grass_tile_texture = load_texture(&texture_creator, "assets/backgrounds/tileable/grass_tile.png")?;
let entity_texture = load_texture(&texture_creator, "assets/sprites/the_entity/entity_awaken.png")?;

// Item textures manually loaded into HashMap:
let mut item_textures = HashMap::new();
for item_def in item_registry.all_items() {
    let texture = load_texture(&texture_creator, &item_def.sprite_path)?;
    item_textures.insert(item_def.id.clone(), texture);
}
```

**Helper Function (lines 136-143):**
```rust
fn load_texture<'a>(
    texture_creator: &'a sdl2::render::TextureCreator<sdl2::video::WindowContext>,
    path: &str,
) -> Result<sdl2::render::Texture<'a>, String> {
    texture_creator
        .load_texture(path)
        .map_err(|e| format!("Failed to load {}: {}", path, e))
}
```

**Pain Points:**
1. **No caching** - Can't reload same texture (but maybe we don't need to?)
2. **Manual management** - Must manually pass textures through 7+ function parameters
3. **Error handling** - Game crashes on missing texture (no fallback)
4. **Load order** - Must load before entities are created (rigid startup sequence)
5. **No organization** - Textures stored in separate variables, no structure
6. **Item textures special-cased** - HashMap only for items, not other assets
7. **No hot-reload** - Must restart game to see asset changes (dev QoL)
8. **Lifetime complexity** - `<'a>` tied to TextureCreator, propagates everywhere

**Specific Issues:**
- main.rs:810 - load_game() needs texture references passed in (7 parameters!)
- main.rs:1116-1119 - Slime spawning duplicates texture loading logic
- main.rs:1169-1177 - Dropped item texture setup repeated 5+ times in code
- main.rs:1245-1251 - Stone drop duplicates texture loading pattern

## Your Task

Design a **Resource Manager** that:
1. **Centralizes** texture loading and access
2. **Simplifies** entity creation (no passing textures through 7 functions)
3. **Handles errors gracefully** (missing texture → fallback/placeholder)
4. **Organizes** assets by type (character sprites, items, tiles, effects)
5. **Integrates** with existing AnimationConfig pattern
6. **Supports** future features (hot-reload, asset bundles) without premature optimization

## Requirements

### Must Have
- [ ] Load all game textures at startup (or lazy load - you decide)
- [ ] Provide access to textures by ID/name (e.g., "slime", "grass_tile")
- [ ] Handle missing textures gracefully (placeholder texture or clear error)
- [ ] Integrate with existing ItemRegistry (items have sprite_path field)
- [ ] Work with SDL2 TextureCreator lifetime constraints
- [ ] Support texture access from load_game() function
- [ ] Reduce texture parameter passing (make entity spawning easier)

### Should Have
- [ ] Organize textures by category (entities, items, tiles, effects, UI)
- [ ] Load from asset manifest/config (don't hardcode paths in code)
- [ ] Clear API: `resources.get_texture("slime")` or similar
- [ ] Error recovery (log missing asset, use placeholder, continue game)
- [ ] Integration with AnimationConfig JSON files

### Nice to Have (Don't Add if Not Needed)
- [ ] Hot-reload support (detect file changes, reload textures)
- [ ] Asset bundles/packs (single file with multiple assets)
- [ ] Texture atlas generation (combine many sprites into one texture)
- [ ] Reference counting (track what textures are actively used)
- [ ] Async loading (load in background - probably overkill)

### Must NOT Have (Premature Features)
- ❌ Virtual file system (assets always on disk for now)
- ❌ Asset compression/encryption
- ❌ Streaming (loading assets on-demand during gameplay)
- ❌ Multiple texture formats beyond PNG
- ❌ Mipmap generation or advanced texture features
- ❌ Audio/font management (textures only for now)

## Design Constraints

**Rust Learning Goals:**
This design should teach:
- Lifetime management (TextureCreator outlives all textures)
- HashMap/BTreeMap for efficient lookup
- Result and Option for error handling
- Borrowing (when to use &Texture vs cloning references)
- Trait objects or enums for heterogeneous collections

**SDL2 Lifetime Constraints:**
```rust
// SDL2 requires this structure:
let texture_creator = canvas.texture_creator();  // Owns texture data
let texture = texture_creator.load_texture(path)?;  // Borrows from creator

// texture has lifetime tied to texture_creator:
// Texture<'a> where 'a is lifetime of TextureCreator
```

**Critical**: TextureCreator must outlive all Texture instances. This means ResourceManager likely needs to own the TextureCreator.

**Integration Points:**
- **AnimationConfig** - Loads JSON, creates AnimationController with textures
- **ItemRegistry** - Defines items with sprite_path field
- **Entity Spawning** - Player, Slime, DroppedItem all need textures
- **load_game()** - Must recreate entities with textures after loading save

**Asset Organization:**
From `docs/systems/assets-structure.md`, assets are organized as:
```
assets/
├── sprites/
│   ├── new_player/
│   ├── slime/
│   ├── the_entity/
│   └── items/
├── backgrounds/
│   └── tileable/
├── config/
│   └── *.json (animation configs)
└── ui/ (future)
```

## Suggested Architecture

Consider these approaches:

### Approach 1: Typed Resource Struct
```rust
pub struct Resources<'a> {
    // Entities
    pub player_texture: Texture<'a>,
    pub slime_texture: Texture<'a>,
    pub entity_texture: Texture<'a>,

    // Items (HashMap because dynamic)
    pub item_textures: HashMap<String, Texture<'a>>,

    // Tiles
    pub grass_texture: Texture<'a>,
    pub dirt_texture: Texture<'a>,

    // Effects
    pub punch_texture: Texture<'a>,
}
```
**Pros**: Type-safe, no lookup overhead
**Cons**: Rigid, must modify struct for new assets

### Approach 2: HashMap Registry
```rust
pub struct ResourceManager<'a> {
    textures: HashMap<String, Texture<'a>>,
    // OR categorized:
    entity_textures: HashMap<String, Texture<'a>>,
    item_textures: HashMap<String, Texture<'a>>,
    tile_textures: HashMap<String, Texture<'a>>,
}

impl<'a> ResourceManager<'a> {
    pub fn get_texture(&self, name: &str) -> Option<&Texture<'a>> {
        self.textures.get(name)
    }
}
```
**Pros**: Flexible, data-driven
**Cons**: String keys (typos at runtime), need to handle missing assets

### Approach 3: Hybrid (Strongly-typed + Dynamic)
```rust
pub struct ResourceManager<'a> {
    // Common assets (strongly typed)
    pub core: CoreAssets<'a>,

    // Dynamic collections
    pub items: HashMap<String, Texture<'a>>,
    pub custom: HashMap<String, Texture<'a>>,
}
```
**Pros**: Best of both worlds
**Cons**: More complex API

### Approach 4: Asset Manifest (Config-Driven)
```toml
# assets/manifest.toml
[entities]
player = "sprites/new_player/Character-Base.png"
slime = "sprites/slime/Slime.png"

[items]
slime_ball = "sprites/items/slime_ball.png"
stone = "sprites/items/stone.png"
```
**Pros**: No hardcoded paths, easy to add assets
**Cons**: Extra file to maintain, parsing overhead

## Specific Problems to Solve

**Problem 1: TextureCreator Ownership**
Who owns the TextureCreator?
- Current: main.rs owns it, passes to load functions
- Option A: ResourceManager owns TextureCreator (borrows from Canvas?)
- Option B: ResourceManager borrows TextureCreator (but then needs same lifetime)

**Problem 2: Entity Creation with Textures**
Current entity spawning requires texture:
```rust
let slime = Slime::new(x, y, animation_controller);
// animation_controller was built with &texture
```
How to simplify? Should ResourceManager provide AnimationControllers, not just Textures?

**Problem 3: Missing Asset Handling**
What happens if `assets/sprites/slime/Slime.png` doesn't exist?
- Option A: Panic (fail-fast, current behavior)
- Option B: Return Result, caller handles error
- Option C: Use placeholder texture (hot-pink square), log warning

**Problem 4: load_game() Integration**
Currently load_game() takes 7 parameters including texture references. With ResourceManager:
```rust
// Instead of:
load_game(&save_manager, &player_config, &slime_config,
          &character_texture, &slime_texture, &entity_texture, &item_textures)

// Could be:
load_game(&save_manager, &resources)
```
How to structure this?

**Problem 5: Item Texture Lookup**
Items use string IDs ("slime_ball", "stone"). Currently:
```rust
let item_texture = item_textures.get(&item_stack.item_id)
    .ok_or(format!("Missing texture for item {}", item_stack.item_id))?;
```
This pattern repeats 5+ times. How to centralize?

**Problem 6: Animation Config Integration**
AnimationConfig creates AnimationController from texture:
```rust
let animation_controller = player_config.create_controller(
    &character_texture,
    &["idle", "running", "attack", "damage", "death"],
)?;
```
Should ResourceManager provide pre-built AnimationControllers?

## Expected Deliverables

Provide a detailed design document including:

1. **Architecture Overview**
   - Module structure (`src/resources.rs`? `src/assets.rs`?)
   - Core struct definition (ResourceManager or equivalent)
   - Lifetime strategy (who owns what)
   - Ownership diagram (TextureCreator → Textures → Entities)

2. **API Design**
   - How to load resources at startup
   - How to access textures/assets
   - How to handle missing assets
   - Code examples showing before/after

3. **Integration Plan**
   - How AnimationConfig uses ResourceManager
   - How entity spawning uses ResourceManager
   - How load_game() simplified with ResourceManager
   - Changes required to existing entity code

4. **Asset Organization Strategy**
   - File structure (keep current assets/ layout?)
   - Naming conventions (IDs vs paths)
   - Configuration file format (if using manifest)
   - Loading order (what loads first)

5. **Error Handling**
   - What happens on missing texture?
   - What happens on corrupted image?
   - Recovery strategy for dev vs release builds
   - Placeholder texture system (if using)

6. **Migration Strategy**
   - Phase 1: Create ResourceManager, load current textures
   - Phase 2: Refactor main.rs to use ResourceManager
   - Phase 3: Simplify entity spawning code
   - Phase 4: Remove redundant texture parameters
   - Which code to migrate first (lowest risk)

7. **Rust Patterns Explained**
   - Lifetime management strategy (`<'a>` everywhere?)
   - Borrowing vs ownership (when to clone)
   - Error types (String vs custom Error enum)
   - HashMap key type (String vs &'static str vs enum)

## Success Criteria

Your design is successful if:
- ✅ Texture loading code reduced by 50%+ lines
- ✅ Entity spawning doesn't require passing 3-5 texture parameters
- ✅ load_game() function signature simplified (fewer parameters)
- ✅ Adding new asset requires changes in < 3 places
- ✅ Missing texture doesn't crash game (or fails clearly in dev mode)
- ✅ No performance regression (texture access is O(1) or cached)
- ✅ Code is more maintainable (obvious where textures come from)
- ✅ Future hot-reload is architecturally feasible

## Anti-Patterns to Avoid

- ❌ Don't create an over-generic asset system (no supporting 20 file formats)
- ❌ Don't use `Rc<RefCell<Texture>>` unless absolutely necessary
- ❌ Don't load every possible asset at startup if lazy loading works
- ❌ Don't abstract away SDL2 Texture type (creates unnecessary wrapper layer)
- ❌ Don't use string IDs if typed enums would work better
- ❌ Don't implement hot-reload if it's not used (dev feature, not for v1)

## References

Study these files for context:
- `src/main.rs:588-621` - Current texture loading code
- `src/main.rs:136-143` - load_texture() helper function
- `src/main.rs:183-315` - load_game() function (texture usage)
- `src/main.rs:1116-1119` - Slime spawning (duplicate texture logic)
- `src/animation.rs` - AnimationConfig and AnimationController
- `src/item/registry.rs` - ItemRegistry with sprite_path fields
- `docs/systems/assets-structure.md` - Current asset organization
- `docs/systems/animation-system.md` - How animations use textures
- `docs/systems/item-system-design.md` - Item sprite loading

## Questions to Answer

As you design, explicitly address:
1. Should ResourceManager own TextureCreator, or borrow it?
2. Should resources be loaded all at once or lazily?
3. How to handle texture lifetimes (`<'a>` everywhere?)
4. String IDs vs enum IDs vs typed fields?
5. Should ResourceManager provide AnimationControllers or just Textures?
6. How to integrate with existing AnimationConfig JSON pattern?
7. What's the error handling strategy (panic vs Result vs placeholder)?
8. Should there be a "get_or_load()" method for dynamic loading?

## Example Use Cases

Show how your design handles:

**Use Case 1: Startup Loading**
```rust
// Current (main.rs:588-621): 30+ lines
// Your design: ???
```

**Use Case 2: Entity Spawning**
```rust
// Current: Requires textures passed in
let slime_animation_controller = slime_config.create_controller(
    &slime_texture,
    &["slime_idle", "jump", "slime_damage", "slime_death"],
)?;
let slime = Slime::new(x, y, slime_animation_controller);

// Your design: ???
```

**Use Case 3: Dropped Item**
```rust
// Current: 8 lines of boilerplate repeated 5 times
let mut item_animation_controller = animation::AnimationController::new();
let item_frames = vec![sprite::Frame::new(0, 0, 32, 32, 300)];
let item_texture = item_textures.get(&item.item_id)
    .ok_or(format!("Missing texture for item {}", item.item_id))?;
let item_sprite_sheet = SpriteSheet::new(item_texture, item_frames);
item_animation_controller.add_animation("item_idle".to_string(), item_sprite_sheet);
item_animation_controller.set_state("item_idle".to_string());
item.set_animation_controller(item_animation_controller);

// Your design: ???
```

## Final Note

Remember: This is about **reducing friction**, not building a AAA game engine. Your design should:
- **Simplify** common operations (spawning entities, loading assets)
- **Centralize** asset management (one place to find/add textures)
- **Handle** errors gracefully (dev experience matters)
- **Integrate** with existing patterns (AnimationConfig, Saveable trait)

Prioritize **developer experience** - the code should be pleasant to work with, not clever.

Good luck! 🎨
