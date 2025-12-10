use director_core::director::{Director, TimelineItem};
use director_core::video_wrapper::RenderMode;
use director_core::node::{VideoNode, VideoSource};
use std::sync::Arc;
use std::path::PathBuf;
use std::time::Instant;
use std::process::Command;

struct MockAssetLoader;
impl director_core::AssetLoader for MockAssetLoader {
    fn load_bytes(&self, _path: &str) -> anyhow::Result<Vec<u8>> {
        Err(anyhow::anyhow!("Mock asset not found"))
    }
    fn load_font_fallback(&self) -> Option<Vec<u8>> { None }
}

#[test]
fn test_sync_video_decoding_blocks() {
    let asset_loader = Arc::new(MockAssetLoader);
    let mut director = Director::new(100, 100, 60, asset_loader, RenderMode::Export, None);

    let mut video_path = PathBuf::from("assets/test_video.mp4");

    // Check if we need to generate or find the video
    if !video_path.exists() {
        // Check if we are in crate directory and assets is in parent
        let alt = PathBuf::from("../../assets/test_video.mp4");
        if alt.exists() {
            video_path = alt;
        } else {
            // Generate it
            let _ = std::fs::create_dir_all("assets");
            let status = Command::new("ffmpeg")
                .args(&[
                    "-y", "-f", "lavfi", "-i", "testsrc=duration=2:size=100x100:rate=30",
                    "-pix_fmt", "yuv420p",
                    "assets/test_video.mp4"
                ])
                .output();

            if status.is_err() || !status.unwrap().status.success() {
                 // Try generating in current dir if relative path failed
                 let _ = std::fs::create_dir_all("assets");
                 let _ = Command::new("ffmpeg")
                    .args(&[
                        "-y", "-f", "lavfi", "-i", "testsrc=duration=2:size=100x100:rate=30",
                        "-pix_fmt", "yuv420p",
                        "assets/test_video.mp4"
                    ])
                    .output();
            }
        }
    }

    if !video_path.exists() {
         // Fallback to absolute path check or parent generation?
         // Assuming generation works if ffmpeg is present.
         // If not, we might be in a CI env without ffmpeg CLI but WITH ffmpeg libs?
         // In that case, we can't generate. But prompt implies environment has tools.
    }

    assert!(video_path.exists(), "Test video must exist at {:?}", video_path);
    // Ensure absolute path for video-rs if needed, though relative usually works
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
    let updates = 10;

    for i in 0..updates {
        director.update(i as f64 * 0.1);
    }

    let duration = start.elapsed();

    println!("Updates took: {:?}", duration);

    assert!(duration.as_millis() >= 5, "Director update should block in Export mode. Took {:?}, expected > 5ms", duration);
}
