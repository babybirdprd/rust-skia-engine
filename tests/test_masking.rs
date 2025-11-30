#[cfg(test)]
mod tests {
    use director_engine::scripting::register_rhai_api;
    use rhai::Engine;
    use director_engine::DefaultAssetLoader;
    use std::sync::Arc;

    // We don't necessarily need to run render_export which requires ffmpeg/video-rs
    // We can just verify the scene graph structure if we could access it,
    // or just ensure `update` runs.

    #[test]
    fn test_masking_script() {
        let mut engine = Engine::new();
        register_rhai_api(&mut engine, Arc::new(DefaultAssetLoader));

        let script = r##"
            let movie = new_director(500, 500, 30);
            let scene = movie.add_scene(1.0);

            // Container
            let container = scene.add_box(#{
                width: "100%",
                height: "100%",
                bg_color: "#000000"
            });

            // Content to be masked (Red Box)
            let red_box = container.add_box(#{
                width: 200.0,
                height: 200.0,
                bg_color: "#FF0000"
            });
            red_box.set_pivot(0.5, 0.5);
            red_box.animate("x", 100.0, 300.0, 1.0, "linear");

            // Mask (Box)
            let mask = container.add_box(#{
                width: 100.0,
                height: 100.0,
                bg_color: "#FFFFFF",
                border_radius: 50.0
            });

            // Set mask - this should move `mask` from `container` children to `red_box.mask_node`
            red_box.set_mask(mask);

            // Set blend mode
            red_box.set_blend_mode("screen");

            movie
        "##;

        let result = engine.eval::<director_engine::scripting::MovieHandle>(script);
        assert!(result.is_ok(), "Script failed: {}", result.err().unwrap());

        if let Ok(movie) = result {
             let mut director = movie.director.lock().unwrap();

             // Run update to verify traversal
             director.update(0.5);

             // Verify mask node structure manually?
             // Accessing node arena requires knowing IDs.
             // We can't easily query by "name".
             // But we know IDs are sequential.
             // 0: root, 1: container, 2: red_box, 3: mask.

             let red_box_node = director.get_node(2).expect("Red box should exist");
             assert_eq!(red_box_node.mask_node, Some(3), "Mask node should be set to ID 3");

             let mask_node = director.get_node(3).expect("Mask node should exist");
             assert_eq!(mask_node.parent, Some(2), "Mask parent should be red_box (2)");

             let container = director.get_node(1).expect("Container should exist");
             assert!(!container.children.contains(&3), "Container should NOT contain mask (3) in children");
             assert!(container.children.contains(&2), "Container SHOULD contain red_box (2) in children");

             println!("Structure verification passed!");
        }
    }
}
