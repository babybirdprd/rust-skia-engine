// Animation Reference
//
// Demonstrates animation system:
// - Keyframe animations with easing
// - Spring physics animations
// - Transform properties (scale, rotation, translate)
// - Opacity animation

let movie = new_director(1920, 1080, 30);
let scene = movie.add_scene(5.0);

let root = scene.add_box(#{
    width: "100%",
    height: "100%",
    flex_direction: "row",
    justify_content: "space_evenly",
    align_items: "center",
    bg_color: "#0a0a0a"
});

// Box 1: Rotation animation
let box1 = root.add_box(#{
    width: 150.0,
    height: 150.0,
    bg_color: "#ff6b6b",
    border_radius: 20.0
});
box1.set_pivot(0.5, 0.5);
box1.animate("rotation", 0.0, 360.0, 2.0, "linear");

// Box 2: Scale with easing
let box2 = root.add_box(#{
    width: 150.0,
    height: 150.0,
    bg_color: "#4ecdc4",
    border_radius: 20.0
});
box2.set_pivot(0.5, 0.5);
box2.animate("scale", 0.5, 1.2, 1.0, "ease_in_out");

// Box 3: Spring physics
let box3 = root.add_box(#{
    width: 150.0,
    height: 150.0,
    bg_color: "#ffe66d",
    border_radius: 20.0
});
box3.set_pivot(0.5, 0.5);
box3.animate("scale", 1.5, #{
    stiffness: 200.0,
    damping: 12.0
});

// Box 4: Translation
let box4 = root.add_box(#{
    width: 150.0,
    height: 150.0,
    bg_color: "#a06cd5",
    border_radius: 20.0
});
box4.animate("y", -50.0, 50.0, 1.0, "bounce_out");

// Box 5: Opacity fade
let box5 = root.add_box(#{
    width: 150.0,
    height: 150.0,
    bg_color: "#6bcb77",
    border_radius: 20.0
});
box5.animate("opacity", 0.0, 1.0, 2.0, "ease_in");

movie
