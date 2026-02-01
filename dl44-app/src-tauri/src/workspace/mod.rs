//! Workspace management for imported documents.
//!
//! Handles SVG and bitmap imports, document list, bounds calculation,
//! and workspace persistence.

pub mod document;
pub mod import;
pub mod persistence;

pub use document::{Document, DocumentId, DocumentKind, DocumentList, BoundingBox, Transform};
pub use import::{import_file, import_from_bytes, ImportError};
pub use persistence::{WorkspaceData, WorkspaceSettings, save_workspace, load_workspace};
