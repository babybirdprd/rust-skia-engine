//! GENESIS Cinematic Showcase Test
//!
//! This test renders the premium cinematic showcase script.
//! Output: output/cinematic_showcase.mp4

use director_core::{scripting::{register_rhai_api, MovieHandle}, DefaultAssetLoader};
use director_core::systems::renderer::render_export;
use rhai::Engine;
use std::sync::Arc;
use std::fs;

#[test]
fn test_cinematic_showcase() {
    let mut engine = Engine::new();
    let loader = Arc::new(DefaultAssetLoader);
    register_rhai_api(&mut engine, loader);

    // Read the showcase script from examples
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workspace_root = std::path::Path::new(manifest_dir).parent().unwrap().parent().unwrap();
    
    // Set current dir to workspace root for asset paths
    std::env::set_current_dir(workspace_root).ok();
    
    let script_path = workspace_root.join("examples/cinematic_showcase.rhai");
    let script = fs::read_to_string(&script_path).expect("Failed to read cinematic_showcase.rhai");

    println!("🎬 Executing GENESIS showcase script...");
    let result = engine.eval::<MovieHandle>(&script).expect("Script execution failed");
    let mut director = result.director.lock().unwrap();

    // Create output directory if needed
    let output_dir = workspace_root.join("output");
    fs::create_dir_all(&output_dir).ok();

    let out_path = output_dir.join("cinematic_showcase.mp4");
    if out_path.exists() {
        fs::remove_file(&out_path).unwrap();
    }

    println!("🎥 Rendering cinematic showcase (this may take a moment)...");
    render_export(&mut director, out_path.clone(), None, None).expect("Export failed");

    assert!(out_path.exists(), "Output video should exist at {:?}", out_path);
    
    let metadata = fs::metadata(&out_path).unwrap();
    println!("✅ Successfully rendered: {:?} ({} bytes)", out_path, metadata.len());
}
