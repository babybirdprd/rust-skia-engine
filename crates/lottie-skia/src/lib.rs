use glam::{Mat4, Vec3, Vec4};
use kurbo::{BezPath, PathEl};
use lottie_core::{
    BlendMode as CoreBlendMode, ColorChannel as CoreColorChannel, Effect, FillRule as CoreFillRule,
    GradientKind, LayerStyle as CoreLayerStyle, LineCap as CoreLineCap, LineJoin as CoreLineJoin,
    MaskMode as CoreMaskMode, MatteMode, MergeMode, NodeContent, Paint as CorePaint, RenderNode,
    RenderTree, ShapeGeometry,
};
use skia_safe::color_filters::Clamp;
use skia_safe::{
    canvas::SaveLayerRec, color_filters, gradient_shader, image_filters, BlendMode, Canvas, ClipOp,
    Color, Color4f, ColorChannel, Data, Font, FontMgr, FontStyle, Image as SkImage, Matrix, M44, Paint,
    PaintStyle, Path, PathBuilder, PathEffect, PathFillType, PathOp, Point, Rect, RuntimeEffect,
    StrokeRec, TextBlob, TileMode,
};

pub trait LottieContext: Send + Sync {
    fn load_typeface(&self, family: &str, style: &str) -> Option<skia_safe::Typeface>;
    fn load_image(&self, id: &str) -> Option<skia_safe::Image>;
}

// Default no-op implementation
impl LottieContext for () {
    fn load_typeface(&self, _family: &str, _style: &str) -> Option<skia_safe::Typeface> {
        None
    }
    fn load_image(&self, _id: &str) -> Option<skia_safe::Image> {
        None
    }
}

pub struct SkiaRenderer;

impl SkiaRenderer {
    /// Draws the computed frame onto the provided canvas.
    pub fn draw(canvas: &Canvas, tree: &RenderTree, dest_rect: Rect, alpha: f32, ctx: &dyn LottieContext) {
        canvas.save();

        // 4.1 Coordinate System & Transforms
        let scale_x = sanitize(dest_rect.width() / tree.width);
        let scale_y = sanitize(dest_rect.height() / tree.height);
        let left = sanitize(dest_rect.left);
        let top = sanitize(dest_rect.top);

        let mut global_matrix = Matrix::translate((left, top));
        global_matrix.pre_scale((scale_x, scale_y), None);

        canvas.concat(&global_matrix);

        // Apply Camera Matrix (3D)
        let camera_matrix = tree.projection_matrix * tree.view_matrix;
        let m44 = glam_to_skia_m44(camera_matrix);
        // Try concat_44
        canvas.concat_44(&m44);

        // Draw Root Node
        draw_node(canvas, &tree.root, alpha, ctx);

        canvas.restore();
    }
}

