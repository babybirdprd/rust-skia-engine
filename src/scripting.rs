use rhai::{Engine, Map};
use crate::director::{Director, NodeId, TimelineItem, PathAnimationState};
use crate::node::{BoxNode, TextNode};
use crate::element::{Color, TextSpan};
use crate::animation::{Animated, EasingType};
use std::sync::{Arc, Mutex};
use cosmic_text::{FontSystem, SwashCache};
use skia_safe::Path;

/// Wrapper around `Director` for Rhai scripting.
#[derive(Clone)]
pub struct MovieHandle {
    pub director: Arc<Mutex<Director>>,
}

/// Handle to a specific Scene (or time segment) in the movie.
#[derive(Clone)]
pub struct SceneHandle {
    pub director: Arc<Mutex<Director>>,
    pub root_id: NodeId,
    pub start_time: f64,
    pub duration: f64,
}

/// Handle to a specific Node in the scene graph.
#[derive(Clone)]
pub struct NodeHandle {
    pub director: Arc<Mutex<Director>>,
    pub id: NodeId,
}

/// Helper to parse hex strings like "#RRGGBB" or "#RGB"
fn parse_hex_color(hex: &str) -> Option<Color> {
    let hex = hex.trim_start_matches('#');
    let (r, g, b) = match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            (r, g, b)
        },
        3 => {
            let r = u8::from_str_radix(&hex[0..1], 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2], 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3], 16).ok()?;
            (r * 17, g * 17, b * 17)
        },
        _ => return None,
    };

    Some(Color::new(
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
        1.0
    ))
}

