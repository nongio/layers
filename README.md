
<p align="center">
  <img src="https://github.com/nongio/layers/blob/main/assets/lay-rs.jpg?raw=true" alt="Layers Engine Logo" width="384" height="192">
</p>

## lay-rs engine




lay-rs is a rendering engine for animated user interfaces, mainly designed in support of the ScreenComposer [project](https://github.com/nongio/screencomposer).

It uses a scene graph to render the nodes in retained mode, optmising the most common UI interpolations (opacity, 2d transformations, blending).
Nodes of the scene graph are graphical layers like text or simple shapes like rectangles but can also be external textures.

- Nodes have animatable properties that accepts changes and schedule them in the engine to be executed. 
- Layers use a Command pattern to receive changes with a consistent api between immediate changes and animated changes.
- The rendering commands are optimised using displaylist.
- Node properties can be animated, hooks to different stages of the animation progress are exposed by the API.

## Read the docs
The api is getting documented, be aware that is also still in evolution.
[documentation](https://nongio.github.io/layers/layers/)


## Rendering
At the moment the components are rendered using Skia on different backends. This enables some drawing optimisation: the draw calls can be cached using a DisplayList. A Skia PictureRecorder is generate on a separate thread and then replayed on the main thread when needed.

### Scene
The scene tree is stored in a memory arena using IndexTree, which allow fast read/write and thread safe parallel iterations.

### Colors
Colors are stored in OK lab color space to enable smooth and uniform looking transitions between them.
more about Oklab in Björn Ottosson [blog](https://bottosson.github.io/posts/oklab/)

### Layout
The layout is done using Taffy, every layer supports Flex, Block and Grid layout.

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

## Usage

## Usage: Setup a basic scene with a root layer
```rust
use lay_rs::prelude::*;
let engine = LayersEngine::new(800.0, 600.0);
let layer = engine.new_layer();
let engine = LayersEngine::new(1024.0, 768.0);
let root_layer = engine.new_layer();
root_layer.set_position(Point { x: 0.0, y: 0.0 });
root_layer.set_background_color(
    PaintColor::Solid {
        color: Color::new_rgba255(180, 180, 180, 255),
    }
);
root_layer.set_border_corner_radius(10.0, None);
root_layer.set_layout_style(taffy::Style {
    position: taffy::Position::Absolute,
    display: taffy::Display::Flex,
    flex_direction: taffy::FlexDirection::Column,
    justify_content: Some(taffy::JustifyContent::Center),
    align_items: Some(taffy::AlignItems::Center),
    ..Default::default()
});
engine.scene_add_layer(root_layer.clone());
```

## Usage: Tick the engine to update layout and animations
```rust
use lay_rs::prelude::*;
let engine = LayersEngine::new(800.0, 600.0);
// setup the scene...
engine.update(0.016);
```

## Usage: View builder
```rust
use lay_rs::prelude::*;

// view rendering function
pub fn render_main_view(state: &bool, view: &View<bool>) -> LayerTree {
    let mut position = 0.0;
    if *state {
        position = 100.0;
    }
    let view = view.clone();

    LayerTreeBuilder::default()
        .key("main_view")
        .position(
            Point {
                x: position,
                y: position,
            }
        )
        .size(
            Size {
                width: taffy::Dimension::Length(50.0),
                height: taffy::Dimension::Length(50.0),
            }
        )
        .build()
        .unwrap()
}

let engine = LayersEngine::new(1000.0, 1000.0);
// create a new layer and add it to the scene
let layer = engine.new_layer();
engine.scene_add_layer(layer.clone());

// define a new view with a boolean state
let initial = false;
let mut view = View::new("test_view", initial, render_one_child_view);
// assign the layer to the view, the rendered tree will be added to the layer
view.mount_layer(layer);
// layout the scene and update animations
engine.update(0.016);
// render the scene to terminal
debug_scene(engine.scene(), engine.scene_root().unwrap());
```

## More examples
Please check the example folder:
https://github.com/nongio/layers/tree/main/examples
