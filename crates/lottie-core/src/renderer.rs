use glam::{Mat4, Vec2, Vec3, Vec4};
use kurbo::BezPath;

#[derive(Clone, Debug)]
pub struct RenderTree {
    pub width: f32,
    pub height: f32,
    pub root: RenderNode,
    pub view_matrix: Mat4,
    pub projection_matrix: Mat4,
}

impl RenderTree {
    /// Returns a mock render tree for testing purposes.
    pub fn mock_sample() -> Self {
        // Create a red rectangle
        let mut rect_path = BezPath::new();
        rect_path.move_to((100.0, 100.0));
        rect_path.line_to((300.0, 100.0));
        rect_path.line_to((300.0, 300.0));
        rect_path.line_to((100.0, 300.0));
        rect_path.close_path();

        let rect_shape = Shape {
            geometry: ShapeGeometry::Path(rect_path),
            fill: Some(Fill {
                paint: Paint::Solid(Vec4::new(1.0, 0.0, 0.0, 1.0)), // Red
                opacity: 1.0,
                rule: FillRule::NonZero,
            }),
            stroke: Some(Stroke {
                paint: Paint::Solid(Vec4::new(0.0, 0.0, 0.0, 1.0)), // Black
                width: 5.0,
                opacity: 1.0,
                cap: LineCap::Round,
                join: LineJoin::Round,
                miter_limit: None,
                dash: None,
            }),
            trim: None,
        };

        let root = RenderNode {
            transform: Mat4::IDENTITY,
            alpha: 1.0,
            blend_mode: BlendMode::Normal,
            content: NodeContent::Shape(rect_shape),
            masks: vec![],
            matte: None,
            effects: vec![],
            styles: vec![],
            is_adjustment_layer: false,
        };

        RenderTree {
            width: 500.0,
            height: 500.0,
            root,
            view_matrix: Mat4::IDENTITY,
            projection_matrix: Mat4::IDENTITY,
        }
    }
}

#[derive(Clone, Debug)]
pub struct RenderNode {
    pub transform: Mat4,
    pub alpha: f32,
    pub blend_mode: BlendMode,
    pub content: NodeContent,
    pub masks: Vec<Mask>,
    pub matte: Option<Box<Matte>>,
    pub effects: Vec<Effect>,
    pub styles: Vec<LayerStyle>,
    pub is_adjustment_layer: bool,
}

#[derive(Clone, Debug)]
pub enum NodeContent {
    Group(Vec<RenderNode>),
    Shape(Shape),
    Text(Text),
    Image(Image),
}

#[derive(Clone, Debug)]
pub struct Shape {
    pub geometry: ShapeGeometry,
    pub fill: Option<Fill>,
    pub stroke: Option<Stroke>,
    pub trim: Option<Trim>,
}

#[derive(Clone, Debug)]
pub enum ShapeGeometry {
    Path(BezPath),
    Boolean {
        mode: MergeMode,
        shapes: Vec<ShapeGeometry>,
    },
}

#[derive(Clone, Copy, Debug)]
pub enum MergeMode {
    Merge,
    Add,
    Subtract,
    Intersect,
    Exclude,
}

#[derive(Clone, Copy, Debug)]
pub struct Trim {
    pub start: f32,  // 0.0 to 1.0
    pub end: f32,    // 0.0 to 1.0
    pub offset: f32, // 0.0 to 1.0 (usually)
}

#[derive(Clone, Debug)]
pub struct Text {
    pub glyphs: Vec<RenderGlyph>,
    pub font_family: String,
    pub size: f32,
    pub justify: Justification,
    pub tracking: f32,
    pub line_height: f32,
}

#[derive(Clone, Debug)]
pub struct RenderGlyph {
    pub character: char,
    pub pos: Vec3,
    pub scale: Vec3,
    pub rotation: Vec3, // Euler angles (radians)
    pub tracking: f32,
    pub alpha: f32,
    pub fill: Option<Fill>,
    pub stroke: Option<Stroke>,
}

