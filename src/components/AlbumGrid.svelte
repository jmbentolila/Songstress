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

  // Library search splits results in two labeled sections: "Songs" (albums
  // containing matching tracks — expanding one shows ONLY the matching songs)
  // and "Albums" (albums matching by title or artist name). An album can
  // appear in both.
  let searchQuery = $derived(ui.search.trim());
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

  // Per-section panel HOST: where the panel row physically sits (also the
  // collapse memory — the row persists at 0px so re-opens are instant).
  // Normally host === the store's expanded album; it LAGS for one snappy
  // close on a cross-row switch (the panel closes at the host row, hands
  // back via onSwitchCloseDone, the 0px row moves — invisible — and the
  // panel opens at the destination). Same-row switches flip the host
  // immediately (the row doesn't move; the panel swaps content in place).
  let songHostId = $state<string | null>(null);
  let albumHostId = $state<string | null>(null);

  // The list the albums-section panel currently renders in.
  let albumList = $derived(searchActive ? titleAlbums : visibleAlbums);

  // If the host's row disappears from its section (search/artist change)
  // mid-close, the panel is unmounted and its close can never complete —
  // the host must follow the target itself or every later switch deadlocks.
  // The box can't be open (its row is gone), so the move is invisible.
  $effect(() => {
    const host = albumHostId;
    const target = ui.expandedAlbum.albums;
    if (host && target && host !== target && !albumList.some((a) => a.id === host))
      albumHostId = target;
  });
  $effect(() => {
    const host = songHostId;
    const target = ui.expandedAlbum.songs;
    if (host && target && host !== target && !songAlbums.some((a) => a.id === host))
      songHostId = target;
  });

  function setHost(section: "songs" | "albums", id: string | null) {
    if (section === "songs") songHostId = id;
    else albumHostId = id;
  }

  // Row-model same-row check (no DOM measurement): buildRows slices the
  // section list by `cols`, so two albums share a row iff their indices
  // floor to the same row. A host that isn't in the list (row vanished)
  // is treated as cross-row — the conservative lag path.
  function sameRowIn(list: Album[], a: string, b: string): boolean {
    const ia = list.findIndex((x) => x.id === a);
    const ib = list.findIndex((x) => x.id === b);
    return ia >= 0 && ib >= 0 && Math.floor(ia / cols) === Math.floor(ib / cols);
  }

  // ── Reframe (the 20px line) ───────────────────────────────────────────
  // Every open / collapse / switch ends with the focused album's row
  // resting 20px (the scroller's top padding) under the top of the window
  // — or as low as the final layout's bottom edge allows (the browser's
  // scrollTop clamp does that for free).
  //
  // The travel is an ANCHOR, not a predicted tween: each frame it nudges
  // scrollTop by however far the row's LIVE viewport position is from the
  // desired one (an eased glide to the line, then a hold). Reading the
  // row live makes it immune to layout changes mid-flight — a switch's
  // close and relocation play under a pinned row instead of shifting it,
  // so the row's motion is one continuous glide with no stall-and-snap.
  // Glides are click-triggered, so their distance is bounded by the
  // viewport (the clicked row is on screen): ease-out cubic, 240–380ms.
  // Wheel / keys / touch cancel (the user takes over; the height
  // animations finish on their own). Reduced motion disables the reframe
  // entirely — no viewport movement at all.
  const LINE = 20; // .content padding-top (var(--gap))
  let heldSwitch: { sectionKey: string; from: string } | null = $state(null);
  let reframeRaf = 0;
  let reframeCleanup: (() => void) | null = null;
  let releaseTimer: ReturnType<typeof setTimeout> | undefined;

  function stopReframe() {
    cancelAnimationFrame(reframeRaf);
    clearTimeout(releaseTimer);
    reframeCleanup?.();
    reframeCleanup = null;
  }

  /**
   * @param holdFrom  Switch only: the panel's target is held at this album
   *                  until the glide lands — the close then plays under
   *                  the pinned row, and the slot relocates while invisible.
   * @param settleMs  Absolute time by which the layout ABOVE the row will
   *                  have stopped changing; the anchor keeps pinning the
   *                  row at the line until then (a switch's close +
   *                  relocation; a mid-switch cancel's close-in-place).
   */
  function reframeTo(sectionKey: string, albumId: string, holdFrom: string | null, settleMs: number) {
    const scroller = document.querySelector<HTMLElement>(".content");
    const tile = document.querySelector<HTMLElement>(
      `[data-section="${sectionKey}"][data-album-id="${CSS.escape(albumId)}"]`,
    );
    if (!scroller || !tile) return;
    const yStart = tile.getBoundingClientRect().top - scroller.getBoundingClientRect().top;
    const delta = yStart - LINE;
    if (Math.abs(delta) <= 4) return; // effectively on the line — no twitch
    if (matchMedia("(prefers-reduced-motion: reduce)").matches) return;

    if (holdFrom) heldSwitch = { sectionKey, from: holdFrom };
    const T = Math.min(380, 240 + Math.abs(delta) * 0.08);
    const t0 = performance.now();
    const settle = Math.max(T, settleMs);

    // The held close starts when the glide lands, not when the anchor
    // ends: the tail exists only to pin the row while the close and the
    // (invisible) relocation play.
    if (holdFrom) {
      releaseTimer = setTimeout(() => {
        heldSwitch = null;
      }, T);
    }

    const onUserScroll = () => {
      stopReframe();
      heldSwitch = null;
    };
    scroller.addEventListener("wheel", onUserScroll, { passive: true });
    scroller.addEventListener("touchstart", onUserScroll, { passive: true });
    scroller.addEventListener("keydown", onUserScroll);
    reframeCleanup = () => {
      scroller.removeEventListener("wheel", onUserScroll);
      scroller.removeEventListener("touchstart", onUserScroll);
      scroller.removeEventListener("keydown", onUserScroll);
    };

    const step = (now: number) => {
      const t = now - t0;
      const p = Math.min(1, t / T);
      const yDesired = yStart + (LINE - yStart) * (1 - Math.pow(1 - p, 3)); // ease-out cubic
      const rowViewY = tile.getBoundingClientRect().top - scroller.getBoundingClientRect().top;
      scroller.scrollTop += rowViewY - yDesired;
      if (t < settle) {
        reframeRaf = requestAnimationFrame(step);
      } else {
        reframeCleanup?.();
        reframeCleanup = null;
      }
    };
    reframeRaf = requestAnimationFrame(step);
  }


  // Each section expands INDEPENDENTLY: the Songs section shows only the
  // matching tracks, the Albums section always the full album.
  let rows = $derived(buildRows(visibleAlbums, cols, albumHostId));
  let songRows = $derived(
    buildRows(
      songAlbums,
      cols,
      songAlbums.some((a) => a.id === songHostId) ? songHostId : null,
    ),
  );
  let titleRows = $derived(
    buildRows(
      titleAlbums,
      cols,
      titleAlbums.some((a) => a.id === albumHostId) ? albumHostId : null,
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
    const next = cur === id ? null : id;
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
    // Measured against the HOST (where the box physically is, which may be
    // mid-close toward another album). Indices come from the row model.
    const host = section === "songs" ? songHostId : albumHostId;
    const list = section === "songs" ? songAlbums : albumList;
    const sectionKey = section === "songs" ? "songs" : searchActive ? "albums" : "all";
    const hIdx = host ? list.findIndex((a) => a.id === host) : -1;
    const iIdx = list.findIndex((a) => a.id === id);
    // The panel slot sits DIRECTLY BELOW the host album's row; it is
    // "above the target row" iff the host's row index is smaller — that's
    // when layout above the row keeps changing (a close / relocation)
    // and the anchor must keep pinning after the glide.
    const slotAbove = hIdx >= 0 && iIdx >= 0 && Math.floor(hIdx / cols) < Math.floor(iIdx / cols);

    // A fresh intent takes over first: any in-flight reframe stops and a
    // held switch is released (the panel then proceeds on its own).
    stopReframe();
    heldSwitch = null;

    if (next !== null && cur !== null && cur !== id) {
      // Switch. Same row → the row doesn't move → flip now (in-place swap
      // + fade), no reframe. Different row → the reframe owns the switch:
      // glide the row to the 20px line (the old slot still open above),
      // hold the panel's target until the glide lands, then the close
      // (120ms) and the invisible slot relocation play under the pinned
      // row, and the new album expands from the settled line. The anchor
      // tail covers the close + relocation window.
      if (sameRowIn(list, host ?? "", id)) {
        setHost(section, id);
      } else {
        reframeTo(sectionKey, id, host ?? null, 380 + 120 + 60);
      }
    } else if (next !== null) {
      // Fresh open / open from closed: flip the host now (the slot move
      // is absorbed by the live anchor); glide and grow run in parallel.
      setHost(section, id);
      reframeTo(sectionKey, id, null, 0);
    } else {
      // Collapse (or a mid-switch cancel): the host stays (per-section
      // memory). A close-in-place above the row keeps the layout in flux
      // — pin through it.
      reframeTo(sectionKey, id, null, slotAbove ? 280 + 20 : 0);
    }
    ui.expandedAlbum[section] = next;
  }
</script>

<main class="content">
  <div class="grid" bind:clientWidth={gridWidth}>
    {#if library.live && (!library.ready || library.albums.length === 0)}
      <EmptyState />
    {:else if searchActive && sections.length === 0}
      <button class="no-match" onclick={() => (ui.search = "")} title="Clear search">
        No albums or songs match “{searchQuery}” — clear search
      </button>
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
                  data-album-id={album.id}
                  data-section={section.key}
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
            <div class="panel-slot" data-section={section.key}>
              <ExpandedPanel
                album={row.album}
                targetId={
                  heldSwitch && heldSwitch.sectionKey === section.key
                    ? heldSwitch.from
                    : section.key === "songs"
                      ? ui.expandedAlbum.songs
                      : ui.expandedAlbum.albums
                }
                visibleTrackIds={section.key === "songs" ? matchingTrackIds(row.album.id) : null}
                onSwitchCloseDone={() =>
                  (section.key === "songs"
                    ? (songHostId = ui.expandedAlbum.songs ?? songHostId)
                    : (albumHostId = ui.expandedAlbum.albums ?? albumHostId))}
              />
            </div>
          {/if}
        {/each}
      {/each}
    {/if}
  </div>
</main>

<style>
  .content {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    /* Bottom clip ends at the playbar's top line: rows slide out UNDER
       the shelf instead of passing behind its glass (where, at 0.7 alpha,
       captions stayed readable). backdrop-filter can't frost in-window
       content on this WebKitGTK, and a per-row `filter: blur()` smears
       the whole row — so don't show the content behind the bar at all. */
    bottom: var(--playbar-h);
    overflow-y: auto;
    overflow-x: hidden;
    scrollbar-width: none;
    padding: var(--gap) var(--gap) var(--gap)
      calc(var(--sidebar-width) + var(--gap));
  }

  /* Same voice as the sidebar's zero-match action (Sidebar .empty): the
   * surface that owns the results offers the remedy. */
  .no-match {
    display: block;
    width: 100%;
    margin: 24px 4px;
    padding: 4px 0;
    border: none;
    background: transparent;
    font-size: 13px;
    color: var(--text-dim);
    text-align: left;
    cursor: pointer;
  }

  .no-match:hover {
    color: var(--text);
  }

  .no-match:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
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
    /* 8px under the caption: the text needs room to breathe off the row's
     * bottom edge (the 13px cover→caption gap was fine; the bottom wasn't). */
    padding: 0 0 8px;
    border: none;
    background: transparent;
    text-align: left;
    cursor: pointer;
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

  /* Press = the cover's own state border, not an overlay wash: the accent
   * ring the tile already uses for playing/expanded. (An ::after wash was
   * tried first — the user read it as a flat film over the artwork, not
   * as the album acknowledging the press. The border IS the album's
   * selected-state language. Must sit after :hover above: equal
   * specificity, later rule wins while both match.) */
  .tile:active .cover {
    outline-color: var(--accent);
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
