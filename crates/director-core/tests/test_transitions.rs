use director_engine::{scripting::register_rhai_api, DefaultAssetLoader, scripting::MovieHandle};
use rhai::Engine;
use std::sync::Arc;

#[test]
fn test_transition_ripple() {
    let mut engine = Engine::new();
    let loader = Arc::new(DefaultAssetLoader);
    register_rhai_api(&mut engine, loader);

    let script = r#"
        let movie = new_director(1920, 1080, 30);

        let s1 = movie.add_scene(10.0); // Ends at 10.0
        let s2 = movie.add_scene(10.0); // Ends at 20.0
        let s3 = movie.add_scene(10.0); // Ends at 30.0

        // Add transition between s1 and s2. Duration 2.0.
        // Expect s2 start to shift to 8.0.
        // Expect s3 start to shift to 18.0.

        movie.add_transition(s1, s2, "fade", 2.0, "linear");

        // Return movie to inspect in Rust
        movie
    "#;

    let result = engine.eval::<MovieHandle>(script);
    if let Err(e) = &result {
        println!("Rhai error: {}", e);
    }
    assert!(result.is_ok());
    let movie = result.unwrap();
    let director = movie.director.lock().unwrap();

    assert_eq!(director.timeline.len(), 3);
    assert_eq!(director.transitions.len(), 1);

    let s2_item = &director.timeline[1];
    let s3_item = &director.timeline[2];

    assert!((s2_item.start_time - 8.0).abs() < 0.001, "S2 start time should be 8.0, got {}", s2_item.start_time);
    assert!((s3_item.start_time - 18.0).abs() < 0.001, "S3 start time should be 18.0, got {}", s3_item.start_time);

    let t = &director.transitions[0];
    assert_eq!(t.start_time, 8.0);
    assert_eq!(t.duration, 2.0);
}
