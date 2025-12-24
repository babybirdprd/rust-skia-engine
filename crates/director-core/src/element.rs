use crate::types::{Color, GradientConfig};
use skia_safe::{Canvas, Rect};
use std::any::Any;
use taffy::geometry::Size;
use taffy::style::AvailableSpace;
use taffy::style::Style;
use tracing::warn;

/// Represents a span of text with specific styling properties.
///
/// Used for Rich Text rendering where different parts of a string have different styles.
#[derive(Clone, Debug, Default)]
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

impl<T> ElementClone for T
where
    T: 'static + Element + Clone,
{
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

    /// Whether this element has intrinsic content size that layout needs to know about.
    fn needs_measure(&self) -> bool {
        false
    }

    /// Computes the size of the element given the available space.
    fn measure(
        &self,
        _known_dimensions: Size<Option<f32>>,
        _available_space: Size<AvailableSpace>,
    ) -> Size<f32> {
        Size::ZERO
    }

    /// Returns the Taffy `Style` for layout computation.
    fn layout_style(&self) -> Style;

    /// Updates the Taffy `Style`.
    fn set_layout_style(&mut self, _style: Style) {
        // Default no-op
    }

    /// Updates the element's state for the given time.
    ///
    /// # Returns
    /// * `true` if the element's visual appearance changed (requiring a repaint).
    fn update(&mut self, time: f64) -> bool;

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
    fn render(
        &self,
        canvas: &Canvas,
        layout_rect: Rect,
        opacity: f32,
        draw_children: &mut dyn FnMut(&Canvas),
    ) -> Result<(), crate::RenderError>;

    /// Animates a specific named property.
    fn animate_property(
        &mut self,
        property: &str,
        start: f32,
        target: f32,
        duration: f64,
        easing: &str,
    );

    /// Animates a property using physics-based spring dynamics.
    fn animate_property_spring(
        &mut self,
        _property: &str,
        _start: Option<f32>,
        _target: f32,
        _config: crate::animation::SpringConfig,
    ) {
        // Default: No-op or Warn
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
        _easing: &str,
    ) {
        // Default implementation: Warn user
        warn!("add_animator called on a non-text node.");
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
