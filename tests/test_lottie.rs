use director_engine::Director;
use director_engine::video_wrapper::RenderMode;
use director_engine::scripting::MovieHandle;
use std::sync::{Arc, Mutex};
use director_engine::AssetLoader;
use anyhow::Result;

struct MockAssetLoader;
impl AssetLoader for MockAssetLoader {
    fn load_bytes(&self, _path: &str) -> Result<Vec<u8>> {
        // Return dummy bytes
        Ok(vec![0; 10])
    }
}

#[test]
fn test_lottie_loading_and_render() {
    let loader = Arc::new(MockAssetLoader);
    let mut engine = rhai::Engine::new();
    director_engine::scripting::register_rhai_api(&mut engine, loader.clone());

    let script = r#"
        let movie = new_director(1920, 1080, 30);
        let scene = movie.add_scene(5.0);

        let lottie = scene.add_lottie("test.json", #{
            "loop": true,
            speed: 2.0,
            width: "50%",
            height: "50%"
        });

        lottie.set_lottie_loop("ping_pong");
        lottie.animate("seek", 0.0, 1.0, 5.0, "linear");
    "#;

    engine.run(script).expect("Script execution failed");
}
