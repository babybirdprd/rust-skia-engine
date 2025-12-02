use crate::element::{Element, Color};
use crate::animation::{Animated, EasingType};
use skia_safe::{Path, Paint, Canvas, Rect as SkRect, PaintCap, PaintJoin, PaintStyle, PathMeasure, Size, Color4f, PathBuilder};
use taffy::style::Style;
use std::any::Any;
use std::fmt;

#[derive(Debug, Clone)]
pub enum ShapeType {
    Rect { width: f32, height: f32, corner_radius: f32 },
    Circle { radius: f32 },
    Ellipse { radius_x: f32, radius_y: f32 },
    Path { d: String }, // SVG path data
}

#[derive(Clone)]
pub struct ShapeNode {
    pub shape_type: ShapeType,

    // Cached Skia Path (Computed once on init)
    pub path: Path,
    // Cached Bounds (Computed once on init for Layout)
    pub intrinsic_size: Size,

    // Visuals
    pub fill_color: Option<Animated<Color>>,
    pub stroke_color: Option<Animated<Color>>,
    pub stroke_width: Animated<f32>,
    pub stroke_cap: PaintCap,
    pub stroke_join: PaintJoin,

    // Trim Path (0.0 to 1.0)
    pub stroke_start: Animated<f32>,
    pub stroke_end: Animated<f32>,
    pub stroke_offset: Animated<f32>, // For rotating the start point

    pub style: Style,
}

impl fmt::Debug for ShapeNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ShapeNode")
         .field("shape_type", &self.shape_type)
         .finish()
    }
}

impl ShapeNode {
    pub fn new(shape_type: ShapeType) -> Self {
        let mut path = Path::new();
        let size;

        match &shape_type {
            ShapeType::Rect { width, height, corner_radius } => {
                let rect = SkRect::from_wh(*width, *height);
                path.add_rrect(skia_safe::RRect::new_rect_xy(&rect, *corner_radius, *corner_radius), None);
                size = Size::new(*width, *height);
            },
            ShapeType::Circle { radius } => {
                // Circle centered at (radius, radius) so it fits in [0, 0, 2r, 2r]
                path.add_circle((*radius, *radius), *radius, None);
                size = Size::new(*radius * 2.0, *radius * 2.0);
            },
            ShapeType::Ellipse { radius_x, radius_y } => {
                let rect = SkRect::from_wh(radius_x * 2.0, radius_y * 2.0);
                path.add_oval(&rect, None);
                size = Size::new(radius_x * 2.0, radius_y * 2.0);
            },
            ShapeType::Path { d } => {
                if let Some(p) = Path::from_svg(d) {
                    path = p;
                    let bounds = path.compute_tight_bounds();
                    // Intrinsic size matches the bounds width/height.
                    // Note: Drawing respects exact coordinates, so if path is offset,
                    // it might draw outside the layout box if layout box is positioned at (0,0).
                    size = Size::new(bounds.width(), bounds.height());
                } else {
                    eprintln!("Failed to parse SVG path: {}", d);
                    size = Size::new(0.0, 0.0);
                }
            }
        }

        Self {
            shape_type,
            path,
            intrinsic_size: size,
            fill_color: None,
            stroke_color: None,
            stroke_width: Animated::new(0.0),
            stroke_cap: PaintCap::Butt,
            stroke_join: PaintJoin::Miter,
            stroke_start: Animated::new(0.0),
            stroke_end: Animated::new(1.0),
            stroke_offset: Animated::new(0.0),
            style: Style::DEFAULT,
        }
    }

    fn get_trim_paths(&self, start: f32, end: f32, offset: f32) -> Vec<Path> {
         let mut paths = Vec::new();

         // 1. Validate inputs
         let duration = end - start;
         // If full path or invalid range
         if duration.abs() >= 0.999 || (start <= 0.001 && end >= 0.999) {
             paths.push(self.path.clone());
             return paths;
         }

         // 2. Measure
         // force_closed = false (usually)
         let mut measure = PathMeasure::new(&self.path, false, None);
         let len = measure.length();

         if len <= 0.0 { return paths; }

         // 3. Apply Offset & Wrap logic
         let shift = offset;
         let s = (start + shift).rem_euclid(1.0);
         let e = (end + shift).rem_euclid(1.0);

         // FIX: If s == e (and not full path), it means empty.
         if (s - e).abs() < 0.0001 {
             return paths;
         }

         let dist_s = s * len;
         let dist_e = e * len;

         if s < e {
             // Normal segment
             let mut dst = PathBuilder::new();
             if measure.get_segment(dist_s, dist_e, &mut dst, true) {
                 paths.push(dst.detach(None));
             }
         } else {
             // Wrapped segment: [s -> 1.0] and [0.0 -> e]
             let mut dst1 = PathBuilder::new();
             if measure.get_segment(dist_s, len, &mut dst1, true) {
                 paths.push(dst1.detach(None));
             }
             let mut dst2 = PathBuilder::new();
             if measure.get_segment(0.0, dist_e, &mut dst2, true) {
                 paths.push(dst2.detach(None));
             }
         }

         paths
    }
}

