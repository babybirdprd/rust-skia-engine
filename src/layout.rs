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

    pub fn compute_layout(&mut self, director: &mut Director, time: f64) {
        // 1. Rebuild Taffy Tree from Director Scene Graph
        self.taffy = TaffyTree::new();
        self.node_map.clear();

        // Iterate timeline to find active roots
        let mut active_roots = Vec::new();
        for item in &director.timeline {
             if time >= item.start_time && time < item.start_time + item.duration {
                 active_roots.push(item.scene_root);
             }
        }

        for root_id in active_roots {
            // Need to handle missing node safely
            if director.get_node(root_id).is_some() {
                // Build tree for this scene
                let taffy_root = self.build_recursive(director, root_id);

                // 2. Compute Layout
                // The root node of a scene fills the screen by default?
                // Or we respect its style?
                // Existing code forced root size.
                // Let's force it for consistency.

                // Note: We can't easily modify the style inside `director` from here without mutable borrow,
                // but `build_recursive` reads style.
                // Taffy allows overriding root size in `compute_layout`.
                // We pass Definite size which acts as constraints.

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
        // Include mask_node in layout
        if let Some(mask_id) = node.mask_node {
            children_ids.push(self.build_recursive(director, mask_id));
        }

        let t_id = self.taffy.new_with_children(style, &children_ids).unwrap();
        self.node_map.insert(node_id, t_id);
        t_id
    }

    fn write_back_recursive(&self, director: &mut Director, node_id: NodeId) {
        if let Some(t_id) = self.node_map.get(&node_id) {
            let layout = self.taffy.layout(*t_id).unwrap();

            // Scope for mutable borrow
            let (children, mask_node) = {
                let node = director.get_node_mut(node_id).unwrap();

                node.layout_rect = skia_safe::Rect::from_xywh(
                    layout.location.x,
                    layout.location.y,
                    layout.size.width,
                    layout.size.height,
                );
                (node.children.clone(), node.mask_node)
            };

            // Recurse
             for child_id in children {
                 self.write_back_recursive(director, child_id);
             }
             if let Some(mask_id) = mask_node {
                 self.write_back_recursive(director, mask_id);
             }
        }
    }
}
