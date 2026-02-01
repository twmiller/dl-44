//! Workspace persistence (save/load).

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use thiserror::Error;

use super::document::DocumentList;

/// Workspace file format version
const FORMAT_VERSION: u32 = 1;

/// Errors during workspace persistence
#[derive(Error, Debug)]
pub enum PersistenceError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Unsupported format version: {0}")]
    UnsupportedVersion(u32),
}

/// Workspace data for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceData {
    /// Format version
    pub version: u32,
    /// Document list
    pub documents: DocumentList,
    /// Workspace settings
    pub settings: WorkspaceSettings,
}

/// Workspace settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceSettings {
    /// Workspace width in mm
    pub width: f64,
    /// Workspace height in mm
    pub height: f64,
    /// Grid spacing in mm (0 = no grid)
    pub grid_spacing: f64,
    /// Show grid
    pub show_grid: bool,
}

impl Default for WorkspaceSettings {
    fn default() -> Self {
        Self {
            width: 400.0,  // Common laser bed size
            height: 400.0,
            grid_spacing: 10.0,
            show_grid: true,
        }
    }
}

impl Default for WorkspaceData {
    fn default() -> Self {
        Self {
            version: FORMAT_VERSION,
            documents: DocumentList::new(),
            settings: WorkspaceSettings::default(),
        }
    }
}

/// Save workspace to a file
pub fn save_workspace(path: &Path, data: &WorkspaceData) -> Result<(), PersistenceError> {
    let json = serde_json::to_string_pretty(data)?;
    fs::write(path, json)?;
    Ok(())
}

/// Load workspace from a file
pub fn load_workspace(path: &Path) -> Result<WorkspaceData, PersistenceError> {
    let json = fs::read_to_string(path)?;
    let data: WorkspaceData = serde_json::from_str(&json)?;

    // Check version compatibility
    if data.version > FORMAT_VERSION {
        return Err(PersistenceError::UnsupportedVersion(data.version));
    }

    Ok(data)
}
