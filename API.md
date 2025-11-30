# API Reference

This document provides a comprehensive reference for the Rhai scripting API exposed by the `director-engine`.

## Global Modules

### `theme`
The `theme` module provides access to the Design System tokens.

*   **`theme.space(key: string) -> float`**
    *   Returns spacing value (pixels).
    *   Keys: `none`, `xxs` (4), `xs` (8), `sm` (12), `md` (16), `lg` (24), `xl` (32), `2xl` (48), `3xl` (64), `4xl` (80), `5xl` (120), `6xl` (160), `7xl` (200).
*   **`theme.radius(key: string) -> float`**
    *   Returns border radius.
    *   Keys: `none`, `xs`, `sm`, `md` (8), `lg`, `xl`, `2xl`, `3xl`, `full`.
*   **`theme.border(key: string) -> float`**
    *   Returns border width.
    *   Keys: `none`, `thin` (1), `base` (2), `thick` (4), `heavy` (8), `ultra` (12).
*   **`theme.z(key: string) -> int`**
    *   Returns z-index value.
    *   Keys: `underground` (-10), `background` (0), `base` (1), `content` (10), `elevated` (20), `overlay` (50), `modal` (100), etc.
*   **`theme.safe_area(platform: string) -> Map`**
    *   Returns a Map `{ top, bottom, left, right }`.
    *   Platforms: `desktop` (16:9), `mobile` (9:16), `youtube_shorts`, `tiktok`, `instagram_reel`, `instagram_story`, `linkedin`, `twitter`.

---

## Objects

### `Movie`
The root object created via `new_director`.

*   **`new_director(width: int, height: int, fps: int) -> Movie`**
    *   Creates a new movie context.
*   **`new_director(width: int, height: int, fps: int, config: Map) -> Movie`**
    *   Creates a movie with config (e.g., `#{ mode: "export" }`).
*   **`add_scene(duration: float) -> Scene`**
    *   Adds a new scene to the timeline.
*   **`add_audio(path: string) -> AudioTrack`**
    *   Adds a global audio track (plays across all scenes).
*   **`add_transition(from: Scene, to: Scene, type: string, duration: float, easing: string)`**
    *   Adds a transition between two scenes.
    *   **Types**: `fade`, `slide_left`, `slide_right`, `wipe_left`, `wipe_right`, `circle_open`.
    *   **Easing**: `linear`, `ease_in`, `ease_out`, `ease_in_out`.
*   **`configure_motion_blur(samples: int, shutter_angle: float)`**
    *   Configures motion blur settings.
    *   `samples`: Number of sub-frames (e.g., 4 or 8).
    *   `shutter_angle`: Standard shutter angle (e.g., 180.0 for cinematic blur).

### `Scene`
Represents a segment of time in the movie.

*   **`add_box(props: Map) -> Node`**
    *   Adds a container box to the scene root.
*   **`add_text(props: Map) -> Node`**
    *   Adds text to the scene root.
*   **`add_composition(movie: Movie) -> Node`**
    *   Adds a nested composition (sub-timeline) to the scene.
*   **`add_composition(movie: Movie, props: Map) -> Node`**
    *   Adds a nested composition with layout properties.
*   **`add_audio(path: string) -> AudioTrack`**
    *   Adds an audio track synced to this scene.

### `Node`
A visual element in the scene graph (Box, Text, Image, Video, Composition).

*   **`add_box(props: Map) -> Node`**
    *   Adds a child box.
*   **`add_text(props: Map) -> Node`**
    *   Adds child text.
*   **`add_image(path: string) -> Node`**
    *   Adds an image child.
*   **`add_video(path: string) -> Node`**
    *   Adds a video child.
*   **`set_content(content: string | Array<Map>)`**
    *   Updates text content (supports Rich Text).
*   **`set_style(style: Map)`**
    *   Updates style properties (text spans).
*   **`set_pivot(x: float, y: float)`**
    *   Sets the transformation pivot point (default 0.5, 0.5).
*   **`set_mask(mask_node: Node)`**
    *   Applies another node as an alpha mask for this node.
*   **`set_blend_mode(mode: string)`**
    *   Sets the blend mode.
    *   **Modes**: `src_over` (default), `screen`, `overlay`, `multiply`, `darken`, `lighten`, `color_dodge`, `soft_light`, `difference`, etc.
*   **`set_blur(radius: float)`**
    *   Sets/Animates gaussian blur.
*   **`animate(prop: string, start: float, end: float, duration: float, easing: string)`**
    *   Animates a numeric property.
    *   **Props**:
        *   Layout: `width`, `height` (if numeric), `flex_grow`, etc.
        *   Transform: `x`, `y`, `scale`, `rotation`, `skew_x`, `skew_y`.
        *   Style: `opacity`, `blur`, `size` (text).
*   **`path_animate(svg_path: string, duration: float, easing: string)`**
    *   Animates the node along an SVG path.
*   **`add_animator(start_idx: int, end_idx: int, prop: string, start: float, end: float, duration: float, easing: string)`**
    *   (Text Only) Animates a property on a specific range of characters (graphemes).

### `AudioTrack`
Handle to an audio resource.

*   **`animate_volume(start: float, end: float, duration: float, easing: string)`**
    *   Animates the volume (0.0 to 1.0+).

---

## Properties

### Layout (Flexbox)
Applied via `add_box` or updated via maps.
*   **Dimensions**: `width`, `height` (float, "auto", "50%").
*   **Flex**: `flex_direction` (row, column, row_reverse, column_reverse), `flex_grow` (float), `flex_shrink` (float).
*   **Alignment**: `align_items`, `justify_content` (start, end, center, stretch, space_between, space_around, space_evenly).
*   **Spacing**: `margin`, `padding` (float, "5%").

### Visual Style
*   **Background**: `bg_color` (Hex string).
*   **Border**: `border_radius`, `border_width` (float), `border_color`.
*   **Shadow**: `shadow_color`, `shadow_blur`, `shadow_x`, `shadow_y`.
*   **Opacity**: `opacity` (0.0 - 1.0).
*   **Overflow**: `overflow` ("visible", "hidden").

### Text Style
*   `content`: String or Array of Rich Text Maps.
*   `color`: Hex string.
*   `size`: Float (font size).
*   `weight`: "bold" or "normal" (numeric weights supported internally but simple mapping exposed).
*   **Rich Text Span Props**:
    *   `text`: String content.
    *   `background_color`: Hex string.
    *   `background_padding`: Float.
    *   `stroke_width`, `stroke_color`.
    *   `fill_gradient`: Array of colors or Map `{ colors: [], start: [x,y], end: [x,y] }`.
