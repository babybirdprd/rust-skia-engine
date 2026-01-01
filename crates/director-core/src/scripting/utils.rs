//! # Scripting Utilities
//!
//! Parsing helpers and utility functions for Rhai script bindings.
//!
//! ## Responsibilities
//! - **Color Parsing**: `parse_hex_color` for hex string to Color conversion
//! - **Layout Parsing**: `parse_layout_style` for Taffy style properties
//! - **Text Parsing**: `parse_text_style`, `parse_spans_from_dynamic`
//! - **Animation Parsing**: `parse_spring_config`, `parse_easing`
//! - **Effect Helpers**: `apply_effect_to_node`, `extract_outer_style`

use crate::animation::{EasingType, SpringConfig};
use crate::director::Director;
use crate::element::{TextShadow, TextSpan};
use crate::node::{BoxNode, CompositionNode, EffectNode, EffectType};
use crate::types::{Color, GradientConfig, NodeId, ObjectFit};
use rhai::Map;
use taffy::prelude::*;
use taffy::style::{GridPlacement, GridTemplateComponent, Style};

/// Extract outer style properties for effect wrapper nodes.
pub fn extract_outer_style(source: &Style) -> Style {
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

/// Apply an effect to a node by wrapping it in an EffectNode.
pub fn apply_effect_to_node(d: &mut Director, node_id: NodeId, effect: EffectType) -> NodeId {
    let parent_id_opt = d.scene.get_node(node_id).and_then(|n| n.parent);

    let wrapper_style;

    // Modify target node style (Steal & Fill)
    if let Some(node) = d.scene.get_node_mut(node_id) {
        let original_style = node.element.layout_style();
        wrapper_style = extract_outer_style(&original_style);

        if let Some(box_node) = node.element.as_any_mut().downcast_mut::<BoxNode>() {
            box_node.style.size = taffy::geometry::Size {
                width: Dimension::percent(1.0),
                height: Dimension::percent(1.0),
            };
            box_node.style.margin = taffy::geometry::Rect::zero();
            box_node.style.flex_grow = 0.0;
            box_node.style.flex_shrink = 1.0;
            box_node.style.position = taffy::style::Position::Relative;
            box_node.style.inset = taffy::geometry::Rect::auto();
        } else if let Some(comp_node) = node.element.as_any_mut().downcast_mut::<CompositionNode>()
        {
            comp_node.style.size = taffy::geometry::Size {
                width: Dimension::percent(1.0),
                height: Dimension::percent(1.0),
            };
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
        shader_cache: d.assets.shader_cache.clone(),
        current_time: 0.0,
    };

    let effect_id = d.scene.add_node(Box::new(effect_node));

    if let Some(parent_id) = parent_id_opt {
        d.scene.remove_child(parent_id, node_id);
        d.scene.add_child(parent_id, effect_id);
    }

    d.scene.add_child(effect_id, node_id);

    // Update Root if needed
    for item in &mut d.timeline {
        if item.scene_root == node_id {
            item.scene_root = effect_id;
        }
    }

    effect_id
}

/// Helper to parse hex strings like "#RRGGBB" or "#RGB"
pub fn parse_hex_color(hex: &str) -> Option<Color> {
    let hex = hex.trim_start_matches('#');
    let (r, g, b) = match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            (r, g, b)
        }
        3 => {
            let r = u8::from_str_radix(&hex[0..1], 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2], 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3], 16).ok()?;
            (r * 17, g * 17, b * 17)
        }
        _ => return None,
    };

    Some(Color::new(
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
        1.0,
    ))
}

