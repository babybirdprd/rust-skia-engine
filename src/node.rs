use skia_safe::{
    Canvas, Paint, Rect, RRect, ClipOp, PaintStyle, Image, Color4f, Data, FontMgr,
    FontStyle, ColorType, AlphaType, Path, Point, gradient_shader, TileMode, Matrix, TextBlobBuilder,
    font_style::{Weight as SkWeight, Width as SkWidth, Slant as SkSlant}
};
use taffy::style::Style;
use crate::element::{Element, Color, TextSpan};
use crate::animation::{Animated, EasingType};
use cosmic_text::{Buffer, FontSystem, Metrics, SwashCache, Attrs, AttrsList, Shaping, Weight, Style as CosmicStyle, Family};
use std::sync::{Arc, Mutex};
use std::fmt;
use std::io::Write;
use tempfile::NamedTempFile;
use unicode_segmentation::UnicodeSegmentation;
use std::ops::Range;

// Video imports
use crate::video_wrapper::{AsyncDecoder, RenderMode, VideoResponse};

// Helper to parse easing
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

// --- Box Node ---
#[derive(Debug)]
pub struct BoxNode {
    pub style: Style,
    pub bg_color: Option<Animated<Color>>,
    pub opacity: Animated<f32>,
    pub blur: Animated<f32>,
    pub shadow_color: Option<Animated<Color>>,
    pub shadow_blur: Animated<f32>,
    pub shadow_offset_x: Animated<f32>,
    pub shadow_offset_y: Animated<f32>,
    // New fields
    pub border_radius: Animated<f32>,
    pub border_width: Animated<f32>,
    pub border_color: Option<Animated<Color>>,
    pub overflow: String,
}

impl BoxNode {
    pub fn new() -> Self {
        Self {
            style: Style::default(),
            bg_color: None,
            opacity: Animated::new(1.0),
            blur: Animated::new(0.0),
            shadow_color: None,
            shadow_blur: Animated::new(0.0),
            shadow_offset_x: Animated::new(0.0),
            shadow_offset_y: Animated::new(0.0),
            border_radius: Animated::new(0.0),
            border_width: Animated::new(0.0),
            border_color: None,
            overflow: "visible".to_string(),
        }
    }
}

impl Element for BoxNode {
    fn layout_style(&self) -> Style {
        self.style.clone()
    }

    fn update(&mut self, time: f64) -> bool {
        let mut changed = false;
        if let Some(bg) = &mut self.bg_color {
            bg.update(time);
            changed = true;
        }
        if let Some(sc) = &mut self.shadow_color {
            sc.update(time);
            changed = true;
        }
        if let Some(bc) = &mut self.border_color {
            bc.update(time);
            changed = true;
        }
        self.opacity.update(time);
        self.blur.update(time);
        self.shadow_blur.update(time);
        self.shadow_offset_x.update(time);
        self.shadow_offset_y.update(time);
        self.border_radius.update(time);
        self.border_width.update(time);
        changed
    }

