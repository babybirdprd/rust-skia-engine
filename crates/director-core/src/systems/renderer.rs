//! # Renderer System
//!
//! Handles visual output via Skia.
//!
//! ## Responsibilities
//! - **Scene Traversal**: Recursively paints `SceneNode`s to Canvas (`render_recursive`).
//! - **Layer Composition**: Manages canvas save/restore for transforms.
//! - **Debug Rendering**: Single-frame rendering for previews (`render_frame`).
//!
//! ## Key Functions
//! - `render_recursive`: Draws a single node hierarchy.
//! - `render_frame`: Debug helper for single-frame rendering.
//!
//! ## See Also
//! - `export::video` for the full export loop (`render_export`).
//! - `systems::transitions` for transition shaders.

use crate::director::{Director, TimelineItem};
use crate::errors::RenderError;
use crate::scene::SceneGraph;
use crate::systems::assets::AssetManager;
use crate::systems::layout::LayoutEngine;
use crate::systems::transitions::draw_transition;
use crate::types::NodeId;
use tracing::debug;

#[cfg(feature = "vulkan")]
use skia_safe::gpu::DirectContext;

#[cfg(feature = "vulkan")]
/// Type alias for the GPU context (Vulkan backend).
pub type GpuContext = DirectContext;
#[cfg(not(feature = "vulkan"))]
/// Placeholder for GPU context when Vulkan feature is disabled.
pub type GpuContext = ();

/// Recursively renders a node and its children to the canvas.
///
/// Handles transformation stack, blending modes, and masking.
pub fn render_recursive(
    scene: &SceneGraph,
    assets: &AssetManager,
    node_id: NodeId,
    canvas: &skia_safe::Canvas,
    parent_opacity: f32,
    depth: usize,
) -> Result<(), RenderError> {
    if depth > 100 {
        return Err(RenderError::RecursionLimit);
    }
    if let Some(node) = scene.get_node(node_id) {
        canvas.save();

        // Layout Position
        let layout_x = node.layout_rect.left;
        let layout_y = node.layout_rect.top;

        // Transform Properties
        let tx = node.transform.translate_x.current_value;
        let ty = node.transform.translate_y.current_value;
        let scale_x = node.transform.scale_x.current_value;
        let scale_y = node.transform.scale_y.current_value;
        let rotation = node.transform.rotation.current_value;
        let skew_x = node.transform.skew_x.current_value;
        let skew_y = node.transform.skew_y.current_value;

        // Pivot Calculation (absolute)
        let pivot_x = node.layout_rect.width() * node.transform.pivot_x;
        let pivot_y = node.layout_rect.height() * node.transform.pivot_y;

        // Apply Transform Stack
        // 1. Move to position
        canvas.translate((layout_x + tx, layout_y + ty));

        // 2. Move to pivot
        canvas.translate((pivot_x, pivot_y));

        // 3. Rotate
        canvas.rotate(rotation, None);

        // 4. Scale
        canvas.scale((scale_x, scale_y));

        // 5. Skew (Degrees to Tangent)
        let tan_skew_x = skew_x.to_radians().tan();
        let tan_skew_y = skew_y.to_radians().tan();
        canvas.skew((tan_skew_x, tan_skew_y));

        // 6. Move back from pivot
        canvas.translate((-pivot_x, -pivot_y));

        let local_rect =
            skia_safe::Rect::from_wh(node.layout_rect.width(), node.layout_rect.height());

        let mut last_error = Ok(());
        let mut draw_children = |canvas: &skia_safe::Canvas| {
            // Z-Index Sorting
            let mut sorted_children: Vec<(NodeId, i32)> = Vec::with_capacity(node.children.len());
            for &child_id in &node.children {
                if let Some(child) = scene.get_node(child_id) {
                    sorted_children.push((child_id, child.z_index));
                }
            }
            sorted_children.sort_by_key(|k| k.1);

            for (child_id, _) in sorted_children {
                if let Err(e) =
                    render_recursive(scene, assets, child_id, canvas, parent_opacity, depth + 1)
                {
                    last_error = Err(e);
                }
            }
        };

        // Check if we need a save layer for blending or masking
        let need_layer =
            node.mask_node.is_some() || node.blend_mode != skia_safe::BlendMode::SrcOver;

        let result = if need_layer {
            let mut paint = skia_safe::Paint::default();
            paint.set_blend_mode(node.blend_mode);

            // Create an isolated layer.
            canvas.save_layer(&skia_safe::canvas::SaveLayerRec::default().paint(&paint));

            let r = node
                .element
                .render(canvas, local_rect, parent_opacity, &mut draw_children);

            if r.is_ok() && last_error.is_ok() {
                if let Some(mask_id) = node.mask_node {
                    let mut mask_paint = skia_safe::Paint::default();
                    mask_paint.set_blend_mode(skia_safe::BlendMode::DstIn);

                    canvas
                        .save_layer(&skia_safe::canvas::SaveLayerRec::default().paint(&mask_paint));
                    if let Err(e) = render_recursive(scene, assets, mask_id, canvas, 1.0, depth + 1)
                    {
                        last_error = Err(e);
                    }
                    canvas.restore();
                }
            }

            canvas.restore();
            r
        } else {
            node.element
                .render(canvas, local_rect, parent_opacity, &mut draw_children)
        };

        canvas.restore();

        result?;
        last_error?;
    }
    Ok(())
}

