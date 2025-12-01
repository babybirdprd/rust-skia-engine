use anyhow::Result;
use std::path::PathBuf;
use crate::director::{Director, TimelineItem, TransitionType};
use crate::layout::LayoutEngine;
use crate::audio::load_audio_bytes;
use skia_safe::{ColorType, AlphaType, ColorSpace, RuntimeEffect, Data, runtime_effect::ChildPtr};
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

    if let Some(path) = audio_track_path {
        if let Ok(bytes) = std::fs::read(path) {
            if let Ok(samples) = load_audio_bytes(&bytes, director.audio_mixer.sample_rate) {
                director.add_global_audio(samples);
            }
        }
    }

    let destination: Locator = out_path.clone().into();
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

    let samples_per_frame = (director.audio_mixer.sample_rate as f64 / fps as f64).round() as usize;

    for i in 0..total_frames {
        let frame_start_time = i as f64 / fps as f64;
        let shutter_start_time = frame_start_time - (shutter_duration / 2.0);

        let render_at_time = |director: &mut Director, layout_engine: &mut LayoutEngine, time: f64, canvas: &skia_safe::Canvas| {
             director.update(time);
             layout_engine.compute_layout(director, time);
             director.run_post_layout(time);

             canvas.clear(skia_safe::Color::BLACK);

             // Check transition
             let transition = director.transitions.iter().find(|t| time >= t.start_time && time < t.start_time + t.duration).cloned();

             // Collect items. Need indices to match with transition.
             let mut items: Vec<(usize, TimelineItem)> = director.timeline.iter().cloned().enumerate()
                 .filter(|(_, item)| time >= item.start_time && time < item.start_time + item.duration)
                 .collect();
             items.sort_by_key(|(_, item)| item.z_index);

             if let Some(trans) = transition {
                 let mut drawn_transition = false;

                 for (idx, item) in items {
                     if idx == trans.from_scene_idx || idx == trans.to_scene_idx {
                         if !drawn_transition {
                             let info = skia_safe::ImageInfo::new(
                                (director.width, director.height),
                                ColorType::RGBA8888,
                                AlphaType::Premul,
                                Some(ColorSpace::new_srgb()),
                             );

                             if let (Some(mut surf_a), Some(mut surf_b)) = (skia_safe::surfaces::raster(&info, None, None), skia_safe::surfaces::raster(&info, None, None)) {
                                 // Render A & B
                                 if let (Some(item_a), Some(item_b)) = (director.timeline.get(trans.from_scene_idx), director.timeline.get(trans.to_scene_idx)) {
                                     render_recursive(director, item_a.scene_root, surf_a.canvas(), 1.0);
                                     render_recursive(director, item_b.scene_root, surf_b.canvas(), 1.0);

                                     let img_a = surf_a.image_snapshot();
                                     let img_b = surf_b.image_snapshot();

                                     let progress = ((time - trans.start_time) / trans.duration).clamp(0.0, 1.0) as f32;
                                     let val = trans.easing.eval(progress);

                                     draw_transition(canvas, &img_a, &img_b, val, &trans.kind, director.width, director.height);
                                 }
                             }
                             drawn_transition = true;
                         }
                     } else {
                         render_recursive(director, item.scene_root, canvas, 1.0);
                     }
                 }
             } else {
                 for (_, item) in items {
                     render_recursive(director, item.scene_root, canvas, 1.0);
                 }
             }
        };

        if samples == 1 {
            render_at_time(director, &mut layout_engine, frame_start_time, surface.canvas());
        } else {
             let scratch_surface = accumulation_surface.as_mut().unwrap();

             for s in 0..samples {
                 let t_offset = if samples > 1 {
                     (s as f64 / (samples - 1) as f64) * shutter_duration
                 } else {
                     0.0
                 };
                 let sample_time = shutter_start_time + t_offset;

                 // Clear handled by render_at_time
                 render_at_time(director, &mut layout_engine, sample_time, scratch_surface.canvas());

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

        let audio_samples = director.mix_audio(samples_per_frame, frame_start_time);
        encoder.encode_audio(&audio_samples, Time::from_secs_f64(frame_start_time))?;
    }

    encoder.finish()?;

    Ok(())
}

pub fn render_recursive(director: &Director, node_id: crate::director::NodeId, canvas: &skia_safe::Canvas, parent_opacity: f32) {
    if let Some(node) = director.get_node(node_id) {
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

         let local_rect = skia_safe::Rect::from_wh(node.layout_rect.width(), node.layout_rect.height());

         let mut draw_children = |canvas: &skia_safe::Canvas| {
             for child_id in &node.children {
                 render_recursive(director, *child_id, canvas, parent_opacity);
             }
         };

         // Check if we need a save layer for blending or masking
         let need_layer = node.mask_node.is_some() || node.blend_mode != skia_safe::BlendMode::SrcOver;

         if need_layer {
             let mut paint = skia_safe::Paint::default();
             paint.set_blend_mode(node.blend_mode);

             // Create an isolated layer.
             // If we have a mask, we need to ensure the content is drawn, then the mask is drawn with DstIn.
             // This layer acts as the composition group.
             canvas.save_layer(&skia_safe::canvas::SaveLayerRec::default().paint(&paint));

             node.element.render(canvas, local_rect, parent_opacity, &mut draw_children);

             if let Some(mask_id) = node.mask_node {
                 let mut mask_paint = skia_safe::Paint::default();
                 mask_paint.set_blend_mode(skia_safe::BlendMode::DstIn);

                 // Create a layer for the mask, which will composite onto the content with DstIn
                 canvas.save_layer(&skia_safe::canvas::SaveLayerRec::default().paint(&mask_paint));
                 render_recursive(director, mask_id, canvas, 1.0);
                 canvas.restore();
             }

             canvas.restore();
         } else {
             node.element.render(canvas, local_rect, parent_opacity, &mut draw_children);
         }

         canvas.restore();
    }
}

fn get_transition_shader(kind: &TransitionType) -> &'static str {
    match kind {
        TransitionType::Fade => r#"
            uniform shader imageA;
            uniform shader imageB;
            uniform float progress;
            half4 main(float2 p) {
                half4 colorA = imageA.eval(p);
                half4 colorB = imageB.eval(p);
                return mix(colorA, colorB, progress);
            }
        "#,
        TransitionType::SlideLeft => r#"
            uniform shader imageA;
            uniform shader imageB;
            uniform float progress;
            uniform float2 resolution;
            half4 main(float2 p) {
                float x_offset = resolution.x * progress;
                if (p.x < (resolution.x - x_offset)) {
                    return imageA.eval(float2(p.x + x_offset, p.y));
                } else {
                    return imageB.eval(float2(p.x - (resolution.x - x_offset), p.y));
                }
            }
        "#,
        TransitionType::SlideRight => r#"
            uniform shader imageA;
            uniform shader imageB;
            uniform float progress;
            uniform float2 resolution;
            half4 main(float2 p) {
                float x_offset = resolution.x * progress;
                if (p.x > x_offset) {
                    return imageA.eval(float2(p.x - x_offset, p.y));
                } else {
                    return imageB.eval(float2(p.x - x_offset + resolution.x, p.y));
                }
            }
        "#,
        TransitionType::WipeLeft => r#"
            uniform shader imageA;
            uniform shader imageB;
            uniform float progress;
            uniform float2 resolution;
            half4 main(float2 p) {
                 float boundary = resolution.x * (1.0 - progress);
                 if (p.x < boundary) {
                     return imageA.eval(p);
                 } else {
                     return imageB.eval(p);
                 }
            }
        "#,
        TransitionType::WipeRight => r#"
            uniform shader imageA;
            uniform shader imageB;
            uniform float progress;
            uniform float2 resolution;
            half4 main(float2 p) {
                 float boundary = resolution.x * progress;
                 if (p.x > boundary) {
                     return imageA.eval(p);
                 } else {
                     return imageB.eval(p);
                 }
            }
        "#,
        TransitionType::CircleOpen => r#"
            uniform shader imageA;
            uniform shader imageB;
            uniform float progress;
            uniform float2 resolution;
            half4 main(float2 p) {
                float2 center = resolution / 2.0;
                float max_radius = length(resolution);
                float current_radius = max_radius * progress;
                float dist = distance(p, center);
                if (dist < current_radius) {
                    return imageB.eval(p);
                } else {
                    return imageA.eval(p);
                }
            }
        "#,
    }
}

