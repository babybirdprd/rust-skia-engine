// Test Z-Index Rendering Order
// Goal: Ensure Red Box (created first) renders ON TOP of Blue Box (created second)
// by using z_index.

let movie = new_director(200, 200, 30);
let scene = movie.add_scene(1.0);

// 1. Create Container
let root = scene.add_box(#{
    width: "100%",
    height: "100%",
    bg_color: "#000000",
    justify_content: "center",
    align_items: "center"
});

// 2. Create Red Box (First)
// By default document order, this would be behind Blue.
let red = root.add_box(#{
    width: 100.0,
    height: 100.0,
    bg_color: "#FF0000",
    position: "absolute",
    // We give it higher Z-Index
    z_index: 10
});

// 3. Create Blue Box (Second)
// By default, this would cover Red.
let blue = root.add_box(#{
    width: 100.0,
    height: 100.0,
    bg_color: "#0000FF",
    position: "absolute",
    // Lower Z-Index (default is 0)
    z_index: 0
});

// 4. Verification Visual
// If Z-Index works: You see a RED box.
// If Z-Index fails: You see a BLUE box.

movie