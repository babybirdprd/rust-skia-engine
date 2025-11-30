use skia_safe::{
    Canvas, Paint, Rect, RRect, ClipOp, PaintStyle, Image, Color4f, Data, TextBlob, FontMgr,
    FontStyle, ColorType, AlphaType, Path, Point, gradient_shader, TileMode,
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
use crossbeam_channel::{bounded, unbounded, Receiver, Sender, TryRecvError};
use std::thread;
// Video imports
use crate::video_wrapper::Decoder;

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
pub struct TextNode {
    pub spans: Vec<TextSpan>,
    pub default_font_size: Animated<f32>,
    pub default_color: Animated<Color>,
    pub buffer: Mutex<Option<Buffer>>,
    pub font_system: Arc<Mutex<FontSystem>>,
    pub swash_cache: Arc<Mutex<SwashCache>>,
}

impl fmt::Debug for TextNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TextNode")
         .field("spans", &self.spans)
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

        let size = self.default_font_size.current_value;
        let line_height = size * 1.2;

        let mut buf_guard = self.buffer.lock().unwrap();
        if let Some(buffer) = buf_guard.as_mut() {
            let mut fs = self.font_system.lock().unwrap();
            buffer.set_metrics(&mut fs, Metrics::new(size, line_height));
        }
        true
    }

    fn render(&self, canvas: &Canvas, rect: Rect, _opacity: f32, draw_children: &mut dyn FnMut(&Canvas)) {
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

            // Pass 1: Backgrounds
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

            // Pass 2: Text (Strokes & Fills)
            for run in buffer.layout_runs() {
                 let origin_y = rect.top + run.line_y;
                 let mut current_span_idx = None;
                 let mut chunk_start_glyph_idx = 0;

                 // We group glyphs by span index
                 for (i, glyph) in run.glyphs.iter().enumerate() {
                     let glyph_span_idx = ranges.iter().position(|r| r.contains(&glyph.start));

                     if glyph_span_idx != current_span_idx {
                         // Flush previous chunk
                         if let Some(s_idx) = current_span_idx {
                             let span = &self.spans[s_idx];
                             let start_g = &run.glyphs[chunk_start_glyph_idx];
                             // The text slice for this chunk
                             // Note: glyph.start is byte index. We need to find the range covered by [chunk_start..i]
                             // It's safer to reconstruct from glyphs but TextBlob wants str.
                             // We can slice run.text based on glyph start/end bytes?
                             // glyphs[i-1].end should be the end of the last glyph.
                             // run.text is the whole line.
                             // Let's assume glyphs are ordered.
                             let start_byte = start_g.start;
                             let end_byte = run.glyphs[i-1].end; // last glyph in chunk

                             // Find where these bytes are in run.text.
                             // run.text might be a substring of the full text?
                             // buffer.layout_runs() yields runs where run.text is a reference to the line's text.
                             // cosmic-text glyph.start is global byte index.
                             // We need local indices into run.text?
                             // cosmic-text 0.11: run.text is slice.
                             // but we can't easily map global index to local slice index without offsets.
                             // Actually, simple hack:
                             // TextBlob::from_str requires valid UTF-8 slice.
                             // We can use span.text? No, span might be split across lines.
                             // We can use the glyphs to find the text content?
                             // Or just trust that we can draw the whole span text clipped? No.

                             // Better: We know the GLOBAL byte range [start_byte..end_byte].
                             // We can lock the buffer, get the full text, and slice it.
                             // But we can't access full text here easily (it's in buffer but buffer is borrowed as mutable).
                             // Wait, buffer is borrowed. `buffer.lines` has text?

                             // Alternative: run.text DOES correspond to the characters in the run.
                             // Does `run.text` start at global index `run.text_start`? Not available on LayoutRun?
                             // Let's look at `run.glyphs[0].start`.
                             // If `run.text` corresponds to `glyphs`, we can find the sub-slice.
                             // Let's assume `run.text` contains the text for all glyphs in the run.
                             // We can try to find the substring in `run.text` that matches the length?
                             // Or just use `start_byte` and `end_byte` if we knew `run.text`'s global offset.

                             // Let's use `TextBlob::from_str(run.text)` and rely on `canvas.draw_text_blob` clipping? No.
                             // We must separate them for styling.

                             // Re-reading cosmic-text: `LayoutRun` has `text` field.
                             // LayoutRun also has `line_i`.
                             // Buffer has `lines`.
                             // We can assume we can match the glyph's text by length?
                             // glyph.c is the char? No.

                             // Let's try to map indices relative to the Run.
                             // If we track the FIRST glyph of the run, say it has start=100.
                             // If `chunk_start` has start=105.
                             // Then offset is 5.
                             // But bytes vs chars.

                             // Robust fallback:
                             // We only need the text to generate the blob.
                             // We have `span.text`. We know we are in `span`.
                             // But `span` might be wrapped.
                             // Using `run.text` is best.
                             // Let's assume `run.text` covers from `run.glyphs.first().start` to `run.glyphs.last().end`.
                             // Then `offset = glyph.start - run.glyphs[0].start`.

                             if let Some(first_g_in_run) = run.glyphs.first() {
                                 let run_start_global = first_g_in_run.start;
                                 if start_byte >= run_start_global && end_byte >= start_byte {
                                      let local_start = start_byte - run_start_global;
                                      let local_end = end_byte - run_start_global;
                                      if local_end <= run.text.len() {
                                          let chunk_text = &run.text[local_start..local_end];
                                          let x_pos = rect.left + start_g.x;

                                          // Draw this chunk
                                          draw_span_chunk(
                                              canvas, chunk_text, x_pos, origin_y, span, &font,
                                              &self.default_color.current_value
                                          );
                                      }
                                 }
                             }
                         }
                         current_span_idx = glyph_span_idx;
                         chunk_start_glyph_idx = i;
                     }
                 }
                 // Flush last chunk
                 if let Some(s_idx) = current_span_idx {
                     let span = &self.spans[s_idx];
                     if let Some(start_g) = run.glyphs.get(chunk_start_glyph_idx) {
                          let start_byte = start_g.start;
                          let end_byte = run.glyphs.last().unwrap().end;
                          if let Some(first_g_in_run) = run.glyphs.first() {
                               let run_start_global = first_g_in_run.start;
                               if start_byte >= run_start_global && end_byte >= start_byte {
                                    let local_start = start_byte - run_start_global;
                                    let local_end = end_byte - run_start_global;
                                    if local_end <= run.text.len() {
                                        let chunk_text = &run.text[local_start..local_end];
                                        let x_pos = rect.left + start_g.x;
                                        draw_span_chunk(
                                            canvas, chunk_text, x_pos, origin_y, span, &font,
                                            &self.default_color.current_value
                                        );
                                    }
                               }
                          }
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
}

// Helper for drawing text chunks
fn draw_span_chunk(
    canvas: &Canvas,
    text: &str,
    x: f32,
    y: f32,
    span: &TextSpan,
    default_font: &skia_safe::Font,
    default_color: &Color
) {
    // Resolve Font
    let size = span.font_size.unwrap_or(default_font.size());
    let mut typeface = default_font.typeface();
    // If typeface() returned Option in other versions, we'd handle it, but here it seems to be Typeface.
    // If it's actually Option and I'm misreading the error chain, this might fail differently.
    // But the previous error `no method unwrap_or_else found for struct RCHandle` is definitive.

    if span.font_weight.is_some() || span.font_style.is_some() || span.font_family.is_some() {
         let mgr = FontMgr::default();
         let family = span.font_family.as_deref().unwrap_or("Sans Serif");

         let weight = span.font_weight.map(|w| SkWeight::from(w as i32)).unwrap_or(SkWeight::NORMAL);
         let slant = if span.font_style.as_deref() == Some("italic") {
             SkSlant::Italic
         } else {
             SkSlant::Upright
         };

         let style = FontStyle::new(weight, SkWidth::NORMAL, slant);
         if let Some(tf) = mgr.match_family_style(family, style) {
             typeface = tf;
         }
    }

    let font = skia_safe::Font::new(typeface, Some(size));

    if let Some(blob) = TextBlob::from_str(text, &font) {
        // 1. Stroke
        if let Some(sw) = span.stroke_width {
            if sw > 0.0 {
                let mut p = Paint::default();
                p.set_anti_alias(true);
                p.set_style(PaintStyle::Stroke);
                p.set_stroke_width(sw);
                if let Some(c) = span.stroke_color {
                    p.set_color4f(c.to_color4f(), None);
                } else {
                    p.set_color(skia_safe::Color::BLACK);
                }
                canvas.draw_text_blob(&blob, (x, y), &p);
            }
        }

        // 2. Fill (Gradient or Color)
        let mut p = Paint::default();
        p.set_anti_alias(true);
        p.set_style(PaintStyle::Fill);

        if let Some(grad) = &span.fill_gradient {
             let bounds = blob.bounds();
             // bounds is relative to (0,0) of the blob.
             // We need relative coordinates based on the text size/bounds.
             // Usually for text gradients, we map relative to the line height or the specific blob bounds?
             // RFC says "Relative (0.0 to 1.0) is relative to the text bounding box".
             // We can use `bounds` offset by (x, y).

             let w = bounds.width();
             let h = bounds.height();
             // Actual bounds of the text rendered
             let origin_rect = Rect::from_xywh(x + bounds.left, y + bounds.top, w, h);

             let p0 = Point::new(
                 origin_rect.left + grad.start.0 * origin_rect.width(),
                 origin_rect.top + grad.start.1 * origin_rect.height()
             );
             let p1 = Point::new(
                 origin_rect.left + grad.end.0 * origin_rect.width(),
                 origin_rect.top + grad.end.1 * origin_rect.height()
             );

             let colors: Vec<skia_safe::Color> = grad.colors.iter().map(|c| c.to_skia()).collect();

             // Convert positions to Option slice
             let positions = grad.positions.as_ref().map(|v| v.as_slice());

             if let Some(shader) = gradient_shader::linear(
                 (p0, p1),
                 colors.as_slice(),
                 positions,
                 TileMode::Clamp,
                 None,
                 None
             ) {
                 p.set_shader(shader);
             }
        } else {
             // Solid Color
             let c = span.color.unwrap_or(*default_color);
             p.set_color4f(c.to_color4f(), None);
        }

        canvas.draw_text_blob(&blob, (x, y), &p);
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

    // Threading
    frame_receiver: Receiver<(f64, Image)>,
    control_sender: Sender<f64>,
    // Keep temp file alive
    #[allow(dead_code)]
    temp_file: Arc<NamedTempFile>,
}

impl VideoNode {
    pub fn new(data: Vec<u8>) -> Self {
        // Write data to temp file
        let mut temp = NamedTempFile::new().expect("Failed to create temp file");
        temp.write_all(&data).expect("Failed to write video data");
        let path = temp.path().to_owned();
        let temp_arc = Arc::new(temp);

        let (frame_tx, frame_rx) = bounded(5);
        let (ctrl_tx, ctrl_rx) = unbounded();

        let temp_clone = temp_arc.clone();

        thread::spawn(move || {
            let _keep_alive = temp_clone;
            if let Ok(mut decoder) = Decoder::new(&*path) {
                // Initial decode at 0
                let mut current_decoder_time: f64 = 0.0;

                loop {
                    // Check for seek/update
                    // We drain the control channel to get the latest requested time
                    let mut target_time = None;
                    while let Ok(t) = ctrl_rx.try_recv() {
                        target_time = Some(t);
                    }

                    if let Some(t) = target_time {
                        // If we are far off, seek
                        let diff: f64 = t - current_decoder_time;
                        if diff.abs() > 0.5 {
                             let ms = (t * 1000.0) as i64;
                             if decoder.seek(ms).is_ok() {
                                 current_decoder_time = t;
                             }
                        }
                    }

                    // Decode next frame
                    // We try to stay ahead of current_decoder_time
                    match decoder.decode() {
                        Ok((_, frame)) => {
                             // Convert to Skia Image
                             let shape = frame.shape();
                             if shape.len() == 3 && shape[2] >= 3 {
                                 let h = shape[0];
                                 let w = shape[1];
                                 let (bytes, _) = frame.into_raw_vec_and_offset();
                                 let data = Data::new_copy(&bytes);

                                 // Assuming RGB888 for now (3 bytes)
                                 let info = skia_safe::ImageInfo::new(
                                     (w as i32, h as i32),
                                     ColorType::RGB888x,
                                     AlphaType::Opaque,
                                     None
                                 );

                                 if let Some(img) = skia_safe::images::raster_from_data(&info, data, w * 3) {
                                      // Send to main thread
                                      if frame_tx.send((current_decoder_time, img)).is_err() {
                                          break; // Channel closed
                                      }
                                 }
                             }
                             // Advance time guess (assuming 30fps if we don't know)
                             // Ideally we get duration from decode result.
                             current_decoder_time += 1.0 / 30.0;
                        }
                        Err(_) => {
                            // End of stream or error, wait a bit
                            thread::sleep(std::time::Duration::from_millis(100));
                        }
                    }
                }
            }
        });

        Self {
            opacity: Animated::new(1.0),
            current_frame: Mutex::new(None),
            frame_receiver: frame_rx,
            control_sender: ctrl_tx,
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

        // Notify thread of current time
        let _ = self.control_sender.send(time);

        // Check if we have a frame in the queue that matches (or is close)
        // Or if we need to wait?
        // Non-blocking check
        loop {
            match self.frame_receiver.try_recv() {
                Ok((t, img)) => {
                    if t >= time - 0.1 {
                        // Found a usable frame
                        *self.current_frame.lock().unwrap() = Some((t, img));
                        // If it's the exact one or future, stop.
                        // If it's slightly past, we still use it (closest next).
                        break;
                    }
                    // If t < time - 0.1, it's too old, discard and loop
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => break,
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
