// Vector Support Demo (SVG)

let movie = new_director(1920, 1080, 30);
let scene = movie.add_scene(5.0);

// Add Background
let bg = scene.add_box(#{
    width: "100%",
    height: "100%",
    bg_color: "#1a1a1a",
});

// Add SVG
let svg = scene.add_svg("tests/assets/simple.svg", #{
    width: 600.0,
    height: 600.0,
    align_self: "center",
    justify_self: "center",
});

// Animate it
svg.animate("scale", 0.0, 1.0, 1.0, "bounce_out");
svg.animate("rotation", 0.0, 360.0, 5.0, "linear");

// Add Title
let title = scene.add_text(#{
    content: "Vector Graphics (SVG)",
    size: 80.0,
    color: "#ffffff",
    width: "100%",
    justify_content: "center",
    position: "absolute",
    bottom: 100.0,
});

title.animate("opacity", 0.0, 1.0, 1.0, "ease_in");

movie
