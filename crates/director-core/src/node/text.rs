use crate::animation::Animated;
use crate::element::{Element, TextFit, TextShadow, TextSpan};
use crate::types::Color;
use skia_safe::{
    image_filters, textlayout::{
        FontCollection, Paragraph, ParagraphBuilder, ParagraphStyle, TextStyle, TextAlign,
        TextDirection, TextHeightBehavior, StrutStyle,
    },
    Canvas, Paint, PaintStyle, Rect, TileMode,
};
use std::any::Any;
use std::fmt;
use std::sync::{Arc, Mutex};
use taffy::geometry::Size;
use taffy::style::{AvailableSpace, Style};
use tracing::warn;

/// A node for rendering rich text using Skia's native text layout engine (SkParagraph).
pub struct TextNode {
    pub spans: Vec<TextSpan>,
    pub default_font_size: Animated<f32>,
    pub default_color: Animated<Color>,

    // Skia Text Layout
    pub paragraph: Mutex<Option<Paragraph>>,
    pub font_collection: Arc<Mutex<FontCollection>>,

    // Layout & Styling
    pub fit_mode: TextFit,
    pub min_size: f32,
    pub max_size: f32,
    pub shadow: Option<TextShadow>,
    pub dirty_layout: bool,
    pub last_layout_rect: Rect,
    pub style: Style,
}

impl Clone for TextNode {
    fn clone(&self) -> Self {
        Self {
            spans: self.spans.clone(),
            default_font_size: self.default_font_size.clone(),
            default_color: self.default_color.clone(),
            paragraph: Mutex::new(None), // Must rebuild
            font_collection: self.font_collection.clone(),
            fit_mode: self.fit_mode.clone(),
            min_size: self.min_size,
            max_size: self.max_size,
            shadow: self.shadow.clone(),
            dirty_layout: true,
            last_layout_rect: Rect::default(),
            style: self.style.clone(),
        }
    }
}

impl fmt::Debug for TextNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TextNode")
            .field("spans", &self.spans)
            .field("fit_mode", &self.fit_mode)
            .finish()
    }
}

impl TextNode {
    pub fn new(
        spans: Vec<TextSpan>,
        font_collection: Arc<Mutex<FontCollection>>,
    ) -> Self {
        let mut node = Self {
            spans,
            default_font_size: Animated::new(20.0),
            default_color: Animated::new(Color::WHITE),
            paragraph: Mutex::new(None),
            font_collection,
            fit_mode: TextFit::None,
            min_size: 10.0,
            max_size: 200.0,
            shadow: None,
            dirty_layout: true,
            last_layout_rect: Rect::default(),
            style: Style::default(),
        };
        node.init_paragraph();
        node
    }

