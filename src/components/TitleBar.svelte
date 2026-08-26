<script lang="ts">
  import { windowClose, windowMinimize, windowToggleMaximize } from "../lib/window";
  import { decoState, loadDecoration } from "../lib/stores/decoration.svelte";
  import { library } from "../lib/stores/library.svelte";
  import { initScanner } from "../lib/stores/scanner.svelte";
  import { menu, activateMenuItem, initMenu, type MenuItem as MenuEntry } from "../lib/stores/menu.svelte";
  import { ui } from "../lib/stores/ui.svelte";

  loadDecoration();
  initScanner();
  void initMenu();

  /** Open top-level menu of the in-titlebar menu bar ("playback" etc.). */
  let barOpenId = $state<string | null>(null);

  let searchEl = $state<HTMLInputElement | null>(null);

  function searchKey(e: KeyboardEvent) {
    if (e.key === "Escape") {
      ui.mediaFilter = "";
      searchEl?.blur();
    }
  }

  function clearSearch() {
    ui.mediaFilter = "";
    searchEl?.focus();
  }

  function openBar(id: string) {
    barOpenId = barOpenId === id ? null : id;
  }

  function hoverBar(id: string) {
    // Classic menubar behavior: hovering another top-level while one is
    // open switches to it.
    if (barOpenId !== null) barOpenId = id;
  }

  function barItem(item: MenuEntry) {
    if (!item.enabled) return;
    barOpenId = null;
    activateMenuItem(item.id);
  }

  type Kind = "X" | "I" | "A";

  const ACTION: Record<Kind, () => void> = {
    X: windowClose,
    I: windowMinimize,
    A: windowToggleMaximize,
  };
  const LABEL: Record<Kind, string> = { X: "Close", I: "Minimize", A: "Maximize" };
  const CLASS: Record<Kind, string> = { X: "tb-close", I: "tb-min", A: "tb-max" };

  // Honor the user's KWin button order; letters we can't implement as a CSD
  // app (Shade, Keep Above, ...) are skipped rather than rendered dead.
  const layout = $derived(
    (decoState.value?.buttonsLeft ?? "XIA")
      .split("")
      .filter((c): c is Kind => c === "X" || c === "I" || c === "A"),
  );

  const chromeVars = $derived.by(() => {
    const d = decoState.value;
    const op = (d?.bgOpacityActive ?? 100) / 100;
    const rgba = (c: [number, number, number]) =>
      `rgba(${c[0]}, ${c[1]}, ${c[2]}, ${op})`;
    const close = d?.close ?? { normal: [255, 95, 87], hover: [195, 63, 69] };
    const min = d?.minimize ?? { normal: [254, 188, 46], hover: [218, 165, 5] };
    const max = d?.maximize ?? { normal: [88, 251, 63], hover: [38, 148, 62] };
    return [
      `--tb-x: ${rgba(close.normal)}`,
      `--tb-x-hover: ${rgba(close.hover)}`,
      `--tb-i: ${rgba(min.normal)}`,
      `--tb-i-hover: ${rgba(min.hover)}`,
      `--tb-a: ${rgba(max.normal)}`,
      `--tb-a-hover: ${rgba(max.hover)}`,
    ].join("; ");
  });
</script>

<header
  class="tb-root glass"
  data-tauri-drag-region
  style={chromeVars}
  role="none"
  onfocusout={(e) => {
    if (!e.currentTarget.contains(e.relatedTarget as Node)) {
      barOpenId = null;
    }
  }}
  onkeydown={(e) => {
    if (e.key === "Escape") {
      barOpenId = null;
    }
  }}
