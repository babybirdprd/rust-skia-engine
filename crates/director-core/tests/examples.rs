//! Example Script Validation Tests
//!
//! This module validates all example scripts in the `examples/` directory.
//! It ensures scripts:
//! - Parse without syntax errors
//! - Execute without runtime errors
//! - Return a valid MovieHandle

use director_core::{
    scripting::{register_rhai_api, MovieHandle},
    DefaultAssetLoader,
};
use rhai::Engine;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

/// Get all .rhai files from examples directory recursively.
fn collect_example_scripts() -> Vec<PathBuf> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workspace_root = PathBuf::from(manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let examples_dir = workspace_root.join("examples");

    let mut scripts = Vec::new();
    collect_rhai_files(&examples_dir, &mut scripts);
    scripts
}

fn collect_rhai_files(dir: &PathBuf, scripts: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_rhai_files(&path, scripts);
            } else if path.extension().map_or(false, |ext| ext == "rhai") {
                scripts.push(path);
            }
        }
    }
}

/// Validate that all example scripts parse and execute successfully.
#[test]
fn validate_all_example_scripts() {
    let scripts = collect_example_scripts();
    assert!(!scripts.is_empty(), "No example scripts found!");

    let mut engine = Engine::new();
    let loader = Arc::new(DefaultAssetLoader);
    register_rhai_api(&mut engine, loader);

    // Set working directory to workspace root for asset paths
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let manifest_path = PathBuf::from(manifest_dir);
    let workspace_root = manifest_path
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    std::env::set_current_dir(&workspace_root).ok();

    let mut passed = 0;
    let mut failed = Vec::new();

    for script_path in &scripts {
        let script_name = script_path.file_name().unwrap().to_string_lossy();

        let script = match fs::read_to_string(script_path) {
            Ok(s) => s,
            Err(e) => {
                failed.push(format!("{}: Failed to read - {}", script_name, e));
                continue;
            }
        };

        match engine.eval::<MovieHandle>(&script) {
            Ok(_) => {
                passed += 1;
            }
            Err(e) => {
                failed.push(format!("{}: {}", script_name, e));
            }
        }
    }

    println!("\n=== Example Script Validation ===");
    println!("Passed: {}/{}", passed, scripts.len());

    if !failed.is_empty() {
        println!("\nFailed scripts:");
        for failure in &failed {
            println!("  ✗ {}", failure);
        }
        panic!("{} example scripts failed validation", failed.len());
    }

    println!("✓ All {} example scripts validated successfully", passed);
}

/// Test each example script individually for better error reporting.
#[test]
fn example_basics_hello_world() {
    validate_script("examples/basics/hello_world.rhai");
}

#[test]
fn example_basics_layout_flexbox() {
    validate_script("examples/basics/layout_flexbox.rhai");
}

#[test]
fn example_basics_animation() {
    validate_script("examples/basics/animation.rhai");
}

#[test]
fn example_basics_text() {
    validate_script("examples/basics/text.rhai");
}

#[test]
fn example_features_effects() {
    validate_script("examples/features/effects.rhai");
}

#[test]
fn example_features_masking() {
    validate_script("examples/features/masking.rhai");
}

#[test]
fn example_features_transitions() {
    validate_script("examples/features/transitions.rhai");
}

#[test]
fn example_features_z_index() {
    validate_script("examples/features/z_index.rhai");
}

#[test]
fn example_features_image() {
    validate_script("examples/features/image.rhai");
}

fn validate_script(relative_path: &str) {
    let mut engine = Engine::new();
    let loader = Arc::new(DefaultAssetLoader);
    register_rhai_api(&mut engine, loader);

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let manifest_path = PathBuf::from(manifest_dir);
    let workspace_root = manifest_path
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    std::env::set_current_dir(&workspace_root).ok();

    let script_path = workspace_root.join(relative_path);
    let script = fs::read_to_string(&script_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", relative_path, e));

    let result = engine.eval::<MovieHandle>(&script);
    assert!(
        result.is_ok(),
        "Script {} failed: {:?}",
        relative_path,
        result.err()
    );
}
