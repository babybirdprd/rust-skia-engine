// Effects Reference
//
// Demonstrates visual effects:
// - Gaussian blur
// - Color matrix filters (grayscale, sepia, invert)
// - Custom shaders (SkSL)

let movie = new_director(1920, 1080, 30);
let scene = movie.add_scene(3.0);

let root = scene.add_box(#{
    width: "100%",
    height: "100%",
    flex_direction: "row",
    justify_content: "space_evenly",
    align_items: "center",
    bg_color: "#0a0a0a"
});

// Original (no effect)
let box1 = root.add_box(#{
    width: 200.0,
    height: 200.0,
    bg_color: "#ff6b6b",
    border_radius: 20.0
});

// Blur effect
let box2 = root.add_box(#{
    width: 200.0,
    height: 200.0,
    bg_color: "#4ecdc4",
    border_radius: 20.0
});
box2.apply_effect("blur", 5.0);

// Grayscale
let box3 = root.add_box(#{
    width: 200.0,
    height: 200.0,
    bg_color: "#ffe66d",
    border_radius: 20.0
});
box3.apply_effect("grayscale");

// Sepia
let box4 = root.add_box(#{
    width: 200.0,
    height: 200.0,
    bg_color: "#a06cd5",
    border_radius: 20.0
});
box4.apply_effect("sepia");

// Invert
let box5 = root.add_box(#{
    width: 200.0,
    height: 200.0,
    bg_color: "#6bcb77",
    border_radius: 20.0
});
box5.apply_effect("invert");

movie
