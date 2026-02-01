<script lang="ts">
  import { onMount } from "svelte";
  import MachinePanel from "./lib/components/MachinePanel.svelte";
  import ErrorToast from "./lib/components/ErrorToast.svelte";
  import { initializeStores, stopPolling } from "./lib/stores/machine";

  let initialized = false;
  let initError: string | null = null;

  onMount(async () => {
    try {
      await initializeStores();
      initialized = true;
    } catch (e: any) {
      initError = e.message || String(e);
    }

    return () => {
      stopPolling();
    };
  });
</script>

<main>
  <header>
    <h1>DL-44</h1>
    <span class="subtitle">Laser Control</span>
  </header>

  {#if initError}
    <div class="error">
      <p>Failed to initialize: {initError}</p>
    </div>
  {:else if !initialized}
    <div class="loading">
      <p>Initializing...</p>
    </div>
  {:else}
    <div class="content">
      <MachinePanel />
    </div>
  {/if}

  <ErrorToast />
</main>

<style>
  :global(body) {
    margin: 0;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Oxygen,
      Ubuntu, Cantarell, sans-serif;
    background: #121212;
    color: #fff;
  }

  main {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
  }

  header {
    display: flex;
    align-items: baseline;
    gap: 0.75rem;
    padding: 1rem 1.5rem;
    background: #1a1a1a;
    border-bottom: 1px solid #333;
  }

  h1 {
    margin: 0;
    font-size: 1.5rem;
    font-weight: 700;
    color: #ff9800;
  }

  .subtitle {
    color: #666;
    font-size: 0.9rem;
  }

  .content {
    flex: 1;
    padding: 1rem;
  }

  .loading,
  .error {
    display: flex;
    align-items: center;
    justify-content: center;
    flex: 1;
    color: #888;
  }

  .error {
    color: #f44336;
  }
</style>
