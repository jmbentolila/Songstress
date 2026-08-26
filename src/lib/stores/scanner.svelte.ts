import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { library } from "./library.svelte";
import { ui } from "./ui.svelte";
import { openContextMenu } from "./contextMenu.svelte";

/**
 * Shared scan orchestration for the title-bar menu and the empty state:
 * running state, live progress, and the folder picker flow.
 */
export const scanner = $state({
  running: false,
  phase: "" as "" | "scan" | "artwork" | "import",
  done: 0,
  total: 0,
});

let started = false;
export function initScanner() {
  if (started || !library.live) return;
  started = true;
  void listen("scan-progress", (e) => {
    const p = e.payload as { phase?: string; done: number; total: number };
    scanner.phase =
      p.phase === "artwork" ? "artwork" : p.phase === "import" ? "import" : "scan";
    scanner.done = p.done;
    scanner.total = p.total;
  });
}

export async function rescan() {
  if (scanner.running) return;
  scanner.running = true;
  try {
    await invoke("scan_library");
  } catch (err) {
    console.error("scan failed", err);
  } finally {
    // scan-finished also flips library.scanning, but make sure a failed/
    // no-op run never leaves us stuck.
    scanner.running = false;
  }
}

/** Reparse every file even when (mtime, size) is unchanged — rebuilds album
 *  grouping after scanner-logic changes (skipped files never regroup). */
export async function rescanFull() {
  if (scanner.running) return;
  scanner.running = true;
  try {
    await invoke("scan_library", { full: true });
  } catch (err) {
    console.error("full scan failed", err);
  } finally {
    scanner.running = false;
  }
}

/** Refresh the folder list from the DB (Step 7c). Best-effort. */
export async function loadMusicFolders(): Promise<void> {
  try {
    ui.musicFolders = await invoke<string[]>("get_music_folders");
  } catch {
    ui.musicFolders = [];
  }
}

/** Open the Music folders modal and refresh the list. */
export function openMusicFolders(): void {
  ui.musicFoldersOpen = true;
  void loadMusicFolders();
}

/** Add a library root (kdialog picker when path is null) → persist + rescan.
 *  Returns the updated folder list, or null if the user cancelled the picker. */
export async function addMusicFolderRoot(): Promise<void> {
  if (scanner.running) return;
  scanner.running = true;
  try {
    // path = null ⇒ backend opens kdialog starting at the primary root.
    const updated = await invoke<string[] | null>("add_music_folder", { path: null });
    if (updated) ui.musicFolders = updated;
  } catch (err) {
    console.error("add music folder failed", err);
  } finally {
    scanner.running = false;
  }
}

/** Remove a library folder and rescan — its tracks drop from the library,
 *  files on disk are NEVER touched. */
export async function removeMusicFolderRoot(path: string): Promise<void> {
  if (scanner.running) return;
  scanner.running = true;
  try {
    const updated = await invoke<string[]>("remove_music_folder", { path });
    ui.musicFolders = updated;
  } catch (err) {
    console.error("remove music folder failed", err);
  } finally {
    scanner.running = false;
  }
}

// --- Import staging (PLAN.md Step 2a) ----------------------------------------

/** Copy files/folders into the import staging area, then rescan. */
export async function importMusic(paths: string[]): Promise<void> {
  if (paths.length === 0 || scanner.running) return;
  scanner.running = true;
  try {
    await invoke("import_music", { paths });
  } catch (err) {
    console.error("import failed", err);
  } finally {
    scanner.running = false;
  }
}

/** kdialog multi-file picker → importMusic. */
export async function addMusicFiles(): Promise<void> {
  const files = await invoke<string[] | null>("choose_import_files");
  if (files) await importMusic(files);
}

/** kdialog folder picker → importMusic. */
export async function addMusicFolder(): Promise<void> {
  const folder = await invoke<string | null>("choose_import_folder");
  if (folder) await importMusic([folder]);
}

/** Copy staged music into the library dir. Scope: one album, one track, or
 * everything staged. */
export async function saveImports(albumId?: string, trackId?: string): Promise<void> {
  if (scanner.running) return;
  scanner.running = true;
  try {
    await invoke("save_imports", { albumId: albumId ?? null, trackId: trackId ?? null });
  } catch (err) {
    console.error("save imports failed", err);
  } finally {
    scanner.running = false;
  }
}

/** Delete staged music (same scoping); library untouched. */
export async function discardImports(albumId?: string, trackId?: string): Promise<void> {
  if (scanner.running) return;
  scanner.running = true;
  try {
    await invoke("discard_imports", { albumId: albumId ?? null, trackId: trackId ?? null });
  } catch (err) {
    console.error("discard imports failed", err);
  } finally {
    scanner.running = false;
  }
}

// --- Missing files (relink / remove) -----------------------------------------

export function notifyError(err: unknown) {
  console.error(err);
  openContextMenu(window.innerWidth / 2 - 160, 90, [
    { label: String(err).slice(0, 120), action: () => {} },
  ]);
}

/** kdialog locate → relink_track → rescan. */
export async function locateMissingTrack(trackId: string): Promise<void> {
  try {
    const picked = await invoke<string | null>("choose_relink_file", {
      start: ui.musicFolders[0] ?? "",
    });
    if (!picked) return;
    await invoke("relink_track", { trackId, newPath: picked });
    await rescan();
  } catch (err) {
    notifyError(err);
  }
}

/** Explicitly remove a missing track's row from the library. */
export async function removeTrack(trackId: string): Promise<void> {
  try {
    await invoke("delete_track", { trackId });
    await rescan();
  } catch (err) {
    notifyError(err);
  }
}

/** Remove every missing track (optionally scoped to one album). */
export async function removeMissingTracks(albumId?: string): Promise<void> {
  try {
    await invoke("delete_missing_tracks", { albumId: albumId ?? null });
    await rescan();
  } catch (err) {
    notifyError(err);
  }
}
