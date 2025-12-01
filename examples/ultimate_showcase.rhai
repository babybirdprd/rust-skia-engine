// RFC 015: The "Ultimate Showcase" Validation
// Target: 720p @ 30fps (Backup Plan)

print("🎬 Initializing Ultimate Showcase...");

let width = 1280;
let height = 720;
let fps = 30;

// Configure for Export Mode
let movie = new_director(width, height, fps, #{ mode: "export" });

// Enable Motion Blur for production polish
movie.configure_motion_blur(8, 180.0);

// Load Assets
let font_inter = "assets/Inter-Bold.ttf";
let font_code = "assets/JetBrainsMono-Bold.ttf";
let bg_music = movie.add_audio("assets/music.mp3");

// Start Music Volume High
bg_music.animate_volume(0.0, 1.0, 0.5, "linear");

// ==========================================
// SCENE 1: THE HOOK (Kinetic Typography)
// ==========================================
let scene1 = movie.add_scene(4.0);
let s1_root = scene1.add_box(#{
    width: "100%", height: "100%",
    bg_color: "#000000",
    flex_direction: "column",
    justify_content: "center",
    align_items: "center"
});

// "PROGRAMMATIC" - Massive Text
let text_prog = s1_root.add_text(#{
    content: "PROGRAMMATIC",
    font_family: "Inter", // Fallback to system if loading fails, but we have file
    size: 150.0,
    color: "#FFFFFF",
    weight: "bold",
    text_shadow_color: "#00FFFF", // Neon Blue Glow
    text_shadow_blur: 20.0,
    text_shadow_x: 0.0, text_shadow_y: 0.0
});

// Animation: Spring Scale (Impact)
// High stiffness for "slam" effect
let spring_impact = #{ stiffness: 300.0, damping: 15.0, mass: 2.0 };
text_prog.animate("scale", 2.0, 1.0, spring_impact);
text_prog.animate("opacity", 0.0, 1.0, 0.2, "ease_out");

// Subtext Container
let sub_box = s1_root.add_box(#{
    margin: 40.0,
    flex_direction: "row",
    gap: 40.0 // Taffy gap support? API.md didn't explicitly list it but extract_outer_style had it.
              // If not, we use margins.
});

let words = ["VIDEO.", "MADE.", "FAST."];
let delays = [1.0, 1.2, 1.4];

for i in 0..3 {
    let w = sub_box.add_text(#{
        content: words[i],
        size: 80.0,
        color: "#3b82f6", // Blue-500
        weight: "bold",
        margin: 20.0
    });

    // Staggered Entry
    w.animate("y", 100.0, 0.0, #{ stiffness: 100.0, velocity: 50.0 });
    w.animate("opacity", 0.0, 1.0, 0.5, "ease_in");

    // Note: In Rhai `animate` start time isn't explicit relative to scene start unless we wait?
    // The current API `animate(prop, start, end, duration)` starts at t=0 of the node creation?
    // No, `animate` adds keys. To delay, we need `add_keyframe` logic or `wait`?
    // The API `add_text` adds it to the scene immediately.
    // The `animate` function adds segments.
    // To delay, we can add a dummy segment: `w.animate("opacity", 0.0, 0.0, delay, "linear")`.

    w.animate("opacity", 0.0, 0.0, delays[i], "linear");
    w.animate("opacity", 0.0, 1.0, 0.5, "ease_out");

    w.animate("translate_y", 50.0, 50.0, delays[i], "linear");
    w.animate("translate_y", 50.0, 0.0, 0.5, "bounce_out");
}

// ==========================================
// SCENE 2: THE GRID (Layout Stress Test)
// ==========================================
let scene2 = movie.add_scene(5.0);
movie.add_transition(scene1, scene2, "slide_left", 0.8, "ease_in_out");

let s2_root = scene2.add_box(#{
    width: "100%", height: "100%",
    bg_color: "#111111",
    padding: 50.0,
    flex_wrap: "wrap",
    flex_direction: "row",
    justify_content: "center",
    align_items: "center",
    overflow: "hidden"
});

// Camera Zoom Effect on Container
s2_root.set_pivot(0.5, 0.5);
s2_root.animate("scale", 1.5, 1.0, 4.0, "ease_out");

let images = [
    "assets/img1.jpg", "assets/img2.jpg", "assets/img3.jpg",
    "assets/img4.jpg", "assets/img5.jpg", "assets/img6.jpg"
];

for i in 0..6 {
    let is_hero = (i == 2); // 3rd image is hero

    let card = s2_root.add_box(#{
        width: 400.0, // Base width
        height: 300.0,
        margin: 20.0,
        flex_grow: 1.0, // Fill space
        border_radius: 20.0,
        overflow: "hidden",
        shadow_color: "#000000",
        shadow_blur: 30.0,
        shadow_y: 10.0
    });

    let img = card.add_image(images[i], #{
        width: "100%", height: "100%"
    });

    if is_hero {
        // Hero Animation
        card.animate("scale", 1.0, 1.1, 2.0, "ease_in_out");
        card.set_style(#{
            border_width: 4.0,
            border_color: "#FFFFFF"
        });
    } else {
        // Desaturate others
        img.apply_effect("grayscale");
        card.animate("opacity", 1.0, 0.5, 2.0, "ease_in_out");
    }
}

