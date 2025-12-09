//! Comprehensive API showcase test (CPU rendering).
//! 
//! This test demonstrates the current Rhai API as documented in SCRIPTING.md and API.md.
//! It runs without any feature flags and uses CPU rasterization.

use director_core::{scripting::{register_rhai_api, MovieHandle}, DefaultAssetLoader};
use director_core::systems::renderer::render_export;
use rhai::Engine;
use std::sync::Arc;
use std::fs;
use std::path::PathBuf;

/// Test showcasing the full Rhai API.
/// 
/// This test creates a 2-second video demonstrating:
/// - Layout (Flexbox)
/// - Typography with rich text
/// - Animations (property + spring)
/// - Theme API usage
#[test]
fn test_api_showcase() {
    let mut engine = Engine::new();
    let loader = Arc::new(DefaultAssetLoader);
    register_rhai_api(&mut engine, loader);

    // Comprehensive script using documented API from SCRIPTING.md and API.md
    let script = r##"
// ============================================================
// API Showcase Test
// Based on SCRIPTING.md and API.md documentation
// ============================================================

// 1. Create movie with export mode
let movie = new_director(1920, 1080, 30, #{ mode: "export" });

// 2. Add a 2-second scene
let scene = movie.add_scene(2.0);

// 3. Create root container with Flexbox layout
let root = scene.add_box(#{
    width: "100%",
    height: "100%",
    flex_direction: "column",
    justify_content: "center",
    align_items: "center",
    bg_color: "#0a0a0a"
});

// 4. Use Theme API for consistent spacing
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

// 5. Add title with spring animation
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

// 6. Rich text with multiple spans
let subtitle = card.add_text(#{
    content: [
        #{ text: "CPU ", color: "#4ecdc4", size: 36.0 },
        #{ text: "Software ", color: "#ff6b6b", size: 36.0 },
        #{ text: "Rendering", color: "#ffe66d", size: 36.0 }
    ]
});

// 7. Animated box
let animated_box = card.add_box(#{
    width: 100.0,
    height: 100.0,
    bg_color: "#4ecdc4",
    border_radius: 20.0
});

animated_box.animate("rotation", 0.0, 360.0, 2.0, "linear");
animated_box.animate("scale", 0.8, 1.2, 1.0, "ease_in_out");

// 8. Footer with shrink-to-fit
let footer = card.add_text(#{
    content: "Powered by Skia",
    size: 24.0,
    color: "#888888",
    fit: "shrink",
    min_size: 12.0
});

movie
"##;

    let result = engine.eval::<MovieHandle>(&script).expect("Script failed");
    let mut director = result.director.lock().unwrap();

    let out_path = PathBuf::from("api_showcase.mp4");
    if out_path.exists() {
        fs::remove_file(&out_path).unwrap();
    }

    println!("Rendering 2-second showcase video (CPU)...");
    
    // CPU rendering - pass None for gpu_context
    render_export(&mut director, out_path.clone(), None, None).expect("Export failed");
    
    assert!(out_path.exists(), "Output video should exist");
    println!("Successfully rendered: {:?}", out_path);
    
    // // Cleanup
    // fs::remove_file(&out_path).ok();
}

/// Quick smoke test for the Rhai scripting engine.
#[test]
fn test_rhai_engine_basic() {
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
    println!("✓ Rhai engine initialized successfully");
}

/// Test layout system with flexbox properties.
#[test]
fn test_layout_flexbox() {
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

// Three equal boxes
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
    assert!(result.is_ok(), "Flexbox layout script should work");
    println!("✓ Flexbox layout test passed");
}

/// Test animation system.
#[test]
fn test_animations() {
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

// Property animation
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
    assert!(result.is_ok(), "Animation script should work");
    println!("✓ Animation test passed");
}

/// Test theme API.
#[test]
fn test_theme_api() {
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
    assert!(result.is_ok(), "Theme API script should work");
    println!("✓ Theme API test passed");
}
