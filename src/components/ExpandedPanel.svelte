<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { fade } from "svelte/transition";
  import { MediaQuery } from "svelte/reactivity";
  import type { Album, Track } from "../lib/types";
  import { library, LIVE_LIBRARY } from "../lib/stores/library.svelte";
  import { playback, playTrack, currentTrack, queueTracks } from "../lib/stores/playback.svelte";
  import { extractArtColors } from "../lib/artColors";
  import { artSrc } from "../lib/artSrc";
  import { artGradient, gradientFromColors } from "../lib/gradient";
  import { tooltip } from "../lib/tooltip";
  import { SPLIT_MIN, discSplitPlan } from "../lib/discSplit";
  import StencilMark from "./StencilMark.svelte";
  import { resolvedTheme, ui } from "../lib/stores/ui.svelte";
  import {
    openContextMenu,
    contextMenu,
    SEP,
    type MenuItem,
  } from "../lib/stores/contextMenu.svelte";
  import {
    saveImports,
    discardImports,
    locateMissingTrack,
    removeTrack,
  } from "../lib/stores/scanner.svelte";
  import { openAlbumMenu } from "../lib/albumMenu";
  import { openImportManager } from "../lib/stores/imports.svelte";

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
    // One builder (lib/albumMenu) for the grid tile and this header, so the
    // two album doors can't drift.
    openAlbumMenu(e, albumId);
  }

  function trackMenu(e: MouseEvent, track: Track) {
    e.preventDefault();
    e.stopPropagation();
    // Right-click selects, like every file browser on the planet: the menu
    // acts on "what you pointed at", and the keyboard verbs that read the
    // selection (Delete = discard/remove, Enter = play) come alive on the
    // row you just looked at instead of whichever one a left-click last
    // touched (owner request, 2026-09-03).
    selectedId = track.id;
    const items: MenuItem[] = [
      { label: "Edit tags…", action: () => { ui.tagEditor = { open: true, albumId: null, trackId: track.id }; } },
      SEP,
      // Three playback verbs, in the order of commitment: NOW interrupts
      // (starts the album AT this track, same index base as row click —
      // allTracks, not the search-filtered view), NEXT inserts before
      // whatever is queued, ADD joins the back. (owner request 2026-09-03;
      // "Play next" was previously the top row of this section.)
      {
        label: "Play now",
        action: () => void playTrack(track.albumId, indexById.get(track.id) ?? 0),
      },
      { label: "Play next", action: () => void queueTracks([track.id], true) },
      { label: "Add to queue", action: () => void queueTracks([track.id], false) },
      SEP,
      // Same door as the album menu's row — Rust-side path resolution,
      // dolphin --select. Fake-library dev mode has no DB rows, so the
      // row only exists where it can actually act.
      ...(LIVE_LIBRARY
        ? [
            {
              label: "Open containing folder",
              action: () =>
                void invoke("reveal_container", { trackId: track.id }).catch((e) =>
                  console.error(e),
                ),
            },
          ]
        : []),
    ];
    if (track.staged || track.missing) items.push(SEP);
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
  // unselected, and missing rows keep double-click = locate). Playing never
  // clears the highlight — Enter keeps it so the arrows can continue from
  // the playing row; only picking another row (or Delete acting on it)
  // moves it. Mouse second-click is the one exception: the toggle needs its
  // off-ramp (keyboard activation arrives with detail 0, mouse with >= 1).
  function onTrackClick(track: Track, e?: MouseEvent) {
    if (selectedId === track.id) {
      if (track.missing) return;
      // Keyboard Enter (detail 0) plays AND holds the highlight so the
      // arrows continue from the playing row; mouse second-click keeps its
      // toggle off-ramp (the current-track styling takes over).
      if (e && e.detail !== 0) selectedId = null;
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
  // ArrowUp/Down walk the highlight through the VISIBLE tracklist instead
  // of scrolling: after selecting a track the list behaves like a listbox.
  // Scope is tight on purpose — body or a track row only. Focus in the
  // search field, on a slider (arrows are theirs), or on any other control
  // keeps native behavior. No selection yet + arrows from the body anchors
  // at the near edge (Down = first track). Ends don't wrap; at an edge the
  // keys fall through to the scroller. Ghost instances stand down via the
  // same targetId guard (their targetId is null).
  function onKeydown(e: KeyboardEvent) {
    if (
      e.key !== "Delete" &&
      e.key !== "Enter" &&
      e.key !== "ArrowUp" &&
      e.key !== "ArrowDown" &&
      e.key !== "ArrowLeft" &&
      e.key !== "ArrowRight"
    )
      return;
    if (targetId === null || ui.tagEditor.open || contextMenu.open) return;
    if (e.key === "ArrowLeft" || e.key === "ArrowRight") {
      // Rows ONLY: on the body Left/Right belong to seek (PlayBar ±5s),
      // and this handler must never steal them — no stopPropagation games,
      // just a scope that never overlaps. Column-major grid: across means
      // ± one column-height within the containing list; out of range (or
      // single column) falls through silently.
      const t = e.target as HTMLElement | null;
      if (!t?.closest("button.track")) return;
      const cur = selectedId;
      if (!cur) return;
      const c = colList(cur);
      if (!c) return;
      const li = c.ids.indexOf(cur);
      if (li === -1) return;
      const ni = li + (e.key === "ArrowLeft" ? -c.rows : c.rows);
      if (ni < 0 || ni >= c.ids.length) return;
      e.preventDefault();
      focusTrack(c.ids[ni]);
      return;
    }
    if (e.key === "ArrowUp" || e.key === "ArrowDown") {
      const t = e.target as HTMLElement | null;
      const onRow = !!t?.closest("button.track");
      if (!onRow && t !== document.body && !t?.closest("main.content")) return;
      const ids = flatIds;
      if (ids.length === 0) return;
      let i = selectedId ? ids.indexOf(selectedId) : -1;
      if (i === -1) {
        // Nothing highlighted: anchor at the edge you're heading in from.
        i = e.key === "ArrowUp" ? ids.length - 1 : 0;
      } else {
        const ni = i + (e.key === "ArrowUp" ? -1 : 1);
        if (ni < 0 || ni >= ids.length) return; // edge: fall through to scroll
        i = ni;
      }
      e.preventDefault();
      focusTrack(ids[i]);
      return;
    }
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
      // Highlight stays: the arrows continue from the playing row.
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
  // mirrored growDur(H) close. No pin, no prediction, no frame counting: a close
  // never moves the row ON the line (it plays below that row, in the
  // row's own panel slot), so the one thing that conflicts with a close
  // above the line is the VIEW'S TRAVEL — and the grid defers it behind
  // a per-section queue that drains when the ghost list is empty. Every
  // height move is a CSS transition with the duration set inline per
  // phase, so a click mid-motion retargets the in-flight transition from
  // its current value — nothing restarts (a mid-collapse click on the
  // host album turns the close straight back into a grow).
  //   open grow        0 -> H   growDur(H): 320–500ms, cubic-bezier(0.22,1,0.36,1)
  //   plain collapse   H -> 0   growDur(H): 320–500ms, content mirrored;
  //                      (per-section memory — the next open re-shows it);
  //                      onClosed fires at 0 (the grid consumes a
  //                      pending cross-row open, if any)
  //   same-row switch  no height motion: the instance persists in place,
  //                      content swaps + fades; a size mismatch settles
  //                      (or grows a short beat if the new album is much
  //                      taller)

  // Token (app.css): the user-verified panel-choreography curve. var()
  // resolves fine in inline styles — .inner lives in the document.
  const CURVE = "var(--ease-out)";
  // Drawer voice for the slot/expander margins (see syncSpacing): the
  // margins move the box WITH the content, so they share the content's
  // curve — an existing token, not a fork.
  // Accordion durations scale with measured travel (px): a fixed duration
  // whips tall panels unreadably fast and dawdles on short ones. Retune
  // 0.16.15 (dated override): the floor rose 200ms → 320ms so short
  // albums read deliberate instead of instant (spread plus floor); tall
  // opens keep slope 0.6, cap 500ms, and fixed content motion. ONE
  // function serves both directions now — the old closeDur (0.5 slope,
  // 400 cap) is retired; exit mirrors entrance on the same clock.
  // Curve stays var(--ease-out): the user-verified panel token, not a fork.
  function growDur(h: number): number {
    return Math.min(500, Math.max(320, Math.round(h * 0.6)));
  }
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

  // --- hybrid open/close choreography (Aina spec, 2026-10-09; retune 0.16.15)
  // Design intent: hybrid, not a pure transform slide. The slot keeps the
  // honest px-height animation (layout really moves); the eye follows
  // compositor-only motion on ONE content wrapper (.glide) inside the
  // .inner clip mask — top edge pinned, content starting 16px up inside
  // the growing box. Exactly one travel layer, never per-row.
  //
  // Transition STRINGS live static in CSS per data-cstate (open:
  // 320ms fade / 380ms travel; closing mirrors it; swap reveals with the
  // open strings); the VALUES ride inline (bindings mirrored to direct
  // styles). Never the reverse: an attribute-selector-driven opacity set
  // in the same batch as its transition string loses the transition on
  // this WebKitGTK. So the state flips a frame BEFORE the values move
  // (double-rAF), and the same-row swap commits its hidden state under a
  // suppressed transition with a forced reflow before the content swaps
  // underneath it. Two-beat stagger = two static delays (header t=0,
  // tracklist + footer +80ms opening; reversed on close), same 320ms
  // durations — no per-row ladder, no --i dials. The shell shadow rides
  // a ::before whose opacity follows the beat-b value (same delays).
  const prefersReducedMotion = new MediaQuery("(prefers-reduced-motion: reduce)");
  // svelte-ignore state_referenced_locally
  const mountRevealed = initialPx > 1 || prefersReducedMotion.current;
  let expanderEl = $state<HTMLElement>();
  let glideEl = $state<HTMLElement>();
  let beatHEl = $state<HTMLElement>();
  let beatBEl = $state<HTMLElement>();
  let cstate = $state<"open" | "closing" | "swap">("open");
  let glideOp = $state(mountRevealed ? 1 : 0);
  let glideTf = $state(mountRevealed ? "none" : "translateY(-16px)");
  let beatOp = $state(mountRevealed ? 1 : 0);
  let shadowOp = $state(mountRevealed ? 1 : 0);
  let innerEl = $state<HTMLElement>();
  let panelEl = $state<HTMLElement>();
  let raf1 = 0;
  let raf2 = 0;
  let settleTimer: ReturnType<typeof setTimeout> | undefined;
  let closeTimer: ReturnType<typeof setTimeout> | undefined;
  let generation = 0;
  // Set by the swap branch, consumed by the open-branch re-entry: the
  // reveal keeps the swap cstate (whose strings equal the open ones since
  // the retune — 320/380, beats 0/+80) instead of flipping to "open".
  let swapPending = false;

  function paintContent(op: number, tf: string, beat: number) {
    glideOp = op;
    glideTf = tf;
    beatOp = beat;
    shadowOp = beat;
    // Synchronous commit: Svelte flushes bindings on its own schedule,
    // so mirror the same values directly — the swap branch's hidden state
    // must be committed (reflow, there) before the content swaps. The
    // shell shadow rides the beat-b value through --shadow-op (opacity-
    // only, geometry still follows the box every layout pass via inset 0).
    if (glideEl) {
      glideEl.style.opacity = String(op);
      glideEl.style.transform = tf;
    }
    if (beatHEl) beatHEl.style.opacity = String(beat);
    if (beatBEl) beatBEl.style.opacity = String(beat);
    if (expanderEl) expanderEl.style.setProperty("--shadow-op", String(beat));
  }

  // The ONE real spacing fix in this pass: slot margin-top (grid-owned,
  // reached via closest — the panel's own slot, never a guess) and
  // expander margin-bottom (owned) take the ACTUAL move duration inline.
  // The static 360/280ms class transitions desynced from the scaled
  // growDur (320–500ms). Set a frame BEFORE the phase flip
  // so the duration never shares a batch with the margin change it times.
  // The static CSS stays as the pre-JS fallback.
  // Curve amendment, dated 2026-10-09 (close-hitch fix): the margins run
  // var(--ease-drawer), NOT the height's ease-out. The margins move the
  // box WITH the content (top edge + content travel sum on the tracked
  // pixel), and ease-out's front-load (~7px of the 20px gap in the first
  // frame-pair, measured) landed as a discrete upward shift ahead of the
  // drawer's slower travel start — the owner's two-phase hitch. On the
  // drawer both edges of the story share one velocity shape, so top edge
  // and content read as a single travel. Durations still match the height
  // move exactly (no jump at either end — that half of the sync law is
  // untouched); heights stay ease-out both ways (the mirrored pair).
  function syncSpacing(dur: number) {
    const t = `${dur}ms var(--ease-drawer)`;
    if (expanderEl) expanderEl.style.transition = `margin-bottom ${t}`;
    const slot = innerEl?.closest(".panel-slot") as HTMLElement | null;
    if (slot) slot.style.transition = `margin-top ${t}`;
  }

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

  function grow(gen: number, preTo?: number, preDur?: number) {
    if (gen !== generation || !innerEl) return;
    const to = preTo ?? Math.max(1, panelEl?.offsetHeight ?? 0);
    if (innerEl.offsetHeight >= to - 2) {
      settleToAuto();
      return;
    }
    // Duration scales with the REMAINING travel (fresh mount = full `to`,
    // mid-close retarget = what's left) so tall panels stay legible and
    // short ones stay crisp — see growDur. The open branch pre-measures
    // `to`/`dur` a frame early (for the spacing sync) and passes them in
    // so the slot margins match THIS move exactly.
    const dur = preDur ?? growDur(to - innerEl.offsetHeight);
    move(to, dur);
    settleTimer = setTimeout(() => {
      if (gen !== generation) return;
      settleToAuto();
    }, dur + 20);
  }

  $effect(() => {
    const gen = ++generation;
    cancelAnimationFrame(raf1);
    cancelAnimationFrame(raf2);
    clearTimeout(settleTimer);
    clearTimeout(closeTimer);
    const rm = prefersReducedMotion.current;

    if (displayId === targetId) {
      // The content IS what the section has expanded: the box should be
      // open. Covers fresh mounts, re-opens, cross-row switch
      // destinations (grow from 0 + reveal, in parallel with the ghost
      // close) and same-row flips (a settled box settles to the new size
      // — or grows a short beat if the new album is much taller). A
      // mid-close click on the host album retargets the close into this
      // grow — the box simply turns back open, from its current px.
      // Content travel starts the SAME frame as the height move and runs
      // shorter than tall grows on purpose (380ms vs up to 500ms).
      if (targetId === null) return; // displayId is never null; TS guard
      raf1 = requestAnimationFrame(() => {
        if (gen !== generation) return;
        // Measure now; the flip below reuses THIS travel so the inline
        // spacing durations (a frame early — never in the flip's batch)
        // match the height move exactly.
        const to = Math.max(1, panelEl?.offsetHeight ?? 0);
        const dur = growDur(Math.max(0, to - (innerEl?.offsetHeight ?? 0)));
        if (!swapPending) cstate = "open";
        if (rm) paintContent(1, "none", 1);
        syncSpacing(dur);
        raf2 = requestAnimationFrame(() => {
          if (gen !== generation) return;
          // Flip with the grow, not before: the slot's margin transition
          // must start the same frame the height transition does.
          phase = "open";
          grow(gen, to, dur);
          if (!rm) paintContent(1, "none", 1);
          swapPending = false;
        });
      });
      return;
    }

    if (targetId === null) {
      // Plain collapse — including the outgoing panel of a cross-row
      // switch (targetId is null while the section is expanded
      // elsewhere): content fades/rises 320/380ms (exact mirror — the
      // upward exit was never eyeballed before this retune, see the probe
      // numbers) while the box closes under growDur(H). Deferred a
      // double-rAF so a ghost close starts the
      // same frame its host's grow does (parallel start, t=0 both).
      // displayId stays (per-section memory). onClosed fires at 0 — the
      // grid drains a pending cross-row glide, if any. A generation bump
      // (a mid-close click retargeting the close into a grow) cancels the
      // frames AND the timer, so onClosed can never fire after a retarget
      // — the guard extends to the content frames above by construction.
      if (innerEl && innerEl.offsetHeight > 2) {
        const dur = growDur(innerEl.offsetHeight);
        raf1 = requestAnimationFrame(() => {
          if (gen !== generation) return;
          cstate = "closing";
          if (rm) paintContent(1, "none", 1);
          syncSpacing(dur);
          raf2 = requestAnimationFrame(() => {
            if (gen !== generation) return;
            phase = "closing";
            move(0, dur);
            if (!rm) paintContent(0, "translateY(-16px)", 0);
            closeTimer = setTimeout(() => {
              if (gen !== generation) return;
              phase = "closed";
              onClosed?.();
            }, dur + 10);
          });
        });
      } else {
        phase = "closed";
        onClosed?.();
      }
      return;
    }

    // album.id !== displayId: a persistent instance whose host row was
    // re-pointed (same-row switch: the shared row now hosts the new
    // album; or defensive relocation when the section's list changed).
    // The incoming block hides INSTANTLY — transition suppressed, hidden
    // committed with a forced reflow — then the content swaps underneath
    // it and the state change re-enters the effect on the open branch,
    // which reveals with the open strings (320/380, beats 0/+80). No height
    // motion unless the new album mismatches in size (the open branch
    // settles or grows a short beat). Reduced motion swaps with final
    // values: no travel, no fade.
    const els = [glideEl, beatHEl, beatBEl];
    for (const el of els) if (el) el.style.transition = "none";
    if (!rm) paintContent(0, "translateY(-6px)", 0);
    else paintContent(1, "none", 1);
    if (glideEl) void glideEl.offsetHeight;
    for (const el of els) if (el) el.style.removeProperty("transition");
    cstate = "swap";
    swapPending = true;
    displayId = album.id;
    selectedId = null;
  });

  // Search-filter (visibleTrackIds) mid-open: the track COUNT changes
  // without an album swap — height re-settles from the current px via the
  // existing move path, the content stays revealed, no fade ever replays.
  // Armed on count change ONLY; album swaps belong to the machine above.
  // Lazy seeds (null/-1): the declarations below (`tracks`) are not
  // initialized yet at this point in the script — the first run only
  // records them, never acts (ids cannot match the seeds).
  let prevTrackLen = -1;
  let prevTrackAlbum: string | null = null;
  $effect(() => {
    const len = tracks.length;
    const d = displayId;
    const ph = phase;
    if (
      d === prevTrackAlbum &&
      len !== prevTrackLen &&
      ph === "open" &&
      innerEl &&
      innerEl.style.height.endsWith("px")
    ) {
      const to = Math.max(1, panelEl?.offsetHeight ?? 0);
      const dur = growDur(Math.max(0, to - innerEl.offsetHeight));
      clearTimeout(settleTimer);
      move(to, dur);
      const g = generation;
      settleTimer = setTimeout(() => {
        if (g !== generation) return;
        settleToAuto();
      }, dur + 20);
    }
    prevTrackLen = len;
    prevTrackAlbum = d;
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
    // The global kill switch (Appearance → Album artwork gradient): off
    // means plain --panel-bg, overrides included — no half-state where a
    // custom color survives the switch its own toggle promised to kill.
    if (!ui.albumGradient) {
      gradient = null;
      return;
    }
    const album = library.albums.find((a) => a.id === id);
    const alpha = PANEL_ALPHA[theme] ?? 0.5;

    // Preferred path: the user's per-album override (Step 9b), then the
    // colors computed during the scan (Phase 2 M3) — instant, no decode
    // round-trip on first expand.
    const override = ui.panelGradients[id];
    if (override) {
      // A corrupt settings entry must not blank the panel: fall through
      // to the artwork colors when the override won't parse.
      const g = artGradient(override.c1, override.c2, theme, alpha);
      if (g) {
        gradient = g;
        return;
      }
    }
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

  // Multi-disc albums render one block per disc instead of one continuous
  // list where a disc's tail shares a column with the next disc's head.
  // Each block carries `rows` — column 1's row count under the album-wide
  // split contract (discSplitPlan), or null to render single-column.
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
    const groups = order
      .sort((a, b) => a - b)
      .map((disc) => ({ disc, tracks: map.get(disc)! }));
    const plan = discSplitPlan(groups.map((g) => g.tracks.length));
    return groups.map((g, i) => ({ ...g, rows: plan[i] }));
  });
  // DOM-truthful flat id order for arrow navigation (group concatenation,
  // never an assumed global sort) + the focused-track handoff.
  let flatIds = $derived(
    hasMultipleDiscs
      ? discGroups.flatMap((g) => g.tracks.map((t) => t.id))
      : tracks.map((t) => t.id),
  );
  function focusTrack(id: string) {
    selectedId = id;
    // By id, not by index: immune to group order, outro cells, rescan
    // re-grouping mid-navigation.
    const btn = panelEl?.querySelector<HTMLElement>(
      `button.track[data-tid="${CSS.escape(id)}"]`,
    );
    btn?.focus({ preventScroll: true });
    btn?.scrollIntoView({ block: "nearest" });
  }
  // The column-height + id list Left/Right moves within: the single list
  // (two-col only — below SPLIT_MIN there is nothing across) or the
  // containing disc group (unsplit groups return null: single column).
  function colList(id: string): { ids: string[]; rows: number } | null {
    if (!hasMultipleDiscs) {
      if (tracks.length < SPLIT_MIN) return null;
      return { ids: tracks.map((t) => t.id), rows: splitRows(tracks.length) };
    }
    const g = discGroups.find((gr) => gr.tracks.some((t) => t.id === id));
    if (!g || g.rows === null) return null;
    return { ids: g.tracks.map((t) => t.id), rows: g.rows };
  }

  function fmt(sec: number): string {
    const m = Math.floor(sec / 60);
    const s = Math.floor(sec % 60);
    return `${m}:${String(s).padStart(2, "0")}`;
  }

  // A row that leaves (discard, remove, rescan re-group) FADES out over
  // 160 ms inside its own cell, and the list closes the gap when it's
  // gone — two legible beats instead of a blink. A row that ARRIVES
  // (watcher adopts a new file) fades in the same 160 ms: mirror, owner
  // rule. The transition sits on the row BUTTON, not the <li>: the list
  // is a grid-auto-flow:column, so an in-flow outro cell is the one thing
  // that must not move — the fading button holds its <li>'s size, the
  // grid does not reflow mid-fade, and the collapse is the deliberate
  // second beat. `local:` — no intro on first mount (the grid entrance
  // owns boot); only store-driven adds and removes animate.
  // prefersReducedMotion lives with the state machine above (mount-time read);
  // ROW_FADE keeps its reactive read here.
  const ROW_FADE = $derived({ duration: prefersReducedMotion.current ? 0 : 160 });

  // Two-column tracklists are an explicit grid (NOT CSS multicol): WebKit's
  // multicol hit-testing maps points below column 1's last item into column
  // 2, so hovering the dead zone highlighted the wrong track.
  //
  // Split threshold (owner call, 2026-09-05, from the Bat Out of Hell (7,
  // single) vs Dead Ringer (8, split 4+4) comparison): the old 8/5 pair put
  // the shape CHANGE between adjacent albums — one track longer and a full
  // list became two half-width stumps floating in the art-tall panel.
  // Short albums don't need the split (the panel is tall anyway — the cover
  // square pins its height), so the two-column shape is reserved for lists
  // that are genuinely long: single column through 8; from 9 the list
  // splits. The 7-cap dominant-column form (9 → 7+2 … 12 → 7+5) exists for
  // SINGLE-disc albums AND for that band only: grid-auto-flow: column
  // spills EVERY cap-sized chunk into a new column, so a big single-disc
  // album capped at 7 grows columns (a 23-track "Forever" measured a
  // 7+7+7+2 four-column octopus the day the cap lost its balance
  // fallback, same day). Single-disc from 13 halves (11–12 stay in the
  // cap band so column one never deals a 6: 11 → 7+4, 12 → 7+5,
  // 13 → 7+6) — the panel's column budget is TWO, always. MULTI-disc albums do not consult these rows at
  // all: their per-disc shape is the album-wide contract in
  // `lib/discSplit.ts` (disc 1 leads with balanced halves, followers
  // split at max(lead, own balance) — owner ruling 2026-09-06, after
  // Ira Dei's 5+5-over-8 and Human.'s 5+4-over-8-over-5+4 read as three
  // unrelated lists).
  const PRE_BALANCE_HEAD = 7;
  const BALANCE_MIN = 13;
  // Column 1's row count (single-disc branch); grid-auto-flow: column
  // fills it before spilling the remainder into column 2.
  function splitRows(n: number): number {
    return n >= BALANCE_MIN ? Math.ceil(n / 2) : Math.min(PRE_BALANCE_HEAD, n);
  }
  function halfRows(n: number): string {
    return `repeat(${splitRows(n)}, auto)`;
  }

</script>

<svelte:window onkeydown={onKeydown} />

<!-- data-cstate + --shadow-op: the ::before shell shadow takes its
     transition delay from the state and its opacity value inline. -->
<div class="expander" bind:this={expanderEl} data-cstate={cstate} style:--shadow-op={shadowOp} class:closed={phase === "closed"} class:closing={phase === "closing"}>
  <div class="inner" bind:this={innerEl} style:height={initInnerH}>
    <!-- .glide: the ONE content-motion wrapper (see the state machine).
         Values ride inline; transition strings come from data-cstate. -->
    <div
      class="glide"
      bind:this={glideEl}
      data-cstate={cstate}
      style:opacity={glideOp}
      style:transform={glideTf}
    >
      <!-- Per-layer background layers: the neutral --panel-bg base under
           the border, the art gradient clipped to the PADDING box.
           The old inline `background: <gradient>` shorthand silently
           reset background-clip to border-box, so the artwork gradient
           painted under the translucent rounded border — the band came
           out as a hard, oversaturated rim (the red arc on the red-
           accented Of Time and Parallels cover; reddest exactly where
           the 135deg ramp is reddest). The shorthand also dropped the
           --panel-bg base the .panel class declares. (owner: "that
           solid line of red looks a bit weird", 2026-09-05)
           The GNOME grain has to be in HERE rather than in app.css with the other
           fields: this element always writes an inline `background`, and an inline
           shorthand beats any stylesheet, so no global rule can reach it.
           `var(--tex)`/`var(--tex-blend)` default to `none`/`normal` at :root, so
           KDE renders this byte-identically to before; layer 2 gets `normal`
           explicitly so the album gradient is not blended with the base. -->
      <section
        class="panel"
        bind:this={panelEl}
        style={`background: var(--tex) padding-box, ${gradient ? `${gradient} padding-box padding-box, ` : ""}var(--panel-bg) border-box; background-blend-mode: var(--tex-blend), normal, normal;`}
      >
        {#if displayAlbum.cover}
          <img class="art" src={artSrc(displayAlbum.cover)} alt="" draggable="false" decoding="async" />
        {:else}
          <div class="art noart"><StencilMark /></div>
        {/if}
        <div class="right">
          <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
          <header class="beat-h" bind:this={beatHEl} style:opacity={beatOp} role="group" oncontextmenu={(e) => albumMenu(e, displayAlbum.id)}>
            <div class="title-row">
              <h2>{displayAlbum.title}</h2>
              {#if displayAlbum.staged}
                <!-- Not a label any more, a door: it opens the import modal AT
                     this album (expand + scroll; the arrival ring was removed
                     0.9.3 — the expanded card is announcement enough) — the
                     same decision home the sidebar's manage row points at,
                     because the destination is what the decision is about and
                     only the modal states it. The grid tile's corner badge
                     stays a label: it lives inside the tile's own button. -->
                <button
                  class="staged-badge"
                  use:tooltip={"Not saved to the library folder yet — open the import list"}
                  onclick={() => void openImportManager(displayAlbum.id)}
                >Imported</button>
              {/if}
              <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
              <button
                class="edit-album"
                aria-label={`Edit tags for ${displayAlbum.title}`}
                use:tooltip={"Edit album tags"}
                onclick={(e) => editAlbumTags(e, displayAlbum.id)}
              >
                <svg viewBox="0 0 16 16"><path d="M11.3 2.2 L13.8 4.7 L5.5 13 H3 V10.5 Z" fill="none" stroke="currentColor" stroke-linejoin="round" /></svg>
              </button>
              <button
                class="play-all"
                aria-label={`Play ${displayAlbum.title}`}
                use:tooltip={"Play"}
                onclick={() => playTrack(displayAlbum.id, 0)}
              >
                <svg viewBox="0 0 16 16"><path d="M5 3 L13 8 L5 13 Z" fill="currentColor" /></svg>
              </button>
            </div>
            <p class="dim">
              {artistName}{displayAlbum.year ? ` · ${displayAlbum.year}` : ""}
            </p>
          </header>

          {#snippet eqIcon(still: boolean)}
            <!-- Mini equalizer in place of the track number: the bouncing
                 and resting layers stay MOUNTED and STACKED, cross-faded by
                 .on — killing the animation outright would teleport, so the
                 play/pause flip dissolves instead. Bars are transform-only
                 (scaleY), currentColor so they follow the row's accent;
                 aria-hidden, the sr-only label carries state. -->
            <span class="eq-swap" aria-hidden="true"
              ><span class="eq" class:on={!still}
                ><i></i><i></i><i></i><i></i></span
              ><span class="eq still" class:on={still}
                ><i></i><i></i><i></i><i></i></span
              ></span
            >
          {/snippet}

          <!-- beat-b: tracklist + footer share the second stagger beat (+50ms). -->
          <div class="beat-b" bind:this={beatBEl} style:opacity={beatOp}>
          {#key displayAlbum.id}
          <!-- Album switch (same-row host flip) swaps the whole tracklist
             block atomically. Without the key, the keyed each treats it as
             remove-all + add-all: every outgoing row plays its 160 ms fade
             IN THE SAME column grid, so for 160 ms the list shows both
             albums' rows — extra phantom columns, scrambled order — before
             settling (the "sort glitch"). Store-driven row adds/removes
             within one album still animate: the key doesn't change, so
             those stay local each-block deltas. -->
          {#if hasMultipleDiscs}
            <div class="discs">
              {#each discGroups as group (group.disc)}
                <section class="disc">
                  <h3 class="disc-title">Disc {group.disc}</h3>
                  <ol
                    class="tracklist"
                    class:two={group.rows !== null}
                    style:grid-template-rows={group.rows !== null
                      ? `repeat(${group.rows}, auto)`
                      : undefined}
                  >
                    {#each group.tracks as track (track.id)}
                      <li>
                        <button
                          class="track"
                          data-tid={track.id}
                          transition:fade|local={ROW_FADE}
                          class:current={isCurrent(track.id)}
                          class:selected={selectedId === track.id}
                          use:tooltip={
                            selectedId === track.id && !track.missing
                              ? "Click again to play (or press Enter)"
                              : undefined
                          }
                          onclick={(e) => onTrackClick(track, e)}
                          ondblclick={() => onTrackDblClick(track)}
                          oncontextmenu={(e) => trackMenu(e, track)}
                        >
                                                    <span class="num" class:missing={track.missing} use:tooltip={track.missing ? "File missing — double-click to locate it" : undefined}>
                            {#if isCurrent(track.id)}
                              {@render eqIcon(!playback.isPlaying)}
                              <span class="sr-only">{playback.isPlaying ? "Now playing" : "Paused"}</span>
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
              class:two={tracks.length >= SPLIT_MIN}
              style:grid-template-rows={tracks.length >= SPLIT_MIN
                ? halfRows(tracks.length)
                : undefined}
            >
              {#each tracks as track (track.id)}
                <li>
                  <button
                    class="track"
                    data-tid={track.id}
                    transition:fade|local={ROW_FADE}
                    class:current={isCurrent(track.id)}
                    class:selected={selectedId === track.id}
                    use:tooltip={
                      selectedId === track.id && !track.missing
                        ? "Click again to play (or press Enter)"
                        : undefined
                    }
                  onclick={(e) => onTrackClick(track, e)}
                  ondblclick={() => onTrackDblClick(track)}
                  oncontextmenu={(e) => trackMenu(e, track)}
                >
                                        <span class="num" class:missing={track.missing} use:tooltip={track.missing ? "File missing — double-click to locate it" : undefined}>
                      {#if isCurrent(track.id)}
                        {@render eqIcon(!playback.isPlaying)}
                        <span class="sr-only">{playback.isPlaying ? "Now playing" : "Paused"}</span>
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
          {/key}

          <footer class="album-meta"
            >{filtering ? `${tracks.length} of ${allTracks.length} tracks` : `${tracks.length} tracks`} · {fmt(totalSec)}</footer
          >
          </div>
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
     geometry transition: its box follows .inner every layout pass, so shell
     and content can never drift apart (the grid-template-rows approach let
     WebKitGTK animate them on different clocks).
     Retune 0.16.15 (dated DESIGN.md Cards amendment): the shadow moved to
     a ::before whose opacity rides the beat-b value (--shadow-op, inline;
     delay 0ms on open+swap+closing (owner call 2026-10-10: the shadow
     rides with no delay — the content beats keep their own 0/+80ms
     stagger). Appearance is opacity-only — no floating unattached
     shadow, no shadow animation beyond opacity, --shadow alphas untouched. */
  .expander {
    position: relative;
    margin-bottom: 4px;
    border-radius: var(--radius-panel);
    /* The 4px shadow room joins the animation (same clock as the height)
     * so the row below the panel never jumps when the state settles.
     * Static duration here is the pre-JS fallback only: syncSpacing()
     * overwrites it inline with the actual move duration every phase
     * change (scaled growDur, 320–500ms). */
    transition: margin-bottom 360ms var(--ease-out);
  }

  .expander::before {
    content: "";
    position: absolute;
    inset: 0;
    border-radius: inherit;
    box-shadow: var(--shadow);
    pointer-events: none;
    opacity: var(--shadow-op, 1);
    transition: opacity 320ms var(--ease-out);
  }

  /* Owner-approved 2026-10-10, GNOME-dark only: the expanded panel casts
     NO shadow here (path intact — inset, radius, --shadow-op plumbing and
     per-cstate delays untouched, so a future return is one line). */
  :global(html[data-de="gnome"][data-theme="dark"]) .expander::before {
    box-shadow: none;
  }

  /* Owner call 2026-10-10: the shadow fades in AS the content moves, no
     delay (was +80ms with beat-b; the beat stagger stays content-only). */
  .expander[data-cstate="open"]::before,
  .expander[data-cstate="swap"]::before {
    transition: opacity 320ms var(--ease-out);
    transition-delay: 0ms;
  }

  .expander[data-cstate="closing"]::before {
    transition: opacity 320ms var(--ease-out);
  }

  .expander.closing {
    margin-bottom: 0;
    transition: margin-bottom 280ms var(--ease-out);
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

  /* Hybrid reveal (2026-10-09; retune 0.16.15): transition STRINGS static
   * per data-cstate (open 320ms fade / 380ms travel; closing mirrors it;
   * swap reveals with the open strings) — the VALUES ride inline from the
   * state machine. Curves are the existing tokens only: ease-out for
   * fades, ease-drawer for content travel. Two-beat stagger = two static
   * delays (header t=0, tracklist + footer +80ms opening; reversed on
   * close), same 320ms durations — no per-row ladder, no --i dials.
   * .glide carries the single travel layer (will-change); the beats fade
   * opacity-only. Reduced motion: the global kill switch zeroes these
   * durations (320/380 content + ::before opacity alike), and the machine
   * mounts final values. */
  .glide {
    will-change: transform, opacity;
  }

  .glide[data-cstate="open"],
  .glide[data-cstate="swap"] {
    transition:
      opacity 320ms var(--ease-out),
      transform 380ms var(--ease-drawer);
  }

  .glide[data-cstate="closing"] {
    transition:
      opacity 320ms var(--ease-out),
      transform 380ms var(--ease-drawer);
  }

  .glide[data-cstate="open"] .beat-h,
  .glide[data-cstate="swap"] .beat-h {
    transition: opacity 320ms var(--ease-out);
  }

  .glide[data-cstate="open"] .beat-b,
  .glide[data-cstate="swap"] .beat-b {
    transition: opacity 320ms var(--ease-out);
    transition-delay: 80ms;
  }

  .glide[data-cstate="closing"] .beat-b {
    transition: opacity 320ms var(--ease-out);
  }

  .glide[data-cstate="closing"] .beat-h {
    transition: opacity 320ms var(--ease-out);
    transition-delay: 80ms;
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

  /* No cover: the stencil (see AlbumGrid — same quiet room, same tint). */
  .art.noart {
    display: grid;
    place-items: center;
    background: var(--panel-bg);
    overflow: hidden;
  }

  .art.noart :global(.stencil-mark) {
    width: 56%;
    height: auto;
    color: var(--text-dim);
    opacity: 0.3;
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
    font-size: 22px;
    font-weight: 700;
    letter-spacing: -0.01em;
    color: var(--text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  /* "Imported" marker for albums with staged (not yet saved) files — a
     DOOR (it opens the import manager at this album), so it earns control
     geometry: the 28px bar PRODUCT.md sets for clickable things, paid
     vertically so the pill stays a pill. Deliberately a quiet sibling of
     Play: text-dim until touched — in this header only the play circle
     wears the accent (critique P2 + cluster ruling, 2026-09-04). */
  .staged-badge {
    flex: none;
    /* A button now (it opens the modal): reset the UA chrome, keep the pill. */
    font: inherit;
    cursor: pointer;
    padding: 7px 9px;
    border: 1px solid var(--border);
    border-radius: 999px;
    background: transparent;
    color: var(--text-dim);
    font-size: 12px;
    font-weight: 600;
    letter-spacing: 0.07em;
    text-transform: uppercase;
  }

  /* Respond on hover/press like every other control — the pill answers
     because it does something now. */
  .staged-badge:hover {
    border-color: var(--accent);
    color: var(--text);
  }

  .staged-badge:active {
    transform: scale(0.96);
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
    color: var(--accent-text, #fff);
  }

  .play-all svg {
    width: 15px;
    height: 15px;
    /* transform, not standalone `translate` — see Sidebar .search-clear. */
    transform: translateX(1px);
  }

  header p {
    margin: 2px 0 0;
  }

  .album-meta {
    flex: none;
    text-align: right;
    padding: 8px 10px 2px;
    font-size: 14px;
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
    /* The list GROWS to fill .right (flex: 1) — the panel is as tall as its
     * cover art (clamp width → 300px square in a wide window), so a short
     * two-column list sits in a box taller than its content. Grid auto tracks
     * STRETCH to absorb that slack: 8 tracks rowed out over a 300px panel
     * with 40px air between the rows. Pin the rows to the top; the slack
     * belongs below the list, not between the rows. */
    align-content: start;
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
    font-size: 13px;
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
    font-size: 15px;
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

  /* The focus family joins at the door (critique P1): the sidebar, the
     import window and the tag modals all ring keyboard focus in accent;
     the panel's own controls were falling back to the UA's generic
     outline — two dialects on one screen. The ring follows each control's
     own rounding, so one rule fits all four shapes. */
  .track:focus-visible,
  .staged-badge:focus-visible,
  .edit-album:focus-visible,
  .play-all:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  /* Single-click selection — the wash PLUS an inset border ring: the wash
     alone collided with :active (pressing a selected row showed nothing,
     and a mid-press on any other row read as "selected" in a still frame).
     The ring makes selection a state, the wash stays the moment. Still
     neutral, still distinct from the accent of the playing row. */
  .track.selected {
    background: var(--active);
    box-shadow: inset 0 0 0 1px var(--border);
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
    font-size: 14px;
  }

  /* Missing file: alert triangle in place of the track number. */
  .num .alert {
    width: 13px;
    height: 13px;
    color: var(--caution);
    display: inline-block;
    vertical-align: -2px;
  }

  .track.current .num {
    color: var(--accent);
  }

  /* Mini equalizer replacing the current track's number: four 2px bars in
     the 22px number slot, bouncing on staggered transform-only loops while
     playing, frozen while paused — the two layers cross-fade so the flip
     dissolves instead of teleporting. Reduced motion: static bars, instant
     flip. */
  .eq-swap {
    position: relative;
    display: inline-flex;
    width: 14px;
    height: 12px;
  }
  .eq {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: flex-end;
    gap: 2px;
    opacity: 0;
    transition: opacity 560ms var(--ease-out);
  }
  .eq.on {
    opacity: 1;
  }
  .eq:not(.on) i {
    animation-play-state: paused;
  }
  .eq i {
    width: 2px;
    height: 12px;
    border-radius: var(--radius-pill);
    background: currentColor;
    transform-origin: bottom;
    animation: eq-loop 0.9s ease-in-out infinite;
  }
  .eq i:nth-child(2) {
    animation-duration: 0.7s;
    animation-delay: -0.25s;
  }
  .eq i:nth-child(3) {
    animation-duration: 1.1s;
    animation-delay: -0.55s;
  }
  .eq i:nth-child(4) {
    animation-duration: 0.8s;
    animation-delay: -0.4s;
  }
  .eq.still i {
    animation: none;
    transform: scaleY(0.35);
  }
  .eq.still i:nth-child(2) {
    transform: scaleY(0.7);
  }
  .eq.still i:nth-child(3) {
    transform: scaleY(0.5);
  }
  @keyframes eq-loop {
    0%,
    100% {
      transform: scaleY(0.25);
    }
    50% {
      transform: scaleY(1);
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .eq {
      transition: none;
    }
    .eq i {
      animation: none;
      transform: scaleY(0.45);
    }
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
    font-size: 14px;
    font-variant-numeric: tabular-nums;
  }
</style>
