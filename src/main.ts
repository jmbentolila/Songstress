import "@fontsource-variable/inter";
import { mount } from "svelte";
import "./app.css";
import { initDevtools } from "./lib/devtools";
import { library } from "./lib/stores/library.svelte";
import { rescan } from "./lib/stores/scanner.svelte";
import App from "./App.svelte";

// DEV-only devtools bridge (no-op in prod): console/IPC/error sink +
// command channel. Must run before App mounts so the invoke spy is in
// place for the first Tauri call. See src/lib/devtools.ts + tools/devctl.mjs.
initDevtools();

// DEV-only: hold the loading state from the console / the devctl bridge
// (`node tools/devctl.mjs eval "__skel(true)"`). The skeleton is otherwise only
// visible while a scan is genuinely in flight, and a PARKED scan cannot be
// re-entered after a page reload — the backend still holds SCAN_RUNNING, so the
// retry dies with "scan already running". Without this hook, reviewing the
// placeholder costs a Rust rebuild (and a window drag) per look.
if (import.meta.env.DEV) {
  // Dev instance marker: `tauri dev` runs the .dev identifier (mono icon,
  // isolated library) — suffix the title so tiled windows are
  // distinguishable without checking the dock.
  document.title = "Songstress — dev";
  (window as unknown as { __skel?: (on: boolean) => boolean }).__skel = (on: boolean) => {
    library.devLoading = on !== false;
    return library.devLoading;
  };
  // Fire a real incremental scan from the bridge — the devctl console has no
  // __TAURI__ (the API is bundle-imported, not exposed), and "click the menu"
  // is not a scriptable door. Used by the thumb-prune verification; kept as
  // the general scan door for future loops.
  (window as unknown as { __scan?: () => Promise<void> }).__scan = () => rescan();
}

const app = mount(App, {
  target: document.getElementById("app")!,
});

export default app;
