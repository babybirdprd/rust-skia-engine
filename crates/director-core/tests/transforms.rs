//! Transform Animation Tests
//!
//! Tests for transform properties: pivot, scale, rotation, skew, translate.

use director_core::{scripting::register_rhai_api, DefaultAssetLoader};
use rhai::Engine;
use std::sync::Arc;

/// Test all transform animations via Rhai API.
///
/// Validates:
/// - set_pivot()
/// - animate("scale", ...)
/// - animate("rotation", ...)
/// - animate("skew_x", ...)
/// - animate("x", ...) / animate("y", ...)
#[test]
fn transform_animations() {
    let mut engine = Engine::new();
    register_rhai_api(&mut engine, Arc::new(DefaultAssetLoader));

    let script = r##"
let movie = new_director(1920, 1080, 30);
let scene = movie.add_scene(5.0);

let box = scene.add_box(#{
    width: 200.0,
    height: 200.0,
    bg_color: "#FF0000"
});

// Set pivot to center
box.set_pivot(0.5, 0.5);

// Uniform scale
box.animate("scale", 0.0, 2.0, 2.0, "bounce_out");

// Individual axis scale
box.animate("scale_x", 1.0, 1.5, 1.0, "ease_in");
box.animate("scale_y", 1.0, 0.5, 1.0, "ease_out");

// Rotation
box.animate("rotation", 0.0, 360.0, 5.0, "linear");

// Skew
box.animate("skew_x", 0.0, 30.0, 2.0, "ease_in_out");
box.animate("skew_y", 0.0, -15.0, 2.0, "ease_in_out");

// Translation (aliases)
box.animate("x", 0.0, 100.0, 2.0, "linear");
box.animate("y", 0.0, 50.0, 2.0, "linear");

// Full property names
box.animate("translate_x", 100.0, 200.0, 1.0, "linear");
box.animate("translate_y", 50.0, 100.0, 1.0, "linear");
"##;

    let result = engine.run(script);
    assert!(
        result.is_ok(),
        "Transform script failed: {:?}",
        result.err()
    );
}

/// Test spring physics on transforms.
#[test]
fn transform_spring_animations() {
    let mut engine = Engine::new();
    register_rhai_api(&mut engine, Arc::new(DefaultAssetLoader));

    let script = r##"
let movie = new_director(800, 600, 30);
let scene = movie.add_scene(2.0);

let box = scene.add_box(#{
    width: 100.0,
    height: 100.0,
    bg_color: "#4ECDC4"
});

// Spring animations
box.animate("scale", 1.5, #{
    stiffness: 200.0,
    damping: 15.0
});

box.animate("rotation", 45.0, #{
    stiffness: 100.0,
    damping: 8.0
});

box.animate("x", 200.0, #{
    stiffness: 150.0,
    damping: 12.0
});
"##;

    let result = engine.run(script);
    assert!(result.is_ok(), "Spring script failed: {:?}", result.err());
}
