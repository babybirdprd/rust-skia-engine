// Flexbox Layout - Layout System Reference
//
// Demonstrates Taffy-based flexbox layout:
// - flex_direction (row, column)
// - justify_content (center, space_between, space_evenly, etc.)
// - align_items (center, flex_start, flex_end, stretch)
// - Percentage and fixed sizing
// - Padding and margin

let movie = new_director(1920, 1080, 30);
let scene = movie.add_scene(3.0);

// Root container - column layout
let root = scene.add_box(#{
    width: "100%",
    height: "100%",
    flex_direction: "column",
    justify_content: "center",
    align_items: "center",
    padding: 40.0,
    bg_color: "#0a0a0a"
});

// Header row - space between
let header = root.add_box(#{
    width: "100%",
    height: 80.0,
    flex_direction: "row",
    justify_content: "space_between",
    align_items: "center",
    padding: 20.0,
    bg_color: "#1a1a2e",
    border_radius: 10.0
});

header.add_text(#{ content: "Logo", size: 24.0, color: "#4ecdc4" });
header.add_text(#{ content: "Menu", size: 18.0, color: "#888888" });

// Content area - centered cards
let content = root.add_box(#{
    width: "100%",
    flex_grow: 1.0,
    flex_direction: "row",
    justify_content: "space_evenly",
    align_items: "center",
    margin: 40.0
});

// Three equal cards
for i in 0..3 {
    let card = content.add_box(#{
        width: 300.0,
        height: 400.0,
        flex_direction: "column",
        align_items: "center",
        justify_content: "center",
        bg_color: "#1a1a2e",
        border_radius: 20.0
    });
    
    card.add_text(#{
        content: `Card ${i + 1}`,
        size: 32.0,
        color: "#ffffff"
    });
}

// Footer
let footer = root.add_box(#{
    width: "100%",
    height: 60.0,
    justify_content: "center",
    align_items: "center",
    bg_color: "#1a1a2e",
    border_radius: 10.0
});

footer.add_text(#{ content: "© 2024 Director Engine", size: 14.0, color: "#666666" });

movie
