// Masking & Blend Modes Reference
//
// Demonstrates:
// - Alpha masking (set_mask)
// - Blend modes (set_blend_mode)

let movie = new_director(1920, 1080, 30);
let scene = movie.add_scene(3.0);

let root = scene.add_box(#{
    width: "100%",
    height: "100%",
    flex_direction: "row",
    justify_content: "space_evenly",
    align_items: "center",
    bg_color: "#1a1a2e"
});

// === MASKING DEMO ===
let mask_container = root.add_box(#{
    width: 300.0,
    height: 300.0,
    justify_content: "center",
    align_items: "center"
});

// Content to mask (gradient box)
let content = mask_container.add_box(#{
    width: 250.0,
    height: 250.0,
    bg_color: "#ff6b6b"
});

// Circular mask
let mask = mask_container.add_box(#{
    width: 200.0,
    height: 200.0,
    bg_color: "#ffffff",
    border_radius: 100.0  // Makes it a circle
});

// Apply mask - content is clipped to mask shape
content.set_mask(mask);

// === BLEND MODE DEMO ===
let blend_container = root.add_box(#{
    width: 300.0,
    height: 300.0,
    justify_content: "center",
    align_items: "center"
});

// Background
let bg = blend_container.add_box(#{
    width: 250.0,
    height: 250.0,
    bg_color: "#4ecdc4",
    border_radius: 20.0,
    position: "absolute"
});

// Overlapping box with blend mode
let overlay = blend_container.add_box(#{
    width: 200.0,
    height: 200.0,
    bg_color: "#ff6b6b",
    border_radius: 20.0,
    position: "absolute"
});

overlay.set_blend_mode("multiply");

movie
