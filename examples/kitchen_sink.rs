use director_engine::scripting::{register_rhai_api, MovieHandle};
use director_engine::{Director, DefaultAssetLoader};
use director_engine::render::render_export;
use rhai::Engine;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

fn main() {
    println!("Initializing Kitchen Sink Showcase...");

    let mut engine = Engine::new();
    register_rhai_api(&mut engine, Arc::new(DefaultAssetLoader));

    let script = r##"
        // 1. Setup Director (1920x1080 @ 30fps)
        let movie = new_director(1920, 1080, 30, #{ mode: "export" });

        // Configure Motion Blur (Cinematic feel)
        movie.configure_motion_blur(4, 180.0);

        // Global Audio
        let bgm = movie.add_audio("assets/music.mp3");
        bgm.animate_volume(0.0, 0.5, 2.0, "linear");

        // ==========================================
        // SCENE 1: Typography & Intro
        // ==========================================
        let scene1 = movie.add_scene(4.0);

        let bg1 = scene1.add_box(#{
            width: "100%", height: "100%",
            bg_color: "#1a1a1a",
            flex_direction: "column",
            justify_content: "center",
            align_items: "center"
        });

        let title = bg1.add_text(#{
            content: [
                #{ text: "Kitchen ", size: 120.0, color: "#FFFFFF", weight: "bold" },
                #{
                    text: "Sink",
                    size: 140.0,
                    weight: "black",
                    fill_gradient: ["#FF0055", "#FFCC00"]
                }
            ]
        });

        // Spring Animation on Scale
        // title.animate("scale", 0.0, 1.0, #{ stiffness: 200.0, damping: 15.0 }); // Simple spring
        // We'll use start/end version to be explicit
        title.animate("scale", 0.0, 1.0, #{ stiffness: 200.0, damping: 15.0 });
        title.animate("rotation", -10.0, 0.0, 1.0, "ease_out");

        let subtitle = bg1.add_text(#{
            content: "Powered by Director Engine",
            size: 40.0,
            color: "#AAAAAA",
            margin_top: 20.0
        });
        subtitle.animate("opacity", 0.0, 1.0, 2.0, "ease_in");

        // ==========================================
        // SCENE 2: Layout & Images
        // ==========================================
        let scene2 = movie.add_scene(5.0);

        // Transition: Slide Left
        movie.add_transition(scene1, scene2, "slide_left", 1.0, "ease_in_out");

        let grid = scene2.add_box(#{
            width: "100%", height: "100%",
            bg_color: "#FFFFFF",
            flex_direction: "row",
            flex_wrap: "wrap",
            padding: 50.0,
            justify_content: "space_evenly",
            align_items: "center",
            align_content: "center"
        });

        // Add 3 cards with images/effects
        let cards = [
            #{ color: "#FF5733", effect: "none", title: "Normal" },
            #{ color: "#33FF57", effect: "blur", title: "Blur" },
            #{ color: "#3357FF", effect: "grayscale", title: "B&W" }
        ];

        // Loop manual unrolling since Rhai loops over arrays are tricky if we need index for float math
        // We'll just do a standard range loop
        for i in 0..3 {
            let card_data = cards[i];
            let card = grid.add_box(#{
                width: 500.0, height: 700.0,
                bg_color: "#F0F0F0",
                border_radius: 20.0,
                flex_direction: "column",
                overflow: "hidden", // Clip image
                shadow_color: "#000000",
                shadow_blur: 20.0,
                shadow_y: 10.0,
                margin: 20.0
            });

            // Image container to control size
            let img_box = card.add_box(#{ width: "100%", height: "70%" });
            let img = img_box.add_image("assets/image.jpg");
            img.set_style(#{ width: "100%", height: "100%" }); // Fill container

            if card_data.effect == "blur" {
                img.apply_effect("blur", 10.0);
            } else if card_data.effect == "grayscale" {
                img.apply_effect("grayscale");
            }

            // Caption
            let caption = card.add_box(#{
                width: "100%", height: "30%",
                justify_content: "center", align_items: "center"
            });
            caption.add_text(#{
                content: card_data.title,
                size: 48.0, color: "#333", weight: "bold"
            });

            // Staggered Entrance
            // Delay by animating from start to start for duration
            let delay = i.to_float() * 0.3;
            if delay > 0.0 {
                card.animate("y", 1000.0, 1000.0, delay, "linear");
            }
            card.animate("y", 1000.0, 0.0, #{ stiffness: 80.0, damping: 12.0 });
        }

        // ==========================================
        // SCENE 3: Video & Compositing & Nested
        // ==========================================
        let scene3 = movie.add_scene(8.0);
        movie.add_transition(scene2, scene3, "circle_open", 1.0, "ease_out");

        // Background Video
        let vid_root = scene3.add_box(#{ width: "100%", height: "100%", position: "absolute" });
        let vid = vid_root.add_video("assets/video.mp4");
        vid.set_style(#{ width: "100%", height: "100%", position: "absolute" });

        // Apply a dark overlay for text readability
        let overlay = scene3.add_box(#{
            width: "100%", height: "100%",
            bg_color: "#000000",
            opacity: 0.7
        });

        // Nested Composition (A spinning logo/shape)
        let comp_movie = new_director(400, 400, 30);
        let c_scene = comp_movie.add_scene(10.0);

        // Draw a shape in composition
        let spinner = c_scene.add_box(#{
            width: 200.0, height: 200.0,
            bg_color: "#00E5FF",
            border_radius: 40.0,
            justify_content: "center", align_items: "center"
        });
        spinner.add_text(#{ content: "PIP", size: 60.0, color: "#000", weight: "bold" });
        spinner.animate("rotation", 0.0, 360.0, 3.0, "linear"); // spins once every 3s

        // Add composition to main scene
        let comp_node = scene3.add_composition(comp_movie, #{
            width: 400.0, height: 400.0,
            position: "absolute",
            right: 50.0, bottom: 50.0
        });

        // Masking Example
        // We want to show the video THROUGH text "MASK"
        // To do this, we need the video to be masked by the text.
        // But the video is background.
        // Let's make a new video instance for masking effect to be clear.

        let mask_container = scene3.add_box(#{
            width: "100%", height: "100%",
            justify_content: "center", align_items: "center"
        });

        // We use an image for the masked content because video decoding multiple streams might be heavy,
        // but let's try image to be safe and clear.
        let masked_content = mask_container.add_image("assets/image.jpg");
        masked_content.set_style(#{ width: 1200.0, height: 800.0 });
        masked_content.apply_effect("sepia"); // Just to look different

        let mask_text = mask_container.add_text(#{
            content: "MASKING",
            size: 250.0,
            weight: "black"
        });

        // Apply mask
        masked_content.set_mask(mask_text);

        // Animate the mask text
        mask_text.animate("scale", 0.8, 1.2, 8.0, "ease_in_out");

        // Fade out at end
        let fader = scene3.add_box(#{
            width: "100%", height: "100%", bg_color: "#000", opacity: 0.0, position: "absolute", z_index: 999
        });
        fader.animate("opacity", 0.0, 0.0, 6.0, "linear"); // delay
        fader.animate("opacity", 0.0, 1.0, 2.0, "linear"); // fade out

        movie
    "##;

    match engine.eval::<MovieHandle>(script) {
        Ok(movie) => {
            println!("Script evaluated. Rendering to kitchen_sink.mp4...");
            let mut director = movie.director.lock().unwrap();

            match render_export(&mut director, PathBuf::from("kitchen_sink.mp4"), None, None) {
                Ok(_) => println!("Success! Video saved to kitchen_sink.mp4"),
                Err(e) => eprintln!("Render failed: {}", e),
            }
        },
        Err(e) => eprintln!("Script Error: {}", e),
    }
}
