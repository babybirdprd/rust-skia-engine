use taffy::prelude::*;
use crate::director::{Director, NodeId};

pub struct LayoutEngine {
    taffy: TaffyTree,
    // Temporary map for the current frame calculation
    node_map: std::collections::HashMap<NodeId, taffy::NodeId>,
}

impl LayoutEngine {
    pub fn new() -> Self {
        Self {
            taffy: TaffyTree::new(),
            node_map: std::collections::HashMap::new(),
        }
    }

    pub fn compute_layout(&mut self, director: &mut Director) {
        // 1. Rebuild Taffy Tree from Director Scene Graph
        self.taffy = TaffyTree::new();
        self.node_map.clear();

        // Assume root_id exists and is the entry point
        if let Some(root_id) = director.root_id {
            // Need to handle missing node safely
            if let Some(node) = director.get_node(root_id) {
                let root_style = node.element.layout_style();
                // Force root size to match video dims
                let mut style = root_style;
                style.size = Size {
                    width: length(director.width as f32),
                    height: length(director.height as f32),
                };

                let taffy_root = self.build_recursive(director, root_id);

                // 2. Compute
                self.taffy.compute_layout(
                    taffy_root,
                    Size {
                        width: AvailableSpace::Definite(director.width as f32),
                        height: AvailableSpace::Definite(director.height as f32),
                    },
                ).unwrap();

                // 3. Write back results to Director Nodes
                self.write_back_recursive(director, root_id);
            }
        }
    }

    fn build_recursive(&mut self, director: &Director, node_id: NodeId) -> taffy::NodeId {
        let node = director.get_node(node_id).unwrap();
        let style = node.element.layout_style();

        let mut children_ids = Vec::new();
        for &child_id in &node.children {
            children_ids.push(self.build_recursive(director, child_id));
        }

        let t_id = self.taffy.new_with_children(style, &children_ids).unwrap();
        self.node_map.insert(node_id, t_id);
        t_id
    }

    fn write_back_recursive(&self, director: &mut Director, node_id: NodeId) {
        if let Some(t_id) = self.node_map.get(&node_id) {
            let layout = self.taffy.layout(*t_id).unwrap();
            let node = director.get_node_mut(node_id).unwrap();

            node.layout_rect = skia_safe::Rect::from_xywh(
                layout.location.x,
                layout.location.y,
                layout.size.width,
                layout.size.height,
            );

            // Recurse
             let children = node.children.clone();
             for child_id in children {
                 self.write_back_recursive(director, child_id);
             }
        }
    }
}
