// ============================================================
// GENESIS: Director Engine Cinematic Showcase
// A production-quality demonstration of the engine's capabilities
// ============================================================

print("🎬 GENESIS - Cinematic Showcase Initializing...");

// === CONFIGURATION ===
let width = 854;
let height = 480;
let fps = 30;

let movie = new_director(width, height, fps, #{ mode: "export" });

// Enable cinematic motion blur - OPTIMIZED (4 samples instead of 8 for faster render)
movie.configure_motion_blur(4, 180.0);

// === AUDIO ===
let bgm = movie.add_audio("assets/music.mp3");
bgm.animate_volume(0.0, 0.8, 1.0, "ease_in");

// ============================================================
// SCENE 1: PARTICLE FIELD (0-4s)
// A mesmerizing field of floating particles with depth
// ============================================================
let scene1 = movie.add_scene(4.0);

// Background image layer container
let bg1_container = scene1.add_box(#{
    width: "100%",
    height: "100%",
    position: "absolute"
});
let bg1 = bg1_container.add_image("assets/3.jpg", #{
    width: "100%",
    height: "100%"
});
bg1.apply_effect("blur", 8.0);
bg1.set_style(#{ opacity: 0.4 });

// Dark overlay for contrast
let overlay1 = scene1.add_box(#{
    width: "100%",
    height: "100%",
    bg_color: "#0a0a1a",
    opacity: 0.7
});

// Particle container
let particles = scene1.add_box(#{
    width: "100%",
    height: "100%",
    overflow: "hidden"
});

// Generate 15 particles (reduced from 40 for performance)
for i in 0..15 {
    let x_pos = rand_float(-100.0, 1920.0);
    let y_pos = rand_float(-100.0, 1080.0);
    let size = rand_float(4.0, 40.0);
    let delay = rand_float(0.0, 1.5);
    let duration = rand_float(2.0, 4.0);
    
    // Alternate between cyan and magenta particles
    let color = if i % 3 == 0 { "#00FFFF" } else if i % 3 == 1 { "#FF00FF" } else { "#FFFFFF" };
    
    let p = particles.add_box(#{
        width: size,
        height: size,
        bg_color: color,
        border_radius: size / 2.0, // Make it circular
        position: "absolute",
        left: x_pos,
        top: y_pos,
        shadow_color: color,
        shadow_blur: size * 2.0,
        opacity: 0.0
    });
    
    // Floating animation with spring physics
    let float_y = rand_float(-80.0, 80.0);
    p.animate("y", 0.0, 0.0, delay, "linear");
    p.animate("y", 0.0, float_y, duration, "ease_in_out");
    
    // Fade in/out
    p.animate("opacity", 0.0, 0.0, delay, "linear");
    p.animate("opacity", 0.0, 0.9, 0.5, "ease_out");
    p.animate("opacity", 0.9, 0.9, duration - 1.5, "linear");
    p.animate("opacity", 0.9, 0.0, 0.5, "ease_in");
    
    // Subtle scale pulsing
    p.animate("scale", 0.8, 1.2, duration / 2.0, "ease_in_out");
    p.animate("scale", 1.2, 0.8, duration / 2.0, "ease_in_out");
}

// Title text appearing through particles
let title_container = scene1.add_box(#{
    width: "100%",
    height: "100%",
    align_items: "center",
    justify_content: "center",
    position: "absolute",
    left: 0.0,
    top: 0.0
});

let intro_text = title_container.add_text(#{
    content: "GENESIS",
    size: 180.0,
    color: "#FFFFFF",
    weight: "bold",
    text_shadow_color: "#00FFFF",
    text_shadow_blur: 40.0,
    text_shadow_x: 0.0,
    text_shadow_y: 0.0,
    flex_shrink: 0,
    fit: "none"
});

intro_text.animate("opacity", 0.0, 0.0, 1.5, "linear");
intro_text.animate("opacity", 0.0, 1.0, 0.8, "ease_out");
intro_text.animate("scale", 0.5, 1.0, #{ stiffness: 120.0, damping: 8.0 });

// ============================================================
// SCENE 2: TYPOGRAPHY WAVE (4-8s)
// Per-character animation creating a wave effect
// ============================================================
let scene2 = movie.add_scene(4.0);
movie.add_transition(scene1, scene2, "fade", 0.5, "ease_in_out");

// Background with gradient feel
let bg2 = scene2.add_box(#{
    width: "100%",
    height: "100%",
    bg_color: "#0f0f23"
});

// Subtle background image container
let bg2_img_container = scene2.add_box(#{
    width: "100%",
    height: "100%",
    position: "absolute",
    opacity: 0.15
});
let bg2_img = bg2_img_container.add_image("assets/3.jpg", #{
    width: "100%",
    height: "100%"
});

let text_container = scene2.add_box(#{
    width: "100%",
    height: "100%",
    flex_direction: "column",
    align_items: "center",
    justify_content: "center"
});

