use director_core::systems::renderer::render_export;
use director_core::scripting::register_rhai_api;
use director_core::DefaultAssetLoader;
use rhai::Engine;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: director-engine <script.rhai> [output.mp4]");
        return;
    }

    let script_path = PathBuf::from(&args[1]);
    let output_path = if args.len() >= 3 {
        PathBuf::from(&args[2])
    } else {
        let mut p = script_path.clone();
        p.set_extension("mp4");
        // If script was examples/showcase.rhai, output is examples/showcase.mp4
        // To keep repo clean, maybe default to root?
        // But following standard behavior (ffmpeg etc), same dir is expected.
        // I will force it to be in the current directory if not specified to avoid cluttering examples?
        // No, let's just default to replacing extension.
        p
    };

    println!("Initializing Director Engine...");
    println!("Script: {:?}", script_path);
    println!("Output: {:?}", output_path);

    let script = match fs::read_to_string(&script_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading script file: {}", e);
            return;
        }
    };

    let mut engine = Engine::new();
    register_rhai_api(&mut engine, Arc::new(DefaultAssetLoader));

    match engine.eval::<director_core::scripting::MovieHandle>(&script) {
        Ok(movie) => {
            println!("Script evaluated successfully. Starting render...");
            let mut director = movie.director.lock().unwrap();
            match render_export(&mut director, output_path, None, None) {
                Ok(_) => println!("Render complete."),
                Err(e) => {
                    eprintln!("Render failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Script Error: {}", e);
            std::process::exit(1);
        }
    }
}
