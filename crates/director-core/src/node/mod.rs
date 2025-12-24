use crate::animation::EasingType;
use crate::types::ObjectFit;
use skia_safe::Rect;

pub mod text;
pub use text::TextNode;

pub mod vector;
pub use vector::VectorNode;

pub mod lottie;
pub use lottie::LottieNode;

pub mod box_node;
pub use box_node::BoxNode;

pub mod image_node;
pub use image_node::ImageNode;

pub mod video_node;
pub use video_node::{VideoNode, VideoSource};

pub mod composition;
pub use composition::CompositionNode;

pub mod effect;
pub use effect::{build_effect_filter, EffectNode, EffectType, ShaderUniform};

// Helper to parse easing
pub fn parse_easing(e: &str) -> EasingType {
    match e {
        "linear" => EasingType::Linear,
        "ease_in" => EasingType::EaseIn,
        "ease_out" => EasingType::EaseOut,
        "ease_in_out" => EasingType::EaseInOut,
        "bounce_out" => EasingType::BounceOut,
        _ => EasingType::Linear,
    }
}

pub fn calculate_object_fit_rect(src_w: f32, src_h: f32, dst_rect: Rect, fit: ObjectFit) -> Rect {
    match fit {
        ObjectFit::Fill => dst_rect,
        ObjectFit::Contain | ObjectFit::Cover => {
            let src_ratio = src_w / src_h;
            let dst_w = dst_rect.width();
            let dst_h = dst_rect.height();
            let dst_ratio = dst_w / dst_h;

            let scale = match fit {
                ObjectFit::Contain => {
                    if src_ratio > dst_ratio {
                        dst_w / src_w
                    } else {
                        dst_h / src_h
                    }
                }
                ObjectFit::Cover => {
                    if src_ratio > dst_ratio {
                        dst_h / src_h
                    } else {
                        dst_w / src_w
                    }
                }
                _ => 1.0,
            };

            let new_w = src_w * scale;
            let new_h = src_h * scale;
            let new_x = dst_rect.left + (dst_w - new_w) / 2.0;
            let new_y = dst_rect.top + (dst_h - new_h) / 2.0;

            Rect::from_xywh(new_x, new_y, new_w, new_h)
        }
    }
}
