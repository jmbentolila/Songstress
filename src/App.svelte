<script lang="ts">
  import ScreenPick from "./components/ScreenPick.svelte";
  import Sidebar from "./components/Sidebar.svelte";
  import AlbumGrid from "./components/AlbumGrid.svelte";
  import PlayBar from "./components/PlayBar.svelte";
  import TagEditor from "./components/TagEditor.svelte";
  import About from "./components/About.svelte";
  import MusicFolders from "./components/MusicFolders.svelte";
  import ManageImports from "./components/ManageImports.svelte";
  import ContextMenu from "./components/ContextMenu.svelte";
  import { ui, resolvedTheme, initSettings, pushSetting } from "./lib/stores/ui.svelte";
  import { accentVariants, fallbackAccentForTheme } from "./lib/accent";
  import { decoVars } from "./lib/stores/decoration.svelte";
  import { initMenu, pushMenuState } from "./lib/stores/menu.svelte";
  import { startViewportGuard } from "./lib/viewportGuard";
  import { initScanner, scanner } from "./lib/stores/scanner.svelte";
  import { importMusic } from "./lib/stores/imports.svelte";
  import { modalOpen } from "./lib/stores/surfaces.svelte";
  import { library } from "./lib/stores/library.svelte";
  import { announcer } from "./lib/stores/announcer.svelte";
  import { playback, initEq, reanchorCurrent } from "./lib/stores/playback.svelte";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { invoke } from "@tauri-apps/api/core";
  import { isTauri, appWindow, syncWindowState } from "./lib/window";

  // Screen-pick overlay window (from the tag editor's dropper): it loads the
  // same bundle at #screen-pick and renders ONLY the picker — none of the
  // library UI, which has no business in a fullscreen loupe.
  const pickOverlay =
    typeof location !== "undefined" && location.hash === "#screen-pick";

  startViewportGuard();
  void initSettings();
  void initEq();
  initScanner();
  void initMenu();

  // Desktop environment for DE-gated stock styling (GNOME Adwaita
  // fallbacks in app.css). Runs once: a chosen accent writes inline vars
  // and beats the stock values outright, and the artwork gradients never
  // read --accent, so they keep their album colors on every DE.
  if (isTauri) {
    invoke<string>("desktop_environment")
      .then((de) => {
        document.documentElement.dataset.de = de;
      })
      .catch(() => {});
  }

  // Frame v2 gate: publish maximized state for the app.css margin+shadow
  // override, then re-check on every resize (maximize/unmaximize/tile).
  if (isTauri) {
    void syncWindowState();
    void appWindow?.onResized(() => {
      void syncWindowState();
    });
  }

  // Keep the Rust-owned menu model's dynamic bits (playing/scanning/staged/
  // theme/gradient) in sync so the Global Menu re-render. The MODE (ui.theme),
  // not resolvedTheme() — the menu radios report which mode you picked,
  // including "system" (2026-09-03 Global Menu pass).
  $effect(() => {
    void playback.current;
    void playback.isPlaying;
    void playback.eq.enabled;
    void playback.eq.preset;
    void scanner.running;
    void library.albums.length;
    void ui.theme;
    void ui.playbarGradient;
    void ui.playbarWaveform;
    void ui.albumGradient;
    pushMenuState();
  });

  // Drag-and-drop = the second import path (Step 2a): dropped files/folders
  // go through the exact same staging flow as the portal pickers.
  $effect(() => {
    if (!library.live) return;
    void getCurrentWebview().onDragDropEvent((event) => {
      if (event.payload.type === "drop") {
        void importMusic(event.payload.paths);
      }
    });
  });

  // The KWin button palette on <html>: the WINDOW's dots read --tb-* from here,
  // so one source (the user's decoration) drives them. Surfaces do not: a modal
  // or popover dismisses with the boxed ✕ (SurfaceClose), because the close
  // colour claims a verb they don't have.
  $effect(() => {
    for (const [k, v] of decoVars()) document.documentElement.style.setProperty(k, v);
  });

  $effect(() => {
    document.documentElement.dataset.theme = resolvedTheme();
    // Progress style drives the playbar height (app.css html[data-playbar]).
    document.documentElement.dataset.playbar = ui.playbarWaveform ? "wave" : "line";
    document.documentElement.style.setProperty("--tile-size", `${ui.tileSize}px`);
    document.documentElement.style.setProperty(
      "--sidebar-row-size",
      `${ui.sidebarRowSize}px`,
    );
  });

  // A retag can move the PLAYING row to another album (the path and the track
  // id survive; album_id does not). Every dump is where that lands, so re-point
  // the playback context at the row's new home — otherwise the playbar reads
  // "Nothing playing" over audio that is still going, and the destination
  // album's rows show no playing state (owner report, 2026-09-26).
  $effect(() => {
    void library.dumpVersion;
    reanchorCurrent();
  });

  // White has no usable light-mode form: a stored white selection entering
  // light mode becomes black, in the store and therefore in persistence.
  $effect(() => {
    const next = fallbackAccentForTheme(ui.accentColor, resolvedTheme());
    if (next !== ui.accentColor) ui.accentColor = next;
  });

  // Accent picker (Step 4): ONE stored hex → per-theme variable overrides on
  // <html>. Unset = removeProperty → stock purple from app.css everywhere.
  $effect(() => {
    const root = document.documentElement;
    const hex = ui.accentColor;
    if (!hex) {
      root.style.removeProperty("--accent");
      root.style.removeProperty("--active");
      root.style.removeProperty("--accent-text");
      return;
    }
    const v = accentVariants(hex, resolvedTheme());
    root.style.setProperty("--accent", v.accent);
    root.style.setProperty("--active", v.active);
    root.style.setProperty("--accent-text", v.accentText);
  });

  $effect(() => {
    localStorage.setItem("songstress.theme", JSON.stringify(ui.theme));
    localStorage.setItem("songstress.tileSize", JSON.stringify(ui.tileSize));
    localStorage.setItem(
      "songstress.sidebarRowSize",
      JSON.stringify(ui.sidebarRowSize),
    );
    localStorage.setItem(
      "songstress.playbarGradient",
      JSON.stringify(ui.playbarGradient),
    );
    localStorage.setItem(
      "songstress.playbarWaveform",
      JSON.stringify(ui.playbarWaveform),
    );
    localStorage.setItem(
      "songstress.albumGradient",
      JSON.stringify(ui.albumGradient),
    );
    localStorage.setItem(
      "songstress.panelGradients",
      JSON.stringify(ui.panelGradients),
    );
    localStorage.setItem("songstress.accentColor", JSON.stringify(ui.accentColor));
    pushSetting("theme", ui.theme);
    pushSetting("tileSize", ui.tileSize);
    pushSetting("sidebarRowSize", ui.sidebarRowSize);
    pushSetting("playbarGradient", ui.playbarGradient);
    pushSetting("playbarWaveform", ui.playbarWaveform);
    pushSetting("albumGradient", ui.albumGradient);
    // No `panelGradients` push: the overrides are one settings key per album
    // (`albumGradient:<id>`), written by `set_album_gradient` together with the
    // album's `.songstress.json`. Pushing the whole map from here would race
    // the backend's sidecar adoption on the next scan.
    pushSetting("accentColor", ui.accentColor);
  });
