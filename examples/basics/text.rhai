// Rich Text Reference
//
// Demonstrates text features:
// - Simple text
// - Rich text with multiple spans
// - Font weight and style
// - Text background (highlight)
// - Text shadow
// - Shrink-to-fit

let movie = new_director(1920, 1080, 30);
let scene = movie.add_scene(3.0);

let root = scene.add_box(#{
    width: "100%",
    height: "100%",
    flex_direction: "column",
    justify_content: "space_evenly",
    align_items: "center",
    padding: 60.0,
    bg_color: "#0a0a0a"
});

// 1. Simple text
root.add_text(#{
    content: "Simple Text",
    size: 48.0,
    color: "#ffffff"
});

// 2. Rich text with multiple colored spans
root.add_text(#{
    content: [
        #{ text: "Rich ", color: "#ff6b6b", size: 48.0 },
        #{ text: "Colored ", color: "#4ecdc4", size: 48.0 },
        #{ text: "Text", color: "#ffe66d", size: 48.0 }
    ]
});

// 3. Bold and weighted text
root.add_text(#{
    content: [
        #{ text: "Light ", weight: 300, size: 36.0, color: "#888888" },
        #{ text: "Normal ", weight: 400, size: 36.0, color: "#aaaaaa" },
        #{ text: "Bold ", weight: 700, size: 36.0, color: "#ffffff" },
        #{ text: "Black", weight: 900, size: 36.0, color: "#ffffff" }
    ]
});

// 4. Text with background highlight
root.add_text(#{
    content: [
        #{ text: "Highlighted", size: 36.0, color: "#000000", bg_color: "#ffe66d", bg_padding: 8.0 }
    ]
});

// 5. Text with stroke
root.add_text(#{
    content: [
        #{ text: "Outlined", size: 48.0, color: "#000000", stroke_color: "#ffffff", stroke_width: 2.0 }
    ]
});

// 6. Shrink-to-fit (long text in constrained width)
let constrained = root.add_box(#{
    width: 400.0,
    height: 60.0,
    bg_color: "#1a1a2e",
    border_radius: 10.0,
    justify_content: "center",
    align_items: "center"
});

constrained.add_text(#{
    content: "This long text will shrink to fit the container",
    size: 32.0,
    color: "#ffffff",
    fit: "shrink",
    min_size: 12.0
});

movie
