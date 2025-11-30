# Director Engine Scripting Guide

This guide explains how to use the Rhai scripting language to create videos with `director-engine`.

## 1. Basic Setup

Every script starts by initializing the `Director` (the movie) and adding a `Scene`.

```rust
// Create a 1080x1920 (Vertical) movie at 30 FPS
let movie = new_director(1080, 1920, 30);

// Add a scene that lasts 5 seconds
let scene = movie.add_scene(5.0);

// Return the movie object at the end
movie
```

## 2. Layout (Flexbox)

The engine uses **Taffy** (Flexbox) for layout. You create boxes and nest them.

```rust
let container = scene.add_box(#{
    width: "100%",
    height: "100%",
    flex_direction: "column",
    justify_content: "center",
    align_items: "center",
    bg_color: "#1a1a1a"
});

let child = container.add_box(#{
    width: 200.0,
    height: 200.0,
    bg_color: "#FF0000",
    border_radius: 20.0
});
```

## 3. Typography & Rich Text

Text supports basic styling and rich text spans (mixed colors, fonts, gradients).

### Simple Text
```rust
container.add_text(#{
    content: "Simple Text",
    size: 48.0,
    color: "#FFFFFF",
    weight: "bold"
});
```

### Rich Text (Spans)
```rust
container.add_text(#{
    content: [
        #{ text: "Hello ", color: "#FFFFFF", size: 48.0 },
        #{
            text: "World",
            size: 64.0,
            fill_gradient: ["#FF0000", "#0000FF"] // Linear gradient
        }
    ]
});
```

## 4. Animation

You can animate numeric properties using `animate()`.

```rust
let box = scene.add_box(#{ width: 100.0, height: 100.0, bg_color: "#00FF00" });

// Animate width from 100 to 500 over 2 seconds
// Easing: linear, ease_in, ease_out, ease_in_out, bounce_out
box.animate("width", 100.0, 500.0, 2.0, "ease_out");

// Transform Animation
box.animate("rotation", 0.0, 360.0, 2.0, "linear");
box.animate("scale", 0.5, 1.5, 1.0, "ease_in_out");

// Path Animation (SVG path)
let ball = scene.add_box(#{ width: 20.0, height: 20.0, bg_color: "#FFF" });
ball.path_animate("M 0 0 C 100 0 100 100 200 100", 3.0, "ease_in_out");
```

## 5. Compositing

### Masks
You can use any node to mask another. The alpha channel of the mask node determines visibility.

```rust
let bg = scene.add_box(#{ width: "100%", height: "100%", bg_color: "#FF0000" });
let text_mask = scene.add_text(#{ content: "MASK", size: 200.0, weight: "bold" });

// 'text_mask' will now mask 'bg'. Only the red pixels where the text is will be visible.
bg.set_mask(text_mask);
```

### Blend Modes
Apply standard Photoshop-style blend modes to nodes.

```rust
let overlay = scene.add_box(#{ width: "100%", height: "100%", bg_color: "#00FF00" });
overlay.set_blend_mode("multiply"); // or "screen", "overlay", "soft_light", etc.
```

## 6. Nested Timelines (Compositions)

You can create reusable movie clips (Compositions) and nest them inside scenes.

```rust
// 1. Define the reusable movie
let clip = new_director(500, 500, 30);
let c_scene = clip.add_scene(2.0);
c_scene.add_box(#{ width: "100%", height: "100%", bg_color: "#0000FF" });

// 2. Add it to the main movie
// It behaves like a normal node but renders its own internal timeline
scene.add_composition(clip, #{
    width: 500.0,
    height: 500.0
});
```

## 7. Using the Theme System

The `theme` module provides standardized tokens.

```rust
let card = scene.add_box(#{
    padding: theme.space("md"),       // 16.0
    border_radius: theme.radius("lg"), // 12.0
    bg_color: "#333333",
    shadow_color: "#000000",
    shadow_blur: 10.0
});

// Safe Areas
let safe = theme.safe_area("tiktok"); // Returns { top, bottom, left, right }
let content = scene.add_box(#{
    margin: safe.top, // Use safe area margin
    width: "100%",
    height: "auto"
});
```

## 8. Audio & Transitions

### Audio
```rust
// Background Music (Global)
let bgm = movie.add_audio("assets/music.mp3");
bgm.animate_volume(0.0, 1.0, 2.0, "linear"); // Fade in

// Sound Effect (Scene Specific)
let sfx = scene.add_audio("assets/pop.wav");
```

### Transitions
Transitions blend two scenes. The engine automatically handles timing overlaps (Ripple Logic).

```rust
let scene1 = movie.add_scene(5.0);
let scene2 = movie.add_scene(5.0);

// Transition from Scene 1 to Scene 2
// This shifts Scene 2 earlier by 1.0s to overlap
movie.add_transition(scene1, scene2, "slide_left", 1.0, "ease_in_out");
```

## 9. Motion Blur

Enable cinematic motion blur for smoother animations.

```rust
// 8 samples per frame, 180 degree shutter angle
movie.configure_motion_blur(8, 180.0);
```
