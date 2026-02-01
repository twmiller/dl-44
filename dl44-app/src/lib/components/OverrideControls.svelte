<script lang="ts">
  import {
    connected,
    overrides,
    feedOverride,
    spindleOverride,
    rapidOverride,
    type OverrideAdjust,
    type RapidOverride,
  } from "../stores/machine";

  let error: string | null = null;

  async function handleFeedAdjust(adjust: OverrideAdjust) {
    error = null;
    try {
      await feedOverride(adjust);
    } catch (e: any) {
      error = e.message || String(e);
    }
  }

  async function handleSpindleAdjust(adjust: OverrideAdjust) {
    error = null;
    try {
      await spindleOverride(adjust);
    } catch (e: any) {
      error = e.message || String(e);
    }
  }

  async function handleRapidPreset(preset: RapidOverride) {
    error = null;
    try {
      await rapidOverride(preset);
    } catch (e: any) {
      error = e.message || String(e);
    }
  }
</script>

<div class="override-controls">
  <h3>Overrides</h3>

  <div class="override-section">
    <div class="override-row">
      <span class="override-label">Feed</span>
      <span class="override-value" class:modified={$overrides.feed !== 100}>
        {$overrides.feed}%
      </span>
      <div class="override-buttons">
        <button
          on:click={() => handleFeedAdjust("CoarseMinus")}
          disabled={!$connected}
          title="Feed -10%"
        >
          -10
        </button>
        <button
          on:click={() => handleFeedAdjust("FineMinus")}
          disabled={!$connected}
          title="Feed -1%"
        >
          -1
        </button>
        <button
          class="reset-btn"
          on:click={() => handleFeedAdjust("Reset")}
          disabled={!$connected}
          title="Reset to 100%"
        >
          100
        </button>
        <button
          on:click={() => handleFeedAdjust("FinePlus")}
          disabled={!$connected}
          title="Feed +1%"
        >
          +1
        </button>
        <button
          on:click={() => handleFeedAdjust("CoarsePlus")}
          disabled={!$connected}
          title="Feed +10%"
        >
          +10
        </button>
      </div>
    </div>

    <div class="override-row">
      <span class="override-label">Spindle</span>
      <span class="override-value" class:modified={$overrides.spindle !== 100}>
        {$overrides.spindle}%
      </span>
      <div class="override-buttons">
        <button
          on:click={() => handleSpindleAdjust("CoarseMinus")}
          disabled={!$connected}
          title="Spindle -10%"
        >
          -10
        </button>
        <button
          on:click={() => handleSpindleAdjust("FineMinus")}
          disabled={!$connected}
          title="Spindle -1%"
        >
          -1
        </button>
        <button
          class="reset-btn"
          on:click={() => handleSpindleAdjust("Reset")}
          disabled={!$connected}
          title="Reset to 100%"
        >
          100
        </button>
        <button
          on:click={() => handleSpindleAdjust("FinePlus")}
          disabled={!$connected}
          title="Spindle +1%"
        >
          +1
        </button>
        <button
          on:click={() => handleSpindleAdjust("CoarsePlus")}
          disabled={!$connected}
          title="Spindle +10%"
        >
          +10
        </button>
      </div>
    </div>

    <div class="override-row">
      <span class="override-label">Rapid</span>
      <span class="override-value" class:modified={$overrides.rapid !== 100}>
        {$overrides.rapid}%
      </span>
      <div class="override-buttons rapid-presets">
        <button
          class:selected={$overrides.rapid === 25}
          on:click={() => handleRapidPreset("Quarter")}
          disabled={!$connected}
          title="25%"
        >
          25%
        </button>
        <button
          class:selected={$overrides.rapid === 50}
          on:click={() => handleRapidPreset("Half")}
          disabled={!$connected}
          title="50%"
        >
          50%
        </button>
        <button
          class="reset-btn"
          class:selected={$overrides.rapid === 100}
          on:click={() => handleRapidPreset("Full")}
          disabled={!$connected}
          title="100%"
        >
          100%
        </button>
      </div>
    </div>
  </div>

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
  .override-controls {
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

  .override-section {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .override-row {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .override-label {
    width: 60px;
    font-size: 0.85rem;
    color: #888;
  }

  .override-value {
    width: 50px;
    font-family: "JetBrains Mono", "Fira Code", monospace;
    font-size: 0.9rem;
    color: #ccc;
    text-align: right;
  }

  .override-value.modified {
    color: #ff9800;
    font-weight: 600;
  }

  .override-buttons {
    display: flex;
    gap: 0.25rem;
  }

  .override-buttons button {
    padding: 0.35rem 0.5rem;
    min-width: 36px;
    background: #2a2a2a;
    border: 1px solid #444;
    border-radius: 3px;
    color: #aaa;
    cursor: pointer;
    font-size: 0.75rem;
    font-family: inherit;
  }

  .override-buttons button:hover:not(:disabled) {
    background: #333;
    border-color: #555;
  }

  .override-buttons button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .override-buttons button.reset-btn {
    background: #2a3a2a;
    border-color: #4caf50;
    color: #4caf50;
  }

  .override-buttons button.reset-btn:hover:not(:disabled) {
    background: #3a4a3a;
  }

  .rapid-presets button.selected {
    background: #2196f3;
    border-color: #2196f3;
    color: white;
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
