<script lang="ts">
  import { contextMenu, closeContextMenu, type MenuItem } from "../lib/stores/contextMenu.svelte";
  import { menuPop } from "../lib/menuPop";

  let el = $state<HTMLDivElement>();

  /* The entrance (menuPop, lib/menuPop.ts) is shared with the tag editors'
   * suggestion list; the transform-origin is pinned to the anchoring corner by
   * the clamp effect below, so the corner nearest the cursor is the one that
   * does not travel. */

  // Keep the menu inside the window once its size is known — and pin the
  // entrance's transform-origin to the corner the click actually anchors
  // (the flip decides which corner that is: a menu pushed left off the
  // right edge grows back to the RIGHT from its top-right corner).
  $effect(() => {
    if (!contextMenu.open || !el) return;
    const r = el.getBoundingClientRect();
    let ox = "left";
    let oy = "top";
    if (contextMenu.x + r.width > window.innerWidth - 4) {
      contextMenu.x = window.innerWidth - r.width - 4;
      ox = "right";
    }
    if (contextMenu.y + r.height > window.innerHeight - 4) {
      contextMenu.y = Math.max(4, contextMenu.y - r.height);
      oy = "bottom";
    }
    el.style.transformOrigin = `${oy} ${ox}`;
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
  <div
    class="menu-surface ctx glass"
    bind:this={el}
    style:top="{contextMenu.y}px"
    style:left="{contextMenu.x}px"
    role="menu"
    transition:menuPop
  >
    <!-- Keyed by index ON PURPOSE: SEP is one shared object, and a menu with
         two breaks would collide under Svelte's default identity keys. -->
    {#each contextMenu.items as item, i (i)}
      {#if item.separator}
        <hr class="menu-sep" role="presentation" />
      {:else}
        <button class="menu-item" role="menuitem" onclick={() => run(item)}>
          {item.label}
        </button>
      {/if}
    {/each}
  </div>
{/if}

<style>
  /* Positioning is all that is left as this component's own: the surface,
   * item, hairline and the hover/active fill live in app.css
   * (`.menu-surface`, `.menu-item`, `.menu-sep`), shared with the tag editors'
   * suggestion list so "looks like our own menus" cannot drift. */
  .ctx {
    position: fixed;
  }
</style>
