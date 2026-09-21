import { announcer } from "./announcer.svelte";
import { invoke } from "@tauri-apps/api/core";
import type { Theme } from "../types";
import { isTauri } from "../window";

function load<T>(key: string, fallback: T): T {
  try {
    const raw = localStorage.getItem(key);
    return raw === null ? fallback : (JSON.parse(raw) as T);
  } catch {
    return fallback;
  }
}

function prefersDark(): Theme {
  return typeof matchMedia === "function" && matchMedia("(prefers-color-scheme: light)").matches
    ? "light"
    : "dark";
}

// The tile/row defaults sit in ONE place: load() seeds the store with them,
// and resetAppearance restores exactly them — the two used to agree only by
// the courtesy of the literal 180/36 appearing twice (owner note, 2026-09-03).
const DEFAULT_TILE_SIZE = 180;
const DEFAULT_SIDEBAR_ROWS = 36;

export const ui = $state({
  theme: load<"light" | "dark" | "system">("songstress.theme", "system"),
  tileSize: load("songstress.tileSize", DEFAULT_TILE_SIZE),
  sidebarRowSize: load("songstress.sidebarRowSize", DEFAULT_SIDEBAR_ROWS),
  /** Playbar backdrop = artwork gradient while something is playing (Step 2b). */
  playbarGradient: load("songstress.playbarGradient", false),
  /** Expanded-panel artwork gradient (Step 9b). True = the shipped look;
   *  false = plain --panel-bg everywhere, per-album overrides included. */
  albumGradient: load("songstress.albumGradient", true),
  /** Per-album gradient overrides (Step 9b): album id → raw hex pair in DB
   *  form (6-digit, no #). Wins over the scan colors; clearing the entry
   *  returns the album to its artwork colors. Your hex, your problem — no
   *  contrast clamping (unlike the playbar's artGradientContrast). */
  panelGradients: load<Record<string, { c1: string; c2: string }>>(
    "songstress.panelGradients",
    {},
  ),
  /** Accent color (Step 4) — ONE hex; null = stock purple from app.css. */
  accentColor: load<string | null>("songstress.accentColor", null),
  /** In-sidebar menu (Step 8b, iOS Settings-style stack): the hamburger swaps
   *  the artist stack for the menu stack; menuDetail = pushed pane
   *  ("appearance" or a Rust menu id). Session-only. */
  menuOpen: false,
  menuDetail: null as string | null,
  activeArtistId: "all" as string,
  /** 4th stack layer (the accent picker), pushed from a pane. Session-only,
   *  like menuDetail; null = no sub layer open. */
  menuSub: null as string | null,
  /** Single library search (session-only): filters the sidebar artist list
   *  AND switches the grid to Songs/Albums match sections when non-empty. */
  search: "",
  /** One expanded panel PER grid section: "songs" (library search, shows
   *  matching tracks only) and "albums" (title/artist matches + the normal
   *  grid, full album). The same album can be open in both at once. */
  expandedAlbum: {
    songs: null as string | null,
    albums: null as string | null,
  },
  /** Library roots (Step 7c) — musicDirs, hydrated from the DB settings. */
  musicFolders: [] as string[],
  /** Epoch ms of the last completed scan (DB-backed settings) — the Library
   *  pane footer answers "when was this built?" instead of staying silent. */
  lastScan: null as number | null,
  /** "Music folders…" modal (Step 7c); session-only. */
  musicFoldersOpen: false,
  /** Tag editor modal (Step 1): exactly one of albumId/trackId is set. */
  tagEditor: {
    open: false,
    albumId: null as string | null,
    trackId: null as string | null,
  },
  /** Equalizer popover over the playbar (Step 6); session-only — the
   *  Playback menu's "Customize…" item opens it remotely. */
  eqOpen: false,
  /** Playbar queue popover (session-only, Step 7a). */
  queueOpen: false,
  /** "About Songstress" dialog — settings root footer row; session-only. */
  aboutOpen: false,
  /** Return address for the modal: whoever opened it, captured at open time. */
  aboutOpener: null as HTMLElement | null,
});

// Opening a modal records where focus came from, so dismissal can put it back.
// Recorded here rather than read on unmount: by then the dialog's own focus trap
// has already moved activeElement inside, so the trigger is no longer findable.
export function openAbout(from: HTMLElement | null = null) {
  ui.aboutOpener = from ?? (document.activeElement as HTMLElement | null);
  ui.aboutOpen = true;
}

export function resolvedTheme(): Theme {
  return ui.theme === "system" ? prefersDark() : ui.theme;
}

