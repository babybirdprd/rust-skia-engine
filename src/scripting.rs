use rhai::{Engine, Map, Module};
use crate::director::{Director, NodeId, TimelineItem, PathAnimationState, Transition, TransitionType};
use crate::node::{BoxNode, TextNode, ImageNode, VideoNode, CompositionNode, EffectType, EffectNode, VectorNode, LottieNode, VideoSource};
use crate::video_wrapper::RenderMode;
use crate::element::{Element, Color, TextSpan, GradientConfig, TextFit, TextShadow};
use crate::animation::{Animated, EasingType};
use crate::tokens::DesignSystem;
use crate::AssetLoader;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use skia_safe::Path;
use taffy::style::{Style, FlexDirection, AlignItems, JustifyContent, Dimension, LengthPercentage, LengthPercentageAuto};
use taffy::geometry::Rect;

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
    pub audio_tracks: Vec<usize>,
}

/// Handle to a specific Node in the scene graph.
#[derive(Clone)]
pub struct NodeHandle {
    pub director: Arc<Mutex<Director>>,
    pub id: NodeId,
}

/// Handle to an audio track.
#[derive(Clone)]
pub struct AudioTrackHandle {
    pub director: Arc<Mutex<Director>>,
    pub id: usize,
}

fn extract_outer_style(source: &Style) -> Style {
    Style {
        display: source.display,
        position: source.position,
        inset: source.inset,
        size: source.size,
        min_size: source.min_size,
        max_size: source.max_size,
        aspect_ratio: source.aspect_ratio,
        margin: source.margin,
        flex_grow: source.flex_grow,
        flex_shrink: source.flex_shrink,
        flex_basis: source.flex_basis,
        align_self: source.align_self,
        justify_self: source.justify_self,
        grid_row: source.grid_row.clone(),
        grid_column: source.grid_column.clone(),
        padding: taffy::geometry::Rect::zero(),
        border: taffy::geometry::Rect::zero(),
        gap: taffy::geometry::Size::zero(),
        ..Default::default()
    }
}