fn draw_node(canvas: &Canvas, node: &RenderNode, parent_alpha: f32, ctx: &dyn LottieContext) {
    canvas.save();

    // Transform
    let m44 = glam_to_skia_m44(node.transform);
    canvas.concat_44(&m44);

    // Masks
    apply_masks(canvas, &node.masks);

    if node.is_adjustment_layer {
        if !node.effects.is_empty() || !node.styles.is_empty() {
            let mut filter = build_filter(&node.effects);
            if !node.styles.is_empty() {
                filter = build_layer_styles_filter(&node.styles, filter);
            }

            if let Some(f) = filter {
                let clip_path = collect_content_path(&node.content);
                canvas.save();
                canvas.clip_path(&clip_path, ClipOp::Intersect, true);
                canvas.save_layer(&SaveLayerRec::default().backdrop(&f));
                canvas.restore();
                canvas.restore();
            }
        }
        canvas.restore();
        return;
    }

    // Determine opacity
    let node_alpha = sanitize(node.alpha * parent_alpha);

    // Check if we need a layer
    let has_matte = node.matte.is_some();
    let has_effects = !node.effects.is_empty();
    let has_styles = !node.styles.is_empty();
    let non_normal_blend = !matches!(node.blend_mode, CoreBlendMode::Normal);
    let is_group = matches!(node.content, NodeContent::Group(_));

    let atomic_opacity_needed = is_group && node_alpha < 1.0;

    let need_layer = has_matte || has_effects || has_styles || non_normal_blend || atomic_opacity_needed;

    if need_layer {
        let mut paint = Paint::default();
        paint.set_alpha_f(node_alpha);
        paint.set_blend_mode(convert_blend_mode(node.blend_mode));

        // 4.5 Effects & Styles
        if has_effects || has_styles {
            let mut filter = build_filter(&node.effects);
            if has_styles {
                filter = build_layer_styles_filter(&node.styles, filter);
            }
            if let Some(f) = filter {
                paint.set_image_filter(f);
            }
        }

        if has_matte {
            // 4.4.2 Track Mattes
            canvas.save_layer(&SaveLayerRec::default().paint(&paint));

            // Draw Content
            draw_content(canvas, &node.content, 1.0, ctx);

            // Matte Logic
            if let Some(matte) = &node.matte {
                let mut matte_paint = Paint::default();
                let blend = match matte.mode {
                    MatteMode::Alpha => BlendMode::DstIn,
                    MatteMode::AlphaInverted => BlendMode::DstOut,
                    MatteMode::Luma => {
                        #[rustfmt::skip]
                         let matrix = [
                             0.0, 0.0, 0.0, 0.0, 0.0,
                             0.0, 0.0, 0.0, 0.0, 0.0,
                             0.0, 0.0, 0.0, 0.0, 0.0,
                             0.2126, 0.7152, 0.0722, 0.0, 0.0,
                         ];
                        matte_paint
                            .set_color_filter(color_filters::matrix_row_major(&matrix, Clamp::Yes));
                        BlendMode::DstIn
                    }
                    MatteMode::LumaInverted => {
                        #[rustfmt::skip]
                         let matrix = [
                             0.0, 0.0, 0.0, 0.0, 0.0,
                             0.0, 0.0, 0.0, 0.0, 0.0,
                             0.0, 0.0, 0.0, 0.0, 0.0,
                             -0.2126, -0.7152, -0.0722, 0.0, 1.0,
                         ];
                        matte_paint
                            .set_color_filter(color_filters::matrix_row_major(&matrix, Clamp::Yes));
                        BlendMode::DstIn
                    }
                };
                matte_paint.set_blend_mode(blend);

                canvas.save_layer(&SaveLayerRec::default().paint(&matte_paint));
                draw_node(canvas, &matte.node, 1.0, ctx);
                canvas.restore();
            }

            canvas.restore();
        } else {
            canvas.save_layer(&SaveLayerRec::default().paint(&paint));
            draw_content(canvas, &node.content, 1.0, ctx);
            canvas.restore();
        }
    } else {
        draw_content(canvas, &node.content, node_alpha, ctx);
    }

    draw_stroke_effects(canvas, node, node_alpha);

    canvas.restore();
}

fn draw_stroke_effects(canvas: &Canvas, node: &RenderNode, alpha: f32) {
    for effect in &node.effects {
        if let Effect::Stroke {
            color,
            width,
            opacity,
            mask_index,
            all_masks,
        } = effect
        {
            let mut paint = Paint::default();
            paint.set_style(PaintStyle::Stroke);
            paint.set_stroke_width(sanitize(*width));
            let c = glam_to_skia_color4f(*color);
            paint.set_color4f(c, None);
            paint.set_alpha_f(sanitize(*opacity * alpha));
            paint.set_stroke_cap(skia_safe::PaintCap::Round);
            paint.set_stroke_join(skia_safe::PaintJoin::Round);

            let mut masks_to_draw = Vec::new();
            if *all_masks {
                for m in &node.masks {
                    masks_to_draw.push(m);
                }
            } else if let Some(idx) = mask_index {
                if *idx > 0 && *idx <= node.masks.len() {
                    masks_to_draw.push(&node.masks[*idx - 1]);
                }
            }

            for mask in masks_to_draw {
                let path = kurbo_to_skia_path(&mask.geometry);
                canvas.draw_path(&path, &paint);
            }
        }
    }
}

