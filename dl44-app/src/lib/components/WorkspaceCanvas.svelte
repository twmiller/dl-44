<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import {
    documents,
    workspaceSettings,
    selectedDocumentId,
    type Document,
  } from "../stores/workspace";

  let canvas: HTMLCanvasElement;
  let ctx: CanvasRenderingContext2D | null = null;
  let containerWidth = 800;
  let containerHeight = 600;

  // View transform (pan/zoom)
  let viewX = 0;
  let viewY = 0;
  let viewScale = 1;

  // Cached images for bitmaps
  const imageCache = new Map<number, HTMLImageElement>();

  // Device pixel ratio for crisp rendering
  let dpr = 1;

  onMount(() => {
    ctx = canvas.getContext("2d");
    dpr = window.devicePixelRatio || 1;
    fitToWorkspace();
    render();
  });

  onDestroy(() => {
    imageCache.clear();
  });

  // Reactively render when documents or settings change
  $: if (ctx && $documents) {
    render();
  }

  $: if (ctx && $workspaceSettings) {
    render();
  }

  /** Fit view to show entire workspace */
  function fitToWorkspace() {
    const padding = 40;
    const availableWidth = containerWidth - padding * 2;
    const availableHeight = containerHeight - padding * 2;

    const scaleX = availableWidth / $workspaceSettings.width;
    const scaleY = availableHeight / $workspaceSettings.height;
    viewScale = Math.min(scaleX, scaleY, 2); // Cap at 2x zoom

    // Center the workspace
    const scaledWidth = $workspaceSettings.width * viewScale;
    const scaledHeight = $workspaceSettings.height * viewScale;
    viewX = (containerWidth - scaledWidth) / 2;
    viewY = (containerHeight - scaledHeight) / 2;
  }

  /** Convert workspace coords to canvas coords */
  function toCanvas(x: number, y: number): [number, number] {
    return [viewX + x * viewScale, viewY + y * viewScale];
  }

  /** Main render function */
  function render() {
    if (!ctx) return;

    // Set canvas size with DPR for crisp rendering
    canvas.width = containerWidth * dpr;
    canvas.height = containerHeight * dpr;
    canvas.style.width = `${containerWidth}px`;
    canvas.style.height = `${containerHeight}px`;
    ctx.scale(dpr, dpr);

    // Clear background
    ctx.fillStyle = "#1e1e1e";
    ctx.fillRect(0, 0, containerWidth, containerHeight);

    // Draw workspace background
    const [wsX, wsY] = toCanvas(0, 0);
    const wsW = $workspaceSettings.width * viewScale;
    const wsH = $workspaceSettings.height * viewScale;

    ctx.fillStyle = "#2a2a2a";
    ctx.fillRect(wsX, wsY, wsW, wsH);

    // Draw grid
    if ($workspaceSettings.show_grid && $workspaceSettings.grid_spacing > 0) {
      drawGrid();
    }

    // Draw documents (back to front)
    for (const doc of $documents) {
      if (doc.visible) {
        drawDocument(doc);
      }
    }

    // Draw workspace border
    ctx.strokeStyle = "#444";
    ctx.lineWidth = 1;
    ctx.strokeRect(wsX, wsY, wsW, wsH);
  }

  /** Draw grid lines */
  function drawGrid() {
    if (!ctx) return;

    const spacing = $workspaceSettings.grid_spacing;
    ctx.strokeStyle = "#333";
    ctx.lineWidth = 0.5;

    // Vertical lines
    for (let x = 0; x <= $workspaceSettings.width; x += spacing) {
      const [cx, cy1] = toCanvas(x, 0);
      const [, cy2] = toCanvas(x, $workspaceSettings.height);
      ctx.beginPath();
      ctx.moveTo(cx, cy1);
      ctx.lineTo(cx, cy2);
      ctx.stroke();
    }

    // Horizontal lines
    for (let y = 0; y <= $workspaceSettings.height; y += spacing) {
      const [cx1, cy] = toCanvas(0, y);
      const [cx2] = toCanvas($workspaceSettings.width, y);
      ctx.beginPath();
      ctx.moveTo(cx1, cy);
      ctx.lineTo(cx2, cy);
      ctx.stroke();
    }
  }

  /** Draw a single document */
  function drawDocument(doc: Document) {
    if (!ctx) return;

    const { transform: t, original_bounds: ob } = doc;
    const [x, y] = toCanvas(t.x, t.y);
    const w = (ob.x_max - ob.x_min) * t.scale * viewScale;
    const h = (ob.y_max - ob.y_min) * t.scale * viewScale;

    if (doc.kind.type === "Svg") {
      drawSvg(doc, x, y, w, h);
    } else if (doc.kind.type === "Bitmap") {
      drawBitmap(doc, x, y, w, h);
    }

    // Draw selection highlight
    if (doc.id === $selectedDocumentId) {
      ctx.strokeStyle = "#2196f3";
      ctx.lineWidth = 2;
      ctx.setLineDash([5, 5]);
      ctx.strokeRect(x - 2, y - 2, w + 4, h + 4);
      ctx.setLineDash([]);
    }
  }

  /** Draw SVG document */
  function drawSvg(doc: Document, x: number, y: number, w: number, h: number) {
    if (!ctx || doc.kind.type !== "Svg") return;

    // Create an image from the SVG
    const svgBlob = new Blob([doc.kind.raw_svg], { type: "image/svg+xml" });
    const url = URL.createObjectURL(svgBlob);

    let img = imageCache.get(doc.id);
    if (!img) {
      img = new Image();
      img.onload = () => {
        imageCache.set(doc.id, img!);
        render(); // Re-render when loaded
      };
      img.src = url;
    } else {
      ctx.drawImage(img, x, y, w, h);
    }
  }

  /** Draw bitmap document */
  function drawBitmap(doc: Document, x: number, y: number, w: number, h: number) {
    if (!ctx || doc.kind.type !== "Bitmap") return;

    let img = imageCache.get(doc.id);
    if (!img) {
      img = new Image();
      img.onload = () => {
        imageCache.set(doc.id, img!);
        render(); // Re-render when loaded
      };
      img.src = doc.kind.data_url;
    } else {
      ctx.drawImage(img, x, y, w, h);
    }
  }

  /** Handle canvas click for selection */
  function handleClick(event: MouseEvent) {
    const rect = canvas.getBoundingClientRect();
    const clickX = event.clientX - rect.left;
    const clickY = event.clientY - rect.top;

    // Convert to workspace coords
    const wsX = (clickX - viewX) / viewScale;
    const wsY = (clickY - viewY) / viewScale;

    // Find clicked document (front to back, so reverse order)
    for (let i = $documents.length - 1; i >= 0; i--) {
      const doc = $documents[i];
      if (!doc.visible) continue;

      const { transform: t, original_bounds: ob } = doc;
      const docW = (ob.x_max - ob.x_min) * t.scale;
      const docH = (ob.y_max - ob.y_min) * t.scale;

      if (wsX >= t.x && wsX <= t.x + docW && wsY >= t.y && wsY <= t.y + docH) {
        selectedDocumentId.set(doc.id);
        return;
      }
    }

    // Clicked on empty space - deselect
    selectedDocumentId.set(null);
  }

  /** Handle window resize */
  function handleResize() {
    if (canvas && canvas.parentElement) {
      containerWidth = canvas.parentElement.clientWidth;
      containerHeight = canvas.parentElement.clientHeight;
      fitToWorkspace();
      render();
    }
  }
