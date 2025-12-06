pub mod animatable;
pub mod modifiers;
pub mod renderer;
#[cfg(feature = "expressions")]
pub mod expressions;

use animatable::Animator;
#[cfg(feature = "expressions")]
use expressions::ExpressionEvaluator;
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use glam::{Mat3, Mat4, Vec2, Vec3, Vec4};
use kurbo::{BezPath, Point, Shape as _};
use lottie_data::model::{self as data, LottieJson};
use modifiers::{
    GeometryModifier, OffsetPathModifier, PuckerBloatModifier, TwistModifier, WiggleModifier,
    ZigZagModifier,
};
pub use renderer::*;
use std::collections::{HashMap, HashSet};
use std::f64::consts::PI;

#[derive(Clone)]
struct PendingGeometry {
    kind: GeometryKind,
    transform: Mat3,
}

#[derive(Clone)]
enum GeometryKind {
    Path(BezPath),
    Rect { size: Vec2, pos: Vec2, radius: f32 },
    Polystar(PolystarParams),
    Ellipse { size: Vec2, pos: Vec2 },
    Merge(Vec<PendingGeometry>, MergeMode),
}

impl PendingGeometry {
    fn to_shape_geometry(&self, builder: &SceneGraphBuilder) -> ShapeGeometry {
        match &self.kind {
            GeometryKind::Merge(geoms, mode) => {
                let shapes = geoms.iter().map(|g| g.to_shape_geometry(builder)).collect();
                ShapeGeometry::Boolean {
                    mode: *mode,
                    shapes,
                }
            }
            _ => ShapeGeometry::Path(self.to_path(builder)),
        }
    }

    fn to_path(&self, builder: &SceneGraphBuilder) -> BezPath {
        let mut path = match &self.kind {
            GeometryKind::Path(p) => p.clone(),
            GeometryKind::Merge(geoms, _) => {
                let mut p = BezPath::new();
                for g in geoms {
                    p.extend(g.to_path(builder));
                }
                p
            }
            GeometryKind::Rect { size, pos, radius } => {
                let half = *size / 2.0;
                let rect = kurbo::Rect::new(
                    (pos.x - half.x) as f64,
                    (pos.y - half.y) as f64,
                    (pos.x + half.x) as f64,
                    (pos.y + half.y) as f64,
                );
                if *radius > 0.0 {
                    rect.to_rounded_rect(*radius as f64).to_path(0.1)
                } else {
                    rect.to_path(0.1)
                }
            }
            GeometryKind::Ellipse { size, pos } => {
                let half = *size / 2.0;
                let ellipse = kurbo::Ellipse::new(
                    (pos.x as f64, pos.y as f64),
                    (half.x as f64, half.y as f64),
                    0.0,
                );
                ellipse.to_path(0.1)
            }
            GeometryKind::Polystar(params) => builder.generate_polystar_path(params),
        };

        let m = self.transform.to_cols_array();
        let affine = kurbo::Affine::new([
            m[0] as f64,
            m[1] as f64,
            m[3] as f64,
            m[4] as f64,
            m[6] as f64,
            m[7] as f64,
        ]);
        path.apply_affine(affine);
        path
    }
}

#[derive(Clone, Copy)]
struct PolystarParams {
    pos: Vec2,
    outer_radius: f32,
    inner_radius: f32,
    outer_roundness: f32,
    inner_roundness: f32,
    rotation: f32,
    points: f32,
    kind: u8,           // 1=star, 2=polygon
    corner_radius: f32, // From RoundCorners modifier
}

pub enum ImageSource {
    Data(Vec<u8>), // Encoded bytes (PNG/JPG)
}

pub trait TextMeasurer: Send + Sync {
    /// Returns the width of the text string for the given font and size.
    fn measure(&self, text: &str, font_family: &str, size: f32) -> f32;
}

pub struct LottiePlayer {
    pub model: Option<LottieJson>,
    pub current_frame: f32,
    pub width: f32,
    pub height: f32,
    pub duration_frames: f32,
    pub frame_rate: f32,
    pub assets: HashMap<String, ImageSource>,
    pub text_measurer: Option<Box<dyn TextMeasurer>>,
    #[cfg(feature = "expressions")]
    pub expression_evaluator: Option<ExpressionEvaluator>,
}

impl LottiePlayer {
    pub fn new() -> Self {
        #[cfg(feature = "expressions")]
        let expression_evaluator = Some(ExpressionEvaluator::new());
        Self {
            model: None,
            current_frame: 0.0,
            width: 500.0,
            height: 500.0,
            duration_frames: 60.0,
            frame_rate: 60.0,
            assets: HashMap::new(),
            text_measurer: None,
            #[cfg(feature = "expressions")]
            expression_evaluator,
        }
    }

    pub fn set_text_measurer(&mut self, measurer: Box<dyn TextMeasurer>) {
        self.text_measurer = Some(measurer);
    }

    pub fn set_asset(&mut self, id: String, data: Vec<u8>) {
        self.assets.insert(id, ImageSource::Data(data));
    }

    pub fn load(&mut self, data: LottieJson) {
        self.width = data.w as f32;
        self.height = data.h as f32;
        self.frame_rate = data.fr;
        self.duration_frames = data.op - data.ip;
        self.current_frame = data.ip; // Start at in-point
        self.model = Some(data);
    }

    pub fn advance(&mut self, dt: f32) {
        if self.model.is_none() {
            return;
        }
        // dt is in seconds
        let frames = dt * self.frame_rate;
        self.current_frame += frames;

        // Loop
        if self.current_frame >= self.model.as_ref().unwrap().op {
            let duration = self.model.as_ref().unwrap().op - self.model.as_ref().unwrap().ip;
            self.current_frame = self.model.as_ref().unwrap().ip
                + (self.current_frame - self.model.as_ref().unwrap().op) % duration;
        }
    }

    pub fn render_tree(&mut self) -> RenderTree {
        if let Some(model) = &self.model {
            #[cfg(feature = "expressions")]
            let evaluator = self.expression_evaluator.as_mut();
            #[cfg(not(feature = "expressions"))]
            let evaluator: Option<&mut ()> = None;

            let mut builder = SceneGraphBuilder::new(
                model,
                self.current_frame,
                &self.assets,
                self.text_measurer.as_deref(),
            );
            builder.build(evaluator)
        } else {
            // Return empty tree
            RenderTree::mock_sample()
        }
    }
}

struct SceneGraphBuilder<'a> {
    model: &'a LottieJson,
    frame: f32,
    assets: HashMap<String, &'a data::Asset>,
    external_assets: &'a HashMap<String, ImageSource>,
    text_measurer: Option<&'a dyn TextMeasurer>,
}

impl<'a> SceneGraphBuilder<'a> {
    fn new(
        model: &'a LottieJson,
        frame: f32,
        external_assets: &'a HashMap<String, ImageSource>,
        text_measurer: Option<&'a dyn TextMeasurer>,
    ) -> Self {
        let mut assets = HashMap::new();
        for asset in &model.assets {
            assets.insert(asset.id.clone(), asset);
        }
        Self {
            model,
            frame,
            assets,
            external_assets,
            text_measurer,
        }
    }

    fn build(&mut self, #[cfg(feature = "expressions")] mut evaluator: Option<&mut ExpressionEvaluator>, #[cfg(not(feature = "expressions"))] mut evaluator: Option<&mut ()>) -> RenderTree {
        let mut layer_map = HashMap::new();
        for layer in &self.model.layers {
            if let Some(ind) = layer.ind {
                layer_map.insert(ind, layer);
            }
        }

        let (view_matrix, projection_matrix) = self.get_camera_matrices(&self.model.layers, &layer_map, evaluator.as_deref_mut());

        let root_node = self.build_composition(&self.model.layers, &layer_map, evaluator);

        RenderTree {
            width: self.model.w as f32,
            height: self.model.h as f32,
            root: root_node,
            view_matrix,
            projection_matrix,
        }
    }

    fn get_camera_matrices(&self, layers: &'a [data::Layer], map: &HashMap<u32, &'a data::Layer>, #[cfg(feature = "expressions")] mut evaluator: Option<&mut ExpressionEvaluator>, #[cfg(not(feature = "expressions"))] mut evaluator: Option<&mut ()>) -> (Mat4, Mat4) {
        // Step 1: Find Active Camera (Top-most, ty=13, visible)
        let mut camera_layer = None;
        for layer in layers {
            if layer.ty == 13 {
                 // Check visibility
                 if self.frame >= layer.ip && self.frame < layer.op {
                     camera_layer = Some(layer);
                     break; // Top-most found
                 }
            }
        }

        if let Some(cam) = camera_layer {
            // Step 2: Compute View Matrix
            let cam_transform = self.resolve_transform(cam, map, evaluator.as_deref_mut());
            let view_matrix = cam_transform.inverse();

            // Step 3: Compute Projection Matrix
            let pe = if let Some(prop) = &cam.pe {
                Animator::resolve(prop, self.frame - cam.st, |v| *v, 0.0, evaluator, self.model.fr)
            } else {
                0.0
            };

            let perspective = if pe > 0.0 { pe } else { 1000.0 }; // Default ?

            // FOV Calculation
            // pe is distance. Height is comp height.
            // tan(fov/2) = (height/2) / pe
            // fov = 2 * atan(height / (2 * pe))
            let fov = 2.0 * (self.model.h as f32 / (2.0 * perspective)).atan();

            let aspect = self.model.w as f32 / self.model.h as f32;
            let near = 0.1;
            let far = 10000.0;

            let projection_matrix = Mat4::perspective_rh(fov, aspect, near, far);

            (view_matrix, projection_matrix)

        } else {
            // Default 2D View
            (Mat4::IDENTITY, Mat4::IDENTITY)
        }
    }

