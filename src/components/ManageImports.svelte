<script lang="ts">
  /**
   * The door that replaces "Save imported music" / "Discard imported music".
   *
   * Those two verbs asked the question in the wrong order: they acted on the
   * whole pile before saying where the files would go, and the pile's shape —
   * which album joins which folder, which one starts a new artist — is what the
   * decision is actually about. So the destination is on screen per album
   * (resolved in Rust, by the same cascade the save itself will follow), the
   * decision is marked per album, and one Apply commits every mark.
   */
  import { ui } from "../lib/stores/ui.svelte";
  import { scanner } from "../lib/stores/scanner.svelte";
  import {
    applyImportDecisions,
    closeImportManager,
    imports,
    markAllImports,
    markImport,
    refreshImportPlan,
    toggleImportTracks,
  } from "../lib/stores/imports.svelte";
  import {
    destinationLine,
    fmtDuration,
    groupByArtist,
    plural,
    summarize,
    trackNumbers,
  } from "../lib/importPlan";
  import SurfaceClose from "./SurfaceClose.svelte";
  import ProgressRing from "./ProgressRing.svelte";

  const groups = $derived(groupByArtist(imports.plan));
  const sum = $derived(summarize(imports.plan, imports.decisions));
  const busy = $derived(imports.applying || scanner.running);

  // The arc is the outer count — albums applied of albums marked — plus the
  // fine-grained fraction the backend reports while files are copying, so a
  // 36-track album moves smoothly instead of jumping at the end. Only the copy
  // phase counts: the scan that follows a save has its own totals.
  const inner = $derived(
    scanner.running && scanner.phase === "import" && scanner.total > 0
      ? Math.min(1, scanner.done / scanner.total)
      : 0,
  );
  const progress = $derived(
    imports.total > 0 ? Math.min(1, (imports.done + inner) / imports.total) : inner,
  );
  // A file imported while this window is open — from the pane behind it, the
  // Global Menu, anywhere — has no reason to be invisible here, and a mark made
  // before it landed would otherwise apply to an album nobody has seen. So the
  // plan is re-read whenever a scan finishes with this window up.
  let wasScanning = false;
  $effect(() => {
    const running = scanner.running;
    if (wasScanning && !running && imports.open) void refreshImportPlan();
    wasScanning = running;
  });

  // What Apply is about to do, said in the order the buttons above it are
  // marked in. "2 decisions marked" would describe the UI; this describes the
  // files.
  const footLine = $derived(
    imports.waiting
      ? "Waiting for the scan to finish…"
      : imports.applying
      ? `${imports.active}…`
        : sum.decided === 0
        ? "Mark an album to apply"
        : sum.save === 0
          ? `${plural(sum.discard, "album")} to discard`
          : sum.discard === 0
            ? `${plural(sum.save, "album")} to save`
            : `${plural(sum.save, "album")} to save · ${plural(sum.discard, "album")} to discard`,
  );

  function onKeydown(e: KeyboardEvent) {
    if (!imports.open) return;
    if (e.key === "Escape" && !imports.applying) {
      closeImportManager();
      return;
    }
    if (e.key !== "ArrowDown" && e.key !== "ArrowUp") return;
    // Arrow-walk the controls, as the menu stack does. Restricted to the panel
    // so a focused element outside cannot be dragged in.
    const btns = [...(panel?.querySelectorAll<HTMLButtonElement>("button:not(:disabled)") ?? [])];
    const i = btns.indexOf(document.activeElement as HTMLButtonElement);
    if (i === -1) return;
    e.preventDefault();
    const next =
      e.key === "ArrowDown" ? Math.min(i + 1, btns.length - 1) : Math.max(i - 1, 0);
    btns[next]?.focus({ preventScroll: true });
  }

  let panel = $state<HTMLElement | null>(null);
  // Closing hands focus back to the door it came from, unless the door itself is
  // gone (an emptied pile removes the row) — a keyboard user should not be dropped
  // at the top of the document for having finished a task.
  let wasOpen = false;
  $effect(() => {
    const open = imports.open;
    if (wasOpen && !open) {
      document.querySelector<HTMLElement>("[data-imports-door]")?.focus({ preventScroll: true });
    }
    wasOpen = open;
  });

  // The window fetches for itself when it is up with nothing in hand: the store
  // fills the plan on open, and a module reload that cleared the plan without
  // closing the window would otherwise leave it claiming nothing is waiting.
  let fetched = false;
  $effect(() => {
    if (imports.open && !fetched) {
      fetched = true;
      void refreshImportPlan();
    }
  });

  // Focus the first decision when the window opens, so the keyboard does not
  // have to cross the panel to get to work. preventScroll: the body is a scroll
  // port and would jump the list to the focused row.
  $effect(() => {
    if (!imports.open) return;
    requestAnimationFrame(() => {
      panel?.querySelector<HTMLButtonElement>("button.mi-name")?.focus({ preventScroll: true });
    });
  });
