import { invoke } from "@tauri-apps/api/core";
import {
  applyOrder,
  type Decision,
  type ImportReport,
  type SaveReport,
  type StagedAlbum,
} from "../importPlan";
import { announcer } from "./announcer.svelte";
import { library } from "./library.svelte";
import type { Album } from "../types";
import { ui } from "./ui.svelte";
import { discardImports, runImport, saveImports, scanner } from "./scanner.svelte";

/**
 * State behind the "Manage imported music" modal.
 *
 * The modal does not act row by row: marking an album says what SHOULD happen,
 * and Apply is the single verb that makes it happen. That is why a marked row
 * can be unmarked, why closing the window loses nothing, and why an album nobody
 * marked is still staged the next time the door is opened — "decide later" has to
 * be a legal state in a flow that can also be left halfway.
 */
export const imports = $state({
  open: false,
  plan: [] as StagedAlbum[],
  /** albumId → the decision the user has marked. Unset means undecided. */
  decisions: {} as Record<string, Decision>,
  expanded: {} as Record<string, boolean>,
  /** The album a door opened this window AT (badge, album menu): the modal
   * expands that row, scrolls it into view, and flashes it once. Consumed
   * by the modal, never survives a close. */
  focus: null as string | null,
  applying: false,
  /** Ops finished / ops marked, for the footer's arc. */
  done: 0,
  total: 0,
  /** The album currently being moved or deleted, for the arc's sentence. */
  active: "",
  /** Set from the state after each op, not from a thrown error: the truth about
   *  a save is whether the album is still staged, not whether the command threw. */
  failed: [] as string[],
  /** Apply is pausing for a scan it did not start (see `waitForIdle`). */
  waiting: false,
  /** What the last import did. Shown until the window closes: pointing at files
   *  the library already holds is not a failure, but it looks like one when the
   *  progress ring ends and the screen is unchanged. */
  report: null as ImportReport | null,
  /** What the last Apply did, including the one deletion of a file the user owns
   *  that this app performs (an identical file they just pointed at). */
  applied: null as (SaveReport & { discarded: number }) | null,
});

export async function refreshImportPlan(): Promise<void> {
  try {
    imports.plan = await invoke<StagedAlbum[]>("staged_import_plan");
    // A decision for an album that is no longer staged (saved from the Global
    // Menu while the modal was open) would otherwise sit in the summary forever.
    for (const id of Object.keys(imports.decisions)) {
      if (!imports.plan.some((a) => a.albumId === id)) delete imports.decisions[id];
    }
    for (const id of Object.keys(imports.expanded)) {
      if (!imports.plan.some((a) => a.albumId === id)) delete imports.expanded[id];
    }
  } catch (err) {
    console.error("staged plan failed", err);
  }
}

export async function openImportManager(focusAlbumId?: string): Promise<void> {
  // Fetch, THEN open. The plan used to arrive a frame or two AFTER the window, which
  // means the modal mounted showing the PREVIOUS pile and re-filled while it was still
  // animating in — a surface that changes height during its own entrance is not a
  // timing problem to tune around, it is a different bug wearing the symptom.
  //
  // No loading state and no skeleton, deliberately: `staged_plan` is a
  // `WHERE staged = 1` query plus three indexed reads and a directory probe per staged
  // album — single-digit milliseconds, against the app's own yardstick of a 56ms
  // full-library scan. Waiting for it is invisible; inventing a placeholder for it
  // would have been a delay we manufactured.
  await refreshImportPlan();
  // Every open starts from ONE rule: the shape of the window is a function of the
  // pile, not of what the user last clicked. `expanded` used to survive a close, so
  // reopening showed whatever was open before.
  //
  //   one album  → shown open. It is the whole subject of the window; making the user
  //                click to read the contents of the only thing on screen is busywork,
  //                and the file list is the reassurance the decision needs.
  //   more       → all collapsed, and the caret is there to be asked.
  imports.expanded = {};
  if (imports.plan.length === 1) imports.expanded[imports.plan[0].albumId] = true;
  // A door that names an album (the panel's Imported badge, an album menu)
  // is asking about THAT album: its row opens even inside a tall pile, and
  // the modal flashes it once (ManageImports consumes `focus`). Focus is
  // additive — the shape of the window is still a function of the pile.
  if (focusAlbumId && imports.plan.some((a) => a.albumId === focusAlbumId)) {
    imports.expanded[focusAlbumId] = true;
    imports.focus = focusAlbumId;
  } else {
    imports.focus = null;
  }
  imports.open = true;
}

export function closeImportManager(): void {
  imports.open = false;
  imports.focus = null;
  imports.report = null;
  imports.applied = null;
  imports.applying = false;
  imports.active = "";
  imports.done = 0;
  imports.total = 0;
  imports.failed = [];
}

/** Mark, and un-mark by pressing the same button again: an undecided row is a
 *  state, not an error, so the way back out of a mark has to be the mark itself. */
export function markImport(albumId: string, decision: Decision): void {
  if (imports.decisions[albumId] === decision) delete imports.decisions[albumId];
  else imports.decisions[albumId] = decision;
}

/** Footer shortcut. Pressing it while every album already carries that mark
 *  clears them, so "Save all" is also the way back to an undecided pile. */
export function markAllImports(decision: Decision): void {
  const all = imports.plan.length > 0 && imports.plan.every((a) => imports.decisions[a.albumId] === decision);
  imports.decisions = all
    ? {}
    : Object.fromEntries(imports.plan.map((a) => [a.albumId, decision as Decision]));
}

export function toggleImportTracks(albumId: string): void {
  imports.expanded[albumId] = !imports.expanded[albumId];
}

/**
 * The verbs this loop calls refuse to run while a scan is in flight — so a save
 * issued during the scan that follows the import that made this pile would be
 * dropped without a word. Wait for it, and say so; give up rather than hang.
 */
