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
            bg_color: "#FF0000"
        });

        let text = box.add_text(#{
            content: "Hello World"
        });

        // "size" is mapped to font_size in our Node impl
        text.animate("size", 20.0, 100.0, 2.0, "bounce_out");

        movie
    "##;

    match engine.eval::<director_engine::scripting::MovieHandle>(script) {
        Ok(movie) => {
            println!("Script evaluated successfully. Starting render...");
            let mut director = movie.director.lock().unwrap();
            match render_export(&mut director, PathBuf::from("output.mp4")) {
                Ok(_) => println!("Render complete: output.mp4"),
                Err(e) => println!("Render failed: {}", e),
            }
        },
        Err(e) => println!("Script Error: {}", e),
    }
}
