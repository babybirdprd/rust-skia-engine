# Director Core Documentation

Welcome to the documentation for `director-core`, the heart of the Director engine.

## Overview

`director-core` is a Rust crate that provides the fundamental data structures, systems, and logic for a 2D rendering and animation engine. It is designed to be embedded in other applications (like the `director-cli` binary) or used as a standalone library.

## Key Concepts

*   **Scene Graph**: A hierarchical tree of nodes representing visual elements.
*   **Elements**: The specific behavior and rendering logic of a node (e.g., `TextNode`, `VideoNode`).
*   **Director**: The central manager that orchestrates updates, layout, and rendering.
*   **Systems**:
    *   **Layout**: Powered by `taffy`, calculates position and size of nodes.
    *   **Rendering**: Powered by `skia-safe`, draws nodes to a canvas.
    *   **Animation**: Interpolates values over time using keyframes or springs.

## Documentation Index

1.  [Architecture](./ARCHITECTURE.md) - The high-level design, frame loop, and data flow.
2.  [Scene Graph](./SCENE_GRAPH.md) - How nodes are managed, stored, and traversed.
3.  [Implementing Elements](./ELEMENT_TRAIT.md) - A guide to creating new node types.
4.  [Systems](./SYSTEMS.md) - Deep dive into Layout, Rendering, Assets, and Audio.
5.  [Animation](./ANIMATION.md) - How the animation system works internally.

## Getting Started

If you are new to the codebase, start by reading the [Architecture](./ARCHITECTURE.md) document to understand the "Big Picture". Then, look at [Implementing Elements](./ELEMENT_TRAIT.md) to see how individual nodes are constructed.
