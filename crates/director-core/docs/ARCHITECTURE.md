# Architecture

This document details the internal architecture of `director-core`.

## The Director Struct

The `Director` struct (`src/director.rs`) is the entry point and state container for the engine. It holds:

*   `scene`: The `SceneGraph` containing all visual nodes.
*   `timeline`: A list of `TimelineItem`s defining which scenes play when.
*   `assets`: The `AssetManager` for loading and caching resources.
*   `audio_mixer`: The `AudioMixer` for managing audio tracks.

It does **not** directly hold the renderer or layout engine state in a way that persists unrelated to the frame loop, though it orchestrates them.

## The Frame Loop

The engine operates on a frame-by-frame basis. The `Director` provides methods to drive this loop.

### 1. Update Phase (`Director::update`)

*   **Input**: Global Time (seconds).
*   **Action**:
    1.  Determines which scenes are active based on the `timeline`.
    2.  Calculates `local_time` for each active node.
    3.  Updates animated properties (springs, keyframes) on `Transform` and custom properties.
    4.  Calls `Element::update(local_time)` on every active node.
*   **Result**: Nodes have updated state (e.g., opacity, position offsets, decoded video frames). `dirty_style` flags are set if layout needs re-calculation.

### 2. Layout Phase (`systems::layout`)

*   **Input**: `SceneGraph`, Output Dimensions (Width/Height).
*   **Action**:
    1.  Constructs or updates a `Taffy` tree matching the active Scene Graph.
    2.  Calls `compute_layout`.
    3.  Taffy callbacks invoke `Element::measure` for nodes with intrinsic size (Text, Image).
    4.  Writes calculated `Rect`s back to `SceneNode.layout_rect`.
    5.  Calls `Director::run_post_layout`, triggering `Element::post_layout` (used for logic like text auto-shrinking).

### 3. Render Phase (`systems::renderer`)

*   **Input**: `SceneGraph`, `AssetManager`, Skia `Canvas`.
*   **Action**:
    1.  Recursively traverses the graph starting from the active scene root(s).
    2.  Applies transformations (Translation, Rotation, Scale, Skew) to the Canvas matrix.
    3.  Applies Opacity.
    4.  If effects (Blur, DropShadow) are present, pushes a `save_layer` with `ImageFilter`.
    5.  Calls `Element::render`.
    6.  Restores Canvas state.

### 4. Audio Phase (`Director::mix_audio`)

*   **Input**: Time, Sample Count.
*   **Action**:
    1.  Mixes global tracks.
    2.  Traverses the graph to find nodes implementing `get_audio` (e.g., Video).
    3.  Aggregates samples into a single buffer.

## Data Flow

Data generally flows one way per frame:
`Script/Animation -> Node State -> Layout Calculation -> Render Command`

## Threading Model

*   **Single-Threaded Core**: The main loop (Update/Layout/Render) is currently single-threaded. `Taffy` styles are not `Send`, preventing easy parallelization of the layout tree construction.
*   **Async Assets**: `AssetLoader` can be async, but the core engine consumes assets synchronously (blocking if not ready, or using pre-loaded handles).
*   **Video Decoding**:
    *   **Preview**: Uses a separate thread (`ThreadedDecoder`) to pre-fetch frames.
    *   **Export**: Uses the main thread (`SyncDecoder`) to ensure frame-perfect determinism.
