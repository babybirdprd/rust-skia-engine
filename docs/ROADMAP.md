# Director Engine Roadmap

This document outlines the development trajectory for `director-engine`. Our mission is to build the most robust, high-performance, and developer-friendly programmatic video generation engine in Rust.

## 📍 Current Status: v0.1.0-alpha (The "Stable Core")
We have achieved a stable architectural foundation.
- ✅ **Frame-Based Rendering**: Deterministic loop with `Director`.
- ✅ **Layout**: Taffy (Flexbox) integration.
- ✅ **Text**: Native `SkParagraph` integration (Correct layout/shaping).
- ✅ **Video**: Synchronous decoding for deterministic exports.
- ✅ **Safety**: Error propagation (Result types) instead of panics.

---

## 🛣️ Milestones

### Milestone 1: Observability & Quality Assurance (Immediate)
*Focus: Ensuring the engine is debuggable and regressions are impossible.*

- [ ] **Visual Regression Harness**: Implement the "Golden Master" test suite (compare rendered frames against blessed snapshots).
- [ ] **Structured Logging**: Replace `eprintln!` with the `tracing` ecosystem to support structured logs (JSON) for production observability.
- [ ] **CI Integration**: Automate visual regression tests in GitHub Actions (caching build artifacts).
- [ ] **Benchmark Suite**: Track render times per frame to catch performance regressions.

### Milestone 2: Typography v2 (Feature Restoration)
*Focus: Re-implementing "flashy" text features on top of the stable SkParagraph foundation.*

- [ ] **Text Shadows**: Re-implement blur/offset shadows using Skia ImageFilters on the Paragraph layer.
- [ ] **Stroke & Fill**: Enable simultaneous Stroke and Fill for text (currently mutually exclusive in the new refactor).
- [ ] **Rich Text Spans**: Expose more `TextStyle` properties (decorations, letter spacing) to Rhai.
- [ ] **Glyph Animation Strategy**: Investigate `SkParagraph::getRectsForRange` to re-enable per-character animations (Wave effects) without breaking layout.

### Milestone 3: Developer Experience (The "Playground")
*Focus: Reducing the feedback loop for script writers.*

- [ ] **WASM Compilation**: Refactor `director-core` to compile to `wasm32-unknown-unknown`.
- [ ] **Web Playground**: Build a simple React/Rust frontend where users can type Rhai scripts and see the render in a `<canvas>` instantly.
- [ ] **Hot Reloading**: Allow the CLI (`director-cli`) to watch `.rhai` files and re-render the preview window on save.
- [ ] **LSP Support**: Basic syntax highlighting or autocomplete for the specific Rhai dialect used in Director.

### Milestone 4: Performance & Hardware
*Focus: Scaling to 4K and high framerates.*

- [ ] **GPU Acceleration**: Stabilize the `vulkan` and `metal` backends for Skia to offload rasterization.
- [ ] **Hardware Encoding**: Enable NVENC/VideoToolbox in `video-rs` for faster MP4 export.
- [ ] **Parallel Rendering**: Investigate frame-parallel rendering (rendering frame N and N+1 on separate threads) for CPU-bound workloads.

### Milestone 5: Advanced Features
*Focus: Expanding creative possibilities.*

- [ ] **Audio FFT**: Expose audio frequency data to Rhai to allow "Audio Reactive" animations.
- [ ] **Complex Shapes**: Expose Skia `PathOps` (Union, Difference, Xor) to scripting.
- [ ] **3D Transforms**: Move from 2D affine transforms to full 3D matrix support (Skia M44).

---

## 🧠 Architecture Decisions Log

*   **Text Engine**: Switched from `cosmic-text` to `SkParagraph` to resolve layout rounding errors and support complex shaping.
*   **Video Decoding**: Split into `Async` (Preview) and `Sync` (Export) backends to guarantee frame-perfect exports.
*   **Asset Loading**: Decoupled IO from Rendering via the `AssetLoader` trait to support WASM/Cloud environments.

---

## 🔮 Future Vision: AI Integration

*These features require external AI model integration (SAM 3, Parakeet, TTS) and are beyond V2 scope.*

### Flagship AI Workflows

| **Capability** | **Description** |
| :--- | :--- |
| **Smart Captions** | Auto-position subtitles to avoid faces (SAM 3 + Parakeet) |
| **Object Pinning** | `node.pin_to_object("face")` — track and attach overlays |
| **Auto-Reframe** | `video.auto_reframe(aspect: "9:16", target: "main_actor")` |
| **Localization** | Translate + re-time video to match new audio duration |

### AI Feature Phases

**Phase 1: Essentials**
- [ ] Object Pinning
- [ ] Auto-Subtitles with karaoke animation
- [ ] Smart Crop / Auto-Reframe

**Phase 2: "Magician" Tools**
- [ ] Object Removal (Inpainting)
- [ ] Depth Sorting ("put text behind the person")
- [ ] Style Transfer (mask-constrained shaders)

**Phase 3: Generative Director**
- [ ] Script-to-Video pipeline (LLM → Rhai → Render)