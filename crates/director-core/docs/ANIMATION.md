# Animation System

`director-core` supports two primary methods of animation: **Keyframes** and **Springs**.

## `Animated<T>` Struct

The core primitive is `Animated<T>`.

```rust
pub struct Animated<T: Clone> {
    pub initial_value: T,
    pub current_value: T,
    pub keyframes: Vec<Keyframe<T>>,
    pub spring: Option<SpringState<T>>,
}
```

*   `current_value`: The calculated value for the current frame. Elements read this during `render`.
*   `keyframes`: A list of time-sorted value targets.
*   `spring`: Physics simulation state if spring animation is active.

## Keyframe Animation

Standard linear/eased interpolation between points.

*   **Inputs**: `target_value`, `duration`, `easing` (e.g., "ease-in", "linear").
*   **Logic**: Finds the two keyframes surrounding the current `local_time`, calculates the progress (`0.0` to `1.0`), applies the easing curve, and interpolates.

## Spring Animation

Physics-based animation.

*   **Inputs**: `stiffness`, `damping`, `mass`, `target_value`.
*   **Logic**:
    *   When a spring is added, we "bake" the simulation into a series of dense keyframes (e.g., at 60fps intervals) from the *current* value to the *target*.
    *   This "Baking" strategy allows us to treat springs and keyframes uniformly during the render read-phase. It avoids unstable integration steps if frame times fluctuate, as the physics is pre-solved.

## Property Mapping

The generic `Element` trait has:
`fn animate_property(&mut self, property: &str, ...)`

Nodes must manually map string keys (e.g., "opacity", "radius") to their internal `Animated<T>` fields.
Standard transforms (x, y, scale, rotation) are handled automatically by `SceneNode` before calling the Element, so Elements don't need to implement those.
