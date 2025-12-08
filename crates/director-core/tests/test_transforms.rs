use director_engine::{scripting::register_rhai_api, DefaultAssetLoader};
use rhai::Engine;
use std::sync::Arc;

#[test]
fn test_transforms_api() {
    let mut engine = Engine::new();
    let loader = Arc::new(DefaultAssetLoader);
    register_rhai_api(&mut engine, loader);

    let script = r##"
        let movie = new_director(1920, 1080, 30);
        let scene = movie.add_scene(5.0);
        let box = scene.add_box(#{
            width: 200,
            height: 200,
            bg_color: "#FF0000"
        });

        // Pivot
        box.set_pivot(0.5, 0.5);

        // Scale shortcut
        box.animate("scale", 0.0, 2.0, 2.0, "bounce_out");

        // Individual properties
        box.animate("rotation", 0.0, 360.0, 5.0, "linear");
        box.animate("skew_x", 0.0, 30.0, 2.0, "ease_in_out");

        // Translation aliases
        box.animate("x", 0.0, 100.0, 2.0, "linear");
        box.animate("y", 0.0, 100.0, 2.0, "linear");
    "##;

    if let Err(e) = engine.run(script) {
        panic!("Rhai script failed: {}", e);
    }
}
