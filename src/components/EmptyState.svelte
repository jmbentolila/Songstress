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
  // The single-folder case renders separately below (path in italic), so
  // the string cases here are the ones with no path in them.
  const singleFolder = $derived(
    ui.musicFolders.length === 1 ? ui.musicFolders[0] : null,
  );
  const headline = $derived.by(() => {
    const folders = ui.musicFolders;
    if (folders.length === 0) {
      return "Pick the folder that holds your music to build your library.";
    }
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
    <!-- Stencil glyph without the squircle: geometry mirrors the #stencil
         mask in assets/app-icon.svg (white = paint, black = knockout) — if
         the mic ever changes there, mirror it here. Inlined (not <img>) on
         purpose: currentColor inside an image-document cannot see the
         page's CSS, so only inline paint follows the theme. -->
    <svg class="mark" viewBox="0 0 512 512" aria-hidden="true">
      <defs>
        <mask id="empty-glyph">
          <rect width="512" height="512" fill="black" />
          <g transform="translate(252 268) rotate(40) translate(-252 -268)">
            <rect x="192" y="222" width="120" height="180" rx="60" fill="white" />
            <path d="M 156 312 a 96 96 0 0 0 192 0" fill="none" stroke="white" stroke-width="30" stroke-linecap="round" />
            <rect x="226" y="404" width="52" height="72" rx="24" fill="white" />
            <rect x="182" y="258" width="140" height="18" fill="black" />
            <rect x="182" y="300" width="140" height="18" fill="black" />
            <path d="M 200 178 A 62 62 0 0 1 304 178" fill="none" stroke="white" stroke-width="26" stroke-linecap="round" />
            <path d="M 172 122 A 96 96 0 0 1 332 122" fill="none" stroke="white" stroke-width="22" stroke-linecap="round" opacity="0.72" />
          </g>
        </mask>
      </defs>
      <g transform="translate(256 256) scale(0.875) translate(-256 -256)">
        <rect width="512" height="512" fill="currentColor" mask="url(#empty-glyph)" />
      </g>
    </svg>
    <h2>Welcome to Songstress</h2>
    {#if singleFolder}
      <p class="dim">No albums found in <em>{singleFolder}</em></p>
    {:else}
      <p class="dim">{headline}</p>
    {/if}
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

  /* App mark above the welcome headline — the stencil glyph without the
     squircle stage (assets/app-glyph.svg), at the asked 75%. currentColor
     via `color` so it survives the light theme, where the tile's fixed
     near-white would wash out. */
  .mark {
    width: 144px;
    height: 144px;
    opacity: 0.75;
    color: var(--text);
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
    /* Long single-folder paths must wrap instead of overflowing —
       break-word (not anywhere): anywhere would also shrink the
       paragraph's intrinsic width and rewrap the whole sentence. */
    overflow-wrap: break-word;
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
    /* Equal widths by construction: the wider label ("Add music folder…",
       measured 177px) sets the floor, so Rescan/Try again match it. */
    min-width: 180px;
  }

  .primary {
    border: none;
    background: var(--accent);
    color: var(--accent-text, #fff);
    /* Ink rhythm: the 24px headline's line slack + descender put ~4px more
       air above the subline than the 10px box gap puts above this button —
       measured live, not eyeballed. */
    margin-top: 4px;
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
