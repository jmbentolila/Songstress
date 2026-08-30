<script lang="ts">
  import Sidebar from "./components/Sidebar.svelte";
  import AlbumGrid from "./components/AlbumGrid.svelte";
  import PlayBar from "./components/PlayBar.svelte";
  import TagEditor from "./components/TagEditor.svelte";
  import About from "./components/About.svelte";
  import MusicFolders from "./components/MusicFolders.svelte";
  import ContextMenu from "./components/ContextMenu.svelte";
  import { ui, resolvedTheme, initSettings, pushSetting } from "./lib/stores/ui.svelte";
  import { accentVariants } from "./lib/accent";
  import { initMenu, pushMenuState } from "./lib/stores/menu.svelte";
  import { startViewportGuard } from "./lib/viewportGuard";
  import { initScanner, importMusic, scanner } from "./lib/stores/scanner.svelte";
  import { library } from "./lib/stores/library.svelte";
  import { playback, initEq } from "./lib/stores/playback.svelte";
  import { getCurrentWebview } from "@tauri-apps/api/webview";

  startViewportGuard();
  void initSettings();
  void initEq();
  initScanner();
  void initMenu();

  // Keep the Rust-owned menu model's dynamic bits (playing/scanning/staged/
  // theme) in sync so the Global Menu re-render.
  $effect(() => {
    void playback.current;
    void playback.isPlaying;
    void playback.eq.enabled;
    void playback.eq.preset;
    void scanner.running;
    void library.albums.length;
    void resolvedTheme();
    pushMenuState();
  });

  // Drag-and-drop = the second import path (Step 2a): dropped files/folders
  // go through the exact same staging flow as the kdialog pickers.
  $effect(() => {
    if (!library.live) return;
    void getCurrentWebview().onDragDropEvent((event) => {
      if (event.payload.type === "drop") {
        void importMusic(event.payload.paths);
      }
    });
  });

  $effect(() => {
    document.documentElement.dataset.theme = resolvedTheme();
    document.documentElement.style.setProperty("--tile-size", `${ui.tileSize}px`);
    document.documentElement.style.setProperty(
      "--sidebar-row-size",
      `${ui.sidebarRowSize}px`,
    );
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
    localStorage.setItem("songstress.accentColor", JSON.stringify(ui.accentColor));
    pushSetting("theme", ui.theme);
    pushSetting("tileSize", ui.tileSize);
    pushSetting("sidebarRowSize", ui.sidebarRowSize);
    pushSetting("playbarGradient", ui.playbarGradient);
    pushSetting("accentColor", ui.accentColor);
  });
</script>

<div class="app">
  <div class="stage">
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
  <TagEditor />
  <About />
  <MusicFolders />
  <ContextMenu />
</div>

<style>
  .app {
    position: fixed;
    inset: 0;
    display: flex;
    flex-direction: column;
    border-radius: 14px;
    overflow: hidden;
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
    z-index: 5;
    pointer-events: none;
    background:
      linear-gradient(to bottom, rgba(0, 0, 0, 0.22), transparent 26px) top /
        100% 26px no-repeat,
      linear-gradient(to top, rgba(0, 0, 0, 0.3), transparent 34px) bottom /
        100% 34px no-repeat;
  }

  /* album-area backdrop at its own alpha; chrome layers sit at theirs. */
  .stage::before {
    content: "";
    position: absolute;
    top: 0;
    left: var(--sidebar-width);
    right: 0;
    bottom: var(--playbar-h);
    background: var(--bg-grid);
  }
</style>
