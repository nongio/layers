# Layer Mirroring / Portals

Portals are a way to connect two layers together. When a layer has a portal into another layer, it can be rendered both in the context of the layer it is in, and in the context of the layer it is connected to. This allows for a variety of effects, such as rendering a layer in a different location, or rendering a layer with a different scale or rotation.

## Use Cases

Portals are useful for a variety of effects, such as:
- Rendering a layer in multiple different location
- Rendering a layer with at different scales or rotations
- Create a "picture-in-picture" effect
- Mini previews of a complex layer hierarchy

## Desktop specific use cases

Portals are particularly useful for creating complex user interfaces for a desktop environment. For example, a window manager might use portals to render windows in different locations, or to create a "picture-in-picture" effect.

For workspace managers, it would be useful to render a preview of a workspace in a different location, or to render a workspace with a different scale or rotation.

For Window Managers, it would be useful to render a window in a different location, or to render a window with a different scale or rotation, like render a window in its position as well as in a taskbar or in a window selection screen.

## Features
- Link a source layer to a target layer
- Render the source layer in the context of the target layer
- Render the source layer in a different location, scale, or rotation
- Render the source and its children in the context of the target layer

### Should the linking be through a texture or a re-rendering of the source layer?

For an optimized implementation, the linking should be through a texture. This way, the source layer can be rendered once and then reused in multiple locations. 

However, for some use cases, it might be useful to re-render the source layer in the context of the target layer. For example, if the source layer is animated, it might be useful to re-render it in the context of the target layer to ensure that the animation is synchronized.

What properties of the layer should be inherited by the portal?
Everything that affects the painting of the layer should be inherited by the portal. This includes:

- content
- background
- border
- border-radius
- shadow
