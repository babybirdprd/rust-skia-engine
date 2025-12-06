use crate::element::Element;
use skia_safe::{Canvas, Rect};
use taffy::style::Style;
use std::sync::{Arc, Mutex};
use std::any::Any;
use lottie_core::LottiePlayer;
use lottie_data::model::LottieJson;
use lottie_skia::SkiaRenderer;
use crate::animation::{Animated, EasingType};

pub struct LottieNode {
    model: Arc<LottieJson>,
    player: Mutex<LottiePlayer>,
    pub style: Style,
    pub opacity: Animated<f32>,
    pub frame: Animated<f32>,
    pub speed: f32,
    pub loop_anim: bool,
}

impl std::fmt::Debug for LottieNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LottieNode")
         .field("style", &self.style)
         .field("opacity", &self.opacity)
         .field("frame", &self.frame)
         .field("speed", &self.speed)
         .field("loop_anim", &self.loop_anim)
         .finish()
    }
}

impl Clone for LottieNode {
    fn clone(&self) -> Self {
        let mut player = LottiePlayer::new();
        player.load((*self.model).clone());

        Self {
            model: self.model.clone(),
            player: Mutex::new(player),
            style: self.style.clone(),
            opacity: self.opacity.clone(),
            frame: self.frame.clone(),
            speed: self.speed,
            loop_anim: self.loop_anim,
        }
    }
}

impl LottieNode {
    pub fn new(data: &[u8]) -> anyhow::Result<Self> {
        let json_str = std::str::from_utf8(data)?;
        let model: LottieJson = serde_json::from_str(json_str)?;
        let mut player = LottiePlayer::new();
        player.load(model.clone());

        Ok(Self {
            model: Arc::new(model),
            player: Mutex::new(player),
            style: Style::DEFAULT,
            opacity: Animated::new(1.0),
            frame: Animated::new(0.0),
            speed: 1.0,
            loop_anim: false,
        })
    }
}

impl Element for LottieNode {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }

    fn layout_style(&self) -> Style { self.style.clone() }
    fn set_layout_style(&mut self, style: Style) { self.style = style; }

    fn update(&mut self, time: f64) -> bool {
        self.opacity.update(time);
        self.frame.update(time);

        let mut player = self.player.lock().unwrap();

        // If frame property has keyframes (more than the initial one), use it.
        if self.frame.raw_keyframes.len() > 1 {
             player.current_frame = self.frame.current_value;
        } else {
             let fps = self.model.fr;
             let start_frame = self.model.ip;
             let end_frame = self.model.op;

             // Check if duration is valid
             let total_frames = end_frame - start_frame;

             let current_raw = time * fps as f64 * self.speed as f64;

             if self.loop_anim && total_frames > 0.0 {
                 let looped = (current_raw % total_frames as f64) + start_frame as f64;
                 player.current_frame = looped as f32;
             } else {
                 let frame = start_frame as f64 + current_raw;
                 // Clamp to end
                 if frame > end_frame as f64 {
                     player.current_frame = end_frame;
                 } else {
                     player.current_frame = frame as f32;
                 }
             }
        }

        true
    }

    fn render(&self, canvas: &Canvas, rect: Rect, parent_opacity: f32, _draw_children: &mut dyn FnMut(&Canvas)) {
        let mut player = self.player.lock().unwrap();
        let tree = player.render_tree();
        let final_opacity = self.opacity.current_value * parent_opacity;

        SkiaRenderer::draw(canvas, &tree, rect, final_opacity, &());
    }

    fn animate_property(&mut self, property: &str, start: f32, target: f32, duration: f64, easing: &str) {
        let ease = match easing {
            "linear" => EasingType::Linear,
            "ease_in" => EasingType::EaseIn,
            "ease_out" => EasingType::EaseOut,
            "ease_in_out" => EasingType::EaseInOut,
             _ => EasingType::Linear,
        };

        if property == "opacity" {
            self.opacity.add_segment(start, target, duration, ease);
        } else if property == "frame" {
            self.frame.add_segment(start, target, duration, ease);
        }
    }
}
