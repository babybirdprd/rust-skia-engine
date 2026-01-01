//! # Nodes API
//!
//! Node creation functions for Rhai scripts.
//!
//! ## Responsibilities
//! - **Box Nodes**: `add_box` for container nodes
//! - **Text Nodes**: `add_text` for rich text
//! - **Image Nodes**: `add_image` for static images
//! - **Video Nodes**: `add_video` for video playback
//! - **Lottie Nodes**: `add_lottie` for Lottie animations
//! - **SVG Nodes**: `add_svg` for vector graphics
//! - **Composition Nodes**: `add_composition` for nested compositions
//! - **Node Destruction**: `destroy` to remove nodes

use crate::animation::Animated;
use crate::element::TextFit;
use crate::node::{
    BoxNode, CompositionNode, ImageNode, LottieNode, TextNode, VectorNode, VideoNode, VideoSource,
};
use crate::AssetLoader;
use rhai::{Engine, Map};
use std::collections::HashMap;
use std::sync::Arc;

use tracing::error;

use super::super::types::{MovieHandle, NodeHandle, SceneHandle};
use super::super::utils::{
    parse_hex_color, parse_layout_style, parse_object_fit, parse_spans_from_dynamic,
    parse_text_shadow,
};

/// Register node creation Rhai functions.
pub fn register(engine: &mut Engine, _loader: Arc<dyn AssetLoader>) {
    engine.register_type_with_name::<NodeHandle>("Node");

    engine.register_fn("destroy", |node: &mut NodeHandle| {
        let mut d = node.director.lock().unwrap();
        d.scene.destroy_node(node.id);
    });

    // ========== ADD_BOX ==========
    engine.register_fn("add_box", |parent: &mut NodeHandle, props: rhai::Map| {
        let mut d = parent.director.lock().unwrap();
        let mut box_node = BoxNode::new();
        apply_box_props(&mut box_node, &props);
        parse_layout_style(&props, &mut box_node.style);

        let id = d.scene.add_node(Box::new(box_node));
        if let Some(z) = props.get("z_index").and_then(|v| v.as_int().ok()) {
            if let Some(n) = d.scene.get_node_mut(id) {
                n.z_index = z as i32;
            }
        }
        d.scene.add_child(parent.id, id);

        NodeHandle {
            director: parent.director.clone(),
            id,
        }
    });

    engine.register_fn("add_box", |scene: &mut SceneHandle, props: rhai::Map| {
        let mut d = scene.director.lock().unwrap();
        let mut box_node = BoxNode::new();
        apply_box_props(&mut box_node, &props);
        parse_layout_style(&props, &mut box_node.style);

        let id = d.scene.add_node(Box::new(box_node));
        if let Some(z) = props.get("z_index").and_then(|v| v.as_int().ok()) {
            if let Some(n) = d.scene.get_node_mut(id) {
                n.z_index = z as i32;
            }
        }
        d.scene.add_child(scene.root_id, id);

        NodeHandle {
            director: scene.director.clone(),
            id,
        }
    });

    // ========== ADD_IMAGE ==========

    engine.register_fn("add_image", move |parent: &mut NodeHandle, path: &str| {
        let mut d = parent.director.lock().unwrap();
        let bytes = d.assets.loader.load_bytes(path).unwrap_or(Vec::new());

        let img_node = ImageNode::new(bytes);
        let id = d.scene.add_node(Box::new(img_node));
        d.scene.add_child(parent.id, id);
        NodeHandle {
            director: parent.director.clone(),
            id,
        }
    });

    engine.register_fn("add_image", |scene: &mut SceneHandle, path: &str| {
        let mut d = scene.director.lock().unwrap();
        let bytes = d.assets.loader.load_bytes(path).unwrap_or(Vec::new());

        let img_node = ImageNode::new(bytes);
        let id = d.scene.add_node(Box::new(img_node));
        d.scene.add_child(scene.root_id, id);
        NodeHandle {
            director: scene.director.clone(),
            id,
        }
    });

    engine.register_fn(
        "add_image",
        |scene: &mut SceneHandle, path: &str, props: rhai::Map| {
            let mut d = scene.director.lock().unwrap();
            let bytes = d.assets.loader.load_bytes(path).unwrap_or(Vec::new());

            let mut img_node = ImageNode::new(bytes);
            parse_layout_style(&props, &mut img_node.style);

            if let Some(fit_str) = props
                .get("object_fit")
                .and_then(|v| v.clone().into_string().ok())
            {
                if let Some(fit) = parse_object_fit(&fit_str) {
                    img_node.object_fit = fit;
                }
            }

            let id = d.scene.add_node(Box::new(img_node));
            if let Some(z) = props.get("z_index").and_then(|v| v.as_int().ok()) {
                if let Some(n) = d.scene.get_node_mut(id) {
                    n.z_index = z as i32;
                }
            }
            d.scene.add_child(scene.root_id, id);
            NodeHandle {
                director: scene.director.clone(),
                id,
            }
        },
    );

    engine.register_fn(
        "add_image",
        |parent: &mut NodeHandle, path: &str, props: rhai::Map| {
            let mut d = parent.director.lock().unwrap();
            let bytes = d.assets.loader.load_bytes(path).unwrap_or(Vec::new());

            let mut img_node = ImageNode::new(bytes);
            parse_layout_style(&props, &mut img_node.style);

            if let Some(fit_str) = props
                .get("object_fit")
                .and_then(|v| v.clone().into_string().ok())
            {
                if let Some(fit) = parse_object_fit(&fit_str) {
                    img_node.object_fit = fit;
                }
            }

            let id = d.scene.add_node(Box::new(img_node));
            if let Some(z) = props.get("z_index").and_then(|v| v.as_int().ok()) {
                if let Some(n) = d.scene.get_node_mut(id) {
                    n.z_index = z as i32;
                }
            }
            d.scene.add_child(parent.id, id);
            NodeHandle {
                director: parent.director.clone(),
                id,
            }
        },
    );

    // ========== ADD_LOTTIE ==========
    engine.register_fn(
        "add_lottie",
        |parent: &mut NodeHandle, path: &str| -> Result<NodeHandle, Box<rhai::EvalAltResult>> {
            let mut d = parent.director.lock().unwrap();
            let bytes = d
                .assets
                .loader
                .load_bytes(path)
                .map_err(|e| e.to_string())?;

            match LottieNode::new(&bytes, HashMap::new(), &d.assets) {
                Ok(lottie_node) => {
                    let id = d.scene.add_node(Box::new(lottie_node));
                    d.scene.add_child(parent.id, id);
                    Ok(NodeHandle {
                        director: parent.director.clone(),
                        id,
                    })
                }
                Err(e) => Err(format!("Failed to parse lottie: {}", e).into()),
            }
        },
    );

    engine.register_fn(
        "add_lottie",
        |parent: &mut NodeHandle,
         path: &str,
         props: rhai::Map|
         -> Result<NodeHandle, Box<rhai::EvalAltResult>> {
            let mut d = parent.director.lock().unwrap();
            let bytes = d
                .assets
                .loader
                .load_bytes(path)
                .map_err(|e| e.to_string())?;

            let mut assets_map = HashMap::new();
            if let Some(assets_prop) = props
                .get("assets")
                .and_then(|v| v.clone().try_cast::<Map>())
            {
                for (key, val) in assets_prop {
                    if let Ok(asset_path) = val.into_string() {
                        let asset_bytes = d
                            .assets
                            .loader
                            .load_bytes(&asset_path)
                            .unwrap_or(Vec::new());
                        let data = skia_safe::Data::new_copy(&asset_bytes);
                        if let Some(image) = skia_safe::Image::from_encoded(data) {
                            assets_map.insert(key.to_string(), image);
                        }
                    }
                }
            }

            match LottieNode::new(&bytes, assets_map, &d.assets) {
                Ok(mut lottie_node) => {
                    parse_layout_style(&props, &mut lottie_node.style);
                    if let Some(v) = props.get("speed").and_then(|v| v.as_float().ok()) {
                        lottie_node.speed = v as f32;
                    }
                    if let Some(v) = props.get("loop").and_then(|v| v.as_bool().ok()) {
                        lottie_node.loop_anim = v;
                    }

                    let id = d.scene.add_node(Box::new(lottie_node));
                    if let Some(z) = props.get("z_index").and_then(|v| v.as_int().ok()) {
                        if let Some(n) = d.scene.get_node_mut(id) {
                            n.z_index = z as i32;
                        }
                    }
                    d.scene.add_child(parent.id, id);
                    Ok(NodeHandle {
                        director: parent.director.clone(),
                        id,
                    })
                }
                Err(e) => Err(format!("Failed to parse lottie: {}", e).into()),
            }
        },
    );

    engine.register_fn(
        "add_lottie",
        |scene: &mut SceneHandle,
         path: &str,
         props: rhai::Map|
         -> Result<NodeHandle, Box<rhai::EvalAltResult>> {
            let mut d = scene.director.lock().unwrap();
            let bytes = d
                .assets
                .loader
                .load_bytes(path)
                .map_err(|e| e.to_string())?;

            let mut assets_map = HashMap::new();
            if let Some(assets_prop) = props
                .get("assets")
                .and_then(|v| v.clone().try_cast::<Map>())
            {
                for (key, val) in assets_prop {
                    if let Ok(asset_path) = val.into_string() {
                        let asset_bytes = d
                            .assets
                            .loader
                            .load_bytes(&asset_path)
                            .unwrap_or(Vec::new());
                        let data = skia_safe::Data::new_copy(&asset_bytes);
                        if let Some(image) = skia_safe::Image::from_encoded(data) {
                            assets_map.insert(key.to_string(), image);
                        }
                    }
                }
            }

            match LottieNode::new(&bytes, assets_map, &d.assets) {
                Ok(mut lottie_node) => {
                    parse_layout_style(&props, &mut lottie_node.style);
                    if let Some(v) = props.get("speed").and_then(|v| v.as_float().ok()) {
                        lottie_node.speed = v as f32;
                    }
                    if let Some(v) = props.get("loop").and_then(|v| v.as_bool().ok()) {
                        lottie_node.loop_anim = v;
                    }

                    let id = d.scene.add_node(Box::new(lottie_node));
                    if let Some(z) = props.get("z_index").and_then(|v| v.as_int().ok()) {
                        if let Some(n) = d.scene.get_node_mut(id) {
                            n.z_index = z as i32;
                        }
                    }
                    d.scene.add_child(scene.root_id, id);
                    Ok(NodeHandle {
                        director: scene.director.clone(),
                        id,
                    })
                }
                Err(e) => Err(format!("Failed to parse lottie: {}", e).into()),
            }
        },
    );

    // ========== ADD_SVG ==========
    engine.register_fn("add_svg", |scene: &mut SceneHandle, path: &str| {
        let mut d = scene.director.lock().unwrap();
        let bytes = d.assets.loader.load_bytes(path).unwrap_or(Vec::new());

        let vec_node = VectorNode::new(&bytes);
        let id = d.scene.add_node(Box::new(vec_node));
        d.scene.add_child(scene.root_id, id);
        NodeHandle {
            director: scene.director.clone(),
            id,
        }
    });

    engine.register_fn(
        "add_svg",
        |scene: &mut SceneHandle, path: &str, props: rhai::Map| {
            let mut d = scene.director.lock().unwrap();
            let bytes = d.assets.loader.load_bytes(path).unwrap_or(Vec::new());

            let mut vec_node = VectorNode::new(&bytes);
            parse_layout_style(&props, &mut vec_node.style);

            let id = d.scene.add_node(Box::new(vec_node));
            if let Some(z) = props.get("z_index").and_then(|v| v.as_int().ok()) {
                if let Some(n) = d.scene.get_node_mut(id) {
                    n.z_index = z as i32;
                }
            }
            d.scene.add_child(scene.root_id, id);
            NodeHandle {
                director: scene.director.clone(),
                id,
            }
        },
    );

    engine.register_fn("add_svg", |parent: &mut NodeHandle, path: &str| {
        let mut d = parent.director.lock().unwrap();
        let bytes = d.assets.loader.load_bytes(path).unwrap_or(Vec::new());

        let vec_node = VectorNode::new(&bytes);
        let id = d.scene.add_node(Box::new(vec_node));
        d.scene.add_child(parent.id, id);
        NodeHandle {
            director: parent.director.clone(),
            id,
        }
    });

    engine.register_fn(
        "add_svg",
        |parent: &mut NodeHandle, path: &str, props: rhai::Map| {
            let mut d = parent.director.lock().unwrap();
            let bytes = d.assets.loader.load_bytes(path).unwrap_or(Vec::new());

            let mut vec_node = VectorNode::new(&bytes);
            parse_layout_style(&props, &mut vec_node.style);

            let id = d.scene.add_node(Box::new(vec_node));
            if let Some(z) = props.get("z_index").and_then(|v| v.as_int().ok()) {
                if let Some(n) = d.scene.get_node_mut(id) {
                    n.z_index = z as i32;
                }
            }
            d.scene.add_child(parent.id, id);
            NodeHandle {
                director: parent.director.clone(),
                id,
            }
        },
    );

    // ========== ADD_VIDEO ==========
    engine.register_fn("add_video", |parent: &mut NodeHandle, path: &str| {
        let mut d = parent.director.lock().unwrap();
        let mode = d.render_mode;
        let p = std::path::Path::new(path);

        let source = if p.exists() && p.is_file() {
            VideoSource::Path(p.to_path_buf())
        } else {
            let bytes = d.assets.loader.load_bytes(path).unwrap_or(Vec::new());
            VideoSource::Bytes(bytes)
        };

        let vid_node = VideoNode::new(source, mode);
        let id = d.scene.add_node(Box::new(vid_node));
        d.scene.add_child(parent.id, id);
        NodeHandle {
            director: parent.director.clone(),
            id,
        }
    });

    engine.register_fn(
        "add_video",
        |parent: &mut NodeHandle, path: &str, props: rhai::Map| {
            let mut d = parent.director.lock().unwrap();
            let mode = d.render_mode;
            let p = std::path::Path::new(path);

            let source = if p.exists() && p.is_file() {
                VideoSource::Path(p.to_path_buf())
            } else {
                let bytes = d.assets.loader.load_bytes(path).unwrap_or(Vec::new());
                VideoSource::Bytes(bytes)
            };

            let mut vid_node = VideoNode::new(source, mode);
            parse_layout_style(&props, &mut vid_node.style);

            if let Some(fit_str) = props
                .get("object_fit")
                .and_then(|v| v.clone().into_string().ok())
            {
                if let Some(fit) = parse_object_fit(&fit_str) {
                    vid_node.object_fit = fit;
                }
            }

            let id = d.scene.add_node(Box::new(vid_node));
            if let Some(z) = props.get("z_index").and_then(|v| v.as_int().ok()) {
                if let Some(n) = d.scene.get_node_mut(id) {
                    n.z_index = z as i32;
                }
            }
            d.scene.add_child(parent.id, id);
            NodeHandle {
                director: parent.director.clone(),
                id,
            }
        },
    );

    engine.register_fn("add_video", |scene: &mut SceneHandle, path: &str| {
        let mut d = scene.director.lock().unwrap();
        let mode = d.render_mode;
        let p = std::path::Path::new(path);

        let source = if p.exists() && p.is_file() {
            VideoSource::Path(p.to_path_buf())
        } else {
            let bytes = d.assets.loader.load_bytes(path).unwrap_or(Vec::new());
            VideoSource::Bytes(bytes)
        };

        let vid_node = VideoNode::new(source, mode);
        let id = d.scene.add_node(Box::new(vid_node));
        d.scene.add_child(scene.root_id, id);
        NodeHandle {
            director: scene.director.clone(),
            id,
        }
    });

    engine.register_fn(
        "add_video",
        |scene: &mut SceneHandle, path: &str, props: rhai::Map| {
            let mut d = scene.director.lock().unwrap();
            let mode = d.render_mode;
            let p = std::path::Path::new(path);

            let source = if p.exists() && p.is_file() {
                VideoSource::Path(p.to_path_buf())
            } else {
                let bytes = d.assets.loader.load_bytes(path).unwrap_or(Vec::new());
                VideoSource::Bytes(bytes)
            };

            let mut vid_node = VideoNode::new(source, mode);
            parse_layout_style(&props, &mut vid_node.style);

            if let Some(fit_str) = props
                .get("object_fit")
                .and_then(|v| v.clone().into_string().ok())
            {
                if let Some(fit) = parse_object_fit(&fit_str) {
                    vid_node.object_fit = fit;
                }
            }

            let id = d.scene.add_node(Box::new(vid_node));
            if let Some(z) = props.get("z_index").and_then(|v| v.as_int().ok()) {
                if let Some(n) = d.scene.get_node_mut(id) {
                    n.z_index = z as i32;
                }
            }
            d.scene.add_child(scene.root_id, id);
            NodeHandle {
                director: scene.director.clone(),
                id,
            }
        },
    );

    // ========== ADD_TEXT ==========
    engine.register_fn("add_text", |parent: &mut NodeHandle, props: rhai::Map| {
        let mut d = parent.director.lock().unwrap();
        let font_collection = d.assets.font_collection.clone();

        let spans = if let Some(c) = props.get("content") {
            parse_spans_from_dynamic(c.clone())
        } else {
            Vec::new()
        };

        let mut text_node = TextNode::new(spans, font_collection);
        apply_text_props(&mut text_node, &props);
        parse_layout_style(&props, &mut text_node.style);
        text_node.shadow = parse_text_shadow(&props);
        text_node.init_paragraph();

        let id = d.scene.add_node(Box::new(text_node));
        if let Some(z) = props.get("z_index").and_then(|v| v.as_int().ok()) {
            if let Some(n) = d.scene.get_node_mut(id) {
                n.z_index = z as i32;
            }
        }
        d.scene.add_child(parent.id, id);

        NodeHandle {
            director: parent.director.clone(),
            id,
        }
    });

    engine.register_fn("add_text", |scene: &mut SceneHandle, props: rhai::Map| {
        let mut d = scene.director.lock().unwrap();
        let font_collection = d.assets.font_collection.clone();

        let spans = if let Some(c) = props.get("content") {
            parse_spans_from_dynamic(c.clone())
        } else {
            Vec::new()
        };

        let mut text_node = TextNode::new(spans, font_collection);
        apply_text_props(&mut text_node, &props);
        parse_layout_style(&props, &mut text_node.style);
        text_node.shadow = parse_text_shadow(&props);
        text_node.init_paragraph();

        let id = d.scene.add_node(Box::new(text_node));
        if let Some(z) = props.get("z_index").and_then(|v| v.as_int().ok()) {
            if let Some(n) = d.scene.get_node_mut(id) {
                n.z_index = z as i32;
            }
        }
        d.scene.add_child(scene.root_id, id);

        NodeHandle {
            director: scene.director.clone(),
            id,
        }
    });

    // ========== ADD_COMPOSITION ==========
    engine.register_fn(
        "add_composition",
        |scene: &mut SceneHandle, comp_def: MovieHandle| {
            // Cycle Detection
            if Arc::ptr_eq(&scene.director, &comp_def.director) {
                error!("Cycle detected. A composition cannot contain itself.");
                let mut d = scene.director.lock().unwrap();
                let id = d.scene.add_node(Box::new(BoxNode::new()));
                return NodeHandle {
                    director: scene.director.clone(),
                    id,
                };
            }

            let mut inner_director = comp_def.director.lock().unwrap().clone();

            // Share resources from parent
            {
                let parent = scene.director.lock().unwrap();
                inner_director.assets = parent.assets.clone();
            }

            let comp_node = CompositionNode::new(inner_director);

            let mut d = scene.director.lock().unwrap();
            let id = d.scene.add_node(Box::new(comp_node));
            d.scene.add_child(scene.root_id, id);

            NodeHandle {
                director: scene.director.clone(),
                id,
            }
        },
    );

    engine.register_fn(
        "add_composition",
        |scene: &mut SceneHandle, comp_def: MovieHandle, props: rhai::Map| {
            // Cycle Detection
            if Arc::ptr_eq(&scene.director, &comp_def.director) {
                error!("Cycle detected. A composition cannot contain itself.");
                let mut d = scene.director.lock().unwrap();
                let id = d.scene.add_node(Box::new(BoxNode::new()));
                return NodeHandle {
                    director: scene.director.clone(),
                    id,
                };
            }

            let mut inner_director = comp_def.director.lock().unwrap().clone();

            // Share resources from parent
            {
                let parent = scene.director.lock().unwrap();
                inner_director.assets = parent.assets.clone();
            }

            let mut comp_node = CompositionNode::new(inner_director);
            parse_layout_style(&props, &mut comp_node.style);

            let mut d = scene.director.lock().unwrap();
            let id = d.scene.add_node(Box::new(comp_node));
            d.scene.add_child(scene.root_id, id);

            NodeHandle {
                director: scene.director.clone(),
                id,
            }
        },
    );
}

