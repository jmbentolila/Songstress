// Seamless marquee action, ONLY on overflow — shared by the playbar
// title/artist lines and the sidebar artist rows.
//
// The viewport (node) keeps its ellipsis rules as the no-JS/reduced-motion
// fallback; when the single copy is wider than the viewport a hidden twin is
// appended and the inner strip loops by exactly one copy + gap, so the wrap
// point is invisible. CSS animation (off the main thread, linear = constant
// speed), duration scaled to distance at a device-pixel-aligned speed — whole
// device pixels per frame read smoother than fractional ones, which shimmer.
// Pausing is animation-play-state in the stylesheet — the animation itself is
// never torn down mid-mode, so resume continues mid-flight.
//
// Modes: default measures on mount (playbar: always sliding, hover pauses via
// CSS); `hover: true` measures on mouseenter and tears down on mouseleave, so
// the row rests at plain ellipsis until hovered (sidebar artist rows). Every
// loop ends with a dwell at the head (the keyframes hold the last quarter of
// the cycle at the loop point) before the next rotation starts.
export function marquee(
  node: HTMLElement,
  opts: string | { key: string; hover?: boolean },
) {
  const MQ_GAP = 48; // px between copy and twin — mirrors [data-clone] margin
  // 37.5px/s = exactly 1 DEVICE pixel per frame on a 60Hz panel at scale 1.6
  // (his setup): fractional device steps shimmer, whole ones don't. If his
  // panel isn't 60Hz this won't help — then we switch technique (fade-paging).
  const SPEED = 37.5; // px/s — ~1px/frame at 60fps (see above)
  let ro: ResizeObserver | null = null;
  let raf = 0;
  let hovering = false;
  const reduce = matchMedia("(prefers-reduced-motion: reduce)");
  const hoverMode = typeof opts !== "string" && opts.hover === true;
  function teardown() {
    const inner = node.querySelector<HTMLElement>(".mq-in");
    inner?.querySelectorAll("[data-clone]").forEach((c) => c.remove());
    node.classList.remove("is-over");
    inner?.style.removeProperty("--mq-to");
    inner?.style.removeProperty("--mq-dur");
  }
  function run() {
    const inner = node.querySelector<HTMLElement>(".mq-in");
    if (!inner) return;
    // Strip last run's twin (text may have changed) and stop the loop so
    // the single-copy measure is honest. Forced reflow restarts the
    // animation below — without it re-adding the class is a no-op.
    teardown();
    void node.offsetWidth;
    if (reduce.matches) return;
    const single = inner.scrollWidth;
    const view = node.clientWidth;
    if (single <= view + 2) return;
    const twin = document.createElement("span");
    twin.setAttribute("data-clone", "");
    twin.setAttribute("aria-hidden", "true");
    twin.textContent = inner.textContent;
    twin.style.marginLeft = `${MQ_GAP}px`;
    inner.appendChild(twin);
    const dist = single + MQ_GAP;
    inner.style.setProperty("--mq-to", `${-dist}px`);
    // Dwell math: the keyframes spend the last quarter of each cycle holding
    // the loop point, so duration is travel/0.75 — the travel portion keeps
    // exactly SPEED and the rest scales with the journey (longer slide,
    // longer rest).
    const travel = dist / SPEED;
    const dur = Math.min(24, Math.max(4, travel / 0.75));
    inner.style.setProperty("--mq-dur", `${dur.toFixed(2)}s`);
    node.classList.add("is-over");
  }
  function schedule() {
    if (hoverMode && !hovering) return;
    cancelAnimationFrame(raf);
    raf = requestAnimationFrame(run);
  }
  const onEnter = () => {
    hovering = true;
    schedule();
  };
  const onLeave = () => {
    hovering = false;
    cancelAnimationFrame(raf);
    teardown();
  };
  if (hoverMode) {
    node.addEventListener("mouseenter", onEnter);
    node.addEventListener("mouseleave", onLeave);
  } else {
    schedule();
  }
  ro = new ResizeObserver(schedule);
  ro.observe(node);
  const onReduce = () => (hoverMode && !hovering ? teardown() : schedule());
  reduce.addEventListener?.("change", onReduce);
  return {
    update() {
      schedule();
    },
    destroy() {
      cancelAnimationFrame(raf);
      ro?.disconnect();
      reduce.removeEventListener?.("change", onReduce);
      node.removeEventListener("mouseenter", onEnter);
      node.removeEventListener("mouseleave", onLeave);
    },
  };
}
