<!-- The app's one checkbox: 16px rounded-square (5px — deliberately below the
     control rung for a 16px object), hover wash + Glass Line at rest, accent
     fill with a check in the luminance-aware `--accent-text`. The native input
     stays in the DOM, invisible, so focus/keyboard/AT come for free; the box
     is decoration, driven by `input:checked + .box`.

     Used by the Appearance and Playback panes and by the playbar's equalizer
     popover — everywhere else a bare native checkbox would be the only widget
     in the app outside the design system. -->
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
    /** Popover/compact cadence: 12.5px/600 instead of the row tier. */
    small?: boolean;
  } = $props();
</script>

<label class="toggle" class:small>
  <input
    type="checkbox"
    {checked}
    onchange={(e) => onchange(e.currentTarget.checked)}
  />
  <span class="box" aria-hidden="true">
    <svg viewBox="0 0 10 10"><path d="M1.5 5.5 L4 8 L8.5 2.5" /></svg>
  </span>
  <span>{label}</span>
</label>

<style>
  .toggle {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    color: var(--text);
    cursor: pointer;
  }

  .toggle.small {
    font-size: 12.5px;
    font-weight: 600;
    gap: 7px;
  }

  .toggle input {
    position: absolute;
    width: 1px;
    height: 1px;
    opacity: 0;
    margin: 0;
  }

  .toggle .box {
    width: 16px;
    height: 16px;
    flex: none;
    display: grid;
    place-items: center;
    border: 1px solid var(--border);
    border-radius: 5px;
    background: var(--hover);
    transition: background 160ms ease-out, border-color 160ms ease-out;
  }

  .toggle .box svg {
    width: 10px;
    height: 10px;
    fill: none;
    /* accent-text: the App's luminance-aware variable (white on dark
       accents, dark on light ones) — a white check on a white accent
       would vanish. */
    stroke: var(--accent-text, #fff);
    stroke-width: 1.8;
    stroke-linecap: round;
    stroke-linejoin: round;
    opacity: 0;
    transform: scale(0.7);
    transition: opacity 160ms ease-out, transform 160ms ease-out;
  }

  .toggle input:checked + .box {
    background: var(--accent);
    border-color: var(--accent);
  }

  .toggle input:checked + .box svg {
    opacity: 1;
    transform: scale(1);
  }

  .toggle input:focus-visible + .box {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
  }
</style>
