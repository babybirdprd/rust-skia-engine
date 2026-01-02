# High-Level API Roadmap (Director Standard Library)

This document outlines the roadmap for creating a "Higher Level" API for the Director Engine. The goal is to evolve the scripting experience from imperative, low-level primitives to a declarative, component-based "Standard Library" that enables humans and AI agents to create professional-quality video content efficiently.

## Vision

Shift from **"Drawing Boxes & Text"** to **"Composing Scenes & Stories"**.

*   **Current State:** Manual layout (Flexbox props), explicit keyframes, repetitive style maps.
*   **Target State:** Semantic components (`TitleCard`, `LowerThird`), fluent animations (`.fade_in()`), and smart layouts (`VStack`, `Grid`).

## Architecture: The "StdLib" approach

Instead of hardcoding high-level features into the Rust core, we will implement a **Standard Library in Rhai**. This allows for rapid iteration, user-extensibility, and keeps the core engine lean.

### Layered Architecture

1.  **Level 0: Rust Core (Existing)**
    *   Primitives: `Box`, `Text`, `Image`, `Video`.
    *   Systems: Taffy Layout, Skia Rendering, FFMpeg encoding.
    *   API: `add_box(map)`, `animate(prop, v1, v2)`.

2.  **Level 1: The `std` Module (New)**
    *   Rhai modules loaded automatically at startup.
    *   `std/core`: Wrappers for fluent method chaining.
    *   `std/layout`: Layout presets (`HStack`, `VStack`, `Grid`).
    *   `std/motion`: Animation presets (`fade_in`, `slide_up`, `shake`).
    *   `std/components`: UI Kits (`Card`, `Badge`, `Avatar`).

3.  **Level 2: The User Script**
    *   Clean, readable, declarative code using `std` modules.

---

## Roadmap

### Phase 1: Core Foundation & Fluent API
*Goal: Eliminate the verbosity of the current API.*

1.  **Create `std/core.rhai`**
    *   Implement a wrapper class/map pattern to enable method chaining.
    *   *Before:* `let b = scene.add_box(props); b.set_style(more_props);`
    *   *After:* `scene.box().size("100%", 200).bg("#333")`
2.  **Smart Defaults**
    *   Nodes should default to reasonable flex properties (e.g., `center` alignment) unless specified.

### Phase 2: The Motion System
*Goal: "Make it move" without doing math.*

1.  **Create `std/motion.rhai`**
    *   Implement behavioral animations relative to current state.
    *   **Entrance:** `.fade_in(duration)`, `.slide_in_up(duration)`, `.zoom_in(duration)`.
    *   **Emphasis:** `.pulse()`, `.shake()`, `.heartbeat()`.
    *   **Exit:** `.fade_out(duration)`, `.slide_out_down(duration)`.
2.  **Staggering & Sequencing**
    *   Helpers to animate children with delays.
    *   `animate_stagger(children, "fade_in", 0.1)`

### Phase 3: Layout & Typography
*Goal: Professional layouts out-of-the-box.*

1.  **Create `std/layout.rhai`**
    *   `HStack(children, spacing)`: Row with gap.
    *   `VStack(children, spacing)`: Column with gap.
    *   `Grid(rows, cols)`: Simplified grid builder.
2.  **Create `std/typography.rhai`**
    *   Define a type scale (H1, H2, Body, Caption) linked to the design system.
    *   `Title("Hero Text")` vs `add_text(#{ size: 120, weight: "bold" ... })`.

### Phase 4: Component Library
*Goal: Reusable video elements.*

1.  **Create `std/components.rhai`**
    *   `Card(content)`: Box with rounded corners, shadow, and glassmorphism support.
    *   `LowerThird(title, subtitle)`: Animated name tag for speakers.
    *   `ProgressBar(percent)`: Animated indicators.
    *   `CodeBlock(code)`: Syntax-highlighted text container (mock).

### Phase 5: AI Context & Documentation
*Goal: Ensure LLMs can use this API effectively.*

1.  **Type Definitions**: Create a pseudo-schema or `.d.ts` equivalent for Rhai modules.
2.  **Examples**: Create `examples/high_level_demo.rhai` showcasing the new syntax.
3.  **System Prompt**: Update `AGENTS.md` with the new API patterns.

---

## Example Syntax Comparison

**Current (Low Level):**
```rhai
let container = scene.add_box(#{
    width: "100%", height: "100%",
    justify_content: "center", align_items: "center",
    flex_direction: "column"
});

let title = container.add_text(#{
    content: "Hello World",
    size: 100.0, color: "#fff"
});
title.animate("opacity", 0.0, 1.0, 1.0, "linear");
title.animate("y", 50, 0, 1.0, "ease_out");
```

**Proposed (High Level):**
```rhai
import "std/layout" as layout;
import "std/typography" as type;

let container = layout.center(scene);

let title = container.add(type.h1("Hello World"))
    .color(theme.white)
    .enter_slide_up(1.0);
```

## Next Steps for Development

1.  Set up the `std` directory structure in `crates/director-core`.
2.  Modify `scripting.rs` to load all `.rhai` files from `std` into the Engine context at startup.
3.  Begin implementing `std/core` and `std/motion` as the first proof-of-concept.
