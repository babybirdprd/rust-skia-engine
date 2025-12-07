use crate::element::Element;
// use rayon::prelude::*; // Rayon disabled due to Taffy !Send
use skia_safe::{Path, PathMeasure};
use crate::animation::{Animated, EasingType};
use crate::AssetLoader;
use crate::audio::{AudioMixer, AudioTrack};
use crate::video_wrapper::RenderMode;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use skia_safe::RuntimeEffect;
use cosmic_text::{FontSystem, SwashCache, fontdb::Source};

/// A unique identifier for a node in the scene graph.
pub type NodeId = usize;

#[derive(Clone)]
pub struct DirectorContext {
    pub asset_loader: Arc<dyn AssetLoader>,
    pub font_system: Arc<Mutex<FontSystem>>,
    pub swash_cache: Arc<Mutex<SwashCache>>,
    pub shader_cache: Arc<Mutex<HashMap<String, RuntimeEffect>>>,
    pub typeface_cache: Arc<Mutex<HashMap<cosmic_text::fontdb::ID, skia_safe::Typeface>>>,
}

#[derive(Clone)]
pub struct PathAnimationState {
    pub path: Path,
    pub progress: Animated<f32>,
}

#[derive(Clone, Debug)]
pub struct Transform {
    pub scale_x: Animated<f32>,
    pub scale_y: Animated<f32>,
    pub rotation: Animated<f32>,
    pub skew_x: Animated<f32>,
    pub skew_y: Animated<f32>,
    pub translate_x: Animated<f32>,
    pub translate_y: Animated<f32>,
    pub pivot_x: f32,
    pub pivot_y: f32,
}

impl Transform {
    pub fn new() -> Self {
        Self {
            scale_x: Animated::new(1.0),
            scale_y: Animated::new(1.0),
            rotation: Animated::new(0.0),
            skew_x: Animated::new(0.0),
            skew_y: Animated::new(0.0),
            translate_x: Animated::new(0.0),
            translate_y: Animated::new(0.0),
            pivot_x: 0.5,
            pivot_y: 0.5,
        }
    }
}

/// A wrapper around an `Element` that adds scene graph relationships.
#[derive(Clone)]
pub struct SceneNode {
    /// The actual visual element (Box, Text, etc.)
    pub element: Box<dyn Element>,
    /// Indices of child nodes.
    pub children: Vec<NodeId>,
    /// Index of parent node.
    pub parent: Option<NodeId>,
    /// The computed absolute layout rectangle (set by `LayoutEngine`).
    pub layout_rect: skia_safe::Rect,
    /// The local time for the current frame (computed during update pass).
    pub local_time: f64,
    /// The global time when this node was last visited/prepared for update.
    pub last_visit_time: f64,

    // Path Animation
    pub path_animation: Option<PathAnimationState>,
    pub transform: Transform,

    // Masking & Compositing
    pub mask_node: Option<NodeId>,
    pub blend_mode: skia_safe::BlendMode,

    pub dirty_style: bool,
}

impl SceneNode {
    pub fn new(element: Box<dyn Element>) -> Self {
        Self {
            element,
            children: Vec::new(),
            parent: None,
            layout_rect: skia_safe::Rect::default(),
            local_time: 0.0,
            last_visit_time: -1.0,
            path_animation: None,
            transform: Transform::new(),
            mask_node: None,
            blend_mode: skia_safe::BlendMode::SrcOver,
            dirty_style: true,
        }
    }
}

/// Represents a scene (or clip) on the timeline.
#[derive(Clone, Debug)]
pub struct TimelineItem {
    pub scene_root: NodeId,
    pub start_time: f64,
    pub duration: f64,
    pub z_index: i32,
    pub audio_tracks: Vec<usize>,
}

#[derive(Clone, Debug)]
pub enum TransitionType {
    Fade,
    SlideLeft,
    SlideRight,
    WipeLeft,
    WipeRight,
    CircleOpen,
}

#[derive(Clone)]
pub struct Transition {
    pub from_scene_idx: usize,
    pub to_scene_idx: usize,
    pub start_time: f64,
    pub duration: f64,
    pub kind: TransitionType,
    pub easing: EasingType,
}

/// The central engine state.
#[derive(Clone)]
pub struct Director {
    /// The Arena of all nodes. Using `Option` allows for future removal/recycling.
    pub nodes: Vec<Option<SceneNode>>,
    /// Indices of nodes that have been removed and can be reused.
    pub free_indices: Vec<usize>,
    /// The timeline of scenes.
    pub timeline: Vec<TimelineItem>,
    /// Transitions
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
    /// Render Mode (Preview or Export)
    pub render_mode: RenderMode,
    /// Asset loader for resolving file paths to bytes.
    pub asset_loader: Arc<dyn AssetLoader>,
    /// Audio Mixer state
    pub audio_mixer: AudioMixer,
    /// Global shader cache
    pub shader_cache: Arc<Mutex<HashMap<String, RuntimeEffect>>>,
    /// Shared Font System (initialized once)
    pub font_system: Arc<Mutex<FontSystem>>,
    /// Shared Swash Cache (initialized once)
    pub swash_cache: Arc<Mutex<SwashCache>>,
    /// Shared Typeface Cache (initialized once)
    pub typeface_cache: Arc<Mutex<HashMap<cosmic_text::fontdb::ID, skia_safe::Typeface>>>,
}

