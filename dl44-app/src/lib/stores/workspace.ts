/**
 * Workspace stores for document management and rendering.
 */
import { writable, derived, get } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

// Types matching Rust structs

export interface BoundingBox {
  x_min: number;
  y_min: number;
  x_max: number;
  y_max: number;
}

export interface Transform {
  x: number;
  y: number;
  scale: number;
  rotation: number;
}

export interface SvgContent {
  width: number;
  height: number;
  paths: unknown[];
  raw_svg: string;
}

export interface BitmapContent {
  width: number;
  height: number;
  data_url: string;
  format: string;
}

export type DocumentKind =
  | { type: "Svg"; width: number; height: number; paths: unknown[]; raw_svg: string }
  | { type: "Bitmap"; width: number; height: number; data_url: string; format: string };

export interface Document {
  id: number;
  name: string;
  source_path: string | null;
  kind: DocumentKind;
  transform: Transform;
  visible: boolean;
  locked: boolean;
  original_bounds: BoundingBox;
}

export interface WorkspaceSettings {
  width: number;
  height: number;
  grid_spacing: number;
  show_grid: boolean;
}

export interface WorkspaceData {
  version: number;
  documents: {
    documents: Document[];
    next_id: number;
  };
  settings: WorkspaceSettings;
}

// Stores

/** All documents in the workspace */
export const documents = writable<Document[]>([]);

/** Workspace settings */
export const workspaceSettings = writable<WorkspaceSettings>({
  width: 400,
  height: 400,
  grid_spacing: 10,
  show_grid: true,
});

/** Currently selected document ID */
export const selectedDocumentId = writable<number | null>(null);

/** Current workspace file path */
export const workspaceFilePath = writable<string | null>(null);

/** Workspace has unsaved changes */
export const hasUnsavedChanges = writable(false);

// Derived stores

/** Combined bounds of all visible documents */
export const workspaceBounds = derived(documents, ($docs): BoundingBox | null => {
  const visible = $docs.filter((d) => d.visible);
  if (visible.length === 0) return null;

  let x_min = Infinity,
    y_min = Infinity,
    x_max = -Infinity,
    y_max = -Infinity;

  for (const doc of visible) {
    const bounds = getDocumentWorkspaceBounds(doc);
    x_min = Math.min(x_min, bounds.x_min);
    y_min = Math.min(y_min, bounds.y_min);
    x_max = Math.max(x_max, bounds.x_max);
    y_max = Math.max(y_max, bounds.y_max);
  }

  return { x_min, y_min, x_max, y_max };
});

/** Selected document */
export const selectedDocument = derived(
  [documents, selectedDocumentId],
  ([$docs, $id]) => ($id !== null ? $docs.find((d) => d.id === $id) ?? null : null)
);

// Helpers

/** Get document bounds in workspace coordinates (with transform applied) */
export function getDocumentWorkspaceBounds(doc: Document): BoundingBox {
  const { original_bounds: ob, transform: t } = doc;
  const w = (ob.x_max - ob.x_min) * t.scale;
  const h = (ob.y_max - ob.y_min) * t.scale;
  return {
    x_min: t.x,
    y_min: t.y,
    x_max: t.x + w,
    y_max: t.y + h,
  };
}

// Actions

/** Refresh documents from backend */
export async function refreshDocuments(): Promise<void> {
  try {
    const docs = await invoke<Document[]>("get_documents");
    documents.set(docs);
  } catch (e) {
    console.error("Failed to get documents:", e);
  }
}

/** Refresh workspace settings from backend */
export async function refreshWorkspaceSettings(): Promise<void> {
  try {
    const settings = await invoke<WorkspaceSettings>("get_workspace_settings");
    workspaceSettings.set(settings);
  } catch (e) {
    console.error("Failed to get workspace settings:", e);
  }
}

/** Import a file by path */
export async function importFile(path: string): Promise<Document | null> {
  try {
    const doc = await invoke<Document>("import_document", { path });
    await refreshDocuments();
    hasUnsavedChanges.set(true);
    return doc;
  } catch (e) {
    console.error("Failed to import file:", e);
    throw e;
  }
}

/** Import from bytes (for drag-drop) */
export async function importBytes(
  name: string,
  bytes: Uint8Array,
  mimeType: string
): Promise<Document | null> {
  try {
    const doc = await invoke<Document>("import_document_bytes", {
      name,
      bytes: Array.from(bytes),
      mimeType,
    });
    await refreshDocuments();
    hasUnsavedChanges.set(true);
    return doc;
  } catch (e) {
    console.error("Failed to import bytes:", e);
    throw e;
  }
}

/** Remove a document */
export async function removeDocument(id: number): Promise<void> {
  try {
    await invoke("remove_document", { id });
    await refreshDocuments();
    hasUnsavedChanges.set(true);
    // Deselect if removed
    if (get(selectedDocumentId) === id) {
      selectedDocumentId.set(null);
    }
  } catch (e) {
    console.error("Failed to remove document:", e);
    throw e;
  }
}

/** Update document transform */
export async function updateDocumentTransform(
  id: number,
  transform: Transform
): Promise<void> {
  try {
    await invoke("update_document_transform", { id, transform });
    await refreshDocuments();
    hasUnsavedChanges.set(true);
  } catch (e) {
    console.error("Failed to update transform:", e);
    throw e;
  }
}

/** Update document visibility */
export async function updateDocumentVisibility(
  id: number,
  visible: boolean
): Promise<void> {
  try {
    await invoke("update_document_visibility", { id, visible });
    await refreshDocuments();
    hasUnsavedChanges.set(true);
  } catch (e) {
    console.error("Failed to update visibility:", e);
    throw e;
  }
}

/** Save workspace to file */
export async function saveWorkspace(path: string): Promise<void> {
  try {
    await invoke("save_workspace_to_file", { path });
    workspaceFilePath.set(path);
    hasUnsavedChanges.set(false);
  } catch (e) {
    console.error("Failed to save workspace:", e);
    throw e;
  }
}

/** Load workspace from file */
export async function loadWorkspace(path: string): Promise<void> {
  try {
    await invoke<WorkspaceData>("load_workspace_from_file", { path });
    await refreshDocuments();
    await refreshWorkspaceSettings();
    workspaceFilePath.set(path);
    hasUnsavedChanges.set(false);
    selectedDocumentId.set(null);
  } catch (e) {
    console.error("Failed to load workspace:", e);
    throw e;
  }
}

/** Create new workspace */
export async function newWorkspace(): Promise<void> {
  try {
    await invoke("new_workspace");
    await refreshDocuments();
    await refreshWorkspaceSettings();
    workspaceFilePath.set(null);
    hasUnsavedChanges.set(false);
    selectedDocumentId.set(null);
  } catch (e) {
    console.error("Failed to create new workspace:", e);
    throw e;
  }
}

/** Clear all documents */
export async function clearWorkspace(): Promise<void> {
  try {
    await invoke("clear_workspace");
    await refreshDocuments();
    hasUnsavedChanges.set(false);
    selectedDocumentId.set(null);
  } catch (e) {
    console.error("Failed to clear workspace:", e);
    throw e;
  }
}

/** Initialize workspace on app start */
export async function initializeWorkspace(): Promise<void> {
  await Promise.all([refreshDocuments(), refreshWorkspaceSettings()]);
}