    /// Builds the paragraph using current spans and style
    fn build_paragraph(&self) -> Paragraph {
        let mut paragraph_style = ParagraphStyle::new();
        paragraph_style.set_text_align(TextAlign::Left);
        paragraph_style.set_text_direction(TextDirection::LTR);

        // --- Fix for Vertical Centering ---
        // 1. Disable "first ascent" which adds extra padding at the top
        paragraph_style.set_text_height_behavior(TextHeightBehavior::DisableAll);

        // 2. Configure StrutStyle to enforce consistent line heights
        let mut strut_style = StrutStyle::new();
        let default_font_families = vec!["Sans Serif"]; // Fallback

        // We use the default font size for the strut, or a fallback of 20.0 if not set.
        let font_size = self.default_font_size.current_value;

        strut_style.set_font_families(&default_font_families);
        strut_style.set_font_size(font_size);
        strut_style.set_height(1.2); // 1.2 Multiplier for nice leading
        strut_style.set_leading(0.0);
        strut_style.set_strut_enabled(true);
        strut_style.set_force_strut_height(true);
        strut_style.set_height_override(true); // Ensures the multiplier is strictly obeyed

        paragraph_style.set_strut_style(strut_style);
        // ----------------------------------

        let font_collection_guard = self.font_collection.lock().unwrap();
        let mut builder = ParagraphBuilder::new(&paragraph_style, &*font_collection_guard);

        for span in &self.spans {
            let mut text_style = TextStyle::new();

            // 1. Font Size
            let size = span.font_size.unwrap_or(self.default_font_size.current_value);
            text_style.set_font_size(size);

            // 2. Font Family
            let mut families = vec![];
            if let Some(f) = &span.font_family {
                families.push(f.as_str());
            } else {
                families.push("Sans Serif");
                families.push("Arial");
            }
            text_style.set_font_families(&families);

            // 3. Color (Foreground)
            let color = span.color.unwrap_or(self.default_color.current_value);
            let mut paint = Paint::default();
            paint.set_color(color.to_skia());
            paint.set_anti_alias(true);
            text_style.set_foreground_paint(&paint);

            // 4. Background
            if let Some(bg_color) = span.background_color {
                let mut bg_paint = Paint::default();
                bg_paint.set_color(bg_color.to_skia());
                bg_paint.set_anti_alias(true);
                text_style.set_background_paint(&bg_paint);
            }

            // 5. Weight / Slant
            if let Some(w) = span.font_weight {
                let weight = skia_safe::font_style::Weight::from(w as i32);
                let slant = if span.font_style.as_deref() == Some("italic") {
                    skia_safe::font_style::Slant::Italic
                } else {
                    skia_safe::font_style::Slant::Upright
                };
                text_style.set_font_style(skia_safe::FontStyle::new(weight, skia_safe::font_style::Width::NORMAL, slant));
            }

            // 6. Stroke (Rich Text)
            if let Some(sw) = span.stroke_width {
                if sw > 0.0 {
                    let mut stroke_paint = Paint::default();
                    stroke_paint.set_style(PaintStyle::Stroke);
                    stroke_paint.set_stroke_width(sw);
                    stroke_paint.set_anti_alias(true);
                    if let Some(sc) = span.stroke_color {
                        stroke_paint.set_color(sc.to_skia());
                    } else {
                         stroke_paint.set_color(skia_safe::Color::BLACK);
                    }
                     text_style.set_foreground_paint(&stroke_paint);
                }
            }

            builder.push_style(&text_style);
            builder.add_text(&span.text);
            builder.pop();
        }

        builder.build()
    }

    pub fn init_paragraph(&mut self) {
        *self.paragraph.lock().unwrap() = Some(self.build_paragraph());
        self.dirty_layout = true;
    }

    pub fn ensure_paragraph_ready(&self) {
        let p_guard = self.paragraph.lock().unwrap();
        if p_guard.is_none() {
            drop(p_guard);
            let mut p_guard = self.paragraph.lock().unwrap();
            *p_guard = Some(self.build_paragraph());
        }
    }
}

impl Element for TextNode {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn needs_measure(&self) -> bool {
        true
    }

    fn measure(
        &self,
        known_dimensions: Size<Option<f32>>,
        available_space: Size<AvailableSpace>,
    ) -> Size<f32> {
        self.ensure_paragraph_ready();
        let mut p_guard = self.paragraph.lock().unwrap();

        if let Some(paragraph) = p_guard.as_mut() {
            // Determine constraint
            let width_opt = match available_space.width {
                AvailableSpace::Definite(w) => w,
                AvailableSpace::MinContent => 0.0,
                AvailableSpace::MaxContent => f32::INFINITY,
            };

            let layout_width = known_dimensions.width.unwrap_or(width_opt);

            paragraph.layout(layout_width);

            let width = if known_dimensions.width.is_some() {
                layout_width
            } else {
                paragraph.longest_line()
            };

            Size {
                width: width.ceil(),
                height: paragraph.height().ceil(),
            }
        } else {
            Size::ZERO
        }
    }

