Layers Engine is a renderering engine for animated user interfaces. It uses a scene graph to render the layer nodes in retained mode, optmising the most common UI interpolations (opacity, 2d transformations, blending).
Nodes of the scene graph are graphical layers like text or simple shapes like rectangles but also external textures. Node properties can be animated, hooks to different stages of the animation progress are exposed by the API.

## Rendering
At the moment the components are rendered using Skia on different backends. This enables 2 levels of caching: the draw calls can be cached using a DisplayList; second cache is by storing the rasterized image in a texture.