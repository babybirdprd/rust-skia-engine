// verify_vfx.rhai
// Script to verify Vector Aspect Ratio and Vignette Shader Fix

fn run() {
    let width = 1920;
    let height = 1080;
    let fps = 30;
    let movie = new_director(width, height, fps);

    // Scene 1: Vector Aspect Ratio Test
    let scene1 = movie.add_scene(2.0);

    // Create a container with a specific aspect ratio (e.g., Square)
    // to test if the SVG (likely not square) is contained and centered.
    // SVG is assumed to be the Rust logo or similar from assets (typically wide).
    let container = scene1.add_box(#{
        width: "50%",
        height: "50%",
        bg_color: "#FF0000", // Red background to see letterboxing
        display: "flex",
        align_items: "center",
        justify_content: "center"
    });

    // Add SVG. Assuming 'vector_demo.mp4' implies there might be a vector asset or we can use a dummy.
    // I will try to use a built-in asset or a dummy path that the loader might handle if I mock it.
    // But since I'm running in the real env, I need a real file.
    // I'll create a simple SVG file for this test.
    container.add_svg("test_rect.svg", #{
        width: "100%", // Fill container
        height: "100%"
    });

    // Scene 2: Vignette Test
    let scene2 = movie.add_scene(2.0);

    // Box with Vignette
    let vbox = scene2.add_box(#{
        width: "100%",
        height: "100%",
        bg_color: "#00FF00" // Green background
    });

    // Apply Vignette with intensity 0.5
    // If bug exists (inverted), 0.5 might look okay, but 0.0 or 1.0 would be wrong.
    // Let's test 0.0 (Should be NO vignette)
    vbox.apply_effect("vignette", 0.0);

    // Let's add another box for 1.0 (Should be MAX vignette)
    let vbox_full = scene2.add_box(#{
        width: "50%",
        height: "50%",
        bg_color: "#0000FF",
        margin: "auto"
    });
    vbox_full.apply_effect("vignette", 1.0);
}

run();
