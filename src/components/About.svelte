<script lang="ts">
  import { getVersion } from "@tauri-apps/api/app";
  import { ui } from "../lib/stores/ui.svelte";
  import SurfaceClose from "./SurfaceClose.svelte";

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
  // already moved activeElement inside). Runs on either dismissal path: the dot
  // or the sidebar's Escape router.
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
  // Escape is handled by the sidebar's global key router (it owns all
  // Escape/`s`/`/` shortcuts) — a second window listener here raced it.
</script>

{#if ui.aboutOpen}
  <div
    class="ab-backdrop scrim"
    role="presentation"
    onclick={(e) => e.target === e.currentTarget && close()}
  >
    <!-- svelte-ignore a11y_no_noninteractive_element_to_interactive_role -->
    <section class="ab glass" role="dialog" aria-modal="true" aria-label="About Songstress">
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

  /* Position only — size, palette, glyph reveal and focus ring are the
     component's. Top-LEFT: dismissal lives where the window's own close dot
     lives, in every floating surface. */
  /* :global: the class is forwarded into the child component, so it lands on an
     element compiled in another file — About's own scope hash is not there.
     Descendant selector (two classes) because SurfaceClose's own
     `.sc { position: relative }` is an equal-specificity rule in a later
     stylesheet: one class would lose the tie and leave the dot in flow. */
  :global(.ab .ab-close) {
    position: absolute;
    top: 8px;
    left: 8px;
  }
</style>