    fn layout_style(&self) -> Style {
        self.style.clone()
    }

    fn update(&mut self, time: f64) -> bool {
        let old_size = self.default_font_size.current_value;
        let old_color = self.default_color.current_value;

        self.default_font_size.update(time);
        self.default_color.update(time);

        let mut changed = false;

        // Check if visual properties changed
        if (self.default_font_size.current_value - old_size).abs() > 0.001 {
            changed = true;
        }

        // Color::PartialEq is derived, compares float fields exactly.
        if self.default_color.current_value != old_color {
             changed = true;
        }

        if changed {
            // Rebuild paragraph
            self.init_paragraph();
        }

        changed
    }

    fn post_layout(&mut self, rect: Rect) {
        if self.fit_mode == TextFit::None && !self.dirty_layout {
            return;
        }
        if self.last_layout_rect == rect && !self.dirty_layout {
            return;
        }

        self.last_layout_rect = rect;

        if self.fit_mode == TextFit::Shrink {
            let target_width = rect.width();
            let target_height = rect.height();

            let mut low = self.min_size;
            let mut high = self.max_size;
            let mut best_size = self.min_size;

            for _ in 0..5 {
                let mid = (low + high) / 2.0;

                self.default_font_size.current_value = mid;
                self.init_paragraph();

                let mut p_guard = self.paragraph.lock().unwrap();
                if let Some(p) = p_guard.as_mut() {
                    p.layout(target_width);
                    if p.height() <= target_height && p.longest_line() <= target_width + 1.0 {
                        best_size = mid;
                        low = mid;
                    } else {
                        high = mid;
                    }
                }
            }

            self.default_font_size.current_value = best_size;
            self.init_paragraph();
        }

        self.dirty_layout = false;
    }

    fn render(
        &self,
        canvas: &Canvas,
        rect: Rect,
        opacity: f32,
        draw_children: &mut dyn FnMut(&Canvas),
    ) -> Result<(), crate::RenderError> {
        let mut p_guard = self.paragraph.lock().unwrap();
        if let Some(paragraph) = p_guard.as_mut() {
            paragraph.layout(rect.width());

            canvas.save();

            if let Some(shadow) = &self.shadow {
                 let mut shadow_paint = Paint::default();
                 shadow_paint.set_color(shadow.color.to_skia());
                 shadow_paint.set_alpha_f(opacity);
                 shadow_paint.set_image_filter(image_filters::blur(
                    (shadow.blur, shadow.blur),
                    TileMode::Decal,
                    None,
                    None,
                ));

                 // Manual shadow rendering isn't easy with SkParagraph + Paint.
                 // We will skip shadow rendering for this PR as per plan,
                 // or accept it might not work fully without layer.
            }

            if opacity < 1.0 {
                let mut paint = Paint::default();
                paint.set_alpha_f(opacity);
                canvas.save_layer(&skia_safe::canvas::SaveLayerRec::default().bounds(&rect).paint(&paint));
                paragraph.paint(canvas, (rect.x(), rect.y()));
                canvas.restore();
            } else {
                paragraph.paint(canvas, (rect.x(), rect.y()));
            }

            canvas.restore();
        }

        draw_children(canvas);
        Ok(())
    }

    fn animate_property(
        &mut self,
        property: &str,
        start: f32,
        target: f32,
        duration: f64,
        easing: &str,
    ) {
         match property {
            "font_size" | "size" => {
                let ease_fn = crate::node::parse_easing(easing);
                self.default_font_size.add_segment(start, target, duration, ease_fn);
                self.dirty_layout = true;
            }
            _ => {}
        }
    }

    // Stub for animations
    fn add_text_animator(
        &mut self,
        _start_idx: usize,
        _end_idx: usize,
        _property: String,
        _start_val: f32,
        _target_val: f32,
        _duration: f64,
        _easing: &str,
    ) {
        warn!("Text animators are temporarily disabled in the new layout engine.");
    }
}
