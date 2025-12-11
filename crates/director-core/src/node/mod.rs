use crate::animation::{Animated, EasingType, TweenableVector};
use crate::director::Director;
use crate::element::Element;
use crate::systems::layout::LayoutEngine;
use crate::systems::renderer::render_recursive;
use crate::types::{Color, ObjectFit};
use skia_safe::{
    color_filters,
    image_filters,
    runtime_effect::RuntimeShaderBuilder,
    AlphaType, Canvas, ClipOp, Color4f, ColorMatrix, ColorType, Data, Image,
    Paint, PaintStyle, Rect, RuntimeEffect, Surface, TileMode,
    RRect,
};
use std::any::Any;
use std::collections::HashMap;
use std::fmt;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use taffy::style::Style;
use tempfile::NamedTempFile;

// Video imports
use crate::video_wrapper::{VideoLoader, RenderMode, VideoResponse};

pub mod text;
pub use text::TextNode;

/// Specifies the data source for a video node.
pub enum VideoSource {
    /// Load from a local file path.
    Path(PathBuf),
    /// Load from raw bytes (in-memory).
    Bytes(Vec<u8>),
}

// Helper to parse easing
pub fn parse_easing(e: &str) -> EasingType {
    match e {
        "linear" => EasingType::Linear,
        "ease_in" => EasingType::EaseIn,
        "ease_out" => EasingType::EaseOut,
        "ease_in_out" => EasingType::EaseInOut,
        "bounce_out" => EasingType::BounceOut,
        _ => EasingType::Linear,
    }
}

fn calculate_object_fit_rect(
    src_w: f32,
    src_h: f32,
    dst_rect: Rect,
    fit: ObjectFit,
) -> Rect {
    match fit {
        ObjectFit::Fill => dst_rect,
        ObjectFit::Contain | ObjectFit::Cover => {
            let src_ratio = src_w / src_h;
            let dst_w = dst_rect.width();
            let dst_h = dst_rect.height();
            let dst_ratio = dst_w / dst_h;

            let scale = match fit {
                ObjectFit::Contain => {
                    if src_ratio > dst_ratio {
                        dst_w / src_w
                    } else {
                        dst_h / src_h
                    }
                }
                ObjectFit::Cover => {
                    if src_ratio > dst_ratio {
                        dst_h / src_h
                    } else {
                        dst_w / src_w
                    }
                }
                _ => 1.0,
            };

            let new_w = src_w * scale;
            let new_h = src_h * scale;
            let new_x = dst_rect.left + (dst_w - new_w) / 2.0;
            let new_y = dst_rect.top + (dst_h - new_h) / 2.0;

            Rect::from_xywh(new_x, new_y, new_w, new_h)
        }
    }
}

// --- Effect Node & Types ---

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
        }
    }
}

