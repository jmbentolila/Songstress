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

export const ui = $state({
  theme: load<"light" | "dark" | "system">("songstress.theme", "system"),
  tileSize: load("songstress.tileSize", 180),
  sidebarRowSize: load("songstress.sidebarRowSize", 36),
  /** Playbar backdrop = artwork gradient while something is playing (Step 2b). */
  playbarGradient: load("songstress.playbarGradient", false),
  /** Accent color (Step 4) — ONE hex; null = stock purple from app.css. */
  accentColor: load<string | null>("songstress.accentColor", null),
  activeArtistId: "all" as string,
  filter: "",
  /** Titlebar search (albums/songs) — session-only, overrides artist filter. */
  mediaFilter: "",
  /** One expanded panel PER grid section: "songs" (titlebar search, shows
   *  matching tracks only) and "albums" (title/artist matches + the normal
   *  grid, full album). The same album can be open in both at once. */
  expandedAlbum: {
    songs: null as string | null,
    albums: null as string | null,
  },
  /** Library roots (Step 7c) — musicDirs, hydrated from the DB settings. */
  musicFolders: [] as string[],
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
});

export function resolvedTheme(): Theme {
  return ui.theme === "system" ? prefersDark() : ui.theme;
}

export function cycleTheme() {
  ui.theme = resolvedTheme() === "dark" ? "light" : "dark";
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
      void invoke("set_setting", { key, value: json }).catch(() => {});
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
    ui.accentColor = parse<string | null>("accentColor", ui.accentColor);
    // musicDirs is the migrated multi-root list (setup writes it from the
    // legacy musicDir on first launch); parse falls back to [] when absent.
    ui.musicFolders = parse<string[]>("musicDirs", ui.musicFolders);
  } catch {
    // No backend (browser dev) — localStorage keeps working.
  }
}
