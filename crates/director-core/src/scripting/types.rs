//! # Scripting Types
//!
//! Handle types for Rhai scripting integration.
//!
//! ## Responsibilities
//! - **MovieHandle**: Wrapper around `Director` for script access
//! - **SceneHandle**: Reference to a timeline scene
//! - **NodeHandle**: Reference to a scene graph node
//! - **AudioTrackHandle**: Reference to an audio track

use crate::director::Director;
use crate::types::NodeId;
use std::sync::{Arc, Mutex};

/// Wrapper around `Director` for Rhai scripting.
#[derive(Clone)]
pub struct MovieHandle {
    pub director: Arc<Mutex<Director>>,
}

/// Handle to a specific Scene (or time segment) in the movie.
#[derive(Clone)]
pub struct SceneHandle {
    pub director: Arc<Mutex<Director>>,
    pub root_id: NodeId,
    pub start_time: f64,
    pub duration: f64,
    pub audio_tracks: Vec<usize>,
}

/// Handle to a specific Node in the scene graph.
#[derive(Clone)]
pub struct NodeHandle {
    pub director: Arc<Mutex<Director>>,
    pub id: NodeId,
}

/// Handle to an audio track.
#[derive(Clone)]
pub struct AudioTrackHandle {
    pub director: Arc<Mutex<Director>>,
    pub id: usize,
}
