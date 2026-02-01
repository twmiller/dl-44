<script lang="ts">
  import { onMount } from "svelte";
  import WorkspaceCanvas from "./WorkspaceCanvas.svelte";
  import DocumentList from "./DocumentList.svelte";
  import { initializeWorkspace, importBytes } from "../stores/workspace";

  let initialized = false;
  let dragOver = false;

  onMount(async () => {
    await initializeWorkspace();
    initialized = true;
  });

  function handleDragOver(event: DragEvent) {
    event.preventDefault();
    dragOver = true;
  }

  function handleDragLeave() {
    dragOver = false;
  }

  async function handleDrop(event: DragEvent) {
    event.preventDefault();
    dragOver = false;

    const files = event.dataTransfer?.files;
    if (!files || files.length === 0) return;

    for (const file of files) {
      const ext = file.name.split(".").pop()?.toLowerCase();
      const supportedExts = ["svg", "png", "jpg", "jpeg", "gif", "bmp", "webp"];
      if (!ext || !supportedExts.includes(ext)) continue;

      try {
        const bytes = new Uint8Array(await file.arrayBuffer());
        await importBytes(file.name, bytes, file.type || `image/${ext}`);
      } catch (e) {
        console.error("Failed to import dropped file:", e);
      }
    }
  }
</script>

<div
  class="workspace-panel"
  class:drag-over={dragOver}
  on:dragover={handleDragOver}
  on:dragleave={handleDragLeave}
  on:drop={handleDrop}
  role="region"
  aria-label="Workspace"
>
  {#if !initialized}
    <div class="loading">Loading workspace...</div>
  {:else}
    <div class="workspace-content">
      <div class="canvas-area">
        <WorkspaceCanvas />
      </div>
      <div class="sidebar">
        <DocumentList />
      </div>
    </div>
  {/if}

  {#if dragOver}
    <div class="drop-overlay">
      <div class="drop-message">Drop files to import</div>
    </div>
  {/if}
</div>

<style>
  .workspace-panel {
    position: relative;
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 400px;
    border: 1px solid #333;
    border-radius: 4px;
    background: #1a1a1a;
    overflow: hidden;
  }

  .workspace-panel.drag-over {
    border-color: #2196f3;
  }

  .loading {
    display: flex;
    align-items: center;
    justify-content: center;
    flex: 1;
    color: #888;
    font-style: italic;
  }

  .workspace-content {
    display: flex;
    flex: 1;
    gap: 0;
  }

  .canvas-area {
    flex: 1;
    display: flex;
  }

  .sidebar {
    width: 220px;
    border-left: 1px solid #333;
    overflow-y: auto;
  }

  .drop-overlay {
    position: absolute;
    inset: 0;
    background: rgba(33, 150, 243, 0.2);
    border: 2px dashed #2196f3;
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    pointer-events: none;
  }

  .drop-message {
    padding: 1rem 2rem;
    background: #2196f3;
    border-radius: 4px;
    color: white;
    font-weight: 600;
  }
</style>
