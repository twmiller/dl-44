<script lang="ts">
  import {
    connected,
    machineState,
    runFrame,
    type Units,
    type FrameMode,
  } from "../stores/machine";
  import { workspaceBounds } from "../stores/workspace";

  // Frame boundary - linked to workspace bounds or manual entry
  let xMin = 0;
  let xMax = 100;
  let yMin = 0;
  let yMax = 100;
  let useWorkspaceBounds = true;

  // Sync with workspace bounds when enabled
  $: if (useWorkspaceBounds && $workspaceBounds) {
    xMin = $workspaceBounds.x_min;
    xMax = $workspaceBounds.x_max;
    yMin = $workspaceBounds.y_min;
    yMax = $workspaceBounds.y_max;
  }

  $: hasWorkspaceBounds = $workspaceBounds !== null;

  // Frame settings
  let frameFeed = 1000; // mm/min
  let framePower = 10; // S value (low power for visibility, not cutting)
  let frameUnits: Units = "Mm";
  let frameMode: FrameMode = "LowPower";

  // Preset power levels for frame
  const powerPresets = [5, 10, 20, 50];
  const feedPresets = [500, 1000, 2000, 3000];

  // Mode options for UI
  const modeOptions: Array<{
    value: FrameMode;
    label: string;
    description: string;
  }> = [
    {
      value: "LowPower",
      label: "M4 Low Power",
      description: "Dynamic power - scales with speed, safe for corners",
    },
    {
      value: "ConstantPower",
      label: "M3 Constant",
      description: "Constant power - may burn at slow speeds/corners",
    },
    {
      value: "LaserOff",
      label: "Guide Only",
      description: "No laser - rapid moves to check travel path",
    },
  ];

  // Get current mode description
  $: currentModeDescription =
    modeOptions.find((m) => m.value === frameMode)?.description ?? "";

  let error: string | null = null;
  let running = false;

  async function handleRunFrame() {
    error = null;
    running = true;
    try {
      await runFrame(
        xMin,
        xMax,
        yMin,
        yMax,
        frameFeed,
        framePower,
        frameUnits,
        frameMode
      );
    } catch (e: any) {
      error = e.message || String(e);
    } finally {
      running = false;
    }
  }

  // Power controls are only relevant when laser is on
  $: showPowerControls = frameMode !== "LaserOff";

  $: canFrame = $connected && $machineState === "idle" && !running;
</script>