    fn render(&self, canvas: &Canvas, rect: Rect, opacity: f32, draw_children: &mut dyn FnMut(&Canvas)) {
        let local_opacity = self.opacity.current_value * opacity;
        let radius = self.border_radius.current_value;
        let rrect = RRect::new_rect_xy(&rect, radius, radius);

        canvas.save();

        if self.overflow == "hidden" {
            canvas.clip_rrect(rrect, ClipOp::Intersect, true);
        }

        let mut paint = Paint::default();
        paint.set_anti_alias(true);

        let mut current_filter = None;

        if self.blur.current_value > 0.0 {
            let sigma = self.blur.current_value;
            current_filter = skia_safe::image_filters::blur(
                (sigma, sigma),
                skia_safe::TileMode::Decal,
                current_filter.clone(),
                None
            );
        }

        if let Some(sc) = &self.shadow_color {
            let color = sc.current_value.to_skia();
            let sigma = self.shadow_blur.current_value;
            let dx = self.shadow_offset_x.current_value;
            let dy = self.shadow_offset_y.current_value;

            current_filter = skia_safe::image_filters::drop_shadow(
                (dx, dy),
                (sigma, sigma),
                color,
                None,
                current_filter,
                None
            );
        }

        if let Some(filter) = current_filter {
            paint.set_image_filter(filter);
        }

        if let Some(bg) = &self.bg_color {
            let mut c = bg.current_value;
            c.a *= local_opacity;
            paint.set_color4f(c.to_color4f(), None);
            canvas.draw_rrect(rrect, &paint);
        }

        // Draw children (clipped if overflow: hidden)
        draw_children(canvas);

        canvas.restore(); // Restore clip and transform (except we didn't transform here, but clip)

        // Draw Border
        let bw = self.border_width.current_value;
        if bw > 0.0 {
            let mut border_paint = Paint::default();
            border_paint.set_anti_alias(true);
            border_paint.set_style(PaintStyle::Stroke);
            border_paint.set_stroke_width(bw);

            let color = if let Some(bc) = &self.border_color {
                bc.current_value
            } else {
                Color::BLACK
            };

            let mut c = color;
            c.a *= local_opacity;
            border_paint.set_color4f(c.to_color4f(), None);

            canvas.draw_rrect(rrect, &border_paint);
        }
    }

    fn animate_property(&mut self, property: &str, start: f32, target: f32, duration: f64, easing: &str) {
        let ease_fn = parse_easing(easing);
        match property {
            "opacity" => self.opacity.add_segment(start, target, duration, ease_fn),
            "blur" => self.blur.add_segment(start, target, duration, ease_fn),
            "shadow_blur" => self.shadow_blur.add_segment(start, target, duration, ease_fn),
            "shadow_x" => self.shadow_offset_x.add_segment(start, target, duration, ease_fn),
            "shadow_y" => self.shadow_offset_y.add_segment(start, target, duration, ease_fn),
            "border_radius" => self.border_radius.add_segment(start, target, duration, ease_fn),
            "border_width" => self.border_width.add_segment(start, target, duration, ease_fn),
            _ => {}
        }
    }
}

// --- Text Node ---

#[derive(Debug)]
pub struct TextAnimator {
    pub range: Range<usize>, // Grapheme indices
    pub property: String,    // "offset_x", "offset_y", "rotation", "scale", "opacity"
    pub animation: Animated<f32>,
}

pub struct TextNode {
    pub spans: Vec<TextSpan>,
    pub default_font_size: Animated<f32>,
    pub default_color: Animated<Color>,
    pub buffer: Mutex<Option<Buffer>>,
    pub font_system: Arc<Mutex<FontSystem>>,
    pub swash_cache: Arc<Mutex<SwashCache>>,
    // RFC 009
    pub animators: Vec<TextAnimator>,
    pub grapheme_starts: Vec<usize>, // Byte offsets of each grapheme start
}

impl fmt::Debug for TextNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TextNode")
         .field("spans", &self.spans)
         .field("animators", &self.animators)
         .finish()
    }
}

impl TextNode {
    pub fn new(spans: Vec<TextSpan>, font_system: Arc<Mutex<FontSystem>>, swash_cache: Arc<Mutex<SwashCache>>) -> Self {
        let mut node = Self {
            spans,
            default_font_size: Animated::new(20.0),
            default_color: Animated::new(Color::WHITE),
            buffer: Mutex::new(None),
            font_system: font_system.clone(),
            swash_cache,
            animators: Vec::new(),
            grapheme_starts: Vec::new(),
        };
        node.init_buffer();
        node
    }

