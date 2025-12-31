use director_core::animation::{EasingType, SpringConfig};
use director_core::types::{Color, GradientConfig};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MovieRequest {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub scenes: Vec<Scene>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Scene {
    pub id: String,
    pub duration_secs: f64,
    pub background: Option<Color>,
    pub root: Node,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Node {
    pub id: String,
    // Common properties
    #[serde(default)]
    pub style: StyleMap,
    #[serde(default)]
    pub transform: TransformMap,
    #[serde(default)]
    pub animations: Vec<Animation>,

    // The specific type (Box, Text, Image, Video, Vector, Lottie, Effect)
    #[serde(flatten)]
    pub kind: NodeKind,

    #[serde(default)]
    pub children: Vec<Node>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NodeKind {
    /// A simple rectangular container with optional border radius.
    Box {
        #[serde(default)]
        border_radius: f32,
    },
    /// A text element with content and font size.
    Text {
        content: String,
        font_size: f32,
        /// Per-glyph animators for kinetic typography
        #[serde(default)]
        animators: Vec<TextAnimator>,
    },
    /// An image element loaded from a file path.
    Image {
        src: String,
        /// How the image should fit within its container: "cover", "contain", "fill"
        #[serde(default)]
        object_fit: Option<String>,
    },
    /// A video element loaded from a file path.
    Video {
        src: String,
        /// How the video should fit within its container: "cover", "contain", "fill"
        #[serde(default)]
        object_fit: Option<String>,
    },
    /// An SVG vector graphics element.
    Vector {
        /// Path to SVG file
        src: String,
    },
    /// A Lottie animation (JSON-based vector animation).
    Lottie {
        /// Path to Lottie JSON file
        src: String,
        /// Playback speed multiplier (default: 1.0)
        #[serde(default = "default_speed")]
        speed: f32,
        /// Whether to loop the animation
        #[serde(default)]
        loop_animation: bool,
    },
    /// A visual effect wrapper (blur, shadows, etc.)
    Effect { effect_type: EffectConfig },
}

fn default_speed() -> f32 {
    1.0
}

/// Configuration for visual effects applied to nodes.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "effect", rename_all = "snake_case")]
pub enum EffectConfig {
    /// Gaussian blur effect.
    Blur {
        /// Blur radius (sigma). Default: 10.0
        #[serde(default = "default_blur")]
        sigma: f32,
    },
    /// Drop shadow effect.
    DropShadow {
        /// Shadow blur radius. Default: 10.0
        #[serde(default = "default_blur")]
        blur: f32,
        /// Horizontal offset
        #[serde(default)]
        offset_x: f32,
        /// Vertical offset
        #[serde(default)]
        offset_y: f32,
        /// Shadow color
        #[serde(default)]
        color: Option<Color>,
    },
}

fn default_blur() -> f32 {
    10.0
}

// Simplified Style Map for JSON (maps to Taffy later)
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct StyleMap {
    // Size
    pub width: Option<String>, // "100%", "50px", "auto"
    pub height: Option<String>,

    // Flexbox
    pub flex_direction: Option<String>, // "row", "column"
    pub justify_content: Option<String>,
    pub align_items: Option<String>,

    // Appearance
    pub bg_color: Option<Color>,
    pub opacity: Option<f32>,

    // Spacing
    pub padding: Option<f32>,
    pub margin: Option<f32>,

    // Positioning (absolute positioning support)
    /// Layout position mode: "relative" (default) or "absolute"
    pub position: Option<String>,
    /// Inset from top edge (for absolute positioning)
    pub top: Option<f32>,
    /// Inset from left edge
    pub left: Option<f32>,
    /// Inset from right edge  
    pub right: Option<f32>,
    /// Inset from bottom edge
    pub bottom: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct TransformMap {
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub rotation: Option<f32>,
    pub scale: Option<f32>,
    pub pivot_x: Option<f32>,
    pub pivot_y: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Animation {
    pub property: String, // "x", "opacity", "scale"
    pub start: Option<f32>,
    pub end: f32,
    pub duration: f64,
    pub start_time: f64, // Relative to scene start
    pub easing: EasingType,
}

/// Per-glyph animator for kinetic typography effects.
///
/// Animates a range of text characters (graphemes) with the specified property.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TextAnimator {
    /// Start index of the grapheme range (inclusive)
    pub start_idx: usize,
    /// End index of the grapheme range (exclusive)
    pub end_idx: usize,
    /// Property to animate: "opacity", "offset_y", "offset_x", "scale", "rotation"
    pub property: String,
    /// Starting value
    pub start_val: f32,
    /// Target value
    pub target_val: f32,
    /// Animation duration in seconds
    pub duration: f64,
    /// Easing function
    pub easing: EasingType,
}

#[cfg(test)]
mod tests {
    use super::*;
    use director_core::types::Color;

    #[test]
    fn test_schema_serialization() {
        let movie = MovieRequest {
            width: 1920,
            height: 1080,
            fps: 30,
            scenes: vec![Scene {
                id: "scene_1".to_string(),
                duration_secs: 5.0,
                background: Some(Color::BLACK),
                root: Node {
                    id: "root".to_string(),
                    kind: NodeKind::Box { border_radius: 0.0 },
                    style: StyleMap {
                        width: Some("100%".to_string()),
                        height: Some("100%".to_string()),
                        bg_color: Some(Color::new(0.1, 0.1, 0.1, 1.0)),
                        ..Default::default()
                    },
                    transform: TransformMap::default(),
                    animations: vec![],
                    children: vec![Node {
                        id: "text_1".to_string(),
                        kind: NodeKind::Text {
                            content: "Hello JSON".to_string(),
                            font_size: 100.0,
                            animators: vec![],
                        },
                        style: StyleMap {
                            bg_color: Some(Color::WHITE),
                            ..Default::default()
                        },
                        transform: TransformMap::default(),
                        animations: vec![],
                        children: vec![],
                    }],
                },
            }],
        };

        let json = serde_json::to_string_pretty(&movie).unwrap();
        println!("{}", json);

        // Ensure we can read it back
        let _loaded: MovieRequest = serde_json::from_str(&json).unwrap();
    }
}
