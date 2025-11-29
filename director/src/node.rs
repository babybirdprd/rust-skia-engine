use skia_safe::{Canvas, Paint, Rect, Image, Color4f, Data, TextBlob};
use taffy::style::Style;
use crate::element::Element;
use crate::animation::{Animated, EasingType};
use cosmic_text::{Buffer, FontSystem, Metrics, SwashCache, Attrs, Shaping};
use std::sync::{Arc, Mutex};
use std::cell::RefCell;
// Video imports
use crate::video_wrapper::{Decoder};

// Helper to parse easing
fn parse_easing(e: &str) -> EasingType {
    match e {
        "linear" => EasingType::Linear,
        "ease_in" => EasingType::EaseIn,
        "ease_out" => EasingType::EaseOut,
        "ease_in_out" => EasingType::EaseInOut,
        "bounce_out" => EasingType::BounceOut,
        _ => EasingType::Linear,
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const BLACK: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const WHITE: Color = Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };

    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn to_skia(&self) -> skia_safe::Color {
        skia_safe::Color::from_argb(
            (self.a * 255.0) as u8,
            (self.r * 255.0) as u8,
            (self.g * 255.0) as u8,
            (self.b * 255.0) as u8,
        )
    }

    pub fn to_color4f(&self) -> Color4f {
        Color4f::new(self.r, self.g, self.b, self.a)
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::BLACK
    }
}

impl keyframe::CanTween for Color {
    fn ease(from: Self, to: Self, time: impl keyframe::num_traits::Float) -> Self {
        let t = time.to_f64().unwrap() as f32;
        Self {
            r: from.r + (to.r - from.r) * t,
            g: from.g + (to.g - from.g) * t,
            b: from.b + (to.b - from.b) * t,
            a: from.a + (to.a - from.a) * t,
        }
    }
}

// --- Box Node ---
#[derive(Debug)]
pub struct BoxNode {
    pub style: Style,
    pub bg_color: Option<Animated<Color>>,
    pub opacity: Animated<f32>,
    pub blur: Animated<f32>,
}

impl BoxNode {
    pub fn new() -> Self {
        Self {
            style: Style::default(),
            bg_color: None,
            opacity: Animated::new(1.0),
            blur: Animated::new(0.0),
        }
    }
}

impl Element for BoxNode {
    fn layout_style(&self) -> Style {
        self.style.clone()
    }

    fn update(&mut self, time: f64) -> bool {
        let mut changed = false;
        if let Some(bg) = &mut self.bg_color {
            bg.update(time);
            changed = true;
        }
        self.opacity.update(time);
        self.blur.update(time);
        changed
    }

    fn render(&self, canvas: &Canvas, rect: Rect, opacity: f32) {
        let local_opacity = self.opacity.current_value * opacity;

        let mut paint = Paint::default();
        if self.blur.current_value > 0.0 {
            let sigma = self.blur.current_value;
            if let Some(filter) = skia_safe::image_filters::blur(
                (sigma, sigma),
                skia_safe::TileMode::Decal,
                None,
                None
            ) {
                paint.set_image_filter(filter);
            }
        }

        if let Some(bg) = &self.bg_color {
            let mut c = bg.current_value;
            c.a *= local_opacity;
            paint.set_color4f(c.to_color4f(), None);
            canvas.draw_rect(rect, &paint);
        }
    }

    fn animate_property(&mut self, property: &str, target: f32, duration: f64, easing: &str) {
        let ease_fn = parse_easing(easing);
        match property {
            "opacity" => self.opacity.add_keyframe(target, duration, ease_fn),
            "blur" => self.blur.add_keyframe(target, duration, ease_fn),
            _ => {}
        }
    }
}

// --- Text Node ---
pub struct TextNode {
    pub content: String,
    pub font_family: String,
    pub font_size: Animated<f32>,
    pub color: Animated<Color>,
    pub buffer: RefCell<Option<Buffer>>,
    pub font_system: Arc<Mutex<FontSystem>>,
    pub swash_cache: Arc<Mutex<SwashCache>>,
}

impl std::fmt::Debug for TextNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextNode")
         .field("content", &self.content)
         .finish()
    }
}

impl TextNode {
    pub fn new(content: String, font_system: Arc<Mutex<FontSystem>>, swash_cache: Arc<Mutex<SwashCache>>) -> Self {
        let mut node = Self {
            content,
            font_family: "Sans Serif".into(),
            font_size: Animated::new(20.0),
            color: Animated::new(Color::WHITE),
            buffer: RefCell::new(None),
            font_system: font_system.clone(),
            swash_cache,
        };
        node.init_buffer();
        node
    }

