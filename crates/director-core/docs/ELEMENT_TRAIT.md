# Implementing Elements

The `Element` trait (`src/element.rs`) is the interface for creating new visual node types.

## The Trait

```rust
pub trait Element: std::fmt::Debug + ElementClone {
    fn update(&mut self, time: f64) -> bool;
    fn layout_style(&self) -> Style;
    fn render(&self, canvas: &Canvas, rect: Rect, opacity: f32, ...);
    // ... optional methods
}
```

## Step-by-Step Implementation

To add a new node type (e.g., `CircleNode`):

1.  **Define the Struct**:
    ```rust
    #[derive(Clone, Debug)]
    pub struct CircleNode {
        pub style: Style,
        pub color: Color,
        pub radius: Animated<f32>,
    }
    ```
    *   Must be `Clone` and `Debug`.
    *   Store layout `Style` if you want it to participate in Flexbox.

2.  **Implement `Element`**:

    *   `layout_style()`: Return `self.style.clone()`.
    *   `update(time)`: Update any animated properties. Return `true` if visuals changed.
        ```rust
        fn update(&mut self, time: f64) -> bool {
            self.radius.update(time);
            // Return true if animating, or if dirty
            true
        }
        ```
    *   `render(...)`: Draw using Skia.
        ```rust
        fn render(&self, canvas: &Canvas, rect: Rect, ...) -> Result<(), RenderError> {
             let paint = Paint::new(self.color.to_skia(), ...);
             canvas.draw_circle((rect.center_x(), rect.center_y()), self.radius.val(), &paint);
             Ok(())
        }
        ```
    *   `animate_property(...)`: Map string keys to your fields.
        ```rust
        fn animate_property(&mut self, key: &str, ...) {
            match key {
                "radius" => self.radius.add_keyframe(...),
                _ => warn!("Unknown property"),
            }
        }
        ```

3.  **Register with API (Optional)**:
    If this node needs to be exposed to Rhai, you will need to add a helper in `director-cli` or `director-scripting` (depending on where the API binding lives) to construct it.

## Lifecycle Hooks

*   **`needs_measure` / `measure`**: Implement these if your node has an intrinsic size (like Text or Image) that layout depends on.
*   **`post_layout`**: Implement this if you need to know your final size before rendering (e.g., `TextFit::Shrink` logic).
*   **`get_audio`**: Implement if your node emits sound.

## Common Pitfalls

*   **Interior Mutability**: `render` takes `&self`. If you need to mutate state during render (e.g., caching), use `Mutex` or `RefCell` internally (like `LottieNode` does).
*   **Asset Loading**: Do not load heavy assets in `new()`. Pass a handle or load them lazily/async via `AssetManager`.
