// use rayon::prelude::*; // Rayon disabled due to Taffy !Send
use skia_safe::PathMeasure;
use crate::animation::EasingType;
use crate::AssetLoader;
use crate::audio::{AudioMixer, AudioTrack};
use crate::video_wrapper::RenderMode;
use crate::types::NodeId;
use crate::systems::assets::AssetManager;
use crate::scene::SceneGraph;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use skia_safe::{Data, FontMgr};
use skia_safe::textlayout::{FontCollection, TypefaceFontProvider};

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

/// The type of visual transition between scenes.
#[derive(Clone, Debug)]
pub enum TransitionType {
    Fade,
    SlideLeft,
    SlideRight,
    WipeLeft,
    WipeRight,
    CircleOpen,
}

/// A definition of a transition between two scenes.
#[derive(Clone)]
pub struct Transition {
    pub from_scene_idx: usize,
    pub to_scene_idx: usize,
    pub start_time: f64,
    pub duration: f64,
    pub kind: TransitionType,
    pub easing: EasingType,
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
    pub fn new(width: i32, height: i32, fps: u32, asset_loader: Arc<dyn AssetLoader>, render_mode: RenderMode, context: Option<DirectorContext>) -> Self {

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
            assets,
        }
    }

    /// Mixes audio for the current frame time by traversing the scene graph.
    ///
    /// This aggregates audio from both global tracks and active scene nodes (including nested compositions).
    pub fn mix_audio(&mut self, samples_needed: usize, time: f64) -> Vec<f32> {
        let mut output = self.audio_mixer.mix(samples_needed, time);

        // Traverse active scenes
        let mut active_roots = Vec::new();
        for item in &self.timeline {
             if time >= item.start_time && time < item.start_time + item.duration {
                 let local_time = time - item.start_time;
                 active_roots.push((item.scene_root, local_time));
             }
        }

        let mut stack = Vec::new();
        for (root, t) in active_roots {
            stack.push((root, t));
        }

        while let Some((id, local_time)) = stack.pop() {
             // We access nodes directly to avoid self borrow issues with get_node if we were using &mut self methods
             // But here we need to read nodes.
             if id < self.scene.nodes.len() {
                 if let Some(node) = &self.scene.nodes[id] {
                     // Check audio
                     if let Some(samples) = node.element.get_audio(local_time, samples_needed, self.audio_mixer.sample_rate) {
                         for (i, val) in samples.iter().enumerate() {
                             if i < output.len() {
                                 output[i] += val;
                             }
                         }
                     }

                     // Children
                     for child_id in &node.children {
                         stack.push((*child_id, local_time));
                     }
                 }
             }
        }

        // Clamp
        for s in output.iter_mut() {
            *s = s.clamp(-1.0, 1.0);
        }

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
            if id >= self.scene.nodes.len() { continue; }
            if self.scene.nodes[id].is_none() { continue; }

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
        for node_opt in self.scene.nodes.iter_mut() {
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
