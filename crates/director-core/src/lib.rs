//! # Director Engine
//!
//! `director-engine` is a high-performance, frame-based 2D rendering engine designed for programmatic video generation.
//!
//! It combines a Scene Graph architecture, a flexbox-based Layout Engine ([Taffy](https://crates.io/crates/taffy)),
//! and a high-quality Rasterizer ([Skia](https://skia.org/)) to render visual content.
//! It also includes an embedded scripting layer using [Rhai](https://rhai.rs/).
//!
//! ## Core Features
//!
//! *   **Frame-Based Rendering**: Designed for non-realtime video rendering where every frame matters.
//! *   **Scene Graph**: Hierarchical node structure with parent-child relationships.
//! *   **Layout**: Full implementation of Flexbox and CSS Grid layout via Taffy.
//! *   **Scripting**: Built-in bindings for Rhai to drive the engine from scripts.
//! *   **Video Encoding**: Integrated FFmpeg support for exporting to MP4.
//! *   **Rich Text**: Advanced typography support using `skia_safe::textlayout`.
//! *   **Animation**: Physics-based springs and keyframe animations.
//!
//! ## Usage
//!
//! The core entry point is the [`Director`] struct, which manages the timeline, scenes, and rendering resources.
//!
//! ```rust,no_run
//! use director_core::{Director, DefaultAssetLoader, video_wrapper::RenderMode};
//! use std::sync::Arc;
//!
//! // Initialize the Director
//! let mut director = Director::new(
//!     1920,
//!     1080,
//!     30,
//!     Arc::new(DefaultAssetLoader),
//!     RenderMode::Preview,
//!     None
//! );
//! ```

/// The Scene Graph Data Structure.
pub mod scene;

/// Defines the base `Element` trait that all visual nodes must implement.
pub mod element;

/// Contains the concrete implementations of visual nodes (Box, Text, Image, etc.).
pub mod node;

/// The core coordinator of the engine, managing the scene graph and timeline.
pub mod director;

/// Animation primitives, including `Animated<T>` and easing functions.
pub mod animation;

/// Rhai scripting API bindings.
pub mod scripting;

/// Video encoding and wrapping utilities.
pub mod video_wrapper;

/// Audio mixing and processing.
pub mod audio;

/// Design system tokens (spacing, colors, typography).
pub mod tokens;

/// Shared data structures used across the engine.
pub mod types;

pub mod errors;
/// Core systems (Asset Management, etc.).
pub mod systems;

/// Video export pipeline (encoding to MP4).
pub mod export;

pub use director::Director;
pub use element::Element;
pub use errors::RenderError;

use anyhow::Result;
use tracing::instrument;

/// A trait for abstracting file system access.
///
/// This allows the engine to be embedded in environments where direct file system access
/// might be restricted or virtualized (e.g., loading assets from a network or an archive).
pub trait AssetLoader: Send + Sync {
    /// Loads the raw bytes of an asset from the given path.
    ///
    /// # Arguments
    /// * `path` - The string path or identifier for the asset.
    ///
    /// # Returns
    /// * `Result<Vec<u8>>` - The bytes of the file or an error.
    fn load_bytes(&self, path: &str) -> Result<Vec<u8>>;

    /// Optionally loads a fallback font (e.g., Emoji) to be used when glyphs are missing.
    ///
    /// The default implementation returns `None`.
    fn load_font_fallback(&self) -> Option<Vec<u8>> {
        None
    }
}

/// The default implementation of `AssetLoader` using the standard `std::fs` filesystem.
pub struct DefaultAssetLoader;

impl AssetLoader for DefaultAssetLoader {
    /// Loads bytes directly from the local filesystem.
    #[instrument(level = "debug", skip(self), fields(path = path))]
    fn load_bytes(&self, path: &str) -> Result<Vec<u8>> {
        if let Ok(bytes) = std::fs::read(path) {
            return Ok(bytes);
        }
        // Fallback to assets/
        let alt = format!("assets/{}", path);
        std::fs::read(&alt).map_err(|e| {
            anyhow::anyhow!(
                "Asset not found: {} (checked '{}' and '{}'): {}",
                path,
                path,
                alt,
                e
            )
        })
    }

    /// Attempts to load an emoji font.
    ///
    /// It first checks the `DIRECTOR_EMOJI_FONT` environment variable.
    /// If not set, it checks for `assets/fonts/emoji.ttf` in the current directory.
    fn load_font_fallback(&self) -> Option<Vec<u8>> {
        if let Ok(path) = std::env::var("DIRECTOR_EMOJI_FONT") {
            if let Ok(bytes) = std::fs::read(path) {
                return Some(bytes);
            }
        }
        // Default fallback path
        if let Ok(bytes) = std::fs::read("assets/fonts/emoji.ttf") {
            return Some(bytes);
        }
        None
    }
}
