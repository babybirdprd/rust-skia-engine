use taffy::prelude::*;
use crate::director::{Director, NodeId};

/// Manages the layout computation using the Taffy engine.
///
/// `LayoutEngine` synchronizes the Director's Scene Graph with Taffy's internal tree,
/// computes the Flexbox/Grid layout, and writes the results back to `SceneNode::layout_rect`.
pub struct LayoutEngine {
    taffy: TaffyTree,
    // Persistent map for mapping Director NodeId -> Taffy NodeId
    node_map: std::collections::HashMap<NodeId, taffy::NodeId>,
}

impl LayoutEngine {
    /// Creates a new LayoutEngine instance.
    pub fn new() -> Self {
        Self {
            taffy: TaffyTree::new(),
            node_map: std::collections::HashMap::new(),
        }
    }

    /// Computes the layout for the current frame.
    ///
    /// # Process
    /// 1. **Sync Phase A**: Creates new Taffy nodes for new SceneNodes and updates styles for dirty nodes.
    /// 2. **Sync Phase B**: Updates parent-child relationships in Taffy to match the Scene Graph.
    /// 3. **Compute**: Triggers `taffy.compute_layout` for all active scene roots.
    /// 4. **Write Back**: Copies the computed (x, y, w, h) from Taffy back to `SceneNode`.
    pub fn compute_layout(&mut self, director: &mut Director, time: f64) {
        // 1. Sync Phase A: Ensure Nodes Exist & Update Styles
        // Iterate over all potential node IDs in the Director
        for (id, node_opt) in director.nodes.iter_mut().enumerate() {
            if let Some(node) = node_opt {
                // Ensure existence in Taffy
                let t_id = if let Some(&existing_t_id) = self.node_map.get(&id) {
                    existing_t_id
                } else {
                    let style = node.element.layout_style();
                    let new_t_id = self.taffy.new_leaf(style).unwrap();
                    self.node_map.insert(id, new_t_id);
                    new_t_id
                };

                // Sync Style if dirty
                if node.dirty_style {
                    let style = node.element.layout_style();
                    self.taffy.set_style(t_id, style).unwrap();
                    node.dirty_style = false;
                }
            } else {
                // Node is deleted in Director
                if let Some(t_id) = self.node_map.remove(&id) {
                    self.taffy.remove(t_id).ok();
                }
            }
        }

        // 2. Sync Phase B: Update Relationships (Children)
        // We iterate again. Since we updated all nodes in Phase A, all valid children should be in node_map.
        for (id, node_opt) in director.nodes.iter().enumerate() {
             if let Some(node) = node_opt {
                 if let Some(&t_id) = self.node_map.get(&id) {
                     let mut children_t_ids = Vec::with_capacity(node.children.len() + 1);

                     for &child_id in &node.children {
                         if let Some(&child_t_id) = self.node_map.get(&child_id) {
                             children_t_ids.push(child_t_id);
                         }
                     }
                     if let Some(mask_id) = node.mask_node {
                         if let Some(&mask_t_id) = self.node_map.get(&mask_id) {
                             children_t_ids.push(mask_t_id);
                         }
                     }

                     // Always set children to ensure structure is correct
                     // Taffy's set_children is optimized to do nothing if children list hasn't changed.
                     self.taffy.set_children(t_id, &children_t_ids).unwrap();
                 }
             }
        }

        // 3. Compute Layout for Active Roots
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
                 if let Some(&root_t_id) = self.node_map.get(&root_id) {
                    self.taffy.compute_layout(
                        root_t_id,
                        Size {
                            width: AvailableSpace::Definite(director.width as f32),
                            height: AvailableSpace::Definite(director.height as f32),
                        },
                    ).unwrap();

                    // 4. Write back results to Director Nodes
                    self.write_back_recursive(director, root_id);
                 }
            }
        }
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
