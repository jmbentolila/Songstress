import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { library, LIVE_LIBRARY } from "./library.svelte";
import { playback } from "./playback.svelte";
// NOTE: roots (Step 7c) are managed inside the Music folders modal, not from
// the menu — library.add-folder is the Step 2a IMPORT path, unrelated to it.
import { scanner, rescan, rescanFull, openMusicFolders } from "./scanner.svelte";
import { addMusicFiles, addMusicFolder, openImportManager } from "./imports.svelte";
import {
  cycleShuffle,
  cycleRepeat,
  albumSkip,
  setEqEnabled,
  cycleEqPreset,
  resetPlayback,
} from "./playback.svelte";
import { resetAppearance, openAbout, ui } from "./ui.svelte";

export type MenuItem = {
  id: string;
  label: string;
  enabled: boolean;
  checked: boolean | null;
  /** Radio-dot checkable (the theme trio) vs checkmark. */
  radio?: boolean;
  /** Section break — mirrors the sidebar panes' group gaps. */
  separator?: boolean;
};

export type Menu = { id: string; label: string; items: MenuItem[] };

/** Mirrors the Rust-owned model (menu.rs) — refreshed via menu-changed.
 *  globalMenuActive: assume the Plasma Global Menu from the start and only
 *  flag it inactive if registration definitively fails. The in-app menu
 *  (sidebar stack, Step 8b) doesn't branch on this — it's always available. */
export const menu = $state({
  menus: [] as Menu[],
  globalMenuActive: true,
});

function handleAction(id: string) {
  const run: Record<string, () => unknown> = {
    "library.rescan": () => rescan(),
    "library.rescan-full": () => rescanFull(),
    "library.add-files": () => addMusicFiles(),
    "library.add-folder": () => addMusicFolder(),
    // The sidebar's staged-pile door, mirrored: opens the manage modal
    // (Save/Discard live there), instead of applying blind from the panel.
    "library.manage-imports": () => openImportManager(),
    "library.choose-folder": () => openMusicFolders(),
    "appearance.theme-system": () => { ui.theme = "system"; },
    "appearance.theme-light": () => { ui.theme = "light"; },
    "appearance.theme-dark": () => { ui.theme = "dark"; },
    "appearance.playbar-gradient": () => { ui.playbarGradient = !ui.playbarGradient; },
    "appearance.album-gradient": () => { ui.albumGradient = !ui.albumGradient; },
    // Rows the menu cannot render: open the sidebar stack AT the pane (the
    // size sliders) or the pane's sub layer (the accent picker).
    "appearance.accent": () => openSettingsLayer("appearance", "accent"),
    "appearance.more": () => openSettingsLayer("appearance", null),
    "appearance.reset": () => resetAppearance(),
    "playback.reset": () => resetPlayback(),
    "playback.shuffle": () => cycleShuffle(),
    "playback.repeat": () => cycleRepeat(),
    "playback.eq": () => setEqEnabled(!playback.eq.enabled),
    "playback.eq-preset": () => cycleEqPreset(),
    "playback.eq-customize": () => { ui.eqOpen = true; },
    "playback.album-prev": () => albumSkip(-1),
    "playback.album-next": () => albumSkip(1),
    // The old no-op: the sidebar's About footer passes its trigger element as
    // the focus-restoration opener; from the panel there is none, and
    // openAbout's null fallback covers it.
    "help.about": () => openAbout(),
  };
  const fn = run[id];
  if (fn) void Promise.resolve(fn()).catch((e) => console.error(e));
}

/** Open the sidebar menu stack at a pane (and optionally its sub layer) —
 *  the Global Menu's door into controls a menu bar cannot host (sliders,
 *  the color grid). State-only: the window typically lacks keyboard focus
 *  when its panel menu is clicked, so there is no focus to hand off. */
function openSettingsLayer(detail: string, sub: string | null) {
  ui.menuOpen = true;
  ui.menuDetail = detail;
  ui.menuSub = sub;
}

/** Item click from any frontend-rendered menu surface: one activation path
 *  through Rust (playback handled natively there, rest comes back as
 *  menu-action). */
export function activateMenuItem(id: string) {
  void invoke("menu_activate", { id }).catch((e) => console.error(e));
}

let wired = false;

/** Idempotent. Fetches the model and subscribes to updates + actions.
 *  Dynamic state (playing/scanning/staged/theme) is pushed by pushMenuState
 *  from an $effect in App.svelte. */
export async function initMenu() {
  if (wired || !LIVE_LIBRARY || !library.live) return;
  wired = true;
  await listen("menu-changed", (e) => {
    menu.menus = e.payload as Menu[];
  });
  await listen("menu-action", (e) => handleAction(e.payload as string));
  await listen("appmenu-registered", () => {
    menu.globalMenuActive = true;
  });
  // The registration races the webview load (retry loop takes seconds).
  // Poll until it resolves: 1 = keep assuming the Global Menu, 2 = failed →
  // fall back: the sidebar hamburger menu is the in-app fallback from now on.
  const poll = setInterval(() => {
    void invoke<number>("appmenu_state")
      .then((state) => {
        if (state === 2) menu.globalMenuActive = false;
        if (state !== 0) clearInterval(poll);
      })
      .catch(() => clearInterval(poll));
  }, 1000);
  try {
    menu.menus = await invoke("get_menu");
  } catch (e) {
    console.error(e);
  }
}

type MenuState = {
  playing: boolean | null;
  hasTrack: boolean;
  scanning: boolean;
  stagedCount: number;
  /** The MODE, not the resolved theme — "system" must be reachable AND
   *  reported from the menu (the old bool flattened it away). */
  theme: string;
  playbarGradient: boolean;
  albumGradient: boolean;
  shuffle: string;
  repeat: string;
  eqEnabled: boolean;
  eqPreset: string;
};

export function pushMenuState() {
  if (!LIVE_LIBRARY || !library.live) return;
  const state: MenuState = {
    playing: playback.current ? playback.isPlaying : null,
    hasTrack: playback.current !== null,
    shuffle: playback.shuffle,
    repeat: playback.repeat,
    eqEnabled: playback.eq.enabled,
    // "Custom" when the gains diverged from every preset (preset = null).
    eqPreset: playback.eq.preset ?? "Custom",
    scanning: scanner.running,
    stagedCount: library.albums.reduce((n, a) => n + (a.staged ? 1 : 0), 0),
    theme: ui.theme,
    playbarGradient: ui.playbarGradient,
    albumGradient: ui.albumGradient,
  };
  void invoke("set_menu_state", { state }).catch(() => {});
}
