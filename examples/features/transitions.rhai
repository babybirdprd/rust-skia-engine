// Scene Transitions Reference
//
// Demonstrates:
// - Multiple scenes
// - Transition types (fade, slide, wipe, circle)
// - Transition timing

let movie = new_director(1920, 1080, 30);

// Scene 1: Red
let s1 = movie.add_scene(3.0);
let root1 = s1.add_box(#{
    width: "100%",
    height: "100%",
    justify_content: "center",
    align_items: "center",
    bg_color: "#ff6b6b"
});
root1.add_text(#{ content: "Scene 1", size: 72.0, color: "#ffffff" });

// Scene 2: Teal
let s2 = movie.add_scene(3.0);
let root2 = s2.add_box(#{
    width: "100%",
    height: "100%",
    justify_content: "center",
    align_items: "center",
    bg_color: "#4ecdc4"
});
root2.add_text(#{ content: "Scene 2", size: 72.0, color: "#ffffff" });

// Scene 3: Yellow
let s3 = movie.add_scene(3.0);
let root3 = s3.add_box(#{
    width: "100%",
    height: "100%",
    justify_content: "center",
    align_items: "center",
    bg_color: "#ffe66d"
});
root3.add_text(#{ content: "Scene 3", size: 72.0, color: "#000000" });

// Add transitions
// movie.add_transition(from, to, type, duration, easing)
movie.add_transition(s1, s2, "fade", 1.0, "ease_in_out");
movie.add_transition(s2, s3, "slide_left", 0.5, "ease_out");

movie
