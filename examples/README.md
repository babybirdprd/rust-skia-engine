# Director Engine Examples

Reference scripts demonstrating the Rhai API. All examples are validated by the test suite.

## Basics

| Script | Description |
|--------|-------------|
| [hello_world.rhai](basics/hello_world.rhai) | Minimal script - create movie, add scene, add text |
| [layout_flexbox.rhai](basics/layout_flexbox.rhai) | Flexbox layout with rows, columns, spacing |
| [animation.rhai](basics/animation.rhai) | Keyframe and spring animations |
| [text.rhai](basics/text.rhai) | Rich text, styling, shrink-to-fit |

## Features

| Script | Description |
|--------|-------------|
| [effects.rhai](features/effects.rhai) | Blur, grayscale, sepia, invert filters |
| [masking.rhai](features/masking.rhai) | Alpha masking and blend modes |
| [transitions.rhai](features/transitions.rhai) | Scene transitions (fade, slide, wipe) |
| [z_index.rhai](features/z_index.rhai) | Z-index ordering with absolute positioning |
| [image.rhai](features/image.rhai) | Image loading and animation |

## Running Examples

```bash
# Render to video
cargo run --release -- examples/basics/hello_world.rhai output.mp4

# Preview (if preview mode is implemented)
cargo run --release -- examples/basics/hello_world.rhai --preview
```

## Testing

All examples are validated by CI:

```bash
cargo test -p director-core examples
```
