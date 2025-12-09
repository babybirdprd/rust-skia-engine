use director_core::{Director, AssetLoader, Element};
use director_core::video_wrapper::RenderMode;
use director_core::node::{BoxNode, TextNode, ImageNode};
use director_core::element::{TextSpan};
use director_core::types::Color;
use director_core::types::NodeId;
use director_core::animation::EasingType;
use director_schema::{MovieRequest, Node, NodeKind, StyleMap, TransformMap, Animation};
use std::sync::Arc;
use taffy::style::{Style, Dimension, FlexDirection, JustifyContent, AlignItems};

/// Converts a Schema Request into a runnable Director instance.
pub fn load_movie(request: MovieRequest, loader: Arc<dyn AssetLoader>) -> Director {
    let mut director = Director::new(
        request.width as i32,
        request.height as i32,
        request.fps,
        loader,
        RenderMode::Export,
        None
    );

    for scene_data in request.scenes {
        // Build the scene graph for this scene
        let root_id = build_node_recursive(&mut director, &scene_data.root);

        // Calculate start time based on previous scenes
        let start_time = director.timeline.last()
            .map(|i| i.start_time + i.duration)
            .unwrap_or(0.0);

        // Add to timeline
        director.timeline.push(director_core::director::TimelineItem {
            scene_root: root_id,
            start_time,
            duration: scene_data.duration_secs,
            z_index: 0,
            audio_tracks: vec![],
        });
    }

    director
}

fn build_node_recursive(director: &mut Director, node_def: &Node) -> NodeId {
    // 1. Create Element based on NodeKind
    let element: Box<dyn Element> = match &node_def.kind {
        NodeKind::Box { border_radius } => {
            let mut b = BoxNode::new();
            b.border_radius = director_core::animation::Animated::new(*border_radius);
            Box::new(b)
        },
        NodeKind::Text { content, font_size } => {
            // TextNode needs access to the AssetManager's font system
            let fs = director.assets.font_system.clone();
            let sc = director.assets.swash_cache.clone();
            let tc = director.assets.typeface_cache.clone();

            let span = TextSpan {
                text: content.clone(),
                font_size: Some(*font_size),
                color: Some(Color::WHITE), // Default to white, overridden by style later
                ..Default::default()
            };

            let t = TextNode::new(vec![span], fs, sc, tc);
            Box::new(t)
        },
        NodeKind::Image { src } => {
            // Load bytes immediately (blocking for now)
            let bytes = director.assets.loader.load_bytes(src).unwrap_or_default();
            let img = ImageNode::new(bytes);
            Box::new(img)
        },
        _ => Box::new(BoxNode::new()), // Fallback
    };

    // 2. Add to Scene Graph
    let id = director.scene.add_node(element);

    // 3. Apply Common Properties (Style, Transform, Animation) via Node access
    if let Some(node) = director.scene.get_node_mut(id) {
        // Apply Style
        let mut style = node.element.layout_style();
        apply_style_map(&mut style, &node_def.style);
        node.element.set_layout_style(style);

        // Apply Background Color if it's a BoxNode
        if let Some(bg) = node_def.style.bg_color {
            if let Some(box_node) = node.element.as_any_mut().downcast_mut::<BoxNode>() {
                box_node.bg_color = Some(director_core::animation::Animated::new(bg));
            }
        }

        // Apply Transform
        apply_transform_map(&mut node.transform, &node_def.transform);

        // Apply Animations (Must be done after layout/transform setup as requested)
        apply_animations(&mut node.element, &node_def.animations);
    }

    // 4. Recurse Children
    for child_def in &node_def.children {
        let child_id = build_node_recursive(director, child_def);
        director.scene.add_child(id, child_id);
    }

    id
}

fn apply_animations(element: &mut Box<dyn Element>, animations: &[Animation]) {
    for anim in animations {
        let easing_str = match anim.easing {
            EasingType::Linear => "linear",
            EasingType::EaseIn => "ease_in",
            EasingType::EaseOut => "ease_out",
            EasingType::EaseInOut => "ease_in_out",
            EasingType::BounceOut => "bounce_out",
        };

        let start_val = anim.start.unwrap_or(0.0); // Fallback if start not provided

        element.animate_property(
            &anim.property,
            start_val,
            anim.end,
            anim.duration,
            easing_str
        );
    }
}

fn apply_style_map(style: &mut Style, map: &StyleMap) {
    if let Some(w) = &map.width { style.size.width = parse_dim(w); }
    if let Some(h) = &map.height { style.size.height = parse_dim(h); }

    if let Some(d) = &map.flex_direction {
        style.flex_direction = match d.as_str() {
            "column" => FlexDirection::Column,
            "row" | _ => FlexDirection::Row,
        };
    }

    if let Some(j) = &map.justify_content {
        style.justify_content = match j.as_str() {
            "center" => Some(JustifyContent::Center),
            "space_between" => Some(JustifyContent::SpaceBetween),
            "flex_end" => Some(JustifyContent::FlexEnd),
            "flex_start" | _ => Some(JustifyContent::FlexStart),
        };
    }

    if let Some(a) = &map.align_items {
        style.align_items = match a.as_str() {
            "center" => Some(AlignItems::Center),
            "stretch" => Some(AlignItems::Stretch),
            "flex_end" => Some(AlignItems::FlexEnd),
            "flex_start" | _ => Some(AlignItems::FlexStart),
        };
    }

    // Padding
    if let Some(p) = map.padding {
        let d = taffy::style::LengthPercentage::length(p);
        style.padding = taffy::geometry::Rect { left: d, right: d, top: d, bottom: d };
    }

    // Margin
    if let Some(m) = map.margin {
        let d = taffy::style::LengthPercentageAuto::length(m);
        style.margin = taffy::geometry::Rect { left: d, right: d, top: d, bottom: d };
    }
}

fn apply_transform_map(transform: &mut director_core::types::Transform, map: &TransformMap) {
    if let Some(v) = map.x { transform.translate_x = director_core::animation::Animated::new(v); }
    if let Some(v) = map.y { transform.translate_y = director_core::animation::Animated::new(v); }
    if let Some(v) = map.rotation { transform.rotation = director_core::animation::Animated::new(v); }
    if let Some(v) = map.scale {
        transform.scale_x = director_core::animation::Animated::new(v);
        transform.scale_y = director_core::animation::Animated::new(v);
    }
    if let Some(v) = map.pivot_x { transform.pivot_x = v; }
    if let Some(v) = map.pivot_y { transform.pivot_y = v; }
}

fn parse_dim(val: &str) -> Dimension {
    if val == "auto" { Dimension::auto() }
    else if val.ends_with("%") {
        let f = val.trim_end_matches("%").parse::<f32>().unwrap_or(0.0);
        Dimension::percent(f / 100.0)
    } else {
        let f = val.parse::<f32>().unwrap_or(0.0);
        Dimension::length(f)
    }
}
