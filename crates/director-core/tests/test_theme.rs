use director_core::{scripting::register_rhai_api, DefaultAssetLoader};
use rhai::Engine;
use std::sync::Arc;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_theme_api() {
    let mut engine = Engine::new();
    let loader = Arc::new(DefaultAssetLoader);
    register_rhai_api(&mut engine, loader);

    // Try finding the file relative to CARGO_MANIFEST_DIR
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/test_theme.rhai");

    let script = fs::read_to_string(&path).expect(&format!("Failed to read test_theme.rhai at {:?}", path));

    if let Err(e) = engine.run(&script) {
        panic!("Rhai script failed: {}", e);
    }
}
