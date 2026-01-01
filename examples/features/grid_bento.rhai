// Grid Bento Layout Demo
// Showcases 3x3 Grid with spanning items and absolute positioning

let movie = new_director(1080, 1080, 30);
let scene = movie.add_scene(5.0);

let root = scene.add_box(#{
    width: "100%",
    height: "100%",
    display: "grid",
    grid_template_columns: ["1fr", "1fr", "1fr"],
    grid_template_rows: ["1fr", "1fr", "1fr"],
    align_items: "stretch",
    justify_content: "stretch",
    gap: 20.0,
    padding: 40.0,
    bg_color: "#1a1a1a"
});

// Main Bento Item (2x2)
let bento_main = root.add_box(#{
    grid_column: "1 / span 2",
    grid_row: "1 / span 2",
    bg_color: "#3b82f6", // Blue 500
    border_radius: 20.0,
    display: "flex",
    justify_content: "center",
    align_items: "center"
});

bento_main.add_text(#{
    content: "Main Piece",
    size: 60.0,
    color: "#ffffff"
});

// Sidebar Item (1x3)
let sidebar = root.add_box(#{
    grid_column: "3",
    grid_row: "1 / span 3",
    bg_color: "#10b981", // Emerald 500
    border_radius: 20.0,
    display: "flex",
    justify_content: "center",
    align_items: "center"
});

sidebar.add_text(#{
    content: "Side",
    size: 40.0,
    rotation: 90.0,
    color: "#ffffff"
});

// Bottom Item 1 (1x1)
let bottom1 = root.add_box(#{
    grid_column: "1",
    grid_row: "3",
    bg_color: "#f59e0b", // Amber 500
    border_radius: 20.0,
    display: "flex",
    justify_content: "center",
    align_items: "center"
});

// Bottom Item 2 (1x1) with Absolute Fit-to-Parent
let bottom2 = root.add_box(#{
    grid_column: "2",
    grid_row: "3",
    bg_color: "#ef4444", // Red 500
    border_radius: 20.0,
    position: "relative" // Containing block for absolute child
});

// Absolute child that should fill bottom2
bottom2.add_box(#{
    position: "absolute",
    top: 0.0,
    left: 0.0,
    right: 0.0,
    bottom: 0.0,
    bg_color: "#ffffff",
    opacity: 0.3,
    border_radius: 20.0
});

bottom2.add_text(#{
    content: "Fit",
    size: 30.0,
    color: "#ffffff"
});

movie
