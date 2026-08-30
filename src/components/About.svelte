<script lang="ts">
  import { getVersion } from "@tauri-apps/api/app";
  import { ui } from "../lib/stores/ui.svelte";

  // The version comes from Tauri itself (tauri.conf.json) — the RPM and
  // the About card can never disagree.
  let version = $state("…");
  let closeBtn = $state<HTMLButtonElement | null>(null);

  $effect(() => {
    void getVersion()
      .then((v) => (version = v))
      .catch(() => (version = "dev"));
  });

  // Focus must land inside the dialog on open.
  $effect(() => {
    if (!ui.aboutOpen) return;
    closeBtn?.focus();
  });

  function close() {
    ui.aboutOpen = false;
  }
  // Escape is handled by the sidebar's global key router (it owns all
  // Escape/`s`/`/` shortcuts) — a second window listener here raced it.
</script>

{#if ui.aboutOpen}
  <div
    class="ab-backdrop"
    role="presentation"
    onclick={(e) => e.target === e.currentTarget && close()}
  >
    <!-- svelte-ignore a11y_no_noninteractive_element_to_interactive_role -->
    <section class="ab glass" role="dialog" aria-modal="true" aria-label="About Songstress">
      <button class="ab-close" bind:this={closeBtn} aria-label="Close" onclick={close}>
        <svg viewBox="0 0 10 10" aria-hidden="true">
          <path d="M2.2 2.2 L7.8 7.8 M7.8 2.2 L2.2 7.8" />
        </svg>
      </button>
      <h2>Songstress</h2>
      <p class="ver">Version {version}</p>
      <p class="tag">Album-grid music player for KDE</p>
    </section>
  </div>
{/if}

<style>
  .ab-backdrop {
    position: fixed;
    inset: 0;
    z-index: 150;
    display: grid;
    place-items: center;
    background: rgba(0, 0, 0, 0.35);
  }

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

  .ab-close {
    position: absolute;
    top: 10px;
    right: 10px;
    width: 28px;
    height: 28px;
    display: grid;
    place-items: center;
    border: none;
    border-radius: 7px;
    background: transparent;
    color: var(--text-dim);
    cursor: pointer;
    padding: 0;
  }

  .ab-close:hover {
    background: var(--hover);
    color: var(--text);
  }

  .ab-close:active {
    background: var(--active);
  }

  .ab-close:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
  }

  .ab-close svg {
    /* Absolute centering — place-items:center on a native <button>
       drifts ~1px down in WebKitGTK (same fix as the traffic lights). */
    position: absolute;
    top: 50%;
    left: 50%;
    translate: -50% -50%;
    width: 8px;
    height: 8px;
    stroke: currentColor;
    stroke-width: 1.4;
    stroke-linecap: round;
    fill: none;
  }
</style>
