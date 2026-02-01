//! Document types for workspace items.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Unique identifier for a document
pub type DocumentId = u64;

/// Axis-aligned bounding box
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct BoundingBox {
    pub x_min: f64,
    pub y_min: f64,
    pub x_max: f64,
    pub y_max: f64,
}

impl BoundingBox {
    pub fn new(x_min: f64, y_min: f64, x_max: f64, y_max: f64) -> Self {
        Self { x_min, y_min, x_max, y_max }
    }

    pub fn width(&self) -> f64 {
        self.x_max - self.x_min
    }

    pub fn height(&self) -> f64 {
        self.y_max - self.y_min
    }

    pub fn is_empty(&self) -> bool {
        self.width() <= 0.0 || self.height() <= 0.0
    }

    /// Merge another bounding box into this one (union)
    pub fn merge(&mut self, other: &BoundingBox) {
        if other.is_empty() {
            return;
        }
        if self.is_empty() {
            *self = *other;
            return;
        }
        self.x_min = self.x_min.min(other.x_min);
        self.y_min = self.y_min.min(other.y_min);
        self.x_max = self.x_max.max(other.x_max);
        self.y_max = self.y_max.max(other.y_max);
    }

    /// Offset the bounding box by a translation
    pub fn translate(&mut self, dx: f64, dy: f64) {
        self.x_min += dx;
        self.y_min += dy;
        self.x_max += dx;
        self.y_max += dy;
    }
}

/// SVG path data (simplified for initial implementation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SvgPath {
    /// SVG path data string (d attribute)
    pub d: String,
    /// Stroke color (if any)
    pub stroke: Option<String>,
    /// Fill color (if any)
    pub fill: Option<String>,
    /// Stroke width
    pub stroke_width: f64,
}

/// SVG document content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SvgContent {
    /// Original SVG width (from viewBox or width attribute)
    pub width: f64,
    /// Original SVG height
    pub height: f64,
    /// Extracted paths
    pub paths: Vec<SvgPath>,
    /// Raw SVG string for rendering
    pub raw_svg: String,
}

/// Bitmap document content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitmapContent {
    /// Image width in pixels
    pub width: u32,
    /// Image height in pixels
    pub height: u32,
    /// Base64-encoded image data (for frontend rendering)
    pub data_url: String,
    /// Original file format
    pub format: String,
}

/// Document content variant
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DocumentKind {
    Svg(SvgContent),
    Bitmap(BitmapContent),
}

/// Transform applied to a document
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Transform {
    /// X position in workspace (mm)
    pub x: f64,
    /// Y position in workspace (mm)
    pub y: f64,
    /// Scale factor (1.0 = original size)
    pub scale: f64,
    /// Rotation in degrees
    pub rotation: f64,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            scale: 1.0,
            rotation: 0.0,
        }
    }
}

/// A document in the workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// Unique identifier
    pub id: DocumentId,
    /// Display name
    pub name: String,
    /// Original file path (if imported from file)
    pub source_path: Option<PathBuf>,
    /// Document content
    pub kind: DocumentKind,
    /// Transform in workspace
    pub transform: Transform,
    /// Visibility
    pub visible: bool,
    /// Locked (cannot be moved/edited)
    pub locked: bool,
    /// Original bounds (before transform)
    pub original_bounds: BoundingBox,
}

impl Document {
    /// Get the transformed bounding box in workspace coordinates
    pub fn workspace_bounds(&self) -> BoundingBox {
        let mut bounds = self.original_bounds;

        // Apply scale
        let w = bounds.width() * self.transform.scale;
        let h = bounds.height() * self.transform.scale;

        // Apply translation (position is top-left corner)
        bounds.x_min = self.transform.x;
        bounds.y_min = self.transform.y;
        bounds.x_max = self.transform.x + w;
        bounds.y_max = self.transform.y + h;

        // Note: rotation not handled yet (would need proper matrix transform)
        bounds
    }
}

/// List of documents in the workspace
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DocumentList {
    documents: Vec<Document>,
    next_id: DocumentId,
}

impl DocumentList {
    pub fn new() -> Self {
        Self {
            documents: Vec::new(),
            next_id: 1,
        }
    }

    /// Add a document and return its ID
    pub fn add(&mut self, mut doc: Document) -> DocumentId {
        let id = self.next_id;
        self.next_id += 1;
        doc.id = id;
        self.documents.push(doc);
        id
    }

    /// Remove a document by ID
    pub fn remove(&mut self, id: DocumentId) -> Option<Document> {
        if let Some(idx) = self.documents.iter().position(|d| d.id == id) {
            Some(self.documents.remove(idx))
        } else {
            None
        }
    }

    /// Get a document by ID
    pub fn get(&self, id: DocumentId) -> Option<&Document> {
        self.documents.iter().find(|d| d.id == id)
    }

    /// Get a mutable document by ID
    pub fn get_mut(&mut self, id: DocumentId) -> Option<&mut Document> {
        self.documents.iter_mut().find(|d| d.id == id)
    }

    /// Get all documents
    pub fn all(&self) -> &[Document] {
        &self.documents
    }

    /// Get all visible documents
    pub fn visible(&self) -> impl Iterator<Item = &Document> {
        self.documents.iter().filter(|d| d.visible)
    }

    /// Compute combined bounds of all visible documents
    pub fn combined_bounds(&self) -> BoundingBox {
        let mut bounds = BoundingBox::default();
        for doc in self.visible() {
            bounds.merge(&doc.workspace_bounds());
        }
        bounds
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.documents.is_empty()
    }

    /// Number of documents
    pub fn len(&self) -> usize {
        self.documents.len()
    }

    /// Clear all documents
    pub fn clear(&mut self) {
        self.documents.clear();
        self.next_id = 1;
    }

    /// Reorder document (move to new index)
    pub fn reorder(&mut self, id: DocumentId, new_index: usize) {
        if let Some(old_idx) = self.documents.iter().position(|d| d.id == id) {
            let doc = self.documents.remove(old_idx);
            let insert_idx = new_index.min(self.documents.len());
            self.documents.insert(insert_idx, doc);
        }
    }
}
