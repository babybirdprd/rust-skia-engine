use director_core::{Director, node::TextNode, element::{TextSpan, TextFit}};
use std::sync::Arc;
use director_core::video_wrapper::RenderMode;
use director_core::DefaultAssetLoader;

#[test]
fn test_text_fit_shrink() {
    let loader = Arc::new(DefaultAssetLoader);
    let mut director = Director::new(1920, 1080, 30, loader, RenderMode::Preview, None);

    let spans = vec![TextSpan {
        text: "This is a very long text that should definitely shrink if the box is small enough.".to_string(),
        color: None,
        font_family: None,
        font_weight: None,
        font_style: None,
        font_size: Some(100.0),
        background_color: None,
        background_padding: None,
        stroke_width: None,
        stroke_color: None,
        fill_gradient: None,
    }];

    // Create TextNode manually
    let font_collection = director.assets.font_collection.clone();
    let mut text_node = TextNode::new(spans, font_collection);
    text_node.fit_mode = TextFit::Shrink;
    text_node.min_size = 10.0;
    text_node.max_size = 100.0;

    // Set explicit layout style on the text node itself
    text_node.style.size = taffy::geometry::Size {
        width: taffy::style::Dimension::length(200.0),
        height: taffy::style::Dimension::length(50.0)
    };

    // Add to director
    let id = director.scene.add_node(Box::new(text_node));

    // Make it scene root
    let item = director_core::director::TimelineItem {
        scene_root: id,
        start_time: 0.0,
        duration: 5.0,
        z_index: 0,
        audio_tracks: vec![],
    };
    director.timeline.push(item);

    // Render frame 0 (trigger layout and post_layout)
    // We don't need a real canvas, just trigger the pipeline
    let mut surface = skia_safe::surfaces::raster_n32_premul((1920, 1080)).unwrap();
    director_core::systems::renderer::render_frame(&mut director, 0.0, surface.canvas());

    // Check font size
    let node = director.scene.get_node(id).unwrap();
    let text_node = node.element.as_any().downcast_ref::<TextNode>().unwrap();

    println!("Final font size: {}", text_node.default_font_size.current_value);
    assert!(text_node.default_font_size.current_value < 100.0, "Font size should have shrunk");
    assert!(text_node.default_font_size.current_value >= 10.0, "Font size should be >= min");
}

#[test]
fn test_render_video_output() {
    let loader = Arc::new(DefaultAssetLoader);
    let mut director = Director::new(1920, 1080, 30, loader, RenderMode::Export, None);

    let spans = vec![TextSpan {
        text: "Typography Engine: Text-to-Fit Test".to_string(),
        color: Some(director_core::types::Color::WHITE),
        font_family: None,
        font_weight: Some(700),
        font_style: None,
        font_size: Some(150.0),
        background_color: Some(director_core::types::Color::new(0.0, 0.0, 1.0, 1.0)),
        background_padding: Some(20.0),
        stroke_width: Some(2.0),
        stroke_color: Some(director_core::types::Color::BLACK),
        fill_gradient: None,
    }];

    let font_collection = director.assets.font_collection.clone();
    let mut text_node = TextNode::new(spans, font_collection);
    text_node.fit_mode = TextFit::Shrink;
    text_node.min_size = 20.0;
    text_node.max_size = 200.0;

    // Constrain width to force shrink
    text_node.style.size = taffy::geometry::Size {
        width: taffy::style::Dimension::length(800.0),
        height: taffy::style::Dimension::auto(),
    };
    // Center it
    text_node.style.margin = taffy::geometry::Rect {
        left: taffy::style::LengthPercentageAuto::auto(),
        right: taffy::style::LengthPercentageAuto::auto(),
        top: taffy::style::LengthPercentageAuto::auto(),
        bottom: taffy::style::LengthPercentageAuto::auto(),
    };

    // Add Shadow
    text_node.shadow = Some(director_core::element::TextShadow {
        color: director_core::types::Color::new(0.0, 0.0, 0.0, 0.5),
        blur: 10.0,
        offset: (10.0, 10.0),
    });

    let id = director.scene.add_node(Box::new(text_node));

    let item = director_core::director::TimelineItem {
        scene_root: id,
        start_time: 0.0,
        duration: 2.0, // 2 seconds
        z_index: 0,
        audio_tracks: vec![],
    };
    director.timeline.push(item);

    let out_path = std::path::PathBuf::from("typography_test.mp4");
    // Ensure we delete old one
    if out_path.exists() {
        std::fs::remove_file(&out_path).unwrap();
    }

    director_core::systems::renderer::render_export(&mut director, out_path, None, None).unwrap();
}
