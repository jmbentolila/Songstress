<script lang="ts">
  import { ui } from "../lib/stores/ui.svelte";
  import SurfaceClose from "./SurfaceClose.svelte";
  import { tooltip } from "../lib/tooltip";
  import { trapTab } from "../lib/focusTrap";
  import {
    addMusicFolderRoot,
    removeMusicFolderRoot,
    scanner,
  } from "../lib/stores/scanner.svelte";

  let panel = $state<HTMLElement | null>(null);

  /** Which row is currently asking "remove this?" — destructive confirms live
   *  INSIDE the glass; native confirm() would render a GTK dialog, which
   *  AGENTS.md forbids. */
  let pendingRemove = $state<string | null>(null);

  // Closed modal, or the list changed under us (root added/removed elsewhere)
  // → never leave a stale confirmation armed.
  $effect(() => {
    void ui.musicFoldersOpen;
    void ui.musicFolders.length;
    pendingRemove = null;
  });

  function confirmRemove() {
    const path = pendingRemove;
    pendingRemove = null;
    if (path) void removeMusicFolderRoot(path);
  }

  function add() {
    void addMusicFolderRoot();
  }

  // --- the outro ------------------------------------------------------------
  // Same shape as About.svelte: `out` puts the class on the scrim, and the OPEN flag
  // stays true until the animation has ended — so `modalOpen()` (which feeds `inert`
  // on `.stage`) and this modal's own Escape ownership both keep telling the truth
  // while the dialog is still on screen. The second press force-closes, which is the
  // backstop for an `animationend` that never arrives.
  let out = $state(false);

  function close() {
    if (out) {
      ui.musicFoldersOpen = false;
      out = false;
      return;
    }
    out = true;
  }

  function onOutroEnd(e: AnimationEvent) {
    if (e.animationName !== "scrim-out") return;
    ui.musicFoldersOpen = false;
    out = false;
  }
</script>

<svelte:window
  onkeydown={(e) => {
    if (!ui.musicFoldersOpen) return;
    if (e.key === "Tab") {
      trapTab(e, panel);
      return;
    }
    if (e.key === "Escape") close();
  }}
/>

{#if ui.musicFoldersOpen}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div
    class="mf-backdrop scrim"
    class:out
    role="presentation"
    onanimationend={onOutroEnd}
    onclick={(e) => e.target === e.currentTarget && close()}
  >
    <!-- svelte-ignore a11y_no_noninteractive_element_to_interactive_role -->
    <section class="mf-modal glass" bind:this={panel} role="dialog" aria-modal="true" aria-label="Music folders">
      <header class="mf-head">
        <h2>Music folders</h2>
        <!-- Dismissal is the boxed ✕ at the far edge, like every other surface.
             (It used to be a font-glyph ✕ here — the objection was the glyph's
             family, not its side: a text ✕ is a third typographic voice in a
             glass surface, and this header now draws the same cross the sidebar
             gear morphs into.) -->
        <SurfaceClose label="Close" onclick={close} />
      </header>

      <div class="mf-body">
        {#if ui.musicFolders.length === 0}
          <p class="mf-empty">No music folders. Add one to build your library.</p>
        {:else}
          <ul class="mf-list">
            {#each ui.musicFolders as folder (folder)}
              {#if pendingRemove === folder}
                <li class="mf-row mf-ask-row">
                  <span class="mf-ask" role="alert">
                    Drop this folder’s tracks from the library? Files on disk are
                    kept.
                  </span>
                  <button
                    class="mf-yes"
                    disabled={scanner.running}
                    onclick={confirmRemove}
                  >
                    Remove
                  </button>
                  <button class="mf-no" onclick={() => (pendingRemove = null)}>
                    Cancel
                  </button>
                </li>
              {:else}
                <li class="mf-row">
                  <span class="mf-path" use:tooltip={folder}>{folder}</span>
                  <button
                    class="mf-remove"
                    aria-label={`Remove ${folder}`}
                    disabled={scanner.running}
                    onclick={() => (pendingRemove = folder)}
                    use:tooltip={"Remove this folder (tracks drop from the library)"}
                  >
                    ✕
                  </button>
                </li>
              {/if}
            {/each}
          </ul>
        {/if}
      </div>

      <footer class="mf-foot">
        <button class="mf-btn mf-add" disabled={scanner.running} onclick={add}>
          {scanner.running ? "Adding…" : "Add folder…"}
        </button>
      </footer>
    </section>
  </div>
{/if}

<style>
  .mf-modal {
    width: min(560px, calc(100vw - 80px));
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

  .mf-head {
    display: flex;
    align-items: center;
    gap: 12px;
    padding-bottom: 10px;
    border-bottom: 1px solid var(--border);
  }

  .mf-head h2 {
    flex: 1;
    margin: 0;
    font-size: 17px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .mf-body {
    overflow-y: auto;
    padding: 8px 0;
  }

  .mf-empty {
    margin: 0;
    padding: 14px 6px;
    font-size: 15px;
    color: var(--text-dim);
    text-align: center;
  }

  .mf-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .mf-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 7px 8px;
    border-radius: 7px;
    background: var(--hover);
  }

  .mf-path {
    flex: 1;
    min-width: 0;
    font-size: 15px;
    font-family:
      "SFMono-Regular", Consolas, "Liberation Mono", Menlo, monospace;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .mf-remove {
    flex: none;
    width: 22px;
    height: 22px;
    border: none;
    /* 7px, the icon-button rung (5 was off the ladder; Toggle's 5px stays — a 16px
       checkbox is a different object and DESIGN.md says so). */
    border-radius: 7px;
    background: transparent;
    color: var(--text-dim);
    cursor: pointer;
    font-size: 14px;
    line-height: 1;
  }

  .mf-remove:hover {
    background: var(--hover);
    color: var(--caution);
  }

  .mf-ask-row {
    gap: 10px;
    background: var(--panel-bg-strong);
    border: 1px solid var(--caution-line);
  }

  .mf-ask {
    flex: 1;
    min-width: 0;
    font-size: 14px;
    line-height: 1.35;
    color: var(--text-dim);
  }

  .mf-yes,
  .mf-no {
    flex: none;
    padding: 5px 11px;
    border-radius: 7px;
    border: 1px solid var(--border);
    background: transparent;
    color: var(--text);
    font-size: 14px;
    cursor: pointer;
  }

  .mf-yes {
    border-color: var(--caution-line);
    color: var(--caution);
  }

  .mf-yes:hover:not(:disabled) {
    background: var(--caution-wash);
  }

  .mf-no:hover {
    background: var(--hover);
  }

  .mf-yes:disabled {
    opacity: 0.55;
    cursor: default;
  }

  .mf-foot {
    margin-top: 10px;
    padding-top: 10px;
    border-top: 1px solid var(--border);
    justify-content: flex-end;
  }

  .mf-add {
    padding: 7px 14px;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--accent);
    /* --accent-text, not #fff: with a light accent ("White") white text on a
       white fill is no text at all. */
    color: var(--accent-text, #fff);
    font-size: 15px;
    cursor: pointer;
  }

  .mf-add:disabled {
    opacity: 0.6;
    cursor: default;
  }
</style>
