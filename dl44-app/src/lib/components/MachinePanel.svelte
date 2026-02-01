<script lang="ts">
  import ConnectionPanel from "./ConnectionPanel.svelte";
  import StatusBar from "./StatusBar.svelte";
  import JogControls from "./JogControls.svelte";
  import OverrideControls from "./OverrideControls.svelte";
  import FrameControls from "./FrameControls.svelte";
</script>

<div class="machine-panel">
  <StatusBar />
  <div class="panel-content">
    <ConnectionPanel />
    <JogControls />
    <OverrideControls />
    <FrameControls />
  </div>
</div>

<style>
  .machine-panel {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .panel-content {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }
</style>
