use director_core::animation::{Animated, EasingType};
use director_core::element::TextSpan;
use director_core::node::video_node::VideoSource;
use director_core::node::{BoxNode, ImageNode, LottieNode, TextNode, VectorNode, VideoNode};
use director_core::node::{EffectNode, EffectType};
use director_core::systems::transitions::{Transition, TransitionType as CoreTransitionType};
use director_core::types::{Color, NodeId, ObjectFit};
use director_core::video_wrapper::RenderMode;
use director_core::{AssetLoader, Director, Element};
use director_schema::{
    Animation, EffectConfig, MovieRequest, Node, NodeKind, StyleMap, TransformMap, TransitionType,
};
use std::collections::HashMap;
use std::sync::Arc;
use taffy::geometry::{Line, Rect, Size};
use taffy::prelude::*;
use taffy::style::{
    AlignItems, Dimension, Display, FlexDirection, GridPlacement, GridTemplateComponent,
    JustifyContent, LengthPercentage, LengthPercentageAuto, Position, Style,
};

/// Converts a Schema Request into a runnable Director instance.
pub fn load_movie(request: MovieRequest, loader: Arc<dyn AssetLoader>) -> Director {
    let mut director = Director::new(
        request.width as i32,
        request.height as i32,
        request.fps,
        loader,
        RenderMode::Export,
        None,
    );

    // Build transition list from scene configs
    let mut scene_end_times = Vec::new();
    let mut cumulative_time = 0.0;

    for scene_data in &request.scenes {
        // Build the scene graph for this scene
        let root_id = build_node_recursive(&mut director, &scene_data.root);

        // Calculate start time based on previous scenes
        let start_time = cumulative_time;
        cumulative_time += scene_data.duration_secs;
        scene_end_times.push(start_time + scene_data.duration_secs);

        // Add to timeline
        director
            .timeline
            .push(director_core::director::TimelineItem {
                scene_root: root_id,
                start_time,
                duration: scene_data.duration_secs,
                z_index: 0,
                audio_tracks: vec![],
            });
    }

    // Wire up transitions between scenes
    for (i, scene_data) in request.scenes.iter().enumerate() {
        if let Some(trans) = &scene_data.transition {
            if i + 1 < request.scenes.len() {
                let transition_start = scene_end_times[i] - trans.duration;
                director.transitions.push(Transition {
                    from_scene_idx: i,
                    to_scene_idx: i + 1,
                    start_time: transition_start,
                    duration: trans.duration,
                    kind: convert_transition_type(&trans.kind),
                    easing: trans.easing.clone(),
                });
            }
        }
    }

    director
}

/// Converts schema TransitionType to core TransitionType
fn convert_transition_type(kind: &TransitionType) -> CoreTransitionType {
    match kind {
        TransitionType::Fade => CoreTransitionType::Fade,
        TransitionType::SlideLeft => CoreTransitionType::SlideLeft,
        TransitionType::SlideRight => CoreTransitionType::SlideRight,
        TransitionType::WipeLeft => CoreTransitionType::WipeLeft,
        TransitionType::WipeRight => CoreTransitionType::WipeRight,
        TransitionType::CircleOpen => CoreTransitionType::CircleOpen,
    }
}

