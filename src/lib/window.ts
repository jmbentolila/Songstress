import { getCurrentWindow } from "@tauri-apps/api/window";

export const isTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

export const appWindow = isTauri ? getCurrentWindow() : null;

export async function windowClose() {
  await appWindow?.close();
}
export async function windowMinimize() {
  await appWindow?.minimize();
}
export async function windowToggleMaximize() {
  await appWindow?.toggleMaximize();
}

// Bottom-corner resize zones (2026-10-05, second attempt): the margin-corner
// handles died because Mutter consumes margin presses pre-client, but presses
// on GLASS demonstrably arrive (drag regions fire there). Two overlay zones
// as PlayBar footer children call this: Window -> tao DragResizeWindow ->
// gtk begin_resize_drag. Capture-phase + stopImmediatePropagation beats the
// ancestor's bare drag-region move (document-bubble never fires). TEMPORARY
// dataset.resizeProbe stamps stay until the owner proves resize wins live.
export type ResizeCorner = "SouthWest" | "SouthEast";

export async function startResizeDrag(direction: ResizeCorner) {
  const stamp = (s: string) => {
    document.documentElement.dataset.resizeProbe = `${direction}:${s}`;
  };
  try {
    stamp("mousedown");
    await appWindow?.startResizeDragging(direction);
    stamp("resolved");
  } catch (e) {
    const name = e instanceof Error ? e.name : typeof e;
    const msg = (e instanceof Error ? e.message : String(e)).slice(0, 140);
    stamp(`rejected:${name}:${msg}`);
  }
}

// Frame v2 gate: a maximized (or tiled-filling) window has no visible edge
// to lift, so `.app` drops its margin + window-seat shadow via
// `html[data-window="maximized"]` (app.css). Called once at startup and on
// every `onResized` (wired in App.svelte): `isMaximized` is async and the
// state can change without a component re-render (titlebar/menu toggle).
// Tiled-but-not-maximized is NOT detectable through the Tauri API, so a
// Mutter-tiled window keeps its margin — recorded, not fixed.
export async function syncWindowState() {
  if (!isTauri || !appWindow) return;
  try {
    const maximized = await appWindow.isMaximized();
    document.documentElement.dataset.window = maximized ? "maximized" : "windowed";
  } catch {
    // Window gone mid-query (shutdown) — leave the last state in place.
  }
}
