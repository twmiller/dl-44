<script lang="ts">
  import {
    lastError,
    dismissError,
    getErrorMessage,
    type UIError,
  } from "../stores/machine";

  // Auto-dismiss after 8 seconds
  let dismissTimer: ReturnType<typeof setTimeout> | null = null;
  let currentErrorId: number | null = null;

  $: if ($lastError && $lastError.id !== currentErrorId) {
    currentErrorId = $lastError.id;

    // Clear previous timer
    if (dismissTimer) {
      clearTimeout(dismissTimer);
    }

    // Set new auto-dismiss timer
    dismissTimer = setTimeout(() => {
      if ($lastError) {
        dismissError($lastError.id);
      }
    }, 8000);
  }

  function handleDismiss() {
    if ($lastError) {
      dismissError($lastError.id);
    }
  }

  function getErrorIcon(code: string): string {
    switch (code) {
      case "ALARM":
        return "⚠";
      case "TIMEOUT":
        return "⏱";
      case "GRBL_ERROR":
        return "✕";
      case "NOT_CONNECTED":
        return "⊘";
      default:
        return "!";
    }
  }

  function getErrorClass(code: string): string {
    switch (code) {
      case "ALARM":
        return "alarm";
      case "TIMEOUT":
        return "timeout";
      default:
        return "error";
    }
  }
</script>

{#if $lastError}
  <div
    class="error-toast {getErrorClass($lastError.error.code)}"
    role="alert"
    aria-live="assertive"
  >
    <span class="icon">{getErrorIcon($lastError.error.code)}</span>
    <div class="content">
      <div class="code">{$lastError.error.code}</div>
      <div class="message">{getErrorMessage($lastError.error)}</div>
      {#if $lastError.error.details}
        <div class="details">{$lastError.error.details}</div>
      {/if}
    </div>
    <button class="dismiss" on:click={handleDismiss} aria-label="Dismiss">
      ×
    </button>
  </div>
{/if}

<style>
  .error-toast {
    position: fixed;
    bottom: 1rem;
    right: 1rem;
    max-width: 400px;
    display: flex;
    align-items: flex-start;
    gap: 0.75rem;
    padding: 0.75rem 1rem;
    border-radius: 6px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
    animation: slideIn 0.2s ease-out;
    z-index: 1000;
  }

  @keyframes slideIn {
    from {
      transform: translateX(100%);
      opacity: 0;
    }
    to {
      transform: translateX(0);
      opacity: 1;
    }
  }

  .error-toast.error {
    background: #2a1a1a;
    border: 1px solid #f44336;
  }

  .error-toast.alarm {
    background: #2a2a1a;
    border: 1px solid #ff9800;
  }

  .error-toast.timeout {
    background: #1a1a2a;
    border: 1px solid #2196f3;
  }

  .icon {
    font-size: 1.25rem;
    line-height: 1;
    flex-shrink: 0;
  }

  .error .icon {
    color: #f44336;
  }

  .alarm .icon {
    color: #ff9800;
  }

  .timeout .icon {
    color: #2196f3;
  }

  .content {
    flex: 1;
    min-width: 0;
  }

  .code {
    font-size: 0.7rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: #888;
    margin-bottom: 0.25rem;
  }

  .message {
    font-size: 0.9rem;
    color: #fff;
    word-wrap: break-word;
  }

  .details {
    font-size: 0.8rem;
    color: #888;
    margin-top: 0.25rem;
    font-family: "JetBrains Mono", "Fira Code", monospace;
  }

  .dismiss {
    background: none;
    border: none;
    color: #666;
    font-size: 1.25rem;
    cursor: pointer;
    padding: 0;
    line-height: 1;
    flex-shrink: 0;
  }

  .dismiss:hover {
    color: #fff;
  }
</style>
