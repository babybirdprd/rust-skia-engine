// vfx_showcase.rhai
let movie = new_director(1920, 1080, 30, #{ mode: "export" });
let scene = movie.add_scene(4.0);

// Background
scene.add_box(#{
    width: "100%", height: "100%",
    flex_direction: "column",
    bg_color: "#222222"
});

let row1 = scene.add_box(#{
    width: "100%", height: "50%",
    flex_direction: "row",
    justify_content: "space_evenly",
    align_items: "center"
});

let row2 = scene.add_box(#{
    width: "100%", height: "50%",
    flex_direction: "row",
    justify_content: "space_evenly",
    align_items: "center"
});

fn add_card(parent, title, effect_name, params) {
    let card = parent.add_box(#{
        width: 400, height: 400,
        bg_color: "#444444",
        border_radius: 20.0,
        align_items: "center",
        justify_content: "center",
        overflow: "hidden"
    });

    // Add SVG - this also tests "Contain" aspect ratio
    // If it was "Stretch", it would look distorted if container was not square (though here it is 400x400)
    // To test Contain better, let's make the container non-square for one of them?
    // But for effects showcase, square is fine. The internal logic updates ensuring it's contained in 80%x80%.

    let img = card.add_svg("examples/rust-logo.svg", #{
        width: "80%", height: "80%"
    });

    // Apply Effect
    img.apply_effect(effect_name, params);

    // Label
    card.add_text(#{
        content: title,
        size: 30.0,
        color: "#FFFFFF",
        position: "absolute",
        bottom: 20
    });
}

add_card(row1, "Film Grain", "film_grain", 0.8);
add_card(row1, "Vignette", "vignette", 0.7);

add_card(row2, "Glitch", "glitch_analog", #{ speed: 2.0, amount: 0.1 });
add_card(row2, "Halftone", "halftone", #{ dot_size: 15.0 });

movie