// Main title with wave animation
let wave_text = text_container.add_text(#{
    content: "DIRECTOR",
    size: 160.0,
    weight: "bold",
    color: "#FFFFFF",
    text_shadow_color: "#6366f1",
    text_shadow_blur: 20.0,
    flex_shrink: 0,
    fit: "none"
});

// Animate each character with staggered timing (wave effect)
// Characters 0-7 for "DIRECTOR"
for i in 0..8 {
    let delay = i * 0.08;
    wave_text.add_animator(i, i + 1, "y", 80.0, 0.0, 0.6, "bounce_out");
    wave_text.add_animator(i, i + 1, "opacity", 0.0, 1.0, 0.4, "ease_out");
    wave_text.add_animator(i, i + 1, "scale", 1.5, 1.0, 0.5, "ease_out");
}

// Subtitle with gradient
let subtitle = text_container.add_text(#{
    content: [
        #{ text: "ENGINE", size: 100.0, fill_gradient: ["#ec4899", "#8b5cf6", "#06b6d4"], weight: "bold" }
    ],
    margin: 20.0,
    flex_shrink: 0,
    fit: "none"
});

subtitle.animate("opacity", 0.0, 0.0, 0.8, "linear");
subtitle.animate("opacity", 0.0, 1.0, 0.5, "ease_out");
subtitle.animate("y", 50.0, 0.0, #{ stiffness: 100.0, damping: 12.0 });

// Tagline
let tagline = text_container.add_text(#{
    content: "Programmatic Video Generation",
    size: 36.0,
    color: "#a1a1aa",
    flex_shrink: 0,
    fit: "none"
});

tagline.animate("opacity", 0.0, 0.0, 1.5, "linear");
tagline.animate("opacity", 0.0, 1.0, 0.8, "ease_out");

// ============================================================
// SCENE 3: MEDIA FUSION (8-13s)
// Video through text mask + image showcase
// ============================================================
let scene3 = movie.add_scene(5.0);
movie.add_transition(scene2, scene3, "slide_left", 0.8, "ease_in_out");

let s3_root = scene3.add_box(#{
    width: "100%",
    height: "100%",
    bg_color: "#000000"
});

// Background video
let bg_video = s3_root.add_video("assets/background.mp4", #{
    width: "100%",
    height: "100%"
});

// Text mask - video shows through letters only
let mask_text = scene3.add_text(#{
    content: "SKIA",
    size: 400.0,
    weight: "bold",
    width: "100%",
    height: "100%",
    align_items: "center",
    justify_content: "center",
    flex_shrink: 0
});

bg_video.set_mask(mask_text);

// Image gallery cards floating around
let gallery = scene3.add_box(#{
    width: "100%",
    height: "100%",
    position: "absolute",
    left: 0.0,
    top: 0.0
});

let card_images = ["assets/1.jpg", "assets/2.jpg", "assets/4.jpg", "assets/5.jpg"];
let card_positions = [[100.0, 100.0], [1500.0, 150.0], [150.0, 700.0], [1400.0, 650.0]];

for i in 0..4 {
    let card = gallery.add_box(#{
        width: 280.0,
        height: 180.0,
        position: "absolute",
        left: card_positions[i][0],
        top: card_positions[i][1],
        border_radius: 16.0,
        overflow: "hidden",
        shadow_color: "#000000",
        shadow_blur: 30.0,
        shadow_y: 10.0,
        border_width: 2.0,
        border_color: "#ffffff33"
    });
    
    let img = card.add_image(card_images[i], #{
        width: "100%",
        height: "100%"
    });
    
    // Entrance animation
    let delay = 0.5 + i * 0.2;
    card.animate("opacity", 0.0, 0.0, delay, "linear");
    card.animate("opacity", 0.0, 1.0, 0.4, "ease_out");
    card.animate("scale", 0.7, 1.0, 0.6, "ease_out");
    
    // Floating motion
    let float_offset = rand_float(-20.0, 20.0);
    card.animate("y", 0.0, float_offset, 2.0, "ease_in_out");
    card.animate("y", float_offset, 0.0, 2.0, "ease_in_out");
    
    // Subtle rotation
    let rot = rand_float(-5.0, 5.0);
    card.animate("rotation", 0.0, rot, 2.5, "ease_in_out");
}

// ============================================================
// SCENE 4: SHADER MAGIC (13-17s)
// Custom shader effects demonstration
// ============================================================
let scene4 = movie.add_scene(4.0);
movie.add_transition(scene3, scene4, "circle_open", 1.0, "ease_in_out");

// Audio ducking for dramatic effect
bgm.animate_volume(0.8, 0.3, 0.5, "ease_out");

let s4_root = scene4.add_box(#{
    width: "100%",
    height: "100%",
    bg_color: "#0a0a0a",
    align_items: "center",
    justify_content: "center"
});

// Background image with effects - in a container
let shader_bg_container = s4_root.add_box(#{
    width: "100%",
    height: "100%",
    position: "absolute"
});
let shader_bg = shader_bg_container.add_image("assets/3.jpg", #{
    width: "100%",
    height: "100%"
});

