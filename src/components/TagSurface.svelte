<script lang="ts">
  /**
   * The tag editor's shared surface (Phase C): backdrop, header, dismissal,
   * Escape and the focus trap — the modal CONTRACT both editors wear, in one
   * place. The body and footer are the editors' business (snippets), because
   * an album save leaves a receipt and a track save does not.
   *
   * The entrance is sized to its FINAL BOX, not to the fetch: the panel
   * appears on the click's frame at the STANDARD size (min-height below)
   * and shows "Reading file tags…" inside it; content only ever EXPANDS
   * the box (owner ruling 2026-09-05 — the old "wait invisible, reveal
   * sized to whatever loaded first" made opening a jump). The fetch
   * behind it (get_album_tags / get_track_tags) is a lofty walk over
   * files, not a cached query, which is why the loader is shown at all.
   */
  import type { Snippet } from "svelte";
  import { ui } from "../lib/stores/ui.svelte";
  import SurfaceClose from "./SurfaceClose.svelte";
  import { trapTab } from "../lib/focusTrap";
  import { scrimDismiss } from "../lib/scrimDismiss";

  let {
    label,
    loading = false,
    busy = false,
    escapeFirst,
    quickKey,
    headerExtra,
    footer,
    children,
  }: {
    label: string;
    /** Fields still loading — the panel stays invisible; the dim still comes. */
    loading?: boolean;
    /** A write is in flight: presses are ignored, not queued. */
    busy?: boolean;
    /** The innermost layer (the artwork lightbox) gets the first Escape.
     *  Return true to consume it. */
    escapeFirst?: () => boolean;
    /** The editors' accelerators (Enter = the primary action — Save when
     *  dirty, Done when clean; ←/→ = the track stepper). Called for every
     *  key except the Tab the trap eats and the Escape this shell owns.
     *  Receives the same close the footer snippet gets. */
    quickKey?: (e: KeyboardEvent, close: () => void) => void;
    /** Between the title and the ✕ — the track modal's stepper. */
    headerExtra?: Snippet;
    /** Receives `close`, the same door the ✕ and the scrim use. */
    footer: Snippet<[close: () => void]>;
    children: Snippet;
  } = $props();

  let out = $state(false);
  let panelEl = $state<HTMLElement>();
  // Sam's ledger (critique 0.9.0): whoever opened this window gets focus
  // back when it closes. Captured at instance creation — the opening click's
  // focus has landed by now, and the rAF autofocus below has not moved it.
  const opener = document.activeElement as HTMLElement | null;

  function close() {
    if (busy) return;
    // The outro — same shape as About.svelte: `out` starts the exit and the
    // OPEN flag stays true until the animation ends, so `modalOpen()` (which
    // drives `inert`) and Escape ownership keep telling the truth while the
    // dialog is still on screen. A second press force-closes: the backstop
    // for an `animationend` that never arrives.
    if (out) {
      finish();
      return;
    }
    out = true;
  }

  function finish() {
    ui.tagEditor.open = false;
    out = false;
    // Focus goes home — the keyboard user lands where they stood, not at
    // the top of the document. (A context-menu opener may already be gone;
    // detached nodes and <body> are silently skipped.)
    if (opener && opener !== document.body && opener.isConnected) {
      opener.focus({ preventScroll: true });
    }
  }

  function onOutroEnd(e: AnimationEvent) {
    // `scrim-out` (190ms) rather than the panel's 150ms: it is the longer of
    // the pair, so the surface is removed once the dim has finished
    // dissolving. The keyframes live in app.css, so this name is not
    // component-scoped and cannot drift.
    if (e.animationName !== "scrim-out") return;
    finish();
  }

  // First field takes focus once the inputs actually exist (not on mount —
  // on mount there is only the loading note).
  let focusedOnce = $state(false);
  $effect(() => {
    if (focusedOnce || !ui.tagEditor.open || loading) return;
    focusedOnce = true;
    requestAnimationFrame(() =>
      panelEl?.querySelector<HTMLElement>("input, button")?.focus(),
    );
  });

  // Height easing: the content lands in stages (tags, then the art
  // inventory, sometimes chips or the receipt), and every landing used to
  // snap the box taller — up to twice per open. A ResizeObserver eases the
  // panel between MEASURED heights instead (this WebKitGTK has no
  // interpolate-size, so height:auto cannot transition in CSS — explicit
  // px endpoints via WAAPI, the same trick as the file-list unfolds).
  // Growth and shrink alike, a full second on the shared ease-out
  // reduced motion (matchMedia, AlbumGrid's pattern — WAAPI ignores the
  // CSS kill switch) and past the max-height cap, where the body scrolls
  // internally and the panel correctly never moves.
  let growAnim: Animation | null = null;
  $effect(() => {
    const el = panelEl;
    if (!el || !ui.tagEditor.open) return;
    let armed = false;
    const reduce = matchMedia("(prefers-reduced-motion: reduce)").matches;
    const ro = new ResizeObserver(() => {
      const h = el.offsetHeight;
      // First callback is the observe baseline, not a change.
      if (!armed) {
        armed = true;
        return;
      }
      if (reduce || out) return;
      // An interrupted ease leaves fill:none, so the box is already at its
      // newest auto height — but the EYE is mid-flight. Ease from the
      // visual height, not the laid-out one, or the box visibly jumps back.
      growAnim?.cancel();
      const from = el.getBoundingClientRect().height;
      if (Math.abs(from - h) < 1) return;
      // Clip for the flight: mid-ease the box is shorter than its content
      // and the spill would paint past the rounded panel edge.
      el.style.overflow = "hidden";
      growAnim = el.animate(
        [{ height: `${from}px` }, { height: `${h}px` }],
        { duration: 1000, easing: "cubic-bezier(0.22, 1, 0.36, 1)" },
      );
      growAnim.onfinish = growAnim.oncancel = () => {
        el.style.overflow = "";
      };
    });
    ro.observe(el);
    return () => {
      ro.disconnect();
      growAnim?.cancel();
      el.style.overflow = "";
    };
  });