fn build_node_recursive(director: &mut Director, node_def: &Node) -> NodeId {
    // 1. Create Element based on NodeKind
    let mut element: Box<dyn Element> = match &node_def.kind {
        NodeKind::Box { border_radius } => {
            let mut b = BoxNode::new();
            b.border_radius = Animated::new(*border_radius);
            Box::new(b)
        }
        NodeKind::Text {
            content,
            font_size,
            animators,
        } => {
            // TextNode needs access to the AssetManager's font system
            let fc = director.assets.font_collection.clone();

            let span = TextSpan {
                text: content.clone(),
                font_size: Some(*font_size),
                color: Some(Color::WHITE), // Default to white, overridden by style later
                ..Default::default()
            };

            let mut t = TextNode::new(vec![span], fc);

            // Apply text animators for kinetic typography
            for anim in animators {
                let easing_str = easing_to_str(&anim.easing);
                t.add_text_animator(
                    anim.start_idx,
                    anim.end_idx,
                    anim.property.clone(),
                    anim.start_val,
                    anim.target_val,
                    anim.duration,
                    easing_str,
                );
            }

            Box::new(t)
        }
        NodeKind::Image { src, object_fit } => {
            // Load bytes immediately (blocking for now)
            let bytes = director.assets.loader.load_bytes(src).unwrap_or_default();
            let mut img = ImageNode::new(bytes);
            if let Some(fit) = object_fit {
                img.object_fit = parse_object_fit(fit);
            }
            Box::new(img)
        }
        NodeKind::Video { src, object_fit } => {
            // Load video bytes and create VideoNode
            let bytes = director.assets.loader.load_bytes(src).unwrap_or_default();
            let source = VideoSource::Bytes(bytes);
            let mut vid = VideoNode::new(source, RenderMode::Export);
            if let Some(fit) = object_fit {
                vid.object_fit = parse_object_fit(fit);
            }
            Box::new(vid)
        }
        NodeKind::Vector { src } => {
            // Load SVG bytes and create VectorNode
            let bytes = director.assets.loader.load_bytes(src).unwrap_or_default();
            Box::new(VectorNode::new(&bytes))
        }
        NodeKind::Lottie {
            src,
            speed,
            loop_animation,
        } => {
            // Load Lottie JSON and create LottieNode
            let bytes = director.assets.loader.load_bytes(src).unwrap_or_default();
            match LottieNode::new(&bytes, HashMap::new(), &director.assets) {
                Ok(mut lottie) => {
                    lottie.speed = *speed;
                    lottie.loop_anim = *loop_animation;
                    Box::new(lottie)
                }
                Err(_) => Box::new(BoxNode::new()), // Fallback on error
            }
        }
        NodeKind::Effect { effect_type } => {
            // Color matrix presets
            const GRAYSCALE_MATRIX: [f32; 20] = [
                0.2126, 0.7152, 0.0722, 0.0, 0.0, 0.2126, 0.7152, 0.0722, 0.0, 0.0, 0.2126, 0.7152,
                0.0722, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0,
            ];
            const SEPIA_MATRIX: [f32; 20] = [
                0.393, 0.769, 0.189, 0.0, 0.0, 0.349, 0.686, 0.168, 0.0, 0.0, 0.272, 0.534, 0.131,
                0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0,
            ];

            // Create EffectNode based on effect configuration
            let effect = match effect_type {
                EffectConfig::Blur { sigma } => EffectType::Blur(Animated::new(*sigma)),
                EffectConfig::DropShadow {
                    blur,
                    offset_x,
                    offset_y,
                    color,
                } => EffectType::DropShadow {
                    blur: Animated::new(*blur),
                    offset_x: Animated::new(*offset_x),
                    offset_y: Animated::new(*offset_y),
                    color: Animated::new(color.unwrap_or(Color::BLACK)),
                },
                EffectConfig::ColorMatrix { matrix } => EffectType::ColorMatrix(matrix.clone()),
                EffectConfig::Grayscale => EffectType::ColorMatrix(GRAYSCALE_MATRIX.to_vec()),
                EffectConfig::Sepia => EffectType::ColorMatrix(SEPIA_MATRIX.to_vec()),
                EffectConfig::DirectionalBlur {
                    strength,
                    angle,
                    samples,
                } => EffectType::DirectionalBlur {
                    strength: Animated::new(*strength),
                    angle: Animated::new(*angle),
                    samples: *samples,
                },
                EffectConfig::FilmGrain { intensity, size } => EffectType::FilmGrain {
                    intensity: Animated::new(*intensity),
                    size: Animated::new(*size),
                },
            };
            Box::new(EffectNode {
                effects: vec![effect],
                style: Style::DEFAULT,
                shader_cache: Arc::new(std::sync::Mutex::new(HashMap::new())),
                current_time: 0.0,
            })
        }
    };

    // Apply opacity from StyleMap if specified
    if let Some(opacity) = node_def.style.opacity {
        element.animate_property("opacity", opacity, opacity, 0.0, "linear");
    }

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
                box_node.bg_color = Some(Animated::new(bg));
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
        let easing_str = easing_to_str(&anim.easing);

        let start_val = anim.start.unwrap_or(0.0); // Fallback if start not provided

        element.animate_property(
            &anim.property,
            start_val,
            anim.end,
            anim.duration,
            easing_str,
        );
    }
}

fn apply_style_map(style: &mut Style, map: &StyleMap) {
    // Size
    if let Some(w) = &map.width {
        style.size.width = parse_dim(w);
    }
    if let Some(h) = &map.height {
        style.size.height = parse_dim(h);
    }

    // Display mode
    if let Some(d) = &map.display {
        style.display = match d.as_str() {
            "grid" => Display::Grid,
            "none" => Display::None,
            _ => Display::Flex,
        };
    }

    // Gap
    if let Some(g) = map.gap {
        let gap_val = LengthPercentage::length(g);
        style.gap = Size {
            width: gap_val,
            height: gap_val,
        };
    }

    // Flexbox
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

    // Grid templates
    if let Some(cols) = &map.grid_template_columns {
        style.grid_template_columns = cols.iter().map(|s| parse_track_sizing_str(s)).collect();
    }
    if let Some(rows) = &map.grid_template_rows {
        style.grid_template_rows = rows.iter().map(|s| parse_track_sizing_str(s)).collect();
    }

    // Grid placement
    if let Some(s) = &map.grid_row {
        style.grid_row = parse_grid_line_str(s);
    }
    if let Some(s) = &map.grid_column {
        style.grid_column = parse_grid_line_str(s);
    }

    // Padding
    if let Some(p) = map.padding {
        let d = LengthPercentage::length(p);
        style.padding = Rect {
            left: d,
            right: d,
            top: d,
            bottom: d,
        };
    }

    // Margin
    if let Some(m) = map.margin {
        let d = LengthPercentageAuto::length(m);
        style.margin = Rect {
            left: d,
            right: d,
            top: d,
            bottom: d,
        };
    }

    // Position mode (absolute/relative)
    if let Some(pos) = &map.position {
        if pos == "absolute" {
            style.position = Position::Absolute;
        }
    }

    // Insets for absolute positioning
    if let Some(top) = map.top {
        style.inset.top = LengthPercentageAuto::length(top);
    }
    if let Some(left) = map.left {
        style.inset.left = LengthPercentageAuto::length(left);
    }
    if let Some(right) = map.right {
        style.inset.right = LengthPercentageAuto::length(right);
    }
    if let Some(bottom) = map.bottom {
        style.inset.bottom = LengthPercentageAuto::length(bottom);
    }
}

