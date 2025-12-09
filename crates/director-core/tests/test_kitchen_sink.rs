use director_core::{scripting::{register_rhai_api, MovieHandle}, DefaultAssetLoader};
use rhai::Engine;
use std::sync::Arc;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_kitchen_sink_layout() {
    let mut engine = Engine::new();
    let loader = Arc::new(DefaultAssetLoader);
    register_rhai_api(&mut engine, loader);

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let script_path = std::path::Path::new(manifest_dir).join("tests/kitchen_sink.rhai");
    let script = fs::read_to_string(&script_path).expect("Failed to read script");
    
    // Set current dir to workspace root for asset paths in script
    let workspace_root = std::path::Path::new(manifest_dir).parent().unwrap().parent().unwrap();
    std::env::set_current_dir(workspace_root).ok();

    let result = engine.eval::<MovieHandle>(&script).expect("Script failed");
    let mut director = result.director.lock().unwrap();

    // Trigger Layout (Frame 0)
    let mut surface = skia_safe::surfaces::raster_n32_premul((1920, 1080)).unwrap();
    director_core::systems::renderer::render_frame(&mut director, 0.0, surface.canvas());

    // Verify Layout Hierarchy
    let scene_root_id = director.timeline[0].scene_root;
    let scene_root = director.scene.get_node(scene_root_id).unwrap();

    // Root Box
    let user_root = director.scene.get_node(scene_root.children[0]).unwrap();
    assert!((user_root.layout_rect.width() - 1920.0).abs() < 1.0);

    // Columns
    assert_eq!(user_root.children.len(), 2, "Should have 2 columns");
    let col_img = director.scene.get_node(user_root.children[0]).unwrap();
    let col_vid = director.scene.get_node(user_root.children[1]).unwrap();

    // Verify Column Widths (45% of 1920 = 864)
    println!("Col Img Rect: {:?}", col_img.layout_rect);
    assert!((col_img.layout_rect.width() - 864.0).abs() < 10.0, "Column width mismatch");
    assert!((col_vid.layout_rect.width() - 864.0).abs() < 10.0, "Column width mismatch");

    // Check Image Container (Child 1 of Col 1, Child 0 is Text)
    let img_container = director.scene.get_node(col_img.children[1]).unwrap();
    assert!((img_container.layout_rect.width() - 300.0).abs() < 1.0);
    assert!((img_container.layout_rect.height() - 300.0).abs() < 1.0);

    // Check Video Container (Child 1 of Col 2)
    let vid_container = director.scene.get_node(col_vid.children[1]).unwrap();
    // Width is 90% of 864 = ~777.6
    let expected_vid_width = 864.0 * 0.9;
    assert!((vid_container.layout_rect.width() - expected_vid_width).abs() < 10.0);

    // Render Video Output (Kitchen Sink)
    // This generates the actual MP4 file to satisfy the showcase requirement.
    let out_path = PathBuf::from("kitchen_sink.mp4");
    if out_path.exists() {
        fs::remove_file(&out_path).unwrap();
    }

    println!("Rendering kitchen_sink.mp4 (this may take a moment)...");
    director_core::systems::renderer::render_export(&mut director, out_path, None, None).expect("Export failed");
}
