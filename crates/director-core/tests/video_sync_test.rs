#[cfg(feature = "video-rs")]
mod video_sync {
    use director_core::director::Director;
    use director_core::video_wrapper::RenderMode;
    use director_core::node::{BoxNode, VideoNode, VideoSource};
    use director_core::types::Color;
    use director_core::AssetLoader;
    use skia_safe::Color4f;
    use std::sync::Arc;
    use anyhow::Result;

    // Mock AssetLoader
    struct MockAssetLoader;
    impl AssetLoader for MockAssetLoader {
        fn load_bytes(&self, _path: &str) -> Result<Vec<u8>> {
             Err(anyhow::anyhow!("Mock loader"))
        }
        fn load_font_fallback(&self) -> Option<Vec<u8>> { None }
    }

    /// Helper to generate a 3-frame video (Red, Green, Blue) at 10fps.
    fn generate_source_video(path: &std::path::Path) {
        // Create Director with correct params
        let mut director = Director::new(
            100,
            100,
            10, // fps
            Arc::new(MockAssetLoader),
            RenderMode::Export,
            None // context
        );

        // Scene 0: Red (0.0 - 0.1)
        let red_root = director.scene.add_node(Box::new(BoxNode::new()));
        {
            let node = director.scene.get_node_mut(red_root).unwrap();
            if let Some(box_node) = node.element.as_any_mut().downcast_mut::<BoxNode>() {
                box_node.bg_color = Some(director_core::animation::Animated::new(Color::new(1.0, 0.0, 0.0, 1.0)));
                box_node.style.size = taffy::geometry::Size {
                    width: taffy::style::Dimension::percent(1.0),
                    height: taffy::style::Dimension::percent(1.0)
                };
            }
        }
        director.timeline.push(director_core::director::TimelineItem {
            scene_root: red_root,
            start_time: 0.0,
            duration: 0.1,
            z_index: 0,
            audio_tracks: vec![],
        });

        // Scene 1: Green (0.1 - 0.2)
        let green_root = director.scene.add_node(Box::new(BoxNode::new()));
        {
            let node = director.scene.get_node_mut(green_root).unwrap();
            if let Some(box_node) = node.element.as_any_mut().downcast_mut::<BoxNode>() {
                box_node.bg_color = Some(director_core::animation::Animated::new(Color::new(0.0, 1.0, 0.0, 1.0)));
                box_node.style.size = taffy::geometry::Size {
                    width: taffy::style::Dimension::percent(1.0),
                    height: taffy::style::Dimension::percent(1.0)
                };
            }
        }
        director.timeline.push(director_core::director::TimelineItem {
            scene_root: green_root,
            start_time: 0.1,
            duration: 0.1,
            z_index: 0,
            audio_tracks: vec![],
        });

        // Scene 2: Blue (0.2 - 0.3)
        let blue_root = director.scene.add_node(Box::new(BoxNode::new()));
        {
            let node = director.scene.get_node_mut(blue_root).unwrap();
            if let Some(box_node) = node.element.as_any_mut().downcast_mut::<BoxNode>() {
                box_node.bg_color = Some(director_core::animation::Animated::new(Color::new(0.0, 0.0, 1.0, 1.0)));
                box_node.style.size = taffy::geometry::Size {
                    width: taffy::style::Dimension::percent(1.0),
                    height: taffy::style::Dimension::percent(1.0)
                };
            }
        }
        director.timeline.push(director_core::director::TimelineItem {
            scene_root: blue_root,
            start_time: 0.2,
            duration: 0.1,
            z_index: 0,
            audio_tracks: vec![],
        });

        // Use correct function for render_export
        director_core::systems::renderer::render_export(
            &mut director,
            path.to_path_buf(),
            None,
            None
        ).expect("Export failed");
    }

