//! Nested Composition Tests
//!
//! Tests for CompositionNode (pre-comps / nested timelines).

use director_core::{scripting, systems::renderer::render_frame, AssetLoader};
use rhai::Engine;
use std::sync::Arc;

struct MockLoader;
impl AssetLoader for MockLoader {
    fn load_bytes(&self, _path: &str) -> anyhow::Result<Vec<u8>> {
        Ok(vec![])
    }
}

/// Test nested composition workflow.
///
/// Validates:
/// - Creating a composition (sub-director)
/// - Adding composition to main movie
/// - Rendering produces correct pixel output
#[test]
fn composition_nesting() {
    let mut engine = Engine::new();
    scripting::register_rhai_api(&mut engine, Arc::new(MockLoader));

    let script = r##"
// 1. Define component (100x100 red box)
let comp = new_director(100, 100, 30);
let s1 = comp.add_scene(1.0);
s1.add_box(#{
    width: 100.0,
    height: 100.0,
    bg_color: "#FF0000"
});

// 2. Create main movie
let main = new_director(200, 200, 30);
let s2 = main.add_scene(1.0);

// 3. Add composition instance
let inst = s2.add_composition(comp, #{ width: 100.0, height: 100.0 });
inst.animate("x", 50.0, 50.0, 0.0, "linear");
inst.animate("y", 50.0, 50.0, 0.0, "linear");

main
"##;

    let result = engine.eval::<scripting::MovieHandle>(script);
    assert!(result.is_ok(), "Script failed: {:?}", result.err());

    let movie = result.unwrap();
    let mut director = movie.director.lock().unwrap();

    // Render frame 0
    let mut surface = skia_safe::surfaces::raster_n32_premul((200, 200)).unwrap();
    render_frame(&mut *director, 0.0, surface.canvas()).unwrap();

    // Read pixels
    let mut pixels = vec![0u8; 200 * 200 * 4];
    let info = skia_safe::ImageInfo::new_n32_premul((200, 200), None);
    assert!(surface.read_pixels(&info, &mut pixels, 200 * 4, (0, 0)));

    let get_pixel = |x: usize, y: usize| -> (u8, u8, u8, u8) {
        let idx = (y * 200 + x) * 4;
        (
            pixels[idx],
            pixels[idx + 1],
            pixels[idx + 2],
            pixels[idx + 3],
        )
    };

    // Center should be red (from composition)
    let center = get_pixel(100, 100);
    assert!(center.3 > 0, "Center should be opaque");

    // Check for red (platform-dependent byte order)
    let is_red_rgba = center.0 > 200 && center.1 < 50 && center.2 < 50;
    let is_red_bgra = center.2 > 200 && center.1 < 50 && center.0 < 50;
    assert!(
        is_red_rgba || is_red_bgra,
        "Center pixel should be red, got {:?}",
        center
    );
}

/// Test that self-referencing compositions are prevented.
#[test]
fn composition_cycle_detection() {
    let mut engine = Engine::new();
    scripting::register_rhai_api(&mut engine, Arc::new(MockLoader));

    // This script attempts to add a composition to itself
    // The engine should handle this gracefully (returns dummy node, logs error)
    let script = r##"
let movie = new_director(100, 100, 30);
let scene = movie.add_scene(1.0);

// This should NOT crash - cycle detection should prevent it
scene.add_composition(movie, #{ width: 50.0, height: 50.0 });

movie
"##;

    let result = engine.eval::<scripting::MovieHandle>(script);
    // Should succeed (cycle is handled gracefully with a warning)
    assert!(result.is_ok(), "Cycle detection should not crash");
}
