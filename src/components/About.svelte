<script lang="ts">
  import { getVersion } from "@tauri-apps/api/app";
  import { ui } from "../lib/stores/ui.svelte";
  import SurfaceClose from "./SurfaceClose.svelte";
  import { trapTab } from "../lib/focusTrap";

  let panel = $state<HTMLElement | null>(null);

  // The version comes from Tauri itself (tauri.conf.json) — the RPM and
  // the About card can never disagree.
  let version = $state("…");
  $effect(() => {
    void getVersion()
      .then((v) => (version = v))
      .catch(() => (version = "dev"));
  });

  // Focus returns to the opener (recorded by openAbout, which knows the trigger
  // — by the time this component unmounts, the dialog's own autofocus has
  // already moved activeElement inside). Runs on either dismissal path: the ✕,
  // Escape, or the scrim.
  $effect(() => {
    const opener = ui.aboutOpener;
    if (!ui.aboutOpen) return;
    return () => {
      if (opener && document.contains(opener)) opener.focus({ preventScroll: true });
      ui.aboutOpener = null;
    };
  });

  function close() {
    ui.aboutOpen = false;
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
    role="presentation"
    onclick={(e) => e.target === e.currentTarget && close()}
  >
    <!-- svelte-ignore a11y_no_noninteractive_element_to_interactive_role -->
    <section class="ab glass" bind:this={panel} role="dialog" aria-modal="true" aria-label="About Songstress">
      <SurfaceClose autofocus class="ab-close" label="Close" onclick={close} />
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
    font-size: 20px;
    font-weight: 700;
    letter-spacing: -0.01em;
    color: var(--text);
  }

  .ver {
    margin: 0 0 12px;
    font-size: 11px;
    font-variant-numeric: tabular-nums;
    color: var(--text-dim);
  }

  .tag {
    margin: 0;
    font-size: 13px;
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
