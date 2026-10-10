<!-- The app's one switch: track + sliding thumb (row tier 38x22 / thumb
     16px, compact tier 32x18 / thumb 12px), OFF = hover wash + Glass Line
     with a dim thumb, ON = accent track with a luminance-aware thumb. The
     native input stays in the DOM, invisible, with role="switch", so
     focus/keyboard/AT come for free; the track is decoration, driven by
     `input:checked + .track`.

     Used by the Appearance and Playback panes (row tier) and by the
     playbar's equalizer popover header (compact tier) — everywhere else a
     bare native checkbox would be the only widget in the app outside the
     design system. -->
<script lang="ts">
  let {
    checked,
    onchange,
    label,
    small = false,
  }: {
    checked: boolean;
    onchange: (on: boolean) => void;
    label: string;
    /** Popover/compact cadence: smaller track, same switch language. */
    small?: boolean;
  } = $props();
</script>

<label class="toggle" class:small>
  <input
    type="checkbox"
    role="switch"
    {checked}
    onchange={(e) => onchange(e.currentTarget.checked)}
  />
  <span class="track" aria-hidden="true">
    <span class="thumb"></span>
  </span>
  <span class="caption">{label}</span>
</label>

<style>
  .toggle {
  /* A switch IS a row: it takes the app's row height, so a toggle sits on
     the same rhythm as the menu rows, the artist list and every .mrow/.irow
     around it — and the whole row, not just the track + caption, is the
     click target. Control-left, label-right; the compact variant (equalizer
     popover header) opts out of the row height. */
    display: flex;
    align-items: center;
    gap: 8px;
    min-height: var(--sidebar-row-size);
    font-size: 15px;
    color: var(--text);
    cursor: pointer;
  }

  .toggle.small {
    min-height: 0;
    font-size: 14.5px;
    font-weight: 600;
    gap: 7px;
  }

  /* Long labels wrap under the row's min-height instead of pushing the
     track out: the track is flex-none, the caption takes the shrink. */
  .toggle .caption {
    flex: 1;
    min-width: 0;
  }

  .toggle input {
    position: absolute;
    width: 1px;
    height: 1px;
    opacity: 0;
    margin: 0;
  }

  /* Row tier: 38x22 track, 16px thumb, 3px inset — 38 − 16 − 2×3 = 16px
     of travel. Flex-none: the track never squeezes, whatever the label
     or the popover width does. */
  .toggle .track {
    flex: none;
    position: relative;
    width: 38px;
    height: 22px;
    border: 1px solid var(--border);
    border-radius: var(--radius-pill);
    background: var(--hover);
    transition:
      background-color 160ms var(--ease-out),
      border-color 160ms var(--ease-out);
  }

  .toggle.small .track {
    width: 32px;
    height: 18px;
  }

  .toggle .thumb {
    position: absolute;
    top: 3px;
    left: 3px;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: var(--text-dim);
    /* Transform-only travel: nothing reflows, nothing queues. */
    transition:
      transform 160ms var(--ease-out),
      background-color 160ms var(--ease-out);
  }

  .toggle.small .thumb {
    width: 12px;
    height: 12px;
  }

  .toggle:hover input:not(:checked) + .track .thumb {
    background: var(--text);
  }

  .toggle input:checked + .track {
    background: var(--accent);
    border-color: var(--accent);
  }

  .toggle input:checked + .track .thumb {
    background: var(--accent-text, #fff);
    transform: translateX(16px);
  }

  .toggle.small input:checked + .track .thumb {
    transform: translateX(14px);
  }

  .toggle input:focus-visible + .track {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
  }

  /* Reduced motion: the flip is instant (same contract as ROW_FADE). */
  @media (prefers-reduced-motion: reduce) {
    .toggle .track,
    .toggle .thumb {
      transition: none;
    }
  }
</style>
