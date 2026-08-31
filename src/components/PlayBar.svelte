<script lang="ts">
  import {
    playback,
    currentTrack,
    togglePlay,
    skip,
    seekTo,
    setVolume,
    toggleMute,
    cycleShuffle,
    cycleRepeat,
    albumSkip,
  } from "../lib/stores/playback.svelte";
  import { library } from "../lib/stores/library.svelte";
  import Toggle from "./Toggle.svelte";
  import SurfaceClose from "./SurfaceClose.svelte";
  import { ui, resolvedTheme } from "../lib/stores/ui.svelte";
  import {
    setEqEnabled,
    setEqPreamp,
    setEqBand,
    queueRemove,
    queueJump,
    queueClear,
  } from "../lib/stores/playback.svelte";
  import { artGradientContrast } from "../lib/gradient";
  import {
    EQ_BANDS,
    EQ_MAX_DB,
    fmtDb,
    fmtHz,
  } from "../lib/eq";

  let track = $derived(currentTrack());
  let album = $derived(track ? library.albums.find((a) => a.id === track!.albumId) : null);
  let artistName = $derived(album ? (library.artistOf(album)?.name ?? "") : "");

  // Step 2b: optional artwork-gradient playbar backdrop at chrome alpha 0.7.
  // Shows whenever a track is loaded (paused included); stopped = plain chrome.
  // Stops are lightness-clamped so text/icons always clear WCAG contrast.
  let backdrop = $derived.by(() => {
    if (!ui.playbarGradient || !album?.colorC1 || !album?.colorC2) return null;
    return artGradientContrast(album.colorC1, album.colorC2, resolvedTheme(), 0.7);
  });

  function fmt(sec: number): string {
    if (!Number.isFinite(sec)) return "0:00";
    const m = Math.floor(sec / 60);
    const s = Math.floor(sec % 60);
    return `${m}:${String(s).padStart(2, "0")}`;
  }

  // Step 7a: queue popover rows — resolve track/album metadata for the ids
  // Rust mirrors over. Missing rows (rescan race) render as "Unknown".
  type QRow = { trackId: string; albumId: string; pos: number; title: string; sub: string };

  function toRows(entries: { trackId: string; albumId: string; albumIndex: number }[]): QRow[] {
    return entries.map((q, pos) => {
      const t = library.tracksOf(q.albumId)[q.albumIndex];
      const al = library.albums.find((a) => a.id === q.albumId);
      return {
        trackId: q.trackId,
        albumId: q.albumId,
        pos,
        title: t?.title ?? "Unknown track",
        sub: al ? `${library.artistOf(al)?.name ?? ""} — ${al.title}` : "",
      };
    });
  }

  let queueRows = $derived(toRows(playback.queue));
  let upNextRows = $derived(toRows(playback.upNext));

  // Step 8c: the EQ and queue popovers share one anchor, so they are
  // mutually exclusive — opening one closes the other (no overlap).
  function toggleEq() {
    ui.eqOpen = !ui.eqOpen;
    if (ui.eqOpen) ui.queueOpen = false;
  }

  // The popover's escape hatch: everything the equalizer owns that is NOT the
  // curve lives in the sidebar's Playback pane, and the EQ button is where the
  // user already was — so the popover offers the way there instead of
  // duplicating the controls poorly. Focus handoff matches the stack's own
  // (preventScroll: `.stack` is a scroll port and the layer is still sliding).
  function openPlaybackSettings() {
    ui.eqOpen = false;
    ui.menuSub = null;
    ui.menuOpen = true;
    ui.menuDetail = "playback";
    requestAnimationFrame(() =>
      document
        .querySelector<HTMLElement>(".layer.detail .backchev")
        ?.focus({ preventScroll: true }),
    );
  }
  function toggleQueue() {
    ui.queueOpen = !ui.queueOpen;
    if (ui.queueOpen) ui.eqOpen = false;
  }

  // Dismissal returns focus to the button that opened the surface, so Tab
  // continues from where the user was instead of restarting at <body>. No
  // preventScroll needed (the playbar is never scrolled) but it costs nothing.
  function closeEqPop() {
    ui.eqOpen = false;
    eqBtn?.focus({ preventScroll: true });
  }
  function closeQueuePop() {
    ui.queueOpen = false;
    qBtn?.focus({ preventScroll: true });
  }

  // Step 8c: standard popover dismissal — outside click + Escape, the same
  // idiom as ContextMenu (svelte:document pointerdown + el.contains).
  let eqEl = $state<HTMLElement>();
  let qEl = $state<HTMLElement>();
  let eqBtn = $state<HTMLElement>();
  let qBtn = $state<HTMLElement>();

  function onDocPointerDown(e: PointerEvent) {
    const t = e.target as HTMLElement;
    // The toggle's own pointerdown must NOT close the popover here — the
    // following click must still see the popover as open so it can toggle
    // it CLOSED (otherwise the button becomes open-only).
    if (eqBtn?.contains(t) || qBtn?.contains(t)) return;
    if (ui.eqOpen && !eqEl?.contains(t)) ui.eqOpen = false;
    if (ui.queueOpen && !qEl?.contains(t)) ui.queueOpen = false;
  }
  // Step 8c: in-app keyboard transport (shape: Space = play/pause,
  // ←/→ = seek ±5s). Fires only when focus is on the body — any interactive
  // element keeps its native key behavior (typing in search, arrows driving
  // a focused slider, Space activating a focused button). Space ignores
  // auto-repeat (held key would toggle play/pause in a loop); arrows repeat
  // on purpose, like dragging the seekbar.
  function isInteractive(t: EventTarget | null): boolean {
    return (
      t instanceof HTMLElement &&
      !!t.closest("input, select, textarea, button, a, [contenteditable]")
    );
  }
  function onDocKeydown(e: KeyboardEvent) {
    if (e.key === "Escape" && (ui.eqOpen || ui.queueOpen)) {
      ui.eqOpen = false;
      ui.queueOpen = false;
      return;
    }
    if (e.repeat && e.key === " ") return;
    if (isInteractive(e.target)) return;
    if (e.key === " ") {
      e.preventDefault();
      void togglePlay();
    } else if (e.key === "ArrowLeft" || e.key === "ArrowRight") {
      if (!playback.current) return;
      e.preventDefault();
      seekTo(playback.positionSec + (e.key === "ArrowRight" ? 5 : -5));
    }
  }
