let movie = new_director(1920, 1080, 60);
let scene = movie.add_scene(5.0);

let root = scene.add_box(#{
    "width": "100%",
    "height": "100%",
    "flex_direction": "row",
    "justify_content": "space_around",
    "align_items": "center"
});

// 1. Working Red Square
let box1 = root.add_box(#{
    "width": 400,
    "height": 400,
    "bg_color": "#222222"
});
let lottie1 = box1.add_lottie("tests/fixtures/lottie_simple.json", #{
    "width": "100%",
    "height": "100%",
    "loop": true
});
lottie1.animate("rotation", 0.0, 360.0, 5.0, "linear");

// 2. User File 1 (u4j)
let box2 = root.add_box(#{
    "width": 400,
    "height": 400,
    "bg_color": "#333333"
});
// Using original downloaded file, not temp
box2.add_lottie("tests/fixtures/lottie_u4j.json", #{
    "width": "100%",
    "height": "100%",
    "loop": true
});

// 3. User File 2 (mobilo)
let box3 = root.add_box(#{
    "width": 400,
    "height": 400,
    "bg_color": "#444444"
});
box3.add_lottie("tests/fixtures/lottie_mobilo_a.json", #{
    "width": "100%",
    "height": "100%",
    "loop": true
});

print("Lottie script evaluated.");
movie