fn apply_effect_to_node(d: &mut Director, node_id: NodeId, effect: EffectType) -> NodeId {
    let parent_id_opt = d.get_node(node_id).and_then(|n| n.parent);

    let mut wrapper_style = Style::default();

    // Modify target node style (Steal & Fill)
    if let Some(node) = d.get_node_mut(node_id) {
        let original_style = node.element.layout_style();
        wrapper_style = extract_outer_style(&original_style);

        if let Some(box_node) = node.element.as_any_mut().downcast_mut::<BoxNode>() {
             box_node.style.size = taffy::geometry::Size { width: Dimension::percent(1.0), height: Dimension::percent(1.0) };
             box_node.style.margin = taffy::geometry::Rect::zero();
             box_node.style.flex_grow = 0.0;
             box_node.style.flex_shrink = 1.0;
             box_node.style.position = taffy::style::Position::Relative;
             box_node.style.inset = taffy::geometry::Rect::auto();
        } else if let Some(comp_node) = node.element.as_any_mut().downcast_mut::<CompositionNode>() {
             comp_node.style.size = taffy::geometry::Size { width: Dimension::percent(1.0), height: Dimension::percent(1.0) };
             comp_node.style.margin = taffy::geometry::Rect::zero();
             comp_node.style.flex_grow = 0.0;
             comp_node.style.flex_shrink = 1.0;
             comp_node.style.position = taffy::style::Position::Relative;
             comp_node.style.inset = taffy::geometry::Rect::auto();
        }
    } else {
        return node_id; // Failure fallback
    }

    let effect_node = EffectNode {
        effects: vec![effect],
        style: wrapper_style,
        shader_cache: d.shader_cache.clone(),
    };

    let effect_id = d.add_node(Box::new(effect_node));

    if let Some(parent_id) = parent_id_opt {
        d.remove_child(parent_id, node_id);
        d.add_child(parent_id, effect_id);
    }

    d.add_child(effect_id, node_id);

    // Update Root if needed
    for item in &mut d.timeline {
        if item.scene_root == node_id {
            item.scene_root = effect_id;
        }
    }

    effect_id
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

fn parse_text_style(map: &rhai::Map, span: &mut TextSpan) {
    if let Some(c) = map.get("color").and_then(|v| v.clone().into_string().ok()) {
        span.color = parse_hex_color(&c);
    }
    if let Some(w_str) = map.get("weight").and_then(|v| v.clone().into_string().ok()) {
        if w_str == "bold" { span.font_weight = Some(700); }
    }
    if let Some(s) = map.get("size").and_then(|v| v.as_float().ok()) {
        span.font_size = Some(s as f32);
    }
    // Rich Text Fields
    if let Some(c) = map.get("background_color").and_then(|v| v.clone().into_string().ok()) {
        span.background_color = parse_hex_color(&c);
    }
    if let Some(p) = map.get("background_padding").and_then(|v| v.as_float().ok()) {
        span.background_padding = Some(p as f32);
    }
    if let Some(w) = map.get("stroke_width").and_then(|v| v.as_float().ok()) {
        span.stroke_width = Some(w as f32);
    }
    if let Some(c) = map.get("stroke_color").and_then(|v| v.clone().into_string().ok()) {
        span.stroke_color = parse_hex_color(&c);
    }
    if let Some(g) = map.get("fill_gradient") {
        if let Ok(arr) = g.clone().into_array() {
            // Simple array of colors
             let mut colors = Vec::new();
             for item in arr {
                 if let Ok(s) = item.into_string() {
                     if let Some(c) = parse_hex_color(&s) {
                         colors.push(c);
                     }
                 }
             }
             if !colors.is_empty() {
                 span.fill_gradient = Some(GradientConfig {
                     colors,
                     ..Default::default()
                 });
             }
        } else if let Some(gmap) = g.clone().try_cast::<Map>() {
             // Advanced map
             let mut config = GradientConfig::default();
             if let Some(arr) = gmap.get("colors").and_then(|v| v.clone().into_array().ok()) {
                  let mut colors = Vec::new();
                  for item in arr {
                      if let Ok(s) = item.into_string() {
                          if let Some(c) = parse_hex_color(&s) {
                              colors.push(c);
                          }
                      }
                  }
                  config.colors = colors;
             }
             if let Some(arr) = gmap.get("start").and_then(|v| v.clone().into_array().ok()) {
                 if arr.len() >= 2 {
                     let x = arr[0].as_float().unwrap_or(0.0) as f32;
                     let y = arr[1].as_float().unwrap_or(0.0) as f32;
                     config.start = (x, y);
                 }
             }
             if let Some(arr) = gmap.get("end").and_then(|v| v.clone().into_array().ok()) {
                 if arr.len() >= 2 {
                     let x = arr[0].as_float().unwrap_or(0.0) as f32;
                     let y = arr[1].as_float().unwrap_or(0.0) as f32;
                     config.end = (x, y);
                 }
             }
             span.fill_gradient = Some(config);
        }
    }
}

fn parse_spring_config(map: &rhai::Map) -> crate::animation::SpringConfig {
    let mut config = crate::animation::SpringConfig::default();
    if let Some(v) = map.get("stiffness").and_then(|v| v.as_float().ok()) { config.stiffness = v as f32; }
    if let Some(v) = map.get("damping").and_then(|v| v.as_float().ok()) { config.damping = v as f32; }
    if let Some(v) = map.get("mass").and_then(|v| v.as_float().ok()) { config.mass = v as f32; }
    if let Some(v) = map.get("velocity").and_then(|v| v.as_float().ok()) { config.velocity = v as f32; }
    config
}

fn parse_layout_style(props: &rhai::Map, style: &mut Style) {
    let to_dim = |v: &rhai::Dynamic| -> Option<Dimension> {
        if let Ok(f) = v.as_float() { Some(Dimension::length(f as f32)) }
        else if let Ok(i) = v.as_int() { Some(Dimension::length(i as f32)) }
        else if let Ok(s) = v.clone().into_string() {
            if s == "auto" { Some(Dimension::auto()) }
            else if s.ends_with("%") {
                if let Ok(p) = s.trim_end_matches('%').parse::<f32>() {
                     Some(Dimension::percent(p / 100.0))
                } else { None }
            } else { None }
        } else { None }
    };

    if let Some(w) = props.get("width").and_then(to_dim) { style.size.width = w; }
    if let Some(h) = props.get("height").and_then(to_dim) { style.size.height = h; }

    if let Some(s) = props.get("flex_direction").and_then(|v| v.clone().into_string().ok()) {
        style.flex_direction = match s.as_str() {
            "row" => FlexDirection::Row,
            "column" => FlexDirection::Column,
            "row_reverse" | "row-reverse" => FlexDirection::RowReverse,
            "column_reverse" | "column-reverse" => FlexDirection::ColumnReverse,
            _ => FlexDirection::Row,
        };
    }

    if let Some(s) = props.get("align_items").and_then(|v| v.clone().into_string().ok()) {
        style.align_items = match s.as_str() {
            "center" => Some(AlignItems::Center),
            "flex_start" | "flex-start" | "start" => Some(AlignItems::FlexStart),
            "flex_end" | "flex-end" | "end" => Some(AlignItems::FlexEnd),
            "stretch" => Some(AlignItems::Stretch),
            _ => Some(AlignItems::Stretch),
        };
    }

    if let Some(s) = props.get("justify_content").and_then(|v| v.clone().into_string().ok()) {
        style.justify_content = match s.as_str() {
            "center" => Some(JustifyContent::Center),
            "flex_start" | "flex-start" | "start" => Some(JustifyContent::FlexStart),
            "flex_end" | "flex-end" | "end" => Some(JustifyContent::FlexEnd),
            "space_between" | "space-between" => Some(JustifyContent::SpaceBetween),
            "space_around" | "space-around" => Some(JustifyContent::SpaceAround),
            "space_evenly" | "space-evenly" => Some(JustifyContent::SpaceEvenly),
            _ => Some(JustifyContent::FlexStart),
        };
    }

    if let Some(f) = props.get("flex_grow").and_then(|v| v.as_float().ok()) {
        style.flex_grow = f as f32;
    }
    if let Some(f) = props.get("flex_shrink").and_then(|v| v.as_float().ok()) {
        style.flex_shrink = f as f32;
    }

    // Padding/Margin
    let to_lp = |v: &rhai::Dynamic| -> Option<LengthPercentage> {
        if let Ok(f) = v.as_float() { Some(LengthPercentage::length(f as f32)) }
        else if let Ok(i) = v.as_int() { Some(LengthPercentage::length(i as f32)) }
        else if let Ok(s) = v.clone().into_string() {
             if s.ends_with("%") {
                if let Ok(p) = s.trim_end_matches('%').parse::<f32>() {
                     Some(LengthPercentage::percent(p / 100.0))
                } else { None }
            } else { None }
        } else { None }
    };

    if let Some(p) = props.get("padding").and_then(to_lp) {
        style.padding = Rect { left: p, right: p, top: p, bottom: p };
    }

    let to_lpa = |v: &rhai::Dynamic| -> Option<LengthPercentageAuto> {
         if let Ok(f) = v.as_float() { Some(LengthPercentageAuto::length(f as f32)) }
         else if let Ok(i) = v.as_int() { Some(LengthPercentageAuto::length(i as f32)) }
         else if let Ok(s) = v.clone().into_string() {
             if s == "auto" { Some(LengthPercentageAuto::auto()) }
             else if s.ends_with("%") {
                if let Ok(p) = s.trim_end_matches('%').parse::<f32>() {
                     Some(LengthPercentageAuto::percent(p / 100.0))
                } else { None }
            } else { None }
         } else { None }
    };

    if let Some(m) = props.get("margin").and_then(to_lpa) {
        style.margin = Rect { left: m, right: m, top: m, bottom: m };
    }
}

fn parse_spans_from_dynamic(content: rhai::Dynamic) -> Vec<TextSpan> {
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
                     background_color: None,
                     background_padding: None,
                     stroke_width: None,
                     stroke_color: None,
                     fill_gradient: None,
                 };
                 parse_text_style(&map, &mut span);
                 spans.push(span);
             } else if let Ok(s) = item.into_string() {
                 spans.push(TextSpan {
                     text: s,
                     color: None,
                     font_family: None,
                     font_weight: None,
                     font_style: None,
                     font_size: None,
                     background_color: None,
                     background_padding: None,
                     stroke_width: None,
                     stroke_color: None,
                     fill_gradient: None,
                 });
             }
         }
     } else if let Ok(s) = content.into_string() {
         spans.push(TextSpan {
             text: s,
             color: None,
             font_family: None,
             font_weight: None,
             font_style: None,
             font_size: None,
             background_color: None,
             background_padding: None,
             stroke_width: None,
             stroke_color: None,
             fill_gradient: None,
         });
     }
     spans
}

