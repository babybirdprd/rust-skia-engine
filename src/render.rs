use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;
use crate::director::{Director, TimelineItem};
use crate::layout::LayoutEngine;
use skia_safe::{ColorType, AlphaType, ColorSpace};
use crate::video_wrapper::{Encoder, EncoderSettings, Locator, Time};
use ndarray::Array3;

#[cfg(feature = "vulkan")]
use skia_safe::gpu::{DirectContext, SurfaceOrigin, Budgeted};

#[cfg(feature = "vulkan")]
pub type GpuContext = DirectContext;
#[cfg(not(feature = "vulkan"))]
pub type GpuContext = ();

#[allow(unused_variables)]
pub fn render_export(director: &mut Director, out_path: PathBuf, gpu_context: Option<&mut GpuContext>, audio_track_path: Option<PathBuf>) -> Result<()> {
    let width = director.width;
    let height = director.height;
    let fps = director.fps;

    let mut max_duration = 0.0;
    for item in &director.timeline {
        let end = item.start_time + item.duration;
        if end > max_duration { max_duration = end; }
    }
    if max_duration == 0.0 { max_duration = 5.0; }

    let total_frames = (max_duration * fps as f64).ceil() as usize;

    // If we have audio, render video to a temporary file first
    let video_out_path = if audio_track_path.is_some() {
        out_path.with_extension("temp.mp4")
    } else {
        out_path.clone()
    };

    let destination: Locator = video_out_path.clone().into();
    let settings = EncoderSettings::preset_h264_yuv420p(width as usize, height as usize, false);

    let mut encoder = Encoder::new(&destination, settings)?;

    let info = skia_safe::ImageInfo::new(
        (width, height),
        ColorType::RGBA8888,
        AlphaType::Premul,
        Some(ColorSpace::new_srgb()),
    );

    #[allow(unused_mut)]
    let mut surface = None;

    #[cfg(feature = "vulkan")]
    if let Some(ctx) = gpu_context {
        surface = skia_safe::surfaces::render_target(
            ctx,
            Budgeted::Yes,
            &info,
            0,
            SurfaceOrigin::TopLeft,
            None,
            false,
            None,
        );
    }

    let mut surface = surface.or_else(|| {
        skia_safe::surfaces::raster(&info, None, None)
    }).expect("Failed to create Skia surface");

    let mut accumulation_surface = if director.samples_per_frame > 1 {
        Some(surface.new_surface(&info).expect("Failed to create accumulation surface"))
    } else {
        None
    };

    let mut layout_engine = LayoutEngine::new();

    let samples = director.samples_per_frame.max(1);
    let shutter_angle = director.shutter_angle.clamp(0.0, 360.0);
    let frame_duration = 1.0 / fps as f64;
    let shutter_duration = frame_duration * (shutter_angle as f64 / 360.0);

    for i in 0..total_frames {
        let frame_start_time = i as f64 / fps as f64;
        let shutter_start_time = frame_start_time - (shutter_duration / 2.0);

        if samples == 1 {
            director.update(frame_start_time);
            layout_engine.compute_layout(director, frame_start_time);

            surface.canvas().clear(skia_safe::Color::BLACK);

            let mut items: Vec<&TimelineItem> = director.timeline.iter()
                .filter(|item| frame_start_time >= item.start_time && frame_start_time < item.start_time + item.duration)
                .collect();
            items.sort_by_key(|item| item.z_index);

            for item in items {
                 render_recursive(director, item.scene_root, surface.canvas(), 1.0);
            }
        } else {
             let scratch_surface = accumulation_surface.as_mut().unwrap();

             for s in 0..samples {
                 let t_offset = if samples > 1 {
                     (s as f64 / (samples - 1) as f64) * shutter_duration
                 } else {
                     0.0
                 };
                 let sample_time = shutter_start_time + t_offset;

                 director.update(sample_time);
                 layout_engine.compute_layout(director, sample_time);

                 scratch_surface.canvas().clear(skia_safe::Color::BLACK);

                 let mut items: Vec<&TimelineItem> = director.timeline.iter()
                    .filter(|item| sample_time >= item.start_time && sample_time < item.start_time + item.duration)
                    .collect();
                 items.sort_by_key(|item| item.z_index);

                 for item in items {
                      render_recursive(director, item.scene_root, scratch_surface.canvas(), 1.0);
                 }

                 let weight = 1.0 / (s as f32 + 1.0);
                 let mut paint = skia_safe::Paint::default();
                 paint.set_alpha_f(weight);
                 let image = scratch_surface.image_snapshot();

                 if s == 0 {
                     surface.canvas().clear(skia_safe::Color::BLACK);
                 }
                 surface.canvas().draw_image(&image, (0, 0), Some(&paint));
             }
        }

        if let Some(pixmap) = surface.peek_pixels() {
             if let Some(bytes) = pixmap.bytes() {
                 let frame_shape = (height as usize, width as usize, 4);
                 // Safety: check size
                 if bytes.len() == width as usize * height as usize * 4 {
                    let vec_bytes = bytes.to_vec();
                    let frame = Array3::from_shape_vec(frame_shape, vec_bytes)?;
                    encoder.encode(&frame, Time::from_secs_f64(i as f64 / fps as f64))?;
                 }
             }
        } else {
             let mut bytes = Vec::with_capacity((width * height * 4) as usize);
             bytes.resize((width * height * 4) as usize, 0);
              let info = skia_safe::ImageInfo::new(
                (width, height),
                ColorType::RGBA8888,
                AlphaType::Premul,
                None
             );
             if surface.read_pixels(&info, &mut bytes, (width * 4) as usize, (0, 0)) {
                 let frame_shape = (height as usize, width as usize, 4);
                 let frame = Array3::from_shape_vec(frame_shape, bytes)?;
                 encoder.encode(&frame, Time::from_secs_f64(i as f64 / fps as f64))?;
             }
        }
    }

    encoder.finish()?;

    if let Some(audio_path) = audio_track_path {
        println!("Muxing audio...");
        let status = Command::new("ffmpeg")
            .arg("-y")
            .arg("-i").arg(&video_out_path)
            .arg("-i").arg(&audio_path)
            .arg("-c:v").arg("copy")
            .arg("-c:a").arg("aac")
            .arg("-shortest") // Stop when video ends
            .arg(&out_path)
            .status()?;

        if !status.success() {
            eprintln!("FFmpeg muxing failed");
        } else {
            // Clean up temp file
            let _ = std::fs::remove_file(video_out_path);
        }
    }

    Ok(())
}

fn render_recursive(director: &Director, node_id: crate::director::NodeId, canvas: &skia_safe::Canvas, parent_opacity: f32) {
    if let Some(node) = director.get_node(node_id) {
         canvas.save();

         let dx = node.layout_rect.left + node.translation.0;
         let dy = node.layout_rect.top + node.translation.1;

         canvas.translate((dx, dy));

         let local_rect = skia_safe::Rect::from_wh(node.layout_rect.width(), node.layout_rect.height());
         node.element.render(canvas, local_rect, parent_opacity);

         for child_id in &node.children {
             render_recursive(director, *child_id, canvas, parent_opacity);
         }

         canvas.restore();
    }
}