pub fn register_rhai_api(engine: &mut Engine) {
    // 1. Director/Movie
    engine.register_type_with_name::<MovieHandle>("Movie");
    engine.register_fn("new_director", |w: i64, h: i64, fps: i64| {
        let director = Director::new(w as i32, h as i32, fps as u32);
        MovieHandle { director: Arc::new(Mutex::new(director)) }
    });

    engine.register_fn("configure_motion_blur", |movie: &mut MovieHandle, samples: i64, shutter_angle: f64| {
         let mut d = movie.director.lock().unwrap();
         d.samples_per_frame = samples as u32;
         d.shutter_angle = shutter_angle as f32;
    });

    // 2. Scene Management
    engine.register_type_with_name::<SceneHandle>("Scene");
    engine.register_fn("add_scene", |movie: &mut MovieHandle, duration: f64| {
        let mut d = movie.director.lock().unwrap();
        let start_time = d.timeline.last().map(|i| i.start_time + i.duration).unwrap_or(0.0);

        let root = BoxNode::new();
        let id = d.add_node(Box::new(root));

        let item = TimelineItem {
            scene_root: id,
            start_time,
            duration,
            z_index: 0,
        };
        d.timeline.push(item);

        SceneHandle {
            director: movie.director.clone(),
            root_id: id,
            start_time,
            duration,
        }
    });

    // 3. Elements
    engine.register_type_with_name::<NodeHandle>("Node");

    engine.register_fn("add_box", |scene: &mut SceneHandle, props: rhai::Map| {
        let mut d = scene.director.lock().unwrap();
        let mut box_node = BoxNode::new();

        if let Some(c) = props.get("bg_color") {
             if let Ok(s) = c.clone().into_string() {
                 if let Some(color) = parse_hex_color(&s) {
                     box_node.bg_color = Some(crate::animation::Animated::new(color));
                 }
             }
        }
         if let Some(c) = props.get("shadow_color") {
             if let Ok(s) = c.clone().into_string() {
                 if let Some(color) = parse_hex_color(&s) {
                     box_node.shadow_color = Some(crate::animation::Animated::new(color));
                 }
             }
        }
        if let Some(v) = props.get("shadow_blur").and_then(|v| v.as_float().ok()) {
            box_node.shadow_blur = crate::animation::Animated::new(v as f32);
        }
        if let Some(v) = props.get("shadow_x").and_then(|v| v.as_float().ok()) {
            box_node.shadow_offset_x = crate::animation::Animated::new(v as f32);
        }
        if let Some(v) = props.get("shadow_y").and_then(|v| v.as_float().ok()) {
            box_node.shadow_offset_y = crate::animation::Animated::new(v as f32);
        }

        let id = d.add_node(Box::new(box_node));
        d.add_child(scene.root_id, id);

        NodeHandle { director: scene.director.clone(), id }
    });

    engine.register_fn("add_text", |parent: &mut NodeHandle, props: rhai::Map| {
         let fs = Arc::new(Mutex::new(FontSystem::new()));
         let sc = Arc::new(Mutex::new(SwashCache::new()));

         let mut spans = Vec::new();

         if let Some(content_array) = props.get("content").and_then(|v| v.clone().into_array().ok()) {
             for item in content_array {
                 if let Some(map) = item.clone().try_cast::<Map>() {
                     let text = map.get("text").map(|v| v.to_string()).unwrap_or_default();
                     let mut span = TextSpan {
                         text,
                         color: None,
                         font_family: None,
                         font_weight: None,
                         font_style: None,
                         font_size: None,
                     };
                     if let Some(c) = map.get("color").and_then(|v| v.clone().into_string().ok()) {
                         span.color = parse_hex_color(&c);
                     }
                     if let Some(w_str) = map.get("weight").and_then(|v| v.clone().into_string().ok()) {
                         if w_str == "bold" { span.font_weight = Some(700); }
                     }
                     if let Some(s) = map.get("size").and_then(|v| v.as_float().ok()) {
                         span.font_size = Some(s as f32);
                     }
                     spans.push(span);
                 } else if let Ok(s) = item.into_string() {
                     spans.push(TextSpan { text: s, color: None, font_family: None, font_weight: None, font_style: None, font_size: None });
                 }
             }
         } else if let Some(s) = props.get("content").map(|v| v.to_string()) {
             spans.push(TextSpan { text: s, color: None, font_family: None, font_weight: None, font_style: None, font_size: None });
         }

         let text_node = TextNode::new(spans, fs, sc);

         let mut d = parent.director.lock().unwrap();
         let id = d.add_node(Box::new(text_node));
         d.add_child(parent.id, id);

         NodeHandle { director: parent.director.clone(), id }
    });

    engine.register_fn("set_content", |node: &mut NodeHandle, content: rhai::Dynamic| {
         let mut spans = Vec::new();
         if let Ok(arr) = content.clone().into_array() {
              for item in arr {
                  if let Some(map) = item.clone().try_cast::<Map>() {
                     let text = map.get("text").map(|v| v.to_string()).unwrap_or_default();
                     let mut span = TextSpan {
                         text,
                         color: None,
                         font_family: None,
                         font_weight: None,
                         font_style: None,
                         font_size: None,
                     };
                     if let Some(c) = map.get("color").and_then(|v| v.clone().into_string().ok()) {
                         span.color = parse_hex_color(&c);
                     }
                     if let Some(w_str) = map.get("weight").and_then(|v| v.clone().into_string().ok()) {
                         if w_str == "bold" { span.font_weight = Some(700); }
                     }
                     if let Some(s) = map.get("size").and_then(|v| v.as_float().ok()) {
                         span.font_size = Some(s as f32);
                     }
                     spans.push(span);
                  }
              }
         }

         let mut d = node.director.lock().unwrap();
         if let Some(n) = d.get_node_mut(node.id) {
             n.element.set_rich_text(spans);
         }
    });

    engine.register_fn("animate", |node: &mut NodeHandle, prop: &str, _start: f64, end: f64, dur: f64, ease: &str| {
        let mut d = node.director.lock().unwrap();
        if let Some(n) = d.get_node_mut(node.id) {
             n.element.animate_property(prop, end as f32, dur, ease);
        }
    });

    engine.register_fn("path_animate", |node: &mut NodeHandle, svg: &str, dur: f64, ease: &str| {
        let mut d = node.director.lock().unwrap();
        if let Some(n) = d.get_node_mut(node.id) {
             // Try to parse SVG path
             if let Some(path) = Path::from_svg(svg) {
                 let ease_fn = match ease {
                     "linear" => EasingType::Linear,
                     "ease_in" => EasingType::EaseIn,
                     "ease_out" => EasingType::EaseOut,
                     "ease_in_out" => EasingType::EaseInOut,
                     _ => EasingType::Linear,
                 };
                 let mut progress = Animated::new(0.0);
                 progress.add_keyframe(1.0, dur, ease_fn);

                 n.path_animation = Some(PathAnimationState {
                     path,
                     progress
                 });
             } else {
                 eprintln!("Failed to parse SVG path: {}", svg);
             }
        }
    });

    engine.register_fn("set_blur", |node: &mut NodeHandle, val: f64| {
         let mut d = node.director.lock().unwrap();
         if let Some(n) = d.get_node_mut(node.id) {
             n.element.animate_property("blur", val as f32, 0.0, "linear");
         }
    });
}