impl Director {
    /// Creates a new Director instance.
    pub fn new(width: i32, height: i32, fps: u32, asset_loader: Arc<dyn AssetLoader>, render_mode: RenderMode, context: Option<DirectorContext>) -> Self {

        let (font_system, swash_cache, shader_cache, typeface_cache, loader_to_use) = if let Some(ctx) = context {
            (ctx.font_system, ctx.swash_cache, ctx.shader_cache, ctx.typeface_cache, ctx.asset_loader)
        } else {
            let mut font_system = FontSystem::new();
            // Load fallback font if available
            if let Some(bytes) = asset_loader.load_font_fallback() {
                 let mut db = font_system.db_mut();
                 db.load_font_source(Source::Binary(Arc::new(bytes)));
            }
            (
                Arc::new(Mutex::new(font_system)),
                Arc::new(Mutex::new(SwashCache::new())),
                Arc::new(Mutex::new(HashMap::new())),
                Arc::new(Mutex::new(HashMap::new())),
                asset_loader
            )
        };

        Self {
            nodes: Vec::new(),
            free_indices: Vec::new(),
            timeline: Vec::new(),
            transitions: Vec::new(),
            width,
            height,
            fps,
            samples_per_frame: 1, // Default to no motion blur
            shutter_angle: 180.0,
            render_mode,
            asset_loader: loader_to_use,
            audio_mixer: AudioMixer::new(48000),
            shader_cache,
            font_system,
            swash_cache,
            typeface_cache,
        }
    }

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
             if id < self.nodes.len() {
                 if let Some(node) = &self.nodes[id] {
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

    /// Adds a new element to the scene graph and returns its ID.
    pub fn add_node(&mut self, element: Box<dyn Element>) -> NodeId {
        if let Some(id) = self.free_indices.pop() {
            self.nodes[id] = Some(SceneNode::new(element));
            id
        } else {
            let id = self.nodes.len();
            self.nodes.push(Some(SceneNode::new(element)));
            id
        }
    }

    /// Recursively destroys a node and its children, freeing their indices for reuse.
    pub fn destroy_node(&mut self, id: NodeId) {
        // 1. Check if node exists (and isn't already deleted)
        if id >= self.nodes.len() || self.nodes[id].is_none() {
            return;
        }

        // 2. Collect IDs to process (to avoid holding borrows on self.nodes)
        let (parent_id, children_ids) = {
            let node = self.nodes[id].as_ref().unwrap();
            (node.parent, node.children.clone())
        };

        // 3. Detach from Parent
        if let Some(pid) = parent_id {
            self.remove_child(pid, id);
        }

        // 4. Recursively destroy children
        for child_id in children_ids {
            self.destroy_node(child_id);
        }

        // 5. Free the slot
        self.nodes[id] = None;
        self.free_indices.push(id);
    }

    /// Establishes a parent-child relationship between two nodes.
    pub fn add_child(&mut self, parent: NodeId, child: NodeId) {
        if let Some(p_node) = self.nodes.get_mut(parent).and_then(|n| n.as_mut()) {
            p_node.children.push(child);
        }
        if let Some(c_node) = self.nodes.get_mut(child).and_then(|n| n.as_mut()) {
            c_node.parent = Some(parent);
        }
    }

    /// Removes a child from a parent node's children list.
    /// Does NOT affect the child's `parent` field (caller must handle that if needed, e.g. re-parenting).
    pub fn remove_child(&mut self, parent: NodeId, child: NodeId) {
        if let Some(p_node) = self.nodes.get_mut(parent).and_then(|n| n.as_mut()) {
            if let Some(pos) = p_node.children.iter().position(|&x| x == child) {
                p_node.children.remove(pos);
            }
        }
    }

    /// Returns a mutable reference to the SceneNode.
    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut SceneNode> {
        self.nodes.get_mut(id).and_then(|n| n.as_mut())
    }

    /// Returns a shared reference to the SceneNode.
    pub fn get_node(&self, id: NodeId) -> Option<&SceneNode> {
        self.nodes.get(id).and_then(|n| n.as_ref())
    }

    /// Updates all active scenes in the timeline.
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
            if id >= self.nodes.len() { continue; }
            if self.nodes[id].is_none() { continue; }

            let node = self.nodes[id].as_mut().unwrap();

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
        for node_opt in self.nodes.iter_mut() {
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

    pub fn run_post_layout(&mut self, global_time: f64) {
        for node_opt in self.nodes.iter_mut() {
            if let Some(node) = node_opt {
                if (node.last_visit_time - global_time).abs() < 0.0001 {
                    node.element.post_layout(node.layout_rect);
                }
            }
        }
    }
}
