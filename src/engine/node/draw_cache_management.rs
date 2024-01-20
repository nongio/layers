use taffy::prelude::Layout;

/// A trait for Nodes to expose their cache management
pub trait DrawCacheManagement {
    fn repaint_if_needed(&self) -> bool;
    fn needs_repaint(&self) -> bool;
    fn needs_layout(&self) -> bool;
    fn set_need_repaint(&self, value: bool);
    fn layout_if_needed(&self, layout: &Layout) -> bool;
    fn set_need_layout(&self, value: bool);
}
