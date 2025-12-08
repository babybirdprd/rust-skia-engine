use director_engine::{scripting, AssetLoader, render::render_frame};
use std::sync::Arc;
use rhai::Engine;

struct MockLoader;
impl AssetLoader for MockLoader {
    fn load_bytes(&self, _path: &str) -> anyhow::Result<Vec<u8>> {
        Ok(vec![])
    }
}

#[test]
fn test_composition_workflow() {
    let mut engine = Engine::new();
    scripting::register_rhai_api(&mut engine, Arc::new(MockLoader));

    // Use ## delimiter to avoid conflict with "#Color"
    let script = r##"
        // 1. Define Component (Red Box 100x100)
        let comp = new_director(100, 100, 30);
        let s1 = comp.add_scene(1.0);
        s1.add_box(#{
            width: 100,
            height: 100,
            bg_color: "#FF0000"
        });

        // 2. Main Movie
        let main = new_director(200, 200, 30);
        let s2 = main.add_scene(1.0);

        // Add composition at (50, 50)
        let inst = s2.add_composition(comp, #{ width: 100, height: 100 });
        // Animate position to verify it moves
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
    let canvas = surface.canvas();

    // Time 0.0
    render_frame(&mut *director, 0.0, canvas);

    let mut pixels = vec![0u8; 200 * 200 * 4];
    let info = skia_safe::ImageInfo::new_n32_premul((200, 200), None);
    assert!(surface.read_pixels(&info, &mut pixels, 200 * 4, (0, 0)));

    // Helper to get pixel
    let get_pixel = |x: usize, y: usize| -> (u8, u8, u8, u8) {
        let idx = (y * 200 + x) * 4;
        (pixels[idx], pixels[idx+1], pixels[idx+2], pixels[idx+3])
    };

    let p_center = get_pixel(100, 100);
    println!("Pixel at 100,100: {:?}", p_center);

    let p_corner = get_pixel(10, 10);
    println!("Pixel at 10,10: {:?}", p_corner);

    // Alpha check (p.3)
    assert!(p_center.3 > 0, "Center should be opaque-ish");

    // Red check.
    // Skia N32 is platform dependent.
    // If RGBA: 255, 0, 0, 255.
    // If BGRA: 0, 0, 255, 255.
    // We check if either R or B is high and G is low.

    let is_red_rgba = p_center.0 > 200 && p_center.1 < 50 && p_center.2 < 50;
    let is_red_bgra = p_center.2 > 200 && p_center.1 < 50 && p_center.0 < 50;

    assert!(is_red_rgba || is_red_bgra, "Center pixel should be red");
}
