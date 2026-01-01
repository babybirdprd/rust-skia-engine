---
trigger: always_on
---

# Agent Instructions for `director-engine`

> [!IMPORTANT]
> **Agent Protocol - MANDATORY**
>
> 1. **Read First**: Before editing any file, check if it has a `//!` doc block at the top. Read it to understand the module's responsibilities.
> 2. **Find via Map**: Use the **Codebase Map** below to locate the correct file for your task. Do not guess file locations.
> 3. **Update In-Code Docs**: When adding/removing/renaming major functions or changing a module's responsibilities, **you MUST update** the `//!` doc block in that file.
> 4. **Update Codebase Map**: When adding a new file, deleting a file, or shifting responsibilities between modules, **you MUST update** the Codebase Map tables below.

---

## Codebase Map

Use this map to locate the correct file for a specific task.

### Core Systems (`crates/director-core/src`)
| Responsibility | Primary File | Key Structs |
| :--- | :--- | :--- |
| **Orchestration** | `director.rs` | `Director`, `TimelineItem` |
| **Scene Graph** | `scene.rs` | `SceneGraph` (Arena), `SceneNode` |
| **Element Trait** | `element.rs` | `Element` trait (all nodes implement) |
| **Shared Types** | `types.rs` | `Color`, `Transform`, `NodeId` |
| **Design System** | `tokens.rs` | `DesignSystem`, spacing, safe areas |
| **Rendering** | `systems/renderer.rs` | `render_recursive`, `render_export` |
| **Layout** | `systems/layout.rs` | `LayoutEngine`, Taffy integration |
| **Assets** | `systems/assets.rs` | `AssetManager`, fonts, shaders |
| **Scripting** | `scripting/mod.rs` | Rhai engine, `register_rhai_api` |
| **Scripting Types** | `scripting/types.rs` | `MovieHandle`, `SceneHandle`, `NodeHandle` |
| **Scripting Utils** | `scripting/utils.rs` | Parsers (`parse_easing`, `parse_layout_style`) |
| **Scripting API** | `scripting/api/*.rs` | Lifecycle, nodes, animation, audio, effects, properties |
| **Animation** | `animation.rs` | `Animated`, `EasingType`, springs |
| **Audio** | `audio.rs` | `AudioMixer`, `AudioTrack` |
| **Video Encoding** | `video_wrapper.rs` | FFMPEG/video-rs wrapper |

### Node Types (`crates/director-core/src/node`)
| Node | File | Use Case |
| :--- | :--- | :--- |
| **Box** | `box_node.rs` | Container with flexbox, borders, backgrounds |
| **Text** | `text.rs` | Rich text rendering (SkParagraph) |
| **Image** | `image_node.rs` | Static image display |
| **Video** | `video_node.rs` | Video playback |
| **Lottie** | `lottie.rs` | Lottie animation embedding |
| **Vector** | `vector.rs` | SVG-like vector graphics |
| **Effect** | `effect.rs` | Visual effects/shaders |
| **Composition** | `composition.rs` | Nested scene composition |

### Schema and Pipeline (`crates/`)
| Responsibility | Primary File | Notes |
| :--- | :--- | :--- |
| **DSL Types** | `director-schema/src/lib.rs` | `NodeKind`, `StyleMap`, JSON serialization |
| **Asset Pipeline** | `director-pipeline/src/lib.rs` | `build_node_recursive`, DSL to SceneGraph |

### Lottie System (`crates/lottie-*`)
| Responsibility | Primary File | Notes |
| :--- | :--- | :--- |
| **Lottie Parsing** | `lottie-core/src/lib.rs` | JSON model, keyframe evaluation |
| **Lottie Data** | `lottie-data/src/model.rs` | Raw Lottie JSON types |
| **Lottie Rendering** | `lottie-skia/src/lib.rs` | Skia path drawing |

---

## Project Overview

**Director Engine** is a programmatic video rendering engine in Rust. It combines:
- **Taffy** — CSS Flexbox layout
- **Skia** — 2D rasterization
- **Rhai** — Scripting language
- **video-rs** — FFmpeg video encoding

---

## Workspace Structure

