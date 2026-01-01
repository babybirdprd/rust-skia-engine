// Ultimate Mashup Showcase - Premium Edition
// A masterclass in Director Engine capabilities.
// Features: Dynamic Z-Index, Glassmorphism, Audio Reactivity, Kinetic Typography, 
// Advanced Physics Easings, Shaders/Effects, Complex Grid Layouts.

let movie = new_director(1920, 1080, 60);

// --- DESIGN SYSTEM ---
let colors = #{
    bg: "#050508",
    card: "rgba(20, 20, 30, 0.7)",
    border: "rgba(255, 255, 255, 0.1)",
    accent: "#8b5cf6", // Vibrant Violet
    text: "#ffffff",
    dim: "#64748b"
};

// Global Audio
let music = movie.add_audio("assets/music.mp3");
let track_id = music.id();

// --- SCENE 1: THE MONOLITH (4.0s) ---
let s1 = movie.add_scene(4.0);
s1.add_box(#{ width: "100%", height: "100%", bg_color: colors.bg });

// Background Cinematic Video
let bg_vid = s1.add_video("assets/background.mp4", #{
    width: "100%", height: "100%", object_fit: "cover", opacity: 0.3
});
bg_vid.apply_effect("blur", 15.0);

let s1_root = s1.add_box(#{
    width: "100%", height: "100%", justify_content: "center", align_items: "center",
    flex_direction: "column"
});

let title = s1_root.add_text(#{
    content: "DIRECTOR",
    size: 220.0,
    color: colors.text,
    weight: "bold",
    text_shadow_color: "rgba(139, 92, 246, 0.5)",
    text_shadow_blur: 40.0
});

// Staggered Kinetic Entrance
title.add_animator(0, 8, "scale", 0.0, 1.0, 2.0, "elastic_out");
title.add_animator(0, 8, "opacity", 0.0, 1.0, 1.0, "ease_out");

let subtitle = s1_root.add_text(#{
    content: "UNLEASH THE POWER OF RUST & SKIA",
    size: 40.0,
    color: colors.dim,
    margin_top: 20
});
subtitle.animate("opacity", 0.0, 1.0, 2.0, "linear");
subtitle.animate("y", 50, 0, 2.0, "back_out");

// --- SCENE 2: THE BENTO GRID (10.0s) ---
let s2 = movie.add_scene(10.0);
s2.add_box(#{ width: "100%", height: "100%", bg_color: colors.bg });

// Grid Container
let grid = s2.add_box(#{
    width: "100%", height: "100%",
    display: "grid",
    grid_template_columns: ["1fr", "1.4fr", "1fr"],
    grid_template_rows: ["1fr", "1fr"],
    gap: 50.0, padding: 80.0
});

// Helper for Card Creation
fn create_card(parent, col, row, span_row) {
    let card = parent.add_box(#{
        grid_column: col, 
        grid_row: if span_row > 1 { row + " / span " + span_row } else { row },
        bg_color: "rgba(255, 255, 255, 0.03)",
        border_radius: 50.0,
        border_width: 1.5,
        border_color: "rgba(255, 255, 255, 0.08)",
        shadow_color: "rgba(0, 0, 0, 0.4)",
        shadow_blur: 30.0,
        shadow_y: 15.0,
        overflow: "hidden",
        position: "relative"
    });
    // Glass Effect
    card.apply_effect("blur", 8.0);
    return card;
}

// 1. DYNAMIC FILTERS (Left)
let c1 = create_card(grid, "1", "1", 1);
let filter_img = c1.add_image("assets/img1.jpg", #{ width: "100%", height: "100%", object_fit: "cover" });
filter_img.animate("opacity", 0.0, 1.0, 1.0, "linear");
// Loop sequence of filters
filter_img.apply_effect("grayscale");
c1.add_text(#{ content: "GRAYSCALE", size: 30, color: "#fff", position: "absolute", bottom: 30, left: 30, weight: "bold" });

// 2. AUDIO REACTIVE CORE (Center)
let c2 = create_card(grid, "2", "1", 2);
let react_box = c2.add_box(#{ width: "100%", height: "100%", justify_content: "center", align_items: "center"});

// Neon Glow Orb
let glow = react_box.add_box(#{
    width: 500, height: 500, border_radius: 250,
    bg_color: "rgba(139, 92, 246, 0.2)",
    shadow_color: "rgba(139, 92, 246, 0.6)",
    shadow_blur: 100.0
});
glow.bind_audio(track_id, "bass", "scale");

let inner_orb = react_box.add_box(#{
    width: 300, height: 300, border_radius: 150,
    bg_color: "#8b5cf6",
    shadow_color: "#8b5cf6",
    shadow_blur: 50.0,
    position: "absolute"
});
inner_orb.bind_audio(track_id, "bass", "scale");

react_box.add_text(#{ content: "AUDIO REACTIVE", size: 40, weight: "bold", margin_top: 400 });

// 3. OBJECT-FIT VIDEO (Right)
let c3 = create_card(grid, "3", "1", 1);
c3.add_video("assets/background.mp4", #{ width: "100%", height: "100%", object_fit: "cover" });
c3.add_text(#{ content: "VIDEO CLIPPING", size: 30, position: "absolute", top: 30, width: "100%", justify_content: "center" });

// 4. KINETIC STATS (Bottom Left)
let c4 = create_card(grid, "1", "2", 1);
let stat_text = c4.add_text(#{ content: "60 FPS", size: 100, color: "#10b981", weight: "bold" });
stat_text.add_animator(0, 6, "scale", 0.5, 1.0, 1.5, "bounce_out");

// 5. Z-INDEX COMPOSITION (Bottom Right)
let c5 = create_card(grid, "3", "2", 1);
c5.add_box(#{ width: 150, height: 150, bg_color: "#ef4444", position: "absolute", top: 80, left: 80, z_index: 20, border_radius: 20 });
c5.add_box(#{ width: 150, height: 150, bg_color: "#3b82f6", position: "absolute", top: 130, left: 130, z_index: 10, border_radius: 20 });
c5.add_text(#{ content: "Z-INDEX LAYER", size: 30, position: "absolute", bottom: 30, width: "100%", justify_content: "center" });

// --- TRANSITIONS ---
movie.add_transition(s1, s2, "circle_open", 2.0, "ease_in_out");

movie