<div class="frame-controls">
  <h3>Frame / Boundary Check</h3>

  <div class="frame-settings">
    <div class="mode-selector">
      <span class="group-label">Laser Mode</span>
      <div class="mode-options">
        {#each modeOptions as option}
          <button
            class="mode-btn"
            class:selected={frameMode === option.value}
            on:click={() => (frameMode = option.value)}
            disabled={!$connected}
            title={option.description}
          >
            {option.label}
          </button>
        {/each}
      </div>
      <span class="mode-description">{currentModeDescription}</span>
    </div>

    <div class="param-row">
      <span class="param-label">Units</span>
      <div class="button-group">
        <button
          class:selected={frameUnits === "Mm"}
          on:click={() => (frameUnits = "Mm")}
          disabled={!$connected}
        >
          mm
        </button>
        <button
          class:selected={frameUnits === "Inches"}
          on:click={() => (frameUnits = "Inches")}
          disabled={!$connected}
        >
          in
        </button>
      </div>
    </div>

    <div class="bounds-group">
      <div class="bounds-header">
        <span class="group-label">Bounds ({frameUnits === "Mm" ? "mm" : "in"})</span>
        {#if hasWorkspaceBounds}
          <label class="auto-bounds-toggle">
            <input
              type="checkbox"
              bind:checked={useWorkspaceBounds}
              disabled={!$connected}
            />
            <span>Use job bounds</span>
          </label>
        {/if}
      </div>
      <div class="bounds-inputs">
        <div class="bound-field">
          <label for="x-min">X min</label>
          <input
            id="x-min"
            type="number"
            bind:value={xMin}
            step="1"
            disabled={!$connected || useWorkspaceBounds}
          />
        </div>
        <div class="bound-field">
          <label for="x-max">X max</label>
          <input
            id="x-max"
            type="number"
            bind:value={xMax}
            step="1"
            disabled={!$connected || useWorkspaceBounds}
          />
        </div>
        <div class="bound-field">
          <label for="y-min">Y min</label>
          <input
            id="y-min"
            type="number"
            bind:value={yMin}
            step="1"
            disabled={!$connected || useWorkspaceBounds}
          />
        </div>
        <div class="bound-field">
          <label for="y-max">Y max</label>
          <input
            id="y-max"
            type="number"
            bind:value={yMax}
            step="1"
            disabled={!$connected || useWorkspaceBounds}
          />
        </div>
      </div>
    </div>

    <div class="param-row">
      <span class="param-label">Feed (mm/min)</span>
      <div class="button-group">
        {#each feedPresets as preset}
          <button
            class:selected={frameFeed === preset}
            on:click={() => (frameFeed = preset)}
            disabled={!$connected}
          >
            {preset}
          </button>
        {/each}
      </div>
    </div>

    {#if showPowerControls}
      <div class="param-row">
        <span class="param-label">Power (S)</span>
        <div class="button-group">
          {#each powerPresets as preset}
            <button
              class:selected={framePower === preset}
              on:click={() => (framePower = preset)}
              disabled={!$connected}
            >
              {preset}
            </button>
          {/each}
        </div>
        <span class="power-note">Low power for visibility</span>
      </div>
    {/if}
  </div>

  <button class="run-frame-btn" on:click={handleRunFrame} disabled={!canFrame}>
    {#if running}
      Running...
    {:else}
      Run Frame
    {/if}
  </button>

  {#if error}
    <div class="error-message">{error}</div>
  {/if}

  {#if !$connected}
    <div class="overlay">
      <span>Connect to enable controls</span>
    </div>
  {/if}
</div>

<style>
  .frame-controls {
    position: relative;
    padding: 1rem;
    border: 1px solid #333;
    border-radius: 4px;
    background: #1a1a1a;
  }

  h3 {
    margin: 0 0 1rem 0;
    font-size: 1rem;
    color: #888;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .frame-settings {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    margin-bottom: 1rem;
  }

  .bounds-group {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .group-label {
    font-size: 0.8rem;
    color: #666;
  }

  .mode-selector {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .mode-options {
    display: flex;
    gap: 0.25rem;
  }

  .mode-btn {
    flex: 1;
    padding: 0.4rem 0.5rem;
    background: #2a2a2a;
    border: 1px solid #444;
    border-radius: 3px;
    color: #aaa;
    cursor: pointer;
    font-size: 0.75rem;
    white-space: nowrap;
  }

  .mode-btn:hover:not(:disabled) {
    background: #333;
  }

  .mode-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .mode-btn.selected {
    background: #00bcd4;
    border-color: #00bcd4;
    color: white;
  }

  .mode-description {
    font-size: 0.7rem;
    color: #666;
    font-style: italic;
  }

  .bounds-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.5rem;
  }

  .auto-bounds-toggle {
    display: flex;
    align-items: center;
    gap: 0.3rem;
    font-size: 0.75rem;
    color: #888;
    cursor: pointer;
  }

  .auto-bounds-toggle input {
    cursor: pointer;
  }

  .auto-bounds-toggle:hover {
    color: #aaa;
  }

  .bounds-inputs {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 0.5rem;
  }

  .bound-field {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .bound-field label {
    font-size: 0.7rem;
    color: #666;
  }

  .bound-field input {
    padding: 0.35rem 0.5rem;
    background: #2a2a2a;
    border: 1px solid #444;
    border-radius: 3px;
    color: #ccc;
    font-family: "JetBrains Mono", "Fira Code", monospace;
    font-size: 0.85rem;
    width: 100%;
    box-sizing: border-box;
  }

  .bound-field input:disabled {
    opacity: 0.5;
  }

  .bound-field input:focus {
    outline: none;
    border-color: #2196f3;
  }

  .param-row {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .param-label {
    width: 90px;
    font-size: 0.85rem;
    color: #888;
  }

  .button-group {
    display: flex;
    gap: 0.25rem;
  }

  .button-group button {
    padding: 0.3rem 0.6rem;
    background: #2a2a2a;
    border: 1px solid #444;
    border-radius: 3px;
    color: #aaa;
    cursor: pointer;
    font-size: 0.8rem;
  }

  .button-group button:hover:not(:disabled) {
    background: #333;
  }

  .button-group button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .button-group button.selected {
    background: #2196f3;
    border-color: #2196f3;
    color: white;
  }

  .power-note {
    font-size: 0.7rem;
    color: #666;
    font-style: italic;
  }

  .run-frame-btn {
    width: 100%;
    padding: 0.75rem;
    background: #00bcd4;
    border: none;
    border-radius: 4px;
    color: white;
    font-weight: 600;
    font-size: 1rem;
    cursor: pointer;
  }

  .run-frame-btn:hover:not(:disabled) {
    background: #00acc1;
  }

  .run-frame-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .error-message {
    margin-top: 0.5rem;
    padding: 0.5rem;
    background: rgba(244, 67, 54, 0.2);
    border: 1px solid #f44336;
    border-radius: 4px;
    color: #f44336;
    font-size: 0.9rem;
    text-align: center;
  }

  .overlay {
    position: absolute;
    inset: 0;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 4px;
    color: #888;
    font-style: italic;
  }
</style>
