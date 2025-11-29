use crate::element::Element;

/// A unique identifier for a node in the scene graph.
pub type NodeId = usize;

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
}

impl SceneNode {
    pub fn new(element: Box<dyn Element>) -> Self {
        Self {
            element,
            children: Vec::new(),
            parent: None,
            layout_rect: skia_safe::Rect::default(),
        }
    }
}

/// The central engine state.
///
/// `Director` holds the Scene Graph (Arena), manages the root node,
/// and stores global configuration like video resolution and FPS.
pub struct Director {
    /// The Arena of all nodes. Using `Option` allows for future removal/recycling.
    pub nodes: Vec<Option<SceneNode>>,
    /// The root node of the scene (usually the main container).
    pub root_id: Option<NodeId>,
    /// Output width in pixels.
    pub width: i32,
    /// Output height in pixels.
    pub height: i32,
    /// Frames Per Second.
    pub fps: u32,
    /// Number of sub-frame samples for motion blur (default: 1).
    pub samples_per_frame: u32,
    /// Shutter angle in degrees (0.0 to 360.0). Default: 180.0.
    /// 180.0 degrees = 50% of frame duration.
    pub shutter_angle: f32,
}

impl Director {
    /// Creates a new Director instance.
    pub fn new(width: i32, height: i32, fps: u32) -> Self {
        Self {
            nodes: Vec::new(),
            root_id: None,
            width,
            height,
            fps,
            samples_per_frame: 1, // Default to no motion blur
            shutter_angle: 180.0,
        }
    }

    /// Adds a new element to the scene graph and returns its ID.
    /// Note: The node is orphaned until attached to a parent (or set as root).
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

    /// Updates all animations in the scene graph to the given time.
    pub fn update(&mut self, time: f64) {
        for node_opt in self.nodes.iter_mut() {
            if let Some(node) = node_opt {
                node.element.update(time);
            }
        }
    }
}
