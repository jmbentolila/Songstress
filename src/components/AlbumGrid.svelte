<script lang="ts">
  import { tick } from "svelte";
  import { ui } from "../lib/stores/ui.svelte";
  import { library } from "../lib/stores/library.svelte";
  import { currentTrack } from "../lib/stores/playback.svelte";
  import { extractArtColors } from "../lib/artColors";
  import { buildRows, columnCount } from "../lib/buildRows";
  import { skeletonRows, libraryLoading } from "../lib/loadingState";
  import { scanner } from "../lib/stores/scanner.svelte";
  import { albumTitleMatches, albumTrackMatches, fold } from "../lib/search";
  import type { Album } from "../lib/types";
  import ExpandedPanel from "./ExpandedPanel.svelte";
  import EmptyState from "./EmptyState.svelte";
  import GridSkeleton from "./GridSkeleton.svelte";

  const GAP = 20;

  let gridWidth = $state(0);
  let stageHeight = $state(0);

  /** Nothing to show + something in flight → the placeholder owns the stage.
   * The facts are spelled out (rather than read from the stores inside a helper)
   * so this and the sidebar's identical list cannot drift silently: the type
   * forces both to answer for every fact. */
  const loading = $derived(
    library.live &&
      libraryLoading({
        ready: library.ready,
        bootSlow: library.bootSlow,
        albums: library.albums.length,
        running: scanner.running,
        scanning: library.scanning,
        devLoading: library.devLoading,
      }),
  );

  // Minus the stage's own 20px padding top and bottom: the rows must fill the
  // visible grid, not the visible grid plus a scroll of nothing.
  const skRows = $derived(
    skeletonRows(gridWidth, Math.max(0, stageHeight - 2 * GAP), ui.tileSize, GAP),
  );

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
  // host === the store's expanded album at all times: a cross-row switch
  // flips it at t=0 (the destination mounts fresh and grows) while the
  // outgoing panel plays its close as a GHOST row at its own row. Same-
  // row switches flip the host in place (the instance persists, content
  // swaps + fades).
  let songHostId = $state<string | null>(null);
  let albumHostId = $state<string | null>(null);

  // Cross-row switches run the two ALBUM animations IN PARALLEL — no
  // bespoke choreography, no pin, no prediction (the old parallel design
  // pinned the outgoing ghost closed under the glide; on downward
  // switches that fight showed as motion in the 20px strip and was never
  // tamed — the strip above the row only ever moves when layout ABOVE
  // the row animates while a one-paint-late scrollTop chases it). What
  // came back is the minimal version:
  //   * the outgoing panel becomes a GHOST row at its own row — a fresh
  //     ExpandedPanel mount at the measured height (seamless hand-off)
  //     playing the plain 280ms CSS close, nothing else;
  //   * the host flips to the destination at t=0 (fresh mount + grow);
  //   * only the VIEW'S TRAVEL waits: a per-section glide queue drains
  //     when the ghost list is empty (drainGlide) — the travel is the one
  //     thing that conflicts with a close above the line, and a close
  //     above the row never moves the row ON the line (its close plays
  //     BELOW that row), so once the ghosts are gone the window is
  //     pin-free and bob-free in both directions.
  // ghosts: album id → measured spawn height, per section. $state: the
  // row model (buildRows) renders one ghost row per entry.
  let songGhosts = $state<Record<string, number>>({});
  let albumGhosts = $state<Record<string, number>>({});
  // Spacing phase per panel instance, keyed by album id (see
  // ExpandedPanel's `phase` prop). Host and ghost ids are always distinct
  // within a section, so one map serves both. The grid mirrors the panel
  // (which drives it via bind:phase) onto the slot element so the slot's
  // margin can sync with the panel's height animation. Ghosts spawn
  // "open" (they look exactly like the panel they replaced) and the
  // panel flips them through closing → closed itself.
  type PanelPhase = "closed" | "open" | "closing";
  let songPanelPhase = $state<Record<string, PanelPhase>>({});
  let albumPanelPhase = $state<Record<string, PanelPhase>>({});
  // Queued view travel per section (target album id). Plain, non-
  // reactive: written from event handlers/effects, consumed by drainGlide.
  let songGlide: string | null = null;
  let albumGlide: string | null = null;

  // The host row's component identity is a two-value token. A cross-row
  // switch flips it at t=0 when the host moves to the destination, which
  // makes the destination row a FRESH mount (with the old key, the
  // outgoing instance would be re-keyed at the destination and teleport
  // there with the old content at the old height). Same-row switches
  // keep the token: the instance persists and swaps content in place —
  // no remount, no height motion. Ghost rows carry their own key
  // (x-ghost-<id>), independent of the token.
  let songTok = $state<"a" | "b">("a");
  let albumTok = $state<"a" | "b">("a");

  // The list the albums-section panel currently renders in.
  let albumList = $derived(searchActive ? titleAlbums : visibleAlbums);

  // A ghost whose row vanishes (search/artist change mid-close) unmounts
  // WITHOUT firing onClosed — it must be removed from the ghost list or
  // the queued view travel would never drain. (A vanished HOST row needs
  // no handling: the host === the store, so the next switch measures
  // nothing, spawns a zero-height ghost that drains immediately, and
  // flips.)
  $effect(() => {
    const list = albumList;
    for (const id of Object.keys(albumGhosts)) {
      if (!list.some((a) => a.id === id)) {
        delete albumGhosts[id];
        delete albumPanelPhase[id];
        void drainGlide("albums");
      }
    }
  });
  $effect(() => {
    const list = songAlbums;
    for (const id of Object.keys(songGhosts)) {
      if (!list.some((a) => a.id === id)) {
        delete songGhosts[id];
        delete songPanelPhase[id];
        void drainGlide("songs");
      }
    }
  });

  function setHost(section: "songs" | "albums", id: string | null) {
    if (section === "songs") songHostId = id;
    else albumHostId = id;
    // Pre-seed the new host's spacing phase so its slot is styled from
    // the first frame (it opens: the panel's effect confirms "open" when
    // the grow starts).
    if (id) {
      if (section === "songs") songPanelPhase[id] = "open";
      else albumPanelPhase[id] = "open";
    }
  }

  function flipToken(section: "songs" | "albums") {
    if (section === "songs") songTok = songTok === "a" ? "b" : "a";
    else albumTok = albumTok === "a" ? "b" : "a";
  }

  function sectionKeyOf(section: "songs" | "albums"): string {
    return section === "songs" ? "songs" : searchActive ? "albums" : "all";
  }

  // ── Ghosts + queued travel (the parallel cross-row switch) ──────────
  // The live host panel's current height — read BEFORE the flip, while
  // the outgoing row is still in the DOM. The ghost mounts at exactly
  // this height, so the swap is invisible (seamless hand-off).
  function measureHost(section: "songs" | "albums"): number {
    const id = section === "songs" ? songHostId : albumHostId;
    if (!id) return 0;
    const slot = document.querySelector<HTMLElement>(
      `.panel-slot[data-section="${sectionKeyOf(section)}"][data-album-id="${CSS.escape(id)}"]`,
    );
    return slot?.querySelector<HTMLElement>(".inner")?.offsetHeight ?? 0;
  }

  function spawnGhost(section: "songs" | "albums", id: string, h: number) {
    if (section === "songs") {
      songGhosts[id] = h;
      songPanelPhase[id] = "open";
    } else {
      albumGhosts[id] = h;
      albumPanelPhase[id] = "open";
    }
  }

  function removeGhost(section: "songs" | "albums", id: string) {
    if (section === "songs") {
      delete songGhosts[id];
      delete songPanelPhase[id];
    } else {
      delete albumGhosts[id];
      delete albumPanelPhase[id];
    }
  }

  const hasGhosts = (section: "songs" | "albums") =>
    Object.keys(section === "songs" ? songGhosts : albumGhosts).length > 0;

  const getGlide = (section: "songs" | "albums") =>
    section === "songs" ? songGlide : albumGlide;
  const setGlide = (section: "songs" | "albums", id: string | null) => {
    if (section === "songs") songGlide = id;
    else albumGlide = id;
  };

  /**
   * Queue the view travel to an album's row. Fires immediately when no
   * ghost is still closing in this section — otherwise it waits for
   * drainGlide to be re-triggered (ghost onClosed / vanished-ghost
   * effect). The travel is deliberately the ONLY deferred piece: it is
   * the one thing that conflicts with a close above the line (writing
   * scrollTop while layout above the row animates — one-paint-late on
   * this WebKitGTK). Panels may grow and close freely; the glide just
   * lands after the dust settles, on an already-grown panel.
   */
  function queueGlide(section: "songs" | "albums", id: string) {
    setGlide(section, id);
    void drainGlide(section);
  }

  async function drainGlide(section: "songs" | "albums") {
    const target = getGlide(section);
    if (!target) return;
    if (hasGhosts(section)) return; // retried when the last ghost closes
    await tick(); // the row list must have flushed before the tile is measured
    // A newer intent may have replaced (or cleared) the target in the
    // window — it always re-decides the queue, so a stale drain no-ops.
    if (getGlide(section) !== target) return;
    setGlide(section, null);
    reframeTo(sectionKeyOf(section), target);
  }

  function onGhostClosed(section: "songs" | "albums", id: string) {
    removeGhost(section, id);
    void drainGlide(section);
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
  // so nothing animated moves the tracked row — and a cross-row switch's
  // ghost close (which CAN sit above the destination) is finished before
  // the anchor starts (the travel drains after the ghost list empties),
  // so the anchor never shares a frame with a close above the row (that
  // is the one-paint-late fight that killed the pinned-ghost design).
  //
  // Glides are click-triggered, so their distance is bounded by the
  // viewport (the clicked row is on screen): ease-out cubic, 240–380ms.
  // Wheel / keys / touch cancel (the user takes over; the height
  // animations finish on their own). Reduced motion disables the reframe
  // entirely — no viewport movement at all.
  //
  // Cross-row switches queue the travel (queueGlide) instead of starting
  // it: the outgoing panel's ghost may still be closing ABOVE the
  // destination, and a close above the row + a live anchor writing
  // scrollTop is exactly the one-paint-late fight this anchor must never
  // be in. drainGlide starts the glide once the ghost list is empty —
  // layout above the target is settled, the distance is exact, and the
  // glide lands on an already-grown panel.
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

  function reframeTo(sectionKey: string, albumId: string): number {
    // A fresh intent supersedes any in-flight glide (one scroller serves
    // all sections — a double rAF loop would fight over scrollTop).
    stopReframe();
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
  let rows = $derived(buildRows(visibleAlbums, cols, albumHostId, Object.keys(albumGhosts)));
  let songRows = $derived(buildRows(songAlbums, cols, songHostId, Object.keys(songGhosts)));
  let titleRows = $derived(buildRows(titleAlbums, cols, albumHostId, Object.keys(albumGhosts)));

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

  // The host row's target: the section's expanded album when it is THIS
  // album, else null (closed). With the parallel cross-row design the
  // host always equals the store (the flip happens at t=0), so this is
  // simply "am I the expanded one?" — the outgoing panel's close lives in
  // the ghost row, never in a lagging host.
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
    // The host is where the box physically is (=== cur: the host flips
    // with the store in every branch below, in the same tick).
    const host = section === "songs" ? songHostId : albumHostId;
    const list = section === "songs" ? songAlbums : albumList;

    // A fresh intent takes over first: any in-flight reframe stops.
    stopReframe();

    if (next !== null && cur !== null && cur !== id) {
      // Switch. Same row → the row doesn't move → flip now (in-place
      // swap + fade), no travel (any queued travel is superseded).
      // Cross-row → the two album animations run IN PARALLEL: the
      // outgoing panel becomes a ghost at its own row (measured height,
      // plain close) while the host flips to the destination at t=0
      // (fresh mount + grow). Only the travel is queued — it drains when
      // the ghost list empties, so it never writes scrollTop while a
      // close above the row is still animating.
      if (sameRowIn(list, host ?? "", id)) {
        setHost(section, id);
        setGlide(section, null);
      } else {
        const h = measureHost(section);
        spawnGhost(section, cur, h);
        flipToken(section);
        setHost(section, id);
        queueGlide(section, id);
      }
    } else if (next !== null) {
      // Fresh open / open from closed: flip the host now; the glide is
      // queued (drains immediately when no ghost is still closing, so in
      // the common case glide and grow run in parallel — and if a
      // previous close is still draining, the travel waits for it).
      setHost(section, id);
      queueGlide(section, id);
    } else {
      // Collapse (or a mid-switch cancel): the host STAYS (per-section
      // memory) and plays the plain close in place; the travel to it is
      // queued the same way.
      queueGlide(section, host ?? id);
    }
    ui.expandedAlbum[section] = next;
  }
</script>

<main class="content" bind:clientHeight={stageHeight} aria-busy={loading}>
  <div class="grid" class:enter={library.entering} bind:clientWidth={gridWidth}>
    {#if loading}
      <GridSkeleton {cols} rows={skRows} />
    {:else if library.live && library.ready && library.albums.length === 0}
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
        {#each section.rows as row, r (
          row.kind === "albums"
            ? `r-${row.items[0].id}`
            : row.kind === "ghost"
              ? `x-ghost-${row.id}`
              : `x-${section.key}-${section.key === "songs" ? songTok : albumTok}`
        )}
          {#if row.kind === "albums"}
            <div class="grid-row" style:--cols={cols}>
              {#each row.items as album, c (album.id)}
                {@const playingAlbum = currentTrack()?.albumId === album.id}
                <button
                  class="tile"
                  style:--i={r + c}
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
          {:else if row.kind === "ghost"}
            {@const ghostAlbum = library.albums.find((a) => a.id === row.id)}
            {@const phaseMap = section.key === "songs" ? songPanelPhase : albumPanelPhase}
            {#if ghostAlbum}
              <!-- Ghost: the outgoing panel of an in-flight cross-row
                   switch. Fresh mount at the measured height (seamless
                   hand-off), plain close, nothing else. -->
              <div
                class="panel-slot"
                class:closed={phaseMap[row.id] === "closed"}
                class:closing={phaseMap[row.id] === "closing"}
                data-section={section.key}
                data-album-id={row.id}
              >
                <ExpandedPanel
                  album={ghostAlbum}
                  targetId={null}
                  initialPx={section.key === "songs" ? songGhosts[row.id] ?? 0 : albumGhosts[row.id] ?? 0}
                  bind:phase={phaseMap[row.id]}
                  visibleTrackIds={section.key === "songs" ? matchingTrackIds(row.id) : null}
                  onClosed={() => onGhostClosed(section.key === "songs" ? "songs" : "albums", row.id)}
                />
              </div>
            {/if}
          {:else}
            {@const phaseMap = section.key === "songs" ? songPanelPhase : albumPanelPhase}
            <div
              class="panel-slot"
              class:closed={phaseMap[row.album.id] === "closed"}
              class:closing={phaseMap[row.album.id] === "closing"}
              data-section={section.key}
              data-album-id={row.album.id}
            >
              <ExpandedPanel
                album={row.album}
                targetId={hostTarget(section.key, row.album.id)}
                bind:phase={phaseMap[row.album.id]}
                visibleTrackIds={section.key === "songs" ? matchingTrackIds(row.album.id) : null}
                onClosed={() => void drainGlide(section.key === "songs" ? "songs" : "albums")}
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

  /* The panel slot's top spacing is part of the panel's animation. The
   * grid gap is fixed (it can't be opted out per item), so while the
   * panel is closed the slot's margin cancels the gap above it and the
   * rows on either side sit at the STANDARD 20px gap instead of 20 + 0 +
   * 20. The transition runs the SAME duration/curve as the height move
   * (360 open / 280 close — the panel flips the slot's class in the same
   * frame the height transition starts), so the panel's top edge glides
   * with the shrinking/growing box and the row below settles into the
   * standard gap without a jump at either end. */
  .panel-slot {
    margin-top: 0;
    transition: margin-top 360ms var(--ease-out);
  }

  .panel-slot.closing {
    margin-top: calc(-1 * var(--gap));
    transition: margin-top 280ms var(--ease-out);
  }

  .panel-slot.closed {
    margin-top: calc(-1 * var(--gap));
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

  /* The grid's half of the arrival entrance (material + keyframes: `.enter` in
   app.css). Two departures from the list, both because this is a GRID:

   * The unit is the TILE. The row is a layout box here, not a thing the eye
     tracks — at a 295px tile barely one and a half rows are on screen, so
     staggering rows could not unfold anything; it would only delay a wall.
   * `--i` is `row + column`, so the delays fall along the DIAGONAL: the top-left
     cover leads and the wave runs down-and-across, which reads as a surface
     opening rather than as N things arriving. A `row * cols + column` index
     marches strictly left-to-right and looks like a typewriter.

   The dials live on the animated element itself (inheriting them off a parent
   would make a child's transform a style recalc for every child), and they are
   quicker than the list's: a tile is far bigger than a row, so the same spread
   would take three times as long to cross the same visual distance. The cut is
   per ROW — from the ninth row down there are thousands of pixels of nothing
   above the fold, and 75 more covers do not need compositing for a third of a
   second. */
  /* The dials go on the TILE (the animated element), not on `.grid.enter`: a
     transform resolved from a variable inherited off a parent is a style recalc for
     every child. Static values, so this is hygiene rather than a fix — but the
     global rule says the same thing, and agreeing with it costs nothing. */
  .grid.enter .tile {
    --enter-dur: 320ms;
    --enter-step: 90ms;
    --enter-cap: 540ms;
    --enter-rise: 14px;
    animation: enter-row var(--enter-dur) var(--ease-out) both;
    animation-delay: min(calc(var(--i, 0) * var(--enter-step)), var(--enter-cap));
  }

  /* The rows themselves do not animate — the tile does. Without this the global
     `.enter > *` rule would move each row AND each tile inside it, so a cover would
     travel twice the distance and the diagonal would fight the cascade. */
  .grid.enter > .grid-row {
    animation: none;
  }

  .grid.enter > :nth-child(n + 9) .tile {
    animation: none;
  }

  /* Reduce: the global kill switch owns the duration, this owns the travel and the
     ladder. Specificity has to reach `.grid.enter .tile` for the delay to lose. */
  @media (prefers-reduced-motion: reduce) {
    .grid.enter .tile {
      animation-delay: 0s;
      animation-play-state: paused;
    }
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
    /* outline-color only: the hover-lift (translate) was removed in
       Step 0c — the transition property went with it. */
    transition: outline-color 150ms ease;
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
