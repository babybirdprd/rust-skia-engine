//! # Types Module
//!
//! Shared data types used across the engine.
//!
//! ## Responsibilities
//! - **Color**: RGBA color representation with Skia conversion.
//! - **Transform**: Animated 2D transforms (scale, rotation, translation).
//! - **ObjectFit**: Image/video scaling modes (Cover, Contain, Fill).
//!
//! ## Key Types
//! - `Color`: Float-based RGBA color.
//! - `Transform`: All animated transform properties.
//! - `NodeId`: Type alias for arena indices (`usize`).

use crate::animation::Animated;
use keyframe::CanTween;
use serde::{Deserialize, Serialize};
use skia_safe::{Color4f, Path};

/// Specifies how the content of a replaceable element (img, video) should
/// be resized to fit its container.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ObjectFit {
    /// The content is sized to maintain its aspect ratio while filling the element's entire content box.
    /// If the object's aspect ratio does not match the aspect ratio of its box, then the object will be clipped to fit.
    Cover,
    /// The content is scaled to maintain its aspect ratio while fitting within the element's content box.
    /// The entire object is made to fill the box, while preserving its aspect ratio, so the object will be "letterboxed"
    /// if its aspect ratio does not match the aspect ratio of the box.
    Contain,
    /// The content is sized to fill the element's content box. The entire object will completely fill the box.
    /// If the object's aspect ratio does not match the aspect ratio of its box, then the object will be stretched to fit.
    Fill,
}

impl Default for ObjectFit {
    fn default() -> Self {
        Self::Cover
    }
}

// --- From director.rs ---

/// A unique identifier for a node in the scene graph.
pub type NodeId = usize;

/// State for a node currently animating along an SVG path.
#[derive(Clone)]
pub struct PathAnimationState {
    pub path: Path,
    pub progress: Animated<f32>,
}

/// Represents the affine transformation state of a node.
#[derive(Clone, Debug)]
pub struct Transform {
    pub scale_x: Animated<f32>,
    pub scale_y: Animated<f32>,
    pub rotation: Animated<f32>,
    pub skew_x: Animated<f32>,
    pub skew_y: Animated<f32>,
    pub translate_x: Animated<f32>,
    pub translate_y: Animated<f32>,
    pub pivot_x: f32,
    pub pivot_y: f32,
}

impl Transform {
    pub fn new() -> Self {
        Self {
            scale_x: Animated::new(1.0),
            scale_y: Animated::new(1.0),
            rotation: Animated::new(0.0),
            skew_x: Animated::new(0.0),
            skew_y: Animated::new(0.0),
            translate_x: Animated::new(0.0),
            translate_y: Animated::new(0.0),
            pivot_x: 0.5,
            pivot_y: 0.5,
        }
    }
}

// --- From element.rs ---

/// Represents a RGBA color in float format (0.0 - 1.0).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const BLACK: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const WHITE: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };

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
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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

// --- From tokens.rs ---

/// Represents a "safe zone" or padding area to avoid UI elements on specific platforms.
#[derive(Clone, Debug)]
pub struct SafeZone {
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
    pub aspect_ratio: String,
}