// Apply chromatic aberration shader
let chroma_shader = `
    uniform shader image;
    uniform float intensity;
    uniform float time;
    
    half4 main(float2 coord) {
        float offset = intensity * 15.0;
        
        // RGB split based on distance from center
        float2 center = float2(427.0, 240.0);
        float2 dir = normalize(coord - center);
        
        half4 color;
        color.r = image.eval(coord + dir * offset).r;
        color.g = image.eval(coord).g;
        color.b = image.eval(coord - dir * offset).b;
        color.a = 1.0;
        
        return color;
    }
`;

let shader_effect = shader_bg.apply_effect("shader", #{
    code: chroma_shader,
    uniforms: #{
        intensity: 0.0,
        time: 0.0
    }
});

// Animate shader intensity for glitch bursts
shader_effect.animate("intensity", 0.0, 0.0, 0.5, "linear");
shader_effect.animate("intensity", 0.0, 1.0, 0.1, "ease_out");
shader_effect.animate("intensity", 1.0, 0.2, 0.3, "ease_out");
shader_effect.animate("intensity", 0.2, 0.8, 0.1, "linear");
shader_effect.animate("intensity", 0.8, 0.0, 0.5, "ease_out");

// Title over shader
let shader_title = s4_root.add_text(#{
    content: [
        #{ text: "REAL-TIME ", color: "#ffffff", size: 72.0, weight: "bold" },
        #{ text: "SHADERS", size: 72.0, fill_gradient: ["#f472b6", "#c084fc", "#60a5fa"], weight: "bold" }
    ],
    flex_shrink: 0,
    fit: "none"
});

shader_title.animate("scale", 0.8, 1.0, #{ stiffness: 150.0, damping: 10.0 });

// Glitch shake effect on title
shader_title.animate("skew_x", 0.0, 0.0, 1.0, "linear");
shader_title.animate("skew_x", 0.0, 8.0, 0.05, "linear");
shader_title.animate("skew_x", 8.0, -8.0, 0.05, "linear");
shader_title.animate("skew_x", -8.0, 5.0, 0.05, "linear");
shader_title.animate("skew_x", 5.0, 0.0, 0.1, "ease_out");

// ============================================================
// SCENE 5: THE FINALE (17-20s)
// Grand ending with logo and credits
// ============================================================
let scene5 = movie.add_scene(3.0);
movie.add_transition(scene4, scene5, "fade", 0.5, "ease_in_out");

// Bring music back up for finale
bgm.animate_volume(0.3, 1.0, 0.5, "ease_in");

let s5_root = scene5.add_box(#{
    width: "100%",
    height: "100%",
    bg_color: "#000000",
    flex_direction: "column",
    align_items: "center",
    justify_content: "center"
});

// Subtle background glow
let glow_box = s5_root.add_box(#{
    width: 600.0,
    height: 600.0,
    position: "absolute",
    bg_color: "#6366f1",
    border_radius: 300.0,
    opacity: 0.0
});

glow_box.set_blur(100.0);
glow_box.animate("opacity", 0.0, 0.3, 1.0, "ease_out");
glow_box.animate("scale", 0.5, 1.2, 2.0, "ease_out");

// Main logo
let logo = s5_root.add_text(#{
    content: "DIRECTOR ENGINE",
    size: 100.0,
    color: "#ffffff",
    weight: "bold",
    text_shadow_color: "#6366f1",
    text_shadow_blur: 30.0,
    flex_shrink: 0,
    fit: "none"
});

logo.animate("scale", 1.5, 1.0, #{ stiffness: 200.0, damping: 12.0 });
logo.animate("opacity", 0.0, 1.0, 0.5, "ease_out");

// Version badge
let version = s5_root.add_text(#{
    content: "v1.1",
    size: 24.0,
    color: "#6366f1",
    margin: 20.0
});

version.animate("opacity", 0.0, 0.0, 0.8, "linear");
version.animate("opacity", 0.0, 1.0, 0.3, "ease_out");

// Credits
let credits = s5_root.add_text(#{
    content: [
        #{ text: "Powered by ", color: "#71717a", size: 28.0 },
        #{ text: "Skia", color: "#22d3ee", size: 28.0, weight: "bold" },
        #{ text: " + ", color: "#71717a", size: 28.0 },
        #{ text: "Rust", color: "#f97316", size: 28.0, weight: "bold" }
    ],
    margin: 40.0,
    flex_shrink: 0,
    fit: "none"
});

credits.animate("opacity", 0.0, 0.0, 1.2, "linear");
credits.animate("opacity", 0.0, 1.0, 0.5, "ease_out");
credits.animate("y", 30.0, 0.0, 0.6, "ease_out");

// Fade out at the very end
bgm.animate_volume(1.0, 1.0, 1.5, "linear");
bgm.animate_volume(1.0, 0.0, 1.5, "ease_out");

print("✅ GENESIS Showcase Ready - 20 seconds of cinematic glory!");
movie
