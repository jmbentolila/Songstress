import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { library } from "./library.svelte";
import { ui } from "./ui.svelte";
import { openContextMenu } from "./contextMenu.svelte";
import type { ImportReport, SaveReport } from "../importPlan";

/**
 * Shared scan orchestration for the title-bar menu and the empty state:
 * running state, live progress, and the folder picker flow.
 */
export const scanner = $state({
  running: false,
  /** Which verb is running. The Library pane hangs its progress ring off this,
   * on the row whose own action is in flight — so it is set by the operation,
   * never by the caller: a scan started from the Global Menu, the empty state or
   * a relink has no click in the pane to hang it off. */
  kind: "" as "" | Kind,
  /** Which row keeps showing a COMPLETE arc after its operation finished. See
   * `end()`. */
  heldKind: "" as "" | Kind,
  phase: "" as "" | Phase,
  done: 0,
  total: 0,
});

type Kind = "scan" | "full" | "import" | "save" | "discard" | "folder";
type Phase = "scan" | "artwork" | "import";

/** A determinate indicator that vanishes at 85% reads as an interruption: the
 * backend's last progress event is usually short of the end, so unmounting the
 * arc in the same frame as the run leaves an unfinished ring on the row that was
 * just asked to do something. So the arc is held at full for a beat — long enough
 * to register as "finished", too short to feel like a stage. Only runs that
 * reported progress get the frame; an operation that never emitted an event never
 * showed an arc to complete. */
const HOLD_MS = 300;

function end() {
  const kind = scanner.kind;
  const reported = scanner.total > 0;
  scanner.running = false;
  scanner.kind = "";
  if (!reported) return;
  scanner.heldKind = kind;
  setTimeout(() => {
    if (scanner.heldKind === kind) scanner.heldKind = "";
  }, HOLD_MS);
}

/** Every operation starts from no news. `done`/`total` survive a finished run,
 * so without this the ring appears at the PREVIOUS run's 100% and the first
 * progress event unwinds it counter-clockwise before it starts climbing — which
 * reads as "it did a lap before it began". Same for `phase`: a re-read must not
 * announce itself as last run's artwork pass. */
function begin(kind: Kind) {
  scanner.running = true;
  scanner.kind = kind;
  scanner.phase = "";
  scanner.done = 0;
  scanner.total = 0;
}

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
  begin("scan");
  try {
    await invoke("scan_library");
  } catch (err) {
    console.error("scan failed", err);
  } finally {
    // scan-finished also flips library.scanning, but make sure a failed/
    // no-op run never leaves us stuck.
    end();
  }
}

/** Reparse every file even when (mtime, size) is unchanged — rebuilds album
 *  grouping after scanner-logic changes (skipped files never regroup). */
export async function rescanFull() {
  if (scanner.running) return;
  begin("full");
  try {
    await invoke("scan_library", { full: true });
  } catch (err) {
    console.error("full scan failed", err);
  } finally {
    end();
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
  begin("folder");
  try {
    // path = null ⇒ backend opens kdialog starting at the primary root.
    const updated = await invoke<string[] | null>("add_music_folder", { path: null });
    if (updated) ui.musicFolders = updated;
  } catch (err) {
    // validate_new_root rejections (duplicate / nested root) are user-facing,
    // and a console line is invisible in the webview.
    notifyError(err);
  } finally {
    end();
  }
}

/** Remove a library folder and rescan — its tracks drop from the library,
 *  files on disk are NEVER touched. */
export async function removeMusicFolderRoot(path: string): Promise<void> {
  if (scanner.running) return;
  begin("folder");
  try {
    const updated = await invoke<string[]>("remove_music_folder", { path });
    ui.musicFolders = updated;
  } catch (err) {
    notifyError(err);
  } finally {
    end();
  }
}

// --- Import staging -------------------------------------------------------
// `importMusic` lives in `imports.svelte.ts`: an import returns a receipt and
// opens the window that explains it, which is the store's business. What stays
// here is the work itself, because the scan state below is shared by every
// operation that rewrites the library.

/** Index files where they are and flag them pending — no copy is made. The
 *  receipt says what became pending and which of the files the library already
 *  held; the caller decides where to show it. */
export async function runImport(paths: string[]): Promise<ImportReport | null> {
  if (paths.length === 0 || scanner.running) return null;
  begin("import");
  try {
    return await invoke<ImportReport>("import_music", { paths });
  } catch (err) {
    console.error("import failed", err);
    return null;
  } finally {
    end();
  }
}

/** Copy staged music into the library dir. Scope: one album, one track, or
 * everything staged. */
export async function saveImports(
  albumId?: string,
  trackId?: string,
): Promise<SaveReport> {
  const none: SaveReport = { moved: 0, duplicates: 0, vanished: 0 };
  if (scanner.running) return none;
  begin("save");
  try {
    return await invoke<SaveReport>("save_imports", {
      albumId: albumId ?? null,
      trackId: trackId ?? null,
    });
  } catch (err) {
    console.error("save imports failed", err);
    return none;
  } finally {
    end();
  }
}

/** Delete staged music (same scoping); library untouched. */
export async function discardImports(albumId?: string, trackId?: string): Promise<number> {
  if (scanner.running) return 0;
  begin("discard");
  try {
    return await invoke<number>("discard_imports", {
      albumId: albumId ?? null,
      trackId: trackId ?? null,
    });
  } catch (err) {
    console.error("discard imports failed", err);
    return 0;
  } finally {
    end();
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
