//! Occlusion culling for the rendering pipeline.
//!
//! After the engine updates node transforms and bounds, this module traverses
//! the scene front-to-back and identifies layers that are fully hidden behind
//! opaque layers. Occluded nodes are collected into a set so the drawing code
//! can skip them.
//!
//! The computation accepts an arbitrary root node so that scenes drawn from
//! different starting points each get their own occlusion data.
//!
//! Clip-awareness: when a parent has `clip_children = true`, child bounds are
//! intersected with the parent clip. Children fully outside the clip are marked
//! occluded. Opaque layers only contribute the intersection of their bounds
//! with the active clip to the occlusion mask.

use std::collections::{HashMap, HashSet};

use indextree::Arena;
use skia_safe::Contains;

use crate::engine::node::SceneNode;
use crate::engine::storage::TreeStorageId;
use crate::engine::NodeRef;

/// Per-root occlusion data: the set of node ids that are fully occluded.
pub type OcclusionMap = HashMap<NodeRef, HashSet<NodeRef>>;

/// Compute the set of occluded nodes for a given root.
///
/// The algorithm flattens the subtree rooted at `root` into draw order
/// (back-to-front via pre-order traversal), then iterates **front-to-back**
/// (reversed). Fully-opaque rectangular layers contribute their global bounds
/// (clipped to any ancestor clip region) to an occlusion list. Any subsequent
/// (i.e. behind) node whose visible bounds are fully contained by an opaque
/// rect is marked occluded.
///
/// Nodes whose bounds fall entirely outside an ancestor's `clip_children`
/// region are also marked occluded since they produce no visible pixels.
#[profiling::function]
pub fn compute_occlusion(root: NodeRef, arena: &Arena<SceneNode>) -> HashSet<NodeRef> {
    let root_id: TreeStorageId = root.into();

    // Collect nodes in draw order (pre-order = back-to-front),
    // along with context opacity and the active clip rect.
    let mut draw_order: Vec<NodeOcclusionInfo> = Vec::new();
    collect_draw_order(root_id, arena, 1.0, None, &mut draw_order);

    // Front-to-back iteration (reverse of draw order).
    let mut opaque_rects: Vec<skia_safe::Rect> = Vec::new();
    let mut occluded = HashSet::new();

    for info in draw_order.iter().rev() {
        // Entire subtree is outside an ancestor clip — always skip
        if info.clipped_out {
            occluded.insert(info.node_ref);
            continue;
        }

        let visible_bounds = info.visible_bounds;

        // Zero-size nodes produce no pixels on their own but may have visible
        // children. Never mark them as occluded and never use them as occluders.
        if visible_bounds.width() <= 0.0 || visible_bounds.height() <= 0.0 {
            continue;
        }

        // Check if this node is fully covered by any existing opaque rect
        let is_occluded = opaque_rects
            .iter()
            .any(|rect| rect.contains(visible_bounds));

        if is_occluded {
            occluded.insert(info.node_ref);
            continue;
        }

        // If this layer is fully opaque, add its *clipped* bounds to the mask
        if info.is_opaque {
            opaque_rects.push(visible_bounds);
        }
    }

    occluded
}