async function waitForIdle(): Promise<boolean> {
  if (!scanner.running) return true;
  imports.waiting = true;
  const deadline = Date.now() + 60_000;
  while (scanner.running && Date.now() < deadline) {
    await new Promise((r) => setTimeout(r, 200));
  }
  imports.waiting = false;
  return !scanner.running;
}

/**
 * Perform the marked decisions, one album at a time, in the order the modal
 * shows them. Sequential on purpose: each verb ends with a library scan, and
 * running them concurrently would have two scans rewriting the same dump. After
 * each one the plan is re-read, so an album leaves the list when it is really
 * gone rather than when the loop happens to finish.
 */
export async function applyImportDecisions(): Promise<void> {
  if (imports.applying) return;
  const ops = applyOrder(imports.plan, imports.decisions);
  if (ops.length === 0) return;
  imports.applying = true;
  imports.failed = [];
  imports.applied = null;
  let moved = 0;
  let duplicates = 0;
  let vanished = 0;
  let discarded = 0;
  imports.total = ops.length;
  imports.done = 0;
  for (const op of ops) {
    imports.active = op.label;
    if (!(await waitForIdle())) {
      // The shared verbs refuse to run during a scan, and a refusal that is
      // invisible is worse than a delay that is said out loud.
      imports.failed.push(op.label);
      continue;
    }
    // The shared verbs own the scan state and the progress events; they report
    // failure by leaving the album staged, which is what the check below reads.
    if (op.decision === "save") {
      const report = await saveImports(op.albumId);
      moved += report.moved;
      duplicates += report.duplicates;
      vanished += report.vanished;
    } else {
      discarded += await discardImports(op.albumId);
    }
    imports.applied = { moved, duplicates, vanished, discarded };
    await refreshImportPlan();
    if (imports.plan.some((a) => a.albumId === op.albumId)) imports.failed.push(op.label);
    imports.done++;
  }
  imports.active = "";
  imports.waiting = false;
  imports.applying = false;
  // Everything staged is decided and gone: the door it was opened from is about
  // to disappear from the pane, so the window goes with it — UNLESS something
  // needs reporting. A save that removed a file the user owns stays open and
  // says so; a window that vanishes on a deletion is a window that hides the one
  // thing they should read.
  const needsReport =
    (imports.applied?.duplicates ?? 0) > 0 || (imports.applied?.vanished ?? 0) > 0;
  if (imports.plan.length === 0 && !needsReport) closeImportManager();
}

/** Import, then LAND on what it did: the artist tab switches to the first
 *  imported album's artist and that album expands in the grid. The modal no
 *  longer opens itself (owner ruling 2026-09-24: it interrupted; the landing
 *  IS the receipt for staged files) — it opens only on direct invocation
 *  (menu, badge, album menu), where `report` still renders. */
export async function importMusic(paths: string[]): Promise<void> {
  const report = await runImport(paths);
  if (!report) return;
  imports.report = report;
  imports.applied = null;
  await refreshImportPlan();
  // First of the imported, staged preferred: the plan order is the pile's
  // order, which is what "first on the list" means.
  const first = imports.plan[0]?.albumId ?? null;
  if (first) {
    const album = await waitForAlbum(first);
    if (album) revealInGrid(album.id, album.artistId);
    return;
  }
  // Nothing staged (pure "already own this"): land on the existing album
  // instead of ending the ring on an unchanged screen — with the sentence
  // the modal band used to say, heard.
  const known = report.already[0];
  if (!known) return;
  const hit =
    library.albums.find(
      (a) => a.title === known.title && library.artistOf(a)?.name === known.artist,
    ) ?? library.albums.find((a) => a.title === known.title);
  if (!hit) return;
  announcer.say(`Already in your library: ${known.artist} — ${known.title}`);
  revealInGrid(hit.id, hit.artistId);
}

/** Wait for the post-import dump to carry the album: the scan finished
 *  inside the invoke, but the dump refresh it triggers races our return.
 *  Gives up quietly — a failed scan leaves its own error state behind. */
async function waitForAlbum(albumId: string): Promise<Album | null> {
  const deadline = Date.now() + 15000;
  for (;;) {
    const hit = library.albums.find((a) => a.id === albumId);
    if (hit) return hit;
    if (Date.now() > deadline) return null;
    await new Promise((r) => setTimeout(r, 150));
  }
}

/** Switch context to one album: drop any menu layer (it would hide the
 *  landing), clear search (it would hide the album in match sections),
 *  switch the tab exactly the way the sidebar does — then ask the grid to
 *  expand + travel AFTER the tab's switch bridge (160ms exit), when the
 *  album is actually listed. Skipped if the user moved on meanwhile: the
 *  landing must never yank them back. One tick — Svelte batches. */
function revealInGrid(albumId: string, artistId: string): void {
  ui.menuOpen = false;
  ui.menuDetail = null;
  ui.menuSub = null;
  ui.search = "";
  ui.pendingOnly = false;
  ui.activeArtistId = artistId;
  ui.expandedAlbum.songs = null;
  ui.expandedAlbum.albums = null;
  const seq = (ui.gridReveal?.seq ?? 0) + 1;
  setTimeout(() => {
    if (ui.activeArtistId !== artistId) return;
    ui.gridReveal = { albumId, seq };
  }, 260);
}

/** kdialog multi-file picker → import. */
export async function addMusicFiles(): Promise<void> {
  const files = await invoke<string[] | null>("choose_import_files");
  if (files) await importMusic(files);
}

/** kdialog folder picker → import. */
export async function addMusicFolder(): Promise<void> {
  const folder = await invoke<string | null>("choose_import_folder");
  if (folder) await importMusic([folder]);
}
