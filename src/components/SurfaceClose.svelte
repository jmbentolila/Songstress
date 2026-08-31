<!-- Close affordance for floating surfaces (popovers, modals) — the member of
     the traffic-light family that dismisses a surface instead of the window.

     Same device as the titlebar dots on purpose: 13px circle, the user's own
     KWin close palette (`--tb-x`, published on <html> from the decoration
     store), glyph revealed on hover only. Red ONLY: a lone dot must never show
     a yellow/green it cannot honour — a popover has no minimize and no zoom.

     The button is 26px around the 13px dot: a 13px hit area is under this app's
     own 28px minimum, and in a popover there is no drag region forgiving it.
     `.q-x` (remove this row) deliberately stays an ✕ — it is not dismissal, and
     putting the close colour on it would teach the wrong verb. -->
<script lang="ts">
  let {
    onclick,
    label = "Close",
    autofocus = false,
    class: klass = "",
  }: {
    onclick: () => void;
    label?: string;
    /** Modals must place focus inside the dialog on open. */
    autofocus?: boolean;
    /** Positioning comes from the parent (absolute in About, static in a header). */
    class?: string;
  } = $props();

  let btn = $state<HTMLButtonElement | null>(null);
  let focusedOnce = false;
  $effect(() => {
    if (autofocus && btn && !focusedOnce) {
      focusedOnce = true;
      btn.focus();
    }
  });
</script>

<button
  type="button"
  class={`sc ${klass}`}
  aria-label={label}
  title={label}
  {onclick}
  bind:this={btn}
>
  <span class="dot" aria-hidden="true">
    <svg viewBox="0 0 10 10">
      <path d="M2.2 2.2 L7.8 7.8 M7.8 2.2 L2.2 7.8" />
    </svg>
  </span>
</button>

<style>
  .sc {
    position: relative;
    flex: none;
    width: 26px;
    height: 26px;
    border: none;
    background: transparent;
    padding: 0;
    cursor: pointer;
  }

  /* Absolutely centered, not place-items: center on the button — that drifts
     ~1px down on a native <button> in WebKitGTK (the traffic lights learned
     this the hard way). */
  .dot {
    position: absolute;
    top: 50%;
    left: 50%;
    translate: -50% -50%;
    width: 13px;
    height: 13px;
    border-radius: 50%;
    border: 1px solid rgba(0, 0, 0, 0.18);
    background: var(--tb-x);
  }

  .dot svg {
    position: absolute;
    top: 50%;
    left: 50%;
    translate: -50% -50%;
    width: 8px;
    height: 8px;
    fill: none;
    stroke: rgba(255, 255, 255, 0.9);
    stroke-width: 1.4;
    stroke-linecap: round;
    opacity: 0;
  }

  /* No transition: the titlebar dots change instantly (Klassy behaviour), and
     the family is one behaviour, not two. */
  .sc:hover .dot {
    background: var(--tb-x-hover);
  }

  /* Glyph on hover AND on keyboard focus — a focused dot that shows nothing is
     a dot whose verb only mouse users get. */
  .sc:hover .dot svg,
  .sc:focus-visible .dot svg {
    opacity: 1;
  }

  .sc:active .dot {
    filter: brightness(0.82);
  }

  .sc:focus-visible {
    outline: none;
  }

  .sc:focus-visible .dot {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
</style>
