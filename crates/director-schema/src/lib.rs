use director_core::animation::{EasingType, SpringConfig};
use director_core::types::{Color, GradientConfig};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MovieRequest {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub scenes: Vec<Scene>,
    /// Global audio tracks for the movie
    #[serde(default)]
    pub audio_tracks: Vec<AudioTrack>,
}

/// Visual transition type between scenes.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "snake_case")]
pub enum TransitionType {
    #[default]
    Fade,
    SlideLeft,
    SlideRight,
    WipeLeft,
    WipeRight,
    CircleOpen,
}

/// Configuration for a scene-to-scene transition.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionConfig {
    /// Transition type
    #[serde(rename = "type")]
    pub kind: TransitionType,
    /// Duration in seconds
    pub duration: f64,
    /// Easing function (default: Linear)
    #[serde(default = "default_easing")]
    pub easing: EasingType,
}

fn default_easing() -> EasingType {
    EasingType::Linear
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Scene {
    pub id: String,
    pub duration_secs: f64,
    pub background: Option<Color>,
    pub root: Node,
    /// Transition to the next scene (optional)
    #[serde(default)]
    pub transition: Option<TransitionConfig>,
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
    /// Audio-reactive bindings for this node
    #[serde(default)]
    pub audio_bindings: Vec<AudioReactiveBinding>,

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
    /// Custom 4x5 color matrix transform (20 floats).
    ColorMatrix {
        /// The 20-element color matrix [R, G, B, A, Offset] x 4 rows
        matrix: Vec<f32>,
    },
    /// Grayscale preset (convenience wrapper for ColorMatrix).
    Grayscale,
    /// Sepia tone preset (convenience wrapper for ColorMatrix).
    Sepia,
    /// Directional blur (motion blur) along an angle.
    DirectionalBlur {
        /// Blur strength (pixel distance). Default: 10.0
        #[serde(default = "default_blur")]
        strength: f32,
        /// Direction angle in degrees (0 = right, 90 = down). Default: 0.0
        #[serde(default)]
        angle: f32,
        /// Number of samples (quality vs performance, 4-64). Default: 16
        #[serde(default = "default_samples")]
        samples: u32,
    },
    /// Film grain / noise overlay for cinematic look.
    FilmGrain {
        /// Grain intensity (0.0 - 1.0). Default: 0.1
        #[serde(default = "default_grain_intensity")]
        intensity: f32,
        /// Grain size/scale in pixels. Default: 1.0
        #[serde(default = "default_grain_size")]
        size: f32,
    },
}

fn default_blur() -> f32 {
    10.0
}

fn default_samples() -> u32 {
    16
}

fn default_grain_intensity() -> f32 {
    0.1
}

fn default_grain_size() -> f32 {
    1.0
}

// Simplified Style Map for JSON (maps to Taffy later)
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct StyleMap {
    // Size
    pub width: Option<String>, // "100%", "50px", "auto"
    pub height: Option<String>,

    // Display mode
    /// Layout display mode: "flex" (default) or "grid"
    pub display: Option<String>,

    // Flexbox
    pub flex_direction: Option<String>, // "row", "column"
    pub justify_content: Option<String>,
    pub align_items: Option<String>,

    // Grid template (container)
    /// Grid column definitions: ["1fr", "2fr", "100px", "auto"]
    pub grid_template_columns: Option<Vec<String>>,
    /// Grid row definitions: ["auto", "1fr", "200px"]
    pub grid_template_rows: Option<Vec<String>>,

    // Grid placement (item)
    /// Grid row placement: "1", "1 / 3", "span 2"
    pub grid_row: Option<String>,
    /// Grid column placement: "1", "1 / 3", "span 2"
    pub grid_column: Option<String>,

    // Gap (works for both flex and grid)
    /// Gap between items in pixels
    pub gap: Option<f32>,

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

/// An audio track in the project.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AudioTrack {
    /// Unique identifier for referencing in audio bindings
    pub id: String,
    /// Path to audio file
    pub src: String,
    /// Start time in seconds (global timeline)
    #[serde(default)]
    pub start_time: f64,
    /// Volume level (0.0 - 1.0)
    #[serde(default = "default_volume")]
    pub volume: f32,
    /// Whether to loop the audio
    #[serde(default)]
    pub loop_audio: bool,
}

fn default_volume() -> f32 {
    1.0
}

/// Binds a node property to an audio analysis value.
///
/// Enables beat-reactive visuals by mapping frequency band energy to node properties.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AudioReactiveBinding {
    /// ID of the audio track to analyze
    pub audio_id: String,
    /// Frequency band: "bass", "mids", "highs"
    pub band: String,
    /// Property to bind: "scale", "opacity", "y", "x", "rotation"
    pub property: String,
    /// Minimum output value (when energy is 0)
    pub min_value: f32,
    /// Maximum output value (when energy is 1)
    pub max_value: f32,
    /// Smoothing factor (0.0 = instant, 0.9 = heavy smoothing)
    #[serde(default)]
    pub smoothing: f32,
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
                    audio_bindings: vec![],
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
                        audio_bindings: vec![],
                        children: vec![],
                    }],
                },
                transition: None,
            }],
            audio_tracks: vec![],
        };

        let json = serde_json::to_string_pretty(&movie).unwrap();
        println!("{}", json);

        // Ensure we can read it back
        let _loaded: MovieRequest = serde_json::from_str(&json).unwrap();
    }

    #[test]
    fn test_effect_config_serialization() {
        // Test each EffectConfig variant roundtrips correctly
        let effects = vec![
            EffectConfig::Blur { sigma: 15.0 },
            EffectConfig::DropShadow {
                blur: 10.0,
                offset_x: 5.0,
                offset_y: 5.0,
                color: Some(Color::BLACK),
            },
            EffectConfig::ColorMatrix {
                matrix: vec![1.0; 20],
            },
            EffectConfig::Grayscale,
            EffectConfig::Sepia,
            EffectConfig::DirectionalBlur {
                strength: 20.0,
                angle: 45.0,
                samples: 32,
            },
            EffectConfig::FilmGrain {
                intensity: 0.15,
                size: 2.0,
            },
        ];

        for effect in effects {
            let json = serde_json::to_string(&effect).unwrap();
            let loaded: EffectConfig = serde_json::from_str(&json).unwrap();
            // Verify via debug output that it roundtrips
            assert_eq!(format!("{:?}", effect), format!("{:?}", loaded));
        }
    }

    #[test]
    fn test_transition_config_serialization() {
        // Test each TransitionType variant roundtrips correctly
        let transitions = vec![
            TransitionConfig {
                kind: TransitionType::Fade,
                duration: 1.0,
                easing: EasingType::Linear,
            },
            TransitionConfig {
                kind: TransitionType::SlideLeft,
                duration: 0.5,
                easing: EasingType::EaseInOut,
            },
            TransitionConfig {
                kind: TransitionType::SlideRight,
                duration: 0.75,
                easing: EasingType::BounceOut,
            },
            TransitionConfig {
                kind: TransitionType::WipeLeft,
                duration: 1.0,
                easing: EasingType::Linear,
            },
            TransitionConfig {
                kind: TransitionType::WipeRight,
                duration: 1.0,
                easing: EasingType::Linear,
            },
            TransitionConfig {
                kind: TransitionType::CircleOpen,
                duration: 2.0,
                easing: EasingType::EaseOut,
            },
        ];

        for trans in transitions {
            let json = serde_json::to_string(&trans).unwrap();
            let loaded: TransitionConfig = serde_json::from_str(&json).unwrap();
            assert_eq!(format!("{:?}", trans), format!("{:?}", loaded));
        }
    }
}
