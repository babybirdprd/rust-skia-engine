use anyhow::Result;
use std::path::PathBuf;
use crate::director::Director;
use crate::layout::LayoutEngine;
use skia_safe::{ColorType, AlphaType, ColorSpace};
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
    // Use `surfaces::raster` instead of deprecated `new_raster`
    let mut surface = skia_safe::surfaces::raster(&info, None, None).expect("Failed to create Skia surface");

    // Create accumulation surface for motion blur if needed
    // We only need this if samples > 1
    let mut accumulation_surface = if director.samples_per_frame > 1 {
        Some(skia_safe::surfaces::raster(&info, None, None).expect("Failed to create accumulation surface"))
    } else {
        None
    };

    let mut layout_engine = LayoutEngine::new();

    let samples = director.samples_per_frame.max(1);
    let shutter_angle = director.shutter_angle.clamp(0.0, 360.0);
    // Shutter interval in seconds. 360 degrees = 1 full frame duration.
    let frame_duration = 1.0 / fps as f64;
    let shutter_duration = frame_duration * (shutter_angle as f64 / 360.0);

    // 3. Frame Loop
    for i in 0..total_frames {
        let frame_start_time = i as f64 / fps as f64;
        // Center the shutter around the frame time? Or start at frame time?
        // Standard is often centered or starting. Let's assume centered for smoother look.
        // Actually, for "nth frame", it's usually the start of the frame.
        // Remotion/AfterEffects usually center the shutter around the current time.
        // Let's center it: start = time - duration/2.
        let shutter_start_time = frame_start_time - (shutter_duration / 2.0);

        if samples == 1 {
            // Simple Render Path
            director.update(frame_start_time);
            layout_engine.compute_layout(director);
            surface.canvas().clear(skia_safe::Color::BLACK);
            if let Some(root_id) = director.root_id {
                 render_recursive(director, root_id, surface.canvas(), 1.0);
            }
        } else {
            // Motion Blur Render Path
            if let Some(acc_surface) = &mut accumulation_surface {
                acc_surface.canvas().clear(skia_safe::Color::TRANSPARENT); // Start fresh for this frame?
                // Actually, we want to accumulate into `surface` (the destination) or `acc_surface`?
                // Let's use `surface` as the final output.
                // We will render each sample into `acc_surface` (scratch), then draw it onto `surface` (accumulator).
                // Wait, easier:
                // 1. Clear `surface` (Accumulator) to Black (or background).
                // 2. Loop samples.
                // 3. Render sample to `acc_surface` (Scratch).
                // 4. Draw `acc_surface` onto `surface` with alpha = 1.0 / sample_index?
                // No, standard averaging:
                // Accumulator = 0.
                // Loop: Accumulator += Sample * (1/N).
                // This requires 16-bit or float buffer for precision, OR the sequential blending trick.
                // Sequential blending trick:
                // Dest holds average of first k samples.
                // New sample S coming in.
                // New Dest = (Dest * k + S) / (k + 1) -> Dest * (k/k+1) + S * (1/k+1).
                // Blend mode: SrcOver?
                // If we draw S over Dest with alpha = 1/(k+1).
                // Result = S * alpha + Dest * (1 - alpha).
                // This matches exactly!

                // So:
                // `surface` is the Accumulator.
                // `acc_surface` is the Scratch buffer for one sample.

                // Clear Accumulator to TRANSPARENT initially if we want true averaging of content?
                // Or clear to BLACK?
                // If we clear to BLACK, the first sample (k=0) weight is 1.0.
                // Result = S0 * 1.0 + Black * 0.0 = S0.
                // Second sample (k=1) weight 0.5. Result = S1 * 0.5 + S0 * 0.5.
                // Correct.

                // NOTE: Using `acc_surface` as the scratchpad to render the scene.
                // `surface` is the final output.
            }

            // We need to unwrap the accumulation surface again or just use the one we created.
            // Using `surface` as accumulator.
            // We need a scratch surface.
             let scratch_surface = accumulation_surface.as_mut().unwrap();

             for s in 0..samples {
                 // Calculate time for this sample
                 // Linearly distribute samples across [shutter_start, shutter_start + shutter_duration]
                 // If samples=1, it should be exactly center? But we handled samples=1 above.
                 // For samples > 1:
                 let t_offset = if samples > 1 {
                     (s as f64 / (samples - 1) as f64) * shutter_duration
                 } else {
                     0.0
                 };
                 let sample_time = shutter_start_time + t_offset;

                 // A. Animation
                 director.update(sample_time);
                 // B. Layout
                 layout_engine.compute_layout(director);

                 // C. Render to Scratch
                 scratch_surface.canvas().clear(skia_safe::Color::TRANSPARENT);
                 // We likely want a background color? If the scene has no background, it's transparent.
                 // If we rely on the final clear, we might get weird alpha.
                 // Let's assume the scene *fills* the background or we want transparent background support.
                 // If we want black background by default:
                 scratch_surface.canvas().clear(skia_safe::Color::BLACK);

                 if let Some(root_id) = director.root_id {
                      render_recursive(director, root_id, scratch_surface.canvas(), 1.0);
                 }

                 // D. Accumulate Scratch -> Surface
                 // Weight = 1.0 / (s + 1)
                 let weight = 1.0 / (s as f32 + 1.0);

                 // If s == 0, weight is 1.0. We just overwrite `surface` effectively (or draw opaque on top).
                 // If `surface` was uninitialized, this fills it.
                 // We use SrcOver.
                 let mut paint = skia_safe::Paint::default();
                 paint.set_alpha_f(weight);
                 // We need to draw the snapshot of scratch_surface.
                 let image = scratch_surface.image_snapshot();

                 // If s == 0, we can just clear `surface` to black and draw?
                 // Or just draw with alpha 1.0.
                 if s == 0 {
                     surface.canvas().clear(skia_safe::Color::BLACK);
                 }

                 surface.canvas().draw_image(&image, (0, 0), Some(&paint));
             }
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
