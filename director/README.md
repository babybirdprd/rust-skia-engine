# Director Engine

A high-performance, frame-based 2D rendering engine written in Rust. It combines a Scene Graph, Taffy for CSS-like layout, Skia for rendering, and Rhai for scripting.

## Architecture

*   **Scene Graph**: A hierarchy of `Node`s (Box, Text, Image, Video) managed by a central `Director`.
*   **Layout**: Uses `taffy` (Flexbox/Grid) to compute layout every frame.
*   **Rendering**: Uses `skia-safe` to rasterize content.
*   **Scripting**: Uses `rhai` to define the scene and animations.
*   **Video Export**: Uses `video-rs` (FFmpeg) to encode frames to MP4.

## Features

*   **Animation**: Keyframe-based animation for any property (Opacity, Size, Color, Blur).
*   **Text Layout**: Advanced text shaping and wrapping using `cosmic-text`.
*   **Effects**: Support for Blur (and extensible for others).
*   **Video**: Basic playback and rendering of video nodes.

## Setup

1.  Install Rust.
2.  Install FFmpeg (required for `video-rs`).
    *   Ubuntu: `sudo apt install libavutil-dev libavformat-dev libavcodec-dev libswscale-dev`
    *   MacOS: `brew install ffmpeg`

## Usage

Run the demo script in `src/main.rs`:

```bash
cargo run
```

### Scripting API

The engine is controlled via Rhai scripts.

```rust
let movie = new_director(1080, 1920, 30);
let scene = movie.add_scene(5.0);

let box = scene.add_box(#{
    bg_color: "#FF0000"
});

// Animations
box.animate("opacity", 0.0, 1.0, 1.0, "linear");
box.set_blur(5.0);
```

## Mock Mode (Development)

If FFmpeg is not available, you can run in mock mode to verify logic:

```bash
cargo run --no-default-features --features mock_video
```
