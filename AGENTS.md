# Agent Instructions for `director-engine`

This file outlines the architectural standards, critical constraints, and navigation tips for working with the `director-engine` codebase.

## 1. Project Structure & Organization
The project is a Cargo Workspace with the following members:
*   `crates/director-core`: The core library containing the rendering engine, scene graph, and systems.
*   `crates/director-cli`: The command-line binary wrapper (`director-engine`).
*   `crates/director-schema`: Data models and JSON contracts.
*   `crates/director-pipeline`: processing pipeline.
*   `crates/lottie-*`: Support crates for Lottie animations.

**Key Locations:**
*   **Documentation:** `docs/` (Centralized documentation).
*   **Tests:** `crates/director-core/tests/` (Integration tests & Rhai scripts).
*   **Systems:** `crates/director-core/src/systems/` (Renderer, Layout, Assets).
*   **Nodes:** `crates/director-core/src/node/` (Implementations of `BoxNode`, `TextNode`, etc.).
*   **Scene Graph:** `crates/director-core/src/scene.rs` (Node container and hierarchy).

## 2. Core Architecture & Concepts

### The Director & Timeline
*   The `Director` struct manages a `timeline` of `TimelineItem`s (Scenes).
*   It does **not** have a single root node. Instead, it renders the active scene's root based on the current time.
*   **Transitions:** Handled by overlap ("Ripple Logic"). Adding a transition shifts subsequent scenes earlier.

### Scene Graph & Nodes
*   **Storage:** Nodes are stored in a flat `Vec<Option<SceneNode>>` (Arena/Free-list) in `SceneGraph`.
*   **IDs:** `NodeId` is a `usize` index.
*   **Hierarchy:** `SceneNode` structs hold `children: Vec<NodeId>` and `parent: Option<NodeId>`.
*   **Element Trait:** All visual objects implement the `Element` trait (`measure`, `layout_style`, `render`, `update`).

### Layout (Taffy)
*   The engine uses **Taffy** (Flexbox) for layout.
*   **Decoupled Transforms:** Layout calculates the base `layout_rect`. Affine transforms (`scale`, `rotation`, `translate`) are applied *visually* during render but do not affect the Taffy layout flow.
*   **Intrinsic Sizing:** Elements like `TextNode` implement `needs_measure()` and `measure()` to inform Taffy of their content size.

### Rendering Pipeline
*   **Backend:** `skia-safe` (Skia).
*   **Pipeline:** `Director::update(time)` -> `LayoutEngine::compute_layout` -> `Director::run_post_layout` -> `render_recursive`.
*   **Threading:** `AssetManager` (specifically shader cache) is `!Send`.
    *   **Constraint:** Structs requiring `Send` (like `LottieContext`) must **not** hold the full `AssetManager`. They should only hold thread-safe components like `AssetLoader`.

## 3. Critical Implementation Details

### Text Rendering
*   **Engine:** Uses `skia_safe::textlayout::Paragraph` (SkParagraph).
*   **No Cosmic Text:** We do not use `cosmic-text`.
*   **Vertical Centering:** Enforced via `StrutStyle` with `height: 1.2` and `ForceStrutHeight`.
*   **Rich Text:** Supported via `TextSpan` and `ParagraphBuilder`.

### Animation & Physics
*   **Tweening:** Linear keyframes via `Animated<f32>`.
*   **Physics:** Spring animations are "baked" into linear keyframes at 60fps at the moment of creation.
*   **Shader Animation:** `EffectNode` supports animating shader uniforms (`Float` and `Vec`) via `TweenableVector`.

### Scripting (Rhai)
*   **Binding:** `crates/director-core/src/scripting.rs` contains all Rhai bindings.
*   **Conventions:**
    *   API methods often accept property maps (e.g., `add_box(#{ ... })`).
    *   Layout properties in maps support snake_case (e.g., `flex_direction`).
    *   `Director` is wrapped in `Arc<Mutex<>>` for handles (`MovieHandle`, `NodeHandle`).

## 4. Common Pitfalls & "Gotchas"
*   **Coordinate System:** Skia uses upper-left origin (0,0).
*   **Z-Index:** `SceneNode.z_index` controls draw order within a sibling group. It is a visual sort, distinct from Taffy's layout order.
*   **ObjectFit:** `VectorNode` currently defaults to **Stretch** (Fill), ignoring aspect ratio. `ImageNode` and `VideoNode` support `Cover`, `Contain`, `Fill`.
*   **Assets:** Large assets are not in git. Use `setup_assets.sh`. Run with `--features mock_video` if system FFmpeg is missing.

## 5. Development Workflow
*   **Verify Changes:** Always run verification tests in `crates/director-core/tests/`.
*   **Docs:** Update `docs/` when changing API surfaces.
*   **Build:** Use `cargo build -p director-cli` to build the engine.