/// Renders a single frame at a specific timestamp to the provided canvas.
///
/// This is helpful for debugging or generating static previews without running the full export loop.
pub fn render_frame(
    director: &mut Director,
    time: f64,
    canvas: &skia_safe::Canvas,
) -> Result<(), RenderError> {
    let mut layout_engine = LayoutEngine::new();
    director.update(time);
    layout_engine.compute_layout(&mut director.scene, director.width, director.height, time);
    director.run_post_layout(time);

    let assets = &director.assets;

    canvas.clear(skia_safe::Color::BLACK);

    let mut items: Vec<(usize, TimelineItem)> = director
        .timeline
        .iter()
        .cloned()
        .enumerate()
        .filter(|(_, item)| time >= item.start_time && time < item.start_time + item.duration)
        .collect();
    items.sort_by_key(|(_, item)| item.z_index);

    for (_, item) in items {
        render_recursive(&director.scene, assets, item.scene_root, canvas, 1.0, 0)?;
    }
    Ok(())
}

/// Renders a frame at the given time, handling transitions between scenes.
///
/// Used internally by the export pipeline.
pub(crate) fn render_at_time(
    director: &mut Director,
    layout_engine: &mut LayoutEngine,
    time: f64,
    canvas: &skia_safe::Canvas,
    surfaces: &mut Option<(skia_safe::Surface, skia_safe::Surface)>,
) -> Result<(), RenderError> {
    director.update(time);
    layout_engine.compute_layout(&mut director.scene, director.width, director.height, time);
    director.run_post_layout(time);

    let assets_ref = &director.assets;

    canvas.clear(skia_safe::Color::BLACK);

    // Check transition
    let transition = director
        .transitions
        .iter()
        .find(|t| time >= t.start_time && time < t.start_time + t.duration)
        .cloned();

    // Collect items. Need indices to match with transition.
    let mut items: Vec<(usize, TimelineItem)> = director
        .timeline
        .iter()
        .cloned()
        .enumerate()
        .filter(|(_, item)| time >= item.start_time && time < item.start_time + item.duration)
        .collect();
    items.sort_by_key(|(_, item)| item.z_index);

    if let Some(trans) = transition {
        let mut drawn_transition = false;

        for (idx, item) in items {
            if idx == trans.from_scene_idx || idx == trans.to_scene_idx {
                if !drawn_transition {
                    if let Some((surf_a, surf_b)) = surfaces {
                        if let (Some(item_a), Some(item_b)) = (
                            director.timeline.get(trans.from_scene_idx),
                            director.timeline.get(trans.to_scene_idx),
                        ) {
                            surf_a.canvas().clear(skia_safe::Color::TRANSPARENT);
                            surf_b.canvas().clear(skia_safe::Color::TRANSPARENT);

                            render_recursive(
                                &director.scene,
                                assets_ref,
                                item_a.scene_root,
                                surf_a.canvas(),
                                1.0,
                                0,
                            )?;
                            render_recursive(
                                &director.scene,
                                assets_ref,
                                item_b.scene_root,
                                surf_b.canvas(),
                                1.0,
                                0,
                            )?;

                            let img_a = surf_a.image_snapshot();
                            let img_b = surf_b.image_snapshot();

                            let progress =
                                ((time - trans.start_time) / trans.duration).clamp(0.0, 1.0) as f32;
                            let val = trans.easing.eval(progress);

                            draw_transition(
                                canvas,
                                &img_a,
                                &img_b,
                                val,
                                &trans.kind,
                                director.width,
                                director.height,
                            );
                        }
                    }
                    drawn_transition = true;
                }
            } else {
                render_recursive(&director.scene, assets_ref, item.scene_root, canvas, 1.0, 0)?;
            }
        }
    } else {
        for (_, item) in items {
            render_recursive(&director.scene, assets_ref, item.scene_root, canvas, 1.0, 0)?;
        }
    }
    if time < 0.1 {
        debug!("[Frame] render complete");
    }
    Ok(())
}