fn build_effect_filter(
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
                                eprintln!("Shader compilation error: {}", e);
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
    ) {
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

// --- Box Node ---
/// A fundamental layout and styling block (div-like).
///
/// Supports background color, borders, shadows, and rounded corners.
#[derive(Debug, Clone)]
pub struct BoxNode {
    pub style: Style,
    pub bg_color: Option<Animated<Color>>,
    pub opacity: Animated<f32>,
    pub blur: Animated<f32>,
    pub shadow_color: Option<Animated<Color>>,
    pub shadow_blur: Animated<f32>,
    pub shadow_offset_x: Animated<f32>,
    pub shadow_offset_y: Animated<f32>,
    // New fields
    pub border_radius: Animated<f32>,
    pub border_width: Animated<f32>,
    pub border_color: Option<Animated<Color>>,
    pub overflow: String,
}

impl BoxNode {
    pub fn new() -> Self {
        Self {
            style: Style::default(),
            bg_color: None,
            opacity: Animated::new(1.0),
            blur: Animated::new(0.0),
            shadow_color: None,
            shadow_blur: Animated::new(0.0),
            shadow_offset_x: Animated::new(0.0),
            shadow_offset_y: Animated::new(0.0),
            border_radius: Animated::new(0.0),
            border_width: Animated::new(0.0),
            border_color: None,
            overflow: "visible".to_string(),
        }
    }
}

impl Element for BoxNode {
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
        let mut changed = false;
        if let Some(bg) = &mut self.bg_color {
            bg.update(time);
            changed = true;
        }
        if let Some(sc) = &mut self.shadow_color {
            sc.update(time);
            changed = true;
        }
        if let Some(bc) = &mut self.border_color {
            bc.update(time);
            changed = true;
        }
        self.opacity.update(time);
        self.blur.update(time);
        self.shadow_blur.update(time);
        self.shadow_offset_x.update(time);
        self.shadow_offset_y.update(time);
        self.border_radius.update(time);
        self.border_width.update(time);
        changed
    }

    fn render(
        &self,
        canvas: &Canvas,
        rect: Rect,
        opacity: f32,
        draw_children: &mut dyn FnMut(&Canvas),
    ) {
        let local_opacity = self.opacity.current_value * opacity;
        let radius = self.border_radius.current_value;
        let rrect = RRect::new_rect_xy(&rect, radius, radius);

        canvas.save();

        if self.overflow == "hidden" {
            canvas.clip_rrect(rrect, ClipOp::Intersect, true);
        }

        let mut paint = Paint::default();
        paint.set_anti_alias(true);

        let mut effects = Vec::new();
        if self.blur.current_value > 0.0 {
            effects.push(EffectType::Blur(self.blur.clone()));
        }
        if let Some(sc) = &self.shadow_color {
            effects.push(EffectType::DropShadow {
                blur: self.shadow_blur.clone(),
                offset_x: self.shadow_offset_x.clone(),
                offset_y: self.shadow_offset_y.clone(),
                color: sc.clone(),
            });
        }

        // BoxNode effects don't use RuntimeShader for now, so we pass dummy resolution/time
        // Or we could pass proper ones if we wanted to support shaders on BoxNode later.
        // For now, these effects (Blur, DropShadow) ignore resolution/time.
        let filter = build_effect_filter(&effects, None, (rect.width(), rect.height()), 0.0);
        if let Some(f) = filter {
            paint.set_image_filter(f);
        }

        if let Some(bg) = &self.bg_color {
            let mut c = bg.current_value;
            c.a *= local_opacity;
            paint.set_color4f(c.to_color4f(), None);
            canvas.draw_rrect(rrect, &paint);
        }

        // Draw children (clipped if overflow: hidden)
        draw_children(canvas);

        canvas.restore(); // Restore clip and transform (except we didn't transform here, but clip)

        // Draw Border
        let bw = self.border_width.current_value;
        if bw > 0.0 {
            let mut border_paint = Paint::default();
            border_paint.set_anti_alias(true);
            border_paint.set_style(PaintStyle::Stroke);
            border_paint.set_stroke_width(bw);

            let color = if let Some(bc) = &self.border_color {
                bc.current_value
            } else {
                Color::BLACK
            };

            let mut c = color;
            c.a *= local_opacity;
            border_paint.set_color4f(c.to_color4f(), None);

            canvas.draw_rrect(rrect, &border_paint);
        }
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
        match property {
            "opacity" => self.opacity.add_segment(start, target, duration, ease_fn),
            "blur" => self.blur.add_segment(start, target, duration, ease_fn),
            "shadow_blur" => self
                .shadow_blur
                .add_segment(start, target, duration, ease_fn),
            "shadow_x" => self
                .shadow_offset_x
                .add_segment(start, target, duration, ease_fn),
            "shadow_y" => self
                .shadow_offset_y
                .add_segment(start, target, duration, ease_fn),
            "border_radius" => self
                .border_radius
                .add_segment(start, target, duration, ease_fn),
            "border_width" => self
                .border_width
                .add_segment(start, target, duration, ease_fn),
            _ => {}
        }
    }

    fn animate_property_spring(
        &mut self,
        property: &str,
        start: Option<f32>,
        target: f32,
        config: crate::animation::SpringConfig,
    ) {
        let apply = |anim: &mut crate::animation::Animated<f32>| {
            if let Some(s) = start {
                anim.add_spring_with_start(s, target, config);
            } else {
                anim.add_spring(target, config);
            }
        };

        match property {
            "opacity" => apply(&mut self.opacity),
            "blur" => apply(&mut self.blur),
            "shadow_blur" => apply(&mut self.shadow_blur),
            "shadow_x" => apply(&mut self.shadow_offset_x),
            "shadow_y" => apply(&mut self.shadow_offset_y),
            "border_radius" => apply(&mut self.border_radius),
            "border_width" => apply(&mut self.border_width),
            _ => {}
        }
    }
}