/// Parse text style properties from a Rhai map into a TextSpan.
pub fn parse_text_style(map: &rhai::Map, span: &mut TextSpan) {
    if let Some(c) = map.get("color").and_then(|v| v.clone().into_string().ok()) {
        span.color = parse_hex_color(&c);
    }
    if let Some(w_str) = map.get("weight").and_then(|v| v.clone().into_string().ok()) {
        if w_str == "bold" {
            span.font_weight = Some(700);
        }
    }
    if let Some(s) = map.get("size").and_then(|v| v.as_float().ok()) {
        span.font_size = Some(s as f32);
    }
    // Rich Text Fields
    if let Some(c) = map
        .get("background_color")
        .and_then(|v| v.clone().into_string().ok())
    {
        span.background_color = parse_hex_color(&c);
    }
    if let Some(p) = map
        .get("background_padding")
        .and_then(|v| v.as_float().ok())
    {
        span.background_padding = Some(p as f32);
    }
    if let Some(w) = map.get("stroke_width").and_then(|v| v.as_float().ok()) {
        span.stroke_width = Some(w as f32);
    }
    if let Some(c) = map
        .get("stroke_color")
        .and_then(|v| v.clone().into_string().ok())
    {
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

/// Parse spring configuration from a Rhai map.
pub fn parse_spring_config(map: &rhai::Map) -> SpringConfig {
    let mut config = SpringConfig::default();
    if let Some(v) = map.get("stiffness").and_then(|v| v.as_float().ok()) {
        config.stiffness = v as f32;
    }
    if let Some(v) = map.get("damping").and_then(|v| v.as_float().ok()) {
        config.damping = v as f32;
    }
    if let Some(v) = map.get("mass").and_then(|v| v.as_float().ok()) {
        config.mass = v as f32;
    }
    if let Some(v) = map.get("velocity").and_then(|v| v.as_float().ok()) {
        config.velocity = v as f32;
    }
    config
}

/// Parse easing type from string.
pub fn parse_easing(ease: &str) -> EasingType {
    match ease {
        "linear" => EasingType::Linear,
        "ease_in" => EasingType::EaseIn,
        "ease_out" => EasingType::EaseOut,
        "ease_in_out" => EasingType::EaseInOut,
        "bounce_out" => EasingType::BounceOut,
        "bounce_in" => EasingType::BounceIn,
        "bounce_in_out" => EasingType::BounceInOut,
        "elastic_out" => EasingType::ElasticOut,
        "elastic_in" => EasingType::ElasticIn,
        "elastic_in_out" => EasingType::ElasticInOut,
        "back_out" => EasingType::BackOut,
        "back_in" => EasingType::BackIn,
        "back_in_out" => EasingType::BackInOut,
        _ => EasingType::Linear,
    }
}

/// Parse layout style properties from a Rhai map into a Taffy Style.
pub fn parse_layout_style(props: &rhai::Map, style: &mut Style) {
    let to_dim = |v: &rhai::Dynamic| -> Option<Dimension> {
        if let Ok(f) = v.as_float() {
            Some(Dimension::length(f as f32))
        } else if let Ok(i) = v.as_int() {
            Some(Dimension::length(i as f32))
        } else if let Ok(s) = v.clone().into_string() {
            if s == "auto" {
                Some(Dimension::auto())
            } else if s.ends_with("%") {
                if let Ok(p) = s.trim_end_matches('%').parse::<f32>() {
                    Some(Dimension::percent(p / 100.0))
                } else {
                    None
                }
            } else if let Ok(val) = s.parse::<f32>() {
                Some(Dimension::length(val))
            } else {
                None
            }
        } else {
            None
        }
    };

    if let Some(w) = props.get("width").and_then(to_dim) {
        style.size.width = w;
    }
    if let Some(h) = props.get("height").and_then(to_dim) {
        style.size.height = h;
    }

    if let Some(s) = props
        .get("flex_direction")
        .and_then(|v| v.clone().into_string().ok())
    {
        style.flex_direction = match s.as_str() {
            "row" => FlexDirection::Row,
            "column" => FlexDirection::Column,
            "row_reverse" | "row-reverse" => FlexDirection::RowReverse,
            "column_reverse" | "column-reverse" => FlexDirection::ColumnReverse,
            _ => FlexDirection::Row,
        };
    }

    if let Some(s) = props
        .get("align_items")
        .and_then(|v| v.clone().into_string().ok())
    {
        style.align_items = match s.as_str() {
            "center" => Some(AlignItems::Center),
            "flex_start" | "flex-start" | "start" => Some(AlignItems::FlexStart),
            "flex_end" | "flex-end" | "end" => Some(AlignItems::FlexEnd),
            "stretch" => Some(AlignItems::Stretch),
            _ => Some(AlignItems::Stretch),
        };
    }

    if let Some(s) = props
        .get("justify_content")
        .and_then(|v| v.clone().into_string().ok())
    {
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
        if let Ok(f) = v.as_float() {
            Some(LengthPercentage::length(f as f32))
        } else if let Ok(i) = v.as_int() {
            Some(LengthPercentage::length(i as f32))
        } else if let Ok(s) = v.clone().into_string() {
            if s.ends_with("%") {
                if let Ok(p) = s.trim_end_matches('%').parse::<f32>() {
                    Some(LengthPercentage::percent(p / 100.0))
                } else {
                    None
                }
            } else if let Ok(val) = s.parse::<f32>() {
                Some(LengthPercentage::length(val))
            } else {
                None
            }
        } else {
            None
        }
    };

    if let Some(p) = props.get("padding").and_then(to_lp) {
        style.padding = Rect {
            left: p,
            right: p,
            top: p,
            bottom: p,
        };
    }

    let to_lpa = |v: &rhai::Dynamic| -> Option<LengthPercentageAuto> {
        if let Ok(f) = v.as_float() {
            Some(LengthPercentageAuto::length(f as f32))
        } else if let Ok(i) = v.as_int() {
            Some(LengthPercentageAuto::length(i as f32))
        } else if let Ok(s) = v.clone().into_string() {
            if s == "auto" {
                Some(LengthPercentageAuto::auto())
            } else if s.ends_with("%") {
                if let Ok(p) = s.trim_end_matches('%').parse::<f32>() {
                    Some(LengthPercentageAuto::percent(p / 100.0))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    };

    if let Some(m) = props.get("margin").and_then(to_lpa) {
        style.margin = Rect {
            left: m,
            right: m,
            top: m,
            bottom: m,
        };
    }

    // Gap
    if let Some(g) = props.get("gap").and_then(|v| v.as_float().ok()) {
        let gap_val = LengthPercentage::length(g as f32);
        style.gap = Size {
            width: gap_val,
            height: gap_val,
        };
    }

    // Display (Flex/Grid)
    if let Some(s) = props
        .get("display")
        .and_then(|v| v.clone().into_string().ok())
    {
        style.display = match s.as_str() {
            "grid" => Display::Grid,
            "none" => Display::None,
            _ => Display::Flex,
        };
    }

    // Grid Templates
    let parse_track_sizing = |s: &str| -> GridTemplateComponent<String> {
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
    };

    if let Some(cols) = props
        .get("grid_template_columns")
        .and_then(|v| v.clone().into_array().ok())
    {
        style.grid_template_columns = cols
            .into_iter()
            .map(|v| parse_track_sizing(&v.to_string()))
            .collect();
    }
    if let Some(rows) = props
        .get("grid_template_rows")
        .and_then(|v| v.clone().into_array().ok())
    {
        style.grid_template_rows = rows
            .into_iter()
            .map(|v| parse_track_sizing(&v.to_string()))
            .collect();
    }

    // Grid Placement (Item)
    let parse_gp = |s: &str| -> GridPlacement<String> {
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
    };

    let parse_line = |s: &str| -> Line<GridPlacement<String>> {
        if s.contains('/') {
            let parts: Vec<&str> = s.split('/').collect();
            Line {
                start: parse_gp(parts[0]),
                end: if parts.len() > 1 {
                    parse_gp(parts[1])
                } else {
                    GridPlacement::Auto
                },
            }
        } else {
            Line {
                start: parse_gp(s),
                end: GridPlacement::Auto,
            }
        }
    };

    if let Some(s) = props
        .get("grid_row")
        .and_then(|v| v.clone().into_string().ok())
    {
        style.grid_row = parse_line(&s);
    }
    if let Some(s) = props
        .get("grid_column")
        .and_then(|v| v.clone().into_string().ok())
    {
        style.grid_column = parse_line(&s);
    }

    // Position (Relative/Absolute)
    if let Some(s) = props
        .get("position")
        .and_then(|v| v.clone().into_string().ok())
    {
        style.position = match s.as_str() {
            "absolute" => taffy::style::Position::Absolute,
            _ => taffy::style::Position::Relative,
        };
    }

    // Insets (Top, Left, Right, Bottom) - reuse to_lpa closure
    if let Some(v) = props.get("left").and_then(|v| to_lpa(v)) {
        style.inset.left = v;
    }
    if let Some(v) = props.get("right").and_then(|v| to_lpa(v)) {
        style.inset.right = v;
    }
    if let Some(v) = props.get("top").and_then(|v| to_lpa(v)) {
        style.inset.top = v;
    }
    if let Some(v) = props.get("bottom").and_then(|v| to_lpa(v)) {
        style.inset.bottom = v;
    }
}

/// Parse object fit mode from string.
pub fn parse_object_fit(val: &str) -> Option<ObjectFit> {
    match val {
        "cover" => Some(ObjectFit::Cover),
        "contain" => Some(ObjectFit::Contain),
        "fill" => Some(ObjectFit::Fill),
        _ => None,
    }
}

/// Parse text spans from a Rhai dynamic value.
pub fn parse_spans_from_dynamic(content: rhai::Dynamic) -> Vec<TextSpan> {
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

/// Parse a text shadow from Rhai props map.
pub fn parse_text_shadow(props: &rhai::Map) -> Option<TextShadow> {
    let mut has_shadow = false;
    let mut shadow = TextShadow {
        color: Color::BLACK,
        blur: 0.0,
        offset: (0.0, 0.0),
    };
    if let Some(c) = props
        .get("text_shadow_color")
        .and_then(|v| v.clone().into_string().ok())
    {
        if let Some(col) = parse_hex_color(&c) {
            shadow.color = col;
            has_shadow = true;
        }
    }
    if let Some(v) = props
        .get("text_shadow_blur")
        .and_then(|v| v.as_float().ok())
    {
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
        Some(shadow)
    } else {
        None
    }
}
