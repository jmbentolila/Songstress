<script lang="ts">
  import TitleBar from "./components/TitleBar.svelte";
  import Sidebar from "./components/Sidebar.svelte";
  import AlbumGrid from "./components/AlbumGrid.svelte";
  import PlayBar from "./components/PlayBar.svelte";
  import TagEditor from "./components/TagEditor.svelte";
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
  // theme) in sync so the titlebar menu bar and the Global Menu re-render.
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
  <TitleBar />
  <div class="stage">
    <AlbumGrid />
    <Sidebar />
    <PlayBar />
  </div>
  <TagEditor />
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

  /* album-area backdrop at its own alpha; chrome layers sit at theirs */
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
