<script>
  import { invoke } from "@tauri-apps/api/core";

  let name = "World";
  let greeting = "";

  async function greet() {
    greeting = await invoke("greet", { name });
  }
</script>

<main>
  <h1>DL-44</h1>
  <p>Minimal Svelte + Rust + Tauri hello world.</p>

  <div class="row">
    <input placeholder="Name" bind:value={name} />
    <button on:click={greet}>Greet</button>
  </div>

  {#if greeting}
    <p class="result">{greeting}</p>
  {/if}
</main>
