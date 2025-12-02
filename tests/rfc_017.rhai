let movie = new_director(1920, 1080, 60);
let scene = movie.add_scene(2.0);

// 1. Loading Spinner
let spinner = scene.add_shape(#{
    type: "circle",
    radius: 100.0,
    stroke_color: "#FFFFFF",
    stroke_width: 20.0,
    stroke_cap: "round",
    stroke_end: 0.0,
});

// Center the spinner using margin auto
spinner.set_style(#{
    margin: "auto",
});

// Animate fill up
spinner.animate("stroke_end", 0.0, 0.8, 1.0, "ease_in_out");
// Animate rotation (standard transform)
spinner.animate("rotation", 0.0, 360.0, 2.0, "linear");
// Animate offset (crawling)
spinner.animate("stroke_offset", 0.0, 1.0, 2.0, "linear");


// 2. Rounded Rectangle (Static visual)
let rect = scene.add_shape(#{
    type: "rect",
    width: 400.0,
    height: 300.0,
    corner_radius: 50.0,
    stroke_color: "#FF5555",
    stroke_width: 10.0,
    fill_color: "#330000",
});
rect.set_style(#{
    position: "absolute",
    left: 50,
    top: 50,
});

// 3. Checkmark (Path Animation)
// Path: M 0 30 L 30 60 L 90 0 (Checkmark shape)
let check = scene.add_shape(#{
    type: "path",
    d: "M 0 30 L 30 60 L 90 0",
    stroke_color: "#55FF55",
    stroke_width: 15.0,
    stroke_cap: "round",
    stroke_join: "round",
    stroke_end: 0.0,
});

check.set_style(#{
    position: "absolute",
    right: 100,
    bottom: 100,
});

// Animate checkmark drawing itself
check.animate("stroke_end", 0.0, 1.0, 0.5, "ease_out");
movie