fn draw_content(canvas: &Canvas, content: &NodeContent, alpha: f32, ctx: &dyn LottieContext) {
    match content {
        NodeContent::Group(children) => {
            for child in children {
                draw_node(canvas, child, alpha, ctx);
            }
        }
        NodeContent::Shape(shape) => {
            let mut path = resolve_geometry(&shape.geometry);

            if let Some(fill) = &shape.fill {
                path.set_fill_type(convert_fill_rule(fill.rule));
                let mut paint = Paint::default();
                paint.set_style(PaintStyle::Fill);
                paint.set_alpha_f(sanitize(fill.opacity * alpha));

                if let Some(trim) = &shape.trim {
                    if let Some(effect) = PathEffect::trim(
                        trim.start,
                        trim.end,
                        skia_safe::trim_path_effect::Mode::Normal,
                    ) {
                        paint.set_path_effect(effect);
                    }
                }

                setup_paint_shader(&mut paint, &fill.paint);
                canvas.draw_path(&path, &paint);
            }

            if let Some(stroke) = &shape.stroke {
                let mut paint = Paint::default();
                paint.set_style(PaintStyle::Stroke);
                paint.set_alpha_f(sanitize(stroke.opacity * alpha));
                paint.set_stroke_width(sanitize(stroke.width));
                paint.set_stroke_cap(convert_cap(stroke.cap));
                paint.set_stroke_join(convert_join(stroke.join));

                if let Some(miter) = stroke.miter_limit {
                    paint.set_stroke_miter(sanitize(miter));
                }

                let mut path_effect = None;
                if let Some(dash) = &stroke.dash {
                    let mut array: Vec<f32> = dash.array.iter().map(|&v| sanitize(v)).collect();
                    if array.len() % 2 != 0 {
                        let clone = array.clone();
                        array.extend(clone);
                    }
                    path_effect = PathEffect::dash(&array, sanitize(dash.offset));
                }

                if let Some(trim) = &shape.trim {
                    if let Some(trim_effect) = PathEffect::trim(
                        trim.start,
                        trim.end,
                        skia_safe::trim_path_effect::Mode::Normal,
                    ) {
                        path_effect = if let Some(pe) = path_effect {
                            Some(PathEffect::compose(trim_effect, pe))
                        } else {
                            Some(trim_effect)
                        }
                    }
                }

                if let Some(pe) = path_effect {
                    paint.set_path_effect(pe);
                }

                setup_paint_shader(&mut paint, &stroke.paint);
                canvas.draw_path(&path, &paint);
            }
        }
        NodeContent::Text(text) => {
            let typeface = ctx
                .load_typeface(&text.font_family, "Normal")
                .or_else(|| {
                    let font_mgr = FontMgr::new();
                    font_mgr
                        .match_family_style(&text.font_family, FontStyle::normal())
                        .or_else(|| font_mgr.match_family_style("Arial", FontStyle::normal()))
                        .or_else(|| font_mgr.match_family_style("", FontStyle::normal()))
                });

            if let Some(typeface) = typeface {
                let font = Font::new(typeface, Some(text.size));

                for glyph in &text.glyphs {
                    let char_str = glyph.character.to_string();
                    if char_str == "\n" {
                        continue;
                    }

                    canvas.save();

                    // 3D Transform for Glyph
                    let t = Mat4::from_translation(glyph.pos);
                    let r = Mat4::from_euler(glam::EulerRot::YXZ, glyph.rotation.y, glyph.rotation.x, glyph.rotation.z);
                    let s = Mat4::from_scale(glyph.scale);
                    let m = t * r * s;

                    let m44 = glam_to_skia_m44(m);
                    canvas.concat_44(&m44);

                    if let Some(blob) = TextBlob::from_str(&char_str, &font) {
                        if let Some(fill) = &glyph.fill {
                            let mut paint = Paint::default();
                            paint.set_style(PaintStyle::Fill);
                            paint.set_alpha_f(sanitize(fill.opacity * glyph.alpha * alpha));
                            setup_paint_shader(&mut paint, &fill.paint);
                            canvas.draw_text_blob(&blob, (0.0, 0.0), &paint);
                        }
                        if let Some(stroke) = &glyph.stroke {
                            let mut paint = Paint::default();
                            paint.set_style(PaintStyle::Stroke);
                            paint.set_alpha_f(sanitize(stroke.opacity * glyph.alpha * alpha));
                            paint.set_stroke_width(sanitize(stroke.width));
                            setup_paint_shader(&mut paint, &stroke.paint);
                            canvas.draw_text_blob(&blob, (0.0, 0.0), &paint);
                        }
                    }
                    canvas.restore();
                }
            }
        }
        NodeContent::Image(image) => {
            let mut drawn = false;

            // Try loading from context first if ID exists
            if let Some(id) = &image.id {
                if let Some(img) = ctx.load_image(id) {
                    let mut paint = Paint::default();
                    paint.set_alpha_f(alpha);
                    let src = Rect::from_wh(img.width() as f32, img.height() as f32);
                    let dst = Rect::from_wh(image.width as f32, image.height as f32);
                    canvas.draw_image_rect(
                        img,
                        Some((&src, skia_safe::canvas::SrcRectConstraint::Strict)),
                        dst,
                        &paint,
                    );
                    drawn = true;
                }
            }

            // Fallback to embedded data
            if !drawn {
                if let Some(data) = &image.data {
                    let sk_data = Data::new_copy(data);
                    if let Some(img) = SkImage::from_encoded(sk_data) {
                        let mut paint = Paint::default();
                        paint.set_alpha_f(alpha);
                        let src = Rect::from_wh(img.width() as f32, img.height() as f32);
                        let dst = Rect::from_wh(image.width as f32, image.height as f32);
                        canvas.draw_image_rect(
                            img,
                            Some((&src, skia_safe::canvas::SrcRectConstraint::Strict)),
                            dst,
                            &paint,
                        );
                        drawn = true;
                    }
                }
            }

            if !drawn {
                let mut paint = Paint::default();
                paint.set_color(Color::MAGENTA);
                paint.set_style(PaintStyle::Fill);
                canvas.draw_rect(
                    Rect::from_wh(image.width as f32, image.height as f32),
                    &paint,
                );
            }
        }
    }
}

