<script lang="ts">
  /**
   * Per-album panel-gradient override (Step 9b) — the second resident of
   * the album modal's artwork column, under the ArtSelector. Two hex stops
   * that replace the scan's artwork colors for THIS album in the expanded
   * panel; "Use artwork colors" forgets them.
   *
   * Applies INSTANTLY (display state, not file tags — it never waits for
   * the modal's Save, same as the Appearance toggles). Your hex, your
   * problem: no contrast clamping, by owner ruling.
   */
  import { library } from "../lib/stores/library.svelte";
  import {
    ui,
    resolvedTheme,
    setPanelGradient,
    clearPanelGradient,
  } from "../lib/stores/ui.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { artGradient, normalizeHex } from "../lib/gradient";
  import { tooltip } from "../lib/tooltip";
  import { announcer } from "../lib/stores/announcer.svelte";
  import { isTauri } from "../lib/window";

  let { albumId }: { albumId: string } = $props();

  const album = $derived(library.albums.find((a) => a.id === albumId));
  const override = $derived(ui.panelGradients[albumId]);

  // Text fields seed from override → artwork colors → empty, and re-seed
  // ONLY when the modal re-points at another album (never on override
  // writes: typing a valid value applies it immediately, and re-seeding
  // from the write would normalize the text out from under the keystroke
  // — "#ABC" would snap to "aabbcc" mid-word).
  let t1 = $state("");
  let t2 = $state("");
  let seededFor = $state("");
  $effect(() => {
    const id = albumId;
    if (id === seededFor) return;
    seededFor = id;
    const o = ui.panelGradients[id];
    const a = library.albums.find((x) => x.id === id);
    t1 = o?.c1 ?? a?.colorC1 ?? "";
    t2 = o?.c2 ?? a?.colorC2 ?? "";
  });

  const n1 = $derived(normalizeHex(t1));
  const n2 = $derived(normalizeHex(t2));

  // Valid pair → live override. Invalid (half-typed) text writes nothing:
  // the panel keeps the last good pair until the field parses again.
  $effect(() => {
    const id = albumId;
    const p1 = n1;
    const p2 = n2;
    if (!p1 || !p2) return;
    const o = ui.panelGradients[id];
    if (o?.c1 === p1 && o?.c2 === p2) return;
    setPanelGradient(id, p1, p2);
  });

  function reset() {
    clearPanelGradient(albumId);
    const a = library.albums.find((x) => x.id === albumId);
    t1 = a?.colorC1 ?? "";
    t2 = a?.colorC2 ?? "";
  }

  // Preview wears what the panel wears: the override, else the artwork
  // colors — same builder, same theme. Nothing valid at all → the base.
  const preview = $derived.by(() => {
    const theme = resolvedTheme();
    const o = override;
    if (o) return artGradient(o.c1, o.c2, theme, 0.5);
    if (album?.colorC1 && album.colorC2)
      return artGradient(album.colorC1, album.colorC2, theme, 0.5);
    return null;
  });

  // The native color input needs a concrete #rrggbb; the text is the truth.
  const pick1 = $derived(n1 ? `#${n1}` : "#000000");
  const pick2 = $derived(n2 ? `#${n2}` : "#000000");

  // Screen dropper (portal crosshair, Rust side): fills the row it was
  // pressed from. Cancel is silence (null); only a real failure speaks.
  // Hidden where there is no portal (browser dev) — the hex fields stay.
  let picking = $state<1 | 2 | null>(null);
  // A click owes an answer: failures print HERE, not just to the console
  // (a dead button with no word was the whole bug report, 2026-09-15).
  let pickerError = $state("");
  async function dropper(which: 1 | 2) {
    if (!isTauri || picking !== null) return;
    picking = which;
    pickerError = "";
    try {
      const hex = await invoke<string | null>("pick_screen_color");
      if (hex) {
        if (which === 1) t1 = hex;
        else t2 = hex;
      }
    } catch (e) {
      console.error(e);
      pickerError = "The screen color picker could not open.";
      announcer.say(pickerError);
    } finally {
      picking = null;
    }
  }
</script>