    pub fn init_buffer(&mut self) {
         let mut fs = self.font_system.lock().unwrap();
         let mut buffer = Buffer::new(&mut fs, Metrics::new(20.0, 24.0));

         let mut full_text = String::new();
         let mut attrs_list = AttrsList::new(&Attrs::new());

         for span in &self.spans {
             let start = full_text.len();
             full_text.push_str(&span.text);
             let end = full_text.len();

             let mut attrs = Attrs::new();
             if let Some(w) = span.font_weight {
                 attrs = attrs.weight(Weight(w));
             }
             if let Some(s) = &span.font_style {
                 if s == "italic" {
                     attrs = attrs.style(CosmicStyle::Italic);
                 }
             }
             if let Some(f) = &span.font_family {
                 attrs = attrs.family(Family::Name(f));
             }
             if let Some(c) = &span.color {
                 let cc = cosmic_text::Color::rgba(
                     (c.r * 255.0) as u8,
                     (c.g * 255.0) as u8,
                     (c.b * 255.0) as u8,
                     (c.a * 255.0) as u8,
                 );
                 attrs = attrs.color(cc);
             }

             attrs_list.add_span(start..end, &attrs);
         }

         let default_attrs = Attrs::new();
         buffer.set_text(&mut fs, &full_text, &default_attrs, Shaping::Advanced, None);

         if !buffer.lines.is_empty() {
             buffer.lines[0].set_attrs_list(attrs_list);
         }

         *self.buffer.lock().unwrap() = Some(buffer);

         // Calculate grapheme starts
         self.grapheme_starts = full_text
             .grapheme_indices(true)
             .map(|(i, _)| i)
             .collect();
    }

    fn build_span_ranges(&self) -> Vec<std::ops::Range<usize>> {
        let mut ranges = Vec::new();
        let mut start = 0;
        for span in &self.spans {
            let len = span.text.len();
            ranges.push(start..start + len);
            start += len;
        }
        ranges
    }
}

impl Element for TextNode {
    fn layout_style(&self) -> Style {
        Style::DEFAULT
    }

    fn update(&mut self, time: f64) -> bool {
        self.default_font_size.update(time);
        self.default_color.update(time);

        // Update Animators
        for anim in &mut self.animators {
            anim.animation.update(time);
        }

        let size = self.default_font_size.current_value;
        let line_height = size * 1.2;

        let mut buf_guard = self.buffer.lock().unwrap();
        if let Some(buffer) = buf_guard.as_mut() {
            let mut fs = self.font_system.lock().unwrap();
            buffer.set_metrics(&mut fs, Metrics::new(size, line_height));
        }
        true
    }

