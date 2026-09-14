// Shared tooltip: every native `title` in the app converts to this — one
// pill, one style (the volume readout's), one behavior. A single singleton
// element serves all anchors, so 49 tooltips cost one node.
//
// Accessibility contract: the action captures the title text at mount,
// REMOVES the attribute (no double tooltip), and links it back via
// `aria-describedby` only while visible — screen readers hear label +
// description on focus, sighted users get the pill on hover/focus. Native
// `title` also misbehaves here (no delay control, no positioning, toolkit
// chrome), the same objection that bans native dialogs/selects in this app.
//
// Show rules: hover after 350ms, focus after 150ms; hide on leave, blur,
// scroll (capture — the anchor moved), Escape, or any pointerdown. Position:
// centered above the anchor, flipped below near the viewport top, clamped
// horizontally. Opacity-only entrance (state feedback), instant under
// reduced-motion (see app.css).
let tip: HTMLElement | null = null;
let current: HTMLElement | null = null;
let timer: ReturnType<typeof setTimeout> | undefined;
// True only after a real pointer move since the last blur/hide/activation:
// a restore remaps the window under a stationary pointer and the engine
// delivers a SYNTHETIC mouseenter for it — arming from that is the stuck
// tip (nothing ever sends the matching leave).
let seenMove = true;

function ensure(): HTMLElement {
  if (tip) return tip;
  tip = document.createElement("div");
  tip.className = "app-tip";
  tip.id = "app-tip";
  tip.setAttribute("role", "tooltip");
  tip.hidden = true;
  document.body.appendChild(tip);
  return tip;
}

// Page-load guards, NOT first-show guards (stuck-tip hole, found live
// 2026-09-14): the wiring above used to run inside ensure(), so when the
// session's FIRST trigger was itself the synthetic restore-enter, nothing
// was disarmed yet and the tip raised. The window flag survives HMR
// re-imports (dev-only) so a hot edit doesn't stack duplicate listeners.
// Stuck-tip guard (minimize/restore cycle): the restore can deliver a
// synthetic mouseenter for a pointer that never moved, and can refire
// focus on whatever held it (a dragged slider parks focus as a side
// effect) — either raises the tip with nothing to dismiss it until
// another anchor steals it. So blur/hide/activation disarm hover and drop
// a visible tip; only a real pointer move re-arms. Focus tips are gated
// separately below (:focus-visible — keyboard-driven focus only).
if (
  typeof window !== "undefined" &&
  !(window as unknown as Record<string, unknown>).__songstressTipWired
) {
  (window as unknown as Record<string, unknown>).__songstressTipWired = true;
  window.addEventListener("scroll", hide, true);
  window.addEventListener("keydown", (e) => {
    if (e.key === "Escape") hide();
  });
  window.addEventListener("pointerdown", hide, true);
  window.addEventListener("blur", () => {
    seenMove = false;
    hide();
  });
  // A fresh activation is not a hover: any mouseenter arriving without an
  // intervening pointermove is the remap talking, not the user.
  window.addEventListener("focus", () => {
    seenMove = false;
  });
  window.addEventListener(
    "pointermove",
    () => {
      seenMove = true;
    },
    true,
  );
  document.addEventListener("visibilitychange", () => {
    if (document.hidden) {
      seenMove = false;
      hide();
    }
  });
}

function hide() {
  clearTimeout(timer);
  if (current) current.removeAttribute("aria-describedby");
  current = null;
  if (tip) {
    tip.classList.remove("show");
    tip.hidden = true;
  }
}

function show(node: HTMLElement) {
  const text = node.dataset.tipText;
  if (!text) return;
  const t = ensure();
  current = node;
  t.textContent = text;
  t.hidden = false;
  t.classList.remove("show");
  const r = node.getBoundingClientRect();
  const w = t.offsetWidth;
  const h = t.offsetHeight;
  const left = Math.max(
    8,
    Math.min(window.innerWidth - w - 8, Math.round(r.left + r.width / 2 - w / 2)),
  );
  let top = Math.round(r.top - h - 8);
  // 80px, not 8: in-window chrome (sidebar traffic lights live ~40px above
  // the search row) is not the viewport edge, so the generic flip must clear
  // it too. Anchors this high always have room below; bottom-dwellers like
  // the playbar never come near this branch.
  if (top < 80) top = Math.round(r.bottom + 8);
  t.style.left = `${left}px`;
  t.style.top = `${top}px`;
  node.setAttribute("aria-describedby", "app-tip");
  // A frame between unhide and .show or the opacity transition never runs.
  requestAnimationFrame(() => {
    if (current === node) t.classList.add("show");
  });
}

function arm(node: HTMLElement, ms: number) {
  clearTimeout(timer);
  timer = setTimeout(() => show(node), ms);
}

/** `use:tooltip={text}` — replaces `title={text}`. Null/undefined disables. */
export function tooltip(node: HTMLElement, text: string | null | undefined) {
  const original = node.getAttribute("title");
  const onEnter = () => {
    if (!seenMove) return;
    arm(node, 350);
  };
  const onFocus = () => {
    // Stale-focus guard: the other half of the stuck seek tip. Focus that
    // wasn't keyboard-driven (a parked slider focus refired by the restore)
    // needs no tip — mouse users get the hover one.
    if (node.matches(":focus-visible")) arm(node, 150);
  };
  const onLeave = () => {
    if (current !== node) clearTimeout(timer);
    else hide();
  };
  node.addEventListener("mouseenter", onEnter);
  node.addEventListener("focus", onFocus);
  node.addEventListener("mouseleave", onLeave);
  node.addEventListener("blur", onLeave);

  function set(next: string | null | undefined) {
    if (next) {
      node.dataset.tipText = next;
      node.removeAttribute("title");
    } else {
      delete node.dataset.tipText;
      if (current === node) hide();
      else clearTimeout(timer);
      // Svelte drops the attribute for undefined — put back whatever was
      // there at mount so unmount/HMR leaves no trace either way.
      if (original !== null) node.setAttribute("title", original);
    }
  }
  set(text);

  return {
    update: set,
    destroy() {
      node.removeEventListener("mouseenter", onEnter);
      node.removeEventListener("focus", onFocus);
      node.removeEventListener("mouseleave", onLeave);
      node.removeEventListener("blur", onLeave);
      if (current === node) hide();
      if (original !== null) node.setAttribute("title", original);
      else node.removeAttribute("title");
      delete node.dataset.tipText;
    },
  };
}
