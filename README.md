## Layers
Layers is a renderering engine for animated user interfaces. It uses a scene graph to render the nodes in retained mode, optmising the most common UI interpolations (opacity, 2d transformations, blending).
Nodes of the scene graph are graphical layers like text or simple shapes like rectangles but can also be external textures. Nodes have animatable properties that accepts changes and schedule them in the engine to be executed. Using this Command pattern, changes to the nodes have a consistent api between immediate changes and animated changes.
The rendering commands are optimised using display list.Node properties can be animated, hooks to different stages of the animation progress are exposed by the API.

## Rendering
At the moment the components are rendered using Skia on different backends. This enables 2 levels of caching: the draw calls can be cached using a DisplayList; second cache is by storing the rasterized image in a texture.

### Scene
The scene tree is stored in a memory arena using IndexTree, which allow fast read/write and thread safe parallel iterations.

## Colors
Colors are stored in OK lab color space to enable smooth and uniform looking transitions between them.
more about Oklab in Björn Ottosson (blog)[https://bottosson.github.io/posts/oklab/] 