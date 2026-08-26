<script lang="ts">
  import { ui, cycleTheme, resolvedTheme } from "../lib/stores/ui.svelte";
  import { library } from "../lib/stores/library.svelte";
  import { ACCENT_PRESETS } from "../lib/accent";

  let settingsOpen = $state(false);

  // The native color input needs a concrete value; presets leave it alone.
  let customHex = $state(ui.accentColor ?? "#ff6ec7");

  const albumCounts = $derived.by(() => {
    const m = new Map<string, number>();
    for (const a of library.albums) {
      m.set(a.artistId, (m.get(a.artistId) ?? 0) + 1);
    }
    return m;
  });

  const filtered = $derived(
    ui.filter.trim() === ""
      ? library.artists
      : library.artists.filter((a) =>
          a.name.toLowerCase().includes(ui.filter.trim().toLowerCase()),
        ),
  );

  function select(id: string) {
    ui.activeArtistId = id;
    ui.expandedAlbum.songs = null;
    ui.expandedAlbum.albums = null;
  }
</script>

<aside class="sidebar glass">
  <header class="brand">
    <span class="app-name">Songstress</span>
    <button class="gear" aria-label="Settings" aria-expanded={settingsOpen} onclick={() => (settingsOpen = !settingsOpen)}>
      <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.3">
        <circle cx="8" cy="8" r="2.2" />
        <path
          d="M8 1.8 v1.9 M8 12.3 v1.9 M14.2 8 h-1.9 M3.7 8 H1.8 M12.5 3.5 l-1.35 1.35 M4.85 11.15 L3.5 12.5 M12.5 12.5 l-1.35 -1.35 M4.85 4.85 L3.5 3.5"
          stroke-linecap="round"
        />
      </svg>
    </button>
    {#if settingsOpen}
      <div class="settings">
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
    {/if}
  </header>

  <div class="search">
    <svg viewBox="0 0 16 16" aria-hidden="true">
      <circle cx="7" cy="7" r="4.4" fill="none" stroke="currentColor" stroke-width="1.4" />
      <path d="M10.4 10.4 L13.6 13.6" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
    </svg>
    <input
      type="text"
      placeholder="Search artist..."
      spellcheck="false"
      bind:value={ui.filter}
    />
  </div>

  <nav>
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
    {#if filtered.length === 0 && ui.filter.trim() !== ""}
      <p class="empty">No artists match.</p>
    {/if}
  </nav>
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
    border-right: 1px solid var(--border);
    border-bottom: 1px solid var(--border);
    user-select: none;
  }

  .brand {
    position: relative;
    flex: none;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 28px 12px 18px;
    height: 30px;
  }

  .app-name {
    font-size: 17px;
    font-weight: 700;
    letter-spacing: 0.01em;
    color: var(--text);
  }

  .brand .gear {
    width: 26px;
    height: 26px;
    border-radius: 7px;
  }

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
    padding: 0 10px 0 30px;
    border-radius: 8px;
    border: 1px solid var(--border);
    background: var(--hover);
    color: var(--text);
    font-size: 13px;
    outline: none;
  }

  .search input:focus {
    border-color: var(--accent);
  }

  .search input::placeholder {
    color: var(--text-dim);
  }

  nav {
    flex: 1;
    overflow-y: auto;
    padding: 0 8px;
    min-height: 0;
  }

  .row {
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

  .row:hover {
    background: var(--hover);
  }

  .row.active {
    background: var(--active);
    /* Translucent wash → normal text color (see PlayBar .playpause note). */
    color: var(--text);
    font-weight: 600;
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

  .empty {
    margin: 14px 10px;
    font-size: 12px;
    color: var(--text-dim);
  }

  .gear {
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

  .gear svg {
    width: 16px;
    height: 16px;
  }

  .settings {
    position: absolute;
    top: calc(100% + 10px);
    left: 12px;
    right: 12px;
    z-index: 30;
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 14px;
    border-radius: 12px;
    border: 1px solid var(--border);
    background: var(--panel-bg-strong);
    backdrop-filter: blur(28px) saturate(160%);
    -webkit-backdrop-filter: blur(28px) saturate(160%);
    box-shadow: var(--shadow);
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 12px;
    color: var(--text-dim);
  }

  input[type="range"] {
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
    flex-direction: row;
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
    transition: outline-color 0.12s ease;
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
