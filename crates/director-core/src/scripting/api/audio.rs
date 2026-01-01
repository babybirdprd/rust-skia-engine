//! # Audio API
//!
//! Audio playback and analysis functions for Rhai scripts.
//!
//! ## Responsibilities
//! - **Audio Loading**: `add_audio` for adding audio tracks
//! - **Volume Animation**: `animate_volume` for volume fades
//! - **Audio Analysis**: `bass`, `mids`, `highs`, `get_energy`, `get_spectrum`
//! - **Audio Reactivity**: `bind_audio` for audio-reactive properties

use rhai::Engine;
use tracing::error;

use super::super::types::{AudioTrackHandle, MovieHandle, NodeHandle, SceneHandle};
use super::super::utils::parse_easing;

/// Register audio-related Rhai functions.
pub fn register(engine: &mut Engine) {
    engine.register_type_with_name::<AudioTrackHandle>("AudioTrack");

    engine.register_fn("add_audio", |movie: &mut MovieHandle, path: &str| {
        let mut d = movie.director.lock().unwrap();
        let bytes = d.assets.loader.load_bytes(path).unwrap_or(Vec::new());
        let samples = crate::audio::load_audio_bytes(&bytes, d.audio_mixer.sample_rate)
            .unwrap_or_else(|e| {
                error!("Audio error: {}", e);
                Vec::new()
            });

        let id = d.add_global_audio(samples);
        AudioTrackHandle {
            director: movie.director.clone(),
            id,
        }
    });

    engine.register_fn("add_audio", |scene: &mut SceneHandle, path: &str| {
        let mut d = scene.director.lock().unwrap();
        let bytes = d.assets.loader.load_bytes(path).unwrap_or(Vec::new());
        let samples = crate::audio::load_audio_bytes(&bytes, d.audio_mixer.sample_rate)
            .unwrap_or_else(|e| {
                error!("Audio error: {}", e);
                Vec::new()
            });

        let id = d.add_scene_audio(samples, scene.start_time, scene.duration);

        // Update SceneHandle tracking
        scene.audio_tracks.push(id);

        // Update Director TimelineItem tracking
        if let Some(item) = d
            .timeline
            .iter_mut()
            .find(|i| i.scene_root == scene.root_id)
        {
            item.audio_tracks.push(id);
        }

        AudioTrackHandle {
            director: scene.director.clone(),
            id,
        }
    });

    engine.register_fn(
        "animate_volume",
        |track: &mut AudioTrackHandle, start: f64, end: f64, dur: f64, ease: &str| {
            let mut d = track.director.lock().unwrap();
            if let Some(t) = d.audio_mixer.get_track_mut(track.id) {
                let ease_fn = parse_easing(ease);
                t.volume.add_segment(start as f32, end as f32, dur, ease_fn);
            }
        },
    );

    // Audio Analysis (FFT)
    engine.register_fn("bass", |track: &mut AudioTrackHandle, time: f64| -> f64 {
        let d = track.director.lock().unwrap();
        if let Some(t) = d.audio_mixer.tracks.get(track.id).and_then(|t| t.as_ref()) {
            d.audio_analyzer.bass(&t.samples, time) as f64
        } else {
            0.0
        }
    });

    engine.register_fn("mids", |track: &mut AudioTrackHandle, time: f64| -> f64 {
        let d = track.director.lock().unwrap();
        if let Some(t) = d.audio_mixer.tracks.get(track.id).and_then(|t| t.as_ref()) {
            d.audio_analyzer.mids(&t.samples, time) as f64
        } else {
            0.0
        }
    });

    engine.register_fn("highs", |track: &mut AudioTrackHandle, time: f64| -> f64 {
        let d = track.director.lock().unwrap();
        if let Some(t) = d.audio_mixer.tracks.get(track.id).and_then(|t| t.as_ref()) {
            d.audio_analyzer.highs(&t.samples, time) as f64
        } else {
            0.0
        }
    });

    engine.register_fn(
        "get_energy",
        |track: &mut AudioTrackHandle, time: f64, band: &str| -> f64 {
            let d = track.director.lock().unwrap();
            if let Some(t) = d.audio_mixer.tracks.get(track.id).and_then(|t| t.as_ref()) {
                d.audio_analyzer.get_energy(&t.samples, time, band) as f64
            } else {
                0.0
            }
        },
    );

    engine.register_fn(
        "get_spectrum",
        |track: &mut AudioTrackHandle, time: f64| -> rhai::Array {
            let d = track.director.lock().unwrap();
            if let Some(t) = d.audio_mixer.tracks.get(track.id).and_then(|t| t.as_ref()) {
                d.audio_analyzer
                    .compute_spectrum(&t.samples, time)
                    .into_iter()
                    .map(|v| rhai::Dynamic::from(v as f64))
                    .collect()
            } else {
                rhai::Array::new()
            }
        },
    );

    // Audio Reactive Binding
    // Usage: node.bind_audio(track_id, "bass", "scale")
    // Maps audio energy (0-1) to property values with sensible defaults
    engine.register_fn(
        "bind_audio",
        |node: &mut NodeHandle, track_id: i64, band: &str, property: &str| {
            let mut d = node.director.lock().unwrap();
            // Default range based on property type
            let (min_val, max_val) = match property {
                "scale" | "scale_x" | "scale_y" => (1.0, 2.0),
                "rotation" => (0.0, 30.0),
                _ => (0.0, 100.0),
            };
            if let Some(scene_node) = d.scene.get_node_mut(node.id) {
                scene_node.audio_bindings.push(crate::scene::AudioBinding {
                    track_id: track_id as usize,
                    band: band.to_string(),
                    property: property.to_string(),
                    min_value: min_val,
                    max_value: max_val,
                    smoothing: 0.3,
                    prev_value: min_val,
                });
            }
        },
    );

    // Get track ID from handle for use with bind_audio
    engine.register_fn("id", |track: &mut AudioTrackHandle| -> i64 {
        track.id as i64
    });
}