/// Fold occlusion into per-node damage to produce an occlusion-aware
/// scene damage rect.
///
/// Walks the subtree rooted at `root` front-to-back with a running
/// `skia::Region` that accumulates the bounds of every valid opaque
/// occluder seen so far. For each node with damage, the running region
/// is subtracted from the damage rect before it is unioned into the
/// scene total. Nodes whose entire damage rect is covered by the
/// running region contribute nothing.
///
/// `per_node_damage` maps a node id to its global-coordinates damage
/// rect as already computed by `update_node_single`. Nodes not present
/// in the map are assumed to have no damage this frame.
///
/// Returns the bounding `Rect` of the resulting damage region, matching
/// the legacy `Engine::damage()` return type so callers don't have to
/// change. The full pixel set is visible to the caller only via this
/// bounding box — sufficient for the use case where downstream
/// consumers clip/redraw a rectangular area.
#[profiling::function]
pub fn compute_occlusion_aware_damage(
    root: NodeRef,
    arena: &Arena<SceneNode>,
    per_node_damage: &HashMap<NodeRef, skia_safe::Rect>,
) -> skia_safe::Rect {
    let root_id: TreeStorageId = root.into();

    // Collect nodes in draw order (pre-order = back-to-front).
    let mut draw_order: Vec<NodeOcclusionInfo> = Vec::new();
    collect_draw_order(root_id, arena, 1.0, None, &mut draw_order);

    // Opaque shapes accumulated in front-to-back order, tagged with the
    // node that contributed each one. The tag lets us filter descendants
    // out when subtracting occlusion from an ancestor's damage — a
    // descendant cannot legitimately occlude its ancestor, because the
    // ancestor's damage represents re-rendering its own subtree (through
    // any filter / effect the ancestor applies).
    let mut occluder_shapes: Vec<(NodeRef, skia_safe::IRect)> = Vec::new();
    let mut damage = skia_safe::Region::new();
    let mut visited: HashSet<NodeRef> = HashSet::new();

    // Front-to-back: the node at the end of draw_order is closest to the
    // viewer, so iterate in reverse.
    for info in draw_order.iter().rev() {
        visited.insert(info.node_ref);

        if info.clipped_out {
            continue;
        }

        // Add the node's damage (if any), minus the effective occluder
        // region for this specific node — which excludes any occluder
        // shape contributed by one of this node's descendants.
        if let Some(rect) = per_node_damage.get(&info.node_ref) {
            if !rect.is_empty() {
                let mut effective = skia_safe::Region::new();
                for (shape_node, shape_irect) in &occluder_shapes {
                    if is_descendant(*shape_node, info.node_ref, arena) {
                        continue;
                    }
                    effective.op_rect(*shape_irect, skia_safe::region::RegionOp::Union);
                }
                let mut d = skia_safe::Region::from_rect(rect_to_irect(*rect));
                d.op_region(&effective, skia_safe::region::RegionOp::Difference);
                damage.op_region(&d, skia_safe::region::RegionOp::Union);
            }
        }

        // After considering this node's damage, if the node itself is a
        // valid opaque occluder, record its visible bounds.
        if info.is_opaque {
            let vb = info.visible_bounds;
            if vb.width() > 0.0 && vb.height() > 0.0 {
                occluder_shapes.push((info.node_ref, rect_to_irect(vb)));
            }
        }
    }

    // `collect_draw_order` skips hidden / fully-transparent subtrees, so
    // nodes that just became hidden or faded to opacity 0 are not in
    // `draw_order`. Their previous bounds still need to be damaged so the
    // compositor clears the pixels they used to occupy. Union any such
    // unvisited dirty node's damage unconditionally (it has no valid
    // front-most ancestor to be clipped against).
    for (node_ref, rect) in per_node_damage.iter() {
        if visited.contains(node_ref) || rect.is_empty() {
            continue;
        }
        damage.op_rect(rect_to_irect(*rect), skia_safe::region::RegionOp::Union);
    }

    let bounds = damage.bounds();
    if bounds.is_empty() {
        skia_safe::Rect::default()
    } else {
        skia_safe::Rect::from_irect(bounds)
    }
}

/// Convert a floating-point rect to the integer rect that `skia::Region`
/// requires. Rounds outward so the integer rect covers every pixel the
/// float rect touches — never under-reports damage.
fn rect_to_irect(r: skia_safe::Rect) -> skia_safe::IRect {
    skia_safe::IRect::from_ltrb(
        r.left.floor() as i32,
        r.top.floor() as i32,
        r.right.ceil() as i32,
        r.bottom.ceil() as i32,
    )
}

