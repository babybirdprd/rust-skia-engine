use skia_safe::{Canvas, Rect, Size, Paint, Color};
use crate::element::Element;
use crate::animation::{Animated, EasingType, SpringConfig};
use std::any::Any;
use std::sync::Mutex;
use taffy::style::Style;

// Helper to parse easing (duplicated from node.rs)
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

#[derive(Clone, Debug, PartialEq)]
pub enum LoopMode {
    Loop,
    PlayOnce,
    PingPong,
}

// Mock Skottie Animation
#[derive(Clone, Debug)]
pub struct Animation {
    duration: f64,
    size: Size,
}

impl Animation {
    pub fn from_data(_data: &[u8]) -> Option<Self> {
        // Mock implementation
        Some(Self {
            duration: 5.0,
            size: Size::new(100.0, 100.0),
        })
    }

    pub fn duration(&self) -> f64 {
        self.duration
    }

    pub fn size(&self) -> &Size {
        &self.size
    }

    pub fn seek_frame_time(&mut self, _t: f32) {
        // Mock seek
    }

    pub fn render(&self, canvas: &Canvas, dst: &Rect) {
        let mut paint = Paint::default();
        paint.set_color(Color::from_rgb(255, 0, 255)); // Magenta placeholder
        canvas.draw_rect(dst, &paint);
    }
}

#[derive(Debug)]
pub struct LottieNode {
    pub animation: Mutex<Animation>,
    pub duration: f64,
    pub size: Size,
    pub loop_mode: LoopMode,
    pub speed: Animated<f32>,
    pub seek_offset: Animated<f32>,
    pub style: Style,

    // Internal state
    current_time: f64,
}

impl Clone for LottieNode {
    fn clone(&self) -> Self {
        let anim = self.animation.lock().unwrap().clone();
        Self {
            animation: Mutex::new(anim),
            duration: self.duration,
            size: self.size,
            loop_mode: self.loop_mode.clone(),
            speed: self.speed.clone(),
            seek_offset: self.seek_offset.clone(),
            style: self.style.clone(),
            current_time: self.current_time,
        }
    }
}

impl LottieNode {
    pub fn new(data: Vec<u8>) -> Option<Self> {
        let animation = Animation::from_data(&data)?;
        let duration = animation.duration();
        let size = *animation.size();

        Some(Self {
            animation: Mutex::new(animation),
            duration,
            size,
            loop_mode: LoopMode::Loop,
            speed: Animated::new(1.0),
            seek_offset: Animated::new(0.0),
            style: Style::default(),
            current_time: 0.0,
        })
    }
}

impl Element for LottieNode {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }

    fn layout_style(&self) -> Style {
        self.style.clone()
    }

    fn update(&mut self, time: f64) -> bool {
        self.speed.update(time);
        self.seek_offset.update(time);
        self.current_time = time;
        true
    }

    fn render(&self, canvas: &Canvas, rect: Rect, opacity: f32, _draw_children: &mut dyn FnMut(&Canvas)) {
        let mut t = (self.current_time * self.speed.current_value as f64) + self.seek_offset.current_value as f64;
        let dur = self.duration;

        if dur > 0.0 {
            match self.loop_mode {
                LoopMode::Loop => {
                    t = t % dur;
                    if t < 0.0 { t += dur; }
                },
                LoopMode::PlayOnce => {
                    t = t.clamp(0.0, dur);
                },
                LoopMode::PingPong => {
                    let double_dur = dur * 2.0;
                    t = t % double_dur;
                    if t < 0.0 { t += double_dur; }
                    if t > dur {
                        t = double_dur - t;
                    }
                }
            }
        }

        let mut anim = self.animation.lock().unwrap();
        anim.seek_frame_time(t as f32);

        if opacity < 1.0 {
            let mut paint = Paint::default();
            paint.set_alpha_f(opacity);
            canvas.save_layer(&skia_safe::canvas::SaveLayerRec::default().bounds(&rect).paint(&paint));
            anim.render(canvas, &rect);
            canvas.restore();
        } else {
            anim.render(canvas, &rect);
        }
    }

    fn animate_property(&mut self, property: &str, start: f32, target: f32, duration: f64, easing: &str) {
        let ease_fn = parse_easing(easing);
        match property {
            "speed" => self.speed.add_segment(start, target, duration, ease_fn),
            "seek" | "seek_offset" => self.seek_offset.add_segment(start, target, duration, ease_fn),
            _ => {}
        }
    }

    fn animate_property_spring(&mut self, property: &str, start: Option<f32>, target: f32, config: SpringConfig) {
        let apply = |anim: &mut Animated<f32>| {
             if let Some(s) = start {
                 anim.add_spring_with_start(s, target, config);
             } else {
                 anim.add_spring(target, config);
             }
        };

        match property {
            "speed" => apply(&mut self.speed),
            "seek" | "seek_offset" => apply(&mut self.seek_offset),
            _ => {}
        }
    }
}
