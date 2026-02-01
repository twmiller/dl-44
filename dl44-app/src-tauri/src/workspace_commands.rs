//! Tauri commands for workspace operations.

use parking_lot::Mutex;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;

use crate::workspace::{
    import_file, import_from_bytes, load_workspace, save_workspace, BoundingBox, Document,
    DocumentId, DocumentList, ImportError, Transform, WorkspaceData, WorkspaceSettings,
};

/// Workspace state
pub struct WorkspaceState {
    pub data: Mutex<WorkspaceData>,
    /// Path to current workspace file (if saved)
    pub current_file: Mutex<Option<PathBuf>>,
}

impl WorkspaceState {
    pub fn new() -> Self {
        Self {
            data: Mutex::new(WorkspaceData::default()),
            current_file: Mutex::new(None),
        }
    }
}

impl Default for WorkspaceState {
    fn default() -> Self {
        Self::new()
    }
}

/// Error type for workspace commands
#[derive(Debug, serde::Serialize)]
pub struct WorkspaceError {
    pub message: String,
    pub code: String,
}

impl From<ImportError> for WorkspaceError {
    fn from(e: ImportError) -> Self {
        Self {
            message: e.to_string(),
            code: "IMPORT_ERROR".into(),
        }
    }
}

impl From<crate::workspace::persistence::PersistenceError> for WorkspaceError {
    fn from(e: crate::workspace::persistence::PersistenceError) -> Self {
        Self {
            message: e.to_string(),
            code: "PERSISTENCE_ERROR".into(),
        }
    }
}

type WorkspaceResult<T> = Result<T, WorkspaceError>;

/// Get current workspace data
#[tauri::command]
pub fn get_workspace(state: State<Arc<WorkspaceState>>) -> WorkspaceData {
    state.data.lock().clone()
}

/// Get workspace settings
#[tauri::command]
pub fn get_workspace_settings(state: State<Arc<WorkspaceState>>) -> WorkspaceSettings {
    state.data.lock().settings.clone()
}

/// Update workspace settings
#[tauri::command]
pub fn update_workspace_settings(
    state: State<Arc<WorkspaceState>>,
    settings: WorkspaceSettings,
) {
    state.data.lock().settings = settings;
}

/// Get all documents
#[tauri::command]
pub fn get_documents(state: State<Arc<WorkspaceState>>) -> Vec<Document> {
    state.data.lock().documents.all().to_vec()
}

/// Get combined bounds of all visible documents
#[tauri::command]
pub fn get_workspace_bounds(state: State<Arc<WorkspaceState>>) -> BoundingBox {
    state.data.lock().documents.combined_bounds()
}

/// Import a file into the workspace
#[tauri::command]
pub fn import_document(
    state: State<Arc<WorkspaceState>>,
    path: String,
) -> WorkspaceResult<Document> {
    let path = PathBuf::from(path);
    let doc = import_file(&path)?;

    let mut data = state.data.lock();
    let id = data.documents.add(doc.clone());

    // Return the document with assigned ID
    Ok(data.documents.get(id).cloned().unwrap())
}

/// Import from raw bytes (for drag-drop)
#[tauri::command]
pub fn import_document_bytes(
    state: State<Arc<WorkspaceState>>,
    name: String,
    bytes: Vec<u8>,
    mime_type: String,
) -> WorkspaceResult<Document> {
    let doc = import_from_bytes(&name, &bytes, &mime_type)?;

    let mut data = state.data.lock();
    let id = data.documents.add(doc);

    Ok(data.documents.get(id).cloned().unwrap())
}

/// Remove a document
#[tauri::command]
pub fn remove_document(
    state: State<Arc<WorkspaceState>>,
    id: DocumentId,
) -> WorkspaceResult<()> {
    let mut data = state.data.lock();
    data.documents.remove(id);
    Ok(())
}

/// Update document transform
#[tauri::command]
pub fn update_document_transform(
    state: State<Arc<WorkspaceState>>,
    id: DocumentId,
    transform: Transform,
) -> WorkspaceResult<()> {
    let mut data = state.data.lock();
    if let Some(doc) = data.documents.get_mut(id) {
        doc.transform = transform;
        Ok(())
    } else {
        Err(WorkspaceError {
            message: format!("Document {} not found", id),
            code: "NOT_FOUND".into(),
        })
    }
}

/// Update document visibility
#[tauri::command]
pub fn update_document_visibility(
    state: State<Arc<WorkspaceState>>,
    id: DocumentId,
    visible: bool,
) -> WorkspaceResult<()> {
    let mut data = state.data.lock();
    if let Some(doc) = data.documents.get_mut(id) {
        doc.visible = visible;
        Ok(())
    } else {
        Err(WorkspaceError {
            message: format!("Document {} not found", id),
            code: "NOT_FOUND".into(),
        })
    }
}

/// Reorder document in the list
#[tauri::command]
pub fn reorder_document(
    state: State<Arc<WorkspaceState>>,
    id: DocumentId,
    new_index: usize,
) -> WorkspaceResult<()> {
    let mut data = state.data.lock();
    data.documents.reorder(id, new_index);
    Ok(())
}

/// Clear all documents
#[tauri::command]
pub fn clear_workspace(state: State<Arc<WorkspaceState>>) {
    let mut data = state.data.lock();
    data.documents.clear();
    *state.current_file.lock() = None;
}

/// Save workspace to file
#[tauri::command]
pub fn save_workspace_to_file(
    state: State<Arc<WorkspaceState>>,
    path: String,
) -> WorkspaceResult<()> {
    let path = PathBuf::from(&path);
    let data = state.data.lock();
    save_workspace(&path, &data)?;
    drop(data);
    *state.current_file.lock() = Some(path);
    Ok(())
}

/// Load workspace from file
#[tauri::command]
pub fn load_workspace_from_file(
    state: State<Arc<WorkspaceState>>,
    path: String,
) -> WorkspaceResult<WorkspaceData> {
    let path = PathBuf::from(&path);
    let data = load_workspace(&path)?;
    *state.data.lock() = data.clone();
    *state.current_file.lock() = Some(path);
    Ok(data)
}

/// Get current workspace file path
#[tauri::command]
pub fn get_workspace_file_path(state: State<Arc<WorkspaceState>>) -> Option<String> {
    state
        .current_file
        .lock()
        .as_ref()
        .map(|p| p.to_string_lossy().to_string())
}

/// Create new workspace (clears current)
#[tauri::command]
pub fn new_workspace(state: State<Arc<WorkspaceState>>) {
    *state.data.lock() = WorkspaceData::default();
    *state.current_file.lock() = None;
}
