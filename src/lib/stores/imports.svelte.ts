import { invoke } from "@tauri-apps/api/core";
import { applyOrder, type Decision, type StagedAlbum } from "../importPlan";
import { discardImports, saveImports } from "./scanner.svelte";

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
  applying: false,
  /** Ops finished / ops marked, for the footer's arc. */
  done: 0,
  total: 0,
  /** The album currently being moved or deleted, for the arc's sentence. */
  active: "",
  /** Set from the state after each op, not from a thrown error: the truth about
   *  a save is whether the album is still staged, not whether the command threw. */
  failed: [] as string[],
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

export async function openImportManager(): Promise<void> {
  imports.open = true;
  await refreshImportPlan();
  // One album waiting: show what is in it straight away. The decision is about
  // that album, and the file list is the reassurance — making the user click to
  // read the contents of the only thing on screen is busywork.
  if (imports.plan.length === 1) imports.expanded[imports.plan[0].albumId] = true;
}

export function closeImportManager(): void {
  imports.open = false;
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
  imports.total = ops.length;
  imports.done = 0;
  for (const op of ops) {
    imports.active = op.label;
    // The shared verbs own the scan state and the progress events; they report
    // failure by leaving the album staged, which is what the check below reads.
    if (op.decision === "save") await saveImports(op.albumId);
    else await discardImports(op.albumId);
    await refreshImportPlan();
    if (imports.plan.some((a) => a.albumId === op.albumId)) imports.failed.push(op.label);
    imports.done++;
  }
  imports.active = "";
  imports.applying = false;
  // Everything staged is decided and gone: the door it was opened from is about
  // to disappear from the pane, so the window goes with it.
  if (imports.plan.length === 0) closeImportManager();
}