</script>

<svelte:window onpointerdown={onDocPointerDown} onkeydown={onDocKeydown} />

<footer class="playbar glass" style:background={backdrop ?? undefined}>
  <div class="now">
    {#if album?.cover}
      <img class="art" src={album.cover} alt="" draggable="false" />
    {:else}
      <div class="art placeholder"></div>
    {/if}
    <div class="text">
      <span class="t">{track?.title ?? "Nothing playing"}</span>
      <span class="sub">{track ? `${artistName} — ${album?.title}` : "Pick a track"}</span>
    </div>
  </div>

  <div class="center">
    <div class="transport">
      <button
        aria-label="Previous album"
        title="Previous album"
        disabled={!track}
        onclick={() => albumSkip(-1)}
      >
        <svg viewBox="0 0 16 16" aria-hidden="true"><path d="M3 3 v10" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"/><path d="M13.5 3.5 v9 L6.5 8 Z" fill="currentColor"/></svg>
      </button>
      <button aria-label="Previous track" disabled={!track} onclick={() => skip(-1)}>
        <!-- Bare triangle on purpose: track-level step. Bar+triangle is
         *reserved* for album-level jumps (the weight diff encodes it). -->
        <svg viewBox="0 0 16 16" aria-hidden="true"><path d="M12.5 3 L6.5 8 l6 5 Z" fill="currentColor" /></svg>
      </button>
      <button
        class="playpause"
        aria-label={playback.isPlaying ? "Pause" : "Play"}
        title={playback.isPlaying ? "Pause — Space" : "Play — Space"}
        onclick={togglePlay}
      >
        {#if playback.isPlaying}
          <svg viewBox="0 0 16 16" aria-hidden="true"><rect x="4" y="3" width="2.8" height="10" rx="1" fill="currentColor"/><rect x="9.2" y="3" width="2.8" height="10" rx="1" fill="currentColor"/></svg>
        {:else}
          <svg viewBox="0 0 16 16" aria-hidden="true"><path d="M5 3 L13 8 L5 13 Z" fill="currentColor"/></svg>
        {/if}
      </button>
      <button aria-label="Next track" disabled={!track} onclick={() => skip(1)}>
        <svg viewBox="0 0 16 16" aria-hidden="true"><path d="M3.5 3 L9.5 8 l-6 5 Z" fill="currentColor" /></svg>
      </button>
      <button
        aria-label="Next album"
        title="Next album"
        disabled={!track}
        onclick={() => albumSkip(1)}
      >
        <svg viewBox="0 0 16 16" aria-hidden="true"><path d="M13 3 v10" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"/><path d="M2.5 3.5 v9 L9.5 8 Z" fill="currentColor"/></svg>
      </button>
    </div>
    <div class="seek">
      <span class="time">{fmt(playback.positionSec)}</span>
      <input
        type="range"
        min="0"
        max={Math.max(1, playback.durationSec)}
        step="0.5"
        value={playback.positionSec}
        disabled={!track}
        oninput={(e) => seekTo(+e.currentTarget.value)}
        aria-label="Seek"
        title="Seek — ←/→ = ±5s"
      />
      <span class="time">{fmt(playback.durationSec)}</span>
    </div>
  </div>

  <div class="side">
    <div class="modes">
      <button
        class="mode-btn"
        class:on={playback.eq.enabled}
        bind:this={eqBtn}
        aria-label="Equalizer"
        title={`Equalizer: ${playback.eq.enabled ? "on" : "off"}${playback.eq.preset ? ` · ${playback.eq.preset}` : " · Custom"}`}
        onclick={toggleEq}
      >
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <line x1="4" y1="21" x2="4" y2="14" />
          <line x1="4" y1="10" x2="4" y2="3" />
          <line x1="12" y1="21" x2="12" y2="12" />
          <line x1="12" y1="8" x2="12" y2="3" />
          <line x1="20" y1="21" x2="20" y2="16" />
          <line x1="20" y1="12" x2="20" y2="3" />
          <line x1="1" y1="14" x2="7" y2="14" />
          <line x1="9" y1="8" x2="15" y2="8" />
          <line x1="17" y1="16" x2="23" y2="16" />
        </svg>
      </button>
      <button
        class="mode-btn"
        class:on={playback.shuffle !== "off"}
        aria-label="Shuffle"
        title={`Shuffle: ${playback.shuffle === "off" ? "off" : playback.shuffle === "album" ? "album" : playback.shuffle === "artist" ? "artist" : "all artists"} (click to cycle)`}
        onclick={cycleShuffle}
      >
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <polyline points="16 3 21 3 21 8" />
          <line x1="4" y1="20" x2="21" y2="3" />
          <polyline points="21 16 21 21 16 21" />
          <line x1="15" y1="15" x2="21" y2="21" />
          <line x1="4" y1="4" x2="9" y2="9" />
        </svg>
        {#if playback.shuffle === "album"}
          <span class="badge">A</span>
        {:else if playback.shuffle === "artist"}
          <span class="badge">R</span>
        {:else if playback.shuffle === "all"}
          <span class="badge">All</span>
        {/if}
      </button>
      <button
        class="mode-btn"
        class:on={playback.repeat !== "off"}
        aria-label="Repeat"
        title={`Repeat: ${playback.repeat === "off" ? "off" : playback.repeat} (click to cycle)`}
        onclick={cycleRepeat}
      >
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <polyline points="17 1 21 5 17 9" />
          <path d="M3 11V9a4 4 0 0 1 4-4h14" />
          <polyline points="7 23 3 19 7 15" />
          <path d="M21 15v2a4 4 0 0 1-4 4H3" />
        </svg>
        {#if playback.repeat === "track"}
          <span class="badge one">1</span>
        {/if}
      </button>
      <button
        class="mode-btn"
        class:on={playback.queue.length > 0}
        aria-label="Queue"
        title={`Queue: ${playback.queue.length} track${playback.queue.length === 1 ? "" : "s"} queued`}
        bind:this={qBtn}
        onclick={toggleQueue}
      >
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <!-- Sized to the same content span as the other mode glyphs
               (was 15.5x14.5 in the 24 box vs 17-22x18-22 — read small). -->
          <line x1="4" y1="4" x2="14" y2="4" />
          <line x1="4" y1="11.5" x2="14" y2="11.5" />
          <line x1="4" y1="19" x2="10" y2="19" />
          <circle cx="16.5" cy="18" r="3.5" />
          <line x1="20" y1="3.5" x2="20" y2="18" />
        </svg>
        {#if playback.queue.length > 0}
          <span class="badge count">{playback.queue.length}</span>
        {/if}
      </button>
    </div>
    <div class="volume">
    <button
      class="vol-btn"
      aria-label={playback.volume === 0 ? "Unmute" : "Mute"}
      onclick={toggleMute}
    >
      {#if playback.volume === 0}
        <svg viewBox="0 0 16 16" aria-hidden="true">
          <path d="M2.5 6 H4.8 L8 3 v10 L4.8 10 H2.5 Z" fill="currentColor" />
          <!-- 1.6 = media-family stroke weight (skip bars); the old 1.3 read
           * visibly thinner two clusters over. -->
          <path d="M10.5 6 l4 4 M14.5 6 l-4 4" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"/>
        </svg>
      {:else}
        <svg viewBox="0 0 16 16" aria-hidden="true">
          <path d="M2.5 6 H4.8 L8 3 v10 L4.8 10 H2.5 Z" fill="currentColor" />
          <path d="M10.5 5.5 a3.4 3.4 0 0 1 0 5 M12.3 4 a5.8 5.8 0 0 1 0 8" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"/>
        </svg>
      {/if}
    </button>
    <input
      type="range"
      min="0"
      max="100"
      step="1"
      value={playback.volume}
      oninput={(e) => setVolume(+e.currentTarget.value)}
      aria-label="Volume"
    />
    </div>
  </div>

  {#if ui.eqOpen}
    <div class="eq-pop glass" bind:this={eqEl} role="dialog" aria-label="Equalizer">
      <!-- Trimmed (2026-08-31): the Playback pane owns enable + preset, so the
           popover shows the preset NAME read-only and hands the rest over. Its
           native checkbox and <select> were the last two widgets outside the
           design system — the select's open menu is the toolkit's own chrome,
           the same objection AGENTS.md records for GTK file choosers. What stays
           is the popover's unique value: shaping the curve while something
           plays, from the button that is already under the cursor. -->
      <header>
        <SurfaceClose label="Close equalizer" onclick={closeEqPop} />
        <button class="route" onclick={openPlaybackSettings}>
          <span>Playback settings</span>
          <svg viewBox="0 0 10 14" aria-hidden="true">
            <path d="M3 2 L7 7 L3 12" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
        </button>
      </header>
      <!-- Three bands, top to bottom: chrome (dot + route), the curve — the one
           thing this surface does that the pane doesn't — and ownership (is it
           on, which preset). The row sits at the BASE, inverting the Playback
           pane's gate-above ordering: a form is read top-down, an instrument
           surface keeps its master at the base. Deliberate, see DESIGN.md.
           When off, the curve sleeps and the checkbox row stays full contrast —
           dimming the gate would hide the only way out. -->
      <div class="bands" class:asleep={!playback.eq.enabled}>
        <label class="band preamp">
          <span class="db">{fmtDb(playback.eq.preampDb)}</span>
          <span class="slot">
            <input
              class="vert"
              type="range"
              min={-EQ_MAX_DB}
              max={EQ_MAX_DB}
              step="0.5"
              value={playback.eq.preampDb}
              ondblclick={() => setEqPreamp(0)}
              oninput={(e) => setEqPreamp(+e.currentTarget.value)}
              aria-label="Preamp"
            />
          </span>
          <span class="hz">pre</span>
        </label>
        {#each EQ_BANDS as hz, i}
          <label class="band">
            <span class="db">{fmtDb(playback.eq.gains[i])}</span>
            <span class="slot">
              <input
                class="vert"
                type="range"
                min={-EQ_MAX_DB}
                max={EQ_MAX_DB}
                step="0.5"
                value={playback.eq.gains[i]}
                ondblclick={() => setEqBand(i, 0)}
                oninput={(e) => setEqBand(i, +e.currentTarget.value)}
                aria-label={`${fmtHz(hz)} Hz`}
                title={`${fmtHz(hz)} Hz · double-click to zero`}
              />
            </span>
            <span class="hz">{fmtHz(hz)}</span>
          </label>
        {/each}
      </div>
      <footer class="own">
        <Toggle
          small
          checked={playback.eq.enabled}
          label="Equalizer"
          onchange={setEqEnabled}
        />
        <span class="preset">{playback.eq.preset ?? "Custom"}</span>
      </footer>
    </div>
  {/if}
  {#if ui.queueOpen}
    <div class="q-pop glass" bind:this={qEl} role="dialog" aria-label="Queue">
      <header>
        <SurfaceClose label="Close queue" onclick={closeQueuePop} />
        <span class="q-title">Queue</span>
        {#if playback.queue.length}
          <span class="q-count">{playback.queue.length}</span>
          <button
            class="q-clear"
            aria-label="Clear queue"
            title="Remove all queued tracks"
            onclick={() => void queueClear()}
          >
            Clear
          </button>
        {/if}
      </header>
      {#if queueRows.length === 0 && upNextRows.length === 0}
        <p class="q-empty">Nothing queued. Right-click a track → “Play next” or “Add to queue”.</p>
      {:else}
        {#if queueRows.length > 0}
          <ul class="q-list">
            {#each queueRows as row (row.trackId)}
              <li>
                <button class="q-row" title="Play now" onclick={() => void queueJump(row.pos)}>
                  <span class="q-name">{row.title}</span>
                  <span class="q-sub">{row.sub}</span>
                </button>
                <button class="q-x" aria-label="Remove from queue" onclick={() => void queueRemove(row.pos)}>
                  <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M18 6 6 18 M6 6 l12 12"/></svg>
                </button>
              </li>
            {/each}
          </ul>
        {:else}
          <p class="q-empty">Queue is empty.</p>
        {/if}
        {#if upNextRows.length > 0}
          <p class="q-next-h">Up next</p>
          <ul class="q-list preview">
            {#each upNextRows as row (row.trackId)}
              <li>
                <span class="q-row">
                  <span class="q-name">{row.title}</span>
                  <span class="q-sub">{row.sub}</span>
                </span>
              </li>
            {/each}
          </ul>
        {/if}
      {/if}
    </div>
  {/if}
</footer>

<style>
  .playbar {
    position: absolute;
    left: 0;
    right: 0;
    bottom: 0;
    z-index: 10;
    display: grid;
    grid-template-columns: minmax(180px, 1fr) minmax(360px, 2fr) minmax(140px, 1fr);
    align-items: center;
    gap: 20px;
    height: var(--playbar-h);
    flex: none;
    padding: 0 18px;
    border-top: 1px solid var(--border);
    user-select: none;
    /* The artwork-gradient swap is a state response, not a scene change:
       inside the 300ms UI budget, strong ease-out. */
    transition: background 280ms var(--ease-out);
  }

  .now {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 0;
  }

  .art {
    width: 48px;
    height: 48px;
    border-radius: 7px;
    object-fit: cover;
    box-shadow: 0 2px 10px rgba(0, 0, 0, 0.35);
    flex: none;
  }

  .art.placeholder {
    background: var(--hover);
  }

  .text {
    display: flex;
    flex-direction: column;
    gap: 1px;
    min-width: 0;
  }

  .text .t {
    font-size: 13px;
    font-weight: 600;
    color: var(--text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .text .sub {
    font-size: 11.5px;
    color: var(--text-dim);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .center {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    min-width: 0;
  }

  .transport {
    display: flex;
    align-items: center;
    gap: 14px;
  }

  .transport button {
    border: none;
    background: transparent;
    color: var(--text-dim);
    width: 30px;
    height: 30px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    cursor: pointer;
    padding: 0;
  }

  .transport button svg {
    width: 17px;
    height: 17px;
  }

  .transport button:hover:not(:disabled) {
    color: var(--text);
    background: var(--hover);
  }

  .transport button:disabled {
    opacity: 0.35;
    cursor: default;
  }

  .playpause {
    background: var(--active) !important;
    /* The wash is translucent — normal text color reads best on it;
     * --accent-text is for SOLID accent fills only. */
    color: var(--text) !important;
  }

  .seek {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    max-width: 520px;
  }

  .seek input {
    flex: 1;
    accent-color: var(--accent);
    height: 14px;
  }

  .time {
    font-size: 11px;
    color: var(--text-dim);
    font-variant-numeric: tabular-nums;
    flex: none;
    width: 36px;
  }

  .time:first-child {
    text-align: right;
  }

  .volume {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .side {
    display: flex;
    align-items: center;
    gap: 12px;
    justify-content: flex-end;
  }

  .modes {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .mode-btn {
    position: relative;
    display: grid;
    place-items: center;
    width: 30px;
    height: 30px;
    border: none;
    border-radius: 8px;
    background: transparent;
    /* OFF: clearly dimmed */
    color: var(--text-dim);
    opacity: 0.75;
    cursor: pointer;
  }

  .mode-btn:hover {
    background: var(--hover);
    color: var(--text);
    opacity: 1;
  }

  /* ON: full-brightness glyph (the hover color), no wash — the dim→bright
   * flip plus the stage badge reads clearly. */
  .mode-btn.on {
    color: var(--text);
    opacity: 1;
  }

  .mode-btn.on:hover {
    background: var(--hover);
  }

  .mode-btn svg {
    width: 16px;
    height: 16px;
    fill: none;
    stroke: currentColor;
    stroke-width: 2;
    stroke-linecap: round;
    stroke-linejoin: round;
  }

  .mode-btn .badge {
    position: absolute;
    right: 2px;
    bottom: 1px;
    font-size: 8px;
    font-weight: 700;
    line-height: 1;
    letter-spacing: 0.02em;
    color: var(--accent);
  }

  .mode-btn .badge.one {
    right: 4px;
    bottom: 2px;
    font-size: 9px;
  }

  .vol-btn {
    /* Step 8c: 28px minimum hit target (was ~19px) — same bar as the
     * sidebar back-chevron. The button is invisible at rest, so the
     * larger box doesn't change the visual. */
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    padding: 0;
    border: none;
    background: transparent;
    color: var(--text-dim);
    cursor: pointer;
  }

  .vol-btn:hover {
    color: var(--text);
  }

  .vol-btn svg {
    width: 15px;
    height: 15px;
    flex: none;
  }

  .volume input {
    width: 110px;
    accent-color: var(--accent);
  }

  /* --- Equalizer popover (Step 6) ------------------------------------------ */

  /* Step 8c: materialize on entry — the popovers sit above their trigger
   * buttons, so they grow from the bottom-right corner (transform-origin on
   * the shared edge). Transform+opacity only = compositor-friendly; the
   * global prefers-reduced-motion switch collapses the duration. */
  .eq-pop,
  .q-pop {
    transform-origin: 100% 100%;
    animation: pop-in 140ms var(--ease-out);
  }

  @keyframes pop-in {
    from {
      opacity: 0;
      transform: scale(0.98);
    }
  }

  .eq-pop {
    position: absolute;
    right: 14px;
    bottom: calc(100% + 10px);
    z-index: 30;
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 14px 16px;
    border-radius: 12px;
    border: 1px solid var(--border);
    box-shadow: 0 8px 28px rgba(0, 0, 0, 0.35);
    user-select: none;
  }

  .eq-pop header {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .eq-pop header {
    justify-content: space-between;
  }

  /* Route out of the popover: same action tier as the panes' footer rows (0.82
     of --text, neutral hover wash, accent ring), because leaving a surface is a
     real action, not decoration. */
  .eq-pop .route {
    display: flex;
    align-items: center;
    gap: 6px;
    border: none;
    border-radius: 8px;
    background: transparent;
    color: var(--text);
    opacity: 0.82;
    font-size: 12.5px;
    cursor: pointer;
    padding: 4px 6px;
    margin: -4px -6px -4px 0;
  }

  .eq-pop .route svg {
    width: 10px;
    height: 14px;
    color: var(--text-dim);
  }

  .eq-pop .route:hover {
    background: var(--hover);
    opacity: 1;
  }

  .eq-pop .route:active {
    background: var(--active);
  }

  .eq-pop .route:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
    opacity: 1;
  }

  /* Asleep, not disabled: the curve stays editable while the equalizer is off
     (shape it first, then switch it on), so only the readouts and the tracks
     dim — the band labels and the ownership row keep their contrast. */
  .bands.asleep .db,
  .bands.asleep .slot {
    opacity: 0.45;
  }

  .bands .db,
  .bands .slot {
    transition: opacity 200ms ease-out;
  }

  /* Ownership, at the base: what is on, and which curve you are editing.
     Inert by design — the route lives in the header now, so this row must not
     look clickable. */
  .eq-pop .own {
    display: flex;
    align-items: center;
    gap: 12px;
    margin: -2px -2px -4px;
    padding-top: 10px;
    border-top: 1px solid var(--border);
    cursor: default;
  }

  /* Read-only, Micro tier: it names what you are editing (and says "Custom"
     when the curve diverged) without pretending to be a picker. */
  .eq-pop .preset {
    margin-left: auto;
    font-size: 11px;
    color: var(--text-dim);
  }

  .bands {
    display: flex;
    align-items: stretch;
    gap: 10px;
  }

  .band {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 5px;
  }

  .band.preamp {
    padding-right: 10px;
    border-right: 1px solid var(--border);
  }

  /* Vertical sliders: a normal horizontal range rotated -90° inside a fixed
   * slot (WebKitGTK ignores writing-mode on range inputs — thumbs never
   * moved). rotate(-90°) puts the min end at the BOTTOM, so up = louder. */
  .slot {
    position: relative;
    width: 26px;
    height: 120px;
  }

  .vert {
    position: absolute;
    top: 50%;
    left: 50%;
    width: 120px;
    height: 26px;
    margin: 0;
    transform: translate(-50%, -50%) rotate(-90deg);
    accent-color: var(--accent);
  }

  .db {
    font-size: 9.5px;
    color: var(--text-dim);
    font-variant-numeric: tabular-nums;
    line-height: 1;
    height: 10px;
  }

  .hz {
    font-size: 9.5px;
    color: var(--text-dim);
    line-height: 1;
  }

  /* --- Queue popover (Step 7a) ---------------------------------------------- */

  .q-pop {
    position: absolute;
    right: 14px;
    bottom: calc(100% + 10px);
    z-index: 30;
    width: 320px;
    max-height: 46vh;
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 12px 8px 10px 14px;
    border-radius: 12px;
    border: 1px solid var(--border);
    box-shadow: 0 8px 28px rgba(0, 0, 0, 0.35);
    user-select: none;
    overflow: hidden;
  }

  .q-pop header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding-right: 8px;
  }

  .q-title {
    font-size: 12.5px;
    font-weight: 700;
    color: var(--text);
  }

  .q-count {
    font-size: 10.5px;
    font-weight: 700;
    color: var(--accent);
    background: var(--active);
    border-radius: 8px;
    padding: 1px 7px;
  }

  .q-clear {
    /* Far edge, opposite the close dot: destructive action as far from
      * dismissal as this header gets. */
    margin-left: auto;
    border: none;
    background: transparent;
    color: var(--text-dim);
    font-size: 11px;
    font-weight: 600;
    cursor: pointer;
    padding: 2px 6px;
    border-radius: 7px;
  }

  .q-clear:hover {
    color: var(--text);
    background: var(--hover);
  }

  .q-clear:active {
    background: var(--active);
  }

    .q-empty {
    font-size: 11.5px;
    color: var(--text-dim);
    margin: 2px 0 4px;
    line-height: 1.45;
    padding-right: 8px;
  }

  .q-list {
    list-style: none;
    margin: 0;
    padding: 0;
    overflow-y: auto;
    scrollbar-width: thin;
  }

  .q-list li {
    display: flex;
    align-items: center;
    gap: 2px;
    border-radius: 7px;
  }

  .q-list li:hover {
    background: var(--hover);
  }

  .q-list li:hover .q-x {
    opacity: 1;
  }

  .q-row {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 1px;
    text-align: left;
    border: none;
    background: transparent;
    padding: 6px 4px;
    cursor: pointer;
    font: inherit;
  }

  .q-list.preview .q-row {
    cursor: default;
  }

  .q-name {
    font-size: 12px;
    font-weight: 600;
    color: var(--text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .q-sub {
    font-size: 10.5px;
    color: var(--text-dim);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .q-x {
    flex: none;
    border: none;
    background: transparent;
    color: var(--text-dim);
    cursor: pointer;
    padding: 4px 8px;
    opacity: 0;
  }

  .q-x:hover {
    color: var(--text);
  }

  .q-next-h {
    margin: 6px 0 0;
    padding-right: 8px;
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--text-dim);
  }

  .q-list.preview {
    opacity: 0.75;
  }

  .mode-btn .badge.count {
    right: 1px;
    bottom: 0;
    font-size: 8.5px;
  }
</style>
