# Scene Graph

The `SceneGraph` (`src/scene.rs`) is the data structure storing all visual objects.

## Structure

It is implemented as a "Flattened Tree" or "Arena" style vector.

```rust
pub struct SceneGraph {
    pub nodes: Vec<Option<SceneNode>>,
    pub free_indices: Vec<usize>,
    pub roots: Vec<NodeId>,
}
```

*   `nodes`: A growable vector storing all nodes. `NodeId` is simply the index into this vector.
*   `Option<SceneNode>`: Allows for "holes" when nodes are deleted.
*   `free_indices`: A stack of indices that were deleted and can be reused.

**Note**: This Simple Free-List approach means `NodeId` reuse is immediate. There are no generational indices. If a script holds a stale `NodeId` after destruction, it might accidentally modify a new, unrelated node.

## The SceneNode

A `SceneNode` wraps the specific content (`Element`) with generic properties common to all objects:

*   **Hierarchy**: `parent`, `children` (vector of IDs).
*   **Transform**: `scale`, `rotation`, `skew`, `translate`.
*   **Layout**: `style` (Taffy Style), `layout_rect` (Computed geometry).
*   **Compositing**: `opacity`, `blend_mode`, `mask_node`.
*   **State**: `visible`, `z_index`, `local_time`.

## Operations

*   **`add_node`**: Allocates a slot (reusing `free_indices` if possible) and returns a `NodeId`.
*   **`add_child`**: Establishes parent-child relationship. Detaches the child from previous parent if necessary.
*   **`destroy_node`**: Recursively destroys children, detaches from parent, and pushes index to `free_indices`.

## Usage in Director

The `Director` holds the `SceneGraph`. However, it doesn't just render "the graph". It renders specific **Scenes** defined in the `Timeline`. A `TimelineItem` points to a specific `NodeId` as its root. Only that root and its descendants are processed during a frame (unless transitions are active, in which case multiple roots are processed).
