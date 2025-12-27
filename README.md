# Director

> ⚠️ **Pre-release** — API may change before v1.0

**A programmatic video rendering engine in Rust.**

Director combines Taffy (CSS Flexbox), Skia (rasterization), and Rhai (scripting) to enable programmatic video generation with a clean, declarative API.

---

## Quick Start

```rhai
let movie = new_director(1920, 1080, 30);
let scene = movie.add_scene(5.0);

let root = scene.add_box(#{
    width: "100%",
    height: "100%",
    justify_content: "center",
    align_items: "center",
    bg_color: "#1a1a2e"
});

let title = root.add_text(#{
    content: "Hello, Director!",
    size: 72.0,
    color: "#ffffff",
    weight: "bold"
});

title.animate("scale", 0.8, 1.0, 1.0, "bounce_out");

movie
```

```bash
cargo run --release -- examples/basics/hello_world.rhai output.mp4
```

---

## Features

| Category | Features |
|----------|----------|
| **Layout** | Flexbox via Taffy (justify, align, padding, margin, absolute positioning) |
| **Text** | SkParagraph with rich spans, weights, colors, backgrounds, shrink-to-fit |
| **Animation** | Keyframes + easing, spring physics, per-property animation |
| **Effects** | Blur, grayscale, sepia, invert, custom SkSL shaders |
| **Compositing** | Alpha masking, blend modes (multiply, screen, overlay, etc.) |
| **Media** | Image loading, video embedding, multi-track audio |
| **Transitions** | Scene transitions (fade, slide, wipe) with ripple edit |

---

## Project Structure

```
director-engine/
├── crates/
│   ├── director-core/       # Main engine (rendering, scripting, layout)
│   ├── director-cli/        # Command-line video renderer
│   ├── director-pipeline/   # Asset pipeline utilities
│   ├── director-schema/     # Schema definitions
│   ├── lottie-core/         # Lottie animation parser
│   ├── lottie-data/         # Lottie data structures
│   └── lottie-skia/         # Lottie Skia renderer
├── examples/
│   ├── basics/              # Hello world, layout, animation, text
│   └── features/            # Effects, masking, transitions, images
├── docs/                    # Documentation
├── assets/                  # Test assets (images, fonts, audio, video)
└── .github/                 # CI/CD and issue templates
```

---

## Installation

### As a Dependency

```toml
[dependencies]
director-engine = "1.1"
rhai = "1.19"
```

### Building from Source

```bash
# Clone
git clone https://github.com/user/director-engine.git
cd director-engine

# Build (requires FFmpeg)
cargo build --release

# Run example
cargo run --release -- examples/basics/hello_world.rhai output.mp4
```

### System Dependencies

| Dependency | Ubuntu | macOS | Windows |
|------------|--------|-------|---------|
| FFmpeg | `apt install libavutil-dev libavformat-dev libavcodec-dev libswscale-dev` | `brew install ffmpeg` | [gyan.dev build](https://www.gyan.dev/ffmpeg/builds/) |
| Clang (for Skia) | `apt install clang` | Xcode | LLVM |

See [docs/BUILD_GUIDE.md](docs/BUILD_GUIDE.md) for detailed setup.

---

## Documentation

| Document | Description |
|----------|-------------|
| [DOCS_INDEX.md](DOCS_INDEX.md) | Documentation navigation index |
| [docs/user/scripting-guide.md](docs/user/scripting-guide.md) | Complete Rhai API reference |
| [docs/architecture/overview.md](docs/architecture/overview.md) | Engine internals |
| [docs/contributing/development.md](docs/contributing/development.md) | Build guide & contributing |
| [docs/architecture/roadmap.md](docs/architecture/roadmap.md) | Development milestones |
| [examples/README.md](examples/README.md) | Example scripts index |


---

## Examples

All examples are tested and serve as API reference:

```bash
# Basics
cargo run --release -- examples/basics/hello_world.rhai output.mp4
cargo run --release -- examples/basics/layout_flexbox.rhai output.mp4
cargo run --release -- examples/basics/animation.rhai output.mp4

# Features
cargo run --release -- examples/features/effects.rhai output.mp4
cargo run --release -- examples/features/masking.rhai output.mp4
cargo run --release -- examples/features/transitions.rhai output.mp4
```

---

## Embedding in Rust

```rust
use director_engine::{scripting, DefaultAssetLoader};
use rhai::Engine;
use std::sync::Arc;

fn main() -> anyhow::Result<()> {
    let mut engine = Engine::new();
    scripting::register_rhai_api(&mut engine, Arc::new(DefaultAssetLoader));

    let script = r#"
        let movie = new_director(1920, 1080, 30);
        let scene = movie.add_scene(3.0);
        scene.add_text(#{ content: "Hello", size: 72.0, color: "#FFF" });
        movie
    "#;

    let movie = engine.eval::<scripting::MovieHandle>(script)?;
    let mut director = movie.director.lock().unwrap();
    
    director_engine::systems::renderer::render_export(
        &mut director,
        "output.mp4".into(),
        None,
        None
    )?;
    
    Ok(())
}
```

---

## License

MIT