pub fn create_theme_api(system: DesignSystem) -> Module {
    let mut module = Module::new();
    let sys = Arc::new(system);

    // 1. Spacing: theme.space("md")
    let s = sys.clone();
    module.set_native_fn("space", move |key: &str| {
        Ok(s.spacing.get(key).copied().map(|v| v as f64).unwrap_or(16.0))
    });

    // 2. Safe Area: theme.safe_area("tiktok") -> Map
    let s = sys.clone();
    module.set_native_fn("safe_area", move |platform: &str| {
        let zone = s.safe_areas.get(platform).or_else(|| s.safe_areas.get("desktop")).unwrap();
        let mut map = Map::new();
        map.insert("top".into(), (zone.top as f64).into());
        map.insert("bottom".into(), (zone.bottom as f64).into());
        map.insert("left".into(), (zone.left as f64).into());
        map.insert("right".into(), (zone.right as f64).into());
        Ok(map)
    });

    // 3. Border Radius: theme.radius("lg")
    let s = sys.clone();
    module.set_native_fn("radius", move |key: &str| {
        Ok(s.border_radius.get(key).copied().map(|v| v as f64).unwrap_or(0.0))
    });

    // 4. Border Width: theme.border("thin")
    let s = sys.clone();
    module.set_native_fn("border", move |key: &str| {
        Ok(s.border_width.get(key).copied().map(|v| v as f64).unwrap_or(0.0))
    });

    // 5. Z-Index: theme.z("overlay")
    let s = sys.clone();
    module.set_native_fn("z", move |key: &str| {
        Ok(s.z_index.get(key).copied().map(|v| v as i64).unwrap_or(1))
    });

    module
}