    fn render(&self, canvas: &Canvas, rect: Rect, opacity: f32, draw_children: &mut dyn FnMut(&Canvas)) {
        let mut buf_guard = self.buffer.lock().unwrap();
        let _sc_guard = self.swash_cache.lock().unwrap();

        if let Some(buffer) = buf_guard.as_mut() {
            let mut fs = self.font_system.lock().unwrap();

            buffer.set_size(&mut fs, Some(rect.width()), Some(rect.height()));
            buffer.shape_until_scroll(&mut fs, false);

            let font_mgr = FontMgr::default();
            // Default font setup (fallback)
            let typeface = font_mgr.match_family_style("Sans Serif", FontStyle::normal()).unwrap();
            let font = skia_safe::Font::new(typeface, Some(self.default_font_size.current_value));

            let ranges = self.build_span_ranges();

            // Pass 1: Backgrounds (Unmodified)
            for (span_idx, span) in self.spans.iter().enumerate() {
                if span.background_color.is_some() {
                    let mut path = Path::new();
                    let padding = span.background_padding.unwrap_or(0.0);
                    let mut has_rects = false;

                    for run in buffer.layout_runs() {
                        let mut first_x = None;
                        let mut last_x = None;
                        let mut last_w = 0.0;

                        // Find extent of this span in this line
                        for glyph in run.glyphs {
                            if ranges[span_idx].contains(&glyph.start) {
                                if first_x.is_none() { first_x = Some(glyph.x); }
                                last_x = Some(glyph.x);
                                last_w = glyph.w;
                            }
                        }

                        if let (Some(fx), Some(lx)) = (first_x, last_x) {
                            let w = (lx + last_w) - fx;
                            let r = Rect::from_xywh(
                                rect.left + fx - padding,
                                rect.top + run.line_y - padding,
                                w + padding * 2.0,
                                run.line_height + padding * 2.0
                            );
                            path.add_rect(r, None);
                            has_rects = true;
                        }
                    }

                    if has_rects {
                        if let Some(c) = span.background_color {
                            let mut p = Paint::default();
                            p.set_color4f(c.to_color4f(), None);
                            p.set_anti_alias(true);
                            canvas.draw_path(&path, &p);
                        }
                    }
                }
            }

            // Pass 2: Text Glyphs (RFC 009)
            for run in buffer.layout_runs() {
                 let origin_y = rect.top + run.line_y;
                 let origin_x = rect.left;

                 for glyph in run.glyphs.iter() {
                     // 1. Identify Grapheme Index
                     let grapheme_idx = match self.grapheme_starts.binary_search(&glyph.start) {
                         Ok(i) => i,
                         Err(i) => i.saturating_sub(1),
                     };

                     // 2. Identify Span (for styling)
                     let span_idx = ranges.iter().position(|r| r.contains(&glyph.start)).unwrap_or(0);
                     let span = &self.spans[span_idx];

                     // 3. Compute Animators
                     let mut offset_x = 0.0;
                     let mut offset_y = 0.0;
                     let mut rotation = 0.0;
                     let mut scale = 1.0;
                     let mut alpha = 1.0;

                     for anim in &self.animators {
                         if anim.range.contains(&grapheme_idx) {
                             let val = anim.animation.current_value;
                             match anim.property.as_str() {
                                 "x" | "offset_x" => offset_x += val,
                                 "y" | "offset_y" => offset_y += val,
                                 "rotation" => rotation += val,
                                 "scale" => scale *= val,
                                 "opacity" => alpha *= val,
                                 _ => {}
                             }
                         }
                     }

                     // 4. Resolve Style
                     let size = span.font_size.unwrap_or(self.default_font_size.current_value);
                     let mut typeface = font.typeface();

                     if span.font_weight.is_some() || span.font_style.is_some() || span.font_family.is_some() {
                         let mgr = FontMgr::default();
                         let family = span.font_family.as_deref().unwrap_or("Sans Serif");
                         let weight = span.font_weight.map(|w| SkWeight::from(w as i32)).unwrap_or(SkWeight::NORMAL);
                         let slant = if span.font_style.as_deref() == Some("italic") {
                             SkSlant::Italic
                         } else {
                             SkSlant::Upright
                         };
                         if let Some(tf) = mgr.match_family_style(family, FontStyle::new(weight, SkWidth::NORMAL, slant)) {
                             typeface = tf;
                         }
                     }
                     let glyph_font = skia_safe::Font::new(typeface, Some(size));

                     // 5. Create Blob using TextBlobBuilder (Preserve Ligatures)
                     let mut builder = TextBlobBuilder::new();
                     let glyph_id = glyph.glyph_id as u16;
                     let blob_buffer = builder.alloc_run(&glyph_font, 1, (0.0, 0.0), None);
                     blob_buffer[0] = glyph_id;

                     if let Some(blob) = builder.make() {
                         canvas.save();

                         // Position
                         let x = origin_x + glyph.x;
                         let y = origin_y + glyph.y;

                         // Pivot: Center of Glyph
                         let mut bounds = [Rect::default(); 1];
                         glyph_font.get_bounds(&[glyph_id], &mut bounds, None);
                         let bound = bounds[0];
                         // Bound is relative to (0,0) of the glyph origin.
                         let pivot_x = bound.center_x();
                         let pivot_y = bound.center_y();

                         let px = x + pivot_x;
                         let py = y + pivot_y;

                         // Apply Transforms
                         canvas.translate((px, py));
                         canvas.rotate(rotation, None);
                         canvas.scale((scale, scale));
                         canvas.translate((-px, -py));

                         // Apply Offset
                         canvas.translate((offset_x, offset_y));

                         // Setup Paint
                         let mut paint = Paint::default();
                         paint.set_anti_alias(true);

                         // Color & Opacity
                         let base_color = span.color.unwrap_or(self.default_color.current_value);
                         let mut final_color = base_color;
                         final_color.a *= alpha * opacity;

                         // Stroke
                         if let Some(sw) = span.stroke_width {
                             if sw > 0.0 {
                                 let mut sp = paint.clone();
                                 sp.set_style(PaintStyle::Stroke);
                                 sp.set_stroke_width(sw);
                                 if let Some(sc) = span.stroke_color {
                                     let mut sc_c = sc;
                                     sc_c.a *= alpha * opacity;
                                     sp.set_color4f(sc_c.to_color4f(), None);
                                 } else {
                                     sp.set_color(skia_safe::Color::BLACK);
                                     sp.set_alpha_f(alpha * opacity);
                                 }
                                 canvas.draw_text_blob(&blob, (x, y), &sp);
                             }
                         }

                         // Fill
                         paint.set_style(PaintStyle::Fill);
                         if let Some(grad) = &span.fill_gradient {
                             // Gradient Logic: Relative to Node's layout rect
                             // Node's layout rect (0,0 to w,h) in local coords
                             let w = rect.width();
                             let h = rect.height();
                             let origin_rect = Rect::from_xywh(0.0, 0.0, w, h);

                             let p0 = Point::new(
                                 origin_rect.left + grad.start.0 * w,
                                 origin_rect.top + grad.start.1 * h
                             );
                             let p1 = Point::new(
                                 origin_rect.left + grad.end.0 * w,
                                 origin_rect.top + grad.end.1 * h
                             );
                             let colors: Vec<skia_safe::Color> = grad.colors.iter().map(|c| c.to_skia()).collect();
                             let positions = grad.positions.as_ref().map(|v| v.as_slice());

                             // Calculate local matrix to undo the glyph's translation
                             // We are currently translated to (x+off, y+off).
                             // We want (0,0) in our shader to map to (x+off, y+off) in global.
                             // So we translate shader by (x+off, y+off).
                             let matrix = Matrix::translate((x + offset_x, y + offset_y));

                             if let Some(shader) = gradient_shader::linear(
                                 (p0, p1),
                                 colors.as_slice(),
                                 positions,
                                 TileMode::Clamp,
                                 None,
                                 Some(&matrix) // Apply local matrix
                             ) {
                                 paint.set_shader(shader);
                                 paint.set_alpha_f(alpha * opacity);
                             }
                         } else {
                             paint.set_color4f(final_color.to_color4f(), None);
                         }

                         canvas.draw_text_blob(&blob, (x, y), &paint);
                         canvas.restore();
                     }
                 }
            }
        }
        draw_children(canvas);
    }

