use rhai::{Engine, FuncRegistration};
use crate::director::{Director, NodeId};
use crate::node::{BoxNode, TextNode, Color};
use std::sync::{Arc, Mutex};
use cosmic_text::{FontSystem, SwashCache};

// Wrapper structs for Rhai to hold handles
#[derive(Clone)]
pub struct MovieHandle {
    pub director: Arc<Mutex<Director>>,
}

#[derive(Clone)]
pub struct SceneHandle {
    pub director: Arc<Mutex<Director>>,
    pub root_id: NodeId,
    pub start_time: f64,
    pub duration: f64,
}

#[derive(Clone)]
pub struct NodeHandle {
    pub director: Arc<Mutex<Director>>,
    pub id: NodeId,
}

pub fn register_rhai_api(engine: &mut Engine) {
    // 1. Director/Movie
    engine.register_type_with_name::<MovieHandle>("Movie");
    engine.register_fn("new_director", |w: i64, h: i64, fps: i64| {
        let director = Director::new(w as i32, h as i32, fps as u32);
        MovieHandle { director: Arc::new(Mutex::new(director)) }
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
             if let Ok(_s) = c.clone().into_string() {
                 box_node.bg_color = Some(crate::animation::Animated::new(Color::WHITE)); // TODO: Parse Hex
             }
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
