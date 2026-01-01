//! # Director Module
//!
//! The central orchestrator for the video rendering engine.
//!
//! ## Responsibilities
//! - **Timeline Management**: Maintains a `Vec<TimelineItem>` of scenes.
//! - **Update Loop**: Drives animation, audio sync, and scene transitions.
//! - **Scene Coordination**: Manages active scenes and their time ranges.
//!
//! ## Key Types
//! - `Director`: The god object that owns timeline, assets, and context.
//! - `TimelineItem`: A scene with start time, duration, and transitions.
//! - `DirectorContext`: Shared state for nested compositions.

// use rayon::prelude::*; // Rayon disabled due to Taffy !Send
use crate::audio::{AudioAnalyzer, AudioMixer, AudioTrack};
use crate::scene::SceneGraph;
use crate::systems::assets::AssetManager;
use crate::systems::transitions::Transition;
use crate::types::NodeId;
use crate::video_wrapper::RenderMode;
use crate::AssetLoader;
use skia_safe::textlayout::{FontCollection, TypefaceFontProvider};
use skia_safe::PathMeasure;
use skia_safe::{Data, FontMgr};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::instrument;

/// Shared resources context that can be passed between Directors (e.g. for sub-compositions).
#[derive(Clone)]
pub struct DirectorContext {
    pub assets: AssetManager,
}

/// Represents a scene (or clip) on the timeline.
#[derive(Clone, Debug)]
pub struct TimelineItem {
    /// The root node of this scene.
    pub scene_root: NodeId,
    /// The global start time in seconds.
    pub start_time: f64,
    /// The duration of the scene in seconds.
    pub duration: f64,
    /// Z-index for rendering order.
    pub z_index: i32,
    /// Associated audio tracks.
    pub audio_tracks: Vec<usize>,
}

/// The central engine coordinator.
///
/// `Director` manages the Scene Graph (nodes), Timeline (sequencing),
/// and shared resources (Assets, Fonts, Audio).
#[derive(Clone)]
pub struct Director {
    /// The Scene Graph.
    pub scene: SceneGraph,
    /// The timeline of scenes.
    pub timeline: Vec<TimelineItem>,
    /// Active transitions.
    pub transitions: Vec<Transition>,
    /// Output width in pixels.
    pub width: i32,
    /// Output height in pixels.
    pub height: i32,
    /// Frames Per Second.
    pub fps: u32,
    /// Number of sub-frame samples for motion blur (default: 1).
    pub samples_per_frame: u32,
    /// Shutter angle in degrees (0.0 to 360.0). Default: 180.0.
    pub shutter_angle: f32,
    /// Render Mode (Preview or Export).
    pub render_mode: RenderMode,
    /// Audio Mixer state.
    pub audio_mixer: AudioMixer,
    /// Audio Analyzer for FFT-based spectrum analysis.
    pub audio_analyzer: AudioAnalyzer,
    /// Shared Asset Manager.
    pub assets: AssetManager,
}

