# Engine Architecture

This framework 
### Engine
- a scene
- a list of changes to be executed on properties
- a list of animations associate to the changes

### Scene
The scene is a tree of renderable nodes (implementing the Renderable trait).
The tree is stored in a memory arena using IndexTree, which allow fast read/write and thread safe parallel iterations.

### Renderable trait
...

### Nodes
Nodes have animatable properties that accepts changes and schedule them in the engine to be executed. Using this Command pattern, changes to the nodes have a consistent api between immediate changes and animated changes.
The rendering commands are optimised using skia display list.



```
trait Renderable: Drawable + DrawCache ...;
```

### Command Pattern

A Renderable when receiving a change to a property, sends a change message to the Engine. Changes are stored in a HashMap like storage, that allows for id based read/write as well thread safe parallel iterations.
The changes can includes an optional Transition description used by the engine to produce animations. Animations are separated from the changes to allow grouping of multiple changes in sync.
A Change when executed returns a set of bit flags to mark as dirty the affected Renderable.
On every update the Engine step forward the animations and applies the changes to the Renderables. Based on the flags the engines marks the nodes as in need of rendering or layout.
Data model for Changes:
*ModelChange*: a change over a property of a node model that could trigger a repaint or layout
*ValueChange*: a change over a property, described by 2 values and am optional description of the transition between them
*Transaction*: ModelChange, AnimationId, NodeId


### Scene hierarchy

- scene graph made of nodes
- a scene is a treestorage of nodes
- a draw cache is a flat storage of display lists
- a node can be rendered to a displaylist

### WIP Scene Drawing
The Scene is the data structure to represent the tree of nodes to be rendered.
It enables the traversing of the nodes top-down and bottom-up

The scene is drawn in 4 steps:
- The *layout* step calculates the dimensions and position of a node and generates a transformation Matrix
- The *draw* step generates a displaylist
- The *render* step uses the displaylist to generate a texture of the node
- The *compose* step generates the final image using the textures

- Scene
    - root: Node
    - node: Treestorage<Node>
        - dyn Renderable
        - dyn Layout

/// The RenderCache is a storage with fast read acesss
/// for display lists from the nodes

- RenderCache
    - displayList[]
    - image[]


- AnimationEngine
    - transactions[]
    - animation[]
    - scene