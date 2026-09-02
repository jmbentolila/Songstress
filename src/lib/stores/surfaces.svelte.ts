import { ui } from "./ui.svelte";
import { imports } from "./imports.svelte";
import { contextMenu } from "./contextMenu.svelte";

/**
 * Is a FLOATING surface on screen — a modal, a popover, or the context menu?
 *
 * This exists to answer ONE question: who owns the Escape key. The rule is
 * one press, one verb, topmost surface first — so while a surface is up, the
 * sidebar's global key router stands down and the surface's own handler answers.
 *
 * Why a predicate and not `stopPropagation`: every one of these handlers listens
 * on `window` (`<svelte:window onkeydown>` in About, Music folders, Tag editor,
 * Manage imports, the playbar popovers, the sidebar router). Listeners on the same
 * node and phase ALL run regardless of stopPropagation — only
 * stopImmediatePropagation cuts them short, and the order they were attached in is
 * not guaranteed across components (Svelte re-attaches them on update). So the
 * precedence has to be decided by a fact each handler checks, not by who happens
 * to fire first. That race is what closed a dialog AND a menu level on one
 * keypress, twice now: once for About (fixed by moving its handler into the
 * router), and once for every other surface (fixed by this).
 *
 * It is a plain function, not a `$derived`: it is called from an event handler,
 * once per keypress, and nothing renders from it.
 */
export function surfaceOpen(): boolean {
  return (
    ui.aboutOpen ||
    ui.musicFoldersOpen ||
    ui.tagEditor.open ||
    ui.eqOpen ||
    ui.queueOpen ||
    imports.open ||
    contextMenu.open
  );
}