</script>

<svelte:window onkeydown={onKeydown} />

{#if imports.open}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div
    class="mi-backdrop scrim"
    role="presentation"
    onclick={(e) => {
      if (e.target === e.currentTarget && !imports.applying) closeImportManager();
    }}
  >
    <!-- svelte-ignore a11y_no_noninteractive_element_to_interactive_role -->
    <section class="mi glass" bind:this={panel} role="dialog" aria-modal="true" aria-label="Imported music">
      <header class="mi-head">
        <SurfaceClose label="Close" onclick={() => (imports.applying ? null : closeImportManager())} />
        <h2>Imported music</h2>
      </header>

      <div class="mi-body">
        {#if groups.length === 0}
          <p class="mi-empty">
            Nothing is waiting to be imported. Files you import will show up here
            with the folder they are about to move to.
          </p>
        {:else}
          {#each groups as g (g.artist)}
            <div class="mi-group" role="group" aria-label={g.artist}>
              <div class="mi-glabel">{g.artist}</div>
              {#each g.albums as a (a.albumId)}
                {@const open = !!imports.expanded[a.albumId]}
                {@const decision = imports.decisions[a.albumId]}
                {@const dest = destinationLine(a, ui.musicFolders)}
                {@const numbers = trackNumbers(a)}
                <div class="mi-album">
                  <div class="mi-main">
                    <div class="mi-row">
                      <button
                        class="mi-caret"
                        aria-expanded={open}
                        aria-label={`${open ? "Hide" : "Show"} the files of ${a.title}`}
                        disabled={imports.applying}
                        onclick={() => toggleImportTracks(a.albumId)}
                      >
                        {open ? "▾" : "▸"}
                      </button>
                      <button
                        class="mi-name"
                        aria-expanded={open}
                        disabled={imports.applying}
                        onclick={() => toggleImportTracks(a.albumId)}
                      >
                        <span class="mi-title">{a.title}</span>
                        <span class="mi-meta">
                          {a.year ?? "—"} · {plural(a.tracks.length, "track")}
                        </span>
                      </button>
                    </div>

                    <div class="mi-dest" title={a.destination.folder}>
                      <span class="mi-rule">{dest.lead}</span>
                      <!-- The two claims are different kinds of statement (what will
                           happen / where), and 6px of gap did not say that. A middle
                           dot, the same separator the meta line and the footer
                           summary already use. Hidden from assistive tech: it is
                           punctuation for the eye, and a screen reader has the two
                           spans in their own order. -->
                      <span class="mi-sep" aria-hidden="true">·</span>
                      <span class="mi-path">{dest.path}</span>
                    </div>

                    {#if open}
                      <ul class="mi-tracks">
                        {#each a.tracks as t, i (t.id)}
                          <li class="mi-track">
                            <span class="mi-num">{numbers[i]}</span>
                            <span class="mi-tname">{t.title}</span>
                            <span class="mi-tdur">{fmtDuration(t.durationSec)}</span>
                          </li>
                        {/each}
                      </ul>
                    {/if}
                  </div>

                  <!-- Outside the expander, so collapsing a long album cannot move
                       the buttons out from under a cursor that is on one. -->
                  <div class="mi-decide" role="group" aria-label="Decision for {a.title}">
                    <button
                      class="mi-mark mi-save"
                      aria-pressed={decision === "save"}
                      disabled={busy}
                      onclick={() => markImport(a.albumId, "save")}
                    >
                      Save
                    </button>
                    <button
                      class="mi-mark mi-drop"
                      aria-pressed={decision === "discard"}
                      disabled={busy}
                      onclick={() => markImport(a.albumId, "discard")}
                    >
                      Discard
                    </button>
                  </div>
                </div>
              {/each}
            </div>
          {/each}
        {/if}
      </div>

      <footer class="mi-foot">
        <div class="mi-sum">
          <span class="mi-counts"
            >{plural(sum.albums, "album")} · {plural(sum.tracks, "track")}</span
          >
          <span class="mi-line" aria-live="polite">
            {#if imports.applying}
              <ProgressRing value={progress} label={footLine} phase="apply" />
            {/if}
            <span>{footLine}</span>
          </span>
          {#if imports.failed.length}
            <span class="mi-failed" role="alert">
              Didn’t move: {imports.failed.join(", ")}
            </span>
          {/if}
        </div>

        <div class="mi-actions">
          <button
            class="mi-quiet"
            disabled={busy || groups.length === 0}
            onclick={() => markAllImports("save")}
          >
            Save all
          </button>
          <button
            class="mi-quiet"
            disabled={busy || groups.length === 0}
            onclick={() => markAllImports("discard")}
          >
            Discard all
          </button>
          <button
            class="mi-apply"
            disabled={busy || sum.decided === 0}
            aria-label={sum.decided > 0 ? `Apply ${sum.decided} decisions` : "Apply"}
            title={sum.decided === 0 ? "Mark an album first" : undefined}
            onclick={() => void applyImportDecisions()}
          >
            Apply
          </button>
        </div>
      </footer>
    </section>
  </div>
{/if}

<style>
  /* Same glass tier, radius and structure as the Music folders modal: two modal
     shapes in one app is one too many. */
  .mi {
    width: min(620px, calc(100vw - 80px));
    max-height: calc(100vh - 140px);
    display: flex;
    flex-direction: column;
    padding: 16px;
    border-radius: var(--radius-panel);
    border: 1px solid var(--border);
    background: var(--panel-bg-strong);
    box-shadow: var(--shadow);
    color: var(--text);
  }

  .mi-head {
    display: flex;
    align-items: center;
    gap: 12px;
    padding-bottom: 10px;
    border-bottom: 1px solid var(--border);
  }

  .mi-head h2 {
    flex: 1;
    margin: 0;
    font-size: 15px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .mi-body {
    overflow-y: auto;
    padding: 10px 0 0;
    display: flex;
    flex-direction: column;
    /* Artists are neighbours. With one card each, that is the only distance this
       window measures between groups. */
    gap: 12px;
  }

  /* One card PER ARTIST, with the albums as touching rows inside it — the
     grouped-list idiom the panes use for `.rows`, and the reason the spacing
     question in this tree went away. When each album was its own card, the 6px
     that bound an album to its artist label had to compete with the 12px that
     separates neighbours, and the reader had to decide which rung was which.
     Now the grouping is structural: the card IS the artist, the albums are its
     rows, and every gap in the window means one thing. */
  .mi-group {
    display: flex;
    flex-direction: column;
    background: var(--hover);
    border-radius: 8px;
    /* clip, not hidden: `hidden` makes the card a scroll port, and focusing a
       row inside it would scroll the card underneath the reader. */
    overflow: clip;
  }

  /* The artist is the card's header, not a caption floating above it: 10px of
     inset, then the 6px hug to the first row, with no divider — a divider would
     separate the artist from the albums it owns. */
  .mi-glabel {
    font-size: 11.5px;
    font-weight: 600;
    letter-spacing: 0.09em;
    text-transform: uppercase;
    color: var(--text-dim);
    padding: 10px 10px 6px;
  }

  .mi-album {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    align-items: start;
    gap: 10px;
    padding: 8px 10px;
  }

  /* Rows are divided, not spaced — the pane's `.rows` touch, made legible at
     this content height. */
  .mi-album + .mi-album {
    border-top: 1px solid var(--border);
  }

  .mi-main {
    min-width: 0;
    display: flex;
    flex-direction: column;
    /* The destination and the file list belong to this album: the ladder's hug. */
    gap: 6px;
  }

  .mi-row {
    display: flex;
    align-items: baseline;
    gap: 6px;
    min-width: 0;
  }

  .mi-caret {
    flex: none;
    width: 14px;
    border: none;
    background: transparent;
    color: var(--text-dim);
    font-size: 10px;
    line-height: 1;
    cursor: pointer;
    padding: 0;
  }

  .mi-caret:disabled {
    cursor: default;
  }

  .mi-name {
    display: flex;
    align-items: baseline;
    gap: 8px;
    min-width: 0;
    border: none;
    background: transparent;
    color: var(--text);
    padding: 0;
    cursor: pointer;
    text-align: left;
  }

  .mi-title {
    min-width: 0;
    font-size: 13.5px;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .mi-meta {
    flex: none;
    font-size: 11.5px;
    color: var(--text-dim);
    font-variant-numeric: tabular-nums;
  }

  .mi-name:hover .mi-title {
    text-decoration: underline;
    text-underline-offset: 2px;
  }

  .mi-name:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 3px;
    border-radius: 4px;
  }

  /* What will happen, then where. The rule is the sentence, the path is the
     evidence: they are different colors because they are different claims. */
  .mi-dest {
    display: flex;
    gap: 6px;
    min-width: 0;
    /* Lines up with the album title, not with the caret. */
    padding-left: 20px;
    font-size: 11.5px;
    color: var(--text-dim);
  }

  .mi-rule {
    flex: none;
    font-style: italic;
  }

  /* Quieter than both sides: a joint, not a third statement. */
  .mi-sep {
    flex: none;
    opacity: 0.5;
  }

  .mi-path {
    min-width: 0;
    font-family:
      "SFMono-Regular", Consolas, "Liberation Mono", Menlo, monospace;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    opacity: 0.85;
  }

  .mi-tracks {
    list-style: none;
    margin: 2px 0 0;
    padding: 0 0 0 20px;
    display: flex;
    flex-direction: column;
    animation: reveal-in 200ms var(--ease-out);
  }

  @keyframes reveal-in {
    from {
      opacity: 0;
      transform: translateY(-3px);
    }
  }

  .mi-track {
    display: flex;
    align-items: baseline;
    gap: 8px;
    padding: 3px 0 3px 8px;
    font-size: 12px;
    color: var(--text-dim);
  }

  .mi-num {
    flex: none;
    min-width: 2ch;
    text-align: right;
    font-variant-numeric: tabular-nums;
    opacity: 0.7;
  }

  .mi-tname {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--text);
  }

  .mi-tdur {
    flex: none;
    font-variant-numeric: tabular-nums;
    opacity: 0.7;
  }

  .mi-decide {
    display: flex;
    gap: 6px;
  }

  .mi-mark {
    padding: 5px 11px;
    border-radius: 7px;
    border: 1px solid var(--border);
    background: transparent;
    color: var(--text-dim);
    font-size: 12px;
    cursor: pointer;
    transition: background 140ms var(--ease-out), color 140ms var(--ease-out),
      border-color 140ms var(--ease-out);
  }

  .mi-mark:hover:not(:disabled) {
    background: var(--hover);
    color: var(--text);
  }

  .mi-mark:disabled {
    cursor: default;
    opacity: 0.6;
  }

  .mi-mark:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  /* A mark is the affirmative choice, so it takes the accent — the same word
     the footer's primary verb will act on. */
  .mi-save[aria-pressed="true"] {
    background: var(--accent);
    border-color: var(--accent);
    color: var(--accent-text, #fff);
    font-weight: 600;
  }

  /* The destructive mark keeps the app's one destructive tint (the Music
     folders modal already spends it on "Remove"). */
  .mi-drop[aria-pressed="true"] {
    background: #ff8f8f22;
    border-color: #ff8f8f88;
    color: #ff8f8f;
    font-weight: 600;
  }

  .mi-foot {
    margin-top: 12px;
    padding-top: 12px;
    border-top: 1px solid var(--border);
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .mi-sum {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 3px;
    font-size: 12px;
    color: var(--text-dim);
  }

  .mi-counts {
    font-variant-numeric: tabular-nums;
  }

  .mi-line {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .mi-failed {
    color: #ff8f8f;
  }

  .mi-actions {
    display: flex;
    gap: 6px;
  }

  .mi-quiet {
    padding: 6px 10px;
    border-radius: 7px;
    border: 1px solid transparent;
    background: transparent;
    color: var(--text-dim);
    font-size: 12px;
    cursor: pointer;
  }

  .mi-quiet:hover:not(:disabled) {
    background: var(--hover);
    color: var(--text);
  }

  .mi-quiet:disabled {
    cursor: default;
    opacity: 0.6;
  }

  .mi-quiet:focus-visible,
  .mi-apply:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  .mi-apply {
    padding: 7px 14px;
    border: 1px solid var(--accent);
    border-radius: 8px;
    background: var(--accent);
    color: var(--accent-text, #fff);
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
  }

  .mi-apply:disabled {
    opacity: 0.55;
    cursor: default;
  }
</style>
