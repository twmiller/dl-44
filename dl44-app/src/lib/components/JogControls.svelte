<script lang="ts">
  import {
    connected,
    machineState,
    jog,
    jogCancel,
    home,
    unlock,
    softReset,
  } from "../stores/machine";

  // Jog step sizes in mm
  const stepSizes = [0.1, 1, 10, 50, 100];
  let selectedStep = 10;

  // Feed rates for jogging (mm/min)
  const feedRates = [100, 500, 1000, 3000, 6000];
  let selectedFeed = 1000;

  let error: string | null = null;

  async function handleJog(axis: "x" | "y" | "z", direction: 1 | -1) {
    error = null;
    try {
      const distance = selectedStep * direction;
      await jog(
        axis === "x" ? distance : null,
        axis === "y" ? distance : null,
        axis === "z" ? distance : null,
        selectedFeed,
        true
      );
    } catch (e: any) {
      error = e.message || String(e);
    }
  }

  async function handleHome() {
    error = null;
    try {
      await home();
    } catch (e: any) {
      error = e.message || String(e);
    }
  }

  async function handleUnlock() {
    error = null;
    try {
      await unlock();
    } catch (e: any) {
      error = e.message || String(e);
    }
  }

  async function handleReset() {
    error = null;
    try {
      await softReset();
    } catch (e: any) {
      error = e.message || String(e);
    }
  }

  async function handleCancel() {
    try {
      await jogCancel();
    } catch (e: any) {
      console.error("Jog cancel failed:", e);
    }
  }

  $: canJog =
    $connected && ($machineState === "idle" || $machineState === "jog");
  $: inAlarm = $machineState === "alarm";
</script>

<div class="jog-controls">
  <h3>Jog Controls</h3>

  <div class="settings-row">
    <div class="setting">
      <label>Step (mm):</label>
      <div class="button-group">
        {#each stepSizes as step}
          <button
            class:selected={selectedStep === step}
            on:click={() => (selectedStep = step)}
          >
            {step}
          </button>
        {/each}
      </div>
    </div>

    <div class="setting">
      <label>Feed (mm/min):</label>
      <div class="button-group">
        {#each feedRates as rate}
          <button
            class:selected={selectedFeed === rate}
            on:click={() => (selectedFeed = rate)}
          >
            {rate}
          </button>
        {/each}
      </div>
    </div>
  </div>

  <div class="jog-grid">
    <div class="xy-controls">
      <div class="jog-row">
        <div class="spacer"></div>
        <button
          class="jog-btn y-plus"
          on:click={() => handleJog("y", 1)}
          disabled={!canJog}
          title="Y+"
        >
          Y+
        </button>
        <div class="spacer"></div>
      </div>
      <div class="jog-row">
        <button
          class="jog-btn x-minus"
          on:click={() => handleJog("x", -1)}
          disabled={!canJog}
          title="X-"
        >
          X-
        </button>
        <button class="jog-btn center" on:click={handleCancel} title="Stop">
          ⏹
        </button>
        <button
          class="jog-btn x-plus"
          on:click={() => handleJog("x", 1)}
          disabled={!canJog}
          title="X+"
        >
          X+
        </button>
      </div>
      <div class="jog-row">
        <div class="spacer"></div>
        <button
          class="jog-btn y-minus"
          on:click={() => handleJog("y", -1)}
          disabled={!canJog}
          title="Y-"
        >
          Y-
        </button>
        <div class="spacer"></div>
      </div>
    </div>

    <div class="z-controls">
      <button
        class="jog-btn z-plus"
        on:click={() => handleJog("z", 1)}
        disabled={!canJog}
        title="Z+"
      >
        Z+
      </button>
      <span class="z-label">Z</span>
      <button
        class="jog-btn z-minus"
        on:click={() => handleJog("z", -1)}
        disabled={!canJog}
        title="Z-"
      >
        Z-
      </button>
    </div>
  </div>

  <div class="action-buttons">
    <button class="action-btn home" on:click={handleHome} disabled={!$connected}>
      Home
    </button>
    <button
      class="action-btn unlock"
      on:click={handleUnlock}
      disabled={!$connected || !inAlarm}
    >
      Unlock
    </button>
    <button class="action-btn reset" on:click={handleReset} disabled={!$connected}>
      Reset
    </button>
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
  .jog-controls {
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

  .settings-row {
    display: flex;
    gap: 1.5rem;
    margin-bottom: 1rem;
  }

  .setting {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .setting label {
    font-size: 0.8rem;
    color: #888;
  }

  .button-group {
    display: flex;
    gap: 0.25rem;
  }

  .button-group button {
    padding: 0.25rem 0.5rem;
    background: #2a2a2a;
    border: 1px solid #444;
    border-radius: 3px;
    color: #aaa;
    cursor: pointer;
    font-size: 0.8rem;
  }

  .button-group button:hover {
    background: #333;
  }

  .button-group button.selected {
    background: #2196f3;
    border-color: #2196f3;
    color: white;
  }

  .jog-grid {
    display: flex;
    gap: 2rem;
    justify-content: center;
    margin-bottom: 1rem;
  }

  .xy-controls {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .jog-row {
    display: flex;
    gap: 0.25rem;
  }

  .spacer {
    width: 50px;
    height: 50px;
  }

  .jog-btn {
    width: 50px;
    height: 50px;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-weight: 600;
    font-size: 0.9rem;
  }

  .jog-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .jog-btn.x-plus,
  .jog-btn.x-minus {
    background: rgba(244, 67, 54, 0.2);
    color: #f44336;
    border: 1px solid rgba(244, 67, 54, 0.5);
  }

  .jog-btn.y-plus,
  .jog-btn.y-minus {
    background: rgba(76, 175, 80, 0.2);
    color: #4caf50;
    border: 1px solid rgba(76, 175, 80, 0.5);
  }

  .jog-btn.center {
    background: #333;
    color: #fff;
    border: 1px solid #555;
  }

  .jog-btn:hover:not(:disabled) {
    filter: brightness(1.2);
  }

  .z-controls {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.25rem;
  }

  .z-label {
    color: #2196f3;
    font-weight: 600;
    font-size: 0.9rem;
  }

  .jog-btn.z-plus,
  .jog-btn.z-minus {
    width: 50px;
    height: 50px;
    background: rgba(33, 150, 243, 0.2);
    color: #2196f3;
    border: 1px solid rgba(33, 150, 243, 0.5);
  }

  .action-buttons {
    display: flex;
    gap: 0.5rem;
    justify-content: center;
  }

  .action-btn {
    padding: 0.5rem 1rem;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-weight: 500;
  }

  .action-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .action-btn.home {
    background: #4caf50;
    color: white;
  }

  .action-btn.unlock {
    background: #ff9800;
    color: white;
  }

  .action-btn.reset {
    background: #f44336;
    color: white;
  }

  .action-btn:hover:not(:disabled) {
    filter: brightness(1.1);
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
