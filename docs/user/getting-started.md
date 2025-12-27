# Getting Started with Director Engine

Director Engine is a programmatic video rendering engine. You write scripts in **Rhai** and render them to video.

## Quick Start

```rhai
let movie = new_director(1920, 1080, 30);
let scene = movie.add_scene(5.0);

let root = scene.add_box(#{
    width: "100%",
    height: "100%",
    bg_color: "#1a1a2e"
});

let title = root.add_text(#{
    content: "Hello, Director!",
    size: 72.0,
    color: "#ffffff"
});

title.animate("scale", 0.8, 1.0, 1.0, "ease_out");

movie
```

## Running Your Script

```bash
cargo run --release -- your_script.rhai output.mp4
```

## Next Steps

- [Scripting Guide](scripting-guide.md) — Complete API reference
- [Examples](../../examples/) — Working demonstration scripts
