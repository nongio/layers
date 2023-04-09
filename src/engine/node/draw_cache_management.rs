use taffy::prelude::Layout;

/// A trait for Nodes to expose their cache management
pub trait DrawCacheManagement {
    fn repaint_if_needed(&self);
    fn set_need_repaint(&self, value: bool);
    fn layout_if_needed(&self, layout: &Layout);
    fn set_need_layout(&self, value: bool);
    fn set_need_raster(&self, value: bool);
    fn need_raster(&self) -> bool;
}