// --- Image Node ---
/// A node that renders a static raster image (PNG, JPG, etc.).
#[derive(Debug, Clone)]
pub struct ImageNode {
    pub image: Option<Image>,
    pub opacity: Animated<f32>,
    pub style: Style,
    pub object_fit: ObjectFit,
}

impl ImageNode {
    pub fn new(data: Vec<u8>) -> Self {
        let image = Image::from_encoded(Data::new_copy(&data));
        Self {
            image,
            opacity: Animated::new(1.0),
            style: Style::DEFAULT,
            object_fit: ObjectFit::Cover,
        }
    }
}

impl Element for ImageNode {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn layout_style(&self) -> Style {
        self.style.clone()
    }

    fn set_layout_style(&mut self, style: Style) {
        self.style = style;
    }

    fn update(&mut self, time: f64) -> bool {
        self.opacity.update(time);
        true
    }

    fn render(
        &self,
        canvas: &Canvas,
        rect: Rect,
        parent_opacity: f32,
        draw_children: &mut dyn FnMut(&Canvas),
    ) {
        let op = self.opacity.current_value * parent_opacity;
        let mut paint = Paint::new(Color4f::new(1.0, 1.0, 1.0, op), None);
        paint.set_anti_alias(true);

        if let Some(img) = &self.image {
            let sampling = skia_safe::SamplingOptions::new(
                skia_safe::FilterMode::Linear,
                skia_safe::MipmapMode::Linear,
            );

            let draw_rect = calculate_object_fit_rect(
                img.width() as f32,
                img.height() as f32,
                rect,
                self.object_fit,
            );

            canvas.save();
            if self.object_fit == ObjectFit::Cover {
                canvas.clip_rect(rect, ClipOp::Intersect, true);
            }
            canvas.draw_image_rect_with_sampling_options(
                img, None, draw_rect, sampling, &paint,
            );
            canvas.restore();
        }
        draw_children(canvas);
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
        if property == "opacity" {
            self.opacity.add_segment(start, target, duration, ease_fn);
        }
    }

    fn animate_property_spring(
        &mut self,
        property: &str,
        start: Option<f32>,
        target: f32,
        config: crate::animation::SpringConfig,
    ) {
        if property == "opacity" {
            if let Some(s) = start {
                self.opacity.add_spring_with_start(s, target, config);
            } else {
                self.opacity.add_spring(target, config);
            }
        }
    }
}

// --- Video Node ---
/// A node that plays a video file.
///
/// Handles async decoding and frame buffering.
#[derive(Debug)]
pub struct VideoNode {
    pub opacity: Animated<f32>,
    pub style: Style,
    pub object_fit: ObjectFit,
    current_frame: Mutex<Option<(f64, Image)>>,

    loader: Option<VideoLoader>,
    render_mode: RenderMode,

    // Keep temp file alive
    #[allow(dead_code)]
    temp_file: Option<Arc<NamedTempFile>>,
    // Also keep path for cloning if it was a file path
    path: PathBuf,
}

impl Clone for VideoNode {
    fn clone(&self) -> Self {
        let loader = if self.loader.is_some() {
            // Create new loader pointing to same file.
            VideoLoader::new(self.path.clone(), self.render_mode).ok()
        } else {
            None
        };

        Self {
            opacity: self.opacity.clone(),
            style: self.style.clone(),
            object_fit: self.object_fit,
            current_frame: Mutex::new(None),
            loader,
            render_mode: self.render_mode,
            temp_file: self.temp_file.clone(),
            path: self.path.clone(),
        }
    }
}