    fn build_composition(&mut self, layers: &'a [data::Layer], layer_map: &HashMap<u32, &'a data::Layer>, #[cfg(feature = "expressions")] mut evaluator: Option<&mut ExpressionEvaluator>, #[cfg(not(feature = "expressions"))] mut evaluator: Option<&mut ()>) -> RenderNode {
        let mut nodes = Vec::new();
        let mut consumed_indices = HashSet::new();
        let len = layers.len();

        for i in (0..len).rev() {
            if consumed_indices.contains(&i) {
                continue;
            }

            let layer = &layers[i];

            if let Some(tt) = layer.tt {
                if i > 0 {
                    let matte_idx = i - 1;
                    if !consumed_indices.contains(&matte_idx) {
                        consumed_indices.insert(matte_idx);
                        let matte_layer = &layers[matte_idx];

                        if let Some(mut content_node) = self.process_layer(layer, layer_map, evaluator.as_deref_mut()) {
                            if let Some(matte_node) = self.process_layer(matte_layer, layer_map, evaluator.as_deref_mut()) {
                                let mode = match tt {
                                    1 => MatteMode::Alpha,
                                    2 => MatteMode::AlphaInverted,
                                    3 => MatteMode::Luma,
                                    4 => MatteMode::LumaInverted,
                                    _ => MatteMode::Alpha,
                                };
                                content_node.matte = Some(Box::new(Matte {
                                    mode,
                                    node: matte_node,
                                }));
                            }
                            nodes.push(content_node);
                        }
                        continue;
                    }
                }
            }

            if let Some(node) = self.process_layer(layer, layer_map, evaluator.as_deref_mut()) {
                nodes.push(node);
            }
        }

        RenderNode {
            transform: Mat4::IDENTITY,
            alpha: 1.0,
            blend_mode: BlendMode::Normal,
            content: NodeContent::Group(nodes),
            masks: vec![], styles: vec![],
            matte: None,
            effects: vec![],
            is_adjustment_layer: false,
        }
    }

