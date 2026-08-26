<script lang="ts">
  import { ui } from "../lib/stores/ui.svelte";
  import { library } from "../lib/stores/library.svelte";
  import { currentTrack } from "../lib/stores/playback.svelte";
  import { extractArtColors } from "../lib/artColors";
  import { buildRows, columnCount } from "../lib/buildRows";
  import { albumTitleMatches, albumTrackMatches, fold } from "../lib/search";
  import type { Album } from "../lib/types";
  import ExpandedPanel from "./ExpandedPanel.svelte";
  import EmptyState from "./EmptyState.svelte";

  const GAP = 20;

  let gridWidth = $state(0);

  let cols = $derived(columnCount(gridWidth, ui.tileSize, GAP));

  // All Artists: alphabetical by artist (sort_name), then year ascending
  // within each artist. Artist view: year ascending (store order).
  let visibleAlbums = $derived.by(() => {
    if (ui.activeArtistId !== "all") {
      return library.albums.filter((a) => a.artistId === ui.activeArtistId);
    }
    return [...library.albums].sort((a, b) => {
      const an = library.artistOf(a)?.sortName ?? "";
      const bn = library.artistOf(b)?.sortName ?? "";
      if (an !== bn) return an.localeCompare(bn);
      return byYearThenTitle(a, b);
    });
  });

  // Titlebar search splits results in two labeled sections: "Songs" (albums
  // containing matching tracks — expanding one shows ONLY the matching songs)
  // and "Albums" (albums matching by title or artist name). An album can
  // appear in both.
  let searchQuery = $derived(ui.mediaFilter.trim());
  let searchActive = $derived(searchQuery !== "");

  let songAlbums = $derived.by(() => {
    if (!searchActive) return [];
    return library.albums.filter((a) =>
      albumTrackMatches(
        { trackTitles: library.tracksOf(a.id).map((t) => t.title) },
        searchQuery,
      ),
    );
  });

  let titleAlbums = $derived.by(() => {
    if (!searchActive) return [];
    return library.albums.filter((a) =>
      albumTitleMatches({ title: a.title }, searchQuery),
    );
  });

  // Per-section expansion memory (keeps the panel mounted while collapsing
  // so the height animation can play).
  let lastSongPanelId = $state<string | null>(null);
  let lastAlbumPanelId = $state<string | null>(null);
  $effect(() => {
    if (ui.expandedAlbum.songs) lastSongPanelId = ui.expandedAlbum.songs;
  });
  $effect(() => {
    if (ui.expandedAlbum.albums) lastAlbumPanelId = ui.expandedAlbum.albums;
  });

  let songExpandedId = $derived(ui.expandedAlbum.songs ?? lastSongPanelId);
  let albumExpandedId = $derived(ui.expandedAlbum.albums ?? lastAlbumPanelId);

  // Each section expands INDEPENDENTLY: the Songs section shows only the
  // matching tracks, the Albums section always the full album.
  let rows = $derived(buildRows(visibleAlbums, cols, albumExpandedId));
  let songRows = $derived(
    buildRows(
      songAlbums,
      cols,
      songAlbums.some((a) => a.id === songExpandedId) ? songExpandedId : null,
    ),
  );
  let titleRows = $derived(
    buildRows(
      titleAlbums,
      cols,
      titleAlbums.some((a) => a.id === albumExpandedId) ? albumExpandedId : null,
    ),
  );

  let sections = $derived(
    searchActive
      ? [
          { key: "songs", label: "Songs", rows: songRows },
          { key: "albums", label: "Albums", rows: titleRows },
        ].filter((s) => s.rows.length > 0)
      : [{ key: "all", label: null as string | null, rows }],
  );

  function matchingTrackIds(albumId: string): Set<string> | null {
    if (!searchActive) return null;
    const q = fold(searchQuery);
    return new Set(
      library.tracksOf(albumId)
        .filter((t) => fold(t.title).includes(q))
        .map((t) => t.id),
    );
  }

  function expandedIn(section: string, id: string): boolean {
    return (section === "songs" ? ui.expandedAlbum.songs : ui.expandedAlbum.albums) === id;
  }

  function byYearThenTitle(a: Album, b: Album): number {
    if (a.year === null && b.year !== null) return 1;
    if (b.year === null && a.year !== null) return -1;
    if (a.year !== b.year) return (a.year ?? 0) - (b.year ?? 0);
    return a.title.localeCompare(b.title);
  }

  function toggleExpand(id: string, section: "songs" | "albums") {
    const cur = ui.expandedAlbum[section];
    if (cur !== id) {
      // Warm the image cache before the panel needs to paint this cover,
      // otherwise a multi-megabyte decode stalls the switch frame.
      const cover = library.albums.find((a) => a.id === id)?.cover;
      if (cover) {
        const warm = new Image();
        warm.decoding = "async";
        warm.src = cover;
        warm.decode().catch(() => {});
        // Warm the color cache too so the panel gradient is ready on arrival.
        extractArtColors(cover);
      }
    }
    ui.expandedAlbum[section] = cur === id ? null : id;
  }
