use crate::animation::{Animated, TweenableVector};
use crate::element::Element;
use crate::errors::RenderError;
use crate::node::parse_easing;
use crate::types::Color;
use skia_safe::{
    color_filters, image_filters, runtime_effect::RuntimeShaderBuilder, Canvas, ColorMatrix, Paint,
    Rect, RuntimeEffect, TileMode,
};
use std::any::Any;
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};
use taffy::style::Style;
use tracing::error;

/// Represents an animatable uniform value for Runtime Shaders.
#[derive(Debug, Clone)]
pub enum ShaderUniform {
    Float(Animated<f32>),
    Vec(Animated<TweenableVector>),
}

impl ShaderUniform {
    pub fn update(&mut self, time: f64) {
        match self {
            ShaderUniform::Float(a) => a.update(time),
            ShaderUniform::Vec(a) => a.update(time),
        }
    }
}

/// Available visual effects that can be applied to nodes.
#[derive(Debug, Clone)]
pub enum EffectType {
    /// Gaussian Blur with animated sigma (radius).
    Blur(Animated<f32>),
    /// 4x5 Color Matrix transform (e.g. Grayscale, Sepia).
    ColorMatrix(Vec<f32>),
    /// Custom SkSL shader with animated uniforms.
    RuntimeShader {
        sksl: String,
        uniforms: HashMap<String, ShaderUniform>,
    },
    /// Drop Shadow effect.
    DropShadow {
        blur: Animated<f32>,
        offset_x: Animated<f32>,
        offset_y: Animated<f32>,
        color: Animated<Color>,
    },
    /// Directional blur (motion blur) along an angle.
    DirectionalBlur {
        /// Blur strength (pixel distance)
        strength: Animated<f32>,
        /// Direction angle in degrees (0 = right, 90 = down)
        angle: Animated<f32>,
        /// Number of samples (quality vs performance, clamped 4-64)
        samples: u32,
    },
    /// Film grain / noise overlay for cinematic look.
    FilmGrain {
        /// Grain intensity (0.0 - 1.0)
        intensity: Animated<f32>,
        /// Grain size/scale (pixels)
        size: Animated<f32>,
    },
}

impl EffectType {
    pub fn update(&mut self, time: f64) {
        match self {
            EffectType::Blur(a) => a.update(time),
            EffectType::ColorMatrix(_) => {}
            EffectType::RuntimeShader { uniforms, .. } => {
                for val in uniforms.values_mut() {
                    val.update(time);
                }
            }
            EffectType::DropShadow {
                blur,
                offset_x,
                offset_y,
                color,
            } => {
                blur.update(time);
                offset_x.update(time);
                offset_y.update(time);
                color.update(time);
            }
            EffectType::DirectionalBlur {
                strength, angle, ..
            } => {
                strength.update(time);
                angle.update(time);
            }
            EffectType::FilmGrain { intensity, size } => {
                intensity.update(time);
                size.update(time);
            }
        }
    }
}

