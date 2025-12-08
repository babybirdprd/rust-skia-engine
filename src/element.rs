use skia_safe::{Canvas, Rect, Color4f};
use taffy::style::Style;
use keyframe::CanTween;
use std::any::Any;

/// Represents a RGBA color in float format (0.0 - 1.0).
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

    /// Converts to Skia's `Color` struct (u8 based).
    pub fn to_skia(&self) -> skia_safe::Color {
        skia_safe::Color::from_argb(
            (self.a * 255.0) as u8,
            (self.r * 255.0) as u8,
            (self.g * 255.0) as u8,
            (self.b * 255.0) as u8,
        )
    }

    /// Converts to Skia's `Color4f` struct (float based).
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

/// Configuration for a linear gradient fill.
#[derive(Clone, Debug, PartialEq)]
pub struct GradientConfig {
    /// List of colors in the gradient.
    pub colors: Vec<Color>,
    /// Optional positions for the color stops (0.0 to 1.0).
    pub positions: Option<Vec<f32>>,
    /// Start point of the gradient (Relative coordinates 0.0 to 1.0).
    pub start: (f32, f32),
    /// End point of the gradient (Relative coordinates 0.0 to 1.0).
    pub end: (f32, f32),
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

/// Represents a span of text with specific styling properties.
///
/// Used for Rich Text rendering where different parts of a string have different styles.
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

/// Strategies for fitting text within a constrained container.
#[derive(Clone, Debug, PartialEq)]
pub enum TextFit {
    /// Normal behavior: text overflows if too long.
    None,
    /// Reduces font size until text fits within the box or hits min size.
    Shrink,
}

/// Drop shadow configuration for text.
#[derive(Clone, Debug)]
pub struct TextShadow {
    pub color: Color,
    pub blur: f32,
    pub offset: (f32, f32),
}

/// Helper trait for cloning trait objects (`Box<dyn Element>`).
pub trait ElementClone {
    fn clone_box(&self) -> Box<dyn Element>;
}

impl<T> ElementClone for T where T: 'static + Element + Clone {
    fn clone_box(&self) -> Box<dyn Element> {
        Box::new(self.clone())
    }
}

/// The core trait defining a visual node in the Scene Graph.
///
/// All renderable entities (Box, Text, Video, etc.) must implement this trait.
pub trait Element: std::fmt::Debug + ElementClone {
    /// Returns self as `Any` for downcasting.
    fn as_any(&self) -> &dyn Any;
    /// Returns mutable self as `Any` for downcasting.
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Returns the Taffy `Style` for layout computation.
    fn layout_style(&self) -> Style;

    /// Updates the Taffy `Style`.
    fn set_layout_style(&mut self, _style: Style) {
        // Default no-op
    }

    /// Updates the element's state for the given time.
    ///
    /// # Returns
    /// * `Ok(true)` if the element's visual appearance changed (requiring a repaint).
    fn update(&mut self, time: f64) -> anyhow::Result<bool>;

    /// Called after layout is computed but before rendering.
    ///
    /// Useful for elements that need to adjust their state based on their final size (e.g. `TextFit`).
    fn post_layout(&mut self, _rect: Rect) {}

    /// Renders the element to the Skia canvas.
    ///
    /// # Arguments
    /// * `canvas` - The Skia canvas to draw on.
    /// * `layout_rect` - The computed layout box for this element.
    /// * `opacity` - The inherited opacity from parent nodes.
    /// * `draw_children` - A closure to trigger rendering of children nodes.
    fn render(&self, canvas: &Canvas, layout_rect: Rect, opacity: f32, draw_children: &mut dyn FnMut(&Canvas));

    /// Animates a specific named property.
    fn animate_property(&mut self, property: &str, start: f32, target: f32, duration: f64, easing: &str);

    /// Animates a property using physics-based spring dynamics.
    fn animate_property_spring(&mut self, _property: &str, _start: Option<f32>, _target: f32, _config: crate::animation::SpringConfig) {
        // Default: No-op or Warn
        // eprintln!("Warning: Spring animation not supported for this property or element.");
    }

    /// Replaces the content with a list of rich text spans.
    fn set_rich_text(&mut self, _spans: Vec<TextSpan>) {}

    /// Modifies the existing text spans in place.
    fn modify_text_spans(&mut self, _f: &dyn Fn(&mut Vec<TextSpan>)) {}

    /// Adds an animator to a specific range of text graphemes.
    ///
    /// Only applicable for `TextNode`.
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
        eprintln!("Warning: add_animator called on a non-text node.");
    }

    /// Returns audio samples if this element contains audio (e.g., Video or Composition).
    ///
    /// # Arguments
    /// * `time` - Local time in the element's timeline.
    /// * `samples_needed` - Number of samples requested.
    /// * `sample_rate` - Target sample rate.
    fn get_audio(&self, _time: f64, _samples_needed: usize, _sample_rate: u32) -> Option<Vec<f32>> {
        None
    }
}

impl Clone for Box<dyn Element> {
    fn clone(&self) -> Box<dyn Element> {
        self.clone_box()
    }
}
