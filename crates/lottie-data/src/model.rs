use serde::{de::SeqAccess, de::DeserializeOwned, Deserialize, Deserializer, Serialize};
use std::fmt;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LottieJson {
    pub v: Option<String>,
    pub ip: f32,
    pub op: f32,
    pub fr: f32,
    pub w: u32,
    pub h: u32,
    pub layers: Vec<Layer>,
    #[serde(default)]
    pub assets: Vec<Asset>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Layer {
    // Common
    #[serde(default)]
    pub ty: u8, // 0..5
    #[serde(default)]
    pub ind: Option<u32>,
    #[serde(default)]
    pub parent: Option<u32>,
    #[serde(default)]
    pub nm: Option<String>,
    #[serde(default)]
    pub ip: f32,
    #[serde(default)]
    pub op: f32,
    #[serde(default = "default_one")]
    pub st: f32,
    #[serde(default)]
    pub ks: Transform,
    #[serde(default)]
    pub ao: Option<u32>,
    #[serde(default)]
    pub tm: Option<Property<f32>>,
    #[serde(default)]
    pub ddd: Option<u8>, // 3D Layer Flag (0=2D, 1=3D)
    #[serde(default)]
    pub pe: Option<Property<f32>>, // Perspective

    #[serde(default, rename = "masksProperties")]
    pub masks_properties: Option<Vec<MaskProperties>>,
    #[serde(default)]
    pub tt: Option<u8>,
    #[serde(default)]
    pub ef: Option<Vec<Effect>>,
    #[serde(default)]
    pub sy: Option<Vec<LayerStyle>>,

    // Type specific (flattened manually as optional fields)
    #[serde(default, rename = "refId")]
    pub ref_id: Option<String>, // PreComp, Image
    #[serde(default)]
    pub w: Option<u32>, // PreComp
    #[serde(default)]
    pub h: Option<u32>, // PreComp
    #[serde(default, rename = "sc")]
    pub color: Option<String>, // Solid color
    #[serde(default)]
    pub sw: Option<u32>, // Solid width
    #[serde(default)]
    pub sh: Option<u32>, // Solid height
    #[serde(default)]
    pub shapes: Option<Vec<Shape>>, // Shape Layer
    #[serde(default)]
    pub t: Option<TextData>, // Text Layer
}

fn default_one() -> f32 {
    1.0
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MaskProperties {
    #[serde(default)]
    pub inv: bool,
    #[serde(default)]
    pub mode: Option<String>,
    pub pt: Property<BezierPath>,
    pub o: Property<f32>,
    #[serde(default)]
    pub x: Property<f32>,
    #[serde(default)]
    pub nm: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Effect {
    #[serde(default)]
    pub ty: Option<u8>,
    #[serde(default)]
    pub nm: Option<String>,
    #[serde(default)]
    pub ix: Option<u32>,
    #[serde(default)]
    pub en: Option<u8>,
    #[serde(default)]
    pub ef: Option<Vec<EffectValue>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EffectValue {
    #[serde(default)]
    pub ty: Option<u8>,
    #[serde(default)]
    pub nm: Option<String>,
    #[serde(default)]
    pub ix: Option<u32>,
    #[serde(default, deserialize_with = "deserialize_effect_value_property")]
    pub v: Option<Property<serde_json::Value>>,
}

fn deserialize_effect_value_property<'de, D>(
    deserializer: D,
) -> Result<Option<Property<serde_json::Value>>, D::Error>
where
    D: Deserializer<'de>,
{
    let mut v = serde_json::Value::deserialize(deserializer)?;
    if v.is_null() {
        return Ok(None);
    }

    // Unwrap single-element array
    if let serde_json::Value::Array(arr) = &v {
        if arr.len() == 1 {
            v = arr[0].clone();
        }
    }

    if v.is_object() && v.get("k").is_some() {
        if let Ok(p) = serde_json::from_value::<Property<serde_json::Value>>(v.clone()) {
            return Ok(Some(p));
        }
    }

    // Fallback: Treat as static value
    Ok(Some(Property {
        k: Value::Static(v),
        a: 0,
        ix: None,
        x: None,
    }))
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LayerStyle {
    #[serde(default)]
    pub ty: Option<u8>,
    #[serde(default)]
    pub nm: Option<String>,
    #[serde(default)]
    pub c: Property<Vec<f32>>, // Color
    #[serde(default)]
    pub o: Property<f32>,      // Opacity
    #[serde(default)]
    pub a: Property<f32>,      // Angle
    #[serde(default)]
    pub d: Property<f32>,      // Distance
    #[serde(default)]
    pub s: Property<f32>,      // Size / Blur
    #[serde(default)]
    pub ch: Property<f32>,     // Choke / Spread / Range
    #[serde(default)]
    pub bm: Option<Property<f32>>, // Blend Mode
}

// Shapes

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "ty")]
pub enum Shape {
    #[serde(rename = "gr")]
    Group(GroupShape),
    #[serde(rename = "rc")]
    Rect(RectShape),
    #[serde(rename = "el")]
    Ellipse(EllipseShape),
    #[serde(rename = "fl")]
    Fill(FillShape),
    #[serde(rename = "st")]
    Stroke(StrokeShape),
    #[serde(rename = "gf")]
    GradientFill(GradientFillShape),
    #[serde(rename = "gs")]
    GradientStroke(GradientStrokeShape),
    #[serde(rename = "tr")]
    Transform(TransformShape),
    #[serde(rename = "sh")]
    Path(PathShape),
    #[serde(rename = "tm")]
    Trim(TrimShape),
    #[serde(rename = "sr")]
    Polystar(PolystarShape),
    #[serde(rename = "rp")]
    Repeater(RepeaterShape),
    #[serde(rename = "rd")]
    RoundCorners(RoundCornersShape),
    #[serde(rename = "zz")]
    ZigZag(ZigZagShape),
    #[serde(rename = "pb")]
    PuckerBloat(PuckerBloatShape),
    #[serde(rename = "tw")]
    Twist(TwistShape),
    #[serde(rename = "op")]
    OffsetPath(OffsetPathShape),
    #[serde(rename = "wgl")]
    WigglePath(WigglePathShape),
    #[serde(rename = "mm")]
    MergePaths(MergePathsShape),
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MergePathsShape {
    #[serde(default)]
    pub nm: Option<String>,
    #[serde(default)]
    pub mm: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ZigZagShape {
    #[serde(default)]
    pub nm: Option<String>,
    pub r: Property<f32>,
    pub s: Property<f32>,
    #[serde(default)]
    pub pt: Property<f32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PuckerBloatShape {
    #[serde(default)]
    pub nm: Option<String>,
    pub a: Property<f32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TwistShape {
    #[serde(default)]
    pub nm: Option<String>,
    pub a: Property<f32>,
    pub c: Property<Vec2>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OffsetPathShape {
    #[serde(default)]
    pub nm: Option<String>,
    pub a: Property<f32>,
    #[serde(default)]
    pub lj: u8,
    #[serde(default)]
    pub ml: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WigglePathShape {
    #[serde(default)]
    pub nm: Option<String>,
    pub s: Property<f32>,
    pub w: Property<f32>,
    #[serde(default)]
    pub r: Property<f32>,
    #[serde(default)]
    pub sh: Property<f32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PolystarShape {
    #[serde(default)]
    pub nm: Option<String>,
    pub p: PositionProperty,
    pub or: Property<f32>,
    #[serde(default)]
    pub os: Property<f32>,
    pub r: Property<f32>,
    pub pt: Property<f32>,
    #[serde(default)]
    pub sy: u8,
    #[serde(default)]
    pub ir: Option<Property<f32>>,
    #[serde(default)]
    pub is: Option<Property<f32>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RepeaterShape {
    #[serde(default)]
    pub nm: Option<String>,
    pub c: Property<f32>,
    pub o: Property<f32>,
    #[serde(default)]
    pub m: u8,
    pub tr: RepeaterTransform,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RepeaterTransform {
    #[serde(flatten)]
    pub t: Transform,
    #[serde(default)]
    pub so: Property<f32>,
    #[serde(default)]
    pub eo: Property<f32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RoundCornersShape {
    #[serde(default)]
    pub nm: Option<String>,
    pub r: Property<f32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GroupShape {
    #[serde(default)]
    pub nm: Option<String>,
    pub it: Vec<Shape>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RectShape {
    #[serde(default)]
    pub nm: Option<String>,
    pub s: Property<Vec2>,
    pub p: Property<Vec2>,
    pub r: Property<f32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EllipseShape {
    #[serde(default)]
    pub nm: Option<String>,
    pub s: Property<Vec2>,
    pub p: Property<Vec2>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FillShape {
    #[serde(default)]
    pub nm: Option<String>,
    pub c: Property<Vec4>,
    pub o: Property<f32>,
    #[serde(default)]
    pub r: Option<u8>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StrokeShape {
    #[serde(default)]
    pub nm: Option<String>,
    pub c: Property<Vec4>,
    pub w: Property<f32>,
    pub o: Property<f32>,
    #[serde(default)]
    pub lc: u8,
    #[serde(default)]
    pub lj: u8,
    #[serde(default)]
    pub ml: Option<f32>,
    #[serde(default)]
    pub d: Vec<DashProperty>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DashProperty {
    pub n: Option<String>,
    pub v: Property<f32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GradientFillShape {
    #[serde(default)]
    pub nm: Option<String>,
    pub o: Property<f32>,
    pub s: Property<Vec2>,
    pub e: Property<Vec2>,
    pub t: u8,
    pub g: GradientColors,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GradientStrokeShape {
    #[serde(default)]
    pub nm: Option<String>,
    pub o: Property<f32>,
    pub w: Property<f32>,
    pub s: Property<Vec2>,
    pub e: Property<Vec2>,
    pub t: u8,
    pub g: GradientColors,
    #[serde(default)]
    pub lc: u8,
    #[serde(default)]
    pub lj: u8,
    #[serde(default)]
    pub ml: Option<f32>,
    #[serde(default)]
    pub d: Vec<DashProperty>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GradientColors {
    pub p: u32,
    pub k: Property<Vec<f32>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PathShape {
    #[serde(default)]
    pub nm: Option<String>,
    pub ks: Property<BezierPath>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrimShape {
    #[serde(default)]
    pub nm: Option<String>,
    pub s: Property<f32>,
    pub e: Property<f32>,
    pub o: Property<f32>,
    #[serde(default)]
    pub m: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransformShape {
    #[serde(flatten)]
    pub t: Transform,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Transform {
    #[serde(default)]
    pub a: Property<Vec3DefaultZero>, // Anchor: Vec3, default z=0
    #[serde(default)]
    pub p: PositionProperty,          // Position: Vec3, default z=0
    #[serde(default)]
    pub s: Property<Vec3Scale>,       // Scale: Vec3, default z=100
    #[serde(default, alias = "r")]
    pub rz: Property<f32>,            // Rotation Z
    #[serde(default)]
    pub rx: Option<Property<f32>>,    // Rotation X
    #[serde(default)]
    pub ry: Option<Property<f32>>,    // Rotation Y
    #[serde(default)]
    pub or: Option<Property<Vec3DefaultZero>>, // Orientation
    #[serde(default)]
    pub o: Property<f32>,             // Opacity
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum PositionProperty {
    Unified(Property<Vec3DefaultZero>),
    Split {
        x: Property<f32>,
        y: Property<f32>,
        #[serde(default)]
        z: Option<Property<f32>>,
    },
}

impl Default for PositionProperty {
    fn default() -> Self {
        PositionProperty::Unified(Property::default())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Property<T> {
    #[serde(default)]
    pub a: u8,
    #[serde(default)]
    #[serde(bound(deserialize = "T: DeserializeOwned"))]
    pub k: Value<T>,
    #[serde(default)]
    pub ix: Option<u32>,
    #[serde(default)]
    pub x: Option<String>,
}

impl<T> Default for Property<T> {
    fn default() -> Self {
        Property {
            a: 0,
            k: Value::Default,
            ix: None,
            x: None,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub enum Value<T> {
    Default,
    Static(T),
    Animated(Vec<Keyframe<T>>),
}

impl<'de, T: DeserializeOwned> Deserialize<'de> for Value<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v = serde_json::Value::deserialize(deserializer)?;

        if v.is_null() {
            return Ok(Value::Default);
        }

        if let Ok(keyframes) = serde_json::from_value::<Vec<Keyframe<T>>>(v.clone()) {
            return Ok(Value::Animated(keyframes));
        }

        if let Ok(val) = serde_json::from_value::<T>(v.clone()) {
            return Ok(Value::Static(val));
        }

        if let Ok(vec) = serde_json::from_value::<Vec<T>>(v) {
            if let Some(first) = vec.into_iter().next() {
                return Ok(Value::Static(first));
            }
        }

        Ok(Value::Default)
    }
}

impl<T> Default for Value<T> {
    fn default() -> Self {
        Value::Default
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(bound(deserialize = "T: DeserializeOwned"))]
pub struct Keyframe<T> {
    pub t: f32,
    #[serde(default, deserialize_with = "deserialize_keyframe_value")]
    pub s: Option<T>,
    #[serde(default, deserialize_with = "deserialize_keyframe_value")]
    pub e: Option<T>,
    pub i: Option<Vec2>,
    pub o: Option<Vec2>,
    pub to: Option<Vec<f32>>,
    pub ti: Option<Vec<f32>>,
    pub h: Option<u8>,
}

fn deserialize_keyframe_value<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned,
{
    let v = serde_json::Value::deserialize(deserializer)?;
    if v.is_null() {
        return Ok(None);
    }

    if let Ok(val) = serde_json::from_value(v.clone()) {
        return Ok(Some(val));
    }

    if let Ok(vec) = serde_json::from_value::<Vec<T>>(v) {
        if let Some(first) = vec.into_iter().next() {
            return Ok(Some(first));
        }
    }

    Ok(None)
}

pub type Vec2 = [f32; 2];
pub type Vec3 = [f32; 3];
pub type Vec4 = [f32; 4];

// Wrapper for Vec3 with Z defaulting to 0.0
#[derive(Debug, Clone, Serialize)]
pub struct Vec3DefaultZero(pub Vec3);

impl Default for Vec3DefaultZero {
    fn default() -> Self {
        Vec3DefaultZero([0.0, 0.0, 0.0])
    }
}

impl<'de> Deserialize<'de> for Vec3DefaultZero {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Vec3Visitor;
        impl<'de> serde::de::Visitor<'de> for Vec3Visitor {
            type Value = Vec3DefaultZero;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a sequence of 2 or 3 floats")
            }
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let x = seq.next_element()?.unwrap_or(0.0);
                let y = seq.next_element()?.unwrap_or(0.0);
                let z = seq.next_element()?.unwrap_or(0.0);
                // Consume remaining if any (shouldn't be)
                while let Some(_) = seq.next_element::<f32>()? {}
                Ok(Vec3DefaultZero([x, y, z]))
            }
        }
        deserializer.deserialize_seq(Vec3Visitor)
    }
}

// Wrapper for Vec3 with Z defaulting to 100.0 (for Scale)
#[derive(Debug, Clone, Serialize)]
pub struct Vec3Scale(pub Vec3);

impl Default for Vec3Scale {
    fn default() -> Self {
        Vec3Scale([100.0, 100.0, 100.0])
    }
}

impl<'de> Deserialize<'de> for Vec3Scale {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Vec3ScaleVisitor;
        impl<'de> serde::de::Visitor<'de> for Vec3ScaleVisitor {
            type Value = Vec3Scale;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a sequence of 2 or 3 floats")
            }
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let x = seq.next_element()?.unwrap_or(0.0);
                let y = seq.next_element()?.unwrap_or(0.0);
                let z = seq.next_element()?.unwrap_or(100.0); // Default to 100%
                while let Some(_) = seq.next_element::<f32>()? {}
                Ok(Vec3Scale([x, y, z]))
            }
        }
        deserializer.deserialize_seq(Vec3ScaleVisitor)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct BezierPath {
    #[serde(default)]
    pub c: bool,
    #[serde(default)]
    pub i: Vec<Vec2>,
    #[serde(default)]
    pub o: Vec<Vec2>,
    #[serde(default)]
    pub v: Vec<Vec2>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Asset {
    pub id: String,
    #[serde(default)]
    pub nm: Option<String>,
    #[serde(default)]
    pub layers: Option<Vec<Layer>>,
    #[serde(default)]
    pub w: Option<u32>,
    #[serde(default)]
    pub h: Option<u32>,
    #[serde(default)]
    pub u: Option<String>,
    #[serde(default)]
    pub p: Option<String>,
    #[serde(default)]
    pub e: Option<u8>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TextData {
    pub d: Property<TextDocument>,
    #[serde(default)]
    pub a: Option<Vec<TextAnimatorData>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct TextAnimatorData {
    pub s: TextSelectorData,
    pub a: TextStyleData,
    #[serde(default)]
    pub nm: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct TextSelectorData {
    #[serde(default)]
    pub s: Option<Property<f32>>,
    #[serde(default)]
    pub e: Option<Property<f32>>,
    #[serde(default)]
    pub o: Option<Property<f32>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct TextStyleData {
    #[serde(default)]
    pub p: Option<Property<Vec3DefaultZero>>, // Updated to Vec3
    #[serde(default)]
    pub s: Option<Property<Vec3Scale>>,       // Updated to Vec3Scale
    #[serde(default)]
    pub o: Option<Property<f32>>,
    #[serde(default)]
    pub r: Option<Property<f32>>,
    #[serde(default)]
    pub t: Option<Property<f32>>,
    #[serde(default)]
    pub fc: Option<Property<Vec4>>,
    #[serde(default)]
    pub sc: Option<Property<Vec4>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct TextDocument {
    #[serde(default)]
    pub t: String,
    #[serde(default)]
    pub f: String,
    #[serde(default)]
    pub s: f32,
    #[serde(default)]
    pub j: u8,
    #[serde(default)]
    pub tr: f32,
    #[serde(default)]
    pub lh: f32,
    #[serde(default)]
    pub ls: Option<f32>,
    #[serde(default)]
    pub fc: Vec4,
    #[serde(default)]
    pub sc: Option<Vec4>,
    #[serde(default)]
    pub sw: Option<f32>,
    #[serde(default)]
    pub of: Option<bool>,
    #[serde(default)]
    pub sz: Option<Vec2>, // Size [w, h] for Box Text
    #[serde(default)]
    pub ps: Option<Vec2>, // Position [x, y] for Box Text
}
