<script lang="ts">
  import { contextMenu, closeContextMenu } from "../lib/stores/contextMenu.svelte";

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

  function run(action: () => void) {
    closeContextMenu();
    action();
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
    {#each contextMenu.items as item (item.label)}
      <button class="ctx-item" role="menuitem" onclick={() => run(item.action)}>
        {item.label}
      </button>
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
</style>
