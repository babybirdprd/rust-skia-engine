use crate::element::Element;
use crate::node::{BoxNode, TextNode, ImageNode, VideoNode};
use taffy::style::Style;

pub type NodeId = usize;

pub struct SceneNode {
    pub element: Box<dyn Element>,
    pub children: Vec<NodeId>,
    pub parent: Option<NodeId>,
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

pub struct Director {
    pub nodes: Vec<Option<SceneNode>>,
    pub root_id: Option<NodeId>,
    pub width: i32,
    pub height: i32,
    pub fps: u32,
}

impl Director {
    pub fn new(width: i32, height: i32, fps: u32) -> Self {
        Self {
            nodes: Vec::new(),
            root_id: None,
            width,
            height,
            fps,
        }
    }

    pub fn add_node(&mut self, element: Box<dyn Element>) -> NodeId {
        let id = self.nodes.len();
        self.nodes.push(Some(SceneNode::new(element)));
        id
    }

    pub fn add_child(&mut self, parent: NodeId, child: NodeId) {
        if let Some(p_node) = self.nodes.get_mut(parent).and_then(|n| n.as_mut()) {
            p_node.children.push(child);
        }
        if let Some(c_node) = self.nodes.get_mut(child).and_then(|n| n.as_mut()) {
            c_node.parent = Some(parent);
        }
    }

    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut SceneNode> {
        self.nodes.get_mut(id).and_then(|n| n.as_mut())
    }

    pub fn get_node(&self, id: NodeId) -> Option<&SceneNode> {
        self.nodes.get(id).and_then(|n| n.as_ref())
    }

    // Animation Update Loop
    pub fn update(&mut self, time: f64) {
        for node_opt in self.nodes.iter_mut() {
            if let Some(node) = node_opt {
                node.element.update(time);
            }
        }
    }
}
