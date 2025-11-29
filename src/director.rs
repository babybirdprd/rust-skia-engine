use crate::element::Element;
// use rayon::prelude::*; // Rayon disabled due to Taffy !Send
use skia_safe::{Path, PathMeasure};
use crate::animation::Animated;

/// A unique identifier for a node in the scene graph.
pub type NodeId = usize;

#[derive(Clone)]
pub struct PathAnimationState {
    pub path: Path,
    pub progress: Animated<f32>,
}

/// A wrapper around an `Element` that adds scene graph relationships.
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
<<<<<<< HEAD

=======

>>>>>>> d1f706e (Apply patch /tmp/d75507de-5e68-4ed1-938f-a41d1f20d671.patch)
    // Path Animation
    pub path_animation: Option<PathAnimationState>,
    pub translation: (f32, f32),
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
            translation: (0.0, 0.0),
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
}

/// The central engine state.
pub struct Director {
    /// The Arena of all nodes. Using `Option` allows for future removal/recycling.
    pub nodes: Vec<Option<SceneNode>>,
    /// The timeline of scenes.
    pub timeline: Vec<TimelineItem>,
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
}

impl Director {
    /// Creates a new Director instance.
    pub fn new(width: i32, height: i32, fps: u32) -> Self {
        Self {
            nodes: Vec::new(),
            timeline: Vec::new(),
            width,
            height,
            fps,
            samples_per_frame: 1, // Default to no motion blur
            shutter_angle: 180.0,
        }
    }

    /// Adds a new element to the scene graph and returns its ID.
    pub fn add_node(&mut self, element: Box<dyn Element>) -> NodeId {
        let id = self.nodes.len();
        self.nodes.push(Some(SceneNode::new(element)));
        id
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
<<<<<<< HEAD

=======

>>>>>>> d1f706e (Apply patch /tmp/d75507de-5e68-4ed1-938f-a41d1f20d671.patch)
        while let Some((id, time)) = stack.pop() {
            if id >= self.nodes.len() { continue; }
            if self.nodes[id].is_none() { continue; }

            let node = self.nodes[id].as_mut().unwrap();
<<<<<<< HEAD

            node.local_time = time;
            node.last_visit_time = global_time;

=======

            node.local_time = time;
            node.last_visit_time = global_time;

>>>>>>> d1f706e (Apply patch /tmp/d75507de-5e68-4ed1-938f-a41d1f20d671.patch)
            let children = node.children.clone();
            for child in children {
                stack.push((child, time));
            }
        }

        // Pass 2: Serial Update (Rayon removed)
        for node_opt in self.nodes.iter_mut() {
            if let Some(node) = node_opt {
                if (node.last_visit_time - global_time).abs() < 0.0001 {
                    node.element.update(node.local_time);
<<<<<<< HEAD

=======

>>>>>>> d1f706e (Apply patch /tmp/d75507de-5e68-4ed1-938f-a41d1f20d671.patch)
                    // Update Path Animation
                    if let Some(path_anim) = &mut node.path_animation {
                        path_anim.progress.update(node.local_time);
                        let mut measure = PathMeasure::new(&path_anim.path, false, None);
                        let length = measure.length();
                        let dist = path_anim.progress.current_value * length;
                        if let Some((p, _tangent)) = measure.pos_tan(dist) {
                             node.translation = (p.x, p.y);
                        }
                    }
                }
            }
        }
    }
}
