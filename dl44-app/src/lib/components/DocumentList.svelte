<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import {
    documents,
    selectedDocumentId,
    importFile,
    removeDocument,
    updateDocumentVisibility,
    type Document,
  } from "../stores/workspace";

  let importing = false;
  let error: string | null = null;

  async function handleImport() {
    error = null;
    try {
      const selected = await open({
        multiple: true,
        filters: [
          {
            name: "Images",
            extensions: ["svg", "png", "jpg", "jpeg", "gif", "bmp", "webp"],
          },
        ],
      });

      if (selected) {
        importing = true;
        const paths = Array.isArray(selected) ? selected : [selected];
        for (const path of paths) {
          await importFile(path);
        }
      }
    } catch (e: any) {
      error = e.message || String(e);
    } finally {
      importing = false;
    }
  }

  async function handleRemove(id: number) {
    try {
      await removeDocument(id);
    } catch (e: any) {
      error = e.message || String(e);
    }
  }

  async function handleVisibilityToggle(doc: Document) {
    try {
      await updateDocumentVisibility(doc.id, !doc.visible);
    } catch (e: any) {
      error = e.message || String(e);
    }
  }

  function handleSelect(id: number) {
    selectedDocumentId.set(id);
  }

  function getDocumentIcon(doc: Document): string {
    if (doc.kind.type === "Svg") return "📐";
    return "🖼️";
  }
</script>

<div class="document-list">
  <div class="header">
    <h3>Documents</h3>
    <button class="import-btn" on:click={handleImport} disabled={importing}>
      {importing ? "..." : "+ Import"}
    </button>
  </div>

  {#if $documents.length === 0}
    <div class="empty-state">
      <p>No documents</p>
      <p class="hint">Import SVG or bitmap files</p>
    </div>
  {:else}
    <ul class="documents">
      {#each $documents as doc (doc.id)}
        <li
          class="document-item"
          class:selected={doc.id === $selectedDocumentId}
          on:click={() => handleSelect(doc.id)}
          on:keydown={(e) => e.key === "Enter" && handleSelect(doc.id)}
          tabindex="0"
          role="button"
        >
          <button
            class="visibility-btn"
            class:hidden={!doc.visible}
            on:click|stopPropagation={() => handleVisibilityToggle(doc)}
            title={doc.visible ? "Hide" : "Show"}
          >
            {doc.visible ? "👁️" : "👁️‍🗨️"}
          </button>

          <span class="icon">{getDocumentIcon(doc)}</span>
          <span class="name" title={doc.name}>{doc.name}</span>

          <button
            class="remove-btn"
            on:click|stopPropagation={() => handleRemove(doc.id)}
            title="Remove"
          >
            ✕
          </button>
        </li>
      {/each}
    </ul>
  {/if}

  {#if error}
    <div class="error">{error}</div>
  {/if}
</div>

<style>
  .document-list {
    display: flex;
    flex-direction: column;
    padding: 0.75rem;
    border: 1px solid #333;
    border-radius: 4px;
    background: #1a1a1a;
    min-width: 200px;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.75rem;
  }

  h3 {
    margin: 0;
    font-size: 0.9rem;
    color: #888;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .import-btn {
    padding: 0.25rem 0.5rem;
    background: #4caf50;
    border: none;
    border-radius: 3px;
    color: white;
    cursor: pointer;
    font-size: 0.75rem;
  }

  .import-btn:hover:not(:disabled) {
    background: #43a047;
  }

  .import-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .empty-state {
    text-align: center;
    padding: 1rem;
    color: #666;
  }

  .empty-state p {
    margin: 0.25rem 0;
  }

  .empty-state .hint {
    font-size: 0.8rem;
    font-style: italic;
  }

  .documents {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .document-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.4rem 0.5rem;
    background: #2a2a2a;
    border: 1px solid transparent;
    border-radius: 3px;
    cursor: pointer;
    font-size: 0.85rem;
  }

  .document-item:hover {
    background: #333;
  }

  .document-item.selected {
    border-color: #2196f3;
    background: rgba(33, 150, 243, 0.1);
  }

  .visibility-btn {
    background: none;
    border: none;
    cursor: pointer;
    padding: 0;
    font-size: 0.9rem;
    opacity: 0.7;
  }

  .visibility-btn:hover {
    opacity: 1;
  }

  .visibility-btn.hidden {
    opacity: 0.3;
  }

  .icon {
    font-size: 0.9rem;
  }

  .name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: #ccc;
  }

  .remove-btn {
    background: none;
    border: none;
    cursor: pointer;
    padding: 0.1rem 0.3rem;
    color: #888;
    font-size: 0.75rem;
    opacity: 0;
    transition: opacity 0.15s;
  }

  .document-item:hover .remove-btn {
    opacity: 1;
  }

  .remove-btn:hover {
    color: #f44336;
  }

  .error {
    margin-top: 0.5rem;
    padding: 0.5rem;
    background: rgba(244, 67, 54, 0.2);
    border: 1px solid #f44336;
    border-radius: 3px;
    color: #f44336;
    font-size: 0.8rem;
  }
</style>