</script>

<svelte:window
  onkeydown={(e) => {
    if (!ui.tagEditor.open) return;
    if (e.key === "Tab") {
      trapTab(e, panelEl);
      return;
    }
    if (e.key === "Escape") {
      // The lightbox is the innermost layer: it owns the first Escape.
      if (escapeFirst?.()) return;
      close();
      return;
    }
    quickKey?.(e, close);
  }}
/>

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
<div
  class="te-backdrop scrim"
  class:out
  role="presentation"
  onanimationend={onOutroEnd}
  use:scrimDismiss={close}
>
  <!-- svelte-ignore a11y_no_noninteractive_element_to_interactive_role -->
  <section class="te glass" bind:this={panelEl} role="dialog" aria-modal="true" aria-label={label} aria-busy={loading}>
    <header class="te-head">
      <h2>{label}</h2>
      {@render headerExtra?.()}
      <SurfaceClose label="Close" onclick={close} />
    </header>

    {#if loading}
      <!-- Content-shaped wait: the same .te-cols grid the body wears, so the
           swap changes texture, not layout (DESIGN.md skeleton rules — the
           placeholder IS the status: no headline, no numbers; announced once
           via the status below, never per shape). Sized under every loaded
           state on purpose: content may only ever EXPAND this box, never
           shrink it back. -->
      <div class="te-body" aria-hidden="true">
        <div class="te-cols">
          <div class="te-skel-art">
            <span class="sk te-sk-title" style:--i={0}></span>
            <span class="sk te-sk-square" style:--i={1}></span>
            <span class="sk te-sk-bar" style:--i={2}></span>
          </div>
          <div class="te-skel-fields">
            <span class="sk te-sk-topline" style:--i={3}></span>
            <span class="sk te-sk-coltitle" style:--i={4}></span>
            {#each [5, 6, 7, 8, 9] as i (i)}
              <span class="sk te-sk-row" style:--i={i}></span>
            {/each}
            <span class="sk te-sk-more" style:--i={10}></span>
          </div>
        </div>
      </div>
      <span class="sr-only" role="status">Reading file tags…</span>
    {:else}
      <div class="te-body te-enter">{@render children()}</div>
    {/if}

    {@render footer(close)}
  </section>
</div>

<style>
  .te {
    width: min(680px, calc(100vw - 80px));
    /* The STANDARD box (owner ruling 2026-09-05): the loading state is
       shown INSIDE the modal's standard dimensions, and content may
       only ever EXPAND it — never the other way around, which read as
       a jump on open. 525px is the measured resting height of the track
       modal (680x525 sampled live); the album modal starts at this
       floor too and grows from its own content (chips, receipt). */
    min-height: 525px;
    max-height: calc(100vh - 120px);
    display: flex;
    flex-direction: column;
    padding: 16px;
    border-radius: var(--radius-panel);
    border: 1px solid var(--border);
    background: var(--panel-bg-strong);
    box-shadow: var(--shadow);
  }

  .te-head {
    display: flex;
    align-items: center;
    gap: 12px;
    padding-bottom: 10px;
    /* The modal's content corridor is 13px on BOTH sides (owner ruling:
       the scrollbar gutter made the content sit visibly left). The body
       pays it as padding-left vs its padding-right + reserved gutter;
       the head and footer, which have no gutter, pay it as an equal
       margin so every line of chrome bounds the same column. */
    margin: 0 13px;
    border-bottom: 1px solid var(--border);
  }

  .te-head h2 {
    flex: 1;
    min-width: 0;
    margin: 0;
    font-size: 17px;
    color: var(--text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .te-body {
    /* `scroll`, not `auto`: WebKitGTK's scrollbar-gutter reserves NOTHING
       until the container truly scrolls (measured: offsetWidth - clientWidth
       = 0 with `stable` while content fit), so paying the gutter on the
       left would flip the lean the moment a modal stopped scrolling.
       Always-scrolling makes the 11px permanent and honest — and with the
       track at 6% white and a transparent resting thumb, permanent here
       means invisible, not noisy. */
    overflow-y: scroll;
    scrollbar-gutter: stable;
    padding: 18px 2px 12px 13px;
  }

  /* Loading skeleton: the body's own shape. Art cell echoes the stacked
     picker (title + square ≈ the 172px tile + a short bar for the add
     tile); fields cell echoes the grid (topline + title + five input-tall
     rows + the disclosure). Deliberately SHORT of every loaded state —
     chips, tiles and the receipt only ever grow this box. Bars reuse the
     global .sk material (own delays so the light never starts in lockstep).
     The arrival below is CSS, not a svelte/transition, so the global
     reduced-motion switch reaches it. */
  .te-skel-art {
    display: flex;
    flex-direction: column;
    gap: 8px;
    min-width: 0;
  }
  .te-sk-title {
    width: 45%;
    height: 14px;
  }
  .te-sk-square {
    width: 100%;
    aspect-ratio: 1;
  }
  .te-sk-bar {
    width: 70%;
    height: 34px;
  }
  .te-skel-fields {
    display: flex;
    flex-direction: column;
    gap: 10px;
    min-width: 0;
    padding-top: 11px;
  }
  .te-sk-topline {
    height: 13px;
  }
  .te-sk-coltitle {
    width: 40%;
    height: 14px;
  }
  .te-sk-row {
    height: 34px;
  }
  .te-sk-more {
    width: 50%;
    height: 20px;
  }
  /* Arrival: the shapes fade up into position — 180ms, 8px of rise, the
     ease-out the motion tokens name. Frequency tier (opens constantly),
     so no stagger ladder: one breath for the whole surface. */
  .te-enter {
    animation: te-arrive 180ms var(--ease-out);
  }
  @keyframes te-arrive {
    from {
      opacity: 0;
      transform: translateY(8px);
    }
  }
  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    margin: -1px;
    padding: 0;
    overflow: hidden;
    clip-path: inset(50%);
    white-space: nowrap;
  }
  /* Footer loading pills (the footer family lives here — snippets cross
     file borders, so editor-scoped rules would not reach them). The album
     caption and the track file/dir slots render shimmer instead of empty
     or zero, so the buttons never slide when the words land. */
  :global(.te-load-pill) {
    width: 200px;
    height: 14px;
    margin-right: auto;
    align-self: center;
  }
  :global(.te-file-sk) {
    width: 120px;
    height: 14px;
    align-self: center;
  }
  :global(.te-dir-sk) {
    width: 100%;
    height: 14px;
    align-self: center;
  }
  /* The footer family is SHARED: an editor renders its footer content in its
     own scope, so a component-scoped rule here would not reach it (AGENTS.md:
     a class forwarded/forward-rendered across file boundaries is unscoped).
     Namespaced `.te-` so the global sheet cannot leak anywhere else. */
  :global(.te-foot) {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 8px;
    padding-top: 12px;
    margin: 0 13px; /* the corridor: see .te-head */
    border-top: 1px solid var(--border);
  }

  /* The promise line — shared by both editors, first thing under the
     header rule (owner ruling: the reassurance must never be a scroll
     away). 11px dim: present, not loud. */
  :global(.te-topline) {
    margin: 0 0 12px;
    font-size: 13px;
    color: var(--text-dim);
    letter-spacing: 0.02em;
  }

  /* The "More tag fields" disclosure — one control in both editors, so
     its sheet lives in the shared surface (snippets cross file borders;
     scoped rules would not follow the class). */
  :global(.te-more) {
    display: flex;
    align-items: center;
    gap: 6px;
    margin: 12px 0 0;
    padding: 2px 2px;
    border: 0;
    background: none;
    color: var(--text-dim);
    font-size: 13.5px;
    cursor: pointer;
  }

  /* Slide's host: the breath below the disclosure lives INSIDE the
     clipped wrapper so the outro leaves no gap behind while it shrinks. */
  :global(.te-more-wrap) {
    margin-top: 6px;
  }

  :global(.te-more:hover) {
    color: var(--text);
  }

  :global(.te-more-chev) {
    width: 10px;
    height: 10px;
    transition: transform 140ms var(--ease-out, ease-out);
  }

  :global(.te-more-chev.open) {
    transform: rotate(90deg);
  }

  :global(.te-more-tag) {
    color: var(--text-dim);
    opacity: 0.75;
  }

  :global(.te-more-edited) {
    color: var(--accent);
    opacity: 1;
  }

  /* The body shape BOTH modals wear: artwork column left, fields right.
     (Rendered inside each editor's scope, hence global — same reason as
     the footer family above.) */
  :global(.te-cols) {
    display: grid;
    grid-template-columns: 190px minmax(0, 1fr);
    gap: 18px;
    align-items: start;
  }

  :global(.te-error) {
    flex: 1;
    min-width: 0;
    font-size: 14px;
    /* The system's one caution hue — the same token a missing-file glyph and
       a discard mark wear. One hue, named. */
    color: var(--caution);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  :global(.te-ok) {
    font-size: 14.5px;
    color: var(--accent);
    font-weight: 600;
  }

  :global(.te-btn) {
    padding: 7px 14px;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: transparent;
    color: var(--text);
    font-size: 15px;
    cursor: pointer;
  }

  :global(.te-btn:hover:not(:disabled)) {
    background: var(--hover);
  }

  :global(.te-btn.primary) {
    background: var(--active);
    color: var(--accent);
    border-color: transparent;
    font-weight: 600;
    /* Save ↔ Done swaps the label by state; pin the width (fits
       "Saving…") so the footer never jitters on the first keystroke. */
    min-width: 7ch;
  }

  :global(.te-btn.primary:hover:not(:disabled)) {
    background: var(--accent);
    color: var(--accent-text, #fff);
  }

  :global(.te-btn:disabled) {
    opacity: 0.45;
    cursor: default;
  }
</style>
