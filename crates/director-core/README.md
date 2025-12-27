# director-core

The heart of Director Engine. Contains rendering, layout, animation, and scripting logic.

## Overview

| Component | Purpose |
|-----------|---------|
| **Scene Graph** | Arena-based node storage with `NodeId` handles |
| **Layout** | Taffy-powered Flexbox (Grid planned) |
| **Rendering** | Skia 2D rasterization |
| **Animation** | Keyframes, springs, easings |
| **Scripting** | Rhai API bindings |

## Usage

```toml
[dependencies]
director-core = "1.1"
```

```rust
use director_core::{scripting, DefaultAssetLoader};
use rhai::Engine;
use std::sync::Arc;

let mut engine = Engine::new();
scripting::register_rhai_api(&mut engine, Arc::new(DefaultAssetLoader));

let movie = engine.eval::<scripting::MovieHandle>(script)?;
```

## Module Map

| Module | Purpose |
|--------|---------|
| `director.rs` | Timeline coordinator |
| `scene.rs` | Scene graph storage |
| `scripting.rs` | Rhai bindings |
| `animation.rs` | Animation system |
| `systems/` | Renderer, Layout, Assets |
| `node/` | Node implementations (Box, Text, Image, etc.) |

## Feature Flags

| Flag | Purpose |
|------|---------|
| `mock_video` | Build without FFmpeg |
| `vulkan` | Enable Vulkan Skia backend |

---

*See [AGENTS.md](../../AGENTS.md) for detailed architecture.*
