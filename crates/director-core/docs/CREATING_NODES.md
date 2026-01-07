# Creating New Nodes

This guide explains how to extend the `director-engine` by adding a new node type (e.g., `AvatarNode`, `GraphNode`, `ParticleNode`).

Nodes are the building blocks of the Scene Graph. They implement the `Element` trait, which handles layout, rendering, and property animation.

## 1. Define the Node Struct

Create a new file in `crates/director-core/src/node/` (e.g., `my_node.rs`).

Define your struct. It must be `Clone` and `Debug`.
Usually, you will want to store:
- `style`: `taffy::style::Style` for layout.
- Animated properties (e.g., `opacity: Animated<f32>`).
- Any specific data (images, text, configuration).

```rust
use crate::animation::Animated;
use crate::element::Element;
use crate::errors::RenderError;
use skia_safe::{Canvas, Paint, Rect, Color4f};
use std::any::Any;
use taffy::style::Style;

#[derive(Debug, Clone)]
pub struct MyNode {
    pub style: Style,
    pub opacity: Animated<f32>,
    pub custom_prop: f32,
}

impl MyNode {
    pub fn new() -> Self {
        Self {
            style: Style::DEFAULT,
            opacity: Animated::new(1.0),
            custom_prop: 0.0,
        }
    }
}
```

## 2. Implement the Element Trait

The `Element` trait allows the engine to interact with your node.

```rust
impl Element for MyNode {
    // Boilerplate for downcasting
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }

    // Layout
    fn layout_style(&self) -> Style { self.style.clone() }
    fn set_layout_style(&mut self, style: Style) { self.style = style; }

    // Update State (called every frame before render)
    // Return true if visual state changed
    fn update(&mut self, time: f64) -> bool {
        self.opacity.update(time);
        true
    }

    // Render (Draw to Skia Canvas)
    fn render(
        &self,
        canvas: &Canvas,
        rect: Rect,
        parent_opacity: f32,
        draw_children: &mut dyn FnMut(&Canvas),
    ) -> Result<(), RenderError> {
        let op = self.opacity.current_value * parent_opacity;

        // Example drawing logic
        let mut paint = Paint::new(Color4f::new(1.0, 0.0, 0.0, op), None);
        canvas.draw_rect(rect, &paint);

        // Draw children (if your node supports children)
        draw_children(canvas);
        Ok(())
    }

    // Animation Support
    fn animate_property(
        &mut self,
        property: &str,
        start: f32,
        target: f32,
        duration: f64,
        easing: &str,
    ) {
        use crate::node::parse_easing;
        let ease = parse_easing(easing);

        match property {
            "opacity" => self.opacity.add_segment(start, target, duration, ease),
            _ => {} // Ignore unknown properties
        }
    }

    // Optional: Spring Animation Support
    fn animate_property_spring(
        &mut self,
        property: &str,
        start: Option<f32>,
        target: f32,
        config: crate::animation::SpringConfig,
    ) {
         match property {
            "opacity" => {
                 if let Some(s) = start {
                     self.opacity.add_spring_with_start(s, target, config);
                 } else {
                     self.opacity.add_spring(target, config);
                 }
            }
            _ => {}
        }
    }
}
```

## 3. Register the Module

Add your new module to `crates/director-core/src/node/mod.rs`:

```rust
pub mod my_node;
pub use my_node::MyNode;
```

## 4. Expose to Rhai Scripting

To let users create this node via scripts (e.g., `node.add_my_node()`), update `crates/director-core/src/scripting.rs`.

Add a registration function inside `register_rhai_api`:

```rust
engine.register_fn("add_my_node", |parent: &mut NodeHandle, props: rhai::Map| {
    let mut d = parent.director.lock().unwrap();

    let mut node = MyNode::new();

    // Helper to parse standard layout props (width, height, etc.)
    parse_layout_style(&props, &mut node.style);

    // Parse custom props
    if let Some(v) = props.get("custom_prop").and_then(|v| v.as_float().ok()) {
        node.custom_prop = v as f32;
    }

    let id = d.scene.add_node(Box::new(node));

    // Add as child to parent
    d.scene.add_child(parent.id, id);

    NodeHandle {
        director: parent.director.clone(),
        id,
    }
});
```

## 5. (Optional) Intrinsic Sizing

If your node has a natural size (like text or image) and doesn't rely solely on explicit `width/height`:
1. Override `needs_measure()` to return `true`.
2. Implement `measure()` in the `Element` trait.

## 6. (Advanced) JSON Schema Support

If you want your node to be created via JSON (not just Rhai scripts), you must update the Schema and Pipeline crates.

1.  **Update Schema**: In `crates/director-schema/src/lib.rs`, add a new variant to `NodeKind`:
    ```rust
    #[derive(Serialize, Deserialize, ...)]
    pub enum NodeKind {
        // ...
        MyNode {
            custom_prop: f32,
        },
    }
    ```

2.  **Update Pipeline**: In `crates/director-pipeline/src/lib.rs`, update `build_node_recursive` to handle the new variant:
    ```rust
    NodeKind::MyNode { custom_prop } => {
        let mut n = MyNode::new();
        n.custom_prop = *custom_prop;
        Box::new(n)
    },
    ```

## Checklist for Contributors

- [ ] **Node Implementation**: Created `crates/director-core/src/node/your_node.rs`.
- [ ] **Trait Compliance**: Implemented `Element`, `Clone`, and `Debug`.
- [ ] **Layout**: Implemented `layout_style` and `set_layout_style`.
- [ ] **Rendering**: Implemented `render` using Skia. Checked generic `rect` bounds.
- [ ] **Animation**: Implemented `animate_property` for relevant fields.
- [ ] **Module Export**: Added `pub mod` in `src/node/mod.rs`.
- [ ] **Scripting API**: Added `add_your_node` function in `src/scripting.rs`.
- [ ] **Property Parsing**: Ensured Rhai maps (props) are parsed correctly (handling floats/ints/strings).
- [ ] **Tests**: Added a basic test or updated `examples/` to verify the node renders.
- [ ] **(Optional) Schema**: Updated `NodeKind` in `director-schema` and loader logic in `director-pipeline`.