    fn animate_property(&mut self, property: &str, start: f32, target: f32, duration: f64, easing: &str) {
        let ease_fn = parse_easing(easing);
        match property {
            "font_size" | "size" => self.default_font_size.add_segment(start, target, duration, ease_fn),
            _ => {}
        }
    }

    fn set_rich_text(&mut self, spans: Vec<TextSpan>) {
        self.spans = spans;
        self.init_buffer();
    }

    fn modify_text_spans(&mut self, f: &dyn Fn(&mut Vec<TextSpan>)) {
        f(&mut self.spans);
        self.init_buffer();
    }

    // RFC 009
    fn add_text_animator(
        &mut self,
        start_idx: usize,
        end_idx: usize,
        property: String,
        start_val: f32,
        target_val: f32,
        duration: f64,
        easing: &str
    ) {
        let ease_fn = parse_easing(easing);
        let mut anim = Animated::new(start_val);
        anim.add_keyframe(target_val, duration, ease_fn);

        let animator = TextAnimator {
            range: start_idx..end_idx,
            property,
            animation: anim,
        };
        self.animators.push(animator);
    }
}

// --- Image Node ---
#[derive(Debug)]
pub struct ImageNode {
    pub image: Option<Image>,
    pub opacity: Animated<f32>,
}

impl ImageNode {
    pub fn new(data: Vec<u8>) -> Self {
        let image = Image::from_encoded(Data::new_copy(&data));
        Self {
            image,
            opacity: Animated::new(1.0),
        }
    }
}

impl Element for ImageNode {
    fn layout_style(&self) -> Style {
        Style::DEFAULT
    }

    fn update(&mut self, time: f64) -> bool {
        self.opacity.update(time);
        true
    }

