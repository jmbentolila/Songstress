<script lang="ts">
  import { contextMenu, closeContextMenu, type MenuItem } from "../lib/stores/contextMenu.svelte";

  let el = $state<HTMLDivElement>();

  // Keep the menu inside the window once its size is known.
  $effect(() => {
    if (!contextMenu.open || !el) return;
    const r = el.getBoundingClientRect();
    if (contextMenu.x + r.width > window.innerWidth - 4) {
      contextMenu.x = window.innerWidth - r.width - 4;
    }
    if (contextMenu.y + r.height > window.innerHeight - 4) {
      contextMenu.y = Math.max(4, contextMenu.y - r.height);
    }
  });

  // The menu is an overlay anchored to screen coordinates — the moment the
  // world moves under it or attention leaves it, its anchor is a lie, so it
  // goes away (2026-09-03). Scroll is captured on the whole document because
  // the grid scrolls in its own container, not the window; focusin covers
  // Tab/walk-away inside the document (an outside pointerdown is already
  // handled by the listener below); and blur covers focus leaving the
  // WINDOW — a panel-menu click sends us no DOM event at all, and without
  // it a stale menu would sit floating over a library the user walked away
  // from. No open-time race: the opener's mousedown focus lands BEFORE the
  // contextmenu event registers this effect, and scroll/blur can't fire
  // from opening a menu.
  $effect(() => {
    if (!contextMenu.open || !el) return;
    const box = el; // narrowed non-undefined handle for the closures below
    const close = () => closeContextMenu();
    const onFocusIn = (e: FocusEvent) => {
      if (!box.contains(e.target as Node)) closeContextMenu();
    };
    window.addEventListener("scroll", close, { capture: true, passive: true });
    window.addEventListener("blur", close);
    document.addEventListener("focusin", onFocusIn);
    return () => {
      window.removeEventListener("scroll", close, { capture: true });
      window.removeEventListener("blur", close);
      document.removeEventListener("focusin", onFocusIn);
    };
  });

  function run(item: MenuItem) {
    closeContextMenu();
    item.action?.();
  }
</script>

<svelte:document
  onpointerdown={(e) => {
    if (contextMenu.open && !el?.contains(e.target as Node)) closeContextMenu();
  }}
/>

<!-- Escape belongs to the menu while it is open (surfaces.svelte.ts counts it as a
     surface, so the sidebar's router stands down for it). It had no Escape owner at
     all before: the key popped a settings level instead of dismissing the thing on
     screen, which is the wrong verb twice over. -->
<svelte:window
  onkeydown={(e) => {
    if (e.key === "Escape" && contextMenu.open) closeContextMenu();
  }}
/>

{#if contextMenu.open}
  <div class="ctx glass" bind:this={el} style:top="{contextMenu.y}px" style:left="{contextMenu.x}px" role="menu">
    <!-- Keyed by index ON PURPOSE: SEP is one shared object, and a menu with
         two breaks would collide under Svelte's default identity keys. -->
    {#each contextMenu.items as item, i (i)}
      {#if item.separator}
        <hr class="ctx-sep" role="presentation" />
      {:else}
        <button class="ctx-item" role="menuitem" onclick={() => run(item)}>
          {item.label}
        </button>
      {/if}
    {/each}
  </div>
{/if}

<style>
  .ctx {
    position: fixed;
    z-index: 200;
    min-width: 180px;
    padding: 6px;
    border-radius: 10px;
    border: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .ctx-item {
    text-align: left;
    padding: 8px 12px;
    border: none;
    border-radius: 7px;
    background: transparent;
    color: var(--text);
    font-size: 13px;
    cursor: pointer;
    white-space: nowrap;
  }

  .ctx-item:hover {
    background: var(--hover);
  }

  /* The menu's section hairline — same quiet line the glass surfaces use,
   * inset so it reads as a break between rows, not a border of the box. */
  .ctx-sep {
    height: 1px;
    margin: 4px 8px;
    border: none;
    background: var(--border);
  }
</style>
