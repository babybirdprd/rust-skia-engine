let movie = new_director(200, 200, 30);
let scene = movie.add_scene(1.0);

// Container with Center alignment
let box = scene.add_box(#{
    width: 200,
    height: 200,
    bg_color: "#333333"
});

// The "H" Test
// Capital H is the standard for checking vertical centering
box.add_text(#{
    content: "H",
    size: 100,
    color: "#FFFFFF",
    bg_color: "#FF0000" // Red background on text span to see the bounding box
});

movie