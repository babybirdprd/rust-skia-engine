// Ultimate Mashup Showcase - Final Polish
// Combining: Grid, Audio Reactivity, Kinetic Text, Effects, Transitions, Video, 
// Images, Masking, Advanced Easings (Elastic, Bounce, Back), Z-Index, and Object-Fit.

let movie = new_director(1920, 1080, 60);

// Shared Assets
let music = movie.add_audio("assets/music.mp3");
let track_id = music.id();

// --- SCENE 1: CINEMATIC INTRO (4s) ---
let s1 = movie.add_scene(4.0);

// Background Video with Frost Effect
let bg_vid1 = s1.add_video("assets/background.mp4", #{
    width: "100%", height: "100%", object_fit: "cover", opacity: 0.8
});
bg_vid1.apply_effect("blur", 8.0);

let s1_root = s1.add_box(#{
    width: "100%", height: "100%", justify_content: "center", align_items: "center"
});

let intro_text = s1_root.add_text(#{
    content: "ULTIMATE SHOWCASE",
    size: 140.0,
    color: "#ffffff",
    weight: "bold"
});
// Kinetic Intro: Slide from Top with Elastic Bounce
intro_text.add_animator(0, 16, "offset_y", -400.0, 0.0, 2.0, "elastic_out");
intro_text.add_animator(16, 32, "scale", 0.0, 1.0, 1.5, "back_out");

// --- SCENE 2: THE BENTO GRID (8s) ---
let s2 = movie.add_scene(8.0);

// Clean BG for Scene 2
s2.add_box(#{ width: "100%", height: "100%", bg_color: "#05070a" });

let grid = s2.add_box(#{
    width: "100%", height: "100%",
    display: "grid",
    grid_template_columns: ["1fr", "1.5fr", "1fr"],
    grid_template_rows: ["1.2fr", "1fr"],
    gap: 40.0, padding: 60.0
});

// 1. TOP LEFT: Z-Index Overlay Demo (Glassmorphism)
let z_demo = grid.add_box(#{
    grid_column: "1", grid_row: "1",
    bg_color: "rgba(255, 255, 255, 0.05)",
    border_radius: 30.0, position: "relative",
    border_width: 1.0, border_color: "rgba(255, 255, 255, 0.1)"
});
z_demo.apply_effect("blur", 10.0);

// Blue box behind
z_demo.add_box(#{
    width: 200, height: 200, bg_color: "#3b82f6", 
    position: "absolute", top: 100, left: 100, z_index: 0,
    border_radius: 20.0
}).animate("rotation", 0.0, 360.0, 4.0, "linear");

// Red box in front (higher z_index)
z_demo.add_box(#{
    width: 200, height: 200, bg_color: "#ef4444", 
    position: "absolute", top: 150, left: 150, z_index: 10,
    border_radius: 20.0
}).animate("rotation", 360.0, 0.0, 4.0, "linear");

z_demo.add_text(#{
    content: "Z-INDEX", size: 40.0, color: "#ffffff", weight: "bold",
    position: "absolute", bottom: 40, width: "100%", justify_content: "center"
});

// 2. CENTER: Reactive Core (The "Eye")
let core = grid.add_box(#{
    grid_column: "2", grid_row: "1 / span 2",
    bg_color: "rgba(0,0,0,0.5)", border_radius: 40.0,
    justify_content: "center", align_items: "center", position: "relative"
});
core.animate("scale", 0.0, 1.0, 1.5, "back_out");

// Pulsing Orb
let orb = core.add_box(#{
    width: 400, height: 400, border_radius: 200,
    bg_color: "rgba(255, 51, 102, 0.3)"
});
orb.set_pivot(0.5, 0.5);
orb.bind_audio(track_id, "bass", "scale");

// Masked Image inside orb
let mask = core.add_box(#{ width: 300, height: 300, border_radius: 150, bg_color: "#fff" });
mask.set_pivot(0.5, 0.5);
mask.bind_audio(track_id, "bass", "scale");

let img_masked = core.add_image("assets/img5.jpg", #{ width: 400, height: 400, object_fit: "cover" });
img_masked.set_mask(mask);

// 3. TOP RIGHT: Filter Stack
let filters = grid.add_box(#{
    grid_column: "3", grid_row: "1",
    bg_color: "rgba(255, 255, 255, 0.05)", border_radius: 30.0,
    padding: 20.0, flex_direction: "column", gap: 15.0
});
fn add_filter_img(parent, path, effect) {
    let img = parent.add_image(path, #{ width: "100%", height: "30%", border_radius: 10.0 });
    img.apply_effect(effect);
}
add_filter_img(filters, "assets/img2.jpg", "grayscale");
add_filter_img(filters, "assets/img3.jpg", "sepia");
add_filter_img(filters, "assets/img4.jpg", "invert");

// 4. BOTTOM LEFT: Kinetic Stats
let stats = grid.add_box(#{
    grid_column: "1", grid_row: "2",
    bg_color: "rgba(16, 185, 129, 0.1)", border_radius: 30.0,
    justify_content: "center", align_items: "center"
});
let stat_text = stats.add_text(#{ content: "60 FPS", size: 80.0, color: "#10b981", weight: "bold" });
stat_text.add_animator(0, 5, "offset_x", -200.0, 0.0, 1.0, "bounce_out");

// 5. BOTTOM RIGHT: Video Window
let vid_box = grid.add_box(#{
    grid_column: "3", grid_row: "2",
    bg_color: "#000", border_radius: 30.0,
    clip_path: "inset(0)", // Assuming clip exists or just border_radius works
    justify_content: "center", align_items: "center"
});
vid_box.add_video("assets/background.mp4", #{ width: "100%", height: "100%", object_fit: "cover" });

// --- TRANSITIONS ---
movie.add_transition(s1, s2, "circle_out", 1.5, "ease_in_out");

movie
