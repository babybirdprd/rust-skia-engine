# Agent Instructions for `director-engine`

Instructions for AI agents working with this codebase.

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
│   │   ├── scripting.rs     # Rhai API bindings
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

## Critical Files

| File | Purpose |
|------|---------|
| `scripting.rs` | All Rhai API bindings |
| `director.rs` | Timeline and update loop |
| `scene.rs` | Scene graph storage |
| `systems/renderer.rs` | Skia rendering |
| `systems/layout.rs` | Taffy layout |
| `node/text.rs` | Text rendering (SkParagraph) |
| `node/box_node.rs` | Box layout/styling |

---

## Common Tasks

### Add a Rhai API
1. Edit `crates/director-core/src/scripting.rs`
2. Use `engine.register_fn("name", |...| { ... })`
3. Update `docs/SCRIPTING.md`

### Add a Node Type
1. Create `crates/director-core/src/node/my_node.rs`
2. Implement `Element` trait
3. Add to `node/mod.rs`
4. Add Rhai binding in `scripting.rs`

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
- Text animators currently DISABLED (V2 feature)

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

| Doc | Purpose |
|-----|---------|
| `docs/SCRIPTING.md` | Rhai API reference |
| `docs/ARCHITECTURE.md` | Engine design |
| `docs/ROADMAP.md` | Development milestones |
| `examples/` | Working Rhai scripts |
