# Systems

Details on the major subsystems within `director-core`.

## Layout Engine (`systems/layout.rs`)

*   **Backend**: `taffy` crate.
*   **Process**:
    *   Converts `SceneGraph` hierarchy to `TaffyTree`.
    *   Because `Taffy` stores its own tree, we must sync the structure every frame or maintain a persistent mapping. Currently, we rebuild/sync the necessary parts.
    *   **MeasureFunc**: The layout engine registers a closure for nodes that return `true` for `needs_measure()`. This closure calls `Element::measure`.
*   **Units**: We map Rhai/JSON styles to Taffy's `Dimension`, `LengthPercentage`, etc.
    *   **Note**: Taffy v0.9.2 requires explicit `Dimension::length` / `percent` constructors.

## Renderer (`systems/renderer.rs`)

*   **Backend**: `skia-safe` (Bindings to Google Skia).
*   **Strategy**: Recursive traversal.
*   **Coordinate System**:
    *   Each node draws into its `layout_rect`.
    *   `canvas.translate(rect.x, rect.y)` is applied before `Element::render`.
    *   Therefore, `Element::render` implementations usually draw relative to `(0,0)` up to `(width, height)`, *unless* they ignore the transform (uncommon).
*   **Effects**:
    *   Effects like Blur are implemented using `SkImageFilter`.
    *   Requires `canvas.save_layer(&save_layer_rec)`.
    *   **Performance**: `save_layer` triggers an off-screen render pass. Heavy use impacts performance.

## Asset Manager (`systems/assets.rs`)

*   **Role**: Central repository for shared, heavy resources.
*   **Storage**:
    *   `images`: `Arc<Mutex<HashMap<String, Image>>>` (Stub/Concept - often managed by nodes locally or via cache).
    *   `fonts`: `FontCollection` (Skia TextLayout).
    *   `shaders`: `HashMap<String, RuntimeEffect>` for SkSL.
*   **Thread Safety**: Must be `Send + Sync` to be passed between threads (though currently, some Skia objects limit this).
*   **Loaders**: Relies on the `AssetLoader` trait to abstract file system / network access.

## Audio Mixer (`audio.rs`)

*   **Backend**: Pure Rust mixing logic + `rubato` for resampling.
*   **Flow**:
    *   `mix(samples_needed)` creates a zeroed buffer.
    *   Iterates over `AudioTrack`s.
    *   Resamples track audio if rate differs from mixer rate (default 48kHz).
    *   Mixes (adds) samples to buffer, applying volume.
    *   Clamps result to `[-1.0, 1.0]`.
*   **Video Audio**: Video nodes stream audio. This is pulled via `Element::get_audio` and mixed just like a standard track.
