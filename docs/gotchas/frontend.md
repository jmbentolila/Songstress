# Frontend gotchas (Svelte 5, CSS, motion)

The measured detail behind the frontend rules in `AGENTS.md`: hot-reload rot, scoped CSS, hit-testing, custom-property animation cost, modal confirmation placement, and the four Svelte 5 traps.

Moved out of `AGENTS.md` on 2026-10-24 to keep the injected file under
the 20,000-char context cap. Rules stay in `AGENTS.md`; this file carries the
measurements and the reproduction.

- **Svelte 5 runes stores** live in `*.svelte.ts`. `$effect`/`$derived` rules apply.
- **HMR rots.** After a long series of hot edits the webview degrades: stale
  `bind:clientWidth`, dropped component CSS, phantom layouts. Non-atomic writes to
  watched files (sed/python rewrite of app.css) make it worse. Fix = cold restart:
  `systemctl --user restart songstress-dev`. Don't debug phantoms before restarting.
- **A COLD start can also drop component CSS — and a reload will not fix it.**
  The webview can request `X.svelte?svelte&type=style&lang.css` before the plugin
  has compiled its parent; the plugin has no metadata for that id, Vite falls
  through to the static-file handler (which ignores the query) and the "CSS" is
  served as the component's RAW SOURCE, so the style tag never injects. Result:
  the shell renders layout-less (full-width grid, playbar stacked vertically)
  while components requested a moment later are fine — it reads as "only one
  pane broke". Diagnose: `curl -s 'localhost:1420/src/App.svelte?svelte&type=style&lang.css' | head -c 200`
  — raw `<script>` means uncached. Fix: `server.warmup.clientFiles` (added to
  vite.config.js); `touch`ing the file is the manual version. `location.reload()`
  does NOTHING here — the id stays uncached, so don't waste a restart cycle on it.
  **The global sheet rots the same way, silently:** `src/app.css` can keep serving a
  module whose `__vite__css` is the EMPTY string (2026-09-02, after a long edit
  series). Nothing 404s and the components still look styled, but every theme token
  reads empty — the tell is `getComputedStyle(document.documentElement)
  .getPropertyValue('--gap')` coming back `""`, or a `.sk`/`.glass` probe returning
  nothing. `touch src/app.css` re-emits it in one second; a screenshot would take a
  restart cycle and show you a page that merely looks a bit flat.
- **Component CSS can be emitted unscoped.** TitleBar once emitted a global sheet,
  leaking `.title { text-align: center }` into every component. (TitleBar was
  deleted in Step 8 — its `tb-*` traffic lights now live in Sidebar; the
  namespace lesson stands.) If styles leak again, namespace the classes — don't
  fight the compiler. Audit served sheets:
  `curl 'http://localhost:1420/src/components/X.svelte?svelte&type=style&lang.css'`
  and check every selector carries a `.s-…` hash.
- **Confirmations inside a modal drop from the top edge of the surface that
  raised them** — the modal normally, the LIGHTBOX when the expanded artwork
  view is what asked (rendered into whichever is innermost; a box on the panel
  behind the image reads as a bug) — as
  notification boxes (`.glass`, safe action first, destructive in caution), and
  the modal's own surface DIMS under them — the window's scrim wash
  (`rgba(0,0,0,.35)`) and timing, scoped to the panel, press-to-cancel. Never a
  modal on top of a modal, never a native dialog. Shared entrance/dim:
  `notifyDrop` + `notifyVeil` (src/lib/notifyDrop.ts). Two traps, both measured:
  `position: fixed` is what lands on the panel (`.te-body` is
  `position: relative`, so `absolute` anchors 64px down at the artwork section),
  and Escape must be handled in CAPTURE with `stopImmediatePropagation` —
  the modal's own Escape listener is on `window` too, and `stopPropagation`
  cannot quiet a listener on the same target. Example: the artwork delete
  confirmation in ArtSelector.
- **Animating an INHERITED registered custom property is a document-wide tax
  (2026-09-28).** The playbar-style toggle eased `--playbar-h` (a `@property` of
  syntax `<length>`) and ran at **~36ms/frame (~27fps)** because every frame
  invalidated computed styles across the whole tree; pinning individual layers
  did NOT help — the cost was the property itself. Fix: change the token
  INSTANTLY and let each consumer transition its OWN concrete property
  (`height` / `bottom`) on a shared duration token (`--playbar-dur`, 0ms under
  reduced motion) — same visual, ~50–60fps. Reach for `@property` animation only
  when nothing inherits the property.
