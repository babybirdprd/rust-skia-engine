use anyhow::Result;
use std::path::PathBuf;
use crate::director::Director;
use crate::layout::LayoutEngine;
use skia_safe::{Surface, ColorType, AlphaType, ColorSpace};
use crate::video_wrapper::{Encoder, EncoderSettings, Locator, Time};
use ndarray::Array3;

pub fn render_export(director: &mut Director, out_path: PathBuf) -> Result<()> {
    let width = director.width;
    let height = director.height;
    let fps = director.fps;
    let duration = 5.0; // Hardcoded for now, should come from Director/Scene
    let total_frames = (duration * fps as f64) as usize;

    // 1. Initialize Encoder
    let destination: Locator = out_path.into();
    let settings = EncoderSettings::for_h264_yuv420p(width as usize, height as usize, false);

    let mut encoder = Encoder::new(&destination, settings)?;

    // 2. Initialize Skia Surface
    let info = skia_safe::ImageInfo::new(
        (width, height),
        ColorType::RGBA8888,
        AlphaType::Premul,
        Some(ColorSpace::new_srgb()),
    );
    let mut surface = Surface::new_raster(&info, None, None).expect("Failed to create Skia surface");

    let mut layout_engine = LayoutEngine::new();

    // 3. Frame Loop
    for i in 0..total_frames {
        let time = i as f64 / fps as f64;

        // A. Animation Step
        director.update(time);

        // B. Layout Step
        layout_engine.compute_layout(director);

        // C. Render Step
        surface.canvas().clear(skia_safe::Color::BLACK);

        if let Some(root_id) = director.root_id {
             render_recursive(director, root_id, surface.canvas(), 1.0);
        }

        // D. Encode Step
        if let Some(pixmap) = surface.peek_pixels() {
             if let Some(bytes) = pixmap.bytes() {
                 let frame_shape = (height as usize, width as usize, 4);
                 if bytes.len() == width as usize * height as usize * 4 {
                     // Unwrap bytes safely (it's slice, to_vec works)
                     let vec_bytes = bytes.to_vec();
                     let frame = Array3::from_shape_vec(frame_shape, vec_bytes)?;

                     encoder.encode(&frame, &Time::from_nth_of_second(i, fps))?;
                 }
             }
        }
    }

    encoder.finish()?;
    Ok(())
}

fn render_recursive(director: &Director, node_id: crate::director::NodeId, canvas: &skia_safe::Canvas, parent_opacity: f32) {
    if let Some(node) = director.get_node(node_id) {
         canvas.save();

         canvas.translate((node.layout_rect.left, node.layout_rect.top));

         let local_rect = skia_safe::Rect::from_wh(node.layout_rect.width(), node.layout_rect.height());
         node.element.render(canvas, local_rect, parent_opacity);

         for child_id in &node.children {
             render_recursive(director, *child_id, canvas, parent_opacity);
         }

         canvas.restore();
    }
}
