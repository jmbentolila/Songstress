import { invoke } from "@tauri-apps/api/core";
import { isTauri } from "../window";

export interface DecoButton {
  normal: [number, number, number];
  hover: [number, number, number];
}

export interface WindowDeco {
  buttonsLeft: string;
  buttonsRight: string;
  close: DecoButton;
  minimize: DecoButton;
  maximize: DecoButton;
  bgOpacityActive: number;
  bgOpacityInactive: number;
}

// Mirrors the user's KWin decoration (layout + Klassy button palette) so our
// client-drawn titlebar matches what their system would have rendered.
export const decoState = $state<{ value: WindowDeco | null }>({ value: null });

let started = false;

export function loadDecoration() {
  if (started || !isTauri) return;
  started = true;
  invoke<WindowDeco>("kde_window_decoration")
    .then((d) => {
      decoState.value = d;
    })
    .catch(() => {});
}