impl VideoNode {
    pub fn new(source: VideoSource, mode: RenderMode) -> Self {
        let (path, temp_file) = match source {
            VideoSource::Path(p) => (p, None),
            VideoSource::Bytes(data) => {
                let mut temp = NamedTempFile::new().expect("Failed to create temp file");
                temp.write_all(&data).expect("Failed to write video data");
                let p = temp.path().to_owned();
                (p, Some(Arc::new(temp)))
            }
        };

        let loader = match VideoLoader::new(path.clone(), mode) {
            Ok(l) => Some(l),
            Err(e) => {
                eprintln!("Failed to create VideoLoader for {:?}: {}", path, e);
                None
            }
        };

        Self {
            opacity: Animated::new(1.0),
            style: Style::DEFAULT,
            object_fit: ObjectFit::Cover,
            current_frame: Mutex::new(None),
            loader,
            render_mode: mode,
            temp_file,
            path,
        }
    }
}

impl Element for VideoNode {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn layout_style(&self) -> Style {
        self.style.clone()
    }

    fn set_layout_style(&mut self, style: Style) {
        self.style = style;
    }

    fn update(&mut self, time: f64) -> bool {
        self.opacity.update(time);

        if let Some(loader) = &mut self.loader {
            match loader {
                VideoLoader::Threaded(decoder) => {
                    decoder.send_request(time);

                    if let Some(resp) = decoder.get_response() {
                        match resp {
                            VideoResponse::Frame(t, data, w, h) => {
                                let data = Data::new_copy(&data);
                                let info = skia_safe::ImageInfo::new(
                                    (w as i32, h as i32),
                                    ColorType::RGBA8888,
                                    AlphaType::Unpremul,
                                    None,
                                );

                                if let Some(img) = skia_safe::images::raster_from_data(
                                    &info,
                                    data,
                                    (w * 4) as usize,
                                ) {
                                    *self.current_frame.lock().unwrap() = Some((t, img));
                                }
                            }
                            _ => {}
                        }
                    }
                }
                VideoLoader::Sync(decoder) => match decoder.get_frame_at(time) {
                    Ok((t, data, w, h)) => {
                        let data = Data::new_copy(&data);
                        let info = skia_safe::ImageInfo::new(
                            (w as i32, h as i32),
                            ColorType::RGBA8888,
                            AlphaType::Unpremul,
                            None,
                        );

                        if let Some(img) =
                            skia_safe::images::raster_from_data(&info, data, (w * 4) as usize)
                        {
                            *self.current_frame.lock().unwrap() = Some((t, img));
                        }
                    }
                    Err(e) => {
                        eprintln!("Sync Video Error: {}", e);
                    }
                },
            }
        }
        true
    }

    fn render(
        &self,
        canvas: &Canvas,
        rect: Rect,
        parent_opacity: f32,
        draw_children: &mut dyn FnMut(&Canvas),
    ) {
        let op = self.opacity.current_value * parent_opacity;

        let current = self.current_frame.lock().unwrap();
        if let Some((_, img)) = current.as_ref() {
            let paint = Paint::new(Color4f::new(1.0, 1.0, 1.0, op), None);

            let draw_rect = calculate_object_fit_rect(
                img.width() as f32,
                img.height() as f32,
                rect,
                self.object_fit,
            );

            canvas.save();
            if self.object_fit == ObjectFit::Cover {
                canvas.clip_rect(rect, ClipOp::Intersect, true);
            }
            canvas.draw_image_rect(img, None, draw_rect, &paint);
            canvas.restore();
        } else {
            let mut p = Paint::new(Color4f::new(0.0, 0.0, 1.0, 1.0), None);
            p.set_alpha_f(op);
            canvas.draw_rect(rect, &p);
        }
        draw_children(canvas);
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
        if property == "opacity" {
            self.opacity.add_segment(start, target, duration, ease_fn);
        }
    }

    fn animate_property_spring(
        &mut self,
        property: &str,
        start: Option<f32>,
        target: f32,
        config: crate::animation::SpringConfig,
    ) {
        if property == "opacity" {
            if let Some(s) = start {
                self.opacity.add_spring_with_start(s, target, config);
            } else {
                self.opacity.add_spring(target, config);
            }
        }
    }
}

