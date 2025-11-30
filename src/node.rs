use skia_safe::{Canvas, Paint, Rect, Image, Color4f, Data, TextBlob, FontMgr, FontStyle, ColorType, AlphaType};
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
        self.opacity.update(time);
        self.blur.update(time);
        self.shadow_blur.update(time);
        self.shadow_offset_x.update(time);
        self.shadow_offset_y.update(time);
        changed
    }

    fn render(&self, canvas: &Canvas, rect: Rect, opacity: f32) {
        let local_opacity = self.opacity.current_value * opacity;

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
            canvas.draw_rect(rect, &paint);
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

    fn render(&self, canvas: &Canvas, rect: Rect, _opacity: f32) {
        let mut buf_guard = self.buffer.lock().unwrap();
        let _sc_guard = self.swash_cache.lock().unwrap();

        if let Some(buffer) = buf_guard.as_mut() {
            let mut fs = self.font_system.lock().unwrap();

            buffer.set_size(&mut fs, Some(rect.width()), Some(rect.height()));
            buffer.shape_until_scroll(&mut fs, false);

            let font_mgr = FontMgr::default();
            let mut paint = Paint::default();
            paint.set_anti_alias(true);

             let typeface = font_mgr.match_family_style("Sans Serif", FontStyle::normal()).unwrap();
             let font = skia_safe::Font::new(typeface, Some(self.default_font_size.current_value));
             paint.set_color4f(self.default_color.current_value.to_color4f(), None);

             for run in buffer.layout_runs() {
                 let origin_y = rect.top + run.line_y;
                 if let Some(blob) = TextBlob::from_str(run.text, &font) {
                      canvas.draw_text_blob(&blob, (rect.left, origin_y), &paint);
                 }
             }
        }
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

    fn render(&self, canvas: &Canvas, rect: Rect, parent_opacity: f32) {
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

    fn render(&self, canvas: &Canvas, rect: Rect, parent_opacity: f32) {
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
    }

    fn animate_property(&mut self, property: &str, start: f32, target: f32, duration: f64, easing: &str) {
         let ease_fn = parse_easing(easing);
         if property == "opacity" {
             self.opacity.add_segment(start, target, duration, ease_fn);
         }
    }
}
