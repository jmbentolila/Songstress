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