```
crates/
├── director-core/       # Main engine (95% of logic)
│   ├── src/
│   │   ├── director.rs      # Timeline coordinator
│   │   ├── scene.rs         # Scene graph (arena storage)
│   │   ├── scripting/       # Rhai API bindings (modular)
│   │   │   ├── mod.rs       # Entry point, register_rhai_api()
│   │   │   ├── types.rs     # Movie/Scene/Node/AudioTrack handles
│   │   │   ├── utils.rs     # Parsers (easing, layout, colors)
│   │   │   ├── theme.rs     # Design system tokens API
│   │   │   └── api/         # Domain-specific registrations
│   │   │       ├── lifecycle.rs  # Director/scene management
│   │   │       ├── nodes.rs      # add_box, add_text, add_image, etc.
│   │   │       ├── animation.rs  # animate, spring, path_animate
│   │   │       ├── audio.rs      # add_audio, FFT analysis
│   │   │       ├── effects.rs    # apply_effect, shaders
│   │   │       └── properties.rs # set_style, set_mask, set_pivot
│   │   ├── animation.rs     # Keyframe/spring animation
│   │   ├── node/            # Node implementations
│   │   └── systems/         # Renderer, Layout, Assets
│   └── tests/               # Integration tests
├── director-cli/        # CLI binary
├── director-schema/     # Schema types
├── director-pipeline/   # Asset pipeline
└── lottie-*/            # Lottie animation support
```

---

## Key Concepts

### Director & Timeline
- `Director` manages a `Vec<TimelineItem>` (scenes)
- Each scene has a root `NodeId` and time range
- Transitions create overlap between scenes

### Scene Graph
- **Arena storage**: `Vec<Option<SceneNode>>` in `SceneGraph`
- **NodeId**: `usize` index
- **Hierarchy**: `children: Vec<NodeId>`, `parent: Option<NodeId>`
- **Element trait**: All nodes implement `Element` (render, update, measure)

### Layout (Taffy)
- Flexbox layout computed every frame
- Transforms (scale, rotation) are visual-only, don't affect layout
- `needs_measure()` nodes report intrinsic size to Taffy

### Rendering Pipeline
1. `Director::update(time)` — Update animations
2. `LayoutEngine::compute_layout()` — Taffy pass
3. `Director::run_post_layout()` — Post-layout hooks
4. `render_recursive()` — Skia drawing

---

## Common Tasks

### Add a Rhai API
1. Identify the appropriate sub-module in `crates/director-core/src/scripting/api/`:
   - `lifecycle.rs` - Director/scene management
   - `nodes.rs` - Node creation functions
   - `animation.rs` - Animation functions
   - `audio.rs` - Audio functions
   - `effects.rs` - Visual effects
   - `properties.rs` - Node property setters
2. Add `engine.register_fn("name", |...| { ... })` in the appropriate module
3. If adding a new utility parser, add it to `scripting/utils.rs`
4. Update `docs/SCRIPTING.md`

### Add a Node Type
1. Create `crates/director-core/src/node/my_node.rs`
2. Implement `Element` trait
3. Add to `node/mod.rs`
4. Add Rhai binding in `scripting/api/nodes.rs`
5. **Update the Codebase Map** (Node Types table)

### Run Tests
```bash
cargo test -p director-core           # All tests
cargo test -p director-core --test examples  # Example validation
cargo test -p director-core layout    # Specific test
```

### Update Snapshots
```bash
$env:UPDATE_SNAPSHOTS="1"; cargo test -p director-core
```

---

## Constraints

### Threading
- `AssetManager` is `!Send` (shader cache)
- Use `Arc<dyn AssetLoader>` for thread-safe asset loading
- `Director` is wrapped in `Arc<Mutex<>>` for Rhai handles

### Text Rendering
- Uses `skia_safe::textlayout::Paragraph` (SkParagraph)
- NOT cosmic-text
- Text animators enabled via `add_text_animator`

### Performance
- Avoid logging in per-pixel or per-frame loops
- Use `tracing::debug!` for development-only logs
- Large assets not in git — use `assets/` folder

---

## Logging

Uses `tracing` ecosystem:
```rust
tracing::info!(width, height, "Director initialized");
tracing::warn!("Feature disabled: {}", name);
tracing::debug!(frame, elapsed_ms, "Frame rendered");
```

For tests:
```rust
let _ = tracing_subscriber::fmt()
    .with_test_writer()
    .try_init();
```

---

## Documentation

> Start here: [DOCS_INDEX.md](../../DOCS_INDEX.md) is the canonical navigation index.

| Doc | Purpose |
|-----|---------|
| `DOCS_INDEX.md` | Documentation navigation |
| `docs/user/scripting-guide.md` | Rhai API reference |
| `docs/architecture/overview.md` | Engine design |
| `docs/architecture/roadmap.md` | Development milestones |
| `docs/contributing/development.md` | Build guide & contributing |
| `docs/specs/` | Design specifications |
| `examples/` | Working Rhai scripts |
