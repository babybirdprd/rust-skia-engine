use lottie_core::{
    BlendMode, Image, NodeContent, RenderNode, RenderTree, Text,
};
use lottie_skia::{LottieContext, SkiaRenderer};
use skia_safe::{Image as SkImage, Rect, Surface};
use std::sync::{Arc, Mutex};

struct MockContext {
    pub loaded_fonts: Arc<Mutex<Vec<String>>>,
    pub loaded_images: Arc<Mutex<Vec<String>>>,
}

impl MockContext {
    fn new() -> Self {
        Self {
            loaded_fonts: Arc::new(Mutex::new(Vec::new())),
            loaded_images: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl LottieContext for MockContext {
    fn load_typeface(&self, family: &str, _style: &str) -> Option<skia_safe::Typeface> {
        self.loaded_fonts.lock().unwrap().push(family.to_string());
        // Return None to trigger fallback, but we verified the call.
        skia_safe::FontMgr::new().match_family_style(family, skia_safe::FontStyle::normal())
    }

    fn load_image(&self, id: &str) -> Option<SkImage> {
        self.loaded_images.lock().unwrap().push(id.to_string());
        // Create a 1x1 red image
        let mut surface = Surface::new_raster_n32_premul((1, 1)).unwrap();
        surface.canvas().clear(skia_safe::Color::RED);
        Some(surface.image_snapshot())
    }
}

#[test]
fn test_lottie_context_integration() {
    let ctx = MockContext::new();

    // 1. Create a RenderTree with Text and Image nodes
    // We can manually construct a RenderTree or use LottiePlayer.
    // Manual construction is more direct for testing renderer behavior.

    let text_node = RenderNode {
        transform: glam::Mat4::IDENTITY,
        alpha: 1.0,
        blend_mode: BlendMode::Normal,
        content: NodeContent::Text(Text {
            glyphs: vec![], // Empty glyphs, but family is what matters for the call
            font_family: "CustomFont".to_string(),
            size: 20.0,
            justify: lottie_core::Justification::Left,
            tracking: 0.0,
            line_height: 20.0,
        }),
        masks: vec![],
        matte: None,
        effects: vec![],
        styles: vec![],
        is_adjustment_layer: false,
    };

    let image_node = RenderNode {
        transform: glam::Mat4::IDENTITY,
        alpha: 1.0,
        blend_mode: BlendMode::Normal,
        content: NodeContent::Image(Image {
            data: None,
            width: 100,
            height: 100,
            id: Some("image_0".to_string()),
        }),
        masks: vec![],
        matte: None,
        effects: vec![],
        styles: vec![],
        is_adjustment_layer: false,
    };

    let root = RenderNode {
        transform: glam::Mat4::IDENTITY,
        alpha: 1.0,
        blend_mode: BlendMode::Normal,
        content: NodeContent::Group(vec![text_node, image_node]),
        masks: vec![],
        matte: None,
        effects: vec![],
        styles: vec![],
        is_adjustment_layer: false,
    };

    let tree = RenderTree {
        width: 100.0,
        height: 100.0,
        root,
        view_matrix: glam::Mat4::IDENTITY,
        projection_matrix: glam::Mat4::IDENTITY,
    };

    // 2. Render
    let mut surface = Surface::new_raster_n32_premul((100, 100)).unwrap();
    let canvas = surface.canvas();

    SkiaRenderer::draw(
        canvas,
        &tree,
        Rect::from_wh(100.0, 100.0),
        1.0,
        &ctx,
    );

    // 3. Verify calls
    let fonts = ctx.loaded_fonts.lock().unwrap();
    let images = ctx.loaded_images.lock().unwrap();

    assert!(fonts.contains(&"CustomFont".to_string()), "Should have attempted to load 'CustomFont'");
    assert!(images.contains(&"image_0".to_string()), "Should have attempted to load 'image_0'");
}
