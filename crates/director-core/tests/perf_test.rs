use director_core::{Director, DefaultAssetLoader, director::DirectorContext, video_wrapper::RenderMode};
use std::sync::Arc;
use std::time::Instant;

#[test]
fn test_director_instantiation_performance() {
    let loader = Arc::new(DefaultAssetLoader);
    let iterations = 20;

    // 1. Without Shared Context
    let start_fresh = Instant::now();
    for _ in 0..iterations {
        let _ = Director::new(500, 500, 60, loader.clone(), RenderMode::Preview, None);
    }
    let duration_fresh = start_fresh.elapsed();
    println!("Fresh Creation ({} iters): {:?}", iterations, duration_fresh);

    // 2. With Shared Context
    // First, create the context (simulate the parent)
    let parent = Director::new(500, 500, 60, loader.clone(), RenderMode::Preview, None);
    let context = DirectorContext {
        assets: parent.assets.clone(),
    };

    let start_shared = Instant::now();
    for _ in 0..iterations {
        let _ = Director::new(500, 500, 60, loader.clone(), RenderMode::Preview, Some(context.clone()));
    }
    let duration_shared = start_shared.elapsed();
    println!("Shared Creation ({} iters): {:?}", iterations, duration_shared);

    // Assert that shared is at least 2x faster (it should be much more)
    // Only run assert if fresh took non-trivial time (e.g., > 10ms)
    // In CI environments without fonts, font scanning might be fast (no fonts), or slow (filesystem).
    // Assuming some fonts exist or fallback check happens.

    if duration_fresh.as_millis() > 10 {
        assert!(duration_shared < duration_fresh / 2, "Shared context should be significantly faster");
    }
}
