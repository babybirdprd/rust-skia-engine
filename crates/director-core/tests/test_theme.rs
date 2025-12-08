use director_engine::{scripting::register_rhai_api, DefaultAssetLoader};
use rhai::Engine;
use std::sync::Arc;
use std::fs;

#[test]
fn test_theme_api() {
    let mut engine = Engine::new();
    let loader = Arc::new(DefaultAssetLoader);
    register_rhai_api(&mut engine, loader);

    let script = fs::read_to_string("tests/test_theme.rhai").expect("Failed to read test_theme.rhai");

    if let Err(e) = engine.run(&script) {
        panic!("Rhai script failed: {}", e);
    }
}