</script>

<main class="content">
  <div class="grid" bind:clientWidth={gridWidth}>
    {#if library.live && (!library.ready || library.albums.length === 0)}
      <EmptyState />
    {:else if searchActive && sections.length === 0}
      <p class="no-match">No albums or songs match “{searchQuery}”.</p>
    {:else}
      {#each sections as section (section.key)}
        {#if section.label}
          <h2 class="section-label">{section.label}</h2>
        {/if}
        {#each section.rows as row (row.kind === "albums" ? `r-${row.items[0].id}` : `x-${section.key}`)}
          {#if row.kind === "albums"}
            <div class="grid-row" style:--cols={cols}>
              {#each row.items as album (album.id)}
                {@const playingAlbum = currentTrack()?.albumId === album.id}
                <button
                  class="tile"
                  class:expanded={expandedIn(section.key, album.id)}
                  onclick={() => toggleExpand(album.id, section.key === "songs" ? "songs" : "albums")}
                >
                  <span class="cover" class:ring={playingAlbum}>
                    {#if album.cover}
                      <!-- no loading="lazy": WebKit re-evaluates lazy images on
                           repaint and evicts decoded data when idle, flashing a
                           blank frame on hover/return; 246 thumbs are cheap -->
                      <img src={album.cover} alt="" draggable="false" />
                    {/if}
                    {#if album.staged}
                      <span class="staged" title="Not saved to the library folder yet">Imported</span>
                    {/if}
                  </span>
                  <span class="caption">
                    <span class="t">{album.title}</span>
                    <span class="sub">
                      {ui.activeArtistId === "all" ? library.artistOf(album)?.name : album.year}
                    </span>
                  </span>
                </button>
              {/each}
            </div>
          {:else}
            <ExpandedPanel
              album={row.album}
              open={expandedIn(section.key, row.album.id)}
              visibleTrackIds={section.key === "songs" ? matchingTrackIds(row.album.id) : null}
            />
          {/if}
        {/each}
      {/each}
    {/if}
  </div>
</main>

<style>
  .content {
    position: absolute;
    inset: 0;
    overflow-y: auto;
    scrollbar-width: none;
    padding: var(--gap) var(--gap) calc(var(--playbar-h) + 28px)
      calc(var(--sidebar-width) + var(--gap));
  }

  .no-match {
    margin: 24px 4px;
    font-size: 13px;
    color: var(--text-dim);
  }

  .section-label {
    margin: 8px 2px -8px;
    font-size: 11.5px;
    font-weight: 600;
    letter-spacing: 0.09em;
    text-transform: uppercase;
    color: var(--text-dim);
  }

  .content::-webkit-scrollbar {
    display: none;
  }

  .grid {
    display: flex;
    flex-direction: column;
    gap: var(--gap);
  }

  .grid-row {
    display: grid;
    grid-template-columns: repeat(var(--cols), minmax(0, 1fr));
    gap: 12px var(--gap);
  }

  .tile {
    display: flex;
    flex-direction: column;
    gap: 13px;
    padding: 0;
    border: none;
    background: transparent;
    text-align: left;
    cursor: default;
  }

  .cover {
    display: block;
    aspect-ratio: 1;
    width: 100%;
    border-radius: var(--radius-cover);
    overflow: hidden;
    box-shadow: 0 4px 18px rgba(0, 0, 0, 0.35);
    outline: 2px solid transparent;
    outline-offset: 1px;
    position: relative;
    transition:
      outline-color 0.15s ease,
      translate 0.15s ease;
  }

  .cover img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }

  /* Artwork flash fix, final: ANY hover transform promotes a compositor
     layer, and WebKitGTK paints the create/destroy churn as a one-frame
     blank (permanent will-change for all covers fixed the flash but its
     246 layers made the blur effect's move artifacts much worse; :has
     batch-promotion churned worse still). So: no transform on hover — the
     lift is replaced by a paint-only outline ring, which never promotes. */
  .tile:hover .cover {
    outline-color: var(--text-dim);
  }

  .cover .staged {
    position: absolute;
    top: 8px;
    left: 8px;
    padding: 2px 8px;
    border-radius: 999px;
    background: rgba(0, 0, 0, 0.62);
    color: #fff;
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.07em;
    text-transform: uppercase;
    pointer-events: none;
  }

  .cover.ring {
    outline-color: var(--accent);
  }

  .tile.expanded .cover {
    outline-color: var(--accent);
  }

  .caption {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .caption .t {
    font-size: 13px;
    font-weight: 600;
    color: var(--text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .caption .sub {
    font-size: 11.5px;
    color: var(--text-dim);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
