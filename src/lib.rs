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
pub mod video_wrapper;

pub use director::Director;
pub use element::Element;
// node::Node is not defined in node.rs, only specific nodes.
