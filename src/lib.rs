//! # Director Engine
//!
//! `director` is a frame-based 2D rendering engine.
//!
//! It provides a Scene Graph, Layout Engine (Taffy), and Renderer (Skia) to generate video content
//! programmatically. It is designed to be driven by Rhai scripts.

pub mod element;
pub mod node;
pub mod director;
pub mod animation;
pub mod render;
pub mod layout;
pub mod scripting;
pub mod shaders;
pub mod video_wrapper;
pub mod audio;
pub mod tokens;

pub use director::Director;
pub use element::Element;
// node::Node is not defined in node.rs, only specific nodes.

use anyhow::Result;

pub trait AssetLoader: Send + Sync {
    fn load_bytes(&self, path: &str) -> Result<Vec<u8>>;
    fn load_font_fallback(&self) -> Option<Vec<u8>> { None }
}

pub struct DefaultAssetLoader;
impl AssetLoader for DefaultAssetLoader {
    fn load_bytes(&self, path: &str) -> Result<Vec<u8>> {
        Ok(std::fs::read(path)?)
    }

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