pub fn build_effect_filter(
    effects: &[EffectType],
    shader_cache: Option<&Arc<Mutex<HashMap<String, RuntimeEffect>>>>,
    resolution: (f32, f32),
    time: f32,
) -> Option<skia_safe::ImageFilter> {
    let mut current_filter = None;
    for effect in effects {
        match effect {
            EffectType::Blur(sigma) => {
                current_filter = image_filters::blur(
                    (sigma.current_value, sigma.current_value),
                    TileMode::Decal,
                    current_filter,
                    None,
                );
            }
            EffectType::DropShadow {
                blur,
                offset_x,
                offset_y,
                color,
            } => {
                current_filter = image_filters::drop_shadow(
                    (offset_x.current_value, offset_y.current_value),
                    (blur.current_value, blur.current_value),
                    color.current_value.to_skia(),
                    None,
                    current_filter,
                    None,
                );
            }
            EffectType::ColorMatrix(matrix) => {
                if matrix.len() == 20 {
                    if let Ok(m) = matrix.as_slice().try_into() {
                        let m: &[f32; 20] = m;
                        let cm = ColorMatrix::new(
                            m[0], m[1], m[2], m[3], m[4], m[5], m[6], m[7], m[8], m[9], m[10],
                            m[11], m[12], m[13], m[14], m[15], m[16], m[17], m[18], m[19],
                        );
                        let cf = color_filters::matrix(&cm, None);
                        current_filter = image_filters::color_filter(cf, current_filter, None);
                    }
                }
            }
            EffectType::RuntimeShader { sksl, uniforms } => {
                if let Some(cache_arc) = shader_cache {
                    let mut cache = cache_arc.lock().unwrap();
                    if !cache.contains_key(sksl) {
                        match RuntimeEffect::make_for_shader(sksl, None) {
                            Ok(effect) => {
                                cache.insert(sksl.clone(), effect);
                            }
                            Err(e) => {
                                error!("Shader compilation error: {}", e);
                                continue;
                            }
                        }
                    }

                    if let Some(effect) = cache.get(sksl) {
                        let mut builder = RuntimeShaderBuilder::new(effect.clone());

                        // Inject Automatic Uniforms
                        let _ = builder
                            .set_uniform_float("u_resolution", &[resolution.0, resolution.1]);
                        let _ = builder.set_uniform_float("u_time", &[time]);

                        for (key, val) in uniforms {
                            match val {
                                ShaderUniform::Float(anim) => {
                                    let _ = builder.set_uniform_float(key, &[anim.current_value]);
                                }
                                ShaderUniform::Vec(anim) => {
                                    let vec_data = &anim.current_value.0;
                                    let _ = builder.set_uniform_float(key, vec_data);
                                }
                            }
                        }
                        // "image" is the standard name for the input texture in SkSL for ImageFilters
                        current_filter =
                            image_filters::runtime_shader(&builder, "image", current_filter);
                    }
                }
            }
            EffectType::DirectionalBlur {
                strength,
                angle,
                samples: _,
            } => {
                // Use fixed 16 samples for simplicity and compatibility
                // (SkSL has restrictions on dynamic loop bounds)
                let sksl = r#"
uniform shader image;
uniform float2 u_resolution;
uniform float u_strength;
uniform float u_angle;

half4 main(float2 pos) {
    float rad = radians(u_angle);
    float2 dir = float2(cos(rad), sin(rad));
    
    half4 color = half4(0.0);
    // Fixed 16 samples for motion blur
    for (int i = 0; i < 16; i++) {
        float t = (float(i) - 7.5) / 8.0;  // Range: -0.9375 to +0.9375
        float2 offset = dir * u_strength * t;
        color += image.eval(pos + offset);
    }
    return color / 16.0;
}
"#;
                if let Some(cache_arc) = shader_cache {
                    let cache_key = "__directional_blur__".to_string();
                    let mut cache = cache_arc.lock().unwrap();
                    if !cache.contains_key(&cache_key) {
                        match RuntimeEffect::make_for_shader(sksl, None) {
                            Ok(effect) => {
                                cache.insert(cache_key.clone(), effect);
                            }
                            Err(e) => {
                                error!("DirectionalBlur shader compilation error: {}", e);
                                continue;
                            }
                        }
                    }
                    if let Some(effect) = cache.get(&cache_key) {
                        let mut builder = RuntimeShaderBuilder::new(effect.clone());
                        let _ = builder
                            .set_uniform_float("u_resolution", &[resolution.0, resolution.1]);
                        let _ = builder.set_uniform_float("u_strength", &[strength.current_value]);
                        let _ = builder.set_uniform_float("u_angle", &[angle.current_value]);
                        current_filter =
                            image_filters::runtime_shader(&builder, "image", current_filter);
                    }
                }
            }
            EffectType::FilmGrain { intensity, size } => {
                let sksl = r#"
uniform shader image;
uniform float2 u_resolution;
uniform float u_time;
uniform float u_intensity;
uniform float u_size;

float hash(float2 p) {
    return fract(sin(dot(p, float2(127.1, 311.7))) * 43758.5453);
}

half4 main(float2 pos) {
    half4 color = image.eval(pos);
    
    // Animated grain based on time and position
    float2 grain_pos = floor(pos / u_size) + floor(u_time * 24.0);
    float grain = hash(grain_pos) * 2.0 - 1.0;
    
    color.rgb += half3(grain * u_intensity);
    return color;
}
"#;
                if let Some(cache_arc) = shader_cache {
                    let cache_key = "__film_grain__".to_string();
                    let mut cache = cache_arc.lock().unwrap();
                    if !cache.contains_key(&cache_key) {
                        match RuntimeEffect::make_for_shader(sksl, None) {
                            Ok(effect) => {
                                cache.insert(cache_key.clone(), effect);
                            }
                            Err(e) => {
                                error!("FilmGrain shader compilation error: {}", e);
                                continue;
                            }
                        }
                    }
                    if let Some(effect) = cache.get(&cache_key) {
                        let mut builder = RuntimeShaderBuilder::new(effect.clone());
                        let _ = builder
                            .set_uniform_float("u_resolution", &[resolution.0, resolution.1]);
                        let _ = builder.set_uniform_float("u_time", &[time]);
                        let _ =
                            builder.set_uniform_float("u_intensity", &[intensity.current_value]);
                        let _ = builder.set_uniform_float("u_size", &[size.current_value]);
                        current_filter =
                            image_filters::runtime_shader(&builder, "image", current_filter);
                    }
                }
            }
        }
    }
    current_filter
}

