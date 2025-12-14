//! Masking and Blend Mode Tests
//!
//! Tests for alpha masking (`set_mask`) and blend mode (`set_blend_mode`) APIs.

use director_core::{scripting::register_rhai_api, DefaultAssetLoader};
use rhai::Engine;
use std::sync::Arc;

/// Test mask assignment via Rhai API.
///
/// Validates:
/// - set_mask() moves node from parent to mask_node
/// - set_blend_mode() applies correctly
/// - Scene graph structure is correct after masking
#[test]
fn mask_assignment() {
    let mut engine = Engine::new();
    register_rhai_api(&mut engine, Arc::new(DefaultAssetLoader));

    let script = r##"
let movie = new_director(500, 500, 30);
let scene = movie.add_scene(1.0);

let container = scene.add_box(#{
    width: "100%",
    height: "100%",
    bg_color: "#000000"
});

// Content to be masked
let red_box = container.add_box(#{
    width: 200.0,
    height: 200.0,
    bg_color: "#FF0000"
});

// Mask shape
let mask = container.add_box(#{
    width: 100.0,
    height: 100.0,
    bg_color: "#FFFFFF",
    border_radius: 50.0
});

// Apply mask
red_box.set_mask(mask);
red_box.set_blend_mode("screen");

movie
"##;

    let result = engine.eval::<director_core::scripting::MovieHandle>(script);
    assert!(result.is_ok(), "Script failed: {:?}", result.err());

    let movie = result.unwrap();
    let mut director = movie.director.lock().unwrap();

    // Trigger update to validate traversal
    director.update(0.5);

    // Verify structure:
    // IDs: 0=root, 1=container, 2=red_box, 3=mask
    let red_box_node = director.scene.get_node(2).expect("Red box should exist");
    assert_eq!(
        red_box_node.mask_node,
        Some(3),
        "Mask node should be assigned"
    );

    let mask_node = director.scene.get_node(3).expect("Mask should exist");
    assert_eq!(mask_node.parent, Some(2), "Mask parent should be red_box");

    let container = director.scene.get_node(1).expect("Container should exist");
    assert!(
        !container.children.contains(&3),
        "Mask should be removed from container children"
    );
    assert!(
        container.children.contains(&2),
        "Red box should still be in container"
    );
}

/// Test all supported blend modes parse correctly.
#[test]
fn blend_modes_parsing() {
    let mut engine = Engine::new();
    register_rhai_api(&mut engine, Arc::new(DefaultAssetLoader));

    let blend_modes = [
        "normal",
        "multiply",
        "screen",
        "overlay",
        "darken",
        "lighten",
        "color_dodge",
        "color_burn",
        "hard_light",
        "soft_light",
        "difference",
        "exclusion",
        "hue",
        "saturation",
        "color",
        "luminosity",
    ];

    for mode in blend_modes {
        let script = format!(
            r##"
let movie = new_director(100, 100, 30);
let scene = movie.add_scene(1.0);
let box = scene.add_box(#{{ width: 50.0, height: 50.0 }});
box.set_blend_mode("{}");
movie
"##,
            mode
        );

        let result = engine.eval::<director_core::scripting::MovieHandle>(&script);
        assert!(
            result.is_ok(),
            "Blend mode '{}' should parse: {:?}",
            mode,
            result.err()
        );
    }
}
