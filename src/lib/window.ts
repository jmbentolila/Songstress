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

// Corner resize (2026-10-05): the frameless window's corners moved instead
// of resizing because NOTHING ever requests a resize there — drag.js only
// knows `start_dragging` (move), Mutter draws no grabs on decorations(false)
// windows, and Tauri wires no resize-region gesture. The fix is four corner
// handles (App.svelte) calling this: Window -> tao DragResizeWindow
// -> gtk begin_resize_drag. No-op outside Tauri; maximized hides the handles
// via the same html[data-window] gate (a resize request on a maximized
// window would unmaximize into a jump). Edges keep whatever behavior they
// have today — untouched.
export type ResizeCorner = "NorthWest" | "NorthEast" | "SouthWest" | "SouthEast";

export async function startResizeDrag(direction: ResizeCorner) {
  // TEMPORARY diagnostic (2026-10-05 corner-resize investigation): record how
  // far each corner press gets, queryable via devctl AFTER a real press:
  //   absent            press never reached the handler (compositor delivery /
  //                     input-region territory — no repo fix)
  //   <dir>:mousedown   handler ran, invoke did not resolve (request lost)
  //   <dir>:resolved    full chain OK — if the owner still sees no resize,
  //                     Mutter refused the client request (compositor-side)
  //   <dir>:rejected    invoke error — fix the call
  // Read with: node tools/devctl.mjs eval
  //   "document.documentElement.dataset.resizeProbe ?? 'absent'"
  // Remove once diagnosed.
  const stamp = (s: string) => {
    document.documentElement.dataset.resizeProbe = `${direction}:${s}`;
  };
  try {
    stamp("mousedown");
    await appWindow?.startResizeDragging(direction);
    stamp("resolved");
  } catch {
    stamp("rejected");
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
