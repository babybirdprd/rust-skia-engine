use lottie_core::RenderTree;
use lottie_skia::SkiaRenderer;
use skia_safe::{EncodedImageFormat, Rect, Surface};
use std::fs::File;
use std::io::Write;

#[test]
fn test_render_mock_tree() {
    let tree = RenderTree::mock_sample();

    let width = 500;
    let height = 500;

    let mut surface =
        Surface::new_raster_n32_premul((width, height)).expect("Failed to create surface");
    let canvas = surface.canvas();

    let dest_rect = Rect::from_wh(width as f32, height as f32);

    SkiaRenderer::draw(canvas, &tree, dest_rect, 1.0, &());

    let image = surface.image_snapshot();
    // Use None for context as it is a raster surface
    let data = image
        .encode(None, EncodedImageFormat::PNG, 100)
        .expect("Failed to encode image");

    let mut file = File::create("output.png").expect("Failed to create file");
    file.write_all(data.as_bytes())
        .expect("Failed to write to file");

    // Assert file exists
    assert!(std::path::Path::new("output.png").exists());
}