    #[test]
    fn test_export_sync() {
        let temp_dir = tempfile::tempdir().unwrap();
        let source_path = temp_dir.path().join("sync_source.mp4");

        // 1. Generate Source
        generate_source_video(&source_path);
        assert!(source_path.exists());

        // 2. Consume Source
        let mut director = Director::new(
            100, 100, 10,
            Arc::new(MockAssetLoader),
            RenderMode::Export,
            None
        );

        // Add consuming scene
        let consume_root = director.scene.add_node(Box::new(BoxNode::new()));
        // Make root full size
        {
             let node = director.scene.get_node_mut(consume_root).unwrap();
             if let Some(box_node) = node.element.as_any_mut().downcast_mut::<BoxNode>() {
                 box_node.style.size = taffy::geometry::Size {
                     width: taffy::style::Dimension::percent(1.0),
                     height: taffy::style::Dimension::percent(1.0)
                 };
             }
        }

        let mut video_node = VideoNode::new(
            VideoSource::Path(source_path.clone()),
            RenderMode::Export // CRITICAL: Use Export mode for synchronous decoding
        );
        // FIX: Set size to 100% so Taffy allocates space for it
        video_node.style.size = taffy::geometry::Size {
            width: taffy::style::Dimension::percent(1.0),
            height: taffy::style::Dimension::percent(1.0),
        };

        let vid_id = director.scene.add_node(Box::new(video_node));
        // Add video as child of root
        director.scene.add_child(consume_root, vid_id);

        director.timeline.push(director_core::director::TimelineItem {
            scene_root: consume_root,
            start_time: 0.0,
            duration: 10.0, // Long enough
            z_index: 0,
            audio_tracks: vec![],
        });

        // 3. Verify

        // Helper to render current frame
        let render = |d: &mut Director, time: f64| {
            // d.update(time); // Removed to avoid double-update
            let mut s = skia_safe::surfaces::raster_n32_premul((100, 100)).unwrap();
            let canvas = s.canvas();
            canvas.clear(skia_safe::Color::TRANSPARENT);
            // We need to run layout
            director_core::systems::layout::LayoutEngine::new().compute_layout(&mut d.scene, 100, 100, time);
            d.run_post_layout(time);

            // Render
            director_core::systems::renderer::render_frame(d, time, canvas);

            s.image_snapshot()
        };

        let check_pixel = |image: skia_safe::Image, expected: Color4f, label: &str| {
            // Get pixel at 50,50
            let mut pixels = vec![0u8; 100 * 100 * 4];
            let info = skia_safe::ImageInfo::new_n32_premul((100, 100), None);
            let success = image.read_pixels(&info, &mut pixels, 400, (0, 0), skia_safe::image::CachingHint::Allow);
            assert!(success);

            let center_idx = (50 * 100 + 50) * 4;

            // Re-read with explicit RGBA8888
             let mut pixels_rgba = vec![0u8; 100 * 100 * 4];
             let info_rgba = skia_safe::ImageInfo::new(
                (100, 100),
                skia_safe::ColorType::RGBA8888,
                skia_safe::AlphaType::Premul,
                None
             );
             image.read_pixels(&info_rgba, &mut pixels_rgba, 400, (0, 0), skia_safe::image::CachingHint::Allow);

             let r = pixels_rgba[center_idx];
             let g = pixels_rgba[center_idx + 1];
             let b = pixels_rgba[center_idx + 2];
             let a = pixels_rgba[center_idx + 3];

            // Expected
            let er = (expected.r * 255.0) as u8;
            let eg = (expected.g * 255.0) as u8;
            let eb = (expected.b * 255.0) as u8;

            println!("[{}] Got RGBA: {},{},{},{} Expected: {},{},{}", label, r, g, b, a, er, eg, eb);

            let tolerance = 50; // High tolerance for video compression

            let diff = |a: u8, b: u8| (a as i32 - b as i32).abs();

            assert!(diff(r, er) < tolerance, "{} Red mismatch", label);
            assert!(diff(g, eg) < tolerance, "{} Green mismatch", label);
            assert!(diff(b, eb) < tolerance, "{} Blue mismatch", label);
        };

        // Check Frame 0 (0.0s) -> Red
        println!("Checking Frame 0 (Red)...");
        let img0 = render(&mut director, 0.0);
        check_pixel(img0, Color4f::new(1.0, 0.0, 0.0, 1.0), "Frame 0");

        // Check Frame 1 (0.1s) -> Green
        // Request exact frame start
        println!("Checking Frame 1 (Green)...");
        let img1 = render(&mut director, 0.1);
        check_pixel(img1, Color4f::new(0.0, 1.0, 0.0, 1.0), "Frame 1");
    }
}
