use director_engine::scripting::register_rhai_api;
use rhai::Engine;
use director_engine::render::render_export;
use std::path::PathBuf;

fn main() {
    println!("Initializing Director Engine...");

    let mut engine = Engine::new();
    register_rhai_api(&mut engine);

    // Use ## as delimiter because script contains " (double quote) and # (hash)
    let script = r##"
        let movie = new_director(1080, 1920, 30);
        let scene = movie.add_scene(5.0);

        let box = scene.add_box(#{
            bg_color: "#FF0000",
            // Layout properties
            width: "80%",
            height: "50%",
            flex_direction: "column",
            align_items: "center",
            justify_content: "center"
        });

        let text = box.add_text(#{
            content: "Hello World"
        });

        // Animate size from 20.0 to 100.0 over 2.0 seconds
        text.animate("size", 20.0, 100.0, 2.0, "bounce_out");

        // Example of adding Image and Video (ensure files exist)
        // box.add_image("assets/logo.png");
        // box.add_video("assets/clip.mp4");

        movie
    "##;

    match engine.eval::<director_engine::scripting::MovieHandle>(script) {
        Ok(movie) => {
            println!("Script evaluated successfully. Starting render...");
            let mut director = movie.director.lock().unwrap();
            match render_export(&mut director, PathBuf::from("output.mp4"), None) {
                Ok(_) => println!("Render complete: output.mp4"),
                Err(e) => println!("Render failed: {}", e),
            }
        },
        Err(e) => println!("Script Error: {}", e),
    }
}
