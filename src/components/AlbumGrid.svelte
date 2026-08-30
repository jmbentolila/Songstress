<script lang="ts">
  import { tick } from "svelte";
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
  // host === the store's expanded album, EXCEPT during a cross-row
  // switch's collapse phase: the host stays at the outgoing album while
  // it plays the plain collapse, and flips to the target when the close
  // finishes (see pending below). Same-row switches flip the host in
  // place (the instance persists, content swaps + fades).
  let songHostId = $state<string | null>(null);
  let albumHostId = $state<string | null>(null);

  // Cross-row switches are COLLAPSE-THEN-EXPAND, composed from the two
  // plain animations — no bespoke choreography (the old parallel design
  // ghosted the outgoing panel and pinned it closed under the glide; on
  // downward switches that fight showed as motion in the 20px strip and
  // was never fully tamed on this WebKitGTK — the strip above the row
  // only ever moves when layout ABOVE the row animates while a
  // one-paint-late scrollTop chases it, so any close-above-the-row
  // design inherits the risk).
  // The pending open survives the collapse: the host panel's onClosed
  // consumes it (flip host + glide + grow). Any other intent replaces or
  // clears it (each branch of toggleExpand sets it explicitly). Plain
  // (non-reactive): written from event handlers and effects only.
  let songPending: string | null = null;
  let albumPending: string | null = null;

  // The host row's component identity is a two-value token. A cross-row
  // switch flips it when the host moves to the destination (phase 2),
  // which makes the destination row a FRESH mount (with the old key, the
  // outgoing instance would be re-keyed at the destination and teleport
  // there with the old content at the old height). Same-row switches
  // keep the token: the instance persists and swaps content in place —
  // no remount, no height motion.
  let songTok = $state<"a" | "b">("a");
  let albumTok = $state<"a" | "b">("a");

  // The list the albums-section panel currently renders in.
  let albumList = $derived(searchActive ? titleAlbums : visibleAlbums);

  // If the host's row disappears from its section (search/artist change
  // mid-collapse), the panel is unmounted and every later switch could
  // deadlock — the host must follow the target itself (the box can't be
  // open: its row is gone, so the move is invisible). The unmount can't
  // fire onClosed, so a pending open is cleared with it (the target's row
  // mounts open on its own when it (re)appears).
  $effect(() => {
    const host = albumHostId;
    const target = ui.expandedAlbum.albums;
    if (host && target && host !== target && !albumList.some((a) => a.id === host)) {
      albumHostId = target;
      albumPending = null;
    }
  });
  $effect(() => {
    const host = songHostId;
    const target = ui.expandedAlbum.songs;
    if (host && target && host !== target && !songAlbums.some((a) => a.id === host)) {
      songHostId = target;
      songPending = null;
    }
  });

  function setHost(section: "songs" | "albums", id: string | null) {
    if (section === "songs") songHostId = id;
    else albumHostId = id;
  }

  function flipToken(section: "songs" | "albums") {
    if (section === "songs") songTok = songTok === "a" ? "b" : "a";
    else albumTok = albumTok === "a" ? "b" : "a";
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
  // row live makes it immune to layout changes mid-flight — the panel
  // grows while the glide is running, and the row's motion stays one
  // continuous glide with no stall-and-snap. Growing during the travel
  // also makes the distance EXACT from the click frame: the final layout
  // height is established while the view is still moving, so there is
  // nothing left to re-target. The anchor only ever tracks rows whose
  // layout ABOVE them is stable: panels grow and close BELOW their row,
  // so nothing animated moves the pinned row (any design that animates
  // layout above the row fights this anchor's one-paint-late scrollTop on
  // this WebKitGTK — that's why cross-row switches collapse-then-expand
  // instead of closing the outgoing panel under the glide).
  //
  // Glides are click-triggered, so their distance is bounded by the
  // viewport (the clicked row is on screen): ease-out cubic, 240–380ms.
  // Wheel / keys / touch cancel (the user takes over; the height
  // animations finish on their own). Reduced motion disables the reframe
  // entirely — no viewport movement at all.
  const LINE = 20; // .content padding-top (var(--gap))
  // Glide duration for a given distance — click-triggered, so bounded by
  // the viewport: ease-out cubic, 240–380ms.
  const glideTime = (delta: number) => Math.min(380, 240 + Math.abs(delta) * 0.08);
  let reframeRaf = 0;
  let reframeCleanup: (() => void) | null = null;

  function stopReframe() {
    cancelAnimationFrame(reframeRaf);
    reframeCleanup?.();
    reframeCleanup = null;
  }

  /**
   * Consume a pending cross-row open: the outgoing panel just finished
   * its collapse, so the destination can take over — flip the host (the
   * token flip makes the destination a fresh mount), wait for the row
   * list to flush, then glide the destination row to the line while the
   * panel grows below it.
   */
  async function consumePending(section: "songs" | "albums") {
    const p = section === "songs" ? songPending : albumPending;
    if (!p) return;
    if (section === "songs") songPending = null;
    else albumPending = null;
    flipToken(section);
    setHost(section, p);
    await tick(); // the row list must have flushed before the glide measures
    // A newer intent may have taken over in the window (it always re-decides
    // host + pending) — don't glide to a stale target.
    if ((section === "songs" ? songHostId : albumHostId) !== p) return;
    const sectionKey = section === "songs" ? "songs" : searchActive ? "albums" : "all";
    reframeTo(sectionKey, p);
  }

  function reframeTo(sectionKey: string, albumId: string): number {
    const scroller = document.querySelector<HTMLElement>(".content");
    const tile = document.querySelector<HTMLElement>(
      `button.tile[data-section="${sectionKey}"][data-album-id="${CSS.escape(albumId)}"]`,
    );
    if (!scroller || !tile) return 0;
    const yStart = tile.getBoundingClientRect().top - scroller.getBoundingClientRect().top;
    const delta = yStart - LINE;
    if (Math.abs(delta) <= 4) return 0; // effectively on the line — no twitch
    if (matchMedia("(prefers-reduced-motion: reduce)").matches) return 0;

    const T = glideTime(delta);
    const t0 = performance.now();

    const onUserScroll = () => {
      stopReframe();
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
      if (t < T) {
        reframeRaf = requestAnimationFrame(step);
      } else {
        reframeCleanup?.();
        reframeCleanup = null;
      }
    };
    reframeRaf = requestAnimationFrame(step);
    return T;
  }


  // Each section expands INDEPENDENTLY: the Songs section shows only the
  // matching tracks, the Albums section always the full album.
  let rows = $derived(buildRows(visibleAlbums, cols, albumHostId));
  let songRows = $derived(buildRows(songAlbums, cols, songHostId));
  let titleRows = $derived(buildRows(titleAlbums, cols, albumHostId));

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

  // The host row's target: the section's expanded album — UNLESS the
  // section is expanded elsewhere (a cross-row switch's collapse phase:
  // the host still shows the outgoing album, so this instance's target is
  // null = closed, and the panel plays the plain collapse).
  function hostTarget(sectionKey: string, albumId: string): string | null {
    const t = sectionKey === "songs" ? ui.expandedAlbum.songs : ui.expandedAlbum.albums;
    return t === albumId ? t : null;
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
    // Indices come from the row model; the host is where the box
    // physically is (=== cur, except during a cross-row collapse phase —
    // the host lags the store until the collapse finishes).
    const host = section === "songs" ? songHostId : albumHostId;
    const list = section === "songs" ? songAlbums : albumList;
    const sectionKey = section === "songs" ? "songs" : searchActive ? "albums" : "all";

    // A fresh intent takes over first: any in-flight reframe stops.
    stopReframe();

    if (next !== null && cur !== null && cur !== id) {
      // Switch. Same row → the row doesn't move → flip now (in-place
      // swap + fade), no reframe.
      // Cross-row → COLLAPSE, THEN EXPAND: the host stays put while the
      // outgoing panel plays the plain collapse; its onClosed consumes
      // the pending open (flip host + glide + fresh grow). Every branch
      // sets pending explicitly — a fresh intent always decides what the
      // next open will be.
      if (sameRowIn(list, host ?? "", id)) {
        if (section === "songs") songPending = null;
        else albumPending = null;
        setHost(section, id);
      } else {
        if (section === "songs") songPending = id;
        else albumPending = id;
      }
    } else if (next !== null) {
      // Fresh open / open from closed: flip the host now (the slot move
      // is absorbed by the live anchor); glide and grow run in parallel.
      if (section === "songs") songPending = null;
      else albumPending = null;
      setHost(section, id);
      reframeTo(sectionKey, id);
    } else {
      // Collapse (or a mid-switch cancel): the host stays (per-section
      // memory).
      if (section === "songs") songPending = null;
      else albumPending = null;
      reframeTo(sectionKey, host ?? id);
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
        {#each section.rows as row (
          row.kind === "albums"
            ? `r-${row.items[0].id}`
            : `x-${section.key}-${section.key === "songs" ? songTok : albumTok}`
        )}
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
            <div class="panel-slot" data-section={section.key} data-album-id={row.album.id}>
              <ExpandedPanel
                album={row.album}
                targetId={hostTarget(section.key, row.album.id)}
                visibleTrackIds={section.key === "songs" ? matchingTrackIds(row.album.id) : null}
                onClosed={() =>
                  consumePending(section.key === "songs" ? "songs" : "albums")}
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
