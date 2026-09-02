/**
 * Keep Tab inside an open dialog.
 *
 * `role="dialog" aria-modal="true"` is a promise: "everything else is gone". The
 * other half of that promise is `inert` on the background (App.svelte, driven by
 * `modalOpen()`), and this is the third half — the ring that stops focus walking
 * out of the panel's own edges. Without it, the last control in a modal Tabs to
 * the first control of a library the user cannot see, and the focus-return-on-close
 * logic then restores to a place the user never left from.
 *
 * It intercepts ONLY at the two edges, so the browser's own order inside the dialog
 * (DOM order, which matches the visual order in every one of these modals) stays
 * untouched. Call it from the surface's existing keydown handler:
 *
 *     if (e.key === "Tab") { trapTab(e, panel); return; }
 */
const FOCUSABLE =
  'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';

export function trapTab(
  e: KeyboardEvent,
  root: HTMLElement | null | undefined,
): void {
  if (!root) return;
  const items = [...root.querySelectorAll<HTMLElement>(FOCUSABLE)].filter(
    (el) => el.offsetParent !== null || el === document.activeElement,
  );
  if (items.length === 0) return;
  const first = items[0];
  const last = items[items.length - 1];
  const at = document.activeElement;
  if (e.shiftKey && (at === first || at === root)) {
    e.preventDefault();
    last.focus({ preventScroll: true });
  } else if (!e.shiftKey && at === last) {
    e.preventDefault();
    first.focus({ preventScroll: true });
  }
}