fn collect_content_path(content: &NodeContent) -> Path {
    match content {
        NodeContent::Group(children) => {
            let mut group_path = Path::new();
            for child in children {
                let child_path = collect_node_path(child);
                if group_path.is_empty() {
                    group_path = child_path;
                } else {
                    if let Some(res) = group_path.op(&child_path, PathOp::Union) {
                        group_path = res;
                    } else {
                        group_path.add_path(&child_path, (0.0, 0.0), None);
                    }
                }
            }
            group_path
        }
        NodeContent::Shape(s) => resolve_geometry(&s.geometry),
        _ => Path::new(),
    }
}

fn resolve_geometry(geometry: &ShapeGeometry) -> Path {
    match geometry {
        ShapeGeometry::Path(p) => kurbo_to_skia_path(p),
        ShapeGeometry::Boolean { mode, shapes } => {
            if matches!(mode, MergeMode::Merge) {
                let mut path = Path::new();
                for shape in shapes {
                    let sub_path = resolve_geometry(shape);
                    path.add_path(&sub_path, (0.0, 0.0), None);
                }
                path
            } else {
                let mut path = Path::new();
                for (i, shape) in shapes.iter().enumerate() {
                    let sub_path = resolve_geometry(shape);
                    if i == 0 {
                        path = sub_path;
                    } else {
                        let op = match mode {
                            MergeMode::Add => PathOp::Union,
                            MergeMode::Subtract => PathOp::Difference,
                            MergeMode::Intersect => PathOp::Intersect,
                            MergeMode::Exclude => PathOp::XOR,
                            _ => PathOp::Union,
                        };
                        if let Some(res) = path.op(&sub_path, op) {
                            path = res;
                        }
                    }
                }
                path
            }
        }
    }
}

fn resolve_mask_path(mask: &lottie_core::Mask) -> Path {
    let mut path = kurbo_to_skia_path(&mask.geometry);
    if mask.expansion > 0.0 {
        let mut paint = Paint::default();
        paint.set_style(PaintStyle::Stroke);
        paint.set_stroke_width(sanitize(mask.expansion * 2.0));
        paint.set_stroke_cap(skia_safe::PaintCap::Round);
        paint.set_stroke_join(skia_safe::PaintJoin::Round);
        paint.set_stroke_miter(4.0);

        let stroke_rec = StrokeRec::from_paint(&paint, PaintStyle::Stroke, 1.0);
        let mut builder = PathBuilder::new();
        if stroke_rec.apply_to_path(&mut builder, &path) {
            let stroke_path = builder.detach(None);
            if let Some(res) = path.op(&stroke_path, PathOp::Union) {
                path = res;
            }
        }
    }
    path
}