- **A transparent absolutely-positioned layer still HIT-TESTS (2026-09-28).** The
  waveform and the straight line are stacked variants; the inactive `.wave` at
  `opacity: 0` swallowed every pointer event, so the line could not be dragged or
  clicked. The inactive variant needs `pointer-events: none` — `opacity: 0` is
  not `display: none`.
- **Switch animation contract**: album switch = instant content swap + 160 ms
  fade-in of the NEW album *(overridden 2026-10-09 for the expanded-panel
  surface only: the same-row swap reveal runs 320ms fade / 380ms travel with
  the 0/+80ms two-beat stagger — the 160ms contract still holds everywhere
  else)*; height changes in ONE step (no per-frame relayout).
  The outgoing album must NEVER render at the new row (user calls it "ghost").
  Expand/collapse keeps the eased `grid-template-rows` animation *(stale since
  0.16.14, corrected 2026-10-09: the panel drives px height inline per phase
  with settle-to-auto at rest — grid-template-rows left the tree rewrites ago
  and the sentence survived by inertia)*.
- **`overflow: hidden` is still a scroll port.** Programmatic `.focus()` on an
  element inside a layer that is `translateX(100%)` SCROLLS the clipper
  (`.stack.scrollLeft` → ~202) and nothing ever scrolls it back: every layer
  shifts sideways at once, so pane titles print over each other and the deeper
  layer's controls leak into view. The 4-layer stack therefore uses
  `overflow: clip` AND `focus({ preventScroll: true })` (`focusFirst`) — the
  layer is animating into place anyway, so it never needs the scroll. If the
  settings panes ever look "printed twice", read `.stack.scrollLeft` first.

## Svelte 5 gotchas learned the hard way

- **A `${}` in a plain attribute is not an interpolation.** Svelte reads the `$` as
  text and only the braces as the tag, so `style:transform="scaleX(${pct/100})"`
  emits the declaration `scaleX($0.4)` — invalid CSS, dropped with no error, and
  `el.style` ends up empty so the stylesheet's `scaleX(0)` wins. Interpolated
  attribute values need a template literal: ``style:transform={`scaleX(${pct/100})`}``.
  The tell (this is how EmptyState's first-scan bar sat still for whole scans):
  the counter *text* next to it is right, `style` is `""`, console says nothing.
- **A `$derived` that early-returns on a plain `let` never wakes up.** Svelte 5
  tracks only what a derived body ACTUALLY reads before it returns. A blast-
  radius derived opened with `if (!snapshot) return null` where `snapshot` was
  a plain script `let` — first pass (before the load finished) early-returned,
  so `touched`/`census` were never registered as dependencies, and the derived
  stayed null through every later edit. Symptom: every input is right, the
  value is `null`, no error. The gate variable of a derived must be `$state`
  (here: `snapshotJson = $state("")`), full stop.
- **A class forwarded into a child component is unscoped.** `<SurfaceClose
  class="ab-close" />` puts the class on an element compiled in another file, so
  the parent's scoped rule `.ab-close { … }` is reported as *unused* and does
  nothing. Use `:global(.ab-close)` — and give it one more class of specificity
  (`:global(.ab .ab-close)`) when the child sets the same property, because the
  child's stylesheet loads later and an equal-specificity tie goes to it.
- **A parent's scoped CSS does not reach a child component's root element.** Two
  bites: About needed `:global(.ab .ab-close)` to position `<SurfaceClose>`, and a
  `.group > * + * { margin-top }` rule silently gave `<Toggle>` no separation
  (0px, no warning). Put box/spacing rules on the **container** (`gap` works
  whoever compiled the child) and use `:global()` only for per-element offsets.
- **Child effects run before parent effects.** Never capture `document.activeElement`
  in a parent `$effect` to remember "who opened this" — by then the child's
  autofocus has already moved focus inside. Record the trigger at open time
  (`openAbout(e.currentTarget)`), and read it back on unmount.
