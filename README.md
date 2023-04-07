## !! WIP in progress project !!
## Layers
Layers is a rendering engine for animated user interfaces. It uses a scene graph to render the nodes in retained mode, optmising the most common UI interpolations (opacity, 2d transformations, blending).
Nodes of the scene graph are graphical layers like text or simple shapes like rectangles but can also be external textures. Nodes have animatable properties that accepts changes and schedule them in the engine to be executed. Using this Command pattern, changes to the nodes have a consistent api between immediate changes and animated changes.
The rendering commands are optimised using displaylist. Node properties can be animated, hooks to different stages of the animation progress are exposed by the API.

## Rendering
At the moment the components are rendered using Skia on different backends. This enables 2 levels of caching: the draw calls can be cached using a DisplayList; second cache is by storing the rasterized image in a texture.

### Scene
The scene tree is stored in a memory arena using IndexTree, which allow fast read/write and thread safe parallel iterations.

## Colors
Colors are stored in OK lab color space to enable smooth and uniform looking transitions between them.
more about Oklab in Björn Ottosson (blog)[https://bottosson.github.io/posts/oklab/] 

# Build the library
At the moment the project requires to setup skia-safe configuration variables before building. See `Cargo.toml`. Once configured the library can be built using cargo.
The C header will be generated in the `target` folder.
The project requires nightly rust to use the negative_impl feature.
```
cargo build
```

## Build the rust example
The rust example is setup as a different workspace.
```
cargo build -p hello-rust
```
Likewise to run the rust example:
```
cargo run -p hello-rust
```

## Build the C example
The C example is setup with meson. It requires linux to be built and run because of the dependency with Wayland.
To build, it first needs to configure meson:
```
meson build/
```
and then run ninja:
```
ninja -c build
```
the executable will be in the `build/` folder.