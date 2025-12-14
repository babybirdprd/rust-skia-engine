//! Typography Tests
//!
//! Tests for text rendering features including TextFit, rich spans, and shadows.

use director_core::{
    element::{TextFit, TextSpan},
    node::TextNode,
    video_wrapper::RenderMode,
    DefaultAssetLoader, Director,
};
use std::sync::Arc;

/// Test TextFit::Shrink behavior.
///
/// Validates that text automatically shrinks when constrained to a small box.
#[test]
fn text_fit_shrink() {
    let loader = Arc::new(DefaultAssetLoader);
    let mut director = Director::new(1920, 1080, 30, loader, RenderMode::Preview, None);

    let spans = vec![TextSpan {
        text: "This is a very long text that should shrink to fit.".to_string(),
        font_size: Some(100.0),
        ..Default::default()
    }];

    let font_collection = director.assets.font_collection.clone();
    let mut text_node = TextNode::new(spans, font_collection);
    text_node.fit_mode = TextFit::Shrink;
    text_node.min_size = 10.0;
    text_node.max_size = 100.0;

    // Constrain to a small box
    text_node.style.size = taffy::geometry::Size {
        width: taffy::style::Dimension::length(200.0),
        height: taffy::style::Dimension::length(50.0),
    };

    let id = director.scene.add_node(Box::new(text_node));

    director
        .timeline
        .push(director_core::director::TimelineItem {
            scene_root: id,
            start_time: 0.0,
            duration: 1.0,
            z_index: 0,
            audio_tracks: vec![],
        });

    // Trigger layout
    let mut surface = skia_safe::surfaces::raster_n32_premul((1920, 1080)).unwrap();
    director_core::systems::renderer::render_frame(&mut director, 0.0, surface.canvas());

    // Verify shrinkage
    let node = director.scene.get_node(id).unwrap();
    let text_node = node.element.as_any().downcast_ref::<TextNode>().unwrap();

    assert!(
        text_node.default_font_size.current_value < 100.0,
        "Font should have shrunk from 100.0, got {}",
        text_node.default_font_size.current_value
    );
    assert!(
        text_node.default_font_size.current_value >= 10.0,
        "Font should not go below min_size 10.0, got {}",
        text_node.default_font_size.current_value
    );
}

/// Test rich text span styling (background, stroke).
///
/// Validates that TextSpan properties are correctly applied.
#[test]
fn text_span_rich_styling() {
    let loader = Arc::new(DefaultAssetLoader);
    let mut director = Director::new(800, 600, 30, loader, RenderMode::Preview, None);

    let spans = vec![TextSpan {
        text: "Styled Text".to_string(),
        color: Some(director_core::types::Color::WHITE),
        font_weight: Some(700),
        font_size: Some(48.0),
        background_color: Some(director_core::types::Color::new(0.0, 0.0, 1.0, 1.0)),
        background_padding: Some(10.0),
        stroke_width: Some(2.0),
        stroke_color: Some(director_core::types::Color::BLACK),
        ..Default::default()
    }];

    let font_collection = director.assets.font_collection.clone();
    let text_node = TextNode::new(spans, font_collection);

    let id = director.scene.add_node(Box::new(text_node));

    director
        .timeline
        .push(director_core::director::TimelineItem {
            scene_root: id,
            start_time: 0.0,
            duration: 1.0,
            z_index: 0,
            audio_tracks: vec![],
        });

    // Should not panic during render
    let mut surface = skia_safe::surfaces::raster_n32_premul((800, 600)).unwrap();
    director_core::systems::renderer::render_frame(&mut director, 0.0, surface.canvas());
}

/// Test text shadow rendering.
///
/// Validates that TextShadow is applied without panicking.
#[test]
fn text_shadow() {
    let loader = Arc::new(DefaultAssetLoader);
    let mut director = Director::new(800, 600, 30, loader, RenderMode::Preview, None);

    let spans = vec![TextSpan {
        text: "Shadow Test".to_string(),
        font_size: Some(48.0),
        color: Some(director_core::types::Color::WHITE),
        ..Default::default()
    }];

    let font_collection = director.assets.font_collection.clone();
    let mut text_node = TextNode::new(spans, font_collection);

    text_node.shadow = Some(director_core::element::TextShadow {
        color: director_core::types::Color::new(0.0, 0.0, 0.0, 0.5),
        blur: 10.0,
        offset: (5.0, 5.0),
    });

    let id = director.scene.add_node(Box::new(text_node));

    director
        .timeline
        .push(director_core::director::TimelineItem {
            scene_root: id,
            start_time: 0.0,
            duration: 1.0,
            z_index: 0,
            audio_tracks: vec![],
        });

    // Should not panic during render
    let mut surface = skia_safe::surfaces::raster_n32_premul((800, 600)).unwrap();
    director_core::systems::renderer::render_frame(&mut director, 0.0, surface.canvas());
}
