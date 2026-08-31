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

// The decoration's button palette as CSS custom properties. It is applied to
// <html> (App.svelte) rather than only to the sidebar header, because the close
// affordance of every floating surface (popovers, modals) is a member of this
// family and must draw from the user's own decoration, not a hardcoded red.
export function decoVars(d: WindowDeco | null = decoState.value): Array<[string, string]> {
  const op = (d?.bgOpacityActive ?? 100) / 100;
  const rgba = (c: [number, number, number]) => `rgba(${c[0]}, ${c[1]}, ${c[2]}, ${op})`;
  const close = d?.close ?? { normal: [255, 95, 87], hover: [195, 63, 69] };
  const min = d?.minimize ?? { normal: [254, 188, 46], hover: [218, 165, 5] };
  const max = d?.maximize ?? { normal: [88, 251, 63], hover: [38, 148, 62] };
  return [
    ["--tb-x", rgba(close.normal)],
    ["--tb-x-hover", rgba(close.hover)],
    ["--tb-i", rgba(min.normal)],
    ["--tb-i-hover", rgba(min.hover)],
    ["--tb-a", rgba(max.normal)],
    ["--tb-a-hover", rgba(max.hover)],
  ];
}
