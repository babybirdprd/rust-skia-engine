// Image Loading Reference
//
// Demonstrates:
// - Loading images from file
// - Image sizing and positioning
// - Image animations

let movie = new_director(1920, 1080, 30);
let scene = movie.add_scene(5.0);

// Container
let root = scene.add_box(#{
    width: "100%",
    height: "100%",
    flex_direction: "column",
    justify_content: "center",
    align_items: "center",
    bg_color: "#1a1a2e"
});

// Load image via parent node
let img = root.add_image("assets/img1.jpg", #{
    width: 400.0,
    height: 300.0,
    border_radius: 20.0
});

// Animate the image
img.set_pivot(0.5, 0.5);
img.animate("scale", 0.8, 1.0, 1.0, "ease_out");
img.animate("opacity", 0.0, 1.0, 0.5, "ease_in");

// Title
root.add_text(#{
    content: "Image Loading",
    size: 48.0,
    color: "#ffffff"
});

movie