fn apply_masks(canvas: &Canvas, masks: &[lottie_core::Mask]) {
    if masks.is_empty() {
        return;
    }
    let mut add_path = Path::new();
    let mut has_add = false;
    for mask in masks {
        if let CoreMaskMode::Add = mask.mode {
            let path = resolve_mask_path(mask);
            if !has_add {
                add_path = path;
                has_add = true;
            } else {
                if let Some(result) = add_path.op(&path, PathOp::Union) {
                    add_path = result;
                } else {
                    add_path.add_path(&path, (0.0, 0.0), None);
                }
            }
        }
    }
    if has_add {
        canvas.clip_path(&add_path, ClipOp::Intersect, true);
    }
    for mask in masks {
        match mask.mode {
            CoreMaskMode::Add => {},
            CoreMaskMode::None => {},
            CoreMaskMode::Subtract => {
                let path = resolve_mask_path(mask);
                canvas.clip_path(&path, ClipOp::Difference, true);
            }
            CoreMaskMode::Intersect => {
                let path = resolve_mask_path(mask);
                canvas.clip_path(&path, ClipOp::Intersect, true);
            }
            _ => {}
        }
    }
}

fn setup_paint_shader(paint: &mut Paint, core_paint: &CorePaint) {
    match core_paint {
        CorePaint::Solid(color) => {
            let c = glam_to_skia_color4f(*color);
            paint.set_color4f(c, None);
        }
        CorePaint::Gradient(grad) => {
            let colors: Vec<Color> = grad
                .stops
                .iter()
                .map(|s| glam_to_skia_color_legacy(s.color))
                .collect();
            let pos: Vec<f32> = grad.stops.iter().map(|s| sanitize(s.offset)).collect();
            let pt1 = Point::new(sanitize(grad.start.x), sanitize(grad.start.y));
            let pt2 = Point::new(sanitize(grad.end.x), sanitize(grad.end.y));

            let shader = match grad.kind {
                GradientKind::Linear => gradient_shader::linear(
                    (pt1, pt2),
                    colors.as_slice(),
                    Some(pos.as_slice()),
                    TileMode::Clamp,
                    None,
                    None,
                ),
                GradientKind::Radial => {
                    let radius = Point::distance(pt1, pt2);
                    gradient_shader::radial(
                        pt1,
                        radius,
                        colors.as_slice(),
                        Some(pos.as_slice()),
                        TileMode::Clamp,
                        None,
                        None,
                    )
                }
            };
            paint.set_shader(shader);
        }
    }
}

// ... helpers ...

fn sanitize(v: f32) -> f32 {
    if v.is_finite() {
        v
    } else {
        0.0
    }
}

fn glam_to_skia_m44(m: Mat4) -> M44 {
    let c0 = m.col(0);
    let c1 = m.col(1);
    let c2 = m.col(2);
    let c3 = m.col(3);

    // M44::new is Row-Major arguments
    M44::new(
        sanitize(c0.x), sanitize(c1.x), sanitize(c2.x), sanitize(c3.x),
        sanitize(c0.y), sanitize(c1.y), sanitize(c2.y), sanitize(c3.y),
        sanitize(c0.z), sanitize(c1.z), sanitize(c2.z), sanitize(c3.z),
        sanitize(c0.w), sanitize(c1.w), sanitize(c2.w), sanitize(c3.w),
    )
}

fn glam_mat4_to_skia_matrix_2d(m: Mat4) -> Matrix {
    let c0 = m.col(0);
    let c1 = m.col(1);
    let c3 = m.col(3);

    Matrix::new_all(
        sanitize(c0.x), sanitize(c1.x), sanitize(c3.x),
        sanitize(c0.y), sanitize(c1.y), sanitize(c3.y),
        0.0, 0.0, 1.0
    )
}

fn glam_to_skia_color4f(v: Vec4) -> Color4f {
    Color4f::new(sanitize(v.x), sanitize(v.y), sanitize(v.z), sanitize(v.w))
}

fn glam_to_skia_color_legacy(v: Vec4) -> Color {
    let c = glam_to_skia_color4f(v);
    c.to_color()
}

fn kurbo_to_skia_path(bez_path: &BezPath) -> Path {
    let mut path = Path::new();
    for el in bez_path.elements() {
        match el {
            PathEl::MoveTo(p) => {
                path.move_to((sanitize(p.x as f32), sanitize(p.y as f32)));
            }
            PathEl::LineTo(p) => {
                path.line_to((sanitize(p.x as f32), sanitize(p.y as f32)));
            }
            PathEl::QuadTo(p1, p2) => {
                path.quad_to(
                    (sanitize(p1.x as f32), sanitize(p1.y as f32)),
                    (sanitize(p2.x as f32), sanitize(p2.y as f32)),
                );
            }
            PathEl::CurveTo(p1, p2, p3) => {
                path.cubic_to(
                    (sanitize(p1.x as f32), sanitize(p1.y as f32)),
                    (sanitize(p2.x as f32), sanitize(p2.y as f32)),
                    (sanitize(p3.x as f32), sanitize(p3.y as f32)),
                );
            }
            PathEl::ClosePath => {
                path.close();
            }
        }
    }
    path
}

