<script lang="ts">
  import { library } from "../lib/stores/library.svelte";
  import { addMusicFolderRoot, rescan, scanner } from "../lib/stores/scanner.svelte";
  import { ui } from "../lib/stores/ui.svelte";

  // This is the EMPTY state, not the LOADING one. While a scan is in flight, or
  // while the boot dump is still out, the grid and the artist list show their
  // skeletons (loadingState + GridSkeleton); nothing here renders, because a
  // "waiting" message with no progress on it is a placeholder, and a placeholder
  // that looks like a decision is the worst of both.
  // What remains is the state that survives the scan: still nothing indexed, and
  // that IS a decision the user has to make.

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
  {#if library.scanError}
    <!-- A scan that FAILED and a library that is genuinely EMPTY are different
         states, and only one of them has ever had a screen: the error used to
         stop at the webview console, which has no window here. Lead with what
         was NOT done — this is the one place the app has touched his files, or
         rather, failed to. -->
    <h2>Your library wasn’t updated</h2>
    <p class="dim">Nothing was moved, changed or deleted — your files are where they are.</p>
    <p class="why">{library.scanError}</p>
  {:else}
    <h2>Welcome to Songstress</h2>
    <p class="dim">{headline}</p>
  {/if}
  <button class="primary" disabled={scanner.running} onclick={() => void addMusicFolderRoot()}>
    Add music folder…
  </button>
  {#if ui.musicFolders.length > 0}
    <button class="ghost" disabled={scanner.running} onclick={() => void rescan()}>
      {library.scanError ? "Try again" : "Rescan"}
    </button>
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
    font-size: 24px;
    letter-spacing: -0.01em;
    color: var(--text);
  }

  p {
    margin: 0;
    max-width: 420px;
  }

  .dim {
    color: var(--text-dim);
    font-size: 15px;
  }

  /* The scanner's own words — a path plus an io error, long and unbroken. Same
     voice as the copy above (one font, no mono pairing), a step smaller, and in
     the system's ONE caution hue: the tag editor's failed SAVE already speaks
     that colour, so a failed scan must not invent a second error tint. */
  .why {
    font-size: 14px;
    color: var(--caution);
    overflow-wrap: anywhere;
  }

  button {
    cursor: pointer;
    border-radius: 8px;
    padding: 9px 18px;
    font-size: 16px;
  }

  .primary {
    border: none;
    background: var(--accent);
    color: var(--accent-text, #fff);
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
</style>
