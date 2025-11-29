use rhai::Engine;
use crate::director::{Director, NodeId};
use crate::node::{BoxNode, TextNode, Color};
use std::sync::{Arc, Mutex};
use cosmic_text::{FontSystem, SwashCache};

/// Wrapper around `Director` for Rhai scripting.
/// Allows sharing the Director instance safely across script calls.
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
/// Used to chain method calls like `.animate()` in Rhai.
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
            // Expand 0xF -> 0xFF
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

/// Registers the Director DSL into the Rhai engine.
///
/// # Supported Types
/// * `Movie`
/// * `Scene`
/// * `Node`
///
/// # Supported Functions
/// * `new_director(w, h, fps)`
/// * `movie.add_scene(duration)`
/// * `scene.add_box(props)`
/// * `box.add_text(props)`
/// * `node.animate(prop, target, duration, easing)`
/// * `node.set_blur(radius)`
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
        let root = BoxNode::new();
        let id = d.add_node(Box::new(root));

        if d.root_id.is_none() {
            d.root_id = Some(id);
        }

        SceneHandle {
            director: movie.director.clone(),
            root_id: id,
            start_time: 0.0,
            duration,
        }
    });

    // 3. Elements
    engine.register_type_with_name::<NodeHandle>("Node");

    // box = scene.add_box(...)
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

        // Shadows
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

    // text = box.add_text(...)
    engine.register_fn("add_text", |parent: &mut NodeHandle, props: rhai::Map| {
         let fs = Arc::new(Mutex::new(FontSystem::new()));
         let sc = Arc::new(Mutex::new(SwashCache::new()));

         let content = props.get("content").unwrap().to_string();
         let text_node = TextNode::new(content, fs, sc);

         let mut d = parent.director.lock().unwrap();
         let id = d.add_node(Box::new(text_node));
         d.add_child(parent.id, id);

         NodeHandle { director: parent.director.clone(), id }
    });

    // 4. Animation & Properties
    engine.register_fn("animate", |node: &mut NodeHandle, prop: &str, _start: f64, end: f64, dur: f64, ease: &str| {
        let mut d = node.director.lock().unwrap();
        if let Some(n) = d.get_node_mut(node.id) {
             // start is unused in add_keyframe logic (uses current value or jumps),
             // but effectively we might want to set start value if time is 0.
             // For now we just animate to 'end'.
             n.element.animate_property(prop, end as f32, dur, ease);
        }
    });

    // 5. Effects (Blur)
    // node.set_blur(10.0) could be added or just via animate("blur")
    engine.register_fn("set_blur", |node: &mut NodeHandle, val: f64| {
         let mut d = node.director.lock().unwrap();
         if let Some(n) = d.get_node_mut(node.id) {
             // We need to jump the blur animation to this value
             // This requires animate_property to handle immediate set or add a helper.
             // We'll just animate it over 0s.
             n.element.animate_property("blur", val as f32, 0.0, "linear");
         }
    });
}