/// Apply box-specific properties from a Rhai map
fn apply_box_props(box_node: &mut BoxNode, props: &rhai::Map) {
    if let Some(c) = props.get("bg_color") {
        if let Ok(s) = c.clone().into_string() {
            if let Some(color) = parse_hex_color(&s) {
                box_node.bg_color = Some(Animated::new(color));
            }
        }
    }
    if let Some(c) = props.get("shadow_color") {
        if let Ok(s) = c.clone().into_string() {
            if let Some(color) = parse_hex_color(&s) {
                box_node.shadow_color = Some(Animated::new(color));
            }
        }
    }
    if let Some(v) = props.get("shadow_blur").and_then(|v| v.as_float().ok()) {
        box_node.shadow_blur = Animated::new(v as f32);
    }
    if let Some(v) = props.get("shadow_x").and_then(|v| v.as_float().ok()) {
        box_node.shadow_offset_x = Animated::new(v as f32);
    }
    if let Some(v) = props.get("shadow_y").and_then(|v| v.as_float().ok()) {
        box_node.shadow_offset_y = Animated::new(v as f32);
    }
    if let Some(v) = props.get("border_radius").and_then(|v| v.as_float().ok()) {
        box_node.border_radius = Animated::new(v as f32);
    }
    if let Some(v) = props.get("border_width").and_then(|v| v.as_float().ok()) {
        box_node.border_width = Animated::new(v as f32);
    }
    if let Some(c) = props.get("border_color") {
        if let Ok(s) = c.clone().into_string() {
            if let Some(color) = parse_hex_color(&s) {
                box_node.border_color = Some(Animated::new(color));
            }
        }
    }
    if let Some(s) = props
        .get("overflow")
        .and_then(|v| v.clone().into_string().ok())
    {
        box_node.overflow = s;
    }
}

