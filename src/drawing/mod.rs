//! Drawing related code for Layer, Scene, and Text.
//! The functions in this module are used to draw into a Skia canvas
//! independent from the backend

pub(crate) mod layer;
pub(crate) mod scene;
pub use layer::draw_layer;
pub use scene::{
    draw_scene, node_tree_list, node_tree_list_visible, print_scene, render_node_tree,
};
