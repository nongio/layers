# Next

All the upcoming changes for the next release are listed here.
(See CHANGELOG.md for released versions)

## [0.6.1] usable
- [ ] [fix] damage tracking for mirrored layers
- [ ] review update logic
- [ ] [fix] examples

## [0.7.0] solid-api
- [ ] review C api
- [ ] review Layer / SceneNode data struct


## [0.x.0] butter-smooth
- [ ] rename to Butter engine


## Refactor

### Engine
The Engine struct is the central component of the rendering engine. It manages the overall state of the application, including the scene, layout tree, animations, and transactions. The Engine is responsible for updating the scene, handling animations, executing scheduled changes, managing the layout and pre-rendering processes. It provides methods for adding, removing, and manipulating layers, as well as updating the scene and triggering callbacks for transitions.

### Scene
The Scene struct represents the tree of nodes to be rendered. It manages the hierarchical structure of the scene graph, allowing for efficient traversal and manipulation of nodes. The Scene provides methods for adding, removing, and accessing nodes, as well as updating the size of the scene. It uses an arena-based storage system to manage the nodes, enabling fast read/write operations and thread-safe parallel iterations.

### SceneNode: fast access (expensive cloning)
The SceneNode struct represents a node in the scene graph tree. 
The SceneNode is responsible for handling the pre-rendering layer, caching, and flags that indicate the need for layout or repaint. It provides methods for managing rendering, layout, and transformations, as well as handling interactions such as pointer events and hover states.

### Layer: animatable properties, (fast cloning)
The Layer struct represents a 2D object in the engine with various properties and behaviors. It manages high-level properties such as visibility, pointer events, and layout properties. The Layer delegates rendering and layout responsibilities to the Engine and provides methods for setting and getting properties, managing state, and handling effects. It acts as a container for graphical content and can be nested to create complex 2D objects.

### Layer: ModelLayer
The ModelLayer struct represents the internal properties storage of a Layer. It contains Attributes such as position, scale, rotation, size, background color, border properties, shadow properties, and opacity. 
The ModelLayer is responsible for managing the state of these properties and providing methods for updating and retrieving their values.

### Animations
Animations are stored into an Arena, and the Engine keeps track of the active animations. The Engine updates the animations on every frame, interpolating the values and triggering callbacks when the animations are completed. The Engine provides methods for adding, removing, and updating animations, as well as triggering callbacks for transitions. One or more transactions can be associated with an animation, and the Engine will update the transactions when the animation is updated.

### Transactions
A transaction is a change of ModelLayer Attribute, it is a change that could be animated. The Engine keeps track of the transactions in a Arena, and it updates the ModelLayer Attributes on every frame. The Engine provides methods for adding, removing, and updating transactions.

## Benchmarks
On every MR we should run benchmarks to ensure that the performance is not regressing.
The results are published on the github pages.
https://github.com/benchmark-action/github-action-benchmark/blob/master/examples/rust/README.md

Ideally every stage of the engine update is benchmarked to ensure that the performance is optimal.

- update_animations
- update_transactions
- update_node
- update_layout
- trigger_callbacks
- transaction_callbacks
- cleanup_animations
- cleanup_transactions
- cleanup_nodes

- Engine::pointer_move