    fn process_layer(
        &mut self,
        layer: &'a data::Layer,
        layer_map: &HashMap<u32, &'a data::Layer>,
        #[cfg(feature = "expressions")] mut evaluator: Option<&mut ExpressionEvaluator>,
        #[cfg(not(feature = "expressions"))] mut evaluator: Option<&mut ()>,
    ) -> Option<RenderNode> {
        let is_adjustment_layer = layer.ao == Some(1);

        if self.frame < layer.ip || self.frame >= layer.op {
            return None;
        }

        let transform = self.resolve_transform(layer, layer_map, evaluator.as_deref_mut());

        let opacity = Animator::resolve(&layer.ks.o, self.frame - layer.st, |v| *v / 100.0, 1.0, evaluator.as_deref_mut(), self.model.fr);

        let content = if let Some(shapes) = &layer.shapes {
            let shape_nodes = self.process_shapes(shapes, self.frame - layer.st, evaluator.as_deref_mut());
            NodeContent::Group(shape_nodes)
        } else if let Some(text_data) = &layer.t {
            // Text Layer
            let doc = Animator::resolve(
                &text_data.d,
                self.frame - layer.st,
                |v| v.clone(),
                data::TextDocument::default(),
                evaluator.as_deref_mut(), self.model.fr
            );

            let base_fill_color = Vec4::new(doc.fc[0], doc.fc[1], doc.fc[2], 1.0);
            let base_stroke_color = if let Some(sc) = &doc.sc {
                Some(Vec4::new(sc[0], sc[1], sc[2], 1.0))
            } else {
                None
            };

            let chars: Vec<char> = doc.t.chars().collect();
            let char_count = chars.len();

            let mut glyphs = Vec::with_capacity(char_count);

            for &c in &chars {
                let g = RenderGlyph {
                    character: c,
                    pos: Vec3::ZERO,
                    scale: Vec3::ONE,
                    rotation: Vec3::ZERO,
                    tracking: 0.0,
                    alpha: 1.0,
                    fill: Some(Fill {
                        paint: Paint::Solid(base_fill_color),
                        opacity: 1.0,
                        rule: FillRule::NonZero,
                    }),
                    stroke: if let Some(col) = base_stroke_color {
                        Some(Stroke {
                            paint: Paint::Solid(col),
                            width: doc.sw.unwrap_or(1.0),
                            opacity: 1.0,
                            cap: LineCap::Round,
                            join: LineJoin::Round,
                            miter_limit: None,
                            dash: None,
                        })
                    } else {
                        None
                    },
                };
                glyphs.push(g);
            }

            if let Some(animators) = &text_data.a {
                for animator in animators {
                    let sel = &animator.s;
                    let start_val = Animator::resolve(
                        sel.s.as_ref().unwrap_or(&data::Property::default()),
                        self.frame - layer.st,
                        |v| *v,
                        0.0,
                        evaluator.as_deref_mut(), self.model.fr
                    );
                    let end_val = Animator::resolve(
                        sel.e.as_ref().unwrap_or(&data::Property::default()),
                        self.frame - layer.st,
                        |v| *v,
                        100.0,
                        evaluator.as_deref_mut(), self.model.fr
                    );
                    let offset_val = Animator::resolve(
                        sel.o.as_ref().unwrap_or(&data::Property::default()),
                        self.frame - layer.st,
                        |v| *v,
                        0.0,
                        evaluator.as_deref_mut(), self.model.fr
                    );

                    let start_idx = char_count as f32 * start_val / 100.0;
                    let end_idx = char_count as f32 * end_val / 100.0;
                    let offset_idx = char_count as f32 * offset_val / 100.0;

                    let style = &animator.a;
                    let p_delta = Animator::resolve(
                        style.p.as_ref().unwrap_or(&data::Property::default()),
                        self.frame - layer.st,
                        |v| Vec3::from(v.0),
                        Vec3::ZERO,
                        evaluator.as_deref_mut(), self.model.fr
                    );
                    let s_val = Animator::resolve(
                        style.s.as_ref().unwrap_or(&data::Property::default()),
                        self.frame - layer.st,
                        |v| Vec3::from(v.0) / 100.0,
                        Vec3::ONE,
                        evaluator.as_deref_mut(), self.model.fr
                    );
                    let o_val = Animator::resolve(
                        style.o.as_ref().unwrap_or(&data::Property::default()),
                        self.frame - layer.st,
                        |v| *v,
                        100.0,
                        evaluator.as_deref_mut(), self.model.fr
                    );
                    // RZ
                    let r_val = Animator::resolve(
                        style.r.as_ref().unwrap_or(&data::Property::default()),
                        self.frame - layer.st,
                        |v| *v,
                        0.0,
                        evaluator.as_deref_mut(), self.model.fr
                    );

                    // Tracking
                    let t_val = Animator::resolve(
                        style.t.as_ref().unwrap_or(&data::Property::default()),
                        self.frame - layer.st,
                        |v| *v,
                        0.0,
                        evaluator.as_deref_mut(), self.model.fr
                    );

                    let fc_val = if let Some(fc_prop) = &style.fc {
                        Some(Animator::resolve(
                            fc_prop,
                            self.frame - layer.st,
                            |v| Vec4::from_slice(v),
                            Vec4::ONE,
                            evaluator.as_deref_mut(), self.model.fr
                        ))
                    } else {
                        None
                    };

                    let sc_val = if let Some(sc_prop) = &style.sc {
                        Some(Animator::resolve(
                            sc_prop,
                            self.frame - layer.st,
                            |v| Vec4::from_slice(v),
                            Vec4::ONE,
                            evaluator.as_deref_mut(), self.model.fr
                        ))
                    } else {
                        None
                    };

                    for (i, glyph) in glyphs.iter_mut().enumerate() {
                        let idx = i as f32;
                        let effective_start = start_idx + offset_idx;
                        let effective_end = end_idx + offset_idx;

                        let overlap_start = idx.max(effective_start);
                        let overlap_end = (idx + 1.0).min(effective_end);

                        let factor = (overlap_end - overlap_start).max(0.0).min(1.0);

                        if factor > 0.0 {
                            glyph.pos += p_delta * factor;

                            // Scale mixing
                            let scale_factor = Vec3::ONE + (s_val - Vec3::ONE) * factor;
                            glyph.scale *= scale_factor;

                            // Rotation (RZ only for now, mapped to Z component)
                            glyph.rotation.z += r_val.to_radians() * factor;

                            glyph.tracking += t_val * factor;

                            let target_alpha = o_val / 100.0;
                            let alpha_mult = 1.0 + (target_alpha - 1.0) * factor;
                            glyph.alpha *= alpha_mult;

                            if let Some(target_color) = fc_val {
                                if let Some(fill) = &mut glyph.fill {
                                    if let Paint::Solid(current_color) = &mut fill.paint {
                                        *current_color = current_color.lerp(target_color, factor);
                                    }
                                }
                            }

                            if let Some(target_color) = sc_val {
                                if let Some(stroke) = &mut glyph.stroke {
                                    if let Paint::Solid(current_color) = &mut stroke.paint {
                                        *current_color = current_color.lerp(target_color, factor);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Layout
            if let Some(measurer) = self.text_measurer {
                let box_size = doc.sz.map(|v| Vec2::from_slice(&v));
                let box_pos = doc.ps.map(|v| Vec2::from_slice(&v)).unwrap_or(Vec2::ZERO);
                let tracking_val = doc.tr;

                if let Some(sz) = box_size {
                    // Box Text
                    let box_width = sz.x;
                    let mut lines: Vec<Vec<usize>> = Vec::new();
                    let mut current_line: Vec<usize> = Vec::new();
                    let mut current_line_width = 0.0;

                    let mut i = 0;
                    while i < glyphs.len() {
                        let start = i;
                        let mut end = i;
                        let mut word_width = 0.0;

                        while end < glyphs.len() {
                            let g = &glyphs[end];
                            let char_str = g.character.to_string();
                            let w = measurer.measure(&char_str, &doc.f, doc.s);
                            let advance = w + tracking_val + g.tracking;
                            word_width += advance;
                            let is_space = g.character == ' ';
                            let is_newline = g.character == '\n';
                            end += 1;
                            if is_space || is_newline {
                                break;
                            }
                        }

                        let is_newline = if end > 0 { glyphs[end-1].character == '\n' } else { false };

                        if is_newline {
                             for k in start..end { current_line.push(k); }
                             lines.push(current_line);
                             current_line = Vec::new();
                             current_line_width = 0.0;
                        } else {
                            if !current_line.is_empty() && current_line_width + word_width > box_width {
                                lines.push(current_line);
                                current_line = Vec::new();
                                current_line_width = 0.0;
                            }
                            for k in start..end { current_line.push(k); }
                            current_line_width += word_width;
                        }
                        i = end;
                    }
                    if !current_line.is_empty() { lines.push(current_line); }

                    let mut current_y = box_pos.y;
                    for line_indices in lines {
                         let mut line_width = 0.0;
                         let mut advances = Vec::new();

                         for &idx in &line_indices {
                             let g = &glyphs[idx];
                             let w = measurer.measure(&g.character.to_string(), &doc.f, doc.s);
                             let advance = w + tracking_val + g.tracking;
                             advances.push(advance);
                             line_width += advance;
                         }

                         let align_width = line_width;
                         let start_x = match doc.j {
                             1 => box_width - align_width,
                             2 => (box_width - align_width) / 2.0,
                             _ => 0.0,
                         };

                         let mut x = box_pos.x + start_x;
                         for (k, &idx) in line_indices.iter().enumerate() {
                             let g = &mut glyphs[idx];
                             g.pos += Vec3::new(x, current_y, 0.0);
                             x += advances[k];
                         }
                         current_y += doc.lh;
                    }

                } else {
                    // Point Text
                    let mut current_y = 0.0;
                    let mut lines: Vec<Vec<usize>> = Vec::new();
                    let mut current_line = Vec::new();

                    for (i, g) in glyphs.iter().enumerate() {
                        if g.character == '\n' {
                            lines.push(current_line);
                            current_line = Vec::new();
                        } else {
                            current_line.push(i);
                        }
                    }
                    lines.push(current_line);

                    for line_indices in lines {
                        let mut line_width = 0.0;
                        let mut advances = Vec::new();
                        for &idx in &line_indices {
                             let g = &glyphs[idx];
                             let w = measurer.measure(&g.character.to_string(), &doc.f, doc.s);
                             let advance = w + tracking_val + g.tracking;
                             advances.push(advance);
                             line_width += advance;
                        }

                        let start_x = match doc.j {
                            1 => -line_width,
                            2 => -line_width / 2.0,
                            _ => 0.0,
                        };

                        let mut x = start_x;
                        for (k, &idx) in line_indices.iter().enumerate() {
                             let g = &mut glyphs[idx];
                             g.pos += Vec3::new(x, current_y, 0.0);
                             x += advances[k];
                         }
                         current_y += doc.lh;
                    }
                }
            } else {
                 let fixed_width = 10.0;
                 let mut x = 0.0;
                 let mut y = 0.0;
                 for g in &mut glyphs {
                     if g.character == '\n' {
                         x = 0.0;
                         y += doc.lh;
                     } else {
                         g.pos += Vec3::new(x, y, 0.0);
                         x += fixed_width + doc.tr + g.tracking;
                     }
                 }
            }

            NodeContent::Text(Text {
                glyphs,
                font_family: doc.f,
                size: doc.s,
                justify: match doc.j {
                    1 => Justification::Right,
                    2 => Justification::Center,
                    _ => Justification::Left,
                },
                tracking: doc.tr,
                line_height: doc.lh,
            })
        } else if let Some(ref_id) = &layer.ref_id {
            if let Some(asset) = self.assets.get(ref_id) {
                if let Some(layers) = &asset.layers {
                    let local_frame = if let Some(tm_prop) = &layer.tm {
                        let tm_sec = Animator::resolve(tm_prop, self.frame, |v| *v, 0.0, evaluator.as_deref_mut(), self.model.fr);
                        tm_sec * self.model.fr
                    } else {
                        self.frame - layer.st
                    };

                    let mut sub_layer_map = HashMap::new();
                    for l in layers {
                        if let Some(ind) = l.ind {
                            sub_layer_map.insert(ind, l);
                        }
                    }

                    let mut sub_builder = SceneGraphBuilder::new(
                        self.model,
                        local_frame,
                        self.external_assets,
                        self.text_measurer,
                    );
                    let root = sub_builder.build_composition(layers, &sub_layer_map, evaluator.as_deref_mut());
                    root.content
                } else {
                    let data = if let Some(ImageSource::Data(bytes)) =
                        self.external_assets.get(&asset.id)
                    {
                        Some(bytes.clone())
                    } else if let Some(p) = &asset.p {
                        if p.starts_with("data:image/") && p.contains(";base64,") {
                            let split: Vec<&str> = p.splitn(2, ',').collect();
                            if split.len() > 1 {
                                match BASE64_STANDARD.decode(split[1]) {
                                    Ok(bytes) => Some(bytes),
                                    Err(_) => None
                                }
                            } else {
                                None
                            }
                        } else {
                            if let Ok(bytes) = std::fs::read(p) {
                                Some(bytes)
                            } else {
                                None
                            }
                        }
                    } else {
                        None
                    };

                    NodeContent::Image(Image {
                        data,
                        width: asset.w.unwrap_or(100),
                        height: asset.h.unwrap_or(100),
                        id: Some(asset.id.clone()),
                    })
                }
            } else {
                NodeContent::Group(vec![])
            }
        } else if let Some(color) = &layer.color {
            let w = layer.sw.unwrap_or(100) as f64;
            let h = layer.sh.unwrap_or(100) as f64;
            let mut path = BezPath::new();
            path.move_to((0.0, 0.0));
            path.line_to((w, 0.0));
            path.line_to((w, h));
            path.line_to((0.0, h));
            path.close_path();

            let c_str = color.trim_start_matches('#');
            let r = u8::from_str_radix(&c_str[0..2], 16).unwrap_or(0) as f32 / 255.0;
            let g = u8::from_str_radix(&c_str[2..4], 16).unwrap_or(0) as f32 / 255.0;
            let b = u8::from_str_radix(&c_str[4..6], 16).unwrap_or(0) as f32 / 255.0;

            NodeContent::Shape(renderer::Shape {
                geometry: renderer::ShapeGeometry::Path(path),
                fill: Some(Fill {
                    paint: Paint::Solid(Vec4::new(r, g, b, 1.0)),
                    opacity: 1.0,
                    rule: FillRule::NonZero,
                }),
                stroke: None,
                trim: None,
            })
        } else {
            NodeContent::Group(vec![])
        };

        let masks = if let Some(props) = &layer.masks_properties {
            self.process_masks(props, self.frame - layer.st, evaluator.as_deref_mut())
        } else {
            vec![]
        };

        let effects = self.process_effects(layer, evaluator.as_deref_mut());
        let styles = self.process_layer_styles(layer, evaluator.as_deref_mut());

        Some(RenderNode {
            transform,
            alpha: opacity,
            blend_mode: BlendMode::Normal,
            content,
            masks,
            matte: None,
            effects,
            styles,
            is_adjustment_layer,
        })
    }

    fn process_layer_styles(&self, layer: &data::Layer, #[cfg(feature = "expressions")] mut evaluator: Option<&mut ExpressionEvaluator>, #[cfg(not(feature = "expressions"))] mut evaluator: Option<&mut ()>) -> Vec<LayerStyle> {
        let mut styles = Vec::new();
        if let Some(sy_list) = &layer.sy {
            for sy in sy_list {
                let ty = sy.ty.unwrap_or(8);
                let mut kind = None;
                if ty == 0 { kind = Some("DropShadow"); }
                else if ty == 1 { kind = Some("InnerShadow"); }
                else if ty == 2 { kind = Some("OuterGlow"); }
                else if let Some(nm) = &sy.nm {
                    if nm.contains("Stroke") { kind = Some("Stroke"); }
                }

                if kind.is_none() {
                    if ty == 3 || ty == 8 {
                        kind = Some("Stroke");
                    }
                }

                if let Some(k) = kind {
                    match k {
                        "DropShadow" => {
                             let color = self.resolve_json_vec4_arr(&sy.c, self.frame - layer.st, evaluator.as_deref_mut());
                             let opacity = Animator::resolve(&sy.o, self.frame - layer.st, |v| *v / 100.0, 1.0, evaluator.as_deref_mut(), self.model.fr);
                             let angle = Animator::resolve(&sy.a, self.frame - layer.st, |v| *v, 0.0, evaluator.as_deref_mut(), self.model.fr);
                             let distance = Animator::resolve(&sy.d, self.frame - layer.st, |v| *v, 0.0, evaluator.as_deref_mut(), self.model.fr);
                             let size = Animator::resolve(&sy.s, self.frame - layer.st, |v| *v, 0.0, evaluator.as_deref_mut(), self.model.fr);
                             let spread = Animator::resolve(&sy.ch, self.frame - layer.st, |v| *v, 0.0, evaluator.as_deref_mut(), self.model.fr);
                             styles.push(LayerStyle::DropShadow {
                                 color, opacity, angle, distance, size, spread
                             });
                        },
                        "InnerShadow" => {
                             let color = self.resolve_json_vec4_arr(&sy.c, self.frame - layer.st, evaluator.as_deref_mut());
                             let opacity = Animator::resolve(&sy.o, self.frame - layer.st, |v| *v / 100.0, 1.0, evaluator.as_deref_mut(), self.model.fr);
                             let angle = Animator::resolve(&sy.a, self.frame - layer.st, |v| *v, 0.0, evaluator.as_deref_mut(), self.model.fr);
                             let distance = Animator::resolve(&sy.d, self.frame - layer.st, |v| *v, 0.0, evaluator.as_deref_mut(), self.model.fr);
                             let size = Animator::resolve(&sy.s, self.frame - layer.st, |v| *v, 0.0, evaluator.as_deref_mut(), self.model.fr);
                             let choke = Animator::resolve(&sy.ch, self.frame - layer.st, |v| *v, 0.0, evaluator.as_deref_mut(), self.model.fr);
                             styles.push(LayerStyle::InnerShadow {
                                 color, opacity, angle, distance, size, choke
                             });
                        },
                        "OuterGlow" => {
                             let color = self.resolve_json_vec4_arr(&sy.c, self.frame - layer.st, evaluator.as_deref_mut());
                             let opacity = Animator::resolve(&sy.o, self.frame - layer.st, |v| *v / 100.0, 1.0, evaluator.as_deref_mut(), self.model.fr);
                             let size = Animator::resolve(&sy.s, self.frame - layer.st, |v| *v, 0.0, evaluator.as_deref_mut(), self.model.fr);
                             let range = Animator::resolve(&sy.ch, self.frame - layer.st, |v| *v, 0.0, evaluator.as_deref_mut(), self.model.fr);
                             styles.push(LayerStyle::OuterGlow {
                                 color, opacity, size, range
                             });
                        },
                        "Stroke" => {
                             let color = self.resolve_json_vec4_arr(&sy.c, self.frame - layer.st, evaluator.as_deref_mut());
                             let opacity = Animator::resolve(&sy.o, self.frame - layer.st, |v| *v / 100.0, 1.0, evaluator.as_deref_mut(), self.model.fr);
                             let width = Animator::resolve(&sy.s, self.frame - layer.st, |v| *v, 0.0, evaluator.as_deref_mut(), self.model.fr);
                             styles.push(LayerStyle::Stroke {
                                 color, width, opacity
                             });
                        },
                        _ => {}
                    }
                }
            }
        }
        styles
    }

    fn process_effects(&self, layer: &data::Layer, #[cfg(feature = "expressions")] mut evaluator: Option<&mut ExpressionEvaluator>, #[cfg(not(feature = "expressions"))] mut evaluator: Option<&mut ()>) -> Vec<Effect> {
        let mut effects = Vec::new();
        if let Some(ef_list) = &layer.ef {
            for ef in ef_list {
                if let Some(en) = ef.en { if en == 0 { continue; } }
                let ty = ef.ty.unwrap_or(0);
                let values = if let Some(vals) = &ef.ef { vals } else { continue; };

                match ty {
                    20 => {
                        let black = self.find_effect_vec4(values, 0, "Black", layer, evaluator.as_deref_mut());
                        let white = self.find_effect_vec4(values, 1, "White", layer, evaluator.as_deref_mut());
                        let amount = self.find_effect_scalar(values, 2, "Intensity", layer, evaluator.as_deref_mut()) / 100.0;
                        effects.push(Effect::Tint { black, white, amount });
                    }
                    21 => {
                        let color = self.find_effect_vec4(values, 2, "Color", layer, evaluator.as_deref_mut());
                        let opacity = self.find_effect_scalar(values, 6, "Opacity", layer, evaluator.as_deref_mut()) / 100.0;
                        effects.push(Effect::Fill { color, opacity });
                    }
                    22 => {
                        let color = self.find_effect_vec4(values, 3, "Color", layer, evaluator.as_deref_mut());
                        let width = self.find_effect_scalar(values, 4, "Brush Size", layer, evaluator.as_deref_mut());
                        let opacity = self.find_effect_scalar(values, 6, "Opacity", layer, evaluator.as_deref_mut()) / 100.0;
                        let all_masks_val = self.find_effect_scalar(values, 9999, "All Masks", layer, evaluator.as_deref_mut());
                        let all_masks = all_masks_val > 0.5;
                        let mut mask_idx_val = self.find_effect_scalar(values, 9999, "Path", layer, evaluator.as_deref_mut());
                        if mask_idx_val < 0.5 { mask_idx_val = self.find_effect_scalar(values, 9999, "Mask", layer, evaluator.as_deref_mut()); }
                        let mask_index = if mask_idx_val >= 0.5 { Some(mask_idx_val.round() as usize) } else { None };
                        effects.push(Effect::Stroke { color, width, opacity, mask_index, all_masks });
                    }
                    23 => {
                        let highlights = self.find_effect_vec4(values, 0, "bright", layer, evaluator.as_deref_mut());
                        let midtones = self.find_effect_vec4(values, 1, "mid", layer, evaluator.as_deref_mut());
                        let shadows = self.find_effect_vec4(values, 2, "dark", layer, evaluator.as_deref_mut());
                        effects.push(Effect::Tritone { highlights, midtones, shadows });
                    }
                    24 => {
                        let in_black = self.find_effect_scalar(values, 3, "inblack", layer, evaluator.as_deref_mut());
                        let in_white = self.find_effect_scalar(values, 4, "inwhite", layer, evaluator.as_deref_mut());
                        let gamma = self.find_effect_scalar(values, 5, "gamma", layer, evaluator.as_deref_mut());
                        let out_black = self.find_effect_scalar(values, 6, "outblack", layer, evaluator.as_deref_mut());
                        let out_white = self.find_effect_scalar(values, 7, "outwhite", layer, evaluator.as_deref_mut());
                        effects.push(Effect::Levels { in_black, in_white, gamma, out_black, out_white });
                    }
                    _ => {}
                }
            }
        }
        effects
    }

    fn find_effect_scalar(&self, values: &[data::EffectValue], index: usize, name_hint: &str, layer: &data::Layer, #[cfg(feature = "expressions")] mut evaluator: Option<&mut ExpressionEvaluator>, #[cfg(not(feature = "expressions"))] mut evaluator: Option<&mut ()>) -> f32 {
        if let Some(v) = values.get(index) {
            if let Some(prop) = &v.v {
                return self.resolve_json_scalar(prop, self.frame - layer.st, evaluator.as_deref_mut());
            }
        }
        for v in values {
            if let Some(nm) = &v.nm {
                if nm.contains(name_hint) {
                    if let Some(prop) = &v.v {
                        return self.resolve_json_scalar(prop, self.frame - layer.st, evaluator.as_deref_mut());
                    }
                }
            }
        }
        0.0
    }

    fn find_effect_vec4(&self, values: &[data::EffectValue], index: usize, name_hint: &str, layer: &data::Layer, #[cfg(feature = "expressions")] mut evaluator: Option<&mut ExpressionEvaluator>, #[cfg(not(feature = "expressions"))] mut evaluator: Option<&mut ()>) -> Vec4 {
        if let Some(v) = values.get(index) {
            if let Some(prop) = &v.v {
                return self.resolve_json_vec4(prop, self.frame - layer.st, evaluator.as_deref_mut());
            }
        }
        for v in values {
            if let Some(nm) = &v.nm {
                if nm.contains(name_hint) {
                    if let Some(prop) = &v.v {
                        return self.resolve_json_vec4(prop, self.frame - layer.st, evaluator.as_deref_mut());
                    }
                }
            }
        }
        Vec4::ZERO
    }

    fn resolve_json_scalar(&self, prop: &data::Property<serde_json::Value>, frame: f32, #[cfg(feature = "expressions")] evaluator: Option<&mut ExpressionEvaluator>, #[cfg(not(feature = "expressions"))] mut evaluator: Option<&mut ()>) -> f32 {
        Animator::resolve(prop, frame, |v| {
            if let Some(n) = v.as_f64() { n as f32 }
            else if let Some(arr) = v.as_array() { arr.get(0).and_then(|x| x.as_f64()).unwrap_or(0.0) as f32 }
            else { 0.0 }
        }, 0.0, evaluator, self.model.fr)
    }

    fn resolve_json_vec4(&self, prop: &data::Property<serde_json::Value>, frame: f32, #[cfg(feature = "expressions")] evaluator: Option<&mut ExpressionEvaluator>, #[cfg(not(feature = "expressions"))] mut evaluator: Option<&mut ()>) -> Vec4 {
        Animator::resolve(prop, frame, |v| {
            if let Some(arr) = v.as_array() {
                let r = arr.get(0).and_then(|x| x.as_f64()).unwrap_or(0.0) as f32;
                let g = arr.get(1).and_then(|x| x.as_f64()).unwrap_or(0.0) as f32;
                let b = arr.get(2).and_then(|x| x.as_f64()).unwrap_or(0.0) as f32;
                let a = arr.get(3).and_then(|x| x.as_f64()).unwrap_or(1.0) as f32;
                Vec4::new(r, g, b, a)
            } else { Vec4::ZERO }
        }, Vec4::ZERO, evaluator, self.model.fr)
    }

    fn resolve_json_vec4_arr(&self, prop: &data::Property<Vec<f32>>, frame: f32, #[cfg(feature = "expressions")] evaluator: Option<&mut ExpressionEvaluator>, #[cfg(not(feature = "expressions"))] mut evaluator: Option<&mut ()>) -> Vec4 {
        Animator::resolve(prop, frame, |v| {
            if v.len() >= 4 { Vec4::new(v[0], v[1], v[2], v[3]) }
            else if v.len() >= 3 { Vec4::new(v[0], v[1], v[2], 1.0) }
            else { Vec4::ZERO }
        }, Vec4::ONE, evaluator, self.model.fr)
    }

    fn resolve_transform(&self, layer: &data::Layer, map: &HashMap<u32, &data::Layer>, #[cfg(feature = "expressions")] mut evaluator: Option<&mut ExpressionEvaluator>, #[cfg(not(feature = "expressions"))] mut evaluator: Option<&mut ()>) -> Mat4 {
        let local = self.get_layer_transform(layer, evaluator.as_deref_mut());
        if let Some(parent_ind) = layer.parent {
            if let Some(parent) = map.get(&parent_ind) {
                return self.resolve_transform(parent, map, evaluator) * local;
            }
        }
        local
    }

    fn get_layer_transform(&self, layer: &data::Layer, #[cfg(feature = "expressions")] mut evaluator: Option<&mut ExpressionEvaluator>, #[cfg(not(feature = "expressions"))] mut evaluator: Option<&mut ()>) -> Mat4 {
        let t_frame = self.frame - layer.st;
        let ks = &layer.ks;

        let is_3d = layer.ddd.unwrap_or(0) == 1 || layer.ty == 13;

        // Camera LookAt Check
        if layer.ty == 13 {
             // Position
             let pos = match &ks.p {
                 data::PositionProperty::Unified(p) => Animator::resolve(p, t_frame, |v| Vec3::from(v.0), Vec3::ZERO, evaluator.as_deref_mut(), self.model.fr),
                 data::PositionProperty::Split { x, y, z } => {
                     let px = Animator::resolve(x, t_frame, |v| *v, 0.0, evaluator.as_deref_mut(), self.model.fr);
                     let py = Animator::resolve(y, t_frame, |v| *v, 0.0, evaluator.as_deref_mut(), self.model.fr);
                     let pz = if let Some(z_prop) = z { Animator::resolve(z_prop, t_frame, |v| *v, 0.0, evaluator.as_deref_mut(), self.model.fr) } else { 0.0 };
                     Vec3::new(px, py, pz)
                 }
             };

             // Point of Interest (Anchor)
             let anchor = Animator::resolve(&ks.a, t_frame, |v| Vec3::from(v.0), Vec3::ZERO, evaluator.as_deref_mut(), self.model.fr);

             // Use LookAt logic
             // View = LookAt(pos, anchor, up)
             // Global Transform = View.inverse()
             // But we are resolving LOCAL transform here?
             // As established, we assume p and a are in parent/global space context.
             // If Camera has parent, this local transform is applied relative to parent.
             // LookAt constructs a matrix that transforms points from Local(Camera) to World (or Parent).
             // Actually `look_at_rh` creates View Matrix (World -> Camera).
             // We want Camera -> World (Transform).
             // So we return `look_at_rh(pos, anchor, UP).inverse()`.

             let up = Vec3::new(0.0, -1.0, 0.0); // Y down -> Up is -Y
             let view = Mat4::look_at_rh(pos, anchor, up);

             // Roll?
             // If we apply roll, it is around the local Z axis.
             // Camera looks down -Z.
             // Roll Z means rotate around Z.
             let rz = Animator::resolve(&ks.rz, t_frame, |v| v.to_radians(), 0.0, evaluator.as_deref_mut(), self.model.fr);
             // Inverse of (Roll * View) ? No.
             // Camera Transform = (RotZ * View).inverse() ?
             // Or Transform = View.inverse() * RotZ?
             // Let's assume Transform = LookAtInv * RotZ.
             return view.inverse() * Mat4::from_rotation_z(rz);
        }

        let mut anchor = Animator::resolve(&ks.a, t_frame, |v| Vec3::from(v.0), Vec3::ZERO, evaluator.as_deref_mut(), self.model.fr);

        let mut pos = match &ks.p {
            data::PositionProperty::Unified(p) => {
                Animator::resolve(p, t_frame, |v| Vec3::from(v.0), Vec3::ZERO, evaluator.as_deref_mut(), self.model.fr)
            }
            data::PositionProperty::Split { x, y, z } => {
                let px = Animator::resolve(x, t_frame, |v| *v, 0.0, evaluator.as_deref_mut(), self.model.fr);
                let py = Animator::resolve(y, t_frame, |v| *v, 0.0, evaluator.as_deref_mut(), self.model.fr);
                let pz = if let Some(z_prop) = z {
                    Animator::resolve(z_prop, t_frame, |v| *v, 0.0, evaluator.as_deref_mut(), self.model.fr)
                } else { 0.0 };
                Vec3::new(px, py, pz)
            }
        };

        let scale = Animator::resolve(&ks.s, t_frame, |v| Vec3::from(v.0) / 100.0, Vec3::ONE, evaluator.as_deref_mut(), self.model.fr);

        let rz = Animator::resolve(&ks.rz, t_frame, |v| v.to_radians(), 0.0, evaluator.as_deref_mut(), self.model.fr);
        let mut rx = 0.0;
        let mut ry = 0.0;
        if let Some(p) = &ks.rx { rx = Animator::resolve(p, t_frame, |v| v.to_radians(), 0.0, evaluator.as_deref_mut(), self.model.fr); }
        if let Some(p) = &ks.ry { ry = Animator::resolve(p, t_frame, |v| v.to_radians(), 0.0, evaluator.as_deref_mut(), self.model.fr); }

        let mut orientation = if let Some(or) = &ks.or {
            Animator::resolve(or, t_frame, |v| Vec3::from(v.0), Vec3::ZERO, evaluator.as_deref_mut(), self.model.fr)
        } else {
            Vec3::ZERO
        };

        // Enforce 2D limits if not 3D layer
        if !is_3d {
             pos.z = 0.0;
             rx = 0.0;
             ry = 0.0;
             orientation = Vec3::ZERO; // Usually ignored in 2D
             // scale.z? leave as is (usually 1.0)
             anchor.z = 0.0;
        }

        // Calculation: T * R * S * -A
        let mat_t = Mat4::from_translation(pos);

        // Rotation: Orientation * X * Y * Z
        // Orientation (degrees)
        let mat_or = Mat4::from_euler(glam::EulerRot::YXZ, orientation.y.to_radians(), orientation.x.to_radians(), orientation.z.to_radians());

        // Axis Rotations
        let mat_rx = Mat4::from_rotation_x(rx);
        let mat_ry = Mat4::from_rotation_y(ry);
        let mat_rz = Mat4::from_rotation_z(rz);

        let mat_r = mat_or * mat_rx * mat_ry * mat_rz;

        let mat_s = Mat4::from_scale(scale);
        let mat_a = Mat4::from_translation(-anchor);

        mat_t * mat_r * mat_s * mat_a
    }

    // Shapes logic must remain 2D (Mat3)
    fn get_shape_transform_2d(&self, ks: &data::Transform, frame: f32, #[cfg(feature = "expressions")] mut evaluator: Option<&mut ExpressionEvaluator>, #[cfg(not(feature = "expressions"))] mut evaluator: Option<&mut ()>) -> Mat3 {
         // Anchor (2D)
         let anchor_3d = Animator::resolve(&ks.a, frame, |v| Vec3::from(v.0), Vec3::ZERO, evaluator.as_deref_mut(), self.model.fr);
         let anchor = Vec2::new(anchor_3d.x, anchor_3d.y);

         // Position (2D)
         let pos = match &ks.p {
             data::PositionProperty::Unified(p) => {
                 let v3 = Animator::resolve(p, frame, |v| Vec3::from(v.0), Vec3::ZERO, evaluator.as_deref_mut(), self.model.fr);
                 Vec2::new(v3.x, v3.y)
             }
             data::PositionProperty::Split { x, y, .. } => {
                 let px = Animator::resolve(x, frame, |v| *v, 0.0, evaluator.as_deref_mut(), self.model.fr);
                 let py = Animator::resolve(y, frame, |v| *v, 0.0, evaluator.as_deref_mut(), self.model.fr);
                 Vec2::new(px, py)
             }
         };

         // Scale (2D)
         let s3 = Animator::resolve(&ks.s, frame, |v| Vec3::from(v.0) / 100.0, Vec3::ONE, evaluator.as_deref_mut(), self.model.fr);
         let scale = Vec2::new(s3.x, s3.y);

         // Rotation (Z)
         let r = Animator::resolve(&ks.rz, frame, |v| v.to_radians(), 0.0, evaluator.as_deref_mut(), self.model.fr);

         let mat_t = Mat3::from_translation(pos);
         let mat_r = Mat3::from_rotation_z(r);
         let mat_s = Mat3::from_scale(scale);
         let mat_a = Mat3::from_translation(-anchor);

         mat_t * mat_r * mat_s * mat_a
    }

    fn process_shapes(&self, shapes: &'a [data::Shape], frame: f32, #[cfg(feature = "expressions")] mut evaluator: Option<&mut ExpressionEvaluator>, #[cfg(not(feature = "expressions"))] mut evaluator: Option<&mut ()>) -> Vec<RenderNode> {
        let mut processed_nodes = Vec::new();
        let mut active_geometries: Vec<PendingGeometry> = Vec::new();

        let mut trim: Option<Trim> = None;
        for item in shapes {
            if let data::Shape::Trim(t) = item {
                let s = Animator::resolve(&t.s, frame, |v| *v / 100.0, 0.0, evaluator.as_deref_mut(), self.model.fr);
                let e = Animator::resolve(&t.e, frame, |v| *v / 100.0, 1.0, evaluator.as_deref_mut(), self.model.fr);
                let o = Animator::resolve(&t.o, frame, |v| *v / 360.0, 0.0, evaluator.as_deref_mut(), self.model.fr);
                trim = Some(Trim { start: s, end: e, offset: o });
            }
        }

        for item in shapes {
            match item {
                data::Shape::MergePaths(mp) => {
                    if !active_geometries.is_empty() {
                        let mode = match mp.mm {
                            1 => MergeMode::Merge, 2 => MergeMode::Add, 3 => MergeMode::Subtract,
                            4 => MergeMode::Intersect, 5 => MergeMode::Exclude, _ => MergeMode::Merge,
                        };
                        let merged = PendingGeometry {
                            kind: GeometryKind::Merge(active_geometries.clone(), mode),
                            transform: Mat3::IDENTITY,
                        };
                        active_geometries.clear();
                        active_geometries.push(merged);
                    }
                }
                data::Shape::Path(p) => {
                    let path = self.convert_path(p, frame, evaluator.as_deref_mut());
                    active_geometries.push(PendingGeometry {
                        kind: GeometryKind::Path(path),
                        transform: Mat3::IDENTITY,
                    });
                }
                data::Shape::Rect(r) => {
                    let size = Animator::resolve(&r.s, frame, |v| Vec2::from_slice(v), Vec2::ZERO, evaluator.as_deref_mut(), self.model.fr);
                    let pos = Animator::resolve(&r.p, frame, |v| Vec2::from_slice(v), Vec2::ZERO, evaluator.as_deref_mut(), self.model.fr);
                    let radius = Animator::resolve(&r.r, frame, |v| *v, 0.0, evaluator.as_deref_mut(), self.model.fr);
                    active_geometries.push(PendingGeometry {
                        kind: GeometryKind::Rect { size, pos, radius },
                        transform: Mat3::IDENTITY,
                    });
                }
                data::Shape::Ellipse(e) => {
                    let size = Animator::resolve(&e.s, frame, |v| Vec2::from_slice(v), Vec2::ZERO, evaluator.as_deref_mut(), self.model.fr);
                    let pos = Animator::resolve(&e.p, frame, |v| Vec2::from_slice(v), Vec2::ZERO, evaluator.as_deref_mut(), self.model.fr);
                    active_geometries.push(PendingGeometry {
                        kind: GeometryKind::Ellipse { size, pos },
                        transform: Mat3::IDENTITY,
                    });
                }
                data::Shape::Polystar(sr) => {
                    let pos = match &sr.p {
                        data::PositionProperty::Unified(p) => Animator::resolve(p, 0.0, |v| Vec2::from_slice(&v.0[0..2]), Vec2::ZERO, evaluator.as_deref_mut(), self.model.fr),
                        data::PositionProperty::Split { x, y, .. } => {
                            let px = Animator::resolve(x, 0.0, |v| *v, 0.0, evaluator.as_deref_mut(), self.model.fr);
                            let py = Animator::resolve(y, 0.0, |v| *v, 0.0, evaluator.as_deref_mut(), self.model.fr);
                            Vec2::new(px, py)
                        }
                    };
                    let or = Animator::resolve(&sr.or, frame, |v| *v, 0.0, evaluator.as_deref_mut(), self.model.fr);
                    let os = Animator::resolve(&sr.os, frame, |v| *v, 0.0, evaluator.as_deref_mut(), self.model.fr);
                    let r = Animator::resolve(&sr.r, frame, |v| *v, 0.0, evaluator.as_deref_mut(), self.model.fr);
                    let pt = Animator::resolve(&sr.pt, frame, |v| *v, 5.0, evaluator.as_deref_mut(), self.model.fr);
                    let ir = if let Some(prop) = &sr.ir { Animator::resolve(prop, 0.0, |v| *v, 0.0, evaluator.as_deref_mut(), self.model.fr) } else { 0.0 };
                    let is = if let Some(prop) = &sr.is { Animator::resolve(prop, 0.0, |v| *v, 0.0, evaluator.as_deref_mut(), self.model.fr) } else { 0.0 };

                    active_geometries.push(PendingGeometry {
                        kind: GeometryKind::Polystar(PolystarParams {
                            pos, outer_radius: or, inner_radius: ir, outer_roundness: os,
                            inner_roundness: is, rotation: r, points: pt, kind: sr.sy, corner_radius: 0.0,
                        }),
                        transform: Mat3::IDENTITY,
                    });
                }
                data::Shape::Transform(tr) => {
                    // Update current active geometries transform
                    let local = self.get_shape_transform_2d(&tr.t, frame, evaluator.as_deref_mut());
                    for geom in &mut active_geometries {
                        geom.transform = local * geom.transform;
                    }
                }
                data::Shape::RoundCorners(rd) => {
                    let r = Animator::resolve(&rd.r, frame, |v| *v, 0.0, evaluator.as_deref_mut(), self.model.fr);
                    if r > 0.0 {
                        for geom in &mut active_geometries {
                            match &mut geom.kind {
                                GeometryKind::Rect { radius, .. } => *radius += r,
                                GeometryKind::Polystar(p) => p.corner_radius += r,
                                _ => {}
                            }
                        }
                    }
                }
                data::Shape::ZigZag(zz) => {
                    let ridges = Animator::resolve(&zz.r, frame, |v| *v, 0.0, evaluator.as_deref_mut(), self.model.fr);
                    let size = Animator::resolve(&zz.s, frame, |v| *v, 0.0, evaluator.as_deref_mut(), self.model.fr);
                    let pt = Animator::resolve(&zz.pt, frame, |v| *v, 1.0, evaluator.as_deref_mut(), self.model.fr);
                    let modifier = ZigZagModifier { ridges, size, smooth: pt > 1.5 };
                    self.apply_modifier_to_active(&mut active_geometries, &modifier);
                }
                data::Shape::PuckerBloat(pb) => {
                    let amount = Animator::resolve(&pb.a, frame, |v| *v, 0.0, evaluator.as_deref_mut(), self.model.fr);
                    let modifier = PuckerBloatModifier { amount, center: Vec2::ZERO };
                    self.apply_modifier_to_active(&mut active_geometries, &modifier);
                }
                data::Shape::Twist(tw) => {
                    let angle = Animator::resolve(&tw.a, frame, |v| *v, 0.0, evaluator.as_deref_mut(), self.model.fr);
                    let center = Animator::resolve(&tw.c, frame, |v| Vec2::from_slice(v), Vec2::ZERO, evaluator.as_deref_mut(), self.model.fr);
                    let modifier = TwistModifier { angle, center };
                    self.apply_modifier_to_active(&mut active_geometries, &modifier);
                }
                data::Shape::OffsetPath(op) => {
                    let amount = Animator::resolve(&op.a, frame, |v| *v, 0.0, evaluator.as_deref_mut(), self.model.fr);
                    let miter_limit = op.ml.unwrap_or(4.0);
                    let line_join = op.lj;
                    let modifier = OffsetPathModifier { amount, miter_limit, line_join };
                    self.apply_modifier_to_active(&mut active_geometries, &modifier);
                }
                data::Shape::WigglePath(wg) => {
                    let speed = Animator::resolve(&wg.s, frame, |v| *v, 0.0, evaluator.as_deref_mut(), self.model.fr);
                    let size = Animator::resolve(&wg.w, frame, |v| *v, 0.0, evaluator.as_deref_mut(), self.model.fr);
                    let correlation = Animator::resolve(&wg.r, frame, |v| *v, 0.0, evaluator.as_deref_mut(), self.model.fr);
                    let seed_prop = Animator::resolve(&wg.sh, frame, |v| *v, 0.0, evaluator.as_deref_mut(), self.model.fr);
                    let mut modifier = WiggleModifier { seed: seed_prop, time: frame / 60.0, speed: speed / self.model.fr, amount: size, correlation };
                    modifier.time = frame;
                    modifier.speed = speed / self.model.fr;
                    self.apply_modifier_to_active(&mut active_geometries, &modifier);
                }
                data::Shape::Repeater(rp) => {
                    let copies = Animator::resolve(&rp.c, frame, |v| *v, 0.0, evaluator.as_deref_mut(), self.model.fr);
                    let offset = Animator::resolve(&rp.o, frame, |v| *v, 0.0, evaluator.as_deref_mut(), self.model.fr);
                    let t_anchor_3d = Animator::resolve(&rp.tr.t.a, frame, |v| Vec3::from(v.0), Vec3::ZERO, evaluator.as_deref_mut(), self.model.fr);
                    let t_anchor = Vec2::new(t_anchor_3d.x, t_anchor_3d.y);

                    let t_pos = match &rp.tr.t.p {
                        data::PositionProperty::Unified(p) => Animator::resolve(p, 0.0, |v| Vec2::from_slice(&v.0[0..2]), Vec2::ZERO, evaluator.as_deref_mut(), self.model.fr),
                        data::PositionProperty::Split { x, y, .. } => {
                            let px = Animator::resolve(x, 0.0, |v| *v, 0.0, evaluator.as_deref_mut(), self.model.fr);
                            let py = Animator::resolve(y, 0.0, |v| *v, 0.0, evaluator.as_deref_mut(), self.model.fr);
                            Vec2::new(px, py)
                        }
                    };
                    let t_scale_3d = Animator::resolve(&rp.tr.t.s, 0.0, |v| Vec3::from(v.0) / 100.0, Vec3::ONE, evaluator.as_deref_mut(), self.model.fr);
                    let t_scale = Vec2::new(t_scale_3d.x, t_scale_3d.y);

                    let t_rot = Animator::resolve(&rp.tr.t.rz, frame, |v| v.to_radians(), 0.0, evaluator.as_deref_mut(), self.model.fr);

                    let start_opacity = Animator::resolve(&rp.tr.so, frame, |v| *v / 100.0, 1.0, evaluator.as_deref_mut(), self.model.fr);
                    let end_opacity = Animator::resolve(&rp.tr.eo, frame, |v| *v / 100.0, 1.0, evaluator.as_deref_mut(), self.model.fr);

                    self.apply_repeater(copies, offset, t_anchor, t_pos, t_scale, t_rot, start_opacity, end_opacity, &mut active_geometries, &mut processed_nodes);
                }
                data::Shape::Fill(f) => {
                    let color = Animator::resolve(&f.c, frame, |v| Vec4::from_slice(v), Vec4::ONE, evaluator.as_deref_mut(), self.model.fr);
                    let opacity = Animator::resolve(&f.o, frame, |v| *v / 100.0, 1.0, evaluator.as_deref_mut(), self.model.fr);
                    for geom in &active_geometries {
                        let path = self.convert_geometry(geom);
                        processed_nodes.push(RenderNode {
                            transform: Mat4::IDENTITY,
                            alpha: 1.0, blend_mode: BlendMode::Normal,
                            content: NodeContent::Shape(renderer::Shape {
                                geometry: path,
                                fill: Some(Fill { paint: Paint::Solid(color), opacity, rule: FillRule::NonZero }),
                                stroke: None, trim: trim.clone(),
                            }),
                            masks: vec![], styles: vec![], matte: None, effects: vec![], is_adjustment_layer: false,
                        });
                    }
                }
                data::Shape::GradientFill(gf) => {
                    let start = Animator::resolve(&gf.s, frame, |v| Vec2::from_slice(v), Vec2::ZERO, evaluator.as_deref_mut(), self.model.fr);
                    let end = Animator::resolve(&gf.e, frame, |v| Vec2::from_slice(v), Vec2::ZERO, evaluator.as_deref_mut(), self.model.fr);
                    let opacity = Animator::resolve(&gf.o, frame, |v| *v / 100.0, 1.0, evaluator.as_deref_mut(), self.model.fr);
                    let raw_stops = Animator::resolve(&gf.g.k, frame, |v| v.clone(), Vec::new(), evaluator.as_deref_mut(), self.model.fr);
                    let stops = parse_gradient_stops(&raw_stops, gf.g.p as usize);
                    let kind = if gf.t == 1 { GradientKind::Linear } else { GradientKind::Radial };
                    for geom in &active_geometries {
                        let path = self.convert_geometry(geom);
                        processed_nodes.push(RenderNode {
                            transform: Mat4::IDENTITY,
                            alpha: 1.0, blend_mode: BlendMode::Normal,
                            content: NodeContent::Shape(renderer::Shape {
                                geometry: path,
                                fill: Some(Fill { paint: Paint::Gradient(Gradient { kind, stops: stops.clone(), start, end }), opacity, rule: FillRule::NonZero }),
                                stroke: None, trim: trim.clone(),
                            }),
                            masks: vec![], styles: vec![], matte: None, effects: vec![], is_adjustment_layer: false,
                        });
                    }
                }
                data::Shape::GradientStroke(gs) => {
                    let start = Animator::resolve(&gs.s, frame, |v| Vec2::from_slice(v), Vec2::ZERO, evaluator.as_deref_mut(), self.model.fr);
                    let end = Animator::resolve(&gs.e, frame, |v| Vec2::from_slice(v), Vec2::ZERO, evaluator.as_deref_mut(), self.model.fr);
                    let width = Animator::resolve(&gs.w, frame, |v| *v, 1.0, evaluator.as_deref_mut(), self.model.fr);
                    let opacity = Animator::resolve(&gs.o, frame, |v| *v / 100.0, 1.0, evaluator.as_deref_mut(), self.model.fr);
                    let raw_stops = Animator::resolve(&gs.g.k, frame, |v| v.clone(), Vec::new(), evaluator.as_deref_mut(), self.model.fr);
                    let stops = parse_gradient_stops(&raw_stops, gs.g.p as usize);
                    let kind = if gs.t == 1 { GradientKind::Linear } else { GradientKind::Radial };
                    let dash = self.resolve_dash(&gs.d, frame, evaluator.as_deref_mut());
                    for geom in &active_geometries {
                        let path = self.convert_geometry(geom);
                        processed_nodes.push(RenderNode {
                            transform: Mat4::IDENTITY,
                            alpha: 1.0, blend_mode: BlendMode::Normal,
                            content: NodeContent::Shape(renderer::Shape {
                                geometry: path,
                                fill: None,
                                stroke: Some(Stroke { paint: Paint::Gradient(Gradient { kind, stops: stops.clone(), start, end }), width, opacity, cap: match gs.lc { 1 => LineCap::Butt, 3 => LineCap::Square, _ => LineCap::Round }, join: match gs.lj { 1 => LineJoin::Miter, 3 => LineJoin::Bevel, _ => LineJoin::Round }, miter_limit: gs.ml, dash: dash.clone() }),
                                trim: trim.clone(),
                            }),
                            masks: vec![], styles: vec![], matte: None, effects: vec![], is_adjustment_layer: false,
                        });
                    }
                }
                data::Shape::Stroke(s) => {
                    let color = Animator::resolve(&s.c, frame, |v| Vec4::from_slice(v), Vec4::ONE, evaluator.as_deref_mut(), self.model.fr);
                    let width = Animator::resolve(&s.w, frame, |v| *v, 1.0, evaluator.as_deref_mut(), self.model.fr);
                    let opacity = Animator::resolve(&s.o, frame, |v| *v / 100.0, 1.0, evaluator.as_deref_mut(), self.model.fr);
                    let dash = self.resolve_dash(&s.d, frame, evaluator.as_deref_mut());
                    for geom in &active_geometries {
                        let path = self.convert_geometry(geom);
                        processed_nodes.push(RenderNode {
                            transform: Mat4::IDENTITY,
                            alpha: 1.0, blend_mode: BlendMode::Normal,
                            content: NodeContent::Shape(renderer::Shape {
                                geometry: path,
                                fill: None,
                                stroke: Some(Stroke { paint: Paint::Solid(color), width, opacity, cap: match s.lc { 1 => LineCap::Butt, 3 => LineCap::Square, _ => LineCap::Round }, join: match s.lj { 1 => LineJoin::Miter, 3 => LineJoin::Bevel, _ => LineJoin::Round }, miter_limit: s.ml, dash: dash.clone() }),
                                trim: trim.clone(),
                            }),
                            masks: vec![], styles: vec![], matte: None, effects: vec![], is_adjustment_layer: false,
                        });
                    }
                }
                data::Shape::Group(g) => {
                    let group_nodes = self.process_shapes(&g.it, frame, evaluator.as_deref_mut());
                    processed_nodes.push(RenderNode {
                        transform: Mat4::IDENTITY,
                        alpha: 1.0, blend_mode: BlendMode::Normal,
                        content: NodeContent::Group(group_nodes),
                        masks: vec![], styles: vec![], matte: None, effects: vec![], is_adjustment_layer: false,
                    });
                }
                _ => {}
            }
        }
        processed_nodes
    }

    fn resolve_dash(&self, props: &[data::DashProperty], frame: f32, #[cfg(feature = "expressions")] mut evaluator: Option<&mut ExpressionEvaluator>, #[cfg(not(feature = "expressions"))] mut evaluator: Option<&mut ()>) -> Option<DashPattern> {
        if props.is_empty() { return None; }
        let mut array = Vec::new();
        let mut offset = 0.0;
        for prop in props {
            match prop.n.as_deref() {
                Some("o") => offset = Animator::resolve(&prop.v, frame, |v| *v, 0.0, evaluator.as_deref_mut(), self.model.fr),
                Some("d") | Some("v") | Some("g") => array.push(Animator::resolve(&prop.v, frame, |v| *v, 0.0, evaluator.as_deref_mut(), self.model.fr)),
                _ => {}
            }
        }
        if !array.is_empty() {
            if array.len() % 2 != 0 { let clone = array.clone(); array.extend(clone); }
            let total: f32 = array.iter().sum();
            if total > 0.0 { offset = (offset % total + total) % total; } else { offset = 0.0; }
            Some(DashPattern { array, offset })
        } else {
            None
        }
    }

    fn apply_repeater(
        &self,
        copies: f32,
        _offset: f32,
        anchor: Vec2,
        pos: Vec2,
        scale: Vec2,
        rot: f32,
        start_op: f32,
        end_op: f32,
        geoms: &mut Vec<PendingGeometry>,
        nodes: &mut Vec<RenderNode>,
    ) {
        let num_copies = copies.round() as usize;
        if num_copies <= 1 { return; }

        let original_geoms = geoms.clone();
        let original_nodes = nodes.clone();

        let mat_t = Mat3::from_translation(pos);
        let mat_r = Mat3::from_rotation_z(rot); // Radians
        let mat_s = Mat3::from_scale(scale);
        let mat_a = Mat3::from_translation(-anchor);
        let mat_pre_a = Mat3::from_translation(anchor);

        let pivot_transform = mat_pre_a * mat_r * mat_s * mat_a;
        let step_transform = mat_t * pivot_transform;

        // RenderNode uses Mat4, but repeater internal logic here is mixed?
        // nodes have Mat4. step_transform is Mat3.
        // We need Mat4 step transform for nodes.
        let mat_t4 = Mat4::from_translation(Vec3::new(pos.x, pos.y, 0.0));
        let mat_r4 = Mat4::from_rotation_z(rot);
        let mat_s4 = Mat4::from_scale(Vec3::new(scale.x, scale.y, 1.0));
        let mat_a4 = Mat4::from_translation(Vec3::new(-anchor.x, -anchor.y, 0.0));
        let mat_pre_a4 = Mat4::from_translation(Vec3::new(anchor.x, anchor.y, 0.0));
        let step_transform4 = mat_t4 * mat_pre_a4 * mat_r4 * mat_s4 * mat_a4;

        geoms.clear();
        nodes.clear();

        for i in 0..num_copies {
            let t = if num_copies > 1 { i as f32 / (num_copies as f32 - 1.0) } else { 0.0 };
            let op = start_op + (end_op - start_op) * t;

            let mut copy_transform = Mat3::IDENTITY;
            let mut copy_transform4 = Mat4::IDENTITY;
            for _ in 0..i {
                copy_transform = copy_transform * step_transform;
                copy_transform4 = copy_transform4 * step_transform4;
            }

            for geom in &original_geoms {
                let mut g = geom.clone();
                g.transform = copy_transform * g.transform;
                geoms.push(g);
            }

            for node in &original_nodes {
                let mut n = node.clone();
                n.transform = copy_transform4 * n.transform;
                n.alpha *= op;
                nodes.push(n);
            }
        }
    }

    fn apply_modifier_to_active(
        &self,
        active: &mut Vec<PendingGeometry>,
        modifier: &impl GeometryModifier,
    ) {
        for geom in active.iter_mut() {
            let mut path = geom.to_path(self);
            modifier.modify(&mut path);
            geom.transform = Mat3::IDENTITY;
            geom.kind = GeometryKind::Path(path);
        }
    }

    fn convert_geometry(&self, geom: &PendingGeometry) -> ShapeGeometry {
        geom.to_shape_geometry(self)
    }

    fn generate_polystar_path(&self, params: &PolystarParams) -> BezPath {
        let mut path = BezPath::new();
        let num_points = params.points.round();
        if num_points < 3.0 { return path; }

        let is_star = params.kind == 1;
        let has_roundness = params.outer_roundness.abs() > 0.01 || (is_star && params.inner_roundness.abs() > 0.01);
        let total_points = if is_star { num_points * 2.0 } else { num_points } as usize;
        let current_angle = (params.rotation - 90.0).to_radians();
        let angle_step = 2.0 * PI / total_points as f64;

        if has_roundness {
            let mut elements = Vec::with_capacity(total_points);
            for i in 0..total_points {
                let (r, roundness) = if is_star {
                    if i % 2 == 0 { (params.outer_radius, params.outer_roundness) } else { (params.inner_radius, params.inner_roundness) }
                } else {
                    (params.outer_radius, params.outer_roundness)
                };

                let angle = current_angle as f64 + angle_step * i as f64;
                let cos_a = angle.cos();
                let sin_a = angle.sin();
                let x = params.pos.x as f64 + r as f64 * cos_a;
                let y = params.pos.y as f64 + r as f64 * sin_a;
                let vertex = Point::new(x, y);

                let tx = -sin_a; let ty = cos_a;
                let tangent = kurbo::Vec2::new(tx, ty);
                let cp_d = r as f64 * angle_step * roundness as f64 * 0.01;
                let in_cp = vertex - tangent * cp_d;
                let out_cp = vertex + tangent * cp_d;
                elements.push((vertex, in_cp, out_cp));
            }
            if elements.is_empty() { return path; }
            path.move_to(elements[0].0);
            let len = elements.len();
            for i in 0..len {
                let curr_idx = i;
                let next_idx = (i + 1) % len;
                let curr_out_cp = elements[curr_idx].2;
                let next_in_cp = elements[next_idx].1;
                let next_vertex = elements[next_idx].0;
                path.curve_to(curr_out_cp, next_in_cp, next_vertex);
            }
            path.close_path();
            return path;
        }

        let mut vertices = Vec::with_capacity(total_points);
        for i in 0..total_points {
            let r = if is_star { if i % 2 == 0 { params.outer_radius } else { params.inner_radius } } else { params.outer_radius };
            let angle = current_angle as f64 + angle_step * i as f64;
            let x = params.pos.x as f64 + r as f64 * angle.cos();
            let y = params.pos.y as f64 + r as f64 * angle.sin();
            vertices.push(Point::new(x, y));
        }

        let radius = params.corner_radius as f64;
        if radius <= 0.1 {
            if !vertices.is_empty() {
                path.move_to(vertices[0]);
                for v in vertices.iter().skip(1) { path.line_to(*v); }
                path.close_path();
            }
            return path;
        }

        let len = vertices.len();
        for i in 0..len {
            let prev = vertices[(i + len - 1) % len];
            let curr = vertices[i];
            let next = vertices[(i + 1) % len];
            let v1 = prev - curr;
            let v2 = next - curr;
            let len1 = v1.hypot();
            let len2 = v2.hypot();

            if len1 < 0.001 || len2 < 0.001 {
                if i == 0 { path.move_to(curr); } else { path.line_to(curr); }
                continue;
            }

            let u1 = v1 * (1.0 / len1);
            let u2 = v2 * (1.0 / len2);
            let dot = (u1.x * u2.x + u1.y * u2.y).clamp(-1.0, 1.0);
            let angle = dot.acos();
            let dist = if angle.abs() < 0.001 { 0.0 } else { radius / (angle / 2.0).tan() };
            let max_d = (len1.min(len2)) * 0.5;
            let d = dist.min(max_d);
            let p_start = curr + u1 * d;
            let p_end = curr + u2 * d;

            if i == 0 { path.move_to(p_start); } else { path.line_to(p_start); }
            path.quad_to(curr, p_end);
        }
        path.close_path();
        path
    }

    fn convert_path(&self, p: &data::PathShape, frame: f32, #[cfg(feature = "expressions")] evaluator: Option<&mut ExpressionEvaluator>, #[cfg(not(feature = "expressions"))] mut evaluator: Option<&mut ()>) -> BezPath {
        let path_data = Animator::resolve(&p.ks, frame, |v| v.clone(), data::BezierPath::default(), evaluator, self.model.fr);
        self.convert_bezier_path(&path_data)
    }

    fn convert_bezier_path(&self, path_data: &data::BezierPath) -> BezPath {
        let mut bp = BezPath::new();
        if path_data.v.is_empty() { return bp; }
        let start = path_data.v[0];
        bp.move_to(Point::new(start[0] as f64, start[1] as f64));
        for i in 0..path_data.v.len() {
            let next_idx = (i + 1) % path_data.v.len();
            if next_idx == 0 && !path_data.c { break; }
            let p0 = path_data.v[i];
            let p1 = path_data.v[next_idx];
            let o = if i < path_data.o.len() { path_data.o[i] } else { [0.0, 0.0] };
            let in_ = if next_idx < path_data.i.len() { path_data.i[next_idx] } else { [0.0, 0.0] };
            let cp1 = [p0[0] + o[0], p0[1] + o[1]];
            let cp2 = [p1[0] + in_[0], p1[1] + in_[1]];
            bp.curve_to(Point::new(cp1[0] as f64, cp1[1] as f64), Point::new(cp2[0] as f64, cp2[1] as f64), Point::new(p1[0] as f64, p1[1] as f64));
        }
        if path_data.c { bp.close_path(); }
        bp
    }

    fn process_masks(&self, masks_props: &[data::MaskProperties], frame: f32, #[cfg(feature = "expressions")] mut evaluator: Option<&mut ExpressionEvaluator>, #[cfg(not(feature = "expressions"))] mut evaluator: Option<&mut ()>) -> Vec<Mask> {
        let mut masks = Vec::new();
        for m in masks_props {
            let mode = match m.mode.as_deref() {
                Some("n") => MaskMode::None, Some("a") => MaskMode::Add, Some("s") => MaskMode::Subtract,
                Some("i") => MaskMode::Intersect, Some("l") => MaskMode::Lighten, Some("d") => MaskMode::Darken,
                Some("f") => MaskMode::Difference, _ => continue,
            };
            let path_data = Animator::resolve(&m.pt, frame, |v| v.clone(), data::BezierPath::default(), evaluator.as_deref_mut(), self.model.fr);
            let geometry = self.convert_bezier_path(&path_data);
            let opacity = Animator::resolve(&m.o, frame, |v| *v / 100.0, 1.0, evaluator.as_deref_mut(), self.model.fr);
            let expansion = Animator::resolve(&m.x, frame, |v| *v, 0.0, evaluator.as_deref_mut(), self.model.fr);
            let inverted = m.inv;
            masks.push(Mask { mode, geometry, opacity, expansion, inverted });
        }
        masks
    }
}

// Helpers
struct ColorStop { t: f32, r: f32, g: f32, b: f32 }
struct AlphaStop { t: f32, a: f32 }

fn parse_gradient_stops(raw: &[f32], color_count: usize) -> Vec<GradientStop> {
    let mut stops = Vec::new();
    if raw.is_empty() { return stops; }
    let mut color_stops = Vec::new();
    let mut alpha_stops = Vec::new();
    let color_data_len = color_count * 4;
    for chunk in raw.iter().take(color_data_len).collect::<Vec<_>>().chunks(4) {
        if chunk.len() == 4 { color_stops.push(ColorStop { t: *chunk[0], r: *chunk[1], g: *chunk[2], b: *chunk[3] }); }
    }
    if raw.len() > color_data_len {
        for chunk in raw[color_data_len..].chunks(2) {
            if chunk.len() == 2 { alpha_stops.push(AlphaStop { t: chunk[0], a: chunk[1] }); }
        }
    }
    if alpha_stops.is_empty() {
        for c in color_stops { stops.push(GradientStop { offset: c.t, color: Vec4::new(c.r, c.g, c.b, 1.0) }); }
        return stops;
    }
    let mut unique_t: Vec<f32> = Vec::new();
    for c in &color_stops { unique_t.push(c.t); }
    for a in &alpha_stops { unique_t.push(a.t); }
    unique_t.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    unique_t.dedup();
    for t in unique_t {
        let (r, g, b) = interpolate_color(&color_stops, t);
        let a = interpolate_alpha(&alpha_stops, t);
        stops.push(GradientStop { offset: t, color: Vec4::new(r, g, b, a) });
    }
    stops
}

fn interpolate_color(stops: &[ColorStop], t: f32) -> (f32, f32, f32) {
    if stops.is_empty() { return (1.0, 1.0, 1.0); }
    if t <= stops[0].t { return (stops[0].r, stops[0].g, stops[0].b); }
    if t >= stops.last().unwrap().t { let last = stops.last().unwrap(); return (last.r, last.g, last.b); }
    for i in 0..stops.len() - 1 {
        let s1 = &stops[i];
        let s2 = &stops[i + 1];
        if t >= s1.t && t <= s2.t {
            let range = s2.t - s1.t;
            let ratio = if range == 0.0 { 0.0 } else { (t - s1.t) / range };
            return (s1.r + (s2.r - s1.r) * ratio, s1.g + (s2.g - s1.g) * ratio, s1.b + (s2.b - s1.b) * ratio);
        }
    }
    (1.0, 1.0, 1.0)
}

fn interpolate_alpha(stops: &[AlphaStop], t: f32) -> f32 {
    if stops.is_empty() { return 1.0; }
    if t <= stops[0].t { return stops[0].a; }
    if t >= stops.last().unwrap().t { return stops.last().unwrap().a; }
    for i in 0..stops.len() - 1 {
        let s1 = &stops[i];
        let s2 = &stops[i + 1];
        if t >= s1.t && t <= s2.t {
            let range = s2.t - s1.t;
            let ratio = if range == 0.0 { 0.0 } else { (t - s1.t) / range };
            return s1.a + (s2.a - s1.a) * ratio;
        }
    }
    1.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use lottie_data::model as data;

    #[test]
    fn test_camera_transform() {
        let camera_layer = data::Layer {
            ty: 13,
            ind: Some(1),
            parent: None,
            nm: Some("Camera".to_string()),
            ip: 0.0,
            op: 60.0,
            st: 0.0,
            ks: data::Transform {
                p: data::PositionProperty::Unified(data::Property {
                    k: data::Value::Static(data::Vec3DefaultZero([0.0, 0.0, -500.0])),
                    ..Default::default()
                }),
                a: data::Property {
                    k: data::Value::Static(data::Vec3DefaultZero([0.0, 0.0, 0.0])),
                    ..Default::default()
                },
                ..Default::default()
            },
            pe: Some(data::Property {
                k: data::Value::Static(1000.0),
                ..Default::default()
            }),
            ddd: Some(1),
            ao: None, tm: None, masks_properties: None, tt: None, ef: None, sy: None,
            ref_id: None, w: None, h: None, color: None, sw: None, sh: None,
            shapes: None, t: None,
        };

        let model = LottieJson {
            v: None, ip: 0.0, op: 60.0, fr: 60.0, w: 1000, h: 1000,
            layers: vec![camera_layer],
            assets: vec![],
        };

        let mut player = LottiePlayer::new();
        player.load(model);
        let tree = player.render_tree();

        let vm = tree.view_matrix;
        // World point (0,0,0) -> View Space
        let p_world = Vec4::new(0.0, 0.0, 0.0, 1.0);
        let p_view = vm * p_world;

        // Camera at -500. Looking at 0.
        // In View Space (RH, -Z forward), the object at 0 (distance 500 in front)
        // should be at Z = -500.
        assert!((p_view.z - (-500.0)).abs() < 1.0, "Expected Z=-500, got {}", p_view.z);
    }
}
