# Scripting API Reference

This document provides a comprehensive reference for the Rhai scripting API used in `director-engine`.

## Table of Contents
1. [Global Functions](#global-functions)
2. [Theme API](#theme-api)
3. [MovieHandle](#moviehandle)
4. [SceneHandle](#scenehandle)
5. [NodeHandle](#nodehandle)
6. [AudioTrackHandle](#audiotrackhandle)
7. [Property Reference](#property-reference)

---

## Global Functions

### `new_director(width, height, fps)`
Creates a new movie project.
*   **width**: `Integer`. Video width in pixels.
*   **height**: `Integer`. Video height in pixels.
*   **fps**: `Integer`. Frames per second.
*   **Returns**: `MovieHandle`

### `new_director(width, height, fps, config)`
Creates a new movie project with configuration.
*   **config**: `Map`.
    *   `mode`: "preview" (default) or "export".
*   **Returns**: `MovieHandle`

### `rand_float(min, max)`
Generates a random floating-point number.
*   **min**: `Float`. Inclusive lower bound.
*   **max**: `Float`. Exclusive upper bound.
*   **Returns**: `Float`

---

## Theme API
The `theme` module provides access to standardized design tokens.

### `theme.space(key)`
*   **key**: `String` (e.g., "sm", "md", "lg").
*   **Returns**: `Float` (pixel value).

### `theme.safe_area(platform)`
Returns safe area insets for a specific platform.
*   **platform**: `String` (e.g., "tiktok", "instagram", "youtube_shorts"). Defaults to "desktop".
*   **Returns**: `Map` `{ top, bottom, left, right }`.

### `theme.radius(key)`
*   **key**: `String`.
*   **Returns**: `Float`.

### `theme.border(key)`
*   **key**: `String`.
*   **Returns**: `Float`.

### `theme.z(key)`
*   **key**: `String` (e.g., "background", "overlay").
*   **Returns**: `Integer`.

---

## MovieHandle
The root object representing the entire video project.

### `add_scene(duration)`
Adds a new scene to the end of the timeline.
*   **duration**: `Float`. Duration in seconds.
*   **Returns**: `SceneHandle`

### `add_transition(from_scene, to_scene, type, duration, easing)`
Adds a visual transition between two scenes.
*   **from_scene**: `SceneHandle`.
*   **to_scene**: `SceneHandle`.
*   **type**: `String`. "fade", "slide_left", "slide_right", "wipe_left", "wipe_right", "circle_open".
*   **duration**: `Float`.
*   **easing**: `String`. "linear", "ease_in", "ease_out", "ease_in_out".

### `add_audio(path)`
Adds a global audio track (e.g., background music) that plays across scenes.
*   **path**: `String`. File path to the audio asset.
*   **Returns**: `AudioTrackHandle`

### `configure_motion_blur(samples, shutter_angle)`
Configures global motion blur settings.
*   **samples**: `Integer`. Samples per frame (e.g., 4, 8, 16). Higher is smoother but slower.
*   **shutter_angle**: `Float`. Degrees (e.g., 180.0 for standard cinematic blur).

---

## SceneHandle
Represents a specific time segment on the timeline.

### `add_box(props)`
Adds a generic container box to the scene root.
*   **props**: `Map`. Layout and style properties.
*   **Returns**: `NodeHandle`

### `add_text(props)`
Adds a text node to the scene root.
*   **props**: `Map`. Text and style properties.
*   **Returns**: `NodeHandle`

### `add_image(path)`
Adds an image to the scene root.
*   **path**: `String`.
*   **Returns**: `NodeHandle`

### `add_image(path, props)`
Adds an image with initial style properties.
*   **props**: `Map`.

### `add_video(path)`
Adds a video element to the scene root.
*   **path**: `String`.
*   **Returns**: `NodeHandle`

### `add_video(path, props)`
Adds a video with initial style properties.

### `add_lottie(path)`
Adds a Lottie animation to the scene root.
*   **path**: `String`. JSON file path.
*   **Returns**: `NodeHandle`

### `add_lottie(path, props)`
Adds a Lottie with configuration.
*   **props**: `Map`. Can include standard style props plus:
    *   `speed`: `Float` (default 1.0).
    *   `loop`: `Boolean` (default true).
    *   `assets`: `Map` (Dynamic asset injection).

### `add_svg(path)`
Adds an SVG vector graphic to the scene root.
*   **path**: `String`.
*   **Returns**: `NodeHandle`

### `add_svg(path, props)`
Adds an SVG with initial style properties.

### `add_composition(movie_handle)`
Nests another `MovieHandle` as a child node in this scene.
*   **movie_handle**: `MovieHandle`. The movie to embed.
*   **Returns**: `NodeHandle`

### `add_composition(movie_handle, props)`
Nests a composition with initial style properties.

### `add_audio(path)`
Adds an audio track specific to this scene. It will be clipped to the scene's duration.
*   **path**: `String`.
*   **Returns**: `AudioTrackHandle`

---

## NodeHandle
Represents a visual element in the scene graph.

### Creation Methods (Children)
These methods add a child node to the current node.
*   `add_box(props)`
*   `add_text(props)`
*   `add_image(path)`
*   `add_image(path, props)`
*   `add_video(path)`
*   `add_video(path, props)`
*   `add_lottie(path)`
*   `add_lottie(path, props)`
*   `add_svg(path)`
*   `add_svg(path, props)`

### Lifecycle
*   `destroy()`: Removes the node and its children from the scene graph.

### Modification
*   `set_style(props)`: Updates the layout/style properties of the node.
*   `set_content(content)`: Updates text content (String or Array of Maps for rich text).
*   `set_pivot(x, y)`: Sets the transform pivot point (0.0 to 1.0). Default is (0.5, 0.5).
*   `set_mask(mask_node)`: Uses another node to mask this node (Alpha Matte). Reparents the mask node.
*   `set_blend_mode(mode)`: Sets the blend mode (e.g., "multiply", "screen", "overlay").
*   `set_blur(radius)`: Quickly applies a Gaussian blur.

### Animation
*   `animate(prop, start, end, duration, easing)`
    *   **prop**: "x", "y", "scale", "rotation", "opacity", etc.
*   `animate(prop, start_vec, end_vec, duration, easing)`
    *   For vector uniforms in shaders.
*   `animate(prop, target, spring_config)`
    *   Physics-based animation from current value to target.
    *   **spring_config**: `#{ stiffness: 100.0, damping: 10.0, mass: 1.0 }`
*   `animate(prop, start, target, spring_config)`
    *   Physics-based animation with explicit start value.
*   `path_animate(svg_path, duration, easing)`
    *   Moves the node along an SVG path string.
*   `add_animator(start_idx, end_idx, prop, start, end, duration, easing)`
    *   **TextNode Only**. Animates specific characters (graphemes) in a text string.
    *   **prop**: "y", "scale", "opacity", "rotation".

### Effects
*   `apply_effect(name)`: Applies a preset effect ("grayscale", "sepia", "invert"). Returns a new `NodeHandle` pointing to the effect wrapper.
*   `apply_effect(name, strength)`: Applies a variable effect ("contrast", "brightness", "blur").
*   `apply_effect("shader", config)`: Applies a custom Runtime Shader (SkSL).
    *   **config**: `#{ code: "...", uniforms: #{ ... } }`

---

## AudioTrackHandle
*   `animate_volume(start, end, duration, easing)`

---

## Property Reference

### Layout Properties (Taffy/Flexbox)
Used in `add_box`, `set_style`, etc.
*   `width`, `height`: Number, "auto", or "50%".
*   `flex_direction`: "row", "column", "row_reverse", "column_reverse".
*   `align_items`: "flex_start", "center", "flex_end", "stretch".
*   `justify_content`: "flex_start", "center", "flex_end", "space_between", "space_around", "space_evenly".
*   `flex_grow`, `flex_shrink`: Number.
*   `padding`, `margin`: Number or "10%".

### Style Properties
*   `bg_color`: Hex String.
*   `border_radius`: Number.
*   `border_width`: Number.
*   `border_color`: Hex String.
*   `shadow_color`: Hex String.
*   `shadow_blur`: Number.
*   `shadow_x`: Number.
*   `shadow_y`: Number.
*   `overflow`: "visible", "hidden".

### Text Properties
*   `content`: String or Array of Spans.
*   `size`: Number.
*   `color`: Hex String.
*   `weight`: "bold" or "normal".
*   `fit`: "shrink" or "none".
*   `min_size`, `max_size`: Number.
*   `text_shadow_color`: Hex String.
*   `text_shadow_blur`: Number.
*   `text_shadow_x`: Number.
*   `text_shadow_y`: Number.

### Text Span Properties (Rich Text)
Used inside the `content` array.
*   `text`: String.
*   `color`, `size`, `weight`.
*   `background_color`: Hex String.
*   `background_padding`: Number.
*   `stroke_width`: Number.
*   `stroke_color`: Hex String.
*   `fill_gradient`: Array of colors or Map (`#{ colors: [...], start: [x, y], end: [x, y] }`).
