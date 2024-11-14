use indextree::Arena;
use skia_safe::M44;
use taffy::prelude::Layout;

use super::SceneNode;

/// A trait for Nodes to expose their cache management
pub trait DrawCacheManagement {
    fn repaint_if_needed(&self, arena: &Arena<SceneNode>) -> skia_safe::Rect;
    fn needs_repaint(&self) -> bool;
    fn needs_layout(&self) -> bool;
    fn set_need_repaint(&self, value: bool);
    fn layout_if_needed(&self, layout: &Layout, matrix: Option<&M44>, context_opacity: f32, arena: &Arena<SceneNode>)
        -> bool;
    fn set_need_layout(&self, value: bool);
    fn is_content_cached(&self) -> bool;
}
