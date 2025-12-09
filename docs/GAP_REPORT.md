# Gap Report: Schema vs. Core Engine

This document identifies discrepancies between the internal engine capabilities (`director-core`) and the exposed API/Schema (`director-schema`, `scripting.rs`).

## 1. Visual Effects

### Drop Shadow
*   **Gap**: The `EffectType::DropShadow` variant exists in `director-core`, but it is not exposed as a generic wrapper effect in `scripting.rs` (via `apply_effect`).
*   **Workaround**: Users must currently rely on the built-in `shadow_color` / `shadow_blur` properties of `BoxNode` and `TextNode`, which use the same underlying logic but are less flexible (e.g., cannot be stacked or applied to Groups/Compositions).

### Shader Uniforms
*   **Status**: `RuntimeShader` supports `Float` and `Vec<f32>` uniforms.
*   **Gap**: There is no support for `Color` uniforms directly; users must pass colors as `Vec<f32>` arrays (normalized 0.0-1.0).

## 2. Text Capabilities

### Text Fit
*   **Status**: `TextFit::Shrink` is implemented and exposed.
*   **Gap**: `TextFit::Wrap` or other sizing strategies available in `cosmic-text` are not fully exposed or configurable via the API.

## 3. Animation

### Easing Functions
*   **Gap**: `EasingType::BounceOut` is mapped, but other standard easings provided by the `keyframe` crate (e.g., `Back`, `Elastic`) are not exposed in the API string parser (`parse_easing`).

## 4. Compositing

### Masking
*   **Gap**: The `set_mask` API reparents the mask node. There is no support for using a node as a mask *without* moving it in the hierarchy (e.g., referencing a shared mask resource).

## 5. Layout

### Taffy Integration
*   **Gap**: Some advanced Taffy properties (e.g., Grid Layout specifics) are available in `director-core`'s `Style` struct but are not parsed in `scripting.rs` (e.g., `grid_template_columns`, `gap` support is partial/implicit).
