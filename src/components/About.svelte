<script lang="ts">
  import { getVersion } from "@tauri-apps/api/app";
  import { ui } from "../lib/stores/ui.svelte";
  import SurfaceClose from "./SurfaceClose.svelte";
  import { trapTab } from "../lib/focusTrap";

  let panel = $state<HTMLElement | null>(null);
  let out = $state(false);

  // The version comes from Tauri itself (tauri.conf.json) — the RPM and
  // the About card can never disagree.
  let version = $state("…");
  $effect(() => {
    void getVersion()
      .then((v) => (version = v))
      .catch(() => (version = "dev"));
  });

  // Focus returns to the opener (recorded by openAbout, which knows the trigger
  // — by the time this component unmounts, the dialog's own focus has already
  // moved activeElement inside). Runs on either dismissal path: the ✕,
  // Escape, or the scrim.
  $effect(() => {
    const opener = ui.aboutOpener;
    if (!ui.aboutOpen) return;
    return () => {
      if (opener && document.contains(opener)) opener.focus({ preventScroll: true });
      ui.aboutOpener = null;
    };
  });

  // Focus the DIALOG on open, not its close button. About used to autofocus
  // the ✕ (SurfaceClose's `autofocus` prop) — defensible in-window, but from
  // the Global Menu the user arrives with no pointer history in the webview,
  // window activation paints the focused control, and the ✕ looked "selected"
  // (owner report, 2026-09-03). A `tabindex="-1"` dialog holding focus is the
  // standard practice: Escape and the Tab trap own the surface instantly, the
  // first Tab lands on the ✕ anyway, and nothing looks pressed.
  $effect(() => {
    if (ui.aboutOpen && panel) panel.focus({ preventScroll: true });
  });

  function close() {
    // The outro. `ui.aboutOpen` stays TRUE until the animation has finished, which is
    // what keeps the rest of the window honest: `inert={modalOpen()}` and the Escape
    // ownership in surfaces.svelte.ts both read that flag, and while a dialog is still
    // on screen both of those claims are still true. So nothing needed to learn about
    // the exit — the flag simply stopped meaning "requested" and kept meaning
    // "visible". (`out` is deliberately local for the same reason: four modals, eight
    // lines each, no shared state to get out of sync.)
    if (out) {
      // Second press — Escape, the ✕ or a click on the fading scrim — stops waiting.
      // Without this, a `animationend` that never arrives (a throttled hidden webview
      // is not hypothetical here) would leave a dialog on screen that refused to close.
      ui.aboutOpen = false;
      out = false;
      return;
    }
    out = true;
  }

  // The scrim's own animation is the longer of the pair (190ms against the panel's
  // 150ms), so ending on it means the dim has finished dissolving before the surface
  // is removed. The panel holds its end state with `forwards` meanwhile. Under
  // prefers-reduced-motion this fires in ~0.01ms, which is exactly why the exit is an
  // animation and not a setTimeout: a fixed delay would be a dead wait for the users
  // who asked for no motion.
  function onOutroEnd(e: AnimationEvent) {
    if (e.animationName !== "scrim-out") return;
    ui.aboutOpen = false;
    out = false;
  }

  // About owns its own Escape, like every other surface: while a surface is up the
  // sidebar's router stands down (surfaces.svelte.ts). It used to be the router's
  // job, which is why a modal opened from a pane closed BOTH on one keypress —
  // same-node `window` listeners cannot be ordered against each other.
  function onKeydown(e: KeyboardEvent) {
    if (!ui.aboutOpen) return;
    if (e.key === "Tab") {
      trapTab(e, panel);
      return;
    }
    if (e.key === "Escape") close();
  }
</script>

<svelte:window onkeydown={onKeydown} />

{#if ui.aboutOpen}
  <div
    class="ab-backdrop scrim"
    class:out
    role="presentation"
    onanimationend={onOutroEnd}
    onclick={(e) => e.target === e.currentTarget && close()}
  >
    <!-- svelte-ignore a11y_no_noninteractive_element_to_interactive_role -->
    <section
      class="ab glass"
      bind:this={panel}
      role="dialog"
      aria-modal="true"
      aria-label="About Songstress"
      tabindex="-1"
    >
      <SurfaceClose class="ab-close" label="Close" onclick={close} />
      <h2>Songstress</h2>
      <p class="ver">Version {version}</p>
      <p class="tag">Album-grid music player for KDE</p>
    </section>
  </div>
{/if}

<style>
  .ab {
    position: relative;
    width: min(340px, calc(100vw - 80px));
    padding: 30px 24px 26px;
    border: 1px solid var(--border);
    border-radius: 12px;
    background: var(--panel-bg-strong);
    box-shadow: 0 8px 28px rgba(0, 0, 0, 0.35);
    text-align: center;
  }

  .ab h2 {
    margin: 0 0 4px;
    font-size: 22px;
    font-weight: 700;
    letter-spacing: -0.01em;
    color: var(--text);
  }

  .ver {
    margin: 0 0 12px;
    font-size: 13px;
    font-variant-numeric: tabular-nums;
    color: var(--text-dim);
  }

  .tag {
    margin: 0;
    font-size: 15px;
    color: var(--text-dim);
  }

  /* Position only — size, palette, glyph and focus ring are the component's.
     Top-RIGHT, 8px from the edges: About is a card with no header row, so the
     box is a corner object here rather than the end of a title line. It stays
     clear of the centred title because the card's own padding (24px) is wider
     than the box. */
  /* :global: the class is forwarded into the child component, so it lands on an
     element compiled in another file — About's own scope hash is not there.
     Descendant selector (two classes) because SurfaceClose's own
     `.sc { position: relative }` is an equal-specificity rule in a later
     stylesheet: one class would lose the tie and leave the ✕ in flow. */
  :global(.ab .ab-close) {
    position: absolute;
    top: 8px;
    right: 8px;
  }
</style>
