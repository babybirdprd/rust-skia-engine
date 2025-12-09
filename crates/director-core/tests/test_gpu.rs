//! GPU-accelerated rendering test using Vulkan backend.
//! 
//! This test demonstrates the current Rhai API as documented in SCRIPTING.md and API.md.
//! It is only compiled when the `vulkan` feature is enabled.

#![cfg(feature = "vulkan")]

use director_core::{scripting::{register_rhai_api, MovieHandle}, DefaultAssetLoader};
use director_core::systems::renderer::render_export;
use rhai::Engine;
use std::sync::Arc;
use std::fs;
use std::path::PathBuf;

use skia_safe::gpu::{self, SurfaceOrigin, Budgeted};

/// Attempts to create a Vulkan DirectContext for GPU-accelerated rendering.
/// 
/// Returns None if Vulkan initialization fails (no GPU, drivers not installed, etc.)
/// In a full implementation, you would use `ash` or `vulkano` to create the Vulkan context.
fn try_create_vulkan_context() -> Option<gpu::DirectContext> {
    // Placeholder - actual Vulkan initialization requires:
    // 1. Create VkInstance via ash/vulkano
    // 2. Select physical device
    // 3. Create logical device and queue
    // 4. Pass to skia via BackendContext
    None
}

/// Test showcasing the full Rhai API with GPU support.
/// 
/// This test creates a 2-second video demonstrating:
/// - Layout (Flexbox)
/// - Typography with rich text
/// - Animations (property + spring)
/// - Theme API usage
/// - Visual effects
#[test]
fn test_gpu_full_api_showcase() {
    let mut engine = Engine::new();
    let loader = Arc::new(DefaultAssetLoader);
    register_rhai_api(&mut engine, loader);

    // Comprehensive script using documented API from SCRIPTING.md and API.md
    let script = r##"
// ============================================================
// GPU Test: Full API Showcase
// Based on SCRIPTING.md and API.md documentation
// ============================================================

// 1. Create movie with export mode for high quality
let movie = new_director(1920, 1080, 30, #{ mode: "export" });

// 2. Add a 2-second scene (shorter for faster GPU tests)
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

// 5. Add title with animation
let title = card.add_text(#{
    content: "Director Engine",
    size: 72.0,
    color: "#ffffff",
    weight: "bold"
});

// Animate title scale using spring physics
title.animate("scale", 1.2, #{
    stiffness: 150.0,
    damping: 12.0
});

// 6. Add subtitle with rich text (multiple spans)
let subtitle = card.add_text(#{
    content: [
        #{ text: "GPU ", color: "#4ecdc4", size: 36.0 },
        #{ text: "Accelerated ", color: "#ff6b6b", size: 36.0 },
        #{ text: "Rendering", color: "#ffe66d", size: 36.0 }
    ]
});

// 7. Add animated box
let animated_box = card.add_box(#{
    width: 100.0,
    height: 100.0,
    bg_color: "#4ecdc4",
    border_radius: 20.0
});

// Property animations
animated_box.animate("rotation", 0.0, 360.0, 2.0, "linear");
animated_box.animate("scale", 0.8, 1.2, 1.0, "ease_in_out");

// 8. Add footer text with shrink-to-fit
let footer = card.add_text(#{
    content: "Powered by Skia + Vulkan",
    size: 24.0,
    color: "#888888",
    fit: "shrink",
    min_size: 12.0
});

// Return the movie
movie
"##;

    let result = engine.eval::<MovieHandle>(&script).expect("Script failed");
    let mut director = result.director.lock().unwrap();

    // Try to get GPU context (falls back to CPU if unavailable)
    let mut gpu_ctx = try_create_vulkan_context();
    
    let out_path = PathBuf::from("gpu_api_showcase.mp4");
    if out_path.exists() {
        fs::remove_file(&out_path).unwrap();
    }

    println!("GPU Context Available: {}", gpu_ctx.is_some());
    println!("Rendering 2-second showcase video...");
    
    let gpu_ref = gpu_ctx.as_mut();
    render_export(&mut director, out_path.clone(), gpu_ref, None).expect("Export failed");
    
    assert!(out_path.exists(), "Output video should exist");
    println!("Successfully rendered: {:?}", out_path);
    
    // Cleanup
    fs::remove_file(&out_path).ok();
}

/// Test GPU surface creation capability.
#[test]
fn test_gpu_surface_creation() {
    use skia_safe::{ImageInfo, ColorType, AlphaType, ColorSpace};
    
    let info = ImageInfo::new(
        (1920, 1080),
        ColorType::RGBA8888,
        AlphaType::Premul,
        Some(ColorSpace::new_srgb()),
    );
    
    if let Some(mut ctx) = try_create_vulkan_context() {
        let surface = skia_safe::gpu::surfaces::render_target(
            &mut ctx,
            Budgeted::Yes,
            &info,
            0,
            SurfaceOrigin::TopLeft,
            None,
            false,
            None,
        );
        
        if surface.is_some() {
            println!("✓ GPU surface created successfully");
        } else {
            println!("✗ GPU context exists but surface creation failed");
        }
    } else {
        // Fall back to CPU - expected on systems without Vulkan
        let surface = skia_safe::surfaces::raster(&info, None, None);
        assert!(surface.is_some(), "CPU surface should always work");
        println!("✓ Using CPU raster surface (Vulkan not available)");
    }
}

/// Quick smoke test for the Rhai scripting engine.
#[test]
fn test_rhai_engine_basic() {
    let mut engine = Engine::new();
    let loader = Arc::new(DefaultAssetLoader);
    register_rhai_api(&mut engine, loader);

    // Minimal script that just creates director and scene
    let script = r##"
let movie = new_director(640, 480, 30);
let scene = movie.add_scene(1.0);
scene.add_text(#{ content: "Test", size: 48.0, color: "#FFFFFF" });
movie
"##;

    let result = engine.eval::<MovieHandle>(&script);
    assert!(result.is_ok(), "Basic script should execute without error");
    println!("✓ Rhai engine initialized successfully");
}
