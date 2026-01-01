//! # Effects API
//!
//! Visual effects functions for Rhai scripts.
//!
//! ## Responsibilities
//! - **Color Effects**: `apply_effect` for grayscale, sepia, invert, contrast, brightness
//! - **Blur Effect**: `apply_effect("blur", value)` for Gaussian blur
//! - **Custom Shaders**: `apply_effect("shader", {...})` for SkSL shaders

use crate::animation::{Animated, TweenableVector};
use crate::node::{EffectType, ShaderUniform};
use rhai::{Engine, Map};
use std::collections::HashMap;

use super::super::types::NodeHandle;
use super::super::utils::apply_effect_to_node;

/// Register effect-related Rhai functions.
pub fn register(engine: &mut Engine) {
    // Named effects (no value)
    engine.register_fn(
        "apply_effect",
        |node: &mut NodeHandle, name: &str| -> NodeHandle {
            let mut d = node.director.lock().unwrap();
            let effect = match name {
                "grayscale" => Some(EffectType::ColorMatrix(vec![
                    0.2126, 0.7152, 0.0722, 0.0, 0.0, 0.2126, 0.7152, 0.0722, 0.0, 0.0, 0.2126,
                    0.7152, 0.0722, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0,
                ])),
                "sepia" => Some(EffectType::ColorMatrix(vec![
                    0.393, 0.769, 0.189, 0.0, 0.0, 0.349, 0.686, 0.168, 0.0, 0.0, 0.272, 0.534,
                    0.131, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0,
                ])),
                "invert" => Some(EffectType::ColorMatrix(vec![
                    -1.0, 0.0, 0.0, 0.0, 1.0, 0.0, -1.0, 0.0, 0.0, 1.0, 0.0, 0.0, -1.0, 0.0, 1.0,
                    0.0, 0.0, 0.0, 1.0, 0.0,
                ])),
                _ => None,
            };

            if let Some(eff) = effect {
                let id = apply_effect_to_node(&mut d, node.id, eff);
                NodeHandle {
                    director: node.director.clone(),
                    id,
                }
            } else {
                NodeHandle {
                    director: node.director.clone(),
                    id: node.id,
                }
            }
        },
    );

    // Effects with a value parameter
    engine.register_fn(
        "apply_effect",
        |node: &mut NodeHandle, name: &str, val: f64| -> NodeHandle {
            let mut d = node.director.lock().unwrap();
            let val = val as f32;
            let effect = match name {
                "contrast" => {
                    let t = (1.0 - val) / 2.0;
                    Some(EffectType::ColorMatrix(vec![
                        val, 0.0, 0.0, 0.0, t, 0.0, val, 0.0, 0.0, t, 0.0, 0.0, val, 0.0, t, 0.0,
                        0.0, 0.0, 1.0, 0.0,
                    ]))
                }
                "brightness" => Some(EffectType::ColorMatrix(vec![
                    1.0, 0.0, 0.0, 0.0, val, 0.0, 1.0, 0.0, 0.0, val, 0.0, 0.0, 1.0, 0.0, val, 0.0,
                    0.0, 0.0, 1.0, 0.0,
                ])),
                "blur" => Some(EffectType::Blur(Animated::new(val))),
                _ => None,
            };

            if let Some(eff) = effect {
                let id = apply_effect_to_node(&mut d, node.id, eff);
                NodeHandle {
                    director: node.director.clone(),
                    id,
                }
            } else {
                NodeHandle {
                    director: node.director.clone(),
                    id: node.id,
                }
            }
        },
    );

    // Custom shader effects
    engine.register_fn(
        "apply_effect",
        |node: &mut NodeHandle, name: &str, map: rhai::Map| -> NodeHandle {
            let mut d = node.director.lock().unwrap();
            if name == "shader" {
                if let Some(code) = map.get("code").and_then(|v| v.clone().into_string().ok()) {
                    let mut uniforms = HashMap::new();
                    if let Some(u_map) = map
                        .get("uniforms")
                        .and_then(|v| v.clone().try_cast::<Map>())
                    {
                        for (k, v) in u_map {
                            if let Ok(f) = v.as_float() {
                                uniforms.insert(
                                    k.to_string(),
                                    ShaderUniform::Float(Animated::new(f as f32)),
                                );
                            } else if let Ok(arr) = v.clone().into_array() {
                                // Handle array -> Vec<f32>
                                let mut vec_data = Vec::new();
                                for item in arr {
                                    if let Ok(f) = item.as_float() {
                                        vec_data.push(f as f32);
                                    }
                                }
                                if !vec_data.is_empty() {
                                    uniforms.insert(
                                        k.to_string(),
                                        ShaderUniform::Vec(Animated::new(TweenableVector(
                                            vec_data,
                                        ))),
                                    );
                                }
                            }
                        }
                    }

                    let effect = EffectType::RuntimeShader {
                        sksl: code,
                        uniforms,
                    };
                    let id = apply_effect_to_node(&mut d, node.id, effect);
                    return NodeHandle {
                        director: node.director.clone(),
                        id,
                    };
                }
            }
            NodeHandle {
                director: node.director.clone(),
                id: node.id,
            }
        },
    );
}
