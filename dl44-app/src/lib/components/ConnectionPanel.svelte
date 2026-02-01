<script lang="ts">
  import {
    ports,
    baudRates,
    selectedPort,
    selectedBaud,
    connectionState,
    connected,
    refreshPorts,
    connect,
    disconnect,
    isConnected,
    isConnecting,
    hasError,
    getConnectionInfo,
  } from "../stores/machine";

  let connecting = false;
  let error: string | null = null;

  async function handleConnect() {
    error = null;
    connecting = true;
    try {
      await connect();
    } catch (e: any) {
      error = e.message || String(e);
    } finally {
      connecting = false;
    }
  }

  async function handleDisconnect() {
    error = null;
    try {
      await disconnect();
    } catch (e: any) {
      error = e.message || String(e);
    }
  }

  async function handleRefresh() {
    await refreshPorts();
  }

  $: connInfo = getConnectionInfo($connectionState);
  $: isConn = isConnected($connectionState);
  $: isConnecting_ = isConnecting($connectionState);
</script>

<div class="connection-panel">
  <h3>Connection</h3>

  {#if isConn && connInfo}
    <div class="connected-info">
      <span class="status-indicator connected"></span>
      <span>Connected to {connInfo.port} @ {connInfo.baud}</span>
      <button on:click={handleDisconnect} class="disconnect-btn">
        Disconnect
      </button>
    </div>
  {:else}
    <div class="connection-form">
      <div class="form-row">
        <label for="port-select">Port:</label>
        <select id="port-select" bind:value={$selectedPort} disabled={connecting}>
          {#each $ports as port}
            <option value={port.path}>
              {port.path}
              {#if port.product}({port.product}){/if}
            </option>
          {/each}
        </select>
        <button on:click={handleRefresh} class="refresh-btn" title="Refresh ports">
          ↻
        </button>
      </div>

      <div class="form-row">
        <label for="baud-select">Baud:</label>
        <select id="baud-select" bind:value={$selectedBaud} disabled={connecting}>
          {#each $baudRates as rate}
            <option value={rate}>{rate}</option>
          {/each}
        </select>
      </div>

      <button
        on:click={handleConnect}
        disabled={connecting || !$selectedPort}
        class="connect-btn"
      >
        {#if connecting || isConnecting_}
          Connecting...
        {:else}
          Connect
        {/if}
      </button>
    </div>
  {/if}

  {#if error}
    <div class="error-message">{error}</div>
  {/if}

  {#if hasError($connectionState)}
    <div class="error-message">
      Connection error: {$connectionState.Error}
    </div>
  {/if}
</div>

<style>
  .connection-panel {
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

  .connected-info {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .status-indicator {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: #666;
  }

  .status-indicator.connected {
    background: #4caf50;
  }

  .connection-form {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .form-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  label {
    width: 50px;
    color: #aaa;
  }

  select {
    flex: 1;
    padding: 0.5rem;
    background: #2a2a2a;
    border: 1px solid #444;
    border-radius: 4px;
    color: #fff;
  }

  button {
    padding: 0.5rem 1rem;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-weight: 500;
  }

  .connect-btn {
    background: #2196f3;
    color: white;
  }

  .connect-btn:hover:not(:disabled) {
    background: #1976d2;
  }

  .connect-btn:disabled {
    background: #444;
    cursor: not-allowed;
  }

  .disconnect-btn {
    background: #f44336;
    color: white;
    margin-left: auto;
  }

  .disconnect-btn:hover {
    background: #d32f2f;
  }

  .refresh-btn {
    padding: 0.5rem;
    background: #333;
    color: #aaa;
  }

  .refresh-btn:hover {
    background: #444;
  }

  .error-message {
    margin-top: 0.5rem;
    padding: 0.5rem;
    background: rgba(244, 67, 54, 0.2);
    border: 1px solid #f44336;
    border-radius: 4px;
    color: #f44336;
    font-size: 0.9rem;
  }
</style>