    fn render(&self, canvas: &Canvas, rect: Rect, parent_opacity: f32, draw_children: &mut dyn FnMut(&Canvas)) {
        let op = self.opacity.current_value * parent_opacity;
        let mut paint = Paint::new(Color4f::new(1.0, 1.0, 1.0, op), None);
        paint.set_anti_alias(true);

        if let Some(img) = &self.image {
             let sampling = skia_safe::SamplingOptions::new(
                 skia_safe::FilterMode::Linear,
                 skia_safe::MipmapMode::Linear
             );
             canvas.draw_image_rect_with_sampling_options(img, None, rect, sampling, &paint);
        }
        draw_children(canvas);
    }

    fn animate_property(&mut self, property: &str, start: f32, target: f32, duration: f64, easing: &str) {
        let ease_fn = parse_easing(easing);
        if property == "opacity" {
            self.opacity.add_segment(start, target, duration, ease_fn);
        }
    }
}

// --- Video Node ---
#[derive(Debug)]
pub struct VideoNode {
    pub opacity: Animated<f32>,
    current_frame: Mutex<Option<(f64, Image)>>,

    decoder: Option<AsyncDecoder>,
    render_mode: RenderMode,

    // Keep temp file alive
    #[allow(dead_code)]
    temp_file: Arc<NamedTempFile>,
}

impl VideoNode {
    pub fn new(data: Vec<u8>, mode: RenderMode) -> Self {
        // Write data to temp file
        let mut temp = NamedTempFile::new().expect("Failed to create temp file");
        temp.write_all(&data).expect("Failed to write video data");
        let path = temp.path().to_owned();
        let temp_arc = Arc::new(temp);

        let decoder = AsyncDecoder::new(path, mode).ok();

        Self {
            opacity: Animated::new(1.0),
            current_frame: Mutex::new(None),
            decoder,
            render_mode: mode,
            temp_file: temp_arc,
        }
    }
}

impl Element for VideoNode {
    fn layout_style(&self) -> Style {
        Style::DEFAULT
    }

    fn update(&mut self, time: f64) -> bool {
        self.opacity.update(time);

        if let Some(decoder) = &self.decoder {
            decoder.send_request(time);

            if let Some(resp) = decoder.get_response() {
                 match resp {
                     VideoResponse::Frame(t, data, w, h) => {
                         let data = Data::new_copy(&data);
                         let info = skia_safe::ImageInfo::new(
                             (w as i32, h as i32),
                             ColorType::RGBA8888,
                             AlphaType::Unpremul,
                             None
                         );

                         if let Some(img) = skia_safe::images::raster_from_data(&info, data, (w * 4) as usize) {
                              *self.current_frame.lock().unwrap() = Some((t, img));
                         }
                     }
                     VideoResponse::EndOfStream => {
                         if self.render_mode == RenderMode::Export {
                             return false;
                         }
                     }
                     VideoResponse::Error(e) => {
                         if self.render_mode == RenderMode::Export {
                             eprintln!("Video Error: {}", e);
                             return false;
                         }
                     }
                 }
            }
        }
        true
    }

    fn render(&self, canvas: &Canvas, rect: Rect, parent_opacity: f32, draw_children: &mut dyn FnMut(&Canvas)) {
         let op = self.opacity.current_value * parent_opacity;

         let current = self.current_frame.lock().unwrap();
         if let Some((_, img)) = current.as_ref() {
             let paint = Paint::new(Color4f::new(1.0, 1.0, 1.0, op), None);
             canvas.draw_image_rect(img, None, rect, &paint);
         } else {
             let mut p = Paint::new(Color4f::new(0.0, 0.0, 1.0, 1.0), None);
             p.set_alpha_f(op);
             canvas.draw_rect(rect, &p);
         }
         draw_children(canvas);
    }

    fn animate_property(&mut self, property: &str, start: f32, target: f32, duration: f64, easing: &str) {
         let ease_fn = parse_easing(easing);
         if property == "opacity" {
             self.opacity.add_segment(start, target, duration, ease_fn);
         }
    }
}
