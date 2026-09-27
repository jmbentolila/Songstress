import { cubicOut } from "svelte/easing";

/**
 * The entrance every menu wears: the surface grows out of the point it was
 * anchored to and shrinks back into it — owner rule: every entrance carries
 * its mirror exit. 125 ms, cubicOut (the accordion's easing), opacity + a 0.97
 * scale. Svelte runs the SAME generator backwards for the outro: same path,
 * same duration, free symmetry. JS-driven, so reduced-motion is honored in JS
 * (the stylesheet's kill switch cannot reach these).
 *
 * Shared by the context menu and the editors' suggestion list, so "looks like
 * our own menus" is true by construction rather than by two people matching
 * numbers. Each site pins `transform-origin` to its anchoring corner.
 */
export function menuPop(_node: Element, _params = {}) {
  const reduce = matchMedia("(prefers-reduced-motion: reduce)").matches;
  return {
    duration: reduce ? 0 : 125,
    easing: cubicOut,
    css: (t: number) =>
      `opacity: ${t}; transform: scale(${(0.97 + 0.03 * t).toFixed(4)}); will-change: transform, opacity`,
  };
}