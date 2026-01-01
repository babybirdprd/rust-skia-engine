//! # Transitions System
//!
//! Handles visual transitions between scenes using SkSL shaders.
//!
//! ## Responsibilities
//! - **Transition Types**: Enum of supported transition effects.
//! - **Shader Definitions**: GLSL/SkSL code for each transition.
//! - **Shader Execution**: Composites two scene images with transition effect.
//!
//! ## Key Types
//! - `TransitionType`: Fade, Slide, Wipe, CircleOpen variants.
//! - `Transition`: Defines a transition between two timeline scenes.

use crate::animation::EasingType;
use skia_safe::{runtime_effect::ChildPtr, Data, RuntimeEffect};
use tracing::error;

/// The type of visual transition between scenes.
#[derive(Clone, Debug)]
pub enum TransitionType {
    Fade,
    SlideLeft,
    SlideRight,
    WipeLeft,
    WipeRight,
    CircleOpen,
}

/// A definition of a transition between two scenes.
#[derive(Clone)]
pub struct Transition {
    pub from_scene_idx: usize,
    pub to_scene_idx: usize,
    pub start_time: f64,
    pub duration: f64,
    pub kind: TransitionType,
    pub easing: EasingType,
}

/// Returns the SkSL shader source for the given transition type.
pub fn get_transition_shader(kind: &TransitionType) -> &'static str {
    match kind {
        TransitionType::Fade => {
            r#"
            uniform shader imageA;
            uniform shader imageB;
            uniform float progress;
            half4 main(float2 p) {
                half4 colorA = imageA.eval(p);
                half4 colorB = imageB.eval(p);
                return mix(colorA, colorB, progress);
            }
        "#
        }
        TransitionType::SlideLeft => {
            r#"
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
        "#
        }
        TransitionType::SlideRight => {
            r#"
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
        "#
        }
        TransitionType::WipeLeft => {
            r#"
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
        "#
        }
        TransitionType::WipeRight => {
            r#"
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
        "#
        }
        TransitionType::CircleOpen => {
            r#"
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
        "#
        }
    }
}

/// Draws a transition effect between two images onto the canvas.
pub fn draw_transition(
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
            TransitionType::Fade => {}
            _ => {
                uniform_bytes.extend_from_slice(&(width as f32).to_le_bytes());
                uniform_bytes.extend_from_slice(&(height as f32).to_le_bytes());
            }
        }

        let uniforms_data = Data::new_copy(&uniform_bytes);

        let shader_a = img_a
            .to_shader(None, skia_safe::SamplingOptions::default(), None)
            .unwrap();
        let shader_b = img_b
            .to_shader(None, skia_safe::SamplingOptions::default(), None)
            .unwrap();

        let children = [ChildPtr::Shader(shader_a), ChildPtr::Shader(shader_b)];

        if let Some(shader) = effect.make_shader(uniforms_data, &children, None) {
            let mut paint = skia_safe::Paint::default();
            paint.set_shader(Some(shader));
            canvas.draw_rect(
                skia_safe::Rect::from_wh(width as f32, height as f32),
                &paint,
            );
        } else {
            error!("Failed to make shader");
        }
    } else {
        error!("Shader compilation error: {:?}", result.err());
    }
}
