use crate::animation::Animated;
use crate::element::Element;
use crate::errors::RenderError;
use crate::node::{build_effect_filter, parse_easing, EffectType};
use crate::types::Color;
use skia_safe::{Canvas, ClipOp, Paint, PaintStyle, RRect, Rect};
use std::any::Any;
use taffy::style::{AlignItems, FlexDirection, JustifyContent, Style};

// --- Box Node ---
/// A fundamental layout and styling block (div-like).
///
/// Supports background color, borders, shadows, and rounded corners.
#[derive(Debug, Clone)]
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
        let mut style = Style::default();
        // Defaults to "Vertical Stack, Centered"
        style.flex_direction = FlexDirection::Column;
        style.align_items = Some(AlignItems::Center);
        style.justify_content = Some(JustifyContent::Center);

        Self {
            style,
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
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

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

    fn render(
        &self,
        canvas: &Canvas,
        rect: Rect,
        opacity: f32,
        draw_children: &mut dyn FnMut(&Canvas),
    ) -> Result<(), RenderError> {
        let local_opacity = self.opacity.current_value * opacity;
        let radius = self.border_radius.current_value;
        let rrect = RRect::new_rect_xy(&rect, radius, radius);

        canvas.save();

        if self.overflow == "hidden" {
            canvas.clip_rrect(rrect, ClipOp::Intersect, true);
        }

        let mut paint = Paint::default();
        paint.set_anti_alias(true);

        let mut effects = Vec::new();
        if self.blur.current_value > 0.0 {
            effects.push(EffectType::Blur(self.blur.clone()));
        }
        if let Some(sc) = &self.shadow_color {
            effects.push(EffectType::DropShadow {
                blur: self.shadow_blur.clone(),
                offset_x: self.shadow_offset_x.clone(),
                offset_y: self.shadow_offset_y.clone(),
                color: sc.clone(),
            });
        }

        // BoxNode effects don't use RuntimeShader for now, so we pass dummy resolution/time
        // Or we could pass proper ones if we wanted to support shaders on BoxNode later.
        // For now, these effects (Blur, DropShadow) ignore resolution/time.
        let filter = build_effect_filter(&effects, None, (rect.width(), rect.height()), 0.0);
        if let Some(f) = filter {
            paint.set_image_filter(f);
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
        let ease_fn = parse_easing(easing);
        match property {
            "opacity" => self.opacity.add_segment(start, target, duration, ease_fn),
            "blur" => self.blur.add_segment(start, target, duration, ease_fn),
            "shadow_blur" => self
                .shadow_blur
                .add_segment(start, target, duration, ease_fn),
            "shadow_x" => self
                .shadow_offset_x
                .add_segment(start, target, duration, ease_fn),
            "shadow_y" => self
                .shadow_offset_y
                .add_segment(start, target, duration, ease_fn),
            "border_radius" => self
                .border_radius
                .add_segment(start, target, duration, ease_fn),
            "border_width" => self
                .border_width
                .add_segment(start, target, duration, ease_fn),
            _ => {}
        }
    }

    fn animate_property_spring(
        &mut self,
        property: &str,
        start: Option<f32>,
        target: f32,
        config: crate::animation::SpringConfig,
    ) {
        let apply = |anim: &mut crate::animation::Animated<f32>| {
            if let Some(s) = start {
                anim.add_spring_with_start(s, target, config);
            } else {
                anim.add_spring(target, config);
            }
        };

        match property {
            "opacity" => apply(&mut self.opacity),
            "blur" => apply(&mut self.blur),
            "shadow_blur" => apply(&mut self.shadow_blur),
            "shadow_x" => apply(&mut self.shadow_offset_x),
            "shadow_y" => apply(&mut self.shadow_offset_y),
            "border_radius" => apply(&mut self.border_radius),
            "border_width" => apply(&mut self.border_width),
            _ => {}
        }
    }
}