fn draw_transition(
    canvas: &skia_safe::Canvas,
    img_a: &skia_safe::Image,
    img_b: &skia_safe::Image,
    progress: f32,
    kind: &TransitionType,
    width: i32,
    height: i32,
) {
    let sksl = get_transition_shader(kind);
    let result = RuntimeEffect::make_for_shader(sksl, None);
    if let Ok(effect) = result {
         let mut uniform_bytes = Vec::new();
         uniform_bytes.extend_from_slice(&progress.to_le_bytes());

         match kind {
             TransitionType::Fade => {},
             _ => {
                 uniform_bytes.extend_from_slice(&(width as f32).to_le_bytes());
                 uniform_bytes.extend_from_slice(&(height as f32).to_le_bytes());
             }
         }

         let uniforms_data = Data::new_copy(&uniform_bytes);

         let shader_a = img_a.to_shader(None, skia_safe::SamplingOptions::default(), None).unwrap();
         let shader_b = img_b.to_shader(None, skia_safe::SamplingOptions::default(), None).unwrap();

         let children = [ChildPtr::Shader(shader_a), ChildPtr::Shader(shader_b)];

         if let Some(shader) = effect.make_shader(uniforms_data, &children, None) {
             let mut paint = skia_safe::Paint::default();
             paint.set_shader(Some(shader));
             canvas.draw_rect(skia_safe::Rect::from_wh(width as f32, height as f32), &paint);
         } else {
             eprintln!("Failed to make shader");
         }
    } else {
        eprintln!("Shader compilation error: {:?}", result.err());
    }
}

pub fn render_frame(director: &mut Director, time: f64, canvas: &skia_safe::Canvas) {
     let mut layout_engine = LayoutEngine::new();
     director.update(time);
     layout_engine.compute_layout(director, time);
     director.run_post_layout(time);

     canvas.clear(skia_safe::Color::BLACK);

     let mut items: Vec<(usize, TimelineItem)> = director.timeline.iter().cloned().enumerate()
         .filter(|(_, item)| time >= item.start_time && time < item.start_time + item.duration)
         .collect();
     items.sort_by_key(|(_, item)| item.z_index);

     for (_, item) in items {
         render_recursive(director, item.scene_root, canvas, 1.0);
     }
}
