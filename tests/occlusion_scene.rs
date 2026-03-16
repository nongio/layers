//! Integration test for occlusion culling against a live/saved scene.
//!
//! Fetches a scene JSON from a local server and tests whether a target node
//! would be skipped (occluded) or drawn for a given root node.
//!
//! # Usage
//!
//! ```sh
//! SCENE_URL=http://localhost:8000/scene/89 \
//!   ROOT_NODE=89 \
//!   TARGET_NODE=130 \
//!   cargo test --test occlusion_scene -- --nocapture
//! ```
//!
//! Or from a saved JSON file:
//!
//! ```sh
//! SCENE_FILE=scene_89.json \
//!   ROOT_NODE=89 \
//!   TARGET_NODE=130 \
//!   cargo test --test occlusion_scene -- --nocapture
//! ```
//!
//! The JSON format matches the output of the layers inspector `/scene/{id}` endpoint:
//!
//! ```json
//! [root_id, {
//!   "node_id": [node_id, { ...render_layer_props... }, [children_ids], { "index1": N }],
//!   ...
//! }]
//! ```

use std::collections::HashMap;

use indextree::Arena;
use layers::engine::occlusion::compute_occlusion;
use layers::engine::{NodeRef, SceneNode};
use layers::prelude::*;

/// Parsed representation of a scene node from JSON.
struct ParsedNode {
    id: u64,
    hidden: bool,
    opacity: f32,
    blend_mode: BlendMode,
    background_color_alpha: f32,
    border_corner_radius: BorderRadius,
    shape_is_roundrect: bool,
    clip_children: bool,
    transformed_bounds: skia_safe::Rect,
    children: Vec<u64>,
    key: String,
}

fn parse_scene_json(json_str: &str) -> (u64, Vec<ParsedNode>) {
    let value: serde_json::Value = serde_json::from_str(json_str).expect("invalid JSON");
    let arr = value
        .as_array()
        .expect("scene JSON must be an array [root_id, nodes_map]");

    let root_id = arr[0].as_u64().expect("root_id must be a number");
    let nodes_map = arr[1].as_object().expect("nodes_map must be an object");

    let mut parsed_nodes = Vec::new();

    for (_key, node_value) in nodes_map {
        let node_arr = node_value
            .as_array()
            .expect("each node must be [id, props, children, index]");
        let node_id = node_arr[0].as_u64().expect("node id must be a number");
        let props = &node_arr[1];
        let children_arr = node_arr[2].as_array().expect("children must be an array");
        let children: Vec<u64> = children_arr.iter().map(|c| c.as_u64().unwrap()).collect();

        let hidden = props["hidden"].as_bool().unwrap_or(false);
        let opacity = props["opacity"].as_f64().unwrap_or(1.0) as f32;

        let blend_mode = match props["blend_mode"].as_str().unwrap_or("Normal") {
            "BackgroundBlur" => BlendMode::BackgroundBlur,
            _ => BlendMode::Normal,
        };

        // Extract alpha from background_color (supports Solid only for now)
        let background_color_alpha = props
            .get("background_color")
            .and_then(|bg| bg.get("Solid"))
            .and_then(|solid| solid.get("color"))
            .and_then(|color| color.get("alpha"))
            .and_then(|a| a.as_f64())
            .unwrap_or(0.0) as f32;

        let bcr = &props["border_corner_radius"];
        let border_corner_radius = BorderRadius {
            top_left: bcr["top_left"].as_f64().unwrap_or(0.0) as f32,
            top_right: bcr["top_right"].as_f64().unwrap_or(0.0) as f32,
            bottom_right: bcr["bottom_right"].as_f64().unwrap_or(0.0) as f32,
            bottom_left: bcr["bottom_left"].as_f64().unwrap_or(0.0) as f32,
        };

        let shape_is_roundrect = match &props["shape"] {
            serde_json::Value::String(s) => s == "RoundRect",
            _ => false, // Path or other custom shapes
        };

        // Use "transformed_bounds" as the global bounds
        let tb = &props["transformed_bounds"];
        let x = tb["x"].as_f64().unwrap_or(0.0) as f32;
        let y = tb["y"].as_f64().unwrap_or(0.0) as f32;
        let w = tb["width"].as_f64().unwrap_or(0.0) as f32;
        let h = tb["height"].as_f64().unwrap_or(0.0) as f32;
        let transformed_bounds = skia_safe::Rect::from_xywh(x, y, w, h);

        let clip_children = false; // Not present in JSON — inspect manually if needed

        let key = props["key"].as_str().unwrap_or("").to_string();

        parsed_nodes.push(ParsedNode {
            id: node_id,
            hidden,
            opacity,
            blend_mode,
            background_color_alpha,
            border_corner_radius,
            shape_is_roundrect,
            clip_children,
            transformed_bounds,
            children,
            key,
        });
    }

    (root_id, parsed_nodes)
}

