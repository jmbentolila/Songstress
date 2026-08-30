<script lang="ts">
  import type { Album, Track } from "../lib/types";
  import { library } from "../lib/stores/library.svelte";
  import { playback, playTrack, currentTrack, queueTracks } from "../lib/stores/playback.svelte";
  import { extractArtColors } from "../lib/artColors";
  import { artGradient, gradientFromColors } from "../lib/gradient";
  import { resolvedTheme, ui } from "../lib/stores/ui.svelte";
  import {
    openContextMenu,
    contextMenu,
    type MenuItem,
  } from "../lib/stores/contextMenu.svelte";
  import {
    saveImports,
    discardImports,
    locateMissingTrack,
    removeTrack,
    removeMissingTracks,
  } from "../lib/stores/scanner.svelte";

  let {
    album,
    targetId,
    onClosed,
    /** When set, only these track ids render (library song search);
     *  playback indices stay anchored to the FULL album track list. */
    visibleTrackIds = null,
    /** Mount height in px. Hosts mount at 0 (grow in). A GHOST — the
     *  outgoing panel of a cross-row switch — mounts at the measured
     *  height of the instance it replaces, so its first paint is
     *  identical (the seamless hand-off) and it then plays the plain
     *  close. */
    initialPx = 0,
    /** Two-way: the panel drives it, the grid mirrors it onto the slot
     *  element so the slot's spacing can sync with the height animation
     *  (a fixed grid gap around a closed 0px panel would leave ~2× the
     *  standard row gap below the row — the slot's margin cancels the
     *  gap while closed and transitions with the SAME duration/curve as
     *  the height, so the row below settles at the standard gap without
     *  a jump at either end):  */
    phase = $bindable("closed"),
  }: {
    album: Album;
    /** What this row should show — the section's expanded album when it
     *  is THIS album; null = closed (nothing is expanded, or the section
     *  is expanded on another row: this instance is the outgoing panel
     *  of a cross-row switch and plays the plain collapse). */
    targetId: string | null;
    /** Called when a close finishes (or on a mount that is already
     *  closed). For ghosts the grid removes the ghost row and drains the
     *  queued view travel; for hosts it's a safety drain. */
    onClosed?: () => void;
    /** When set, only these track ids render (library song search);
     *  playback indices stay anchored to the FULL album track list. */
    visibleTrackIds?: Set<string> | null;
    /** Mount height in px (see above). Defaults to 0. */
    initialPx?: number;
    /** Spacing phase, see the prop docs above. Bind with
     *  `bind:phase={...}` in the grid. */
    phase?: "closed" | "open" | "closing";
  } = $props();

  function editAlbumTags(e: MouseEvent, albumId: string) {
    e.preventDefault();
    ui.tagEditor = { open: true, albumId, trackId: null };
  }

  function albumMenu(e: MouseEvent, albumId: string) {
    e.preventDefault();
    const items: MenuItem[] = [
      { label: "Edit album tags…", action: () => (ui.tagEditor = { open: true, albumId, trackId: null }) },
      { label: "Play album next", action: () => void queueTracks(library.tracksOf(albumId).filter((t) => !t.missing).map((t) => t.id), true) },
    ];
    const album = library.albums.find((a) => a.id === albumId);
    if (album?.staged) {
      items.push(
        { label: "Save to library", action: () => void saveImports(albumId) },
        { label: "Discard import", action: () => void discardImports(albumId) },
      );
    }
    if (library.tracksOf(albumId).some((t) => t.missing)) {
      items.push({
        label: "Remove missing tracks",
        action: () => void removeMissingTracks(albumId),
      });
    }
    openContextMenu(e.clientX, e.clientY, items);
  }

  function trackMenu(e: MouseEvent, track: Track) {
    e.preventDefault();
    e.stopPropagation();
    const items: MenuItem[] = [
      { label: "Edit tags…", action: () => (ui.tagEditor = { open: true, albumId: null, trackId: track.id }) },
      { label: "Play next", action: () => void queueTracks([track.id], true) },
      { label: "Add to queue", action: () => void queueTracks([track.id], false) },
    ];
    if (track.staged) {
      items.push(
        { label: "Save to library", action: () => void saveImports(undefined, track.id) },
        { label: "Discard import", action: () => void discardImports(undefined, track.id) },
      );
    }
    if (track.missing) {
      items.push(
        { label: "Locate file…", action: () => void locateMissingTrack(track.id) },
        { label: "Remove from library", action: () => void removeTrack(track.id) },
      );
    }
    openContextMenu(e.clientX, e.clientY, items);
  }

  // Single click selects; a second click on the ALREADY-selected row plays
  // it (the selection gains a consequence; double-click still plays from
  // unselected, and missing rows keep double-click = locate).
  function onTrackClick(track: Track) {
    if (selectedId === track.id) {
      if (track.missing) return;
      selectedId = null; // the current-track styling takes over
      void playTrack(displayAlbum.id, indexById.get(track.id) ?? 0);
      return;
    }
    selectedId = track.id;
  }

  function onTrackDblClick(track: Track) {
    selectedId = track.id;
    if (track.missing) {
      void locateMissingTrack(track.id);
      return;
    }
    void playTrack(displayAlbum.id, indexById.get(track.id) ?? 0);
  }

  // Delete key on a selected staged track = discard that import; on a
  // selected missing track = remove it from the library. Enter on a
  // selected track = play it. Both need the panel open (it stays mounted
  // while collapsed, so a hidden selection must not act) and both stand
  // down while the tag editor or a context menu is up. Enter also ignores
  // interactive targets: a focused row already plays on native Enter
  // (keydown → click → second-click path), so this only covers focus on
  // the body — no double-fire.
  function onKeydown(e: KeyboardEvent) {
    if (e.key !== "Delete" && e.key !== "Enter") return;
    if (targetId === null || ui.tagEditor.open || contextMenu.open) return;
    const id = selectedId;
    if (!id) return;
    const track = tracks.find((t) => t.id === id);
    if (!track) return;
    if (e.key === "Enter") {
      const t = e.target as HTMLElement | null;
      if (t?.closest("input, select, textarea, button, a, [contenteditable]"))
        return;
      if (track.missing) {
        void locateMissingTrack(id);
        return;
      }
      selectedId = null;
      void playTrack(displayAlbum.id, indexById.get(id) ?? 0);
      return;
    }
    if (track.staged) void discardImports(undefined, id);
    else if (track.missing) void removeTrack(id);
  }

  // --- panel expand / collapse / switch — one state machine ---------------
  // Height is driven in px on .inner. Never grid-template-rows: WebKitGTK
  // animates the track and the content clip on different clocks, which read
  // as a lagging shell/gap. Px height is the sanctioned accordion case where
  // no transform equivalent exists — the grid rows really have to move.
  //
  // Rest states: closed = "0px", open settled = "auto" (tracks content
  // changes: the two-column threshold, missing-track alerts, resizes). The
  // markup starts at initialPx (0 for hosts — a fresh mount never flashes
  // open; the measured spawn height for ghosts — see below).
  //
  // The grid owns WHERE the panel row sits (the "host"). Cross-row
  // switches run the two ALBUM animations in parallel: at t=0 the host
  // flips to the destination (token flip → fresh mount → grow) while the
  // outgoing panel is re-mounted as a GHOST row at its own row — a fresh
  // instance at the measured height (initialPx) that plays the plain
  // 280ms CSS close. No pin, no prediction, no frame counting: a close
  // never moves the row ON the line (it plays below that row, in the
  // row's own panel slot), so the one thing that conflicts with a close
  // above the line is the VIEW'S TRAVEL — and the grid defers it behind
  // a per-section queue that drains when the ghost list is empty. Every
  // height move is a CSS transition with the duration set inline per
  // phase, so a click mid-motion retargets the in-flight transition from
  // its current value — nothing restarts (a mid-collapse click on the
  // host album turns the close straight back into a grow).
  //   open grow        0 -> H   360ms cubic-bezier(0.22,1,0.36,1)
  //   plain collapse   H -> 0   280ms, content visible; displayId survives
  //                      (per-section memory — the next open re-shows it);
  //                      onClosed fires at 0 (the grid consumes a
  //                      pending cross-row open, if any)
  //   same-row switch  no height motion: the instance persists in place,
  //                      content swaps + fades; a size mismatch settles
  //                      (or grows a short beat if the new album is much
  //                      taller)

  const CURVE = "cubic-bezier(0.22, 1, 0.36, 1)";
  const GROW_MS = 360;
  const CLOSE_MS = 280;
  // The content the box currently shows — deliberately NOT tracking
  // album (the host): only the state machine below changes it. Captured
  // at mount so $state sees a plain value, not a prop reference.
  // svelte-ignore state_referenced_locally
  const mountAlbumId = album.id;
  // Captured at mount (a prop value, not a reactive binding): the spawn
  // height is a birth-time fact — hosts 0, ghosts the measured px.
  // svelte-ignore state_referenced_locally
  const initInnerH = initialPx > 1 ? `${initialPx}px` : "0px";
  let displayId = $state(mountAlbumId);

  // Same-row switches fade the swapped-in content.
  let entering = $state(false);
  let innerEl = $state<HTMLElement>();
  let panelEl = $state<HTMLElement>();
  let raf1 = 0;
  let raf2 = 0;
  let settleTimer: ReturnType<typeof setTimeout> | undefined;
  let closeTimer: ReturnType<typeof setTimeout> | undefined;
  let enterRaf1 = 0;
  let enterRaf2 = 0;
  let generation = 0;

  function move(px: number, dur: number) {
    const el = innerEl;
    if (!el) return;
    // Pin `auto` (the settled rest state) to its current px first: a
    // transition cannot interpolate from `auto`, so an unpinned shrink
    // jumps to 0 instead of animating. Mid-motion, this pins the current
    // animated value and retargets from it.
    el.style.transition = "none";
    el.style.height = `${el.offsetHeight}px`;
    void el.offsetHeight;
    el.style.transition = `height ${dur}ms ${CURVE}`;
    el.style.height = `${px}px`;
  }

  // Settle an open panel to auto (a px->auto transition can't interpolate,
  // so the transition is dropped in the same no-jank reframe as before).
  function settleToAuto() {
    if (!innerEl) return;
    innerEl.style.transition = "none";
    innerEl.style.height = "auto";
    void innerEl.offsetHeight;
    innerEl.style.removeProperty("transition");
    innerEl.style.removeProperty("height");
  }

  function grow(gen: number) {
    if (gen !== generation || !innerEl) return;
    const to = Math.max(1, panelEl?.offsetHeight ?? 0);
    if (innerEl.offsetHeight >= to - 2) {
      settleToAuto();
      return;
    }
    move(to, GROW_MS);
    settleTimer = setTimeout(() => {
      if (gen !== generation) return;
      settleToAuto();
    }, GROW_MS + 20);
  }

  $effect(() => {
    const gen = ++generation;
    cancelAnimationFrame(raf1);
    cancelAnimationFrame(raf2);
    cancelAnimationFrame(enterRaf1);
    cancelAnimationFrame(enterRaf2);
    clearTimeout(settleTimer);
    clearTimeout(closeTimer);

    if (displayId === targetId) {
      // The content IS what the section has expanded: the box should be
      // open. Covers fresh mounts, re-opens, cross-row switch
      // destinations (grow from 0 + fade, in parallel with the glide) and
      // same-row flips (a settled box settles to the new size — or grows
      // a short beat if the new album is much taller). A mid-close click
      // on the host album retargets the close into this grow — the box
      // simply turns back open.
      if (targetId === null) return; // displayId is never null; TS guard
      raf1 = requestAnimationFrame(() => {
        raf2 = requestAnimationFrame(() => {
          // Flip with the grow, not before: the slot's margin transition
          // must start the same frame the height transition does.
          phase = "open";
          grow(gen);
          if (entering) {
            enterRaf1 = requestAnimationFrame(() => {
              enterRaf2 = requestAnimationFrame(() => {
                if (gen === generation) entering = false;
              });
            });
          }
        });
      });
      return;
    }

    if (targetId === null) {
      // Plain collapse — including the outgoing panel of a cross-row
      // switch (targetId is null while the section is expanded
      // elsewhere): close with the content visible; displayId stays
      // (per-section memory). onClosed fires at 0 — the grid consumes a
      // pending cross-row open, if any. A generation bump (a mid-close
      // click retargeting the close into a grow) cancels both timers, so
      // onClosed can never fire after a retarget.
      entering = false;
      if (innerEl && innerEl.offsetHeight > 2) {
        phase = "closing"; // the slot's margin collapse starts now too
        move(0, CLOSE_MS);
        closeTimer = setTimeout(() => {
          if (gen !== generation) return;
          phase = "closed";
          onClosed?.();
        }, CLOSE_MS + 10);
      } else {
        phase = "closed";
        onClosed?.();
      }
      return;
    }

    // album.id !== displayId: a persistent instance whose host row was
    // re-pointed (same-row switch: the shared row now hosts the new
    // album; or defensive relocation when the section's list changed):
    // swap the content; the state change re-enters the effect on the
    // open branch, which grows.
    displayId = album.id;
    selectedId = null;
    entering = true;
  });

  let displayAlbum = $derived(
    (displayId ? library.albums.find((a) => a.id === displayId) : null) ?? album,
  );

  let allTracks = $derived(library.tracksOf(displayAlbum.id));
  let filtering = $derived(visibleTrackIds !== null && displayAlbum.id === album.id);
  let tracks = $derived(filtering ? allTracks.filter((t) => visibleTrackIds!.has(t.id)) : allTracks);
  let hasMultipleDiscs = $derived(tracks.some((t) => t.disc > 1));
  let artistName = $derived(library.artistOf(displayAlbum)?.name ?? "");
  let totalSec = $derived(tracks.reduce((s, t) => s + t.durationSec, 0));

  // --- artwork-derived gradient, same alpha as --panel-bg ------------------
  let gradient = $state<string | null>(null);

  const PANEL_ALPHA: Record<string, number> = { dark: 0.36, light: 0.3 };

  $effect(() => {
    // Tracked so the gradient regenerates on theme flips too. Runs for
    // ghosts too (targetId null): the ghost must show the outgoing
    // panel's art-derived background while it closes.
    const theme = resolvedTheme();
    const id = displayId;
    const album = library.albums.find((a) => a.id === id);
    const alpha = PANEL_ALPHA[theme] ?? 0.5;

    // Preferred path: colors computed during the scan (Phase 2 M3) — instant,
    // no decode round-trip on first expand.
    if (album?.colorC1 && album.colorC2) {
      gradient = artGradient(album.colorC1, album.colorC2, theme, alpha);
      return;
    }

    // Fallback (fake library / pre-scan albums): per-cover extraction.
    gradient = null;
    const cover = album?.cover;
    if (!cover) return;
    let alive = true;
    extractArtColors(cover).then((colors) => {
      if (!alive) return;
      if (!colors) return;
      const alpha = PANEL_ALPHA[theme] ?? 0.5;
      gradient = gradientFromColors(colors[0], colors[1], theme, alpha);
    });
    return () => {
      alive = false;
    };
  });

  // Selection is by track ID (indices shift around in grouped disc views);
  // "current" is derived from the playback context the same way.
  let selectedId = $state<string | null>(null);

  let indexById = $derived(new Map(allTracks.map((t, i) => [t.id, i] as const)));
  const isCurrent = (id: string) =>
    currentTrack()?.albumId === displayAlbum.id && currentTrack()?.id === id;

  // Multi-disc albums render one block per disc (each with its own 2-column
  // threshold) instead of one continuous list where a disc's tail shares a
  // column with the next disc's head.
  let discGroups = $derived.by(() => {
    if (!hasMultipleDiscs) return [];
    const order: number[] = [];
    const map = new Map<number, typeof tracks>();
    for (const t of tracks) {
      if (!map.has(t.disc)) {
        map.set(t.disc, []);
        order.push(t.disc);
      }
      map.get(t.disc)!.push(t);
    }
    return order.sort((a, b) => a - b).map((disc) => ({ disc, tracks: map.get(disc)! }));
  });

  function fmt(sec: number): string {
    const m = Math.floor(sec / 60);
    const s = Math.floor(sec % 60);
    return `${m}:${String(s).padStart(2, "0")}`;
  }

  // Two-column tracklists are an explicit grid (NOT CSS multicol): WebKit's
  // multicol hit-testing maps points below column 1's last item into column
  // 2, so hovering the dead zone highlighted the wrong track.
  function halfRows(n: number): string {
    return `repeat(${Math.ceil(n / 2)}, auto)`;
  }

