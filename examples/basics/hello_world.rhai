// Hello World - Minimal Director Script
//
// This is the simplest possible script that produces output.
// It demonstrates:
// - Creating a director (movie)
// - Adding a scene
// - Adding text

let movie = new_director(1920, 1080, 30);
let scene = movie.add_scene(3.0);

// Add centered text
let root = scene.add_box(#{
    width: "100%",
    height: "100%",
    justify_content: "center",
    align_items: "center",
    bg_color: "#1a1a2e"
});

root.add_text(#{
    content: "Hello, Director!",
    size: 72.0,
    color: "#ffffff",
    weight: "bold"
});

movie
