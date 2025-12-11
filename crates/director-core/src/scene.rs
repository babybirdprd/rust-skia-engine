use crate::element::Element;
use crate::types::{NodeId, PathAnimationState, Transform};

/// A wrapper around an `Element` that adds scene graph relationships and state.
///
/// `SceneNode` encapsulates the specific logic for hierarchy, layout positioning,
/// masking, and temporal state (local time).
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

    /// Explicit render order z-index (default: 0).
    /// Higher values render on top of lower values.
    /// Sorting is local to the parent's children list.
    pub z_index: i32,

    pub dirty_style: bool,
}

impl SceneNode {
    /// Creates a new SceneNode wrapping the given Element.
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
            z_index: 0,
            dirty_style: true,
        }
    }
}

/// The Scene Graph data structure.
///
/// Manages the arena of nodes and their relationships.
#[derive(Clone)]
pub struct SceneGraph {
    /// The Arena of all nodes. Using `Option` allows for future removal/recycling.
    pub nodes: Vec<Option<SceneNode>>,
    /// Indices of nodes that have been removed and can be reused.
    pub free_indices: Vec<usize>,
}

impl SceneGraph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            free_indices: Vec::new(),
        }
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
}
