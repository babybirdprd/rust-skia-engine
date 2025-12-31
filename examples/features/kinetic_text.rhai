// Kinetic Typography Demo - Full Showcase
//
// Demonstrates ALL per-letter animation properties:
// 1. Opacity (fade in)
// 2. Offset Y (drop in from above)
// 3. Offset X (slide in from left)
// 4. Scale (grow from small)
// 5. Rotation (spin in)

let movie = new_director(1920, 1080, 30);
let scene = movie.add_scene(6.0);

let root = scene.add_box(#{
    width: "100%",
    height: "100%",
    flex_direction: "column",
    justify_content: "space_evenly",
    align_items: "center",
    padding: 80.0,
    bg_color: "#0f0f23"
});

// Row 1: OPACITY - Fade in effect
let text1 = root.add_text(#{
    content: "FADE IN",
    size: 72.0,
    color: "#ff6b6b"
});
// Animate opacity from 0 to 1
text1.add_animator(0, 7, "opacity", 0.0, 1.0, 2.0, "ease_out");

// Row 2: OFFSET Y - Drop from above
let text2 = root.add_text(#{
    content: "DROP IN",
    size: 72.0,
    color: "#4ecdc4"
});
// Animate Y offset from -50 to 0 (starts above, drops down)
text2.add_animator(0, 7, "offset_y", -80.0, 0.0, 2.0, "ease_out");

// Row 3: SCALE - Grow effect
let text3 = root.add_text(#{
    content: "SCALE UP",
    size: 72.0,
    color: "#ffe66d"
});
// Animate scale from 0.2 to 1.0
text3.add_animator(0, 8, "scale", 0.2, 1.0, 2.0, "ease_out");

// Row 4: ROTATION - Spin effect
let text4 = root.add_text(#{
    content: "SPIN",
    size: 72.0,
    color: "#a29bfe"
});
// Animate rotation from -180 degrees to 0
text4.add_animator(0, 4, "rotation", -180.0, 0.0, 2.0, "ease_out");

// Row 5: OFFSET X - Slide from side
let text5 = root.add_text(#{
    content: "SLIDE IN",
    size: 72.0,
    color: "#fd79a8"
});
// Animate X offset from -100 to 0 (slide in from left)
text5.add_animator(0, 8, "offset_x", -150.0, 0.0, 2.0, "ease_out");

movie