fn convert_blend_mode(mode: CoreBlendMode) -> BlendMode {
    match mode {
        CoreBlendMode::Normal => BlendMode::SrcOver,
        CoreBlendMode::Multiply => BlendMode::Multiply,
        CoreBlendMode::Screen => BlendMode::Screen,
        CoreBlendMode::Overlay => BlendMode::Overlay,
        CoreBlendMode::Darken => BlendMode::Darken,
        CoreBlendMode::Lighten => BlendMode::Lighten,
        CoreBlendMode::ColorDodge => BlendMode::ColorDodge,
        CoreBlendMode::ColorBurn => BlendMode::ColorBurn,
        CoreBlendMode::HardLight => BlendMode::HardLight,
        CoreBlendMode::SoftLight => BlendMode::SoftLight,
        CoreBlendMode::Difference => BlendMode::Difference,
        CoreBlendMode::Exclusion => BlendMode::Exclusion,
        CoreBlendMode::Hue => BlendMode::Hue,
        CoreBlendMode::Saturation => BlendMode::Saturation,
        CoreBlendMode::Color => BlendMode::Color,
        CoreBlendMode::Luminosity => BlendMode::Luminosity,
    }
}

fn convert_fill_rule(rule: CoreFillRule) -> PathFillType {
    match rule {
        CoreFillRule::NonZero => PathFillType::Winding,
        CoreFillRule::EvenOdd => PathFillType::EvenOdd,
    }
}

fn convert_cap(cap: CoreLineCap) -> skia_safe::PaintCap {
    match cap {
        CoreLineCap::Butt => skia_safe::PaintCap::Butt,
        CoreLineCap::Round => skia_safe::PaintCap::Round,
        CoreLineCap::Square => skia_safe::PaintCap::Square,
    }
}

fn convert_join(join: CoreLineJoin) -> skia_safe::PaintJoin {
    match join {
        CoreLineJoin::Miter => skia_safe::PaintJoin::Miter,
        CoreLineJoin::Round => skia_safe::PaintJoin::Round,
        CoreLineJoin::Bevel => skia_safe::PaintJoin::Bevel,
    }
}

fn convert_color_channel(c: CoreColorChannel) -> ColorChannel {
    match c {
        CoreColorChannel::R => ColorChannel::R,
        CoreColorChannel::G => ColorChannel::G,
        CoreColorChannel::B => ColorChannel::B,
        CoreColorChannel::A => ColorChannel::A,
    }
}

// ... existing build_filter ...