// The Appearance pane's footer reset — shared with the Global Menu's
// "Reset appearance" row so the two surfaces can never disagree about what
// "reset" means (was a Sidebar-local function until the Global Menu pass,
// 2026-09-03).
export function resetAppearance() {
  ui.tileSize = DEFAULT_TILE_SIZE;
  ui.sidebarRowSize = DEFAULT_SIDEBAR_ROWS;
  ui.theme = "system";
  ui.accentColor = null;
  ui.playbarGradient = false; // the pane's other visible control; reset means reset
  ui.albumGradient = true;
  ui.panelGradients = {}; // custom panel colors are appearance too
}

/** Per-album panel gradient override (Step 9b): hex WITHOUT # (DB form).
 *  Applies instantly — this is display state, not file tags, so it never
 *  waits for the modal's Save. Survives rescans: it lives in settings,
 *  keyed by album id, never in the color_c1/c2 columns a scan rewrites. */
export function setPanelGradient(albumId: string, c1: string, c2: string) {
  ui.panelGradients = { ...ui.panelGradients, [albumId]: { c1, c2 } };
}

/** Forget the override: the album returns to its artwork colors. */
export function clearPanelGradient(albumId: string) {
  const { [albumId]: _, ...rest } = ui.panelGradients;
  ui.panelGradients = rest;
}

// --- SQLite-backed settings (Phase 2 M4) ------------------------------------
// localStorage stays as the instant-paint mirror; the DB is authoritative
// once hydrated. Writes go through debounced set_setting.

const pendingWrites = new Map<string, ReturnType<typeof setTimeout>>();

export function pushSetting(key: string, value: unknown) {
  if (!isTauri) return;
  const json = JSON.stringify(value);
  const existing = pendingWrites.get(key);
  if (existing) clearTimeout(existing);
  pendingWrites.set(
    key,
    setTimeout(() => {
      pendingWrites.delete(key);
      // A failed persist used to be invisible forever — the UI believed
      // the write, the pane read back the lie. The announcer is the
      // cheapest honest surface: no visual syntax for a failure mode
      // nobody should see, but nobody should be silently lied to either
      // (critique re-run 2026-09-04; "who knows if someone else ends up
      // using it").
      void invoke("set_setting", { key, value: json }).catch(() =>
        announcer.say("Could not save a setting — your change may not stick next launch."),
      );
    }, 300),
  );
}

let hydrated = false;
export async function initSettings() {
  if (!isTauri || hydrated) return;
  hydrated = true;
  try {
    // Seed first-run migration from whatever localStorage knows.
      await invoke("init_settings", {
        values: {
          theme: JSON.stringify(ui.theme),
          tileSize: JSON.stringify(ui.tileSize),
          sidebarRowSize: JSON.stringify(ui.sidebarRowSize),
          playbarGradient: JSON.stringify(ui.playbarGradient),
          albumGradient: JSON.stringify(ui.albumGradient),
          panelGradients: JSON.stringify(ui.panelGradients),
          accentColor: JSON.stringify(ui.accentColor),
        },
      });
    const all = await invoke<Record<string, string>>("get_settings");
    const parse = <T,>(key: string, fallback: T): T => {
      if (all[key] === undefined) return fallback;
      try {
        return JSON.parse(all[key]) as T;
      } catch {
        return fallback;
      }
    };
    ui.theme = parse<(typeof ui)["theme"]>("theme", ui.theme);
    ui.tileSize = parse<number>("tileSize", ui.tileSize);
    ui.sidebarRowSize = parse<number>("sidebarRowSize", ui.sidebarRowSize);
    ui.playbarGradient = parse<boolean>("playbarGradient", ui.playbarGradient);
    ui.albumGradient = parse<boolean>("albumGradient", ui.albumGradient);
    ui.panelGradients = parse<typeof ui.panelGradients>(
      "panelGradients",
      ui.panelGradients,
    );
    ui.accentColor = parse<string | null>("accentColor", ui.accentColor);
    // musicDirs is the migrated multi-root list (setup writes it from the
    // legacy musicDir on first launch); parse falls back to [] when absent.
    ui.musicFolders = parse<string[]>("musicDirs", ui.musicFolders);
    ui.lastScan = parse<number | null>("lastScan", ui.lastScan);
  } catch {
    // No backend (browser dev) — localStorage keeps working. On Tauri a
    // failure here means the DB exists but said no: defaults are about to
    // be presented as your settings, which deserves the same one sentence.
    if (isTauri) announcer.say("Could not load your saved settings — using defaults.");
  }
}