</script>

<svelte:window onkeydown={onKeydown} />

<div class="expander" class:closed={phase === "closed"} class:closing={phase === "closing"}>
  <div class="inner" bind:this={innerEl} style:height={initInnerH}>
    <div class="fade" class:entering>
      <section class="panel" bind:this={panelEl} style:background={gradient ?? undefined}>
        {#if displayAlbum.cover}
          <img class="art" src={displayAlbum.cover} alt="" draggable="false" decoding="async" />
        {/if}
        <div class="right">
          <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
          <header role="group" oncontextmenu={(e) => albumMenu(e, displayAlbum.id)}>
            <div class="title-row">
              <h2>{displayAlbum.title}</h2>
              {#if displayAlbum.staged}
                <span class="staged-badge" title="Not saved to the library folder yet">Imported</span>
              {/if}
              <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
              <button
                class="edit-album"
                aria-label={`Edit tags for ${displayAlbum.title}`}
                title="Edit album tags"
                onclick={(e) => editAlbumTags(e, displayAlbum.id)}
              >
                <svg viewBox="0 0 16 16"><path d="M11.3 2.2 L13.8 4.7 L5.5 13 H3 V10.5 Z" fill="none" stroke="currentColor" stroke-linejoin="round" /></svg>
              </button>
              <button
                class="play-all"
                aria-label={`Play ${displayAlbum.title}`}
                title="Play"
                onclick={() => playTrack(displayAlbum.id, 0)}
              >
                <svg viewBox="0 0 16 16"><path d="M5 3 L13 8 L5 13 Z" fill="currentColor" /></svg>
              </button>
            </div>
            <p class="dim">
              {artistName}{displayAlbum.year ? ` · ${displayAlbum.year}` : ""}
            </p>
          </header>

          {#if hasMultipleDiscs}
            <div class="discs">
              {#each discGroups as group (group.disc)}
                <section class="disc">
                  <h3 class="disc-title">Disc {group.disc}</h3>
                  <ol
                    class="tracklist"
                    class:two={group.tracks.length >= 5}
                    style:grid-template-rows={group.tracks.length >= 5
                      ? halfRows(group.tracks.length)
                      : undefined}
                  >
                    {#each group.tracks as track (track.id)}
                      <li>
                        <button
                          class="track"
                          class:current={isCurrent(track.id)}
                          class:selected={selectedId === track.id}
                          title={
                            selectedId === track.id && !track.missing
                              ? "Click again to play (or press Enter)"
                              : undefined
                          }
                          onclick={() => onTrackClick(track)}
                          ondblclick={() => onTrackDblClick(track)}
                          oncontextmenu={(e) => trackMenu(e, track)}
                        >
                                                    <span class="num" class:missing={track.missing} title={track.missing ? "File missing — double-click to locate it" : undefined}>
                            {#if isCurrent(track.id)}
                              <span aria-hidden="true">{playback.isPlaying ? "▶" : "❚❚"}</span
                              ><span class="sr-only">Now playing</span>
                            {:else if track.missing}
                              <svg class="alert" viewBox="0 0 16 16" aria-hidden="true"><path d="M8 2.2 L14.6 13.4 H1.4 Z" fill="none" stroke="currentColor" stroke-linejoin="round" /><path d="M8 6.6 V9.6 M8 11.4 V11.5" stroke="currentColor" stroke-linecap="round" /></svg>
                            {:else}
                              {track.track}
                            {/if}
                          </span>
                          <span class="title">{track.title}</span>
                          <span class="dur">{fmt(track.durationSec)}</span>
                        </button>
                      </li>
                    {/each}
                  </ol>
                </section>
              {/each}
            </div>
          {:else}
            <ol
              class="tracklist"
              class:two={tracks.length >= 8}
              style:grid-template-rows={tracks.length >= 8
                ? halfRows(tracks.length)
                : undefined}
            >
              {#each tracks as track (track.id)}
                <li>
                  <button
                    class="track"
                    class:current={isCurrent(track.id)}
                    class:selected={selectedId === track.id}
                    title={
                      selectedId === track.id && !track.missing
                        ? "Click again to play (or press Enter)"
                        : undefined
                    }
                  onclick={() => onTrackClick(track)}
                  ondblclick={() => onTrackDblClick(track)}
                  oncontextmenu={(e) => trackMenu(e, track)}
                >
                                        <span class="num" class:missing={track.missing} title={track.missing ? "File missing — double-click to locate it" : undefined}>
                      {#if isCurrent(track.id)}
                        <span aria-hidden="true">{playback.isPlaying ? "▶" : "❚❚"}</span
                        ><span class="sr-only">Now playing</span>
                      {:else if track.missing}
                        <svg class="alert" viewBox="0 0 16 16" aria-hidden="true"><path d="M8 2.2 L14.6 13.4 H1.4 Z" fill="none" stroke="currentColor" stroke-linejoin="round" /><path d="M8 6.6 V9.6 M8 11.4 V11.5" stroke="currentColor" stroke-linecap="round" /></svg>
                      {:else}
                        {track.track}
                      {/if}
                    </span>
                    <span class="title">{track.title}</span>
                    <span class="dur">{fmt(track.durationSec)}</span>
                  </button>
                </li>
              {/each}
            </ol>
          {/if}

          <footer class="album-meta"
            >{filtering ? `${tracks.length} of ${allTracks.length} tracks` : `${tracks.length} tracks`} · {fmt(totalSec)}</footer
          >
        </div>
      </section>
    </div>
  </div>
</div>

<style>
  /* Shadow lives on the expander, not the panel: the inner wrapper must stay
     overflow:hidden for the height animation, and that clip cut the panel's
     own box-shadow into sharp corners. The expander is never clipped and its
     box tracks the animated height, so the shadow follows every frame.
     Spacing goes on the expander too — margin inside the box would leave a
     gap between the panel edge and its shadow. The expander itself has NO
     transition: its geometry follows .inner every layout pass, so shell and
     content can never drift apart (the grid-template-rows approach let
     WebKitGTK animate them on different clocks). */
  .expander {
    margin-bottom: 4px;
    border-radius: var(--radius-panel);
    box-shadow: var(--shadow);
    /* The 4px shadow room joins the animation (same clock as the height)
     * so the row below the panel never jumps when the state settles. */
    transition: margin-bottom 360ms cubic-bezier(0.22, 1, 0.36, 1);
  }

  .expander.closing {
    margin-bottom: 0;
    transition: margin-bottom 280ms cubic-bezier(0.22, 1, 0.36, 1);
  }

  .expander.closed {
    margin-bottom: 0;
  }

  .inner {
    overflow: hidden;
    min-height: 0;
    /* No static transition: every move is set inline per phase (320/280/
     * 140ms, cubic-bezier(0.22,1,0.36,1)) so a mid-motion click retargets
     * the in-flight transition instead of restarting it. */
  }

  .fade {
    transition: opacity 0.16s ease-out;
  }

  .fade.entering {
    opacity: 0;
    transition: none;
  }

  .panel {
    display: flex;
    gap: 24px;
    padding: 20px;
    border-radius: var(--radius-panel);
    border: 1px solid var(--border);
    background: var(--panel-bg);
  }

  .art {
    flex: none;
    width: clamp(170px, 24%, 300px);
    aspect-ratio: 1;
    align-self: flex-start;
    object-fit: cover;
    border-radius: var(--radius-cover);
    box-shadow: 0 4px 18px rgba(0, 0, 0, 0.35);
  }

  .right {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  header {
    flex: none;
    padding: 2px 10px 7px;
    border-bottom: 1px solid var(--border);
    margin-bottom: 8px;
  }

  .title-row {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 0;
  }

  .title-row h2 {
    flex: 1;
    min-width: 0;
    margin: 0;
    font-size: 20px;
    font-weight: 700;
    letter-spacing: -0.01em;
    color: var(--text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  /* "Imported" marker for albums with staged (not yet saved) files. */
  .staged-badge {
    flex: none;
    padding: 3px 9px;
    border: 1px solid var(--border);
    border-radius: 999px;
    background: var(--active);
    color: var(--accent);
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.07em;
    text-transform: uppercase;
  }

  .play-all {
    flex: none;
    width: 34px;
    height: 34px;
    border-radius: 50%;
    border: none;
    background: var(--active);
    color: var(--accent);
    display: grid;
    place-items: center;
    cursor: pointer;
    padding: 0;
  }

  /* Always-visible album tag editor entry point (Step 1). */
  .edit-album {
    flex: none;
    /* 28px: the PRODUCT.md minimum hit-target bar (was 26, tuned before
     * the bar was written). */
    width: 28px;
    height: 28px;
    border: none;
    border-radius: 7px;
    background: transparent;
    color: var(--text-dim);
    display: grid;
    place-items: center;
    cursor: pointer;
    padding: 0;
  }

  .edit-album:hover {
    background: var(--hover);
    color: var(--text);
  }

  .edit-album svg {
    width: 13px;
    height: 13px;
  }

  .play-all:hover {
    background: var(--accent);
    color: #fff;
  }

  .play-all svg {
    width: 15px;
    height: 15px;
    translate: 1px 0;
  }

  header p {
    margin: 2px 0 0;
  }

  .album-meta {
    flex: none;
    text-align: right;
    padding: 8px 10px 2px;
    font-size: 12px;
    color: var(--text-dim);
    font-variant-numeric: tabular-nums;
  }

  .tracklist {
    flex: 1;
    min-width: 0;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  /* Explicit column grid — see halfRows() comment. grid-auto-flow: column
     fills column 1 first (same split multicol balanced), DOM order intact. */
  .tracklist.two {
    display: grid;
    grid-auto-flow: column;
    grid-template-columns: 1fr 1fr;
    column-gap: 24px;
  }

  .tracklist li {
    min-width: 0;
  }

  .discs {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .disc-title {
    margin: 0;
    padding: 6px 10px 2px;
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--text-dim);
  }

  .track {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    padding: 7px 10px;
    border: none;
    border-radius: 7px;
    background: transparent;
    color: var(--text);
    font-size: 13px;
    text-align: left;
    /* Clickable (select / play) — same cursor + :active wash as the other
     * pressable families in the chrome. */
    cursor: pointer;
  }

  .track:active {
    background: var(--active);
  }

  .track:hover {
    background: var(--hover);
  }

  /* Single-click selection — neutral highlight, distinct from the accent
     styling of the currently playing track. */
  .track.selected {
    background: var(--active);
  }

  .track.current {
    color: var(--accent);
    font-weight: 600;
  }

  .num {
    flex: none;
    width: 22px;
    text-align: right;
    color: var(--text-dim);
    font-variant-numeric: tabular-nums;
    font-size: 12px;
  }

  /* Missing file: alert triangle in place of the track number. */
  .num .alert {
    width: 13px;
    height: 13px;
    color: #f2a33c;
    display: inline-block;
    vertical-align: -2px;
  }

  .track.current .num {
    color: var(--accent);
  }

  .title {
    flex: 1;
    min-width: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .dur {
    flex: none;
    color: var(--text-dim);
    font-size: 12px;
    font-variant-numeric: tabular-nums;
  }
</style>