// ==========================================
// SCENE 3: THE COMPOSITION (Masking & Nested)
// ==========================================
let scene3 = movie.add_scene(6.0);
movie.add_transition(scene2, scene3, "wipe_right", 1.0, "ease_in_out");

let s3_root = scene3.add_box(#{
    width: "100%", height: "100%",
    bg_color: "#000000"
});

// Background Video (Jellyfish)
let bg_video = s3_root.add_video("assets/background.mp4", #{
    width: "100%", height: "100%",
    // object_fit: "cover" // If supported? Default usually stretches or fits.
    // We enforce filling via explicit 100%
});

// Stencil Mask Text
let mask_text = scene3.add_text(#{
    content: "DIRECTOR",
    size: 350.0,
    weight: "bold",
    // Position center
    width: "100%", height: "100%", // To help centering if container
    align_items: "center", justify_content: "center"
});

// Apply Mask: Video is ONLY visible inside Text
bg_video.set_mask(mask_text);
// Note: If we want inverted mask (video outside), current API is Alpha Matte (video inside).
// This is what the prompt asked: "we see the video only inside the letters". Correct.

// PIP Window (Nested Timeline)
// Define inner movie
let pip_movie = new_director(400, 400, 30);
let pip_scene = pip_movie.add_scene(6.0);
let pip_bg = pip_scene.add_box(#{
    width: "100%", height: "100%",
    bg_color: "#FFFFFF",
    align_items: "center", justify_content: "center"
});
let spinner = pip_bg.add_box(#{
    width: 200.0, height: 200.0,
    bg_color: "#FF0055",
    border_radius: 20.0
});
spinner.animate("rotation", 0.0, 360.0, 2.0, "linear");
spinner.animate("rotation", 360.0, 720.0, 2.0, "linear");
spinner.animate("rotation", 720.0, 1080.0, 2.0, "linear");

// Add PIP to main scene
let pip_node = scene3.add_composition(pip_movie, #{
    width: 400.0, height: 400.0,
    position: "absolute",
    right: 50.0, bottom: 50.0,
    shadow_color: "#000000",
    shadow_blur: 20.0
});

// ==========================================
// SCENE 4: THE DATA (Procedural Generation)
// ==========================================
let scene4 = movie.add_scene(5.0);
movie.add_transition(scene3, scene4, "circle_open", 1.0, "ease_in_out");

// Audio Ducking for this scene (simulate voiceover/data sounds)
bg_music.animate_volume(1.0, 0.2, 0.5, "ease_out");

let s4_root = scene4.add_box(#{
    width: "100%", height: "100%",
    bg_color: "#0f172a", // Slate-900
    flex_direction: "row",
    align_items: "flex_end", // Bars rise from bottom
    justify_content: "center",
    padding: 100.0,
    gap: 20.0
});

for i in 0..10 {
    // Random height between 20% and 90%
    let h_pct = rand_float(20.0, 90.0);

    let bar = s4_root.add_box(#{
        width: 80.0,
        height: 0.0, // Start at 0
        bg_color: "#38bdf8", // Sky-400
        border_radius: 10.0,
        shadow_color: "#38bdf8",
        shadow_blur: 10.0
    });

    // Animate height
    // We can't animate "height" (layout prop) via `animate` easily if it expects float but style is %.
    // `animate` supports specific layout props? API says: "Layout: width, height (if numeric)".
    // So we use pixels?
    // Let's use numeric height relative to a max of 800px.
    let h_px = h_pct * 8.0;

    let delay = i * 0.1;

    bar.animate("height", 0.0, 0.0, delay, "linear"); // Wait
    bar.animate("height", 0.0, h_px, 0.5, "bounce_out");
}

// ==========================================
// SCENE 5: GLITCH OUTRO
// ==========================================
let scene5 = movie.add_scene(3.0);
// Bring volume back for end
bg_music.animate_volume(0.2, 1.0, 0.5, "ease_in");

let s5_root = scene5.add_box(#{
    width: "100%", height: "100%",
    bg_color: "#000000",
    align_items: "center", justify_content: "center"
});

let logo = s5_root.add_text(#{
    content: "DIRECTOR ENGINE",
    size: 100.0,
    color: "#FFFFFF",
    weight: "bold",
    font_family: "Inter"
});

// Chromatic Aberration Shader
let shader_code = `
    uniform shader image;
    uniform float intensity; // 0.0 to 1.0

    half4 main(float2 coord) {
        half4 color = image.eval(coord);
        // Shift Red Left, Blue Right
        float shift = intensity * 20.0;
        color.r = image.eval(coord + float2(shift, 0)).r;
        color.b = image.eval(coord - float2(shift, 0)).b;
        return color;
    }
`;

let shader_effect = logo.apply_effect("shader", #{
    code: shader_code,
    uniforms: #{
        intensity: 0.0
    }
});

// Animate the Uniform "intensity"
shader_effect.animate("intensity", 0.0, 0.02, 0.1, "linear");
shader_effect.animate("intensity", 0.02, 0.0, 0.4, "bounce_out");

// Add some shake to the logo as well
logo.animate("skew_x", 0.0, 10.0, 0.05, "linear");
logo.animate("skew_x", 10.0, -10.0, 0.05, "linear");
logo.animate("skew_x", -10.0, 0.0, 0.05, "linear");

print("✅ Script Loaded. Returning Movie.");
movie