impl Director {
    /// Creates a new Director instance.
    ///
    /// # Arguments
    /// * `width` - Output width in pixels.
    /// * `height` - Output height in pixels.
    /// * `fps` - Frame rate.
    /// * `asset_loader` - Implementation of asset loading strategy.
    /// * `render_mode` - Mode hint (e.g. Preview vs Export).
    /// * `context` - Optional existing context (for nested compositions).
    pub fn new(
        width: i32,
        height: i32,
        fps: u32,
        asset_loader: Arc<dyn AssetLoader>,
        render_mode: RenderMode,
        context: Option<DirectorContext>,
    ) -> Self {
        let assets = if let Some(ctx) = context {
            ctx.assets
        } else {
            let mut font_collection = FontCollection::new();
            let mut font_provider = TypefaceFontProvider::new();

            // Load system fonts fallback
            font_collection.set_default_font_manager(FontMgr::default(), None);

            // Load fallback font (e.g. Emoji) if available
            if let Some(bytes) = asset_loader.load_font_fallback() {
                let data = Data::new_copy(&bytes);
                // We need to register the typeface with an alias if possible,
                // or just register it. TypefaceFontProvider::register_typeface returns usize.
                if let Some(typeface) = FontMgr::new().new_from_data(&data, 0) {
                    font_provider.register_typeface(typeface, Some("Fallback"));
                }
            }

            // Connect provider to collection
            font_collection.set_asset_font_manager(Some(font_provider.clone().into()));

            AssetManager::new(
                asset_loader,
                Arc::new(Mutex::new(font_collection)),
                Arc::new(Mutex::new(font_provider)),
                Arc::new(Mutex::new(HashMap::new())),
            )
        };

        Self {
            scene: SceneGraph::new(),
            timeline: Vec::new(),
            transitions: Vec::new(),
            width,
            height,
            fps,
            samples_per_frame: 1, // Default to no motion blur
            shutter_angle: 180.0,
            render_mode,
            audio_mixer: AudioMixer::new(48000),
            audio_analyzer: AudioAnalyzer::new(2048, 48000),
            assets,
        }
    }

    /// Mixes audio for the current frame time by traversing the scene graph.
    ///
    /// This aggregates audio from both global tracks and active scene nodes (including nested compositions).
    pub fn mix_audio(&mut self, samples_needed: usize, time: f64) -> Vec<f32> {
        let mut output = self.audio_mixer.mix(samples_needed, time);

        // Collect active scene roots
        let active_roots: Vec<(crate::types::NodeId, f64)> = self
            .timeline
            .iter()
            .filter(|item| time >= item.start_time && time < item.start_time + item.duration)
            .map(|item| (item.scene_root, time - item.start_time))
            .collect();

        // Delegate scene graph audio mixing to audio module
        crate::audio::mix_scene_audio(
            &mut output,
            &self.scene.nodes,
            &active_roots,
            samples_needed,
            self.audio_mixer.sample_rate,
        );

        output
    }

    /// Adds a global audio track that plays independently of scenes.
    pub fn add_global_audio(&mut self, samples: Vec<f32>) -> usize {
        let track = AudioTrack {
            samples,
            volume: crate::animation::Animated::new(1.0),
            start_time: 0.0,
            duration: None,
            loop_audio: false,
        };
        self.audio_mixer.add_track(track)
    }

    /// Adds an audio track synchronized to a specific scene's start time.
    pub fn add_scene_audio(&mut self, samples: Vec<f32>, start_time: f64, duration: f64) -> usize {
        let track = AudioTrack {
            samples,
            volume: crate::animation::Animated::new(1.0),
            start_time,
            duration: Some(duration),
            loop_audio: false,
        };
        self.audio_mixer.add_track(track)
    }

