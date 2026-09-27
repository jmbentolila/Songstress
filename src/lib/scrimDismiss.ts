/**
 * `use:scrimDismiss={close}` — dismiss a modal when the press STARTS on the
 * scrim itself, never on the panel.
 *
 * Why not `onclick` with the usual `e.target === e.currentTarget` guard: the
 * `click` event's target is the nearest common ancestor of its mousedown and
 * mouseup targets. A text selection that begins inside the panel and is
 * released over the backdrop has the backdrop as that common ancestor, so the
 * guard passed and the modal closed out from under the user's hand — the
 * selection ended outside and the panel vanished (owner report, 2026-09-26).
 * A `pointerdown` target is always the element under the pointer at press time,
 * so a press that starts inside can never be read as an outside press, no
 * matter where the pointer is released.
 *
 * Dismissal therefore happens on POINTER-DOWN — respond on press, the same
 * idiom as the ContextMenu and the playbar popovers (and Apple's own rule:
 * feedback belongs on the press, not on the tenth-of-a-second-later click).
 */

/** Primary button only, and only the primary touch — a right-click or a
 *  second finger on the backdrop is not a dismissal. */
function isDismissPress(e: PointerEvent): boolean {
  if (e.button !== 0) return false;
  if (e.isPrimary === false) return false;
  return true;
}

export function scrimDismiss(node: HTMLElement, close: () => void) {
  let cb = close;

  const onPointerDown = (e: PointerEvent) => {
    if (!isDismissPress(e)) return;
    // The panel is a child of the scrim, so "started on the panel" is exactly
    // "the target is not the scrim itself".
    if (e.target !== node) return;
    cb();
  };

  node.addEventListener("pointerdown", onPointerDown);

  return {
    update(next: () => void) {
      cb = next;
    },
    destroy() {
      node.removeEventListener("pointerdown", onPointerDown);
    },
  };
}