    fn init_buffer(&mut self) {
         let mut fs = self.font_system.lock().unwrap();
         let mut buffer = Buffer::new(&mut fs, Metrics::new(20.0, 24.0));
         buffer.set_text(&mut fs, &self.content, &Attrs::new(), Shaping::Advanced, None);
         *self.buffer.borrow_mut() = Some(buffer);
    }
}

impl Element for TextNode {
    fn layout_style(&self) -> Style {
        Style::DEFAULT
    }

    fn update(&mut self, time: f64) -> bool {
        self.font_size.update(time);
        self.color.update(time);

        let size = self.font_size.current_value;
        let line_height = size * 1.2;

        let mut buf_ref = self.buffer.borrow_mut();
        if let Some(buffer) = buf_ref.as_mut() {
            let mut fs = self.font_system.lock().unwrap();
            buffer.set_metrics(&mut fs, Metrics::new(size, line_height));
        }
        true
    }

    fn render(&self, canvas: &Canvas, rect: Rect, opacity: f32) {
        let mut buf_ref = self.buffer.borrow_mut();
        if let Some(buffer) = buf_ref.as_mut() {
            let mut fs = self.font_system.lock().unwrap();

            // Set size for wrapping
            buffer.set_size(&mut fs, Some(rect.width()), Some(rect.height()));
            buffer.shape_until_scroll(&mut fs, false);

            // Draw Loop
            let mut paint = Paint::default();
            let mut c = self.color.current_value;
            c.a *= opacity;
            paint.set_color4f(c.to_color4f(), None);

            let font = skia_safe::Font::default();

            for run in buffer.layout_runs() {
                 let origin_y = rect.top + run.line_y;

                 if let Some(blob) = TextBlob::from_str(run.text, &font) {
                      canvas.draw_text_blob(&blob, (rect.left, origin_y), &paint);
                 }
            }
        }
    }

    fn animate_property(&mut self, property: &str, target: f32, duration: f64, easing: &str) {
        let ease_fn = parse_easing(easing);
        match property {
            "font_size" | "size" => self.font_size.add_keyframe(target, duration, ease_fn),
            _ => {}
        }
    }
}

// --- Image Node ---
#[derive(Debug)]
pub struct ImageNode {
    pub image: Option<Image>,
    pub opacity: Animated<f32>,
}

impl ImageNode {
    pub fn new(path: &str) -> Self {
        let image = match std::fs::read(path) {
            Ok(bytes) => {
                 Image::from_encoded(Data::new_copy(&bytes))
            },
            Err(_) => None,
        };

        Self {
            image,
            opacity: Animated::new(1.0),
        }
    }
}

impl Element for ImageNode {
    fn layout_style(&self) -> Style {
        Style::DEFAULT
    }

    fn update(&mut self, time: f64) -> bool {
        self.opacity.update(time);
        true
    }

    fn render(&self, canvas: &Canvas, rect: Rect, parent_opacity: f32) {
        let op = self.opacity.current_value * parent_opacity;
        let paint = Paint::new(Color4f::new(1.0, 1.0, 1.0, op), None);

        if let Some(img) = &self.image {
             canvas.draw_image_rect(img, None, rect, &paint);
        }
    }

    fn animate_property(&mut self, property: &str, target: f32, duration: f64, easing: &str) {
        let ease_fn = parse_easing(easing);
        if property == "opacity" {
            self.opacity.add_keyframe(target, duration, ease_fn);
        }
    }
}

// --- Video Node ---
#[derive(Debug)]
pub struct VideoNode {
    decoder: Option<Decoder>,
    pub opacity: Animated<f32>,
    source_path: String,
}

impl VideoNode {
    pub fn new(path: &str) -> Self {
        let decoder = Decoder::new(std::path::Path::new(path)).ok();
        Self {
            decoder,
            opacity: Animated::new(1.0),
            source_path: path.to_string(),
        }
    }
}

impl Element for VideoNode {
    fn layout_style(&self) -> Style {
        Style::DEFAULT
    }

    fn update(&mut self, time: f64) -> bool {
        self.opacity.update(time);
        true
    }

    fn render(&self, canvas: &Canvas, rect: Rect, parent_opacity: f32) {
         let op = self.opacity.current_value * parent_opacity;
         let mut p = Paint::new(Color4f::new(0.0, 0.0, 1.0, 1.0), None);
         p.set_alpha_f(op);
         canvas.draw_rect(rect, &p);
    }

    fn animate_property(&mut self, property: &str, target: f32, duration: f64, easing: &str) {
         let ease_fn = parse_easing(easing);
         if property == "opacity" {
             self.opacity.add_keyframe(target, duration, ease_fn);
         }
    }
}
