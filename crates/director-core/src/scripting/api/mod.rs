//! # API Module
//!
//! Aggregates all Rhai API sub-modules and provides a single registration point.
//!
//! ## Sub-modules
//! - **lifecycle**: Director creation, scene management, transitions
//! - **nodes**: Node creation (box, text, image, video, lottie, svg, composition)
//! - **animation**: Keyframe, spring, and path animations
//! - **audio**: Audio loading, analysis, and reactivity
//! - **effects**: Visual effects and shaders
//! - **properties**: Node property setters

pub mod animation;
pub mod audio;
pub mod effects;
pub mod lifecycle;
pub mod nodes;
pub mod properties;

use crate::AssetLoader;
use rhai::Engine;
use std::sync::Arc;

/// Register all API functions with the Rhai engine.
pub fn register_all(engine: &mut Engine, loader: Arc<dyn AssetLoader>) {
    lifecycle::register(engine, loader.clone());
    nodes::register(engine, loader.clone());
    animation::register(engine);
    audio::register(engine);
    effects::register(engine);
    properties::register(engine);
}