>
  <div class="tb-traffic" role="group" aria-label="Window controls">
    {#each layout as kind (kind)}
      <button
        class={`tb-light ${CLASS[kind]}`}
        aria-label={`${LABEL[kind]} window`}
        onclick={ACTION[kind]}
      >
        <svg viewBox="0 0 10 10">
          {#if kind === "X"}
            <path d="M2.2 2.2 L7.8 7.8 M7.8 2.2 L2.2 7.8" />
          {:else if kind === "I"}
            <path d="M1.8 5 L8.2 5" />
          {:else}
            <path d="M2 6 L2 8 L4 8 M8 4 L8 2 L6 2 M2 8 L4.2 5.8 M8 2 L5.8 4.2" />
          {/if}
        </svg>
      </button>
    {/each}
  </div>

  {#if library.live && !menu.globalMenuActive}
    <nav class="tb-menubar" aria-label="Application menu bar">      {#each menu.menus as top (top.id)}
        <div class="tb-menubar-wrap">
          <button
            class="tb-menubar-btn"
            aria-expanded={barOpenId === top.id}
            onclick={() => openBar(top.id)}
            onmouseenter={() => hoverBar(top.id)}
          >
            {top.label}
          </button>
          {#if barOpenId === top.id}
            <div class="tb-pop glass" role="menu">
              {#each top.items as item (item.id)}
                <button
                  class="tb-item"
                  role={item.checked !== null ? "menuitemcheckbox" : "menuitem"}
                  aria-checked={item.checked ?? undefined}
                  disabled={!item.enabled}
                  onclick={() => barItem(item)}
                >
                  <span class="tb-item-check">
                    {#if item.checked === true}✓{/if}
                  </span>
                  {item.label}
                </button>
              {/each}
            </div>
          {/if}
        </div>
      {/each}
    </nav>
  {/if}

  <div class="tb-balance" data-tauri-drag-region></div>

  {#if library.live}
    <div class="tb-search" role="search">
      <svg viewBox="0 0 16 16" aria-hidden="true">
        <circle cx="7" cy="7" r="4.4" fill="none" stroke="currentColor" stroke-width="1.4" />
        <path d="M10.4 10.4 L13.6 13.6" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
      </svg>
      <input
        bind:this={searchEl}
        bind:value={ui.mediaFilter}
        onkeydown={searchKey}
        type="text"
        placeholder="Search albums & songs..."
        spellcheck="false"
        aria-label="Search albums and songs"
      />
      {#if ui.mediaFilter !== ""}
        <button class="tb-search-clear" aria-label="Clear search" onclick={clearSearch}>
          <svg viewBox="0 0 10 10">
            <path d="M2.2 2.2 L7.8 7.8 M7.8 2.2 L2.2 7.8" />
          </svg>
        </button>
      {/if}
    </div>
  {/if}
</header>

<style>
  .tb-root {
    position: relative;
    /* Above Sidebar (z-10) and its gear popover (z-30): .stage creates no
       stacking context, so those compete at the root level and previously
       painted over the app menu — rendering it click-through-dead. */
    z-index: 40;
    display: flex;
    align-items: center;
    gap: 12px;
    height: var(--titlebar-h);
    flex: none;
    padding: 0 14px;
    border-bottom: 1px solid var(--border);
    user-select: none;
  }

  .tb-traffic {
    display: flex;
    gap: 8px;
  }

  .tb-light {
    position: relative;
    width: 13px;
    height: 13px;
    border-radius: 50%;
    border: 1px solid rgba(0, 0, 0, 0.18);
    padding: 0;
    cursor: pointer;
  }

  /* Absolute centering — place-items:center on a native <button> drifts
     ~1px down in WebKitGTK (shadow-DOM layout), which read as off-glyphs. */
  .tb-light svg {
    position: absolute;
    top: 50%;
    left: 50%;
    translate: -50% -50%;
    width: 8px;
    height: 8px;
    /* Klassy draws button icons in the titlebar text color — white here. */
    stroke: rgba(255, 255, 255, 0.9);
    stroke-width: 1.4;
    stroke-linecap: round;
    fill: none;
    opacity: 0;
  }

  /* Palette comes from the user's KWin decoration via --tb-* custom props
     (set on .tb-root from kde_window_decoration). */
  .tb-close { background: var(--tb-x); }
  .tb-close:hover { background: var(--tb-x-hover); }
  .tb-min   { background: var(--tb-i); }
  .tb-min:hover { background: var(--tb-i-hover); }
  .tb-max   { background: var(--tb-a); }
  .tb-max:hover { background: var(--tb-a-hover); }

  /* Glyph appears only on the button actually hovered (Klassy behavior). */
  .tb-light:hover svg {
    opacity: 1;
  }

  .tb-balance {
    flex: 1;
  }

  /* --- in-titlebar menu bar (Step 3) --- */
  .tb-menubar {
    display: flex;
    align-items: center;
    gap: 2px;
  }

  .tb-menubar-wrap {
    position: relative;
  }

  .tb-menubar-btn {
    border: none;
    background: transparent;
    color: var(--text);
    font-size: 12.5px;
    padding: 4px 9px;
    border-radius: 6px;
    cursor: pointer;
    white-space: nowrap;
  }

  .tb-menubar-btn:hover,
  .tb-menubar-btn[aria-expanded="true"] {
    background: var(--hover);
  }

  .tb-menubar .tb-pop {
    left: 0;
  }

  .tb-item-check {
    display: inline-block;
    width: 14px;
    flex: none;
  }

  /* --- albums & songs search (right end) --- */
  .tb-search {
    position: relative;
    flex: none;
  }

  .tb-search svg {
    position: absolute;
    left: 9px;
    top: 50%;
    width: 13px;
    height: 13px;
    transform: translateY(-50%);
    color: var(--text-dim);
    pointer-events: none;
  }

  .tb-search input {
    width: 210px;
    height: 28px;
    padding: 0 26px 0 28px;
    border-radius: 8px;
    border: 1px solid var(--border);
    background: var(--hover);
    color: var(--text);
    font-size: 12.5px;
    outline: none;
    transition: width 160ms ease, border-color 120ms ease;
  }

  .tb-search input:focus {
    border-color: var(--accent);
    width: 260px;
  }

  .tb-search input::placeholder {
    color: var(--text-dim);
  }

  .tb-search-clear {
    position: absolute;
    right: 5px;
    top: 50%;
    translate: 0 -50%;
    display: grid;
    place-items: center;
    width: 18px;
    height: 18px;
    border: none;
    border-radius: 50%;
    background: transparent;
    color: var(--text-dim);
    cursor: pointer;
  }

  .tb-search-clear:hover {
    background: var(--hover);
    color: var(--text);
  }

  .tb-search-clear svg {
    width: 8px;
    height: 8px;
    stroke: currentColor;
    stroke-width: 1.4;
    stroke-linecap: round;
    fill: none;
  }

  .tb-pop {
    position: absolute;
    top: calc(100% + 8px);
    left: 0;
    z-index: 100;
    min-width: 210px;
    padding: 6px;
    border-radius: 10px;
    border: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .tb-item {
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

  .tb-item:hover:not(:disabled) {
    background: var(--hover);
  }

  .tb-item:disabled {
    color: var(--text-dim);
    cursor: default;
  }
</style>