fn parse_easing(e: &str) -> EasingType {
    match e {
        "linear" => EasingType::Linear,
        "ease_in" => EasingType::EaseIn,
        "ease_out" => EasingType::EaseOut,
        "ease_in_out" => EasingType::EaseInOut,
        "bounce_out" => EasingType::BounceOut,
        _ => EasingType::Linear,
    }
}

impl Element for ShapeNode {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }

    fn layout_style(&self) -> Style {
        self.style.clone()
    }

    fn set_layout_style(&mut self, style: Style) {
        self.style = style;
    }

    fn update(&mut self, time: f64) -> bool {
        if let Some(c) = &mut self.fill_color { c.update(time); }
        if let Some(c) = &mut self.stroke_color { c.update(time); }
        self.stroke_width.update(time);
        self.stroke_start.update(time);
        self.stroke_end.update(time);
        self.stroke_offset.update(time);
        true
    }

    fn render(&self, canvas: &Canvas, rect: SkRect, opacity: f32, draw_children: &mut dyn FnMut(&Canvas)) {
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        paint.set_alpha_f(opacity);

        // 1. Draw Fill
        if let Some(fc) = &self.fill_color {
            let mut fill_paint = paint.clone();
            fill_paint.set_style(PaintStyle::Fill);
            let c = fc.current_value;
            // Apply opacity
            fill_paint.set_color4f(Color4f::new(c.r, c.g, c.b, c.a * opacity), None);

            canvas.save();
            canvas.translate((rect.left, rect.top));

            canvas.draw_path(&self.path, &fill_paint);

            canvas.restore();
        }

        // 2. Draw Stroke (with Trim)
        let sw = self.stroke_width.current_value;
        if sw > 0.0 && self.stroke_color.is_some() {
             let sc_anim = self.stroke_color.as_ref().unwrap();
             let c = sc_anim.current_value;

             let mut stroke_paint = paint.clone();
             stroke_paint.set_style(PaintStyle::Stroke);
             stroke_paint.set_stroke_width(sw);
             stroke_paint.set_stroke_cap(self.stroke_cap);
             stroke_paint.set_stroke_join(self.stroke_join);
             stroke_paint.set_color4f(Color4f::new(c.r, c.g, c.b, c.a * opacity), None);

             let start = self.stroke_start.current_value.clamp(0.0, 1.0);
             let end = self.stroke_end.current_value.clamp(0.0, 1.0);
             let offset = self.stroke_offset.current_value;

             let (visible_start, visible_end) = if start <= end {
                 (start, end)
             } else {
                 (end, start)
             };

             let paths = self.get_trim_paths(visible_start, visible_end, offset);

             canvas.save();
             canvas.translate((rect.left, rect.top));

             for p in paths {
                 canvas.draw_path(&p, &stroke_paint);
             }

             canvas.restore();
        }

        draw_children(canvas);
    }

    fn animate_property(&mut self, property: &str, start: f32, target: f32, duration: f64, easing: &str) {
        let ease_fn = parse_easing(easing);
        match property {
            "stroke_width" | "line_width" => self.stroke_width.add_segment(start, target, duration, ease_fn),
            "stroke_start" | "trim_start" => self.stroke_start.add_segment(start, target, duration, ease_fn),
            "stroke_end" | "trim_end" => self.stroke_end.add_segment(start, target, duration, ease_fn),
            "stroke_offset" | "trim_offset" => self.stroke_offset.add_segment(start, target, duration, ease_fn),
            _ => {}
        }
    }

    fn animate_property_spring(&mut self, property: &str, start: Option<f32>, target: f32, config: crate::animation::SpringConfig) {
        let apply = |anim: &mut Animated<f32>| {
             if let Some(s) = start {
                 anim.add_spring_with_start(s, target, config);
             } else {
                 anim.add_spring(target, config);
             }
        };

        match property {
            "stroke_width" | "line_width" => apply(&mut self.stroke_width),
            "stroke_start" | "trim_start" => apply(&mut self.stroke_start),
            "stroke_end" | "trim_end" => apply(&mut self.stroke_end),
            "stroke_offset" | "trim_offset" => apply(&mut self.stroke_offset),
            _ => {}
        }
    }
}