/// A node that applies visual effects (filters) to its children.
///
/// It does not render content itself but wraps its children in a layer with applied image filters.
pub struct EffectNode {
    pub effects: Vec<EffectType>,
    pub style: Style,
    pub shader_cache: Arc<Mutex<HashMap<String, RuntimeEffect>>>,
    pub current_time: f32,
}

impl Clone for EffectNode {
    fn clone(&self) -> Self {
        Self {
            effects: self.effects.clone(),
            style: self.style.clone(),
            shader_cache: self.shader_cache.clone(),
            current_time: self.current_time,
        }
    }
}

impl fmt::Debug for EffectNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EffectNode")
            .field("effects", &self.effects)
            .finish()
    }
}

impl Element for EffectNode {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn layout_style(&self) -> Style {
        self.style.clone()
    }

    fn update(&mut self, time: f64) -> bool {
        self.current_time = time as f32;
        for effect in &mut self.effects {
            effect.update(time);
        }
        true
    }

    fn render(
        &self,
        canvas: &Canvas,
        rect: Rect,
        opacity: f32,
        draw_children: &mut dyn FnMut(&Canvas),
    ) -> Result<(), RenderError> {
        let resolution = (rect.width(), rect.height());
        let filter = build_effect_filter(
            &self.effects,
            Some(&self.shader_cache),
            resolution,
            self.current_time,
        );

        let mut paint = Paint::default();
        paint.set_alpha_f(opacity);
        if let Some(f) = filter {
            paint.set_image_filter(f);
        }

        // Do not restrict bounds to rect, otherwise shadows/blurs are clipped
        canvas.save_layer(&skia_safe::canvas::SaveLayerRec::default().paint(&paint));
        draw_children(canvas);
        canvas.restore();
        Ok(())
    }

    fn animate_property(
        &mut self,
        property: &str,
        start: f32,
        target: f32,
        duration: f64,
        easing: &str,
    ) {
        let ease_fn = parse_easing(easing);
        for effect in &mut self.effects {
            if let EffectType::RuntimeShader { uniforms, .. } = effect {
                if let Some(anim) = uniforms.get_mut(property) {
                    if let ShaderUniform::Float(a) = anim {
                        a.add_segment(start, target, duration, ease_fn);
                    }
                }
            }
        }
    }

    fn animate_property_spring(
        &mut self,
        property: &str,
        start: Option<f32>,
        target: f32,
        config: crate::animation::SpringConfig,
    ) {
        for effect in &mut self.effects {
            if let EffectType::RuntimeShader { uniforms, .. } = effect {
                if let Some(anim) = uniforms.get_mut(property) {
                    if let ShaderUniform::Float(a) = anim {
                        if let Some(s) = start {
                            a.add_spring_with_start(s, target, config);
                        } else {
                            a.add_spring(target, config);
                        }
                    }
                }
            }
        }
    }
}
