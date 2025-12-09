use director_core::scripting::{register_rhai_api, MovieHandle};
use director_core::DefaultAssetLoader;
use rhai::Engine;
use std::sync::Arc;

#[test]
fn test_text_animator_api() {
    let mut engine = Engine::new();
    register_rhai_api(&mut engine, Arc::new(DefaultAssetLoader));

    let script = r##"
        let movie = new_director(100, 100, 30);
        let scene = movie.add_scene(5.0);

        // 1. Add Text
        let text = scene.add_text(#{
            content: "Hello World"
        });

        // 2. Add Animator (Scale graphemes 0-5 i.e. "Hello")
        // add_animator(start_idx, end_idx, prop, start, target, duration, easing)
        text.add_animator(0, 5, "scale", 1.0, 2.0, 1.0, "ease_out");

        // 3. Add Animator (Offset 'W' - index 6)
        text.add_animator(6, 7, "y", 0.0, -20.0, 1.0, "linear");

        movie
    "##;

    let result = engine.eval::<MovieHandle>(script);
    assert!(result.is_ok(), "Script failed: {:?}", result.err());
}

#[test]
fn test_text_animator_rendering_safety() {
    // This test ensures that rendering with animators doesn't panic
    let mut engine = Engine::new();
    register_rhai_api(&mut engine, Arc::new(DefaultAssetLoader));

    let script = r##"
        let movie = new_director(100, 100, 30);
        let scene = movie.add_scene(1.0);
        let text = scene.add_text(#{ content: "Test" });
        text.add_animator(0, 4, "rotation", 0.0, 360.0, 1.0, "linear");
        movie
    "##;

    let movie = engine.eval::<MovieHandle>(script).expect("Script failed");
    let mut director = movie.director.lock().unwrap();

    // Simulate a few frames
    for i in 0..10 {
        let time = i as f64 * 0.1;

        // Render to a dummy canvas
        let mut surface = skia_safe::surfaces::raster_n32_premul((100, 100)).expect("surface");
        director_core::systems::renderer::render_frame(&mut *director, time, surface.canvas());
    }
}
