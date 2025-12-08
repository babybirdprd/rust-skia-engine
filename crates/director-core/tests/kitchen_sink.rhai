// Initialize with Export mode for high-quality rendering
let movie = new_director(1920, 1080, 30, #{ mode: "export" });
let scene = movie.add_scene(10.0);

let root = scene.add_box(#{
    width: "100%",
    height: "100%",
    flex_direction: "row",
    align_items: "center",
    justify_content: "space_evenly",
    bg_color: "#1e1e1e"
});

// Column 1
let col_img = root.add_box(#{
    flex_direction: "column",
    align_items: "center",
    width: "45%",
    height: "80%",
    justify_content: "center",
    bg_color: "#252525",
    border_radius: 20,
    shadow_color: "#000000",
    shadow_blur: 30.0,
    shadow_y: 10.0
});

col_img.add_text(#{
    content: "Image",
    size: 50,
    color: "#ffffff",
    margin: 30
});

// Image Container (Circle)
let img_container = col_img.add_box(#{
    width: 300,
    height: 300,
    border_radius: 150,
    border_width: 8,
    border_color: "#ffffff",
    overflow: "hidden", // Clip image to circle
    shadow_color: "#000000",
    shadow_blur: 20.0
});

let img = img_container.add_image("tests/assets/test_image.png", #{
    width: "100%",
    height: "100%",
});

// Animate Container
img_container.animate("scale", 0.8, 1.1, 4.0, "ease_in_out");
img_container.animate("rotation", -10.0, 10.0, 5.0, "ease_in_out");


// Column 2
let col_vid = root.add_box(#{
    flex_direction: "column",
    align_items: "center",
    width: "45%",
    height: "80%",
    justify_content: "center",
    bg_color: "#252525",
    border_radius: 20,
    shadow_color: "#000000",
    shadow_blur: 30.0,
    shadow_y: 10.0
});

col_vid.add_text(#{
    content: "Video",
    size: 50,
    color: "#ffffff",
    margin: 30
});

// Video Container
let vid_container = col_vid.add_box(#{
    width: "90%",
    height: 400, // Fixed height for container
    border_radius: 10,
    border_width: 4,
    border_color: "#4CAF50",
    overflow: "hidden",
    shadow_color: "#000000",
    shadow_blur: 20.0
});

let vid = vid_container.add_video("tests/assets/test_video.mp4", #{
    width: "100%",
    height: "100%"
});

// Animate Video Opacity
vid.animate("opacity", 0.0, 1.0, 2.0, "ease_in");

movie
