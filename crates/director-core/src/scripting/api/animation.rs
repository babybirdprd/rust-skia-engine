//! # Animation API
//!
//! Animation functions for Rhai scripts.
//!
//! ## Responsibilities
//! - **Keyframe Animation**: `animate` for property tweening
//! - **Spring Animation**: `spring` for physics-based motion
//! - **Path Animation**: `path_animate` for SVG path following
//! - **Text Animation**: `add_animator` for per-glyph animations
//! - **Instant Setters**: `set_blur` for immediate property changes

use crate::animation::Animated;
use crate::types::PathAnimationState;
use rhai::Engine;
use skia_safe::Path;
use tracing::error;

use super::super::types::NodeHandle;
use super::super::utils::{parse_easing, parse_spring_config};

/// Register animation-related Rhai functions.
pub fn register(engine: &mut Engine) {
    // ========== ANIMATE ==========
    engine.register_fn(
        "animate",
        |node: &mut NodeHandle, prop: &str, start: f64, end: f64, dur: f64, ease: &str| {
            let mut d = node.director.lock().unwrap();
            if let Some(n) = d.scene.get_node_mut(node.id) {
                let ease_fn = parse_easing(ease);

                match prop {
                    "scale" => {
                        n.transform
                            .scale_x
                            .add_segment(start as f32, end as f32, dur, ease_fn);
                        n.transform
                            .scale_y
                            .add_segment(start as f32, end as f32, dur, ease_fn);
                    }
                    "scale_x" => {
                        n.transform
                            .scale_x
                            .add_segment(start as f32, end as f32, dur, ease_fn)
                    }
                    "scale_y" => {
                        n.transform
                            .scale_y
                            .add_segment(start as f32, end as f32, dur, ease_fn)
                    }
                    "rotation" => {
                        n.transform
                            .rotation
                            .add_segment(start as f32, end as f32, dur, ease_fn)
                    }
                    "skew_x" => {
                        n.transform
                            .skew_x
                            .add_segment(start as f32, end as f32, dur, ease_fn)
                    }
                    "skew_y" => {
                        n.transform
                            .skew_y
                            .add_segment(start as f32, end as f32, dur, ease_fn)
                    }
                    "translate_x" | "x" => {
                        n.transform
                            .translate_x
                            .add_segment(start as f32, end as f32, dur, ease_fn)
                    }
                    "translate_y" | "y" => {
                        n.transform
                            .translate_y
                            .add_segment(start as f32, end as f32, dur, ease_fn)
                    }
                    _ => {
                        n.element
                            .animate_property(prop, start as f32, end as f32, dur, ease);
                    }
                }
            }
        },
    );

    // Integer Overload for animate
    engine.register_fn(
        "animate",
        |node: &mut NodeHandle, prop: &str, start: i64, end: i64, dur: f64, ease: &str| {
            let mut d = node.director.lock().unwrap();
            if let Some(n) = d.scene.get_node_mut(node.id) {
                let ease_fn = parse_easing(ease);

                match prop {
                    "scale" => {
                        n.transform
                            .scale_x
                            .add_segment(start as f32, end as f32, dur, ease_fn);
                        n.transform
                            .scale_y
                            .add_segment(start as f32, end as f32, dur, ease_fn);
                    }
                    "scale_x" => {
                        n.transform
                            .scale_x
                            .add_segment(start as f32, end as f32, dur, ease_fn)
                    }
                    "scale_y" => {
                        n.transform
                            .scale_y
                            .add_segment(start as f32, end as f32, dur, ease_fn)
                    }
                    "rotation" => {
                        n.transform
                            .rotation
                            .add_segment(start as f32, end as f32, dur, ease_fn)
                    }
                    "skew_x" => {
                        n.transform
                            .skew_x
                            .add_segment(start as f32, end as f32, dur, ease_fn)
                    }
                    "skew_y" => {
                        n.transform
                            .skew_y
                            .add_segment(start as f32, end as f32, dur, ease_fn)
                    }
                    "translate_x" | "x" => {
                        n.transform
                            .translate_x
                            .add_segment(start as f32, end as f32, dur, ease_fn)
                    }
                    "translate_y" | "y" => {
                        n.transform
                            .translate_y
                            .add_segment(start as f32, end as f32, dur, ease_fn)
                    }
                    _ => {
                        n.element
                            .animate_property(prop, start as f32, end as f32, dur, ease);
                    }
                }
            }
        },
    );

    // Spring animation via animate (shorthand: animate(prop, end, config))
    engine.register_fn(
        "animate",
        |node: &mut NodeHandle, prop: &str, end: f64, config: rhai::Map| {
            let mut d = node.director.lock().unwrap();
            let spring_conf = parse_spring_config(&config);

            if let Some(n) = d.scene.get_node_mut(node.id) {
                match prop {
                    "scale" => {
                        n.transform
                            .scale_x
                            .add_spring(end as f32, spring_conf.clone());
                        n.transform.scale_y.add_spring(end as f32, spring_conf);
                    }
                    "scale_x" => n.transform.scale_x.add_spring(end as f32, spring_conf),
                    "scale_y" => n.transform.scale_y.add_spring(end as f32, spring_conf),
                    "rotation" => n.transform.rotation.add_spring(end as f32, spring_conf),
                    "skew_x" => n.transform.skew_x.add_spring(end as f32, spring_conf),
                    "skew_y" => n.transform.skew_y.add_spring(end as f32, spring_conf),
                    "translate_x" | "x" => {
                        n.transform.translate_x.add_spring(end as f32, spring_conf)
                    }
                    "translate_y" | "y" => {
                        n.transform.translate_y.add_spring(end as f32, spring_conf)
                    }
                    _ => {
                        n.element
                            .animate_property_spring(prop, None, end as f32, spring_conf);
                    }
                }
            }
        },
    );

    // ========== SPRING ==========
    engine.register_fn(
        "spring",
        |node: &mut NodeHandle, prop: &str, start: f64, end: f64, config: rhai::Map| {
            let mut d = node.director.lock().unwrap();
            let spring_conf = parse_spring_config(&config);

            if let Some(n) = d.scene.get_node_mut(node.id) {
                match prop {
                    "scale" => {
                        n.transform.scale_x.add_spring_with_start(
                            start as f32,
                            end as f32,
                            spring_conf.clone(),
                        );
                        n.transform.scale_y.add_spring_with_start(
                            start as f32,
                            end as f32,
                            spring_conf,
                        );
                    }
                    "scale_x" => n.transform.scale_x.add_spring_with_start(
                        start as f32,
                        end as f32,
                        spring_conf,
                    ),
                    "scale_y" => n.transform.scale_y.add_spring_with_start(
                        start as f32,
                        end as f32,
                        spring_conf,
                    ),
                    "rotation" => n.transform.rotation.add_spring_with_start(
                        start as f32,
                        end as f32,
                        spring_conf,
                    ),
                    "skew_x" => n.transform.skew_x.add_spring_with_start(
                        start as f32,
                        end as f32,
                        spring_conf,
                    ),
                    "skew_y" => n.transform.skew_y.add_spring_with_start(
                        start as f32,
                        end as f32,
                        spring_conf,
                    ),
                    "translate_x" | "x" => n.transform.translate_x.add_spring_with_start(
                        start as f32,
                        end as f32,
                        spring_conf,
                    ),
                    "translate_y" | "y" => n.transform.translate_y.add_spring_with_start(
                        start as f32,
                        end as f32,
                        spring_conf,
                    ),
                    _ => {
                        n.element.animate_property_spring(
                            prop,
                            Some(start as f32),
                            end as f32,
                            spring_conf,
                        );
                    }
                }
            }
        },
    );

    // ========== PATH_ANIMATE ==========
    engine.register_fn(
        "path_animate",
        |node: &mut NodeHandle, svg: &str, dur: f64, ease: &str| {
            let mut d = node.director.lock().unwrap();
            if let Some(n) = d.scene.get_node_mut(node.id) {
                // Try to parse SVG path
                if let Some(path) = Path::from_svg(svg) {
                    let ease_fn = parse_easing(ease);
                    let mut progress = Animated::new(0.0);
                    progress.add_keyframe(1.0, dur, ease_fn);

                    n.path_animation = Some(PathAnimationState { path, progress });
                } else {
                    error!("Failed to parse SVG path: {}", svg);
                }
            }
        },
    );

    // ========== SET_BLUR ==========
    engine.register_fn("set_blur", |node: &mut NodeHandle, val: f64| {
        let mut d = node.director.lock().unwrap();
        if let Some(n) = d.scene.get_node_mut(node.id) {
            n.element
                .animate_property("blur", val as f32, val as f32, 0.0, "linear");
        }
    });

    // ========== ADD_ANIMATOR (Text Animation) ==========
    engine.register_fn(
        "add_animator",
        |node: &mut NodeHandle,
         start_idx: i64,
         end_idx: i64,
         prop: &str,
         start: f64,
         end: f64,
         dur: f64,
         ease: &str| {
            let mut d = node.director.lock().unwrap();
            if let Some(n) = d.scene.get_node_mut(node.id) {
                n.element.add_text_animator(
                    start_idx as usize,
                    end_idx as usize,
                    prop.to_string(),
                    start as f32,
                    end as f32,
                    dur,
                    ease,
                );
            }
        },
    );

    // Integer Overload for add_animator
    engine.register_fn(
        "add_animator",
        |node: &mut NodeHandle,
         start_idx: i64,
         end_idx: i64,
         prop: &str,
         start: i64,
         end: i64,
         dur: f64,
         ease: &str| {
            let mut d = node.director.lock().unwrap();
            if let Some(n) = d.scene.get_node_mut(node.id) {
                n.element.add_text_animator(
                    start_idx as usize,
                    end_idx as usize,
                    prop.to_string(),
                    start as f32,
                    end as f32,
                    dur,
                    ease,
                );
            }
        },
    );

    // With stagger
    engine.register_fn(
        "add_animator",
        |node: &mut NodeHandle,
         start_idx: i64,
         end_idx: i64,
         prop: &str,
         start: f64,
         end: f64,
         dur: f64,
         ease: &str,
         stagger: f64| {
            let mut d = node.director.lock().unwrap();
            if let Some(n) = d.scene.get_node_mut(node.id) {
                n.element.add_text_animator_full(
                    start_idx as usize,
                    end_idx as usize,
                    prop.to_string(),
                    start as f32,
                    end as f32,
                    dur,
                    ease,
                    stagger as f32,
                );
            }
        },
    );

    // Integer overload with stagger
    engine.register_fn(
        "add_animator",
        |node: &mut NodeHandle,
         start_idx: i64,
         end_idx: i64,
         prop: &str,
         start: i64,
         end: i64,
         dur: f64,
         ease: &str,
         stagger: f64| {
            let mut d = node.director.lock().unwrap();
            if let Some(n) = d.scene.get_node_mut(node.id) {
                n.element.add_text_animator_full(
                    start_idx as usize,
                    end_idx as usize,
                    prop.to_string(),
                    start as f32,
                    end as f32,
                    dur,
                    ease,
                    stagger as f32,
                );
            }
        },
    );
}
