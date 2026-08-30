<script lang="ts">
  import { ui, cycleTheme, resolvedTheme } from "../lib/stores/ui.svelte";
  import { library } from "../lib/stores/library.svelte";
  import { ACCENT_PRESETS } from "../lib/accent";
  import { menu, activateMenuItem } from "../lib/stores/menu.svelte";
  import { scanner } from "../lib/stores/scanner.svelte";
  import { fold } from "../lib/search";
  import {
    windowClose,
    windowMinimize,
    windowToggleMaximize,
  } from "../lib/window";
  import { decoState, loadDecoration } from "../lib/stores/decoration.svelte";

  loadDecoration();

  // --- menu stack (Step 8b, iOS Settings-style) ----------------------------
  // Three layers in .stack, sliding on transform only (compositor):
  //   home (search + artists) → root ("Settings") → detail (pane).
  // The pushed root recedes -30% (iOS parallax) instead of leaving.
  // Real panes are the Rust-owned model (menu.svelte); "appearance" is
  // frontend-only (the old gear-popover settings).
  const APPEARANCE = "appearance";

  let inMenu = $derived(ui.menuOpen);
  let atDetail = $derived(ui.menuDetail !== null);
  let detailMenu = $derived(
    atDetail ? menu.menus.find((m) => m.id === ui.menuDetail) ?? null : null,
  );
  let detailItems = $derived(detailMenu?.items ?? []);
  let detailTitle = $derived(
    ui.menuDetail === APPEARANCE ? "Appearance" : (detailMenu?.label ?? "Settings"),
  );

  function openDetail(id: string) {
    ui.menuDetail = id;
  }
  function back() {
    ui.menuDetail = null;
  }
  function closeSettings() {
    ui.menuDetail = null;
    ui.menuOpen = false;
  }
  // Full-pop close choreography: root exits LEFT, parallel to home
  // arriving at the same speed (no crossing, no 2× sweep, no centered
  // main-menu intermediate). While off-screen at -200% it then teleports
  // back to its +100% entry slot (transition suppressed one frame) so the
  // next open re-enters from the right.
  let rootNoAnim = $state(false);
  let closeTimer = 0;
  function toggleSettings() {
    if (!ui.menuOpen) {
      ui.menuOpen = true;
      ui.menuDetail = null;
    } else {
      ui.menuOpen = false;
      if (ui.menuDetail) {
        clearTimeout(closeTimer);
        closeTimer = setTimeout(() => {
          if (ui.menuOpen) return; // re-opened mid-close: keep the state
          ui.menuDetail = null;
          rootNoAnim = true;
          requestAnimationFrame(() =>
            requestAnimationFrame(() => (rootNoAnim = false)),
          );
        }, 340);
      }
    }
  }

  // The native color input needs a concrete value; presets leave it alone.
  let customHex = $state(ui.accentColor ?? "#ff6ec7");

  const albumCounts = $derived.by(() => {
    const m = new Map<string, number>();
    for (const a of library.albums) {
      m.set(a.artistId, (m.get(a.artistId) ?? 0) + 1);
    }
    return m;
  });

  // One field drives both surfaces: the artist list here and the
  // Songs/Albums sections in the grid. Diacritic-insensitive via fold()
  // — same matching rule as the grid ("bjork" finds Björk in both).
  const filtered = $derived.by(() => {
    const q = ui.search.trim();
    if (q === "") return library.artists;
    const fq = fold(q);
    return library.artists.filter((a) => fold(a.name).includes(fq));
  });

  function select(id: string) {
    ui.activeArtistId = id;
    ui.expandedAlbum.songs = null;
    ui.expandedAlbum.albums = null;
    ui.search = "";
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
  // Keyboard accelerators (critique P3): `/` focuses the library search,
  // `s` toggles the Settings stack — both inert while typing in any field.
  let searchInput = $state<HTMLInputElement | null>(null);

  function onGlobalKey(e: KeyboardEvent) {
    // Escape pops one menu level: detail → root → home.
    if (e.key === "Escape" && ui.menuOpen) {
      if (ui.menuDetail) ui.menuDetail = null;
      else ui.menuOpen = false;
      return;
    }
    const t = e.target as HTMLElement | null;
    const typing =
      !!t &&
      (t.tagName === "INPUT" ||
        t.tagName === "TEXTAREA" ||
        t.tagName === "SELECT" ||
        t.isContentEditable);
    if (typing || e.ctrlKey || e.metaKey || e.altKey) return;
    if (e.key === "/" && !ui.menuOpen) {
      e.preventDefault();
      searchInput?.focus();
    } else if (e.key === "s") {
      toggleSettings();
    }
  }
</script>

<svelte:window onkeydown={onGlobalKey} />

<aside class="sidebar glass">
  <header class="head" data-tauri-drag-region style={chromeVars}>
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
    <button
      class="gear"
      aria-label={ui.menuOpen ? "Close settings" : "Open settings"}
      title="Settings — press s to toggle"
      aria-expanded={ui.menuOpen}
      onclick={toggleSettings}
    >
      <svg class="icon icon-gear" class:show={!ui.menuOpen} viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.3">
        <circle cx="8" cy="8" r="2.2" />
        <path
          d="M8 1.8 v1.9 M8 12.3 v1.9 M14.2 8 h-1.9 M3.7 8 H1.8 M12.5 3.5 l-1.35 1.35 M4.85 11.15 L3.5 12.5 M12.5 12.5 l-1.35 -1.35 M4.85 4.85 L3.5 3.5"
          stroke-linecap="round"
        />
      </svg>
      <svg class="icon icon-x" class:show={ui.menuOpen} viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round">
        <path d="M4 4 L12 12 M12 4 L4 12" />
      </svg>
    </button>
  </header>

  <div class="stack">
    <!-- home: search + artists (the sidebar's normal job) -->
    <div class="layer" class:off={inMenu}>
      <div class="search">
        <svg viewBox="0 0 16 16" aria-hidden="true">
          <circle cx="7" cy="7" r="4.4" fill="none" stroke="currentColor" stroke-width="1.4" />
          <path d="M10.4 10.4 L13.6 13.6" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
        </svg>
        <input
          type="text"
          placeholder="Search library..."
          spellcheck="false"
          aria-label="Search library"
          title="Tip: press / anywhere to focus search — Escape clears"
          bind:this={searchInput}
          bind:value={ui.search}
          onkeydown={(e) => {
            // Escape in the field: clear if there's a query, else blur —
            // the old titlebar search's behavior, kept through the merge.
            if (e.key === "Escape") {
              if (ui.search) ui.search = "";
              else searchInput?.blur();
            }
          }}
        />
        {#if ui.search !== ""}
          <button class="search-clear" aria-label="Clear search" onclick={() => (ui.search = "")}>
            <svg viewBox="0 0 10 10"><path d="M2.2 2.2 L7.8 7.8 M7.8 2.2 L2.2 7.8" /></svg>
          </button>
        {/if}
      </div>

      {#if library.albums.length > 0 && scanner.running}
        <p class="scan-note" aria-live="polite">Updating library…</p>
      {/if}

      <nav class="scroll">
        <!-- Always visible: it's a control (clear selection + total count),
             not a search result — filtering it out made a zero-match query
             a dead end (critique P1). -->
        <button
          class="row"
          class:active={ui.activeArtistId === "all"}
          style:height="var(--sidebar-row-size)"
          onclick={() => select("all")}
        >
          <span class="name">All Artists</span>
          <span class="count">{library.albums.length}</span>
        </button>
        {#each filtered as artist (artist.id)}
          <button
            class="row"
            class:active={ui.activeArtistId === artist.id}
            style:height="var(--sidebar-row-size)"
            onclick={() => select(artist.id)}
          >
            <span class="name">{artist.name}</span>
            <span class="count">{albumCounts.get(artist.id) ?? 0}</span>
          </button>
        {/each}
        {#if filtered.length === 0 && ui.search.trim() !== ""}
          <!-- Action, not caption: a zero-match state that offers no remedy
               is a dead end (critique P3). -->
          <button class="empty" onclick={() => (ui.search = "")}>
            No artists match — clear search
          </button>
        {/if}
      </nav>
    </div>

    <!-- menu root: top-level panes -->
    <div
      class="layer root"
      class:active={inMenu && !atDetail}
      class:receded={inMenu && atDetail}
      class:exiting={!inMenu && atDetail}
      class:no-anim={rootNoAnim}
      aria-hidden={!(inMenu && !atDetail)}
    >
      <div class="navrow">
        <button class="backchev" aria-label="Back to library" onclick={closeSettings}>
          <svg viewBox="0 0 10 14" aria-hidden="true"><path d="M7 2 L3 7 L7 12" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" /></svg>
        </button>
        <span class="navtitle">Settings</span>
      </div>
      <nav class="scroll">
        <button class="mrow" onclick={() => openDetail(APPEARANCE)}>
          <span class="name">Appearance</span>
        </button>
        {#each menu.menus as top (top.id)}
          <button class="mrow" onclick={() => openDetail(top.id)}>
            <span class="name">{top.label}</span>
          </button>
        {/each}
      </nav>
    </div>

    <!-- detail: one pane at a time -->
    <div class="layer detail" class:active={inMenu && atDetail} aria-hidden={!(inMenu && atDetail)}>
      <div class="navrow">
        <button class="backchev" aria-label="Back to settings" onclick={back}>
          <svg viewBox="0 0 10 14" aria-hidden="true"><path d="M7 2 L3 7 L7 12" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" /></svg>
        </button>
        <span class="navtitle">{detailTitle}</span>
      </div>
      {#if ui.menuDetail === APPEARANCE}
        <div class="appearance">
          <label>
            <span>Tile size</span>
            <input
              type="range" min="120" max="320" step="4"
              value={ui.tileSize}
              oninput={(e) => (ui.tileSize = +e.currentTarget.value)}
            />
          </label>
          <label>
            <span>Sidebar rows</span>
            <input
              type="range" min="28" max="52" step="2"
              value={ui.sidebarRowSize}
              oninput={(e) => (ui.sidebarRowSize = +e.currentTarget.value)}
            />
          </label>
          <button class="theme" onclick={cycleTheme}>
            Theme: {resolvedTheme() === "dark" ? "Dark" : "Light"}
          </button>
          <label class="toggle">
            <input type="checkbox" bind:checked={ui.playbarGradient} />
            <span>Playbar artwork gradient</span>
          </label>
          <div class="accent">
            <span>Accent</span>
            <div class="swatches">
              {#each ACCENT_PRESETS as p (p.name)}
                <button
                  class="swatch"
                  class:selected={ui.accentColor === p.hex}
                  style:background={p.hex ?? "linear-gradient(135deg, #a78bfa 50%, #7c58f0 50%)"}
                  title={p.name}
                  aria-label={p.name}
                  onclick={() => (ui.accentColor = p.hex)}
                ></button>
              {/each}
              <label
                class="swatch custom"
                class:selected={ui.accentColor !== null && !ACCENT_PRESETS.some((p) => p.hex === ui.accentColor)}
                title="Custom color"
              >
                <input
                  type="color"
                  bind:value={customHex}
                  oninput={(e) => (ui.accentColor = e.currentTarget.value)}
                />
              </label>
            </div>
          </div>
        </div>
      {:else}
        <div class="scroll items">
          {#each detailItems as item (item.id)}
            <button
              class="irow"
              disabled={!item.enabled}
              onclick={() => activateMenuItem(item.id)}
            >
              <span class="check">{item.checked === true ? "✓" : ""}</span>
              <span class="name">{item.label}</span>
            </button>
          {/each}
        </div>
      {/if}
    </div>
  </div>
</aside>

<style>
  .sidebar {
    position: absolute;
    top: 0;
    left: 0;
    bottom: var(--playbar-h);
    z-index: 10;
    display: flex;
    flex-direction: column;
    width: var(--sidebar-width);
    /* border-bottom was doubling the seam: the playbar's own border-top
       already runs full width underneath, so sidebar-bottom + playbar-top
       read as a 2px line along the sidebar only. */
    border-right: 1px solid var(--border);
    user-select: none;
  }

  .head {
    position: relative;
    flex: none;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 16px 10px 10px 14px;
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
    stroke: rgba(255, 255, 255, 0.9);
    stroke-width: 1.4;
    stroke-linecap: round;
    fill: none;
    opacity: 0;
  }

  /* Palette comes from the user's KWin decoration via --tb-* custom props
     (set on .head from kde_window_decoration). */
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

  .gear {
    position: relative;
    flex: none;
    border: none;
    background: transparent;
    color: var(--text-dim);
    width: 28px;
    height: 28px;
    border-radius: 7px;
    display: grid;
    place-items: center;
    cursor: pointer;
    padding: 0;
  }

  .gear:hover,
  .gear[aria-expanded="true"] {
    background: var(--hover);
    color: var(--text);
  }

  .gear:active {
    background: var(--active);
  }

  /* Icon morph: gear and ✕ crossfade while rotating — a state change,
     not a swap. 160ms ease-out (fast, purposeful). */
  .gear .icon {
    position: absolute;
    top: 50%;
    left: 50%;
    width: 16px;
    height: 16px;
    translate: -50% -50%;
    transition:
      opacity 160ms ease-out,
      rotate 160ms ease-out;
  }

  .gear .icon:not(.show) {
    opacity: 0;
  }

  .gear .icon-gear:not(.show) {
    rotate: 90deg;
  }

  .gear .icon-x:not(.show) {
    rotate: -90deg;
  }

  /* --- menu stack (iOS Settings-style) -----------------------------------
     Layers are absolute and slide on transform ONLY (compositor-friendly,
     interruptible by state changes without layout). Apple's standard curve.
     Off-screen layers keep pointer-events off; the receded root is a
     visual-only parallax, not interactive. */
  .stack {
    position: relative;
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }

  .layer {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    pointer-events: none;
    transition: transform 320ms var(--ease-drawer);
  }

  /* On-screen layers are interactive; off-screen ones aren't. (The home
     layer once had no rule at all and the artist list went click-dead —
     layers default to pointer-events: none up there.) Off-screen
     siblings are fully outside .stack's clip, so auto never over-captures. */
  .layer:not(.off) {
    pointer-events: auto;
  }

  /* home: in at 0, out to the LEFT (it was pushed) */
  .layer.off {
    transform: translateX(-100%);
  }

  /* root: in from the RIGHT, at 0 when active, and a FULL push left when a
     detail is open (the -30% iOS parallax recede was tried first — on a flat
     glass sidebar the peeking root items read as clutter, user rejected it). */
  .layer.root {
    transform: translateX(100%);
    z-index: 1;
  }

  .layer.root.active {
    transform: translateX(0);
  }

  .layer.root.receded {
    transform: translateX(-100%);
  }

  /* full-pop close: root exits further LEFT, in parallel with home sliding
     in — same speed, never crossing the home layer */
  .layer.root.exiting {
    transform: translateX(-200%);
  }

  /* one-frame transition suppression while the off-screen root teleports
     from -200% back to its +100% entry slot */
  .layer.root.no-anim {
    transition: none;
  }

  /* detail: in from the RIGHT, on top */
  .layer.detail {
    transform: translateX(100%);
    z-index: 2;
  }

  .layer.detail.active {
    transform: translateX(0);
  }

  /* --- nav row (per-layer title bar; wayfinding per the iOS pattern) ---
     No divider under the title (2026-08-27, user decision): the root pane
     dropped its line first and read correctly bare — keeping it on detail
     panes only made the identical navrow render differently per depth. */
  .navrow {
    flex: none;
    display: flex;
    align-items: center;
    gap: 4px;
    height: 44px;
    padding: 0 10px 0 12px;
  }

  .navtitle {
    font-size: 15px;
    font-weight: 700;
    letter-spacing: 0.01em;
    color: var(--text);
  }

  .backchev {
    border: none;
    background: transparent;
    width: 28px;
    height: 28px;
    border-radius: 7px;
    display: grid;
    place-items: center;
    cursor: pointer;
    padding: 0;
    color: var(--accent);
  }

  .backchev:hover,
  .backchev:active {
    background: var(--active);
  }

  .backchev svg {
    width: 10px;
    height: 14px;
  }

  /* --- scrollable list body ---------------------------------------------- */
  .scroll {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 0 8px;
  }

  /* home: search */
  .search {
    position: relative;
    margin: 10px 12px 12px;
    flex: none;
  }

  .search svg {
    position: absolute;
    left: 9px;
    top: 50%;
    width: 14px;
    height: 14px;
    transform: translateY(-50%);
    color: var(--text-dim);
    pointer-events: none;
  }

  .search input {
    width: 100%;
    height: 30px;
    padding: 0 26px 0 30px;
    border-radius: 8px;
    border: 1px solid var(--border);
    background: var(--hover);
    color: var(--text);
    font-size: 13px;
    outline: none;
  }

  /* Focus state = accent stroke only (the system focus language). The old
     inset outline ring was invisible against the near-opaque field fill. */
  .search input:focus {
    border-color: var(--accent);
  }

  .search input::placeholder {
    color: var(--text-dim);
  }

  .search-clear {
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

  .search-clear:hover {
    background: var(--hover);
    color: var(--text);
  }

  .search-clear svg {
    /* Absolute centering — place-items:center on a native <button> drifts
       ~1px down in WebKitGTK (shadow-DOM layout); same fix as the traffic
       lights (critique: "× not centered in its hover circle"). */
    position: absolute;
    top: 50%;
    left: 50%;
    translate: -50% -50%;
    width: 8px;
    height: 8px;
    stroke: currentColor;
    stroke-width: 1.4;
    stroke-linecap: round;
    fill: none;
  }

  /* rows (artists / menu top-level / items share the base) */
  .row,
  .mrow,
  .irow {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    width: 100%;
    padding: 0 10px;
    border: none;
    border-radius: 8px;
    background: transparent;
    color: var(--text);
    font-size: 13px;
    cursor: default;
    text-align: left;
  }

  /* .row height comes from --sidebar-row-size (user-tunable, min 28px),
     so no min-height here. Menu rows are deliberately 40px (≈iOS 44pt at
     our base size) — their own roomier cadence, not the list's. */
  .mrow,
  .irow {
    height: 40px;
    font-size: 13.5px;
  }

  /* menu items: check column left, label fills — not the name/trailing
     split of the other rows */
  .irow {
    justify-content: flex-start;
  }

  .irow .name {
    flex: 1;
  }

  .row:hover,
  .mrow:hover,
  .irow:hover:not(:disabled) {
    background: var(--hover);
  }

  /* Press feedback: instant and distinct from hover */
  .mrow:active:not(:disabled),
  .irow:active:not(:disabled) {
    background: var(--active);
  }

  /* Keyboard focus: explicit accent ring (inset) instead of the browser's
     inconsistent default */
  .mrow:focus-visible,
  .irow:focus-visible,
  .backchev:focus-visible,
  .gear:focus-visible,
  .search-clear:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
  }

  .row.active {
    background: var(--active);
    /* Translucent wash → normal text color (see PlayBar .playpause note). */
    color: var(--text);
    font-weight: 600;
  }

  .irow:disabled {
    color: var(--text-dim);
    cursor: default;
  }

  .name {
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
  }

  .count {
    font-size: 11px;
    color: var(--text-dim);
    flex: none;
  }

  /* menu item check column (matches the old in-titlebar menu bar's ✓) */
  .check {
    width: 14px;
    flex: none;
    color: var(--accent);
  }

  .empty {
    display: block;
    width: 100%;
    margin: 14px 10px;
    padding: 4px 0;
    border: none;
    background: transparent;
    font-size: 12px;
    color: var(--text-dim);
    text-align: left;
    cursor: pointer;
  }

  .empty:hover {
    color: var(--text);
  }

  .empty:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
  }

  /* Transient scanning state for a populated library (EmptyState owns the
     empty-library progress UI). */
  .scan-note {
    flex: none;
    margin: -8px 12px 4px;
    font-size: 11px;
    color: var(--text-dim);
  }

  /* --- Appearance pane (the old gear-popover settings) ------------------- */
  .appearance {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 16px 14px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .appearance label {
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 12px;
    color: var(--text-dim);
  }

  .appearance input[type="range"] {
    accent-color: var(--accent);
  }

  .theme {
    border: 1px solid var(--border);
    background: var(--hover);
    color: var(--text);
    font-size: 12px;
    border-radius: 8px;
    padding: 6px 8px;
    cursor: pointer;
  }

  .theme:hover {
    background: var(--active);
  }

  .toggle {
    flex-direction: row !important;
    align-items: center;
    gap: 8px;
  }

  .toggle input {
    accent-color: var(--accent);
    margin: 0;
  }

  .accent {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .accent > span {
    font-size: 12px;
    color: var(--text-dim);
  }

  .swatches {
    display: flex;
    align-items: center;
    gap: 7px;
    flex-wrap: wrap;
  }

  .swatch {
    width: 20px;
    height: 20px;
    padding: 0;
    border: 1px solid var(--border);
    border-radius: 50%;
    cursor: pointer;
    outline: 2px solid transparent;
    outline-offset: 2px;
    transition: outline-color 120ms ease;
  }

  .swatch:hover {
    outline-color: var(--text-dim);
  }

  .swatch.selected {
    outline-color: var(--text);
  }

  .swatch.custom {
    position: relative;
    overflow: hidden;
    background: conic-gradient(#e5484d, #ffb224, #46a758, #00a2c7, #7c58f0, #e5484d);
  }

  .swatch.custom input {
    position: absolute;
    inset: -4px;
    opacity: 0;
    cursor: pointer;
  }
</style>
