//! Scene Transition Tests
//!
//! Tests for transitions between scenes and ripple edit logic.

use director_core::{
    scripting::{register_rhai_api, MovieHandle},
    DefaultAssetLoader,
};
use rhai::Engine;
use std::sync::Arc;

/// Test transition ripple edit logic.
///
/// When a transition is added between two scenes, subsequent scenes
/// should shift earlier in the timeline by the transition duration.
#[test]
fn transition_ripple_edit() {
    let mut engine = Engine::new();
    register_rhai_api(&mut engine, Arc::new(DefaultAssetLoader));

    let script = r#"
let movie = new_director(1920, 1080, 30);

let s1 = movie.add_scene(10.0); // 0-10
let s2 = movie.add_scene(10.0); // 10-20
let s3 = movie.add_scene(10.0); // 20-30

// Add 2-second fade transition between s1 and s2
// This should shift s2 to start at 8.0 and s3 to 18.0
movie.add_transition(s1, s2, "fade", 2.0, "linear");

movie
"#;

    let result = engine.eval::<MovieHandle>(script);
    assert!(result.is_ok(), "Script failed: {:?}", result.err());

    let movie = result.unwrap();
    let director = movie.director.lock().unwrap();

    assert_eq!(director.timeline.len(), 3);
    assert_eq!(director.transitions.len(), 1);

    // S1 unchanged at 0.0
    let s1_item = &director.timeline[0];
    assert!(
        (s1_item.start_time - 0.0).abs() < 0.001,
        "S1 should start at 0.0, got {}",
        s1_item.start_time
    );

    // S2 shifted from 10.0 to 8.0 (ripple left by 2.0)
    let s2_item = &director.timeline[1];
    assert!(
        (s2_item.start_time - 8.0).abs() < 0.001,
        "S2 should start at 8.0, got {}",
        s2_item.start_time
    );

    // S3 shifted from 20.0 to 18.0
    let s3_item = &director.timeline[2];
    assert!(
        (s3_item.start_time - 18.0).abs() < 0.001,
        "S3 should start at 18.0, got {}",
        s3_item.start_time
    );

    // Transition metadata
    let transition = &director.transitions[0];
    assert_eq!(transition.start_time, 8.0);
    assert_eq!(transition.duration, 2.0);
}

/// Test all transition types parse correctly.
#[test]
fn transition_types_parsing() {
    let mut engine = Engine::new();
    register_rhai_api(&mut engine, Arc::new(DefaultAssetLoader));

    let transition_types = [
        "fade",
        "slide_left",
        "slide-left",
        "slide_right",
        "slide-right",
        "wipe_left",
        "wipe-left",
        "wipe_right",
        "wipe-right",
        "circle_open",
        "circle-open",
    ];

    for transition in transition_types {
        let script = format!(
            r#"
let movie = new_director(100, 100, 30);
let s1 = movie.add_scene(2.0);
let s2 = movie.add_scene(2.0);
movie.add_transition(s1, s2, "{}", 0.5, "linear");
movie
"#,
            transition
        );

        let result = engine.eval::<MovieHandle>(&script);
        assert!(
            result.is_ok(),
            "Transition '{}' should parse: {:?}",
            transition,
            result.err()
        );
    }
}