<section class="pg" aria-label="Panel background">
  <div class="pg-title">Panel background</div>
  <!-- background-IMAGE, not the shorthand: an inline `background:` would
       silently reset background-clip to border-box (the panel's own old
       bug) and the tinted rim would come straight back. -->
  <div
    class="pg-preview"
    style:background-image={preview ?? undefined}
    aria-hidden="true"
  ></div>
  <div class="pg-row">
    <label
      class="pg-swatch"
      use:tooltip={"Pick the first gradient color"}
    >
      <span class="pg-chip" style:background={pick1} aria-hidden="true"></span>
      <span class="sr-only">First gradient color</span>
      <input
        type="color"
        value={pick1}
        aria-label="First gradient color"
        oninput={(e) => (t1 = e.currentTarget.value)}
      />
    </label>
    <input
      class="pg-hex"
      class:bad={t1.trim() !== "" && !n1}
      value={t1}
      placeholder={album?.colorC1 ?? "rrggbb"}
      aria-label="First gradient color hex"
      spellcheck="false"
      oninput={(e) => (t1 = e.currentTarget.value)}
    />
    {#if isTauri}
      <button
        class="pg-tool"
        aria-label="Pick the first gradient color from the screen"
        use:tooltip={"Pick from the screen"}
        disabled={picking !== null}
        onclick={() => void dropper(1)}
      >
        <svg viewBox="0 0 16 16" aria-hidden="true"><circle cx="11" cy="5" r="2.5" fill="none" stroke="currentColor" stroke-width="1.5" /><path d="M9.2 6.8 5 11" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" /><path d="M5.2 10.8 4 12" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" /><path d="M3.6 12.2c.8.8.8 2.1 0 2.9-.8-.8-.8-2.1 0-2.9z" fill="currentColor" /></svg>
      </button>
    {/if}
  </div>
  <div class="pg-row">
    <label
      class="pg-swatch"
      use:tooltip={"Pick the second gradient color"}
    >
      <span class="pg-chip" style:background={pick2} aria-hidden="true"></span>
      <span class="sr-only">Second gradient color</span>
      <input
        type="color"
        value={pick2}
        aria-label="Second gradient color"
        oninput={(e) => (t2 = e.currentTarget.value)}
      />
    </label>
    <input
      class="pg-hex"
      class:bad={t2.trim() !== "" && !n2}
      value={t2}
      placeholder={album?.colorC2 ?? "rrggbb"}
      aria-label="Second gradient color hex"
      spellcheck="false"
      oninput={(e) => (t2 = e.currentTarget.value)}
    />
    {#if isTauri}
      <button
        class="pg-tool"
        aria-label="Pick the second gradient color from the screen"
        use:tooltip={"Pick from the screen"}
        disabled={picking !== null}
        onclick={() => void dropper(2)}
      >
        <svg viewBox="0 0 16 16" aria-hidden="true"><circle cx="11" cy="5" r="2.5" fill="none" stroke="currentColor" stroke-width="1.5" /><path d="M9.2 6.8 5 11" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" /><path d="M5.2 10.8 4 12" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" /><path d="M3.6 12.2c.8.8.8 2.1 0 2.9-.8-.8-.8-2.1 0-2.9z" fill="currentColor" /></svg>
      </button>
    {/if}
  </div>
  <div class="pg-foot">
    {#if override}
      <button
        class="pg-link"
        onclick={reset}
        use:tooltip={"Forget the custom colors — back to the artwork"}
      >Use artwork colors</button>
    {:else}
      <span class="pg-note">Custom colors apply instantly.</span>
    {/if}
  </div>
  {#if pickerError}
    <p class="pg-error">{pickerError}</p>
  {/if}
  {#if !ui.albumGradient}
    <p class="pg-note">Album gradients are off — turn them on in Appearance to see this.</p>
  {/if}
</section>

<style>
  .pg {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding-top: 10px;
    border-top: 1px solid var(--border);
  }

  .pg-title {
    font-size: 14px;
    color: var(--text-dim);
  }

  .pg-preview {
    height: 34px;
    border-radius: 8px;
    border: 1px solid var(--border);
    background: var(--panel-bg);
    /* The gradient must stop at the padding box: --border is translucent,
       so a border-box fill bleeds through as a tinted rim (the panel's
       red-arc lesson, smaller). */
    background-clip: padding-box;
  }

  .pg-row {
    display: flex;
    align-items: stretch;
    gap: 8px;
  }

  /* The swatch is a dark WELL holding a color chip — not a solid bright
     block. A bright fill edge-to-edge reads TALLER than its identical
     dark neighbours (irradiation), which is what the last round of
     "the swatch is bigger" was: measured 32px all around, illusion only.
     The native input rides over the well INVISIBLE (keyboard/AT/picker
     all free) — letting it paint meant its own box peeked through as a
     hole along the bottom. Height follows the row (the hex field sets
     it), never a fixed 28px. */
  .pg-swatch {
    position: relative;
    display: flex;
    flex: none;
    width: 34px;
    min-height: 28px;
    padding: 4px;
    border-radius: 7px;
    border: 1px solid var(--border);
    background: transparent;
    cursor: pointer;
  }

  .pg-chip {
    flex: 1;
    border-radius: 4px;
  }

  .pg-swatch input {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    padding: 0;
    border: none;
    opacity: 0;
    cursor: pointer;
  }

  .pg-hex {
    flex: 1;
    min-width: 0;
    padding: 6px 8px;
    font-size: 15px;
    font-family: ui-monospace, monospace;
    text-align: center;
    color: var(--text);
    border: 1px solid var(--border);
    border-radius: 7px;
    background: transparent;
  }

  .pg-hex:focus {
    outline: none;
    border-color: var(--accent);
  }

  .pg-hex.bad {
    border-color: var(--caution);
  }

  /* The dropper: the modal's quiet-tool language (see .edit-album in the
     panel header) — 28px hit target, text-dim until touched. */
  .pg-tool {
    flex: none;
    width: 34px;
    min-height: 28px;
    border: 1px solid var(--border);
    border-radius: 7px;
    background: transparent;
    color: var(--text-dim);
    display: grid;
    place-items: center;
    cursor: pointer;
    padding: 0;
  }

  .pg-tool:hover:not(:disabled) {
    color: var(--text);
    border-color: var(--accent);
  }

  .pg-tool:disabled {
    opacity: 0.45;
    cursor: default;
  }

  .pg-tool svg {
    width: 13px;
    height: 13px;
  }

  .pg-tool:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  .pg-foot {
    display: flex;
    align-items: center;
    min-height: 18px;
  }

  .pg-link {
    border: 0;
    background: none;
    padding: 0;
    font: inherit;
    font-size: 13px;
    color: var(--accent);
    cursor: pointer;
  }

  .pg-link:hover {
    text-decoration: underline;
  }

  .pg-note {
    margin: 0;
    font-size: 13px;
    color: var(--text-dim);
  }

  .pg-error {
    margin: 0;
    font-size: 13px;
    color: var(--caution);
  }

  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip: rect(0 0 0 0);
    white-space: nowrap;
  }
</style>
