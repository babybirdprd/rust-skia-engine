use skia_safe::{Canvas, Rect, Color4f};
use taffy::style::Style;
use keyframe::CanTween;

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

impl CanTween for Color {
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

#[derive(Clone, Debug, PartialEq)]
pub struct GradientConfig {
    pub colors: Vec<Color>,
    pub positions: Option<Vec<f32>>,
    pub start: (f32, f32), // Relative 0.0 to 1.0
    pub end: (f32, f32),   // Relative 0.0 to 1.0
}

impl Default for GradientConfig {
    fn default() -> Self {
        Self {
            colors: vec![Color::BLACK, Color::WHITE],
            positions: None,
            start: (0.0, 0.0),
            end: (0.0, 1.0), // Default Top-to-Bottom
        }
    }
}

#[derive(Clone, Debug)]
pub struct TextSpan {
    pub text: String,
    pub color: Option<Color>,
    pub font_family: Option<String>,
    pub font_weight: Option<u16>,
    pub font_style: Option<String>,
    pub font_size: Option<f32>,
    // Rich Text / RFC 003
    pub background_color: Option<Color>,
    pub background_padding: Option<f32>,
    pub stroke_width: Option<f32>,
    pub stroke_color: Option<Color>,
    pub fill_gradient: Option<GradientConfig>,
}

pub trait Element: std::fmt::Debug {
    // 1. Layout Phase
    fn layout_style(&self) -> Style;

    // 2. Update Phase
    fn update(&mut self, time: f64) -> bool;

    // 3. Render Phase
    fn render(&self, canvas: &Canvas, layout_rect: Rect, opacity: f32, draw_children: &mut dyn FnMut(&Canvas));

    // 4. Animation Interface
    fn animate_property(&mut self, property: &str, start: f32, target: f32, duration: f64, easing: &str);

    // 5. Rich Text Interface
    fn set_rich_text(&mut self, _spans: Vec<TextSpan>) {}
    fn modify_text_spans(&mut self, _f: &dyn Fn(&mut Vec<TextSpan>)) {}

    // 6. Text Animator Interface (RFC 009)
    fn add_text_animator(
        &mut self,
        _start_idx: usize,
        _end_idx: usize,
        _property: String,
        _start_val: f32,
        _target_val: f32,
        _duration: f64,
        _easing: &str
    ) {
        // Default implementation: Warn user
        // We can't access node ID easily here, but we can print "non-text node"
        eprintln!("Warning: add_animator called on a non-text node.");
    }
}
