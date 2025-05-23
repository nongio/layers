use indextree::NodeId;
use taffy::TaffyTree;

use crate::{
    engine::{storage::TreeStorageData, *},
    layers::layer::render_layer::RenderLayer,
};

// this function recursively update the node picture and its children
// and returns the area of pixels that are changed compared to the previeous frame
#[allow(unused_assignments, unused_mut)]
#[profiling::function]
pub(crate) fn update_node_recursive(
    engine: &Engine,
    arena: &mut TreeStorageData<SceneNode>,
    layout_tree: &TaffyTree,
    node_id: NodeId,
    parent: Option<&RenderLayer>,
    parent_changed: bool,
) -> (bool, skia::Rect) {
    // update the layout of the node
    let children: Vec<_> = node_id.children(arena).collect();

    let mut damaged = false;
    let mut layout_changed = false;
    let mut pos_changed = false;
    let mut node_damage = skia::Rect::default();

    let mut transformed_bounds = skia::Rect::default();
    let mut new_transformed_bounds = skia::Rect::default();
    let mut render_layer = {
        let node = arena.get_mut(node_id);

        if node.is_none() {
            return (false, skia::Rect::default());
        }

        let node = node.unwrap().get_mut();
        let layer = engine.get_layer(&NodeRef(node_id)).unwrap();
        let node_layout = layout_tree.layout(layer.layout_id).unwrap();

        let mut opacity;
        (transformed_bounds, opacity) = {
            let render_layer = &node.render_layer;
            (
                render_layer.global_transformed_bounds,
                render_layer.premultiplied_opacity,
            )
        };

        let cumulative_transform = parent.map(|p| &p.transform);
        let context_opacity = parent.map(|p| p.premultiplied_opacity).unwrap_or(1.0);

        let _changed_render_layer = node.update_render_layer_if_needed(
            node_layout,
            layer.model.clone(),
            cumulative_transform,
            context_opacity,
        );

        // update the picture of the node
        node_damage = node.repaint_if_needed();

        let render_layer = node.render_layer();

        new_transformed_bounds = render_layer.global_transformed_bounds;

        let repainted = !node_damage.is_empty();

        layout_changed = transformed_bounds.width() != new_transformed_bounds.width()
            || transformed_bounds.height() != new_transformed_bounds.height();

        pos_changed = transformed_bounds.x() != new_transformed_bounds.x()
            || transformed_bounds.y() != new_transformed_bounds.y();

        let opacity_changed = opacity != render_layer.premultiplied_opacity;

        if (pos_changed && !transformed_bounds.is_empty())
            && render_layer.premultiplied_opacity > 0.0
            || opacity_changed
        {
            node_damage.join(node.repaint_damage);
            node_damage.join(new_transformed_bounds);

            node.repaint_damage = new_transformed_bounds;
        }
        damaged = layout_changed || repainted || parent_changed;

        let render_layer = node.render_layer();

        render_layer.clone()
    };

    let mut rl = render_layer.clone();
    let (damaged, mut node_damage) = children
        .iter()
        .map(|child| {
            // println!("**** map ({}) ", child);
            let (child_damaged, child_damage) = update_node_recursive(
                engine,
                arena,
                layout_tree,
                *child,
                Some(&render_layer.clone()),
                parent_changed,
            );
            let rn = arena.get(*child).unwrap().get().render_layer.clone();
            // damaged = damaged || child_repainted || child_relayout;
            (child_damaged, child_damage, child, rn)
        })
        .fold(
            (damaged, node_damage),
            |(damaged, node_damage), (child_damaged, child_damage, _nodeid, r)| {
                // update the bounds of the node to include the children

                rl.global_transformed_bounds_with_children
                    .join(r.global_transformed_bounds_with_children);

                let (_, _) = r.local_transform.to_m33().map_rect(r.bounds_with_children);
                let child_bounds = r.bounds_with_children;

                rl.bounds_with_children.join(child_bounds);

                let node_damage = skia::Rect::join2(node_damage, child_damage);
                (damaged || child_damaged, node_damage)
            },
        );
    {
        let node = arena.get_mut(node_id).unwrap().get_mut();
        node.render_layer = rl;
        // if the node has some drawing in it, and has changed size or position
        // we need to repaint
        let last_repaint_damage = node.repaint_damage;
        if !last_repaint_damage.is_empty() && (layout_changed || pos_changed || parent_changed) {
            transformed_bounds.join(new_transformed_bounds);
            node_damage.join(transformed_bounds);
        }
    }
    if damaged {
        // if !node_damage.is_empty() {
        if let Some(node) = arena.get_mut(node_id) {
            let node = node.get_mut();
            node.increase_frame();
        }
    }
    (damaged, node_damage)
}

// this function recursively update the node picture and its children
// and returns the area of pixels that are changed compared to the previeous frame
#[allow(unused_assignments, unused_mut)]
#[profiling::function]
pub(crate) fn update_node_single(
    engine: &Engine,
    layout_tree: &TaffyTree,
    node_id: NodeId,
    parent: Option<&RenderLayer>,
    parent_changed: bool,
) -> bool {
    let mut damaged = false;
    let mut layout_changed = false;
    let mut pos_changed = false;
    let mut node_damage = skia::Rect::default();

    let mut transformed_bounds = skia::Rect::default();
    let mut new_transformed_bounds = skia::Rect::default();
    // let mut render_layer = {
    let layer = engine.get_layer(&NodeRef(node_id)).unwrap();
    let node_layout = layout_tree.layout(layer.layout_id).unwrap();
    damaged = engine.scene.with_arena_mut(|arena| {
        let node = arena.get_mut(node_id);
        // if node is not found, early return
        if node.is_none() {
            return false;
        }
        let mut node = node.unwrap();
        let node = node.get_mut();
        let mut opacity;
        (transformed_bounds, opacity) = {
            let render_layer = &node.render_layer;
            (
                render_layer.global_transformed_bounds,
                render_layer.premultiplied_opacity,
            )
        };

        let cumulative_transform = parent.map(|p| &p.transform);
        let context_opacity = parent.map(|p| p.premultiplied_opacity).unwrap_or(1.0);

        let _changed_render_layer = node.update_render_layer_if_needed(
            node_layout,
            layer.model.clone(),
            cumulative_transform,
            context_opacity,
        );

        // update the picture of the node
        node_damage = node.repaint_if_needed();

        let render_layer = node.render_layer();

        new_transformed_bounds = render_layer.global_transformed_bounds;

        let repainted = !node_damage.is_empty();

        layout_changed = transformed_bounds.width() != new_transformed_bounds.width()
            || transformed_bounds.height() != new_transformed_bounds.height();

        pos_changed = transformed_bounds.x() != new_transformed_bounds.x()
            || transformed_bounds.y() != new_transformed_bounds.y();

        let opacity_changed = opacity != render_layer.premultiplied_opacity;

        if (pos_changed && !transformed_bounds.is_empty())
            && render_layer.premultiplied_opacity > 0.0
            || opacity_changed
        {
            node_damage.join(node.repaint_damage);
            node_damage.join(new_transformed_bounds);

            node.repaint_damage = new_transformed_bounds;
        }
        damaged = layout_changed || repainted || parent_changed;

        damaged
    });

    return damaged;
}