fn build_filter(effects: &[Effect]) -> Option<skia_safe::ImageFilter> {
    let mut filter: Option<skia_safe::ImageFilter> = None;
    for effect in effects {
        let next_filter = match effect {
            Effect::GaussianBlur { sigma } => {
                let s = sanitize(*sigma);
                image_filters::blur((s, s), TileMode::Decal, filter.clone(), None)
            }
            Effect::DropShadow {
                color,
                offset,
                blur,
            } => {
                let c = glam_to_skia_color_legacy(*color);
                let dx = sanitize(offset.x);
                let dy = sanitize(offset.y);
                let b = sanitize(*blur);
                image_filters::drop_shadow((dx, dy), (b, b), c, None, filter, None)
            }
            Effect::ColorMatrix { matrix } => image_filters::color_filter(
                color_filters::matrix_row_major(matrix, Clamp::Yes),
                filter,
                None,
            ),
            Effect::DisplacementMap {
                scale,
                x_channel,
                y_channel,
            } => image_filters::displacement_map(
                (
                    convert_color_channel(*x_channel),
                    convert_color_channel(*y_channel),
                ),
                sanitize(*scale),
                None,
                filter,
                None,
            ),
            Effect::Fill { color, opacity } => {
                let c = glam_to_skia_color_legacy(*color);
                let a = sanitize(*opacity);
                if let Some(fill_cf) = color_filters::blend(c, BlendMode::SrcIn) {
                    let identity = color_filters::matrix_row_major(
                        &[
                            1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0,
                            0.0, 0.0, 0.0, 0.0, 1.0, 0.0,
                        ],
                        color_filters::Clamp::Yes,
                    );
                    if let Some(lerp_cf) = color_filters::lerp(a, &fill_cf, &identity) {
                        image_filters::color_filter(lerp_cf, filter, None)
                    } else {
                        image_filters::color_filter(fill_cf, filter, None)
                    }
                } else {
                    filter
                }
            }
            Effect::Tint {
                black,
                white,
                amount,
            } => {
                let sksl = r#"
                    uniform vec4 uBlack;
                    uniform vec4 uWhite;
                    uniform float uAmount;

                    half4 main(half4 color) {
                        float lum = dot(color.rgb, half3(0.2126, 0.7152, 0.0722));
                        vec3 mapped = mix(uBlack.rgb, uWhite.rgb, lum);
                        vec3 result = mix(color.rgb, mapped, uAmount);
                        return half4(result, color.a);
                    }
                "#;
                if let Ok(effect) = RuntimeEffect::make_for_color_filter(sksl, None) {
                    let mut data = Vec::with_capacity(36);
                    for v in [
                        black.x, black.y, black.z, black.w, white.x, white.y, white.z, white.w,
                        *amount,
                    ] {
                        data.extend_from_slice(&v.to_ne_bytes());
                    }
                    let uniforms = Data::new_copy(&data);
                    if let Some(cf) = effect.make_color_filter(uniforms, None) {
                        image_filters::color_filter(cf, filter, None)
                    } else {
                        filter
                    }
                } else {
                    filter
                }
            }
            Effect::Tritone {
                highlights,
                midtones,
                shadows,
            } => {
                let sksl = r#"
                    uniform vec4 uHighlights;
                    uniform vec4 uMidtones;
                    uniform vec4 uShadows;

                    half4 main(half4 color) {
                        float lum = dot(color.rgb, half3(0.2126, 0.7152, 0.0722));
                        vec3 mapped;
                        if (lum < 0.5) {
                            mapped = mix(uShadows.rgb, uMidtones.rgb, lum * 2.0);
                        } else {
                            mapped = mix(uMidtones.rgb, uHighlights.rgb, (lum - 0.5) * 2.0);
                        }
                        return half4(mapped, color.a);
                    }
                "#;
                if let Ok(effect) = RuntimeEffect::make_for_color_filter(sksl, None) {
                    let mut data = Vec::with_capacity(48);
                    for v in [
                        highlights.x,
                        highlights.y,
                        highlights.z,
                        highlights.w,
                        midtones.x,
                        midtones.y,
                        midtones.z,
                        midtones.w,
                        shadows.x,
                        shadows.y,
                        shadows.z,
                        shadows.w,
                    ] {
                        data.extend_from_slice(&v.to_ne_bytes());
                    }
                    let uniforms = Data::new_copy(&data);
                    if let Some(cf) = effect.make_color_filter(uniforms, None) {
                        image_filters::color_filter(cf, filter, None)
                    } else {
                        filter
                    }
                } else {
                    filter
                }
            }
            Effect::Stroke { .. } | Effect::Levels { .. } => filter,
        };
        filter = next_filter;
    }
    filter
}

fn collect_node_path(node: &RenderNode) -> Path {
    let mut path = collect_content_path(&node.content);
    let matrix = glam_mat4_to_skia_matrix_2d(node.transform);
    path.transform(&matrix);
    path
}