fn apply_transform_map(transform: &mut director_core::types::Transform, map: &TransformMap) {
    if let Some(v) = map.x {
        transform.translate_x = Animated::new(v);
    }
    if let Some(v) = map.y {
        transform.translate_y = Animated::new(v);
    }
    if let Some(v) = map.rotation {
        transform.rotation = Animated::new(v);
    }
    if let Some(v) = map.scale {
        transform.scale_x = Animated::new(v);
        transform.scale_y = Animated::new(v);
    }
    if let Some(v) = map.pivot_x {
        transform.pivot_x = v;
    }
    if let Some(v) = map.pivot_y {
        transform.pivot_y = v;
    }
}

fn parse_dim(val: &str) -> Dimension {
    if val == "auto" {
        Dimension::auto()
    } else if val.ends_with("%") {
        let f = val.trim_end_matches("%").parse::<f32>().unwrap_or(0.0);
        Dimension::percent(f / 100.0)
    } else {
        let f = val.parse::<f32>().unwrap_or(0.0);
        Dimension::length(f)
    }
}

fn parse_object_fit(val: &str) -> ObjectFit {
    match val {
        "contain" => ObjectFit::Contain,
        "fill" => ObjectFit::Fill,
        "cover" | _ => ObjectFit::Cover,
    }
}

fn easing_to_str(easing: &EasingType) -> &'static str {
    match easing {
        EasingType::Linear => "linear",
        EasingType::EaseIn => "ease_in",
        EasingType::EaseOut => "ease_out",
        EasingType::EaseInOut => "ease_in_out",
        EasingType::BounceOut => "bounce_out",
        EasingType::BounceIn => "bounce_in",
        EasingType::BounceInOut => "bounce_in_out",
        EasingType::ElasticOut => "elastic_out",
        EasingType::ElasticIn => "elastic_in",
        EasingType::ElasticInOut => "elastic_in_out",
        EasingType::BackOut => "back_out",
        EasingType::BackIn => "back_in",
        EasingType::BackInOut => "back_in_out",
    }
}

fn parse_track_sizing_str(s: &str) -> GridTemplateComponent<String> {
    if s == "auto" {
        GridTemplateComponent::Single(auto())
    } else if s == "min-content" || s == "min_content" {
        GridTemplateComponent::Single(min_content())
    } else if s == "max-content" || s == "max_content" {
        GridTemplateComponent::Single(max_content())
    } else if s.ends_with("fr") {
        let val = s.trim_end_matches("fr").parse::<f32>().unwrap_or(1.0);
        GridTemplateComponent::Single(fr(val))
    } else if s.ends_with("%") {
        let val = s.trim_end_matches("%").parse::<f32>().unwrap_or(0.0);
        GridTemplateComponent::Single(percent(val / 100.0))
    } else {
        let val = s.parse::<f32>().unwrap_or(0.0);
        GridTemplateComponent::Single(length(val))
    }
}

fn parse_grid_placement_str(s: &str) -> GridPlacement<String> {
    let s = s.trim();
    if s.starts_with("span") {
        let val = s
            .trim_start_matches("span")
            .trim()
            .parse::<u16>()
            .unwrap_or(1);
        GridPlacement::Span(val)
    } else if let Ok(val) = s.parse::<i16>() {
        GridPlacement::Line(val.into())
    } else {
        GridPlacement::Auto
    }
}

fn parse_grid_line_str(s: &str) -> Line<GridPlacement<String>> {
    if s.contains('/') {
        let parts: Vec<&str> = s.split('/').collect();
        Line {
            start: parse_grid_placement_str(parts[0]),
            end: if parts.len() > 1 {
                parse_grid_placement_str(parts[1])
            } else {
                GridPlacement::Auto
            },
        }
    } else {
        Line {
            start: parse_grid_placement_str(s),
            end: GridPlacement::Auto,
        }
    }
}
