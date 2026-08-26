<script lang="ts">
  import { ui } from "../lib/stores/ui.svelte";
  import {
    addMusicFolderRoot,
    removeMusicFolderRoot,
    scanner,
  } from "../lib/stores/scanner.svelte";

  function remove(path: string) {
    if (
      !confirm(
        `Remove "${path}" from your music folders?\n\n` +
          "Its tracks will be dropped from the library. Files on disk are kept.",
      )
    ) {
      return;
    }
    void removeMusicFolderRoot(path);
  }

  function add() {
    void addMusicFolderRoot();
  }
</script>

<svelte:window
  onkeydown={(e) => {
    if (e.key === "Escape" && ui.musicFoldersOpen) ui.musicFoldersOpen = false;
  }}
/>

{#if ui.musicFoldersOpen}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div
    class="mf-backdrop"
    role="presentation"
    onclick={(e) => e.target === e.currentTarget && (ui.musicFoldersOpen = false)}
  >
    <section class="mf-modal glass" role="dialog" aria-modal="true" aria-label="Music folders">
      <header class="mf-head">
        <h2>Music folders</h2>
        <button class="mf-close" aria-label="Close" onclick={() => (ui.musicFoldersOpen = false)}>✕</button>
      </header>

      <div class="mf-body">
        {#if ui.musicFolders.length === 0}
          <p class="mf-empty">No music folders. Add one to build your library.</p>
        {:else}
          <ul class="mf-list">
            {#each ui.musicFolders as folder (folder)}
              <li class="mf-row">
                <span class="mf-path" title={folder}>{folder}</span>
                <button
                  class="mf-remove"
                  aria-label={`Remove ${folder}`}
                  disabled={scanner.running}
                  onclick={() => remove(folder)}
                  title="Remove this folder (tracks drop from the library)"
                >
                  ✕
                </button>
              </li>
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
  .mf-backdrop {
    position: fixed;
    inset: 0;
    z-index: 150;
    display: grid;
    place-items: center;
    background: rgba(0, 0, 0, 0.35);
  }

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
    font-size: 15px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .mf-close {
    width: 24px;
    height: 24px;
    border: none;
    border-radius: 6px;
    background: transparent;
    color: var(--text-dim);
    cursor: pointer;
  }

  .mf-close:hover {
    background: var(--hover);
    color: var(--text);
  }

  .mf-body {
    overflow-y: auto;
    padding: 8px 0;
  }

  .mf-empty {
    margin: 0;
    padding: 14px 6px;
    font-size: 13px;
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
    font-size: 13px;
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
    border-radius: 5px;
    background: transparent;
    color: var(--text-dim);
    cursor: pointer;
    font-size: 12px;
    line-height: 1;
  }

  .mf-remove:hover {
    background: var(--hover);
    color: #ff8f8f;
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
    color: #fff;
    font-size: 13px;
    cursor: pointer;
  }

  .mf-add:disabled {
    opacity: 0.6;
    cursor: default;
  }
</style>
