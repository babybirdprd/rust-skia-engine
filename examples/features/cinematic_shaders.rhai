// Cinematic Shaders Demo
//
// Demonstrates the new cinematic effects:
// - Directional blur (motion blur)
// - Film grain
//
// These effects add professional, cinematic quality to animations.

let movie = new_director(1920, 1080, 30);

// Scene 1: Directional Blur (Motion Blur)
let scene1 = movie.add_scene(3.0);
let root1 = scene1.add_box(#{
    width: "100%",
    height: "100%",
    flex_direction: "column",
    justify_content: "center",
    align_items: "center",
    gap: 40.0,
    bg_color: "#0f0f23"
});

// Title
root1.add_text(#{
    content: "Directional Blur (Motion Blur)",
    font_size: 48.0,
    color: "#ffffff",
    font_weight: "bold"
});

// Fast-moving element with motion blur
let motion_container = root1.add_box(#{
    width: 800.0,
    height: 200.0,
    justify_content: "center",
    align_items: "center"
});

let moving_box = motion_container.add_box(#{
    width: 120.0,
    height: 120.0,
    bg_color: "#ff6b6b",
    border_radius: 20.0
});

// Apply motion blur effect (blur along horizontal axis)
moving_box.apply_effect("directional_blur", #{
    strength: 25.0,    // Blur distance in pixels
    angle: 0.0,        // 0° = horizontal (right)
    samples: 24        // Higher = smoother blur
});

// Animate the box moving fast across screen (positional args: prop, start, end, duration, easing)
moving_box.animate("x", -300.0, 300.0, 2.0, "ease_in_out");

// Subtitle showing the settings
root1.add_text(#{
    content: "strength: 25px, angle: 0°, samples: 24",
    font_size: 24.0,
    color: "#888888"
});

// Scene 2: Film Grain
let scene2 = movie.add_scene(3.0);
let root2 = scene2.add_box(#{
    width: "100%",
    height: "100%",
    flex_direction: "column",
    justify_content: "center",
    align_items: "center",
    gap: 40.0,
    bg_color: "#1a1a2e"
});

// Title
root2.add_text(#{
    content: "Film Grain Effect",
    font_size: 48.0,
    color: "#ffffff",
    font_weight: "bold"
});

// Container for comparison
let grain_row = root2.add_box(#{
    width: "80%",
    height: 300.0,
    flex_direction: "row",
    justify_content: "space_evenly",
    align_items: "center"
});

// Left: No grain (clean)
let clean_card = grain_row.add_box(#{
    width: 350.0,
    height: 250.0,
    flex_direction: "column",
    justify_content: "center",
    align_items: "center",
    gap: 20.0,
    bg_color: "#16213e",
    border_radius: 20.0
});

clean_card.add_text(#{
    content: "No Grain",
    font_size: 28.0,
    color: "#e94560",
    font_weight: "bold"
});

clean_card.add_text(#{
    content: "Clean digital look",
    font_size: 18.0,
    color: "#888888"
});

// Right: With film grain
let grainy_card = grain_row.add_box(#{
    width: 350.0,
    height: 250.0,
    flex_direction: "column",
    justify_content: "center",
    align_items: "center",
    gap: 20.0,
    bg_color: "#16213e",
    border_radius: 20.0
});

grainy_card.add_text(#{
    content: "Film Grain",
    font_size: 28.0,
    color: "#e94560",
    font_weight: "bold"
});

grainy_card.add_text(#{
    content: "Organic, cinematic feel",
    font_size: 18.0,
    color: "#888888"
});

// Apply film grain effect
grainy_card.apply_effect("grain", #{
    intensity: 0.15,   // Strength of the grain (0.0 - 1.0)
    size: 2.0          // Size of grain particles
});

// Subtitle
root2.add_text(#{
    content: "Grain adds texture and warmth to digital video",
    font_size: 24.0,
    color: "#888888"
});

// Scene 3: Combined Effects
let scene3 = movie.add_scene(3.0);
let root3 = scene3.add_box(#{
    width: "100%",
    height: "100%",
    flex_direction: "column",
    justify_content: "center",
    align_items: "center",
    gap: 40.0,
    bg_color: "#0d1117"
});

root3.add_text(#{
    content: "Combined Cinematic Look",
    font_size: 48.0,
    color: "#ffffff",
    font_weight: "bold"
});

// Hero element with both effects
let hero_container = root3.add_box(#{
    width: 600.0,
    height: 300.0,
    justify_content: "center",
    align_items: "center",
    bg_color: "#161b22",
    border_radius: 30.0
});

let hero_box = hero_container.add_box(#{
    width: 150.0,
    height: 150.0,
    justify_content: "center",
    align_items: "center",
    bg_color: "#58a6ff",
    border_radius: 25.0
});

hero_box.add_text(#{
    content: "CINEMA",
    font_size: 22.0,
    color: "#ffffff",
    font_weight: "bold"
});

// Apply subtle motion blur
hero_box.apply_effect("directional_blur", #{
    strength: 8.0,
    angle: 45.0,   // Diagonal blur
    samples: 16
});

// Apply grain to the entire container
hero_container.apply_effect("grain", #{
    intensity: 0.08,
    size: 1.5
});

// Animate scale for dynamic motion (positional args: prop, start, end, duration, easing)
hero_box.animate("scale", 0.8, 1.2, 1.5, "ease_in_out");

root3.add_text(#{
    content: "Motion blur + Film grain = Professional quality",
    font_size: 24.0,
    color: "#8b949e"
});

movie