pub fn register_rhai_api(engine: &mut Engine, loader: Arc<dyn AssetLoader>) {
    let theme_module = create_theme_api(DesignSystem::new());
    engine.register_static_module("theme", theme_module.into());

    // Randomness
    engine.register_fn("rand_float", |min: f64, max: f64| {
        use rand::Rng;
        rand::thread_rng().gen_range(min..max)
    });

    // 1. Director/Movie
    engine.register_type_with_name::<MovieHandle>("Movie");
    let loader_clone = loader.clone();

    // Overload 1: 3 args (Default Preview)
    let l1 = loader_clone.clone();
    engine.register_fn("new_director", move |w: i64, h: i64, fps: i64| {
        let director = Director::new(w as i32, h as i32, fps as u32, l1.clone(), RenderMode::Preview, None);
        MovieHandle { director: Arc::new(Mutex::new(director)) }
    });

    // Overload 2: 4 args (Config)
    let l2 = loader_clone.clone();
    engine.register_fn("new_director", move |w: i64, h: i64, fps: i64, config: rhai::Map| {
        let mode_str = config.get("mode").and_then(|v| v.clone().into_string().ok()).unwrap_or_else(|| "preview".to_string());
        let mode = match mode_str.as_str() {
            "export" => RenderMode::Export,
            _ => RenderMode::Preview,
        };
        let director = Director::new(w as i32, h as i32, fps as u32, l2.clone(), mode, None);
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

        let mut root = BoxNode::new();
        root.style.size = taffy::geometry::Size {
            width: Dimension::percent(1.0),
            height: Dimension::percent(1.0),
        };
        let id = d.add_node(Box::new(root));

        let item = TimelineItem {
            scene_root: id,
            start_time,
            duration,
            z_index: 0,
            audio_tracks: Vec::new(),
        };
        d.timeline.push(item);

        SceneHandle {
            director: movie.director.clone(),
            root_id: id,
            start_time,
            duration,
            audio_tracks: Vec::new(),
        }
    });

    engine.register_fn("add_transition", |movie: &mut MovieHandle, from: SceneHandle, to: SceneHandle, type_str: &str, duration: f64, easing_str: &str| {
        let mut d = movie.director.lock().unwrap();

        // Find indices
        let from_idx = d.timeline.iter().position(|i| i.scene_root == from.root_id);
        let to_idx = d.timeline.iter().position(|i| i.scene_root == to.root_id);

        if let (Some(f_idx), Some(t_idx)) = (from_idx, to_idx) {
             // Ripple Left Logic
             // We shift 'to' scene and all subsequent scenes (index >= t_idx) left by duration.

             for i in t_idx..d.timeline.len() {
                 d.timeline[i].start_time -= duration;

                 // Sync Audio
                 let audio_ids = d.timeline[i].audio_tracks.clone();
                 for track_id in audio_ids {
                     if let Some(track) = d.audio_mixer.get_track_mut(track_id) {
                         track.start_time -= duration;
                     }
                 }
             }

             let kind = match type_str {
                 "fade" => TransitionType::Fade,
                 "slide_left" | "slide-left" => TransitionType::SlideLeft,
                 "slide_right" | "slide-right" => TransitionType::SlideRight,
                 "wipe_left" | "wipe-left" => TransitionType::WipeLeft,
                 "wipe_right" | "wipe-right" => TransitionType::WipeRight,
                 "circle_open" | "circle-open" => TransitionType::CircleOpen,
                 _ => TransitionType::Fade,
             };

             let easing = match easing_str {
                 "linear" => EasingType::Linear,
                 "ease_in" => EasingType::EaseIn,
                 "ease_out" => EasingType::EaseOut,
                 "ease_in_out" => EasingType::EaseInOut,
                 _ => EasingType::Linear,
             };

             let start_time = d.timeline[t_idx].start_time;

             let transition = Transition {
                 from_scene_idx: f_idx,
                 to_scene_idx: t_idx,
                 start_time,
                 duration,
                 kind,
                 easing
             };

             d.transitions.push(transition);
        }
    });

    // 3. Elements
    engine.register_type_with_name::<NodeHandle>("Node");

    engine.register_fn("destroy", |node: &mut NodeHandle| {
        let mut d = node.director.lock().unwrap();
        d.destroy_node(node.id);
    });

    engine.register_fn("add_box", |parent: &mut NodeHandle, props: rhai::Map| {
        let mut d = parent.director.lock().unwrap();
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
        if let Some(v) = props.get("border_radius").and_then(|v| v.as_float().ok()) {
            box_node.border_radius = crate::animation::Animated::new(v as f32);
        }
        if let Some(v) = props.get("border_width").and_then(|v| v.as_float().ok()) {
            box_node.border_width = crate::animation::Animated::new(v as f32);
        }
        if let Some(c) = props.get("border_color") {
             if let Ok(s) = c.clone().into_string() {
                 if let Some(color) = parse_hex_color(&s) {
                     box_node.border_color = Some(crate::animation::Animated::new(color));
                 }
             }
        }
        if let Some(s) = props.get("overflow").and_then(|v| v.clone().into_string().ok()) {
            box_node.overflow = s;
        }

        parse_layout_style(&props, &mut box_node.style);

        let id = d.add_node(Box::new(box_node));
        d.add_child(parent.id, id);

        NodeHandle { director: parent.director.clone(), id }
    });

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
        if let Some(v) = props.get("border_radius").and_then(|v| v.as_float().ok()) {
            box_node.border_radius = crate::animation::Animated::new(v as f32);
        }
        if let Some(v) = props.get("border_width").and_then(|v| v.as_float().ok()) {
            box_node.border_width = crate::animation::Animated::new(v as f32);
        }
        if let Some(c) = props.get("border_color") {
             if let Ok(s) = c.clone().into_string() {
                 if let Some(color) = parse_hex_color(&s) {
                     box_node.border_color = Some(crate::animation::Animated::new(color));
                 }
             }
        }
        if let Some(s) = props.get("overflow").and_then(|v| v.clone().into_string().ok()) {
            box_node.overflow = s;
        }

        parse_layout_style(&props, &mut box_node.style);

        let id = d.add_node(Box::new(box_node));
        d.add_child(scene.root_id, id);

        NodeHandle { director: scene.director.clone(), id }
    });

    engine.register_fn("add_image", |parent: &mut NodeHandle, path: &str| {
         let mut d = parent.director.lock().unwrap();
         let bytes = d.asset_loader.load_bytes(path).unwrap_or(Vec::new());

         let img_node = ImageNode::new(bytes);
         let id = d.add_node(Box::new(img_node));
         d.add_child(parent.id, id);
         NodeHandle { director: parent.director.clone(), id }
    });

    engine.register_fn("add_lottie", |parent: &mut NodeHandle, path: &str| {
         let mut d = parent.director.lock().unwrap();
         let bytes = d.asset_loader.load_bytes(path).unwrap_or(Vec::new());

         match LottieNode::new(&bytes, HashMap::new()) {
             Ok(lottie_node) => {
                 let id = d.add_node(Box::new(lottie_node));
                 d.add_child(parent.id, id);
                 NodeHandle { director: parent.director.clone(), id }
             }
             Err(e) => {
                 eprintln!("Failed to load lottie: {}", e);
                 let id = d.add_node(Box::new(BoxNode::new()));
                 NodeHandle { director: parent.director.clone(), id }
             }
         }
    });

    engine.register_fn("add_lottie", |parent: &mut NodeHandle, path: &str, props: rhai::Map| {
         let mut d = parent.director.lock().unwrap();
         let bytes = d.asset_loader.load_bytes(path).unwrap_or(Vec::new());

         let mut assets_map = HashMap::new();
         if let Some(assets_prop) = props.get("assets").and_then(|v| v.clone().try_cast::<Map>()) {
             for (key, val) in assets_prop {
                 if let Ok(asset_path) = val.into_string() {
                      let asset_bytes = d.asset_loader.load_bytes(&asset_path).unwrap_or(Vec::new());
                      let data = skia_safe::Data::new_copy(&asset_bytes);
                      if let Some(image) = skia_safe::Image::from_encoded(data) {
                          assets_map.insert(key.to_string(), image);
                      }
                 }
             }
         }

         match LottieNode::new(&bytes, assets_map) {
             Ok(mut lottie_node) => {
                 parse_layout_style(&props, &mut lottie_node.style);
                 if let Some(v) = props.get("speed").and_then(|v| v.as_float().ok()) {
                     lottie_node.speed = v as f32;
                 }
                 if let Some(v) = props.get("loop").and_then(|v| v.as_bool().ok()) {
                     lottie_node.loop_anim = v;
                 }

                 let id = d.add_node(Box::new(lottie_node));
                 d.add_child(parent.id, id);
                 NodeHandle { director: parent.director.clone(), id }
             }
             Err(e) => {
                 eprintln!("Failed to load lottie: {}", e);
                 let id = d.add_node(Box::new(BoxNode::new()));
                 NodeHandle { director: parent.director.clone(), id }
             }
         }
    });

    engine.register_fn("add_svg", |scene: &mut SceneHandle, path: &str| {
         let mut d = scene.director.lock().unwrap();
         let bytes = d.asset_loader.load_bytes(path).unwrap_or(Vec::new());

         let vec_node = VectorNode::new(&bytes);
         let id = d.add_node(Box::new(vec_node));
         d.add_child(scene.root_id, id);
         NodeHandle { director: scene.director.clone(), id }
    });

    engine.register_fn("add_svg", |scene: &mut SceneHandle, path: &str, props: rhai::Map| {
         let mut d = scene.director.lock().unwrap();
         let bytes = d.asset_loader.load_bytes(path).unwrap_or(Vec::new());

         let mut vec_node = VectorNode::new(&bytes);
         parse_layout_style(&props, &mut vec_node.style);

         let id = d.add_node(Box::new(vec_node));
         d.add_child(scene.root_id, id);
         NodeHandle { director: scene.director.clone(), id }
    });

    engine.register_fn("add_image", |parent: &mut NodeHandle, path: &str, props: rhai::Map| {
         let mut d = parent.director.lock().unwrap();
         let bytes = d.asset_loader.load_bytes(path).unwrap_or(Vec::new());

         let mut img_node = ImageNode::new(bytes);
         parse_layout_style(&props, &mut img_node.style);

         let id = d.add_node(Box::new(img_node));
         d.add_child(parent.id, id);
         NodeHandle { director: parent.director.clone(), id }
    });

    engine.register_fn("add_svg", |parent: &mut NodeHandle, path: &str| {
         let mut d = parent.director.lock().unwrap();
         let bytes = d.asset_loader.load_bytes(path).unwrap_or(Vec::new());

         let vec_node = VectorNode::new(&bytes);
         let id = d.add_node(Box::new(vec_node));
         d.add_child(parent.id, id);
         NodeHandle { director: parent.director.clone(), id }
    });

    engine.register_fn("add_svg", |parent: &mut NodeHandle, path: &str, props: rhai::Map| {
         let mut d = parent.director.lock().unwrap();
         let bytes = d.asset_loader.load_bytes(path).unwrap_or(Vec::new());

         let mut vec_node = VectorNode::new(&bytes);
         parse_layout_style(&props, &mut vec_node.style);

         let id = d.add_node(Box::new(vec_node));
         d.add_child(parent.id, id);
         NodeHandle { director: parent.director.clone(), id }
    });

    engine.register_fn("add_video", |parent: &mut NodeHandle, path: &str| {
         let mut d = parent.director.lock().unwrap();
         let mode = d.render_mode;
         let p = std::path::Path::new(path);

         let source = if p.exists() && p.is_file() {
             VideoSource::Path(p.to_path_buf())
         } else {
             let bytes = d.asset_loader.load_bytes(path).unwrap_or(Vec::new());
             VideoSource::Bytes(bytes)
         };

         let vid_node = VideoNode::new(source, mode);
         let id = d.add_node(Box::new(vid_node));
         d.add_child(parent.id, id);
         NodeHandle { director: parent.director.clone(), id }
    });

    engine.register_fn("add_video", |parent: &mut NodeHandle, path: &str, props: rhai::Map| {
         let mut d = parent.director.lock().unwrap();
         let mode = d.render_mode;
         let p = std::path::Path::new(path);

         let source = if p.exists() && p.is_file() {
             VideoSource::Path(p.to_path_buf())
         } else {
             let bytes = d.asset_loader.load_bytes(path).unwrap_or(Vec::new());
             VideoSource::Bytes(bytes)
         };

         let mut vid_node = VideoNode::new(source, mode);
         parse_layout_style(&props, &mut vid_node.style);

         let id = d.add_node(Box::new(vid_node));
         d.add_child(parent.id, id);
         NodeHandle { director: parent.director.clone(), id }
    });

    engine.register_fn("add_text", |parent: &mut NodeHandle, props: rhai::Map| {
         let mut d = parent.director.lock().unwrap();
         let fs = d.font_system.clone();
         let sc = d.swash_cache.clone();

         let spans = if let Some(c) = props.get("content") {
             parse_spans_from_dynamic(c.clone())
         } else {
             Vec::new()
         };

         let mut text_node = TextNode::new(spans, fs, sc);

         parse_layout_style(&props, &mut text_node.style);

         if let Some(s) = props.get("fit").and_then(|v| v.clone().into_string().ok()) {
             text_node.fit_mode = match s.as_str() {
                 "shrink" => TextFit::Shrink,
                 _ => TextFit::None
             };
         }
         if let Some(v) = props.get("min_size").and_then(|v| v.as_float().ok()) {
             text_node.min_size = v as f32;
         }
         if let Some(v) = props.get("max_size").and_then(|v| v.as_float().ok()) {
             text_node.max_size = v as f32;
         }

         // Shadow parsing
         let mut has_shadow = false;
         let mut shadow = TextShadow {
             color: Color::BLACK,
             blur: 0.0,
             offset: (0.0, 0.0)
         };
         if let Some(c) = props.get("text_shadow_color").and_then(|v| v.clone().into_string().ok()) {
             if let Some(col) = parse_hex_color(&c) {
                 shadow.color = col;
                 has_shadow = true;
             }
         }
         if let Some(v) = props.get("text_shadow_blur").and_then(|v| v.as_float().ok()) {
             shadow.blur = v as f32;
             has_shadow = true;
         }
         if let Some(v) = props.get("text_shadow_x").and_then(|v| v.as_float().ok()) {
             shadow.offset.0 = v as f32;
             has_shadow = true;
         }
         if let Some(v) = props.get("text_shadow_y").and_then(|v| v.as_float().ok()) {
             shadow.offset.1 = v as f32;
             has_shadow = true;
         }
         if has_shadow {
             text_node.shadow = Some(shadow);
         }

         let id = d.add_node(Box::new(text_node));
         d.add_child(parent.id, id);

         NodeHandle { director: parent.director.clone(), id }
    });

    engine.register_fn("add_text", |scene: &mut SceneHandle, props: rhai::Map| {
         let mut d = scene.director.lock().unwrap();
         let fs = d.font_system.clone();
         let sc = d.swash_cache.clone();

         let spans = if let Some(c) = props.get("content") {
             parse_spans_from_dynamic(c.clone())
         } else {
             Vec::new()
         };

         let mut text_node = TextNode::new(spans, fs, sc);

         parse_layout_style(&props, &mut text_node.style);

         if let Some(s) = props.get("fit").and_then(|v| v.clone().into_string().ok()) {
             text_node.fit_mode = match s.as_str() {
                 "shrink" => TextFit::Shrink,
                 _ => TextFit::None
             };
         }
         if let Some(v) = props.get("min_size").and_then(|v| v.as_float().ok()) {
             text_node.min_size = v as f32;
         }
         if let Some(v) = props.get("max_size").and_then(|v| v.as_float().ok()) {
             text_node.max_size = v as f32;
         }

         // Shadow parsing
         let mut has_shadow = false;
         let mut shadow = TextShadow {
             color: Color::BLACK,
             blur: 0.0,
             offset: (0.0, 0.0)
         };
         if let Some(c) = props.get("text_shadow_color").and_then(|v| v.clone().into_string().ok()) {
             if let Some(col) = parse_hex_color(&c) {
                 shadow.color = col;
                 has_shadow = true;
             }
         }
         if let Some(v) = props.get("text_shadow_blur").and_then(|v| v.as_float().ok()) {
             shadow.blur = v as f32;
             has_shadow = true;
         }
         if let Some(v) = props.get("text_shadow_x").and_then(|v| v.as_float().ok()) {
             shadow.offset.0 = v as f32;
             has_shadow = true;
         }
         if let Some(v) = props.get("text_shadow_y").and_then(|v| v.as_float().ok()) {
             shadow.offset.1 = v as f32;
             has_shadow = true;
         }
         if has_shadow {
             text_node.shadow = Some(shadow);
         }

         let id = d.add_node(Box::new(text_node));
         d.add_child(scene.root_id, id);

         NodeHandle { director: scene.director.clone(), id }
    });

    engine.register_fn("add_composition", |scene: &mut SceneHandle, comp_def: MovieHandle| {
         // Cycle Detection
         if Arc::ptr_eq(&scene.director, &comp_def.director) {
             eprintln!("Error: Cycle detected. A composition cannot contain itself.");
             let mut d = scene.director.lock().unwrap();
             let id = d.add_node(Box::new(BoxNode::new()));
             return NodeHandle { director: scene.director.clone(), id };
         }

         let mut inner_director = comp_def.director.lock().unwrap().clone();

         // Share resources from parent
         {
             let parent = scene.director.lock().unwrap();
             inner_director.font_system = parent.font_system.clone();
             inner_director.swash_cache = parent.swash_cache.clone();
             inner_director.shader_cache = parent.shader_cache.clone();
             inner_director.asset_loader = parent.asset_loader.clone();
         }

         let comp_node = CompositionNode {
             internal_director: Mutex::new(inner_director),
             start_offset: 0.0,
             surface_cache: Mutex::new(None),
             style: Style::default(),
         };

         let mut d = scene.director.lock().unwrap();
         let id = d.add_node(Box::new(comp_node));
         d.add_child(scene.root_id, id);

         NodeHandle { director: scene.director.clone(), id }
    });

    engine.register_fn("add_composition", |scene: &mut SceneHandle, comp_def: MovieHandle, props: rhai::Map| {
         // Cycle Detection
         if Arc::ptr_eq(&scene.director, &comp_def.director) {
             eprintln!("Error: Cycle detected. A composition cannot contain itself.");
             let mut d = scene.director.lock().unwrap();
             let id = d.add_node(Box::new(BoxNode::new()));
             return NodeHandle { director: scene.director.clone(), id };
         }

         let mut inner_director = comp_def.director.lock().unwrap().clone();

         // Share resources from parent
         {
             let parent = scene.director.lock().unwrap();
             inner_director.font_system = parent.font_system.clone();
             inner_director.swash_cache = parent.swash_cache.clone();
             inner_director.shader_cache = parent.shader_cache.clone();
             inner_director.asset_loader = parent.asset_loader.clone();
         }

         let mut style = Style::default();
         parse_layout_style(&props, &mut style);

         let comp_node = CompositionNode {
             internal_director: Mutex::new(inner_director),
             start_offset: 0.0,
             surface_cache: Mutex::new(None),
             style,
         };

         let mut d = scene.director.lock().unwrap();
         let id = d.add_node(Box::new(comp_node));
         d.add_child(scene.root_id, id);

         NodeHandle { director: scene.director.clone(), id }
    });

    engine.register_fn("set_content", |node: &mut NodeHandle, content: rhai::Dynamic| {
         let spans = parse_spans_from_dynamic(content);
         let mut d = node.director.lock().unwrap();
         if let Some(n) = d.get_node_mut(node.id) {
             n.element.set_rich_text(spans);
             // Trigger layout dirty flag? TextNode handles it in set_rich_text
         }
    });

    engine.register_fn("set_style", |node: &mut NodeHandle, style: rhai::Map| {
         let mut d = node.director.lock().unwrap();
         if let Some(n) = d.get_node_mut(node.id) {
             let mut layout_style = n.element.layout_style();
             parse_layout_style(&style, &mut layout_style);
             n.element.set_layout_style(layout_style);
             n.dirty_style = true;

             n.element.modify_text_spans(&|spans| {
                 for span in spans {
                     parse_text_style(&style, span);
                 }
             });
         }
    });

    engine.register_fn("set_pivot", |node: &mut NodeHandle, x: f64, y: f64| {
         let mut d = node.director.lock().unwrap();
         if let Some(n) = d.get_node_mut(node.id) {
             n.transform.pivot_x = x as f32;
             n.transform.pivot_y = y as f32;
         }
    });

    engine.register_fn("set_mask", |node: &mut NodeHandle, mask: NodeHandle| {
        let mut d = node.director.lock().unwrap();

        // 1. Get the mask node's current parent
        let old_parent = if let Some(m_node) = d.get_node(mask.id) {
            m_node.parent
        } else {
            None
        };

        // 2. Remove mask from old parent's children list
        if let Some(p_id) = old_parent {
            d.remove_child(p_id, mask.id);
        }

        // 3. Set mask's parent to the new owner (node.id)
        if let Some(m_node) = d.get_node_mut(mask.id) {
            m_node.parent = Some(node.id);
        }

        // 4. Assign mask_node to owner
        if let Some(n) = d.get_node_mut(node.id) {
            n.mask_node = Some(mask.id);
        }
    });

    engine.register_fn("set_blend_mode", |node: &mut NodeHandle, mode_str: &str| {
         let mut d = node.director.lock().unwrap();
         let mode = match mode_str {
             "clear" => skia_safe::BlendMode::Clear,
             "src" => skia_safe::BlendMode::Src,
             "dst" => skia_safe::BlendMode::Dst,
             "src_over" | "src-over" | "normal" => skia_safe::BlendMode::SrcOver,
             "dst_over" | "dst-over" => skia_safe::BlendMode::DstOver,
             "src_in" | "src-in" => skia_safe::BlendMode::SrcIn,
             "dst_in" | "dst-in" => skia_safe::BlendMode::DstIn,
             "src_out" | "src-out" => skia_safe::BlendMode::SrcOut,
             "dst_out" | "dst-out" => skia_safe::BlendMode::DstOut,
             "src_atop" | "src-atop" => skia_safe::BlendMode::SrcATop,
             "dst_atop" | "dst-atop" => skia_safe::BlendMode::DstATop,
             "xor" => skia_safe::BlendMode::Xor,
             "plus" | "add" => skia_safe::BlendMode::Plus,
             "modulate" => skia_safe::BlendMode::Modulate,
             "screen" => skia_safe::BlendMode::Screen,
             "overlay" => skia_safe::BlendMode::Overlay,
             "darken" => skia_safe::BlendMode::Darken,
             "lighten" => skia_safe::BlendMode::Lighten,
             "color_dodge" | "color-dodge" => skia_safe::BlendMode::ColorDodge,
             "color_burn" | "color-burn" => skia_safe::BlendMode::ColorBurn,
             "hard_light" | "hard-light" => skia_safe::BlendMode::HardLight,
             "soft_light" | "soft-light" => skia_safe::BlendMode::SoftLight,
             "difference" => skia_safe::BlendMode::Difference,
             "exclusion" => skia_safe::BlendMode::Exclusion,
             "multiply" => skia_safe::BlendMode::Multiply,
             "hue" => skia_safe::BlendMode::Hue,
             "saturation" => skia_safe::BlendMode::Saturation,
             "color" => skia_safe::BlendMode::Color,
             "luminosity" => skia_safe::BlendMode::Luminosity,
             _ => skia_safe::BlendMode::SrcOver,
         };
         if let Some(n) = d.get_node_mut(node.id) {
             n.blend_mode = mode;
         }
    });

    engine.register_fn("add_animator", |node: &mut NodeHandle, start_idx: i64, end_idx: i64, prop: &str, start: f64, end: f64, dur: f64, ease: &str| {
        let mut d = node.director.lock().unwrap();
        if let Some(n) = d.get_node_mut(node.id) {
             n.element.add_text_animator(
                 start_idx as usize,
                 end_idx as usize,
                 prop.to_string(),
                 start as f32,
                 end as f32,
                 dur,
                 ease
             );
        }
    });

    engine.register_fn("animate", |node: &mut NodeHandle, prop: &str, start: f64, end: f64, dur: f64, ease: &str| {
        let mut d = node.director.lock().unwrap();
        if let Some(n) = d.get_node_mut(node.id) {
             let ease_fn = match ease {
                 "linear" => EasingType::Linear,
                 "ease_in" => EasingType::EaseIn,
                 "ease_out" => EasingType::EaseOut,
                 "ease_in_out" => EasingType::EaseInOut,
                 "bounce_out" => EasingType::BounceOut,
                 _ => EasingType::Linear,
             };

             match prop {
                 "scale" => {
                     n.transform.scale_x.add_segment(start as f32, end as f32, dur, ease_fn);
                     n.transform.scale_y.add_segment(start as f32, end as f32, dur, ease_fn);
                 },
                 "scale_x" => n.transform.scale_x.add_segment(start as f32, end as f32, dur, ease_fn),
                 "scale_y" => n.transform.scale_y.add_segment(start as f32, end as f32, dur, ease_fn),
                 "rotation" => n.transform.rotation.add_segment(start as f32, end as f32, dur, ease_fn),
                 "skew_x" => n.transform.skew_x.add_segment(start as f32, end as f32, dur, ease_fn),
                 "skew_y" => n.transform.skew_y.add_segment(start as f32, end as f32, dur, ease_fn),
                 "translate_x" | "x" => n.transform.translate_x.add_segment(start as f32, end as f32, dur, ease_fn),
                 "translate_y" | "y" => n.transform.translate_y.add_segment(start as f32, end as f32, dur, ease_fn),
                 _ => {
                     n.element.animate_property(prop, start as f32, end as f32, dur, ease);
                 }
             }
        }
    });

    engine.register_fn("animate", |node: &mut NodeHandle, prop: &str, end: f64, config: rhai::Map| {
        let mut d = node.director.lock().unwrap();
        let spring_conf = parse_spring_config(&config);

        if let Some(n) = d.get_node_mut(node.id) {
             match prop {
                 "scale" => {
                     n.transform.scale_x.add_spring(end as f32, spring_conf);
                     n.transform.scale_y.add_spring(end as f32, spring_conf);
                 },
                 "scale_x" => n.transform.scale_x.add_spring(end as f32, spring_conf),
                 "scale_y" => n.transform.scale_y.add_spring(end as f32, spring_conf),
                 "rotation" => n.transform.rotation.add_spring(end as f32, spring_conf),
                 "skew_x" => n.transform.skew_x.add_spring(end as f32, spring_conf),
                 "skew_y" => n.transform.skew_y.add_spring(end as f32, spring_conf),
                 "translate_x" | "x" => n.transform.translate_x.add_spring(end as f32, spring_conf),
                 "translate_y" | "y" => n.transform.translate_y.add_spring(end as f32, spring_conf),
                 _ => {
                     n.element.animate_property_spring(prop, None, end as f32, spring_conf);
                 }
             }
        }
    });

    engine.register_fn("animate", |node: &mut NodeHandle, prop: &str, start: f64, end: f64, config: rhai::Map| {
        let mut d = node.director.lock().unwrap();
        let spring_conf = parse_spring_config(&config);

        if let Some(n) = d.get_node_mut(node.id) {
             match prop {
                 "scale" => {
                     n.transform.scale_x.add_spring_with_start(start as f32, end as f32, spring_conf);
                     n.transform.scale_y.add_spring_with_start(start as f32, end as f32, spring_conf);
                 },
                 "scale_x" => n.transform.scale_x.add_spring_with_start(start as f32, end as f32, spring_conf),
                 "scale_y" => n.transform.scale_y.add_spring_with_start(start as f32, end as f32, spring_conf),
                 "rotation" => n.transform.rotation.add_spring_with_start(start as f32, end as f32, spring_conf),
                 "skew_x" => n.transform.skew_x.add_spring_with_start(start as f32, end as f32, spring_conf),
                 "skew_y" => n.transform.skew_y.add_spring_with_start(start as f32, end as f32, spring_conf),
                 "translate_x" | "x" => n.transform.translate_x.add_spring_with_start(start as f32, end as f32, spring_conf),
                 "translate_y" | "y" => n.transform.translate_y.add_spring_with_start(start as f32, end as f32, spring_conf),
                 _ => {
                     n.element.animate_property_spring(prop, Some(start as f32), end as f32, spring_conf);
                 }
             }
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
             n.element.animate_property("blur", val as f32, val as f32, 0.0, "linear");
         }
    });

    // Audio
    engine.register_type_with_name::<AudioTrackHandle>("AudioTrack");

    engine.register_fn("add_audio", |movie: &mut MovieHandle, path: &str| {
        let mut d = movie.director.lock().unwrap();
        let bytes = d.asset_loader.load_bytes(path).unwrap_or(Vec::new());
        let samples = crate::audio::load_audio_bytes(&bytes, d.audio_mixer.sample_rate)
            .unwrap_or_else(|e| { eprintln!("Audio error: {}", e); Vec::new() });

        let id = d.add_global_audio(samples);
        AudioTrackHandle { director: movie.director.clone(), id }
    });

    engine.register_fn("add_audio", |scene: &mut SceneHandle, path: &str| {
        let mut d = scene.director.lock().unwrap();
        let bytes = d.asset_loader.load_bytes(path).unwrap_or(Vec::new());
        let samples = crate::audio::load_audio_bytes(&bytes, d.audio_mixer.sample_rate)
            .unwrap_or_else(|e| { eprintln!("Audio error: {}", e); Vec::new() });

        let id = d.add_scene_audio(samples, scene.start_time, scene.duration);

        // Update SceneHandle tracking
        scene.audio_tracks.push(id);

        // Update Director TimelineItem tracking
        if let Some(item) = d.timeline.iter_mut().find(|i| i.scene_root == scene.root_id) {
            item.audio_tracks.push(id);
        }

        AudioTrackHandle { director: scene.director.clone(), id }
    });

    engine.register_fn("animate_volume", |track: &mut AudioTrackHandle, start: f64, end: f64, dur: f64, ease: &str| {
        let mut d = track.director.lock().unwrap();
        if let Some(t) = d.audio_mixer.get_track_mut(track.id) {
             let ease_fn = match ease {
                 "linear" => EasingType::Linear,
                 "ease_in" => EasingType::EaseIn,
                 "ease_out" => EasingType::EaseOut,
                 "ease_in_out" => EasingType::EaseInOut,
                 _ => EasingType::Linear,
             };
             t.volume.add_segment(start as f32, end as f32, dur, ease_fn);
        }
    });

    // Effects
    engine.register_fn("apply_effect", |node: &mut NodeHandle, name: &str| -> NodeHandle {
        let mut d = node.director.lock().unwrap();
        let effect = match name {
            "grayscale" => Some(EffectType::ColorMatrix(vec![
                0.2126, 0.7152, 0.0722, 0.0, 0.0,
                0.2126, 0.7152, 0.0722, 0.0, 0.0,
                0.2126, 0.7152, 0.0722, 0.0, 0.0,
                0.0,    0.0,    0.0,    1.0, 0.0,
            ])),
            "sepia" => Some(EffectType::ColorMatrix(vec![
                0.393, 0.769, 0.189, 0.0, 0.0,
                0.349, 0.686, 0.168, 0.0, 0.0,
                0.272, 0.534, 0.131, 0.0, 0.0,
                0.0,   0.0,   0.0,   1.0, 0.0,
            ])),
            "invert" => Some(EffectType::ColorMatrix(vec![
                -1.0,  0.0,  0.0, 0.0, 1.0,
                0.0, -1.0,  0.0, 0.0, 1.0,
                0.0,  0.0, -1.0, 0.0, 1.0,
                0.0,  0.0,  0.0, 1.0, 0.0,
            ])),
            _ => None
        };

        if let Some(eff) = effect {
            let id = apply_effect_to_node(&mut d, node.id, eff);
            NodeHandle { director: node.director.clone(), id }
        } else {
            NodeHandle { director: node.director.clone(), id: node.id }
        }
    });

    engine.register_fn("apply_effect", |node: &mut NodeHandle, name: &str, val: f64| -> NodeHandle {
        let mut d = node.director.lock().unwrap();
        let val = val as f32;
        let effect = match name {
            "contrast" => {
                let t = (1.0 - val) / 2.0;
                Some(EffectType::ColorMatrix(vec![
                    val, 0.0, 0.0, 0.0, t,
                    0.0, val, 0.0, 0.0, t,
                    0.0, 0.0, val, 0.0, t,
                    0.0, 0.0, 0.0, 1.0, 0.0,
                ]))
            },
            "brightness" => {
                Some(EffectType::ColorMatrix(vec![
                    1.0, 0.0, 0.0, 0.0, val,
                    0.0, 1.0, 0.0, 0.0, val,
                    0.0, 0.0, 1.0, 0.0, val,
                    0.0, 0.0, 0.0, 1.0, 0.0,
                ]))
            },
            "blur" => {
                Some(EffectType::Blur(Animated::new(val)))
            },
            _ => None
        };

        if let Some(eff) = effect {
            let id = apply_effect_to_node(&mut d, node.id, eff);
            NodeHandle { director: node.director.clone(), id }
        } else {
             NodeHandle { director: node.director.clone(), id: node.id }
        }
    });

    engine.register_fn("apply_effect", |node: &mut NodeHandle, name: &str, map: rhai::Map| -> NodeHandle {
        let mut d = node.director.lock().unwrap();
        if name == "shader" {
             if let Some(code) = map.get("code").and_then(|v| v.clone().into_string().ok()) {
                 let mut uniforms = HashMap::new();
                 if let Some(u_map) = map.get("uniforms").and_then(|v| v.clone().try_cast::<Map>()) {
                     for (k, v) in u_map {
                         if let Ok(f) = v.as_float() {
                             uniforms.insert(k.to_string(), Animated::new(f as f32));
                         }
                     }
                 }

                 let effect = EffectType::RuntimeShader {
                     sksl: code,
                     uniforms
                 };
                 let id = apply_effect_to_node(&mut d, node.id, effect);
                 return NodeHandle { director: node.director.clone(), id };
             }
        }
        NodeHandle { director: node.director.clone(), id: node.id }
    });
}
