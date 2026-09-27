import { cubicOut } from "svelte/easing";

/**
 * The in-modal confirmation's entrance: it DROPS from the top edge of the
 * modal it belongs to (owner rule, 2026-09-26: "confirmations inside modals
 * should drop down from the top of the modal as notification-style boxes
 * instead of a modal on top of a modal").
 *
 * 200ms, cubicOut, and a short 10px travel — the box belongs to the surface
 * behind it, so it arrives rather than flies. Svelte runs the same generator
 * backwards for the outro, which is the house rule: every entrance wears its
 * mirror exit. JS-driven, so reduced motion is honored here (the stylesheet's
 * kill switch cannot reach a transition), and it stays inside the modal's own
 * bounds — nothing paints over the scrim above.
 */
export function notifyDrop(_node: Element, _params = {}) {
  const reduce = matchMedia("(prefers-reduced-motion: reduce)").matches;
  return {
    duration: reduce ? 0 : 200,
    easing: cubicOut,
    css: (t: number) =>
      `opacity: ${t}; transform: translateY(${(-10 * (1 - t)).toFixed(2)}px); will-change: transform, opacity`,
  };
}

/**
 * The dim the in-modal confirmation lays over its OWN surface: the same wash
 * strength and the same 200ms entrance as the app's modal scrim
 * (`app.css .scrim`: rgba(0, 0, 0, 0.35), scrim-in), scoped to the panel a
 * modal opened FROM instead of the window (owner ask, 2026-09-26: "darken the
 * modal just like you do for the app when the modal is invoked").
 */
export function notifyVeil(_node: Element, _params = {}) {
  const reduce = matchMedia("(prefers-reduced-motion: reduce)").matches;
  return {
    duration: reduce ? 0 : 200,
    easing: cubicOut,
    css: (t: number) => `opacity: ${t}`,
  };
}
