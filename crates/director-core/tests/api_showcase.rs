//! API Showcase and Smoke Tests
//!
//! This module contains comprehensive tests that demonstrate the Rhai API
//! as documented in SCRIPTING.md and API.md.

use director_core::{
    scripting::{register_rhai_api, MovieHandle},
    DefaultAssetLoader,
};
use rhai::Engine;
use std::sync::Arc;

/// Quick smoke test for the Rhai scripting engine.
///
/// Verifies that the engine initializes and can execute a minimal script.
#[test]
fn smoke_test_rhai_engine() {
    let mut engine = Engine::new();
    let loader = Arc::new(DefaultAssetLoader);
    register_rhai_api(&mut engine, loader);

    let script = r##"
let movie = new_director(1920, 1080, 30);
let scene = movie.add_scene(1.0);
scene.add_text(#{ content: "Test", size: 48.0, color: "#FFFFFF" });
movie
"##;

    let result = engine.eval::<MovieHandle>(&script);
    assert!(result.is_ok(), "Basic script should execute without error");
}

/// Test Flexbox layout properties via Rhai API.
///
/// Validates:
/// - flex_direction
/// - justify_content
/// - align_items
/// - Percentage sizing
/// - Padding
#[test]
fn layout_flexbox() {
    let mut engine = Engine::new();
    let loader = Arc::new(DefaultAssetLoader);
    register_rhai_api(&mut engine, loader);

    let script = r##"
let movie = new_director(800, 600, 30);
let scene = movie.add_scene(1.0);

let container = scene.add_box(#{
    width: "100%",
    height: "100%",
    flex_direction: "row",
    justify_content: "space_between",
    align_items: "center",
    padding: 20.0
});

// Three colored boxes
for i in 0..3 {
    container.add_box(#{
        width: 150.0,
        height: 150.0,
        bg_color: if i == 0 { "#FF0000" } else if i == 1 { "#00FF00" } else { "#0000FF" },
        border_radius: 10.0
    });
}

movie
"##;

    let result = engine.eval::<MovieHandle>(&script);
    assert!(
        result.is_ok(),
        "Flexbox layout script should execute: {:?}",
        result.err()
    );
}

/// Test keyframe and spring animation APIs.
///
/// Validates:
/// - Property animation with easing
/// - Spring physics animation
/// - Multiple concurrent animations
#[test]
fn animation_keyframes_and_springs() {
    let mut engine = Engine::new();
    let loader = Arc::new(DefaultAssetLoader);
    register_rhai_api(&mut engine, loader);

    let script = r##"
let movie = new_director(1920, 1080, 30);
let scene = movie.add_scene(2.0);

let box = scene.add_box(#{
    width: 100.0,
    height: 100.0,
    bg_color: "#FF6B6B"
});

// Keyframe animations
box.animate("rotation", 0.0, 360.0, 2.0, "ease_in_out");
box.animate("scale", 1.0, 1.5, 1.0, "bounce_out");

// Spring animation
box.animate("opacity", 1.0, #{
    stiffness: 100.0,
    damping: 10.0
});

movie
"##;

    let result = engine.eval::<MovieHandle>(&script);
    assert!(
        result.is_ok(),
        "Animation script should execute: {:?}",
        result.err()
    );
}

/// Test theme API (design tokens).
///
/// Validates:
/// - theme::space()
/// - theme::radius()
/// - theme::safe_area()
#[test]
fn theme_design_tokens() {
    let mut engine = Engine::new();
    let loader = Arc::new(DefaultAssetLoader);
    register_rhai_api(&mut engine, loader);

    let script = r##"
let movie = new_director(1920, 1080, 30);
let scene = movie.add_scene(1.0);

let card = scene.add_box(#{
    width: 300.0,
    height: 200.0,
    padding: theme::space("md"),
    border_radius: theme::radius("lg"),
    bg_color: "#333333"
});

let safe = theme::safe_area("tiktok");

movie
"##;

    let result = engine.eval::<MovieHandle>(&script);
    assert!(
        result.is_ok(),
        "Theme API script should execute: {:?}",
        result.err()
    );
}

/// Test rich text with multiple styled spans.
///
/// Validates:
/// - Array-based content with per-span styling
/// - Font size, color, weight per span
#[test]
fn text_rich_spans() {
    let mut engine = Engine::new();
    let loader = Arc::new(DefaultAssetLoader);
    register_rhai_api(&mut engine, loader);

    let script = r##"
let movie = new_director(1920, 1080, 30);
let scene = movie.add_scene(1.0);

let root = scene.add_box(#{
    width: "100%",
    height: "100%",
    justify_content: "center",
    align_items: "center"
});

