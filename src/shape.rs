use serde::Serialize;
use skia_safe::{Path, RRect, Rect};

use crate::types::BorderRadius;

/// Shape defines the visual boundary of a layer.
///
/// Shapes are applied post-layout and affect rendering, hit-testing, and clipping.
/// The default shape is `RoundRect`, which uses the layer's `border_corner_radius` attribute.
#[derive(Debug, Clone, Default, Serialize)]
pub enum Shape {
    /// Rounded rectangle shape (default).
    /// Uses the separate `border_corner_radius` attribute which can be animated.
    #[default]
    RoundRect,

    /// Arbitrary Skia path.
    /// The path should be defined in local coordinates relative to the layer's bounds.
    ///
    /// Thread-safety is handled automatically via serialization in PathData.
    Path(PathData),
}

/// Thread-safe wrapper for Skia Path data
#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)] // Will be used when users create custom path shapes
pub struct PathData {
    // Store path as serialized data for thread safety
    data: Vec<u8>,
}

impl PathData {
    /// Create PathData from a Skia Path
    pub fn from_path(path: &Path) -> Self {
        // Serialize path to bytes for thread-safe storage
        let serialized = path.serialize();
        let data = serialized.as_bytes().to_vec();
        Self { data }
    }

    /// Regenerate Skia Path from stored data
    pub fn to_path(&self) -> Path {
        if self.data.is_empty() {
            Path::new()
        } else {
            // Convert Vec<u8> to skia_safe::Data first
            let skia_data = skia_safe::Data::new_copy(&self.data);
            if let Some(path) = Path::deserialize(&skia_data) {
                path
            } else {
                Path::new()
            }
        }
    }
}

impl Shape {
    /// Generate a Skia path from this shape definition.
    ///
    /// # Arguments
    /// * `bounds` - The layer's layout bounds
    /// * `border_corner_radius` - Corner radius (only used for `RoundRect`)
    ///
    /// # Returns
    /// A Skia `Path` representing the shape in local coordinates.
    pub fn to_path(&self, bounds: Rect, border_corner_radius: &BorderRadius) -> Path {
        match self {
            Shape::RoundRect => {
                let rrect = RRect::new_rect_radii(bounds, &(*border_corner_radius).into());
                Path::rrect(rrect, None)
            }
            Shape::Path(path_data) => path_data.to_path(),
        }
    }

    /// Get the bounds of the shape.
    /// For hit-testing without regenerating the full path.
    pub fn bounds(&self, layer_bounds: Rect, border_corner_radius: &BorderRadius) -> Rect {
        match self {
            Shape::RoundRect => {
                // RoundRect bounds are the same as layer bounds
                layer_bounds
            }
            Shape::Path(_path_data) => {
                // For custom paths, we need to compute the actual path bounds
                let path = self.to_path(layer_bounds, border_corner_radius);
                *path.bounds()
            }
        }
    }

    /// Helper: Create a Shape from a Skia Path
    #[allow(dead_code)] // Public API - will be used by users
    pub fn from_path(path: &Path) -> Self {
        Shape::Path(PathData::from_path(path))
    }
}
