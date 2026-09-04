<!-- Dismissal for a floating SURFACE — modal or popover — the boxed ✕, top-right.

     THE TEMPLATE, in the order it matters:
       • TITLE LEFT, DISMISSAL RIGHT, one row, one seam under it. The head is the
         surface's identity and its state — nothing else. A route OUT to another
         surface is not identity: the equalizer popover used to open with
         "Playback settings ›" on its left, which made it the only head in the app
         that started with navigation instead of naming what you were looking at.
       • 28×28 box, radius 7, glyph 16px — this app's icon-button rung, and the
         ~28px minimum hit target met by construction (a floating surface has no
         drag region forgiving a smaller one).
       • Dim at rest → hover wash + full-strength glyph → press = `--active` wash.
         No transition: the wash answers on pointer-down, and a dismissal that
         fades in reads as a delay.
       • Inset accent ring on `:focus-visible` — the app's only focus language,
         inset because the box sits inside the surface's padding.
       • Surfaces place focus inside themselves on open — but ON THE DIALOG
         (a `tabindex="-1"` container, see About), never on this button: an
         autofocused ✕ paints on window activation and reads as "selected"
         to an owner who never touched it (2026-09-03). First Tab arrives
         here anyway.
       • Escape and the scrim/backdrop do the same verb. This is the visible one,
         never the only one.
       • Closing returns focus to whatever opened it — the caller's job, and the
         reason `onclick` is a prop instead of an `open` binding.

     WHY A BOX AND NOT THE RED DOT: the dot is the WINDOW's close — the user's own
     KWin close colour, the window's own circle — and a surface wearing it claims a
     verb it cannot honour (dismissing the queue does not quit the app). So the dot
     lives only where the window's controls live (the sidebar header), and every
     surface dismisses with this box, which is already the app's "put this away"
     glyph: the sidebar gear morphs into the same ✕, same stroke, same geometry.

     WHY TOP-RIGHT: dismissal joins the far edge, so a head reads as
     "what this is" → "what to do about it" left to right, and a destructive action
     (Clear queue) keeps the whole width between itself and dismissal. The old
     top-left position existed to mirror the window's dot; mirroring its colour and
     shape without its verb was the mistake, not the position.

     Position comes from the parent: a header row's flex puts it right for free
     (modals, popovers); a head-less surface (About) places it absolutely and needs
     `:global()` — a class forwarded into a child component is unscoped here. -->
<script lang="ts">
  let {
    onclick,
    label = "Close",
    class: klass = "",
  }: {
    onclick: () => void;
    /** Also the accessible name and the tooltip — one string, so they cannot
     *  drift apart. "Close" is enough inside a dialog that says what it is;
     *  "Close equalizer" is right when several surfaces look alike. */
    label?: string;
    /** Positioning for a surface with no header row (About). Forwarded onto the
     *  button, so the rule that uses it must live in `:global()`. */
    class?: string;
  } = $props();
</script>

<button
  type="button"
  class={`sc ${klass}`}
  aria-label={label}
  title={label}
  {onclick}
>
  <!-- The gear's ✕, verbatim: 16-unit box, 8-unit cross, 1.4 stroke, round caps.
       Drawn, not a font glyph — a text ✕ is a third typographic family in a glass
       surface (Music folders learned this the hard way). A row's own ✕ (`.q-x`,
       "remove this row") stays BARE, inside the row, revealed on row hover: no
       box, no head, so the two ✕ never read as the same verb. -->
  <svg
    viewBox="0 0 16 16"
    fill="none"
    stroke="currentColor"
    stroke-width="1.4"
    stroke-linecap="round"
    aria-hidden="true"
  >
    <path d="M4 4 L12 12 M12 4 L4 12" />
  </svg>
</button>

<style>
  .sc {
    position: relative;
    flex: none;
    width: 28px;
    height: 28px;
    border: none;
    border-radius: 7px;
    background: transparent;
    color: var(--text-dim);
    padding: 0;
    cursor: pointer;
  }

  /* Absolutely centered, not place-items on a native <button> — that drifts ~1px
     down on this WebKitGTK (the traffic lights learned it first). In
     `transform`: the standalone `translate` property mis-resolves percentages
     on svg in this engine (measured 2026-09-05). */
  .sc svg {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: 16px;
    height: 16px;
  }

  .sc:hover {
    background: var(--hover);
    color: var(--text);
  }

  .sc:active {
    background: var(--active);
  }

  .sc:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
  }
</style>
