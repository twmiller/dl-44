<script lang="ts">
  import {
    machineState,
    machinePosition,
    workPosition,
    machineStatus,
    connected,
    statusIsFresh,
    overrides,
    overridesReported,
  } from "../stores/machine";

  function formatPos(n: number): string {
    return n.toFixed(3);
  }

  function stateColor(state: string): string {
    switch (state) {
      case "idle":
        return "#4caf50";
      case "run":
        return "#2196f3";
      case "hold":
        return "#ff9800";
      case "jog":
        return "#9c27b0";
      case "alarm":
        return "#f44336";
      case "door":
        return "#ff5722";
      case "home":
        return "#00bcd4";
      default:
        return "#666";
    }
  }
</script>

<div class="status-bar">
  <div class="state-display" style="--state-color: {stateColor($machineState)}">
    <span class="state-indicator"></span>
    <span class="state-text">{$machineState.toUpperCase()}</span>
  </div>

  {#if $connected}
    <div class="position-display">
      <div class="pos-group">
        <span class="pos-label">Work</span>
        <div class="pos-values">
          <span class="axis x">X: {formatPos($workPosition.x)}</span>
          <span class="axis y">Y: {formatPos($workPosition.y)}</span>
          <span class="axis z">Z: {formatPos($workPosition.z)}</span>
        </div>
      </div>

      <div class="pos-group">
        <span class="pos-label">Machine</span>
        <div class="pos-values">
          <span class="axis x">X: {formatPos($machinePosition.x)}</span>
          <span class="axis y">Y: {formatPos($machinePosition.y)}</span>
          <span class="axis z">Z: {formatPos($machinePosition.z)}</span>
        </div>
      </div>
    </div>

    {#if !$statusIsFresh}
      <div class="stale-indicator" title="Last status update timed out">
        STALE
      </div>
    {/if}

    {#if $machineStatus?.feed_rate !== null}
      <div class="rate-display">
        <span class="rate-label">Feed:</span>
        <span class="rate-value">{$machineStatus?.feed_rate ?? 0}</span>
      </div>
    {/if}

    {#if $machineStatus?.spindle_speed !== null}
      <div class="rate-display">
        <span class="rate-label">S:</span>
        <span class="rate-value">{$machineStatus?.spindle_speed ?? 0}</span>
      </div>
    {/if}

    {#if !$overridesReported}
      <div class="override-display not-reported" title="Overrides not reported (check $10 setting)">
        <span class="override-item">Ovr: N/A</span>
      </div>
    {:else if $overrides.feed !== 100 || $overrides.spindle !== 100 || $overrides.rapid !== 100}
      <div class="override-display">
        {#if $overrides.feed !== 100}
          <span class="override-item feed" title="Feed override">
            F:{$overrides.feed}%
          </span>
        {/if}
        {#if $overrides.rapid !== 100}
          <span class="override-item rapid" title="Rapid override">
            R:{$overrides.rapid}%
          </span>
        {/if}
        {#if $overrides.spindle !== 100}
          <span class="override-item spindle" title="Spindle/Laser override">
            S:{$overrides.spindle}%
          </span>
        {/if}
      </div>
    {/if}
  {:else}
    <div class="disconnected-message">Not connected</div>
  {/if}
</div>

<style>
  .status-bar {
    display: flex;
    align-items: center;
    gap: 1.5rem;
    padding: 0.75rem 1rem;
    background: #1a1a1a;
    border: 1px solid #333;
    border-radius: 4px;
    font-family: "JetBrains Mono", "Fira Code", monospace;
  }

  .state-display {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .state-indicator {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: var(--state-color);
    box-shadow: 0 0 8px var(--state-color);
  }

  .state-text {
    font-weight: 600;
    color: var(--state-color);
    min-width: 60px;
  }

  .position-display {
    display: flex;
    gap: 1.5rem;
  }

  .pos-group {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .pos-label {
    font-size: 0.7rem;
    color: #666;
    text-transform: uppercase;
  }

  .pos-values {
    display: flex;
    gap: 1rem;
  }

  .axis {
    font-size: 0.9rem;
    color: #ccc;
  }

  .axis.x {
    color: #f44336;
  }

  .axis.y {
    color: #4caf50;
  }

  .axis.z {
    color: #2196f3;
  }

  .rate-display {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    color: #888;
  }

  .stale-indicator {
    padding: 0.2rem 0.5rem;
    border: 1px solid #ff9800;
    border-radius: 4px;
    color: #ff9800;
    font-size: 0.7rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  .rate-label {
    font-size: 0.8rem;
  }

  .rate-value {
    color: #ccc;
  }

  .disconnected-message {
    color: #666;
    font-style: italic;
  }

  .override-display {
    display: flex;
    gap: 0.5rem;
    padding: 0.2rem 0.5rem;
    background: rgba(255, 152, 0, 0.1);
    border: 1px solid #ff9800;
    border-radius: 4px;
  }

  .override-display.not-reported {
    background: rgba(100, 100, 100, 0.1);
    border-color: #666;
  }

  .override-display.not-reported .override-item {
    color: #666;
    font-style: italic;
  }

  .override-item {
    font-size: 0.8rem;
    font-weight: 500;
  }

  .override-item.feed {
    color: #4caf50;
  }

  .override-item.rapid {
    color: #2196f3;
  }

  .override-item.spindle {
    color: #ff9800;
  }
</style>
