use director_engine::{scripting::{register_rhai_api, MovieHandle}, AssetLoader};
use rhai::Engine;
use std::sync::Arc;
use anyhow::Result;

struct MockLoader;
impl AssetLoader for MockLoader {
    fn load_bytes(&self, _path: &str) -> Result<Vec<u8>> {
        Ok(vec![0; 10])
    }
}

#[test]
fn test_kitchen_sink_layout() {
    let mut engine = Engine::new();
    let loader = Arc::new(MockLoader);
    register_rhai_api(&mut engine, loader);

    let script = r#"
       let movie = new_director(1000, 1000, 30);
       let scene = movie.add_scene(5.0);

       // Create a container filling the screen
       let root = scene.add_box(#{
           width: "100%",
           height: "100%",
           flex_grow: 1.0,
           flex_direction: "row",
           align_items: "stretch"
       });

       // Image with flex grow 1
       // Should take (1000 - 200) = 800px width
       root.add_image("test.jpg", #{
           flex_grow: 1.0,
           height: "auto"
       });

       // Video with fixed width
       root.add_video("test.mp4", #{
           width: 200,
           height: "100%"
       });

       movie
    "#;

    let result = engine.eval::<MovieHandle>(script).expect("Script failed");
    let mut director = result.director.lock().unwrap();

    let mut surface = skia_safe::surfaces::raster_n32_premul((100, 100)).unwrap();
    director_engine::render::render_frame(&mut director, 0.0, surface.canvas());

    let scene_root_id = director.timeline[0].scene_root;
    let scene_root = director.get_node(scene_root_id).unwrap();

    // Check Root Box
    assert_eq!(scene_root.children.len(), 1, "Scene root should have 1 child");
    let user_root_id = scene_root.children[0];
    let user_root = director.get_node(user_root_id).unwrap();

    // Check Image and Video
    assert_eq!(user_root.children.len(), 2, "User root should have 2 children");

    let image_id = user_root.children[0];
    let video_id = user_root.children[1];

    let image_node = director.get_node(image_id).unwrap();
    let video_node = director.get_node(video_id).unwrap();

    // Verify Video Width = 200.0
    assert!((video_node.layout_rect.width() - 200.0).abs() < 0.1, "Video width should be 200");

    // Verify Image Width = 800.0
    assert!((image_node.layout_rect.width() - 800.0).abs() < 0.1, "Image width should be 800");

    // Verify Heights = 1000.0
    assert!((image_node.layout_rect.height() - 1000.0).abs() < 0.1, "Image height should be 1000");
    assert!((video_node.layout_rect.height() - 1000.0).abs() < 0.1, "Video height should be 1000");

    // Verify positions
    assert!((image_node.layout_rect.left - 0.0).abs() < 0.1, "Image should start at 0");
    assert!((video_node.layout_rect.left - 800.0).abs() < 0.1, "Video should start at 800");
}