let title = root.add_text(#{
    content: [
        #{ text: "Director ", color: "#4ecdc4", size: 48.0, weight: "bold" },
        #{ text: "Engine", color: "#ff6b6b", size: 48.0 }
    ]
});

movie
"##;

    let result = engine.eval::<MovieHandle>(&script);
    assert!(
        result.is_ok(),
        "Rich text script should execute: {:?}",
        result.err()
    );
}

/// Test effects API.
///
/// Validates:
/// - apply_effect with preset filters
/// - Blur effect
#[test]
fn effects_blur_and_color_matrix() {
    let mut engine = Engine::new();
    let loader = Arc::new(DefaultAssetLoader);
    register_rhai_api(&mut engine, loader);

    let script = r##"
let movie = new_director(800, 600, 30);
let scene = movie.add_scene(1.0);

let box = scene.add_box(#{
    width: 200.0,
    height: 200.0,
    bg_color: "#FF0000"
});

// Apply blur
box.apply_effect("blur", 5.0);

// Create another box with grayscale
let gray_box = scene.add_box(#{
    width: 200.0,
    height: 200.0,
    bg_color: "#00FF00"
});
gray_box.apply_effect("grayscale");

movie
"##;

    let result = engine.eval::<MovieHandle>(&script);
    assert!(
        result.is_ok(),
        "Effects script should execute: {:?}",
        result.err()
    );
}

// =============================================================================
// E2E Tests (Video Generation) - Run with: cargo test -- --ignored
// =============================================================================

/// Full API showcase - generates a video demonstrating all features.
///
/// This is a slow test that renders an actual video file.
/// Run with: `cargo test api_showcase_video_export -- --ignored`
#[test]
#[ignore = "Slow: Generates video file. Run with --ignored flag."]
fn api_showcase_video_export() {
    use director_core::export::render_export;
    use std::fs;
    use std::path::PathBuf;

    let mut engine = Engine::new();
    let loader = Arc::new(DefaultAssetLoader);
    register_rhai_api(&mut engine, loader);

    let script = r##"
// Full API Showcase
let movie = new_director(1920, 1080, 30, #{ mode: "export" });
let scene = movie.add_scene(2.0);

let root = scene.add_box(#{
    width: "100%",
    height: "100%",
    flex_direction: "column",
    justify_content: "center",
    align_items: "center",
    bg_color: "#0a0a0a"
});

let card = root.add_box(#{
    width: "80%",
    height: "60%",
    padding: theme::space("lg"),
    border_radius: theme::radius("lg"),
    bg_color: "#1a1a2e",
    shadow_color: "#000000",
    shadow_blur: 30.0,
    shadow_y: 10.0,
    flex_direction: "column",
    align_items: "center",
    justify_content: "space_evenly"
});

let title = card.add_text(#{
    content: "Director Engine",
    size: 72.0,
    color: "#ffffff",
    weight: "bold"
});

title.animate("scale", 1.2, #{
    stiffness: 150.0,
    damping: 12.0
});

let subtitle = card.add_text(#{
    content: [
        #{ text: "Video ", color: "#4ecdc4", size: 36.0 },
        #{ text: "Rendering ", color: "#ff6b6b", size: 36.0 },
        #{ text: "Engine", color: "#ffe66d", size: 36.0 }
    ]
});

let animated_box = card.add_box(#{
    width: 100.0,
    height: 100.0,
    bg_color: "#4ecdc4",
    border_radius: 20.0
});

animated_box.animate("rotation", 0.0, 360.0, 2.0, "linear");
animated_box.animate("scale", 0.8, 1.2, 1.0, "ease_in_out");

movie
"##;

    let result = engine.eval::<MovieHandle>(&script).expect("Script failed");
    let mut director = result.director.lock().unwrap();

    let out_path = PathBuf::from("target/api_showcase.mp4");
    if out_path.exists() {
        fs::remove_file(&out_path).unwrap();
    }

    render_export(&mut director, out_path.clone(), None, None).expect("Export failed");

    assert!(out_path.exists(), "Output video should exist");
    println!("✅ Rendered: {:?}", out_path);
}
