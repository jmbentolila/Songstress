<script lang="ts">
  import { library } from "../lib/stores/library.svelte";
  import { addMusicFolderRoot, rescan, scanner } from "../lib/stores/scanner.svelte";
  import { ui } from "../lib/stores/ui.svelte";

  const pct = $derived(
    scanner.total > 0 ? Math.round((scanner.done / scanner.total) * 100) : 0,
  );

  // Step 7c: the library can have several roots, so the headline names the
  // folder only while there is exactly one — a 4-path list here is noise.
  const headline = $derived.by(() => {
    const folders = ui.musicFolders;
    if (folders.length === 0) {
      return "Pick the folder that holds your music to build your library.";
    }
    if (folders.length === 1) return `No albums found in ${folders[0]}`;
    return `No albums found in your ${folders.length} music folders`;
  });
</script>

<div class="empty">
  {#if library.scanning || scanner.running}
    <h2>Building your library…</h2>
    {#if scanner.total > 0}
      <div class="bar">
        <div class="fill" style:width={`${pct}%`}></div>
      </div>
      <p class="dim">
        {scanner.phase === "artwork" ? "Extracting artwork" : "Scanning"} —
        {scanner.done}/{scanner.total}
      </p>
    {:else}
      <p class="dim">Starting scan…</p>
    {/if}
  {:else}
    <h2>Welcome to Songstress</h2>
    <p class="dim">{headline}</p>
    <button class="primary" disabled={scanner.running} onclick={() => void addMusicFolderRoot()}>
      {scanner.running ? "Scanning…" : "Add music folder…"}
    </button>
    {#if ui.musicFolders.length > 0}
      <button class="ghost" disabled={scanner.running} onclick={() => void rescan()}>
        Rescan
      </button>
    {/if}
  {/if}
</div>

<style>
  .empty {
    position: absolute;
    inset: var(--gap);
    left: calc(var(--sidebar-width) + var(--gap));
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 10px;
    text-align: center;
  }

  h2 {
    margin: 0;
    font-size: 22px;
    letter-spacing: -0.01em;
    color: var(--text);
  }

  p {
    margin: 0;
    max-width: 420px;
  }

  .dim {
    color: var(--text-dim);
    font-size: 13px;
  }

  button {
    cursor: pointer;
    border-radius: 8px;
    padding: 9px 18px;
    font-size: 14px;
  }

  .primary {
    border: none;
    background: var(--accent);
    color: #fff;
  }

  .primary:hover {
    filter: brightness(1.1);
  }

  /* roots can be added while a scan is already running elsewhere (watcher,
     menu rescan) — the buttons go quiet rather than lying about being usable */
  button:disabled {
    opacity: 0.55;
    cursor: default;
  }

  .primary:disabled:hover {
    filter: none;
  }

  .ghost {
    background: transparent;
    color: var(--text-dim);
    border: 1px solid var(--border);
  }

  .bar {
    width: 320px;
    height: 6px;
    border-radius: 3px;
    overflow: hidden;
    background: var(--hover);
  }

  .fill {
    height: 100%;
    background: var(--accent);
    transition: width 0.25s ease-out;
  }
</style>