/// Returns `true` if `maybe_descendant` is a strict descendant of
/// `ancestor` in the scene tree (i.e. lives in the subtree rooted at
/// `ancestor` but is not `ancestor` itself). Walks `maybe_descendant`'s
/// parent chain; bounded by tree depth, which is shallow in practice.
fn is_descendant(maybe_descendant: NodeRef, ancestor: NodeRef, arena: &Arena<SceneNode>) -> bool {
    if maybe_descendant == ancestor {
        return false;
    }
    let ancestor_id: TreeStorageId = ancestor.into();
    let mut current: TreeStorageId = maybe_descendant.into();
    while let Some(node) = arena.get(current) {
        match node.parent() {
            Some(pid) => {
                if pid == ancestor_id {
                    return true;
                }
                current = pid;
            }
            None => return false,
        }
    }
    false
}

struct NodeOcclusionInfo {
    node_ref: NodeRef,
    /// The node bounds intersected with the active clip (what is actually visible).
    visible_bounds: skia_safe::Rect,
    /// Whether the node is fully opaque and can act as an occluder.
    is_opaque: bool,
    /// Whether the node (including its children) is entirely outside its ancestor clip region.
    clipped_out: bool,
}

/// Pre-order traversal collecting occlusion info in draw order.
///
/// `clip_rect` is the intersection of all ancestor `clip_children` bounds
/// in global coordinates. `None` means no clipping is active.
fn collect_draw_order(
    node_id: TreeStorageId,
    arena: &Arena<SceneNode>,
    context_opacity: f32,
    clip_rect: Option<skia_safe::Rect>,
    out: &mut Vec<NodeOcclusionInfo>,
) {
    let Some(node) = arena.get(node_id) else {
        return;
    };
    if node.is_removed() {
        return;
    }
    let scene_node = node.get();

    // Hidden nodes and their entire subtree are invisible — skip them.
    if scene_node.hidden() {
        return;
    }

    let opacity = scene_node.render_layer.opacity * context_opacity;

    // Fully transparent subtrees produce no pixels — skip entirely.
    if opacity <= 0.0 {
        return;
    }

    let render_layer = &scene_node.render_layer;
    let bounds = render_layer.global_transformed_bounds;
    let bounds_with_children = render_layer.global_transformed_bounds_with_children;

    // Compute visible bounds: intersect with active clip.
    // Use bounds_with_children for the clip-out test so that a zero-size
    // container whose children have visible area is not incorrectly culled.
    let clipped_out = if let Some(clip) = clip_rect {
        let child_area = intersect_rects(bounds_with_children, clip);
        child_area.is_empty()
    } else {
        false
    };

    let visible_bounds = if let Some(clip) = clip_rect {
        intersect_rects(bounds, clip)
    } else {
        bounds
    };

    // A node is only an occluder when it AND all its ancestors are fully opaque.
    // Layers with opacity < 1.0, hidden parents, or semi-transparent subtrees
    // never contribute to the occlusion mask (but can still be occluded).
    let is_opaque = !clipped_out && render_layer.is_fully_opaque() && context_opacity >= 1.0;

    out.push(NodeOcclusionInfo {
        node_ref: NodeRef(node_id),
        visible_bounds,
        is_opaque,
        clipped_out,
    });

    // Compute the clip rect for children
    let child_clip = if render_layer.clip_children {
        // Intersect this node's global bounds with the existing clip
        let node_clip = render_layer.global_transformed_bounds;
        Some(match clip_rect {
            Some(parent_clip) => intersect_rects(parent_clip, node_clip),
            None => node_clip,
        })
    } else {
        clip_rect
    };

    for child_id in node_id.children(arena).collect::<Vec<_>>() {
        collect_draw_order(child_id, arena, opacity, child_clip, out);
    }
}

/// Intersect two rects, returning an empty rect if they don't overlap.
fn intersect_rects(a: skia_safe::Rect, b: skia_safe::Rect) -> skia_safe::Rect {
    let left = a.left.max(b.left);
    let top = a.top.max(b.top);
    let right = a.right.min(b.right);
    let bottom = a.bottom.min(b.bottom);
    if left < right && top < bottom {
        skia_safe::Rect::from_ltrb(left, top, right, bottom)
    } else {
        skia_safe::Rect::default()
    }
}
