//! Video Rendering Tests
//!
//! Tests for VideoNode behavior in different render modes.

use director_core::{
    director::{Director, TimelineItem},
    node::{VideoNode, VideoSource},
    video_wrapper::RenderMode,
    AssetLoader,
};
use std::sync::Arc;
use std::time::Instant;

struct MockAssetLoader;
impl AssetLoader for MockAssetLoader {
    fn load_bytes(&self, _path: &str) -> anyhow::Result<Vec<u8>> {
        Err(anyhow::anyhow!("Mock loader - no assets"))
    }
}

/// Test that VideoNode in Export mode blocks during update.
///
/// In Export mode, video decoding should be synchronous to ensure
/// frame-accurate output. This test verifies that update() takes
/// measurable time when a video is being decoded.
///
/// Note: This test is ignored by default as it requires a real video file.
#[test]
#[ignore = "Requires test video asset. Run with: cargo test video -- --ignored"]
fn video_export_mode_blocks() {
    let asset_loader = Arc::new(MockAssetLoader);
    let mut director = Director::new(100, 100, 60, asset_loader, RenderMode::Export, None);

    // This test requires a video file at assets/test_video.mp4
    // Generate with: ffmpeg -f lavfi -i testsrc=duration=2:size=100x100:rate=30 -pix_fmt yuv420p assets/test_video.mp4
    let video_path = std::path::PathBuf::from("assets/test_video.mp4");

    if !video_path.exists() {
        // Try workspace root
        let workspace_path = std::path::PathBuf::from("../../assets/test_video.mp4");
        if !workspace_path.exists() {
            panic!(
                "Test video not found at {:?} or {:?}. \
                 Generate with: ffmpeg -f lavfi -i testsrc=duration=2:size=100x100:rate=30 -pix_fmt yuv420p assets/test_video.mp4",
                video_path, workspace_path
            );
        }
    }

    let abs_path = std::fs::canonicalize(&video_path).unwrap();
    let video_node = VideoNode::new(VideoSource::Path(abs_path), RenderMode::Export);
    let node_id = director.scene.add_node(Box::new(video_node));

    director.timeline.push(TimelineItem {
        scene_root: node_id,
        start_time: 0.0,
        duration: 10.0,
        z_index: 0,
        audio_tracks: vec![],
    });

    let start = Instant::now();
    for i in 0..10 {
        director.update(i as f64 * 0.1);
    }
    let duration = start.elapsed();

    assert!(
        duration.as_millis() >= 5,
        "Export mode should block for decoding. Took {:?}",
        duration
    );
}

/// Test that VideoNode can be created with bytes source.
#[test]
fn video_bytes_source() {
    let asset_loader = Arc::new(MockAssetLoader);
    let mut director = Director::new(100, 100, 30, asset_loader, RenderMode::Preview, None);

    // Empty bytes should fail gracefully
    let video_node = VideoNode::new(VideoSource::Bytes(vec![]), RenderMode::Preview);
    let node_id = director.scene.add_node(Box::new(video_node));

    director.timeline.push(TimelineItem {
        scene_root: node_id,
        start_time: 0.0,
        duration: 1.0,
        z_index: 0,
        audio_tracks: vec![],
    });

    // Should not panic
    director.update(0.0);
}