    /// Updates the state of all active nodes for the given global time.
    ///
    /// This method calculates local time for each node, updates animations (transform, path),
    /// and calls `update()` on the underlying Elements.
    #[instrument(level = "debug", skip(self), fields(time = global_time))]
    pub fn update(&mut self, global_time: f64) {
        // Pass 1: Mark active nodes and set local time
        let mut active_roots = Vec::new();
        for item in &self.timeline {
            if global_time >= item.start_time && global_time < item.start_time + item.duration {
                let local_time = global_time - item.start_time;
                active_roots.push((item.scene_root, local_time));
            }
        }

        let mut stack = Vec::new();
        for (root, t) in active_roots {
            stack.push((root, t));
        }

        while let Some((id, time)) = stack.pop() {
            if id >= self.scene.nodes.len() {
                continue;
            }
            if self.scene.nodes[id].is_none() {
                continue;
            }

            let node = self.scene.nodes[id].as_mut().unwrap();

            node.local_time = time;
            node.last_visit_time = global_time;

            let children = node.children.clone();
            for child in children {
                stack.push((child, time));
            }

            // Also traverse mask node to ensure it gets updates
            if let Some(mask_id) = node.mask_node {
                stack.push((mask_id, time));
            }
        }

        // Pass 2: Serial Update (Rayon removed)
        for (node_id, node_opt) in self.scene.nodes.iter_mut().enumerate() {
            if let Some(node) = node_opt {
                if (node.last_visit_time - global_time).abs() < 0.0001 {
                    if node.element.update(node.local_time) {
                        node.dirty_style = true;
                    }

                    // Update Transform Animations
                    node.transform.scale_x.update(node.local_time);
                    node.transform.scale_y.update(node.local_time);
                    node.transform.rotation.update(node.local_time);
                    node.transform.skew_x.update(node.local_time);
                    node.transform.skew_y.update(node.local_time);
                    node.transform.translate_x.update(node.local_time);
                    node.transform.translate_y.update(node.local_time);

                    // Update Path Animation
                    if let Some(path_anim) = &mut node.path_animation {
                        path_anim.progress.update(node.local_time);
                        let mut measure = PathMeasure::new(&path_anim.path, false, None);
                        let length = measure.length();
                        let dist = path_anim.progress.current_value * length;
                        if let Some((p, _tangent)) = measure.pos_tan(dist) {
                            node.transform.translate_x.current_value = p.x;
                            node.transform.translate_y.current_value = p.y;
                        }
                    }

                    if node.transform.translate_x.current_value > 0.0 {
                        tracing::debug!(
                            "Node {} updated: x={}, local_time={}",
                            node_id,
                            node.transform.translate_x.current_value,
                            node.local_time
                        );
                    }
                }
            }
        }

        // Pass 3: Audio Reactive Bindings
        // Process after animations so audio values take priority
        for node_opt in self.scene.nodes.iter_mut() {
            if let Some(node) = node_opt {
                if (node.last_visit_time - global_time).abs() < 0.0001 {
                    for binding in &mut node.audio_bindings {
                        // Get audio samples from the track
                        let energy = if let Some(Some(track)) =
                            self.audio_mixer.tracks.get(binding.track_id)
                        {
                            self.audio_analyzer.get_energy(
                                &track.samples,
                                global_time,
                                &binding.band,
                            )
                        } else {
                            0.0
                        };

                        // Map energy (0-1) to output range
                        let raw_value =
                            binding.min_value + energy * (binding.max_value - binding.min_value);

                        // Apply temporal smoothing
                        let smoothed = if binding.smoothing > 0.0 {
                            binding.prev_value * binding.smoothing
                                + raw_value * (1.0 - binding.smoothing)
                        } else {
                            raw_value
                        };
                        binding.prev_value = smoothed;

                        // Apply to property
                        match binding.property.as_str() {
                            "scale" => {
                                node.transform.scale_x.current_value = smoothed;
                                node.transform.scale_y.current_value = smoothed;
                            }
                            "scale_x" => node.transform.scale_x.current_value = smoothed,
                            "scale_y" => node.transform.scale_y.current_value = smoothed,
                            "x" => node.transform.translate_x.current_value = smoothed,
                            "y" => node.transform.translate_y.current_value = smoothed,
                            "rotation" => node.transform.rotation.current_value = smoothed,
                            // Note: opacity requires Element trait extension
                            // "opacity" => node.element.set_opacity(smoothed),
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    /// Triggers `post_layout` on all active nodes.
    ///
    /// This is called after the Layout Engine has computed the final boxes, allowing elements
    /// to adjust their internal state (e.g., text resizing) based on the final layout.
    pub fn run_post_layout(&mut self, global_time: f64) {
        for node_opt in self.scene.nodes.iter_mut() {
            if let Some(node) = node_opt {
                if (node.last_visit_time - global_time).abs() < 0.0001 {
                    node.element.post_layout(node.layout_rect);
                }
            }
        }
    }
}