// Helper for Layer Styles
fn build_layer_styles_filter(
    styles: &[CoreLayerStyle],
    input: Option<skia_safe::ImageFilter>,
) -> Option<skia_safe::ImageFilter> {
    let mut current = input;

    for style in styles {
        match style {
            CoreLayerStyle::DropShadow {
                color,
                opacity,
                angle,
                distance,
                size,
                spread,
            } => {
                let c = glam_to_skia_color_legacy(*color).with_a(sanitize((opacity * 255.0).round()) as u8);
                let dx = sanitize(distance * (angle.to_radians() - std::f32::consts::PI / 2.0).cos());
                let dy = sanitize(distance * (angle.to_radians() - std::f32::consts::PI / 2.0).sin());
                let b = sanitize(*size);

                // Spread logic: Dilate input, then Shadow
                let shadow_input = if *spread > 0.0 {
                    let dilation = sanitize(size * (spread / 100.0));
                    image_filters::dilate((dilation, dilation), current.clone(), None)
                } else {
                    current.clone()
                };

                let shadow = image_filters::drop_shadow_only((dx, dy), (b, b), c, None, shadow_input, None);

                // Composite: Shadow behind Source (Current)
                // Source Over Shadow
                // merge([Shadow, Source]) -> Painter's algo -> Draws Shadow then Source. Correct.
                current = image_filters::merge(vec![shadow, current], None);
            }
            CoreLayerStyle::InnerShadow {
                color,
                opacity,
                angle,
                distance,
                size,
                choke,
            } => {
                // Inner Shadow
                // 1. Invert Alpha of Source
                // 2. Drop Shadow of Inverted
                // 3. Mask with Source
                // 4. Composite Over Source

                let c = glam_to_skia_color_legacy(*color).with_a(sanitize((opacity * 255.0).round()) as u8);
                let dx = sanitize(distance * (angle.to_radians() - std::f32::consts::PI / 2.0).cos());
                let dy = sanitize(distance * (angle.to_radians() - std::f32::consts::PI / 2.0).sin());
                let b = sanitize(*size);

                // Invert Alpha
                // ColorMatrix: A' = 1 - A
                #[rustfmt::skip]
                let matrix = [
                     0.0, 0.0, 0.0, 0.0, 0.0,
                     0.0, 0.0, 0.0, 0.0, 0.0,
                     0.0, 0.0, 0.0, 0.0, 0.0,
                     0.0, 0.0, 0.0, -1.0, 1.0,
                 ];
                 let inverted = image_filters::color_filter(
                     color_filters::matrix_row_major(&matrix, Clamp::Yes),
                     current.clone(),
                     None
                 );

                 // Shadow of Inverted
                 // Choke? Shrink hole -> Erode Inverted?
                 let shadow_input = if *choke > 0.0 {
                     let erosion = sanitize(size * (choke / 100.0));
                     image_filters::erode((erosion, erosion), inverted, None)
                 } else {
                     inverted
                 };

                 let shadow = image_filters::drop_shadow_only((dx, dy), (b, b), c, None, shadow_input, None);

                 // Mask with Source (Source In Shadow -> Shadow where Source exists)
                 // KDstIn: Dst (Shadow) In Src (Source)
                 // blend(DstIn, background=Shadow, foreground=Source)?
                 // blend: (src, dst) -> src OP dst.
                 // We want Shadow masked by SourceAlpha.
                 // Src=Source, Dst=Shadow.
                 // DstIn: Dst * SrcAlpha.
                 // blend(mode=DstIn, dst=Shadow, src=Source)
                 let masked_shadow = image_filters::blend(
                     BlendMode::DstIn,
                     shadow,
                     current.clone(), // Source
                     None
                 );

                 // Composite: Source Over Shadow?
                 // Inner Shadow is inside. So Source (Normal) -> Shadow on top?
                 // Yes, Inner Shadow draws ON TOP of the object.
                 // merge([Source, Shadow])
                 current = image_filters::merge(vec![current, masked_shadow], None);
            }
            CoreLayerStyle::OuterGlow {
                color,
                opacity,
                size,
                range,
            } => {
                let c = glam_to_skia_color_legacy(*color).with_a(sanitize((opacity * 255.0).round()) as u8);
                let b = sanitize(*size);

                // Spread/Range logic similar to DropShadow
                let glow_input = if *range > 0.0 {
                    let dilation = sanitize(size * (range / 100.0));
                    image_filters::dilate((dilation, dilation), current.clone(), None)
                } else {
                    current.clone()
                };

                let glow = image_filters::drop_shadow_only((0.0, 0.0), (b, b), c, None, glow_input, None);

                // Composite: Glow Behind Source
                current = image_filters::merge(vec![glow, current], None);
            }
            CoreLayerStyle::Stroke {
                color,
                width,
                opacity,
            } => {
                // Outside Stroke
                let c = glam_to_skia_color_legacy(*color).with_a(sanitize((opacity * 255.0).round()) as u8);
                let s = sanitize(*width);

                // Dilate
                let dilated = image_filters::dilate((s, s), current.clone(), None);

                // Colorize Dilated
                // SrcIn with Color?
                let color_filter = color_filters::blend(c, BlendMode::SrcIn);
                let stroke_base = image_filters::color_filter(color_filter.unwrap(), dilated, None);

                // Composite: Stroke Behind Source
                current = image_filters::merge(vec![stroke_base, current], None);
            }
        }
    }
    current
}
