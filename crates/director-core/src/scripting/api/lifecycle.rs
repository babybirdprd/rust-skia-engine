//! # Lifecycle API
//!
//! Director and scene lifecycle management for Rhai scripts.
//!
//! ## Responsibilities
//! - **Director Creation**: `new_director` with various overloads
//! - **Scene Management**: `add_scene`, `add_transition`
//! - **Configuration**: `configure_motion_blur`

use crate::director::{Director, TimelineItem};
use crate::node::BoxNode;
use crate::systems::transitions::{Transition, TransitionType};
use crate::video_wrapper::RenderMode;
use crate::AssetLoader;
use rhai::Engine;
use std::sync::{Arc, Mutex};
use taffy::prelude::*;

use super::super::types::{MovieHandle, SceneHandle};
use super::super::utils::parse_easing;

/// Register lifecycle-related Rhai functions.
pub fn register(engine: &mut Engine, loader: Arc<dyn AssetLoader>) {
    // Randomness
    engine.register_fn("rand_float", |min: f64, max: f64| {
        use rand::Rng;
        rand::thread_rng().gen_range(min..max)
    });

    // 1. Director/Movie
    engine.register_type_with_name::<MovieHandle>("Movie");

    // Overload 1: 3 args (Default Preview)
    let l1 = loader.clone();
    engine.register_fn("new_director", move |w: i64, h: i64, fps: i64| {
        let director = Director::new(
            w as i32,
            h as i32,
            fps as u32,
            l1.clone(),
            RenderMode::Preview,
            None,
        );
        MovieHandle {
            director: Arc::new(Mutex::new(director)),
        }
    });

    // Overload 2: 4 args (Config)
    let l2 = loader.clone();
    engine.register_fn(
        "new_director",
        move |w: i64, h: i64, fps: i64, config: rhai::Map| {
            let mode_str = config
                .get("mode")
                .and_then(|v| v.clone().into_string().ok())
                .unwrap_or_else(|| "preview".to_string());
            let mode = match mode_str.as_str() {
                "export" => RenderMode::Export,
                _ => RenderMode::Preview,
            };
            let director = Director::new(w as i32, h as i32, fps as u32, l2.clone(), mode, None);
            MovieHandle {
                director: Arc::new(Mutex::new(director)),
            }
        },
    );

    engine.register_fn(
        "configure_motion_blur",
        |movie: &mut MovieHandle, samples: i64, shutter_angle: f64| {
            let mut d = movie.director.lock().unwrap();
            d.samples_per_frame = samples as u32;
            d.shutter_angle = shutter_angle as f32;
        },
    );

    // 2. Scene Management
    engine.register_type_with_name::<SceneHandle>("Scene");
    engine.register_fn("add_scene", |movie: &mut MovieHandle, duration: f64| {
        let mut d = movie.director.lock().unwrap();
        let start_time = d
            .timeline
            .last()
            .map(|i| i.start_time + i.duration)
            .unwrap_or(0.0);

        let mut root = BoxNode::new();
        root.style.size = taffy::geometry::Size {
            width: Dimension::percent(1.0),
            height: Dimension::percent(1.0),
        };
        let id = d.scene.add_node(Box::new(root));

        let item = TimelineItem {
            scene_root: id,
            start_time,
            duration,
            z_index: 0,
            audio_tracks: Vec::new(),
        };
        d.timeline.push(item);

        SceneHandle {
            director: movie.director.clone(),
            root_id: id,
            start_time,
            duration,
            audio_tracks: Vec::new(),
        }
    });

    engine.register_fn(
        "add_transition",
        |movie: &mut MovieHandle,
         from: SceneHandle,
         to: SceneHandle,
         type_str: &str,
         duration: f64,
         easing_str: &str| {
            let mut d = movie.director.lock().unwrap();

            // Find indices
            let from_idx = d.timeline.iter().position(|i| i.scene_root == from.root_id);
            let to_idx = d.timeline.iter().position(|i| i.scene_root == to.root_id);

            if let (Some(f_idx), Some(t_idx)) = (from_idx, to_idx) {
                // Ripple Left Logic
                // We shift 'to' scene and all subsequent scenes (index >= t_idx) left by duration.

                for i in t_idx..d.timeline.len() {
                    d.timeline[i].start_time -= duration;

                    // Sync Audio
                    let audio_ids = d.timeline[i].audio_tracks.clone();
                    for track_id in audio_ids {
                        if let Some(track) = d.audio_mixer.get_track_mut(track_id) {
                            track.start_time -= duration;
                        }
                    }
                }

                let kind = match type_str {
                    "fade" => TransitionType::Fade,
                    "slide_left" | "slide-left" => TransitionType::SlideLeft,
                    "slide_right" | "slide-right" => TransitionType::SlideRight,
                    "wipe_left" | "wipe-left" => TransitionType::WipeLeft,
                    "wipe_right" | "wipe-right" => TransitionType::WipeRight,
                    "circle_open" | "circle-open" => TransitionType::CircleOpen,
                    _ => TransitionType::Fade,
                };

                let easing = parse_easing(easing_str);

                let start_time = d.timeline[t_idx].start_time;

                let transition = Transition {
                    from_scene_idx: f_idx,
                    to_scene_idx: t_idx,
                    start_time,
                    duration,
                    kind,
                    easing,
                };

                d.transitions.push(transition);
            }
        },
    );
}