#[derive(Clone, Debug)]
pub struct Image {
    // Encoded image data (e.g. PNG, JPEG)
    pub data: Option<Vec<u8>>,
    pub width: u32,
    pub height: u32,
    pub id: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Fill {
    pub paint: Paint,
    pub opacity: f32,
    pub rule: FillRule,
}

#[derive(Clone, Debug)]
pub struct Stroke {
    pub paint: Paint,
    pub width: f32,
    pub opacity: f32,
    pub cap: LineCap,
    pub join: LineJoin,
    pub miter_limit: Option<f32>,
    pub dash: Option<DashPattern>,
}

#[derive(Clone, Debug)]
pub enum Paint {
    Solid(Vec4), // R, G, B, A
    Gradient(Gradient),
}

#[derive(Clone, Debug)]
pub struct Gradient {
    pub kind: GradientKind,
    pub stops: Vec<GradientStop>,
    // Coordinates are handled by the gradient shader construction.
    // Lottie gradients usually have start/end points.
    pub start: Vec2,
    pub end: Vec2,
}

#[derive(Clone, Copy, Debug)]
pub enum GradientKind {
    Linear,
    Radial,
}

#[derive(Clone, Debug)]
pub struct GradientStop {
    pub offset: f32,
    pub color: Vec4,
}

#[derive(Clone, Debug)]
pub struct DashPattern {
    pub array: Vec<f32>,
    pub offset: f32,
}

#[derive(Clone, Debug)]
pub struct Mask {
    pub mode: MaskMode,
    pub geometry: BezPath,
    pub opacity: f32,
    pub expansion: f32,
    pub inverted: bool,
}

#[derive(Clone, Debug)]
pub struct Matte {
    pub mode: MatteMode,
    pub node: RenderNode,
}

#[derive(Clone, Debug)]
pub enum Effect {
    GaussianBlur {
        sigma: f32,
    },
    DropShadow {
        color: Vec4,
        offset: Vec2,
        blur: f32,
    },
    ColorMatrix {
        matrix: [f32; 20],
    },
    DisplacementMap {
        scale: f32,
        x_channel: ColorChannel,
        y_channel: ColorChannel,
    },
    Tint {
        black: Vec4,
        white: Vec4,
        amount: f32,
    },
    Fill {
        color: Vec4,
        opacity: f32,
    },
    Tritone {
        highlights: Vec4,
        midtones: Vec4,
        shadows: Vec4,
    },
    Stroke {
        color: Vec4,
        width: f32,
        opacity: f32,
        mask_index: Option<usize>,
        all_masks: bool,
    },
    Levels {
        in_black: f32,
        in_white: f32,
        gamma: f32,
        out_black: f32,
        out_white: f32,
    },
}

#[derive(Clone, Debug)]
pub enum LayerStyle {
    DropShadow {
        color: Vec4,
        opacity: f32,
        angle: f32,
        distance: f32,
        size: f32,
        spread: f32,
    },
    InnerShadow {
        color: Vec4,
        opacity: f32,
        angle: f32,
        distance: f32,
        size: f32,
        choke: f32,
    },
    OuterGlow {
        color: Vec4,
        opacity: f32,
        size: f32,
        range: f32,
    },
    Stroke {
        color: Vec4,
        width: f32,
        opacity: f32,
    },
}

// Enums
#[derive(Clone, Copy, Debug)]
pub enum ColorChannel {
    R,
    G,
    B,
    A,
}

#[derive(Clone, Copy, Debug)]
pub enum Justification {
    Left,
    Right,
    Center,
}

#[derive(Clone, Copy, Debug)]
pub enum BlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
    Hue,
    Saturation,
    Color,
    Luminosity,
    // Add others as needed
}

#[derive(Clone, Copy, Debug)]
pub enum FillRule {
    NonZero,
    EvenOdd,
}

#[derive(Clone, Copy, Debug)]
pub enum LineCap {
    Butt,
    Round,
    Square,
}

#[derive(Clone, Copy, Debug)]
pub enum LineJoin {
    Miter,
    Round,
    Bevel,
}

#[derive(Clone, Copy, Debug)]
pub enum MaskMode {
    None,
    Add,
    Subtract,
    Intersect,
    Lighten,
    Darken,
    Difference,
}

#[derive(Clone, Copy, Debug)]
pub enum MatteMode {
    Alpha,
    AlphaInverted,
    Luma,
    LumaInverted,
}