/// Build an indextree arena from parsed nodes, returning the root NodeRef
/// and a mapping from scene node IDs to arena NodeIds.
fn build_arena(
    root_id: u64,
    parsed_nodes: &[ParsedNode],
) -> (Arena<SceneNode>, NodeRef, HashMap<u64, indextree::NodeId>) {
    let mut arena = Arena::<SceneNode>::new();
    let mut id_map: HashMap<u64, indextree::NodeId> = HashMap::new();

    // First pass: create all arena nodes
    for pn in parsed_nodes {
        let mut scene_node = SceneNode::default();
        scene_node.set_hidden(pn.hidden);

        let rl = scene_node.render_layer_mut();
        rl.key = pn.key.clone();
        rl.opacity = pn.opacity;
        rl.premultiplied_opacity = pn.opacity; // will be fixed in second pass
        rl.blend_mode = pn.blend_mode;
        rl.global_transformed_bounds = pn.transformed_bounds;
        rl.border_corner_radius = pn.border_corner_radius;
        rl.clip_children = pn.clip_children;

        if pn.shape_is_roundrect {
            rl.shape = Shape::RoundRect;
        }
        // else it keeps default (which should be RoundRect from Default impl)

        // Set background_color alpha
        rl.background_color = PaintColor::Solid {
            color: Color {
                l: 0.0,
                a: 0.0,
                b: 0.0,
                alpha: pn.background_color_alpha,
            },
        };

        let node_id = arena.new_node(scene_node);
        id_map.insert(pn.id, node_id);
    }

    // Second pass: build parent-child hierarchy
    for pn in parsed_nodes {
        if let Some(&parent_arena_id) = id_map.get(&pn.id) {
            for &child_scene_id in &pn.children {
                if let Some(&child_arena_id) = id_map.get(&child_scene_id) {
                    parent_arena_id.append(child_arena_id, &mut arena);
                }
            }
        }
    }

    // Third pass: compute premultiplied_opacity by walking from root
    let root_arena_id = *id_map
        .get(&root_id)
        .expect("root_id not found in parsed nodes");
    propagate_opacity(root_arena_id, 1.0, &mut arena);

    let root_ref = NodeRef(root_arena_id);
    (arena, root_ref, id_map)
}

fn propagate_opacity(
    node_id: indextree::NodeId,
    parent_opacity: f32,
    arena: &mut Arena<SceneNode>,
) {
    let node_opacity = arena.get(node_id).unwrap().get().render_layer().opacity;
    let premultiplied = node_opacity * parent_opacity;
    arena
        .get_mut(node_id)
        .unwrap()
        .get_mut()
        .render_layer_mut()
        .premultiplied_opacity = premultiplied;

    let children: Vec<_> = node_id.children(arena).collect();
    for child_id in children {
        propagate_opacity(child_id, premultiplied, arena);
    }
}

fn fetch_scene_json() -> String {
    if let Ok(file_path) = std::env::var("SCENE_FILE") {
        return std::fs::read_to_string(&file_path)
            .unwrap_or_else(|e| panic!("failed to read {}: {}", file_path, e));
    }

    if let Ok(url) = std::env::var("SCENE_URL") {
        let output = std::process::Command::new("curl")
            .args(["-s", "--fail", &url])
            .output()
            .expect("failed to execute curl");
        if !output.status.success() {
            panic!(
                "curl failed with status {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            );
        }
        return String::from_utf8(output.stdout).expect("invalid UTF-8 from curl");
    }

    panic!("set either SCENE_URL or SCENE_FILE environment variable");
}

#[test]
fn test_occlusion_for_scene() {
    let root_id: u64 = std::env::var("ROOT_NODE")
        .expect("set ROOT_NODE env var")
        .parse()
        .expect("ROOT_NODE must be a number");
    let target_id: u64 = std::env::var("TARGET_NODE")
        .expect("set TARGET_NODE env var")
        .parse()
        .expect("TARGET_NODE must be a number");

    let json_str = fetch_scene_json();
    let (scene_root_id, parsed_nodes) = parse_scene_json(&json_str);

    eprintln!("Scene root: {}", scene_root_id);
    eprintln!("Nodes parsed: {}", parsed_nodes.len());
    eprintln!("Using root: {}, target: {}", root_id, target_id);

    let (arena, root_ref, id_map) = build_arena(root_id, &parsed_nodes);

    let occluded = compute_occlusion(root_ref, &arena);

    let target_arena_id = id_map
        .get(&target_id)
        .unwrap_or_else(|| panic!("target node {} not found in scene", target_id));
    let target_ref = NodeRef(*target_arena_id);

    let is_occluded = occluded.contains(&target_ref);

    // Print all occluded nodes for debugging
    eprintln!("\n--- Occlusion Results ---");
    eprintln!("Total occluded nodes: {}", occluded.len());

    // Reverse-map arena IDs to scene IDs for readable output
    let arena_to_scene: HashMap<indextree::NodeId, u64> =
        id_map.iter().map(|(&sid, &aid)| (aid, sid)).collect();

    for occ_ref in &occluded {
        let arena_id: indextree::NodeId = (*occ_ref).into();
        let scene_id = arena_to_scene.get(&arena_id).unwrap_or(&0);
        let key = arena
            .get(arena_id)
            .map(|n| n.get().render_layer().key.clone())
            .unwrap_or_default();
        eprintln!("  occluded: node {} (key: {})", scene_id, key);
    }

    // Print target node info
    let target_node = arena.get(*target_arena_id).unwrap().get();
    let target_rl = target_node.render_layer();
    eprintln!("\n--- Target Node {} ---", target_id);
    eprintln!("  key: {}", target_rl.key);
    eprintln!("  hidden: {}", target_node.hidden());
    eprintln!("  opacity: {}", target_rl.opacity);
    eprintln!(
        "  premultiplied_opacity: {}",
        target_rl.premultiplied_opacity
    );
    eprintln!("  bounds: {:?}", target_rl.global_transformed_bounds);
    eprintln!("  is_fully_opaque: {}", target_rl.is_fully_opaque());
    eprintln!(
        "\n  RESULT: node {} will be {}",
        target_id,
        if is_occluded { "SKIPPED" } else { "DRAWN" }
    );
}
