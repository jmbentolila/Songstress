import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { library, LIVE_LIBRARY } from "./library.svelte";
import { playback } from "./playback.svelte";
// NOTE: roots (Step 7c) are managed inside the Music folders modal, not from
// the menu — library.add-folder is the Step 2a IMPORT path, unrelated to it.
import { scanner, rescan, rescanFull, saveImports, openMusicFolders } from "./scanner.svelte";
import { addMusicFiles, addMusicFolder } from "./imports.svelte";
import { cycleShuffle, cycleRepeat, albumSkip, setEqEnabled, cycleEqPreset } from "./playback.svelte";
import { cycleTheme, resolvedTheme, ui } from "./ui.svelte";

export type MenuItem = {
  id: string;
  label: string;
  enabled: boolean;
  checked: boolean | null;
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
    "library.save-imports": () => saveImports(),
    "library.choose-folder": () => openMusicFolders(),
    "view.theme": () => cycleTheme(),
    "playback.shuffle": () => cycleShuffle(),
    "playback.repeat": () => cycleRepeat(),
    "playback.eq": () => setEqEnabled(!playback.eq.enabled),
    "playback.eq-preset": () => cycleEqPreset(),
    "playback.eq-customize": () => (ui.eqOpen = true),
    "playback.album-prev": () => albumSkip(-1),
    "playback.album-next": () => albumSkip(1),
    "help.about": () => {},
  };
  const fn = run[id];
  if (fn) void Promise.resolve(fn()).catch((e) => console.error(e));
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
  // fall back: the sidebar gear menu is the in-app fallback from now on.
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
  anyStaged: boolean;
  themeDark: boolean;
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
    anyStaged: library.albums.some((a) => a.staged),
    themeDark: resolvedTheme() === "dark",
  };
  void invoke("set_menu_state", { state }).catch(() => {});
}
