# Architecture

## Main concepts
- The scene tree is stored in a memory arena using IndexTree, which allow fast read/write and thread safe parallel iterations
- The basic node of the scene tree has a dyn Renderable model, the rendering commands are cached using skia display list
- Animatable properties of the nodes, animations and transactions are stored in a HashMap like storage, that allows for id based read/write as well thread safe parallel iterations
- Changes to the nodes are applied using the command pattern, which enable a consistent api between immediate changes and animated changes

### Scene
The scene is:
- a tree of renderable nodes,
- a list of changes to be applied to properties
- a list of animations associate to the changes

```
trait Renderable: Drawable + DrawCache ...;
```

### Command Pattern

A Renderable produces Change messages that are processed by the Engine.
The changes includes an optional Transition description used by the engine to produce animations and a set of bit flags to update on scene node.
On every update the Engine step forward the animations and applies the changes to the Renderables. Based on the flags the engines marks the nodes as in need of rendering or layout.
*ModelChange*: a change over a property of a node model that could trigger a repaint or layout
*ValueChange*: a change over a property, described by 2 values and a description of the transition between them
*Transaction*: ModelChange, AnimationId, NodeId


### Hierarchy

- scene graph made of nodes
- a scene is a treestorage of nodes
- a draw cache is a flat storage of display lists
- a node can be rendered to a displaylist


/// The Scene is the data structure to represent the tree of nodes to be rendered.
/// It enables the traversing of the nodes top-down and bottom-up

/// The scene is drawn in 3 steps:
/// The *layout* step calculates the dimensions and position of a node and generates a transformation Matrix
/// The *draw* step generates a displaylist
/// The *render* step uses the displaylist to generate a texture of the node
/// The *compose* step generates the final image using the textures

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