// --- Composition Node (RFC 010) ---

/// A node that contains its own isolated timeline and Director.
///
/// Used for nesting compositions (e.g. pre-comps). It renders the sub-timeline to a surface.
pub struct CompositionNode {
    pub internal_director: Mutex<Director>,
    pub start_offset: f64,
    pub surface_cache: Mutex<Option<Surface>>,
    pub style: Style,
}

impl Clone for CompositionNode {
    fn clone(&self) -> Self {
        let dir = self.internal_director.lock().unwrap().clone();
        Self {
            internal_director: Mutex::new(dir),
            start_offset: self.start_offset,
            surface_cache: Mutex::new(None),
            style: self.style.clone(),
        }
    }
}

impl fmt::Debug for CompositionNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CompositionNode")
            .field("start_offset", &self.start_offset)
            .finish()
    }
}

impl Element for CompositionNode {
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
        let comp_time = time - self.start_offset;
        #[allow(unused_mut)]
        let mut d = self.internal_director.lock().unwrap();
        d.update(comp_time);

        let mut layout_engine = LayoutEngine::new();
        let w = d.width;
        let h = d.height;
        layout_engine.compute_layout(&mut d.scene, w, h, comp_time);
        d.run_post_layout(comp_time);

        true
    }

    fn render(
        &self,
        canvas: &Canvas,
        rect: Rect,
        opacity: f32,
        draw_children: &mut dyn FnMut(&Canvas),
    ) {
        let d = self.internal_director.lock().unwrap();

        let width = d.width;
        let height = d.height;

        let mut surface_opt = self.surface_cache.lock().unwrap();

        // Recreate surface if needed
        let need_new = if let Some(s) = surface_opt.as_ref() {
            s.width() != width || s.height() != height
        } else {
            true
        };

        if need_new {
            let info = skia_safe::ImageInfo::new(
                (width, height),
                ColorType::RGBA8888,
                AlphaType::Premul,
                Some(skia_safe::ColorSpace::new_srgb()),
            );
            *surface_opt = skia_safe::surfaces::raster(&info, None, None);
        }

        if let Some(surface) = surface_opt.as_mut() {
            // Render internal director to surface
            let c = surface.canvas();
            c.clear(skia_safe::Color::TRANSPARENT);

            // Reuse render logic
            let current_time = d
                .scene
                .nodes
                .iter()
                .flatten()
                .next()
                .map(|n| n.last_visit_time)
                .unwrap_or(0.0);

            let mut items: Vec<(usize, crate::director::TimelineItem)> = d
                .timeline
                .iter()
                .cloned()
                .enumerate()
                .filter(|(_, item)| {
                    current_time >= item.start_time
                        && current_time < item.start_time + item.duration
                })
                .collect();
            items.sort_by_key(|(_, item)| item.z_index);

            for (_, item) in items {
                render_recursive(&d.scene, &d.assets, item.scene_root, c, 1.0);
            }

            // Now draw surface to main canvas
            let image = surface.image_snapshot();
            let mut paint = Paint::default();
            paint.set_alpha_f(opacity);

            // Draw image filling the layout rect
            canvas.draw_image_rect(&image, None, rect, &paint);
        }

        draw_children(canvas);
    }

    fn animate_property(
        &mut self,
        _property: &str,
        _start: f32,
        _target: f32,
        _duration: f64,
        _easing: &str,
    ) {
        // No animatable properties on CompositionNode itself yet (e.g. opacity is handled by SceneNode blending)
    }

    fn get_audio(&self, time: f64, samples_needed: usize, _sample_rate: u32) -> Option<Vec<f32>> {
        let comp_time = time - self.start_offset;
        #[allow(unused_mut)]
        let mut d = self.internal_director.lock().unwrap();
        Some(d.mix_audio(samples_needed, comp_time))
    }
}
pub mod vector;
pub use vector::VectorNode;

pub mod lottie;
pub use lottie::LottieNode;