</script>

{#if pickOverlay}
  <ScreenPick />
{:else}
<div class="app">
  <!-- A modal claims the window, so the window's other contents are inert: without
       this, `aria-modal` is a claim the code does not honour and Tab walks the grid,
       the sidebar and the playbar behind the scrim (the same failure the settings
       stack was fixed for). Popovers and the context menu are deliberately NOT
       covered — see modalOpen(). Dragging the window is already blocked by the scrim
       itself, so nothing is taken away that worked. -->
  <div class="stage" inert={modalOpen()}>
    <!-- No titlebar: the grid's top padding band is the window drag strip
         (the sidebar header is the other drag region). It overlaps the
         20px content padding, so nothing clickable hides under it at
         scroll 0; scrolled content just can't be grabbed in its top 20px. -->
    <div class="drag-strip" data-tauri-drag-region></div>
    <!-- Soft cast shadows where content meets the window top and the
         playbar shelf — rounds the hard scroll-clip (see .edge-shadows) -->
    <div class="edge-shadows"></div>
    <AlbumGrid />
    <Sidebar />
    <PlayBar />
  </div>
  <!-- The app's one status-message region (audit 0.9.0 / WCAG 4.1.3):
       visually hidden, OUTSIDE the inert stage so a modal cannot mute it,
       fed by announcer.say() from the stores and editors — the same
       sentences the footers and notes show. -->
  <div class="sr-only" role="status" aria-live="polite">{announcer.message}</div>
  <TagEditor />
  <About />
  <MusicFolders />
  <ManageImports />
  <ContextMenu />
</div>
{/if}

<style>
  .app {
    position: fixed;
    /* --frame-margin (frame v2): the window seat's shadow paints into this
       transparent margin, outside the border box. Maximized drops it (see
       the html[data-window] gate in app.css). */
    inset: var(--frame-margin, 20px);
    display: flex;
    flex-direction: column;
    border-radius: var(--radius-window);
    overflow: hidden;
    /* Structural guarantee for every overlay: paint containment makes .app the
       containing block for position:fixed descendants, so they are clipped to
       the window's rounded shape instead of escaping it (a bare overflow:hidden
       does NOT clip fixed descendants — their containing block is the viewport).
       Without this, any future full-window overlay can paint square corners over
       the transparent ones again. */
    contain: paint;
  }

  .stage {
    position: relative;
    flex: 1;
    min-height: 0;
  }

  .drag-strip {
    position: absolute;
    top: 0;
    left: var(--sidebar-width);
    right: 0;
    height: var(--gap);
    z-index: 5;
  }

  /* The grid scroller clips at the playbar's top line and at the window
     top (Step 8): rows slide out under those edges. backdrop-filter can't
     frost in-window content on this WebKitGTK (probe-verified), and a
     per-row `filter: blur()` smears the whole row — so the cut is softened
     with static cast shadows: chrome casts a soft shadow onto the grid.
     pointer-events: none, so it never blocks tiles or the drag strip. */
  .edge-shadows {
    position: absolute;
    top: 0;
    left: var(--sidebar-width);
    right: 0;
    bottom: var(--playbar-h);
    transition: bottom var(--playbar-dur, 200ms) var(--ease-out);
    z-index: 5;
    pointer-events: none;
    background:
      linear-gradient(to bottom, rgba(0, 0, 0, 0.14), transparent 26px) top /
        100% 26px no-repeat,
      linear-gradient(to top, rgba(0, 0, 0, 0.2), transparent 34px) bottom /
        100% 34px no-repeat;
  }

  /* Light mode casts softer shadows: the dark-tuned alphas sit too heavy
     on the porcelain field. */
  :global(html[data-theme="light"]) .edge-shadows {
    background:
      linear-gradient(to bottom, rgba(0, 0, 0, 0.09), transparent 26px) top /
        100% 26px no-repeat,
      linear-gradient(to top, rgba(0, 0, 0, 0.12), transparent 34px) bottom /
        100% 34px no-repeat;
  }

  /* Dated 2026-10-10 round-2 (Aina spec, owner strip ruling: 1 playbar flat
     bg stays, 2 playbar hairline stays, 3 shelf shadow GOES, 4 grid stays).
     The shelf read as a band because it sat beside the bright 2px hairline
     (Ciel: 1 level of shelf vs 31-39 hairline); bottom is now a 12px
     clip-ease at 0.05 — half the alpha, one-third the length, sub-band-
     threshold, reading as anti-alias of the hairline rather than a shelf.
     The NAMED edge under sliding rows is the playbar's own border-top
     hairline (var(--border), PlayBar.svelte, strip 2, untouched); the 12px
     fade only rounds the scroll cut so the hard clip line does not come
     back (standing constraint). Top layer kept: the window-top clip has no
     hairline beside it, so removing the top fade would expose a hard cut
     at the drag strip. Lengths otherwise unchanged; light/KDE untouched. */
  :global(html[data-de="gnome"][data-theme="dark"]) .edge-shadows {
    background:
      linear-gradient(to bottom, rgba(0, 0, 0, 0.07), transparent 26px) top /
        100% 26px no-repeat,
      linear-gradient(to top, rgba(0, 0, 0, 0.05), transparent 12px) bottom /
        100% 12px no-repeat;
  }

  /* album-area backdrop at its own alpha; chrome layers sit at theirs. */
  .stage::before {
    content: "";
    position: absolute;
    top: 0;
    left: var(--sidebar-width);
    right: 0;
    bottom: var(--playbar-h);
    transition: bottom var(--playbar-dur, 200ms) var(--ease-out);
    background: var(--bg-grid);
  }
</style>
