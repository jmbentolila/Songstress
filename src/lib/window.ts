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
