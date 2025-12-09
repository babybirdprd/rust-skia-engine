use serde::{Serialize, Deserialize};
use director_core::types::{Color, GradientConfig};
use director_core::animation::{EasingType, SpringConfig};

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

    // The specific type (Box, Text, Image)
    #[serde(flatten)]
    pub kind: NodeKind,

    #[serde(default)]
    pub children: Vec<Node>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NodeKind {
    Box {
        #[serde(default)]
        border_radius: f32,
        // ... other box props
    },
    Text {
        content: String,
        font_size: f32,
        // ... other text props
    },
    Image {
        src: String,
    },
    Video {
        src: String,
    }
}

// Simplified Style Map for JSON (maps to Taffy later)
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct StyleMap {
    pub width: Option<String>, // "100%", "50px", "auto"
    pub height: Option<String>,
    pub flex_direction: Option<String>, // "row", "column"
    pub justify_content: Option<String>,
    pub align_items: Option<String>,
    pub bg_color: Option<Color>,
    pub padding: Option<f32>,
    pub margin: Option<f32>,
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