</script>

<svelte:window on:resize={handleResize} />

<div class="canvas-container" bind:clientWidth={containerWidth} bind:clientHeight={containerHeight}>
  <canvas bind:this={canvas} on:click={handleClick}></canvas>

  <div class="canvas-controls">
    <button on:click={fitToWorkspace} title="Fit to workspace">Fit</button>
    <span class="zoom-label">{Math.round(viewScale * 100)}%</span>
  </div>
</div>

<style>
  .canvas-container {
    position: relative;
    flex: 1;
    min-height: 300px;
    background: #1e1e1e;
    overflow: hidden;
  }

  canvas {
    display: block;
    cursor: crosshair;
  }

  .canvas-controls {
    position: absolute;
    bottom: 8px;
    right: 8px;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.25rem 0.5rem;
    background: rgba(0, 0, 0, 0.7);
    border-radius: 4px;
  }

  .canvas-controls button {
    padding: 0.25rem 0.5rem;
    background: #333;
    border: 1px solid #555;
    border-radius: 3px;
    color: #ccc;
    cursor: pointer;
    font-size: 0.75rem;
  }

  .canvas-controls button:hover {
    background: #444;
  }

  .zoom-label {
    font-size: 0.75rem;
    color: #888;
    font-family: monospace;
  }
</style>