/// Apply text-specific properties from a Rhai map
fn apply_text_props(text_node: &mut TextNode, props: &rhai::Map) {
    if let Some(s) = props.get("size").and_then(|v| v.as_float().ok()) {
        text_node.default_font_size = Animated::new(s as f32);
    }
    if let Some(c) = props
        .get("color")
        .and_then(|v| v.clone().into_string().ok())
    {
        if let Some(col) = parse_hex_color(&c) {
            text_node.default_color = Animated::new(col);
        }
    }

    let weight = props
        .get("weight")
        .and_then(|v| v.clone().into_string().ok())
        .and_then(|s| if s == "bold" { Some(700) } else { None });

    // Apply span-level defaults if missing
    for span in &mut text_node.spans {
        if span.font_weight.is_none() {
            span.font_weight = weight;
        }
    }

    if let Some(s) = props.get("fit").and_then(|v| v.clone().into_string().ok()) {
        text_node.fit_mode = match s.as_str() {
            "shrink" => TextFit::Shrink,
            _ => TextFit::None,
        };
    }
    if let Some(v) = props.get("min_size").and_then(|v| v.as_float().ok()) {
        text_node.min_size = v as f32;
    }
    if let Some(v) = props.get("max_size").and_then(|v| v.as_float().ok()) {
        text_node.max_size = v as f32;
    }
}
