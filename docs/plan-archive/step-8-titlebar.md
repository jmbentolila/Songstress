# Step 8 (plan archive)

Moved out of `PLAN.md` on 2026-10-24 so the plan file stays readable in one pass.
This is the record, not the state: current state lives in `PLAN.md`, and this
file is the evidence behind it. Sections are kept verbatim, in their original
order. The index of every section is `README.md` next to this file.

## Step 8 — Titlebar elimination: chrome moves into the sidebar 🔶 (2026-08-27)
DECIDED (2026-08-27, supersedes the Step 3 "Global Menu XOR in-titlebar bar"
row for the in-app surface): the 44px titlebar is gone. New arrangement:
- Traffic lights (KWin-palette Klassy dots) → sidebar header, TOP-LEFT;
  gear button TOP-RIGHT, lights and gear side by side.
- App title ("Songstress") removed from the sidebar.
- ONE search field ("Search library...") in the sidebar: filters the artist
  list AND drives the grid's Songs/Albums sections (old `ui.filter` +
  `ui.mediaFilter` merged into `ui.search`; clicking an artist row clears
  it). Titlebar's media search deleted.
- Menu: gear opens the app menu (Playback/Library/View/Help + Appearance
  will live inside it). **Gear popover is a placeholder for now** — the real
  menu UI is the next task. Plasma Global Menu stays the live menu and is
  unaffected. The old in-titlebar menubar fallback rendering is gone.
- Window drag regions: the sidebar header + a 20px strip over the grid's
  top padding (`.drag-strip` in App.svelte) — covers the grid's 20px
  content padding; scrolled content can't be grabbed in its top 20px.

**Landed (2026-08-27)**: `TitleBar.svelte` deleted; its traffic-light /
chromeVars/layout code moved into `Sidebar.svelte` (classes keep the
`tb-*` namespace, styles re-scoped there); `Sidebar` header rebuilt
(lights left, gear right, `data-tauri-drag-region` on the header, gear
toggles a glass placeholder popover, closes on outside pointerdown /
Escape); `App.svelte` loses `<TitleBar />`, gains `.drag-strip` (z-5 over
the grid, below sidebar/playbar z-10); `ui.svelte.ts` merges `filter` +
`mediaFilter` → `search`; `AlbumGrid` reads `ui.search`; `--titlebar-h`
token dropped from `app.css`. Gates: check 0/0, vitest 53, build OK;
cold-restart + screenshot verified (lights, gear, search, full-height
grid).

**Menu stack (Step 8b, 2026-08-27)**: the gear's placeholder popover is
replaced by an iOS-Settings-style stack in the sidebar (apple-design skill:
transform-only slide, standard curve, parallax recede). Three absolute
layers in `Sidebar .stack`: home (search + artists) → root ("Menu" navrow +
Appearance/Playback/Library/View/Help rows with chevrons) → detail (navrow
with accent back-chevron + title; Appearance pane = the old gear-popover
settings; other panes render the live Rust `menu.menus` model via
`menu_activate`). Gear (icon flips to ✕) opens/closes; back-chevron or
Escape pops one level. State: session-only `ui.menuOpen` / `ui.menuDetail`
(`"appearance"` or a menu id). Root recedes -30% + 0.85 opacity behind an
open detail. Gates: check 0/0, vitest 53; root + closed states verified by
screenshot (detail-pane visuals + the push/pop motion are for the user's
hands). Swipe-back gesture deferred (CSS transitions aren't grabbable;
would need a spring-driven drag layer).

**Menu iteration (user-feedback driven, same day)**:
- parallax recede -30% → full push: on a flat glass sidebar the peeking
  root items read as clutter ("I can still see Appearance").
- root divider removed; left back-chevron added to the root title; right
  chevrons removed from rows (user spec).
- ✕ semantics: one-level pop → ALL levels; then two-phase pop (root
  centered intermediate) → single conveyor: home in from left, root out
  LEFT in parallel (same speed, no crossing), detail out right. Root
  teleports -200%→+100% off-screen (one no-anim frame) so re-open always
  enters from the right. Close timer 340ms.
- **BUG (found by user): home layer had no pointer-events grant — the
  artist list was click-dead.** `.layer` defaults to none; grant it to
  every on-screen layer (`.layer:not(.off)`), not just menu `.active`s.
- Polish batch (emil-design-eng audit): 400→320ms layers, `:active`
  press wash on rows/chevrons/gear, gear↔✕ crossfade+90° rotate 160ms,
  back-chevron hit target 18px→28px, explicit `:focus-visible` accent
  rings, menu rows 38→40px (deliberate cadence), dead opacity transition
  removed. Stagger-on-entry audited but deferred (could read busy at 236px).

**Critique pass (impeccable, 2026-08-27): sidebar scored 25/36 Acceptable**
(first run, snapshot in `.impeccable/critique/`). Fixed in sequence:
- P1 search dead end: "All Artists" row is now ALWAYS visible (it's a
  control, not a result) + clear (×) button in the field when non-empty
  (old titlebar pattern; × glyph absolute-centered — WebKitGTK button
  place-items drift, same fix as the traffic lights).
- P2 diacritics: artist filter now uses `fold()` like the grid ("bjork"
  finds Björk in both surfaces).
- P2 a11y: search input `aria-label="Search library"`.
- P2 gear IA: user chose option A + rename — the root pane is titled
  "Settings" (not "Menu"); Appearance stays its first row; aria-labels
  follow ("Open settings"/"Back to settings"); DESIGN.md updated.
- P3 (landed): `.backchev` 6px→7px (on-scale); sidecar colorMeta gained
  `traffic-light-border` (rgba(0,0,0,0.18), system-mirror chrome) and the
  four `picker-spectrum-*` conic colors (functional rainbow) as documented
  exceptions — detect.mjs on Sidebar is now clean (exit 0).
- P3 (landed, round 2): keyboard accelerators — `/` focuses the library
  search (inert while typing, while the Settings stack is open, or with
  modifier keys), `s` toggles the Settings stack, Escape in the search
  field clears the query then blurs (old titlebar behavior, preserved
  through the merge). Re-critique: 25 → 33/36 (Good), detector 0 findings.
- P3 (landed, round 3 — the re-critique's three items): accelerator hints
  as `title` tooltips (search input: "/ to focus — Escape clears"; gear:
  "Settings — press s to toggle"), "No artists match." upgraded from inert
  caption to a clear-search action button (`.empty` voice), and a transient
  11px dim "Updating library…" note under the field while `scanner.running`
  with a populated library (aria-live polite). Sidebar re-critique: 33/36
  (Good), detector 0 findings.
- **PlayBar critique (2026-08-27, same evening)**: 30/40 (Good). Empty-state
  copy fixed ("Pick a track"), then the critique ran. Findings carried into
  Step 8c below — NOT yet fixed.
- **RPM rebuild + install (user, 2026-08-27 20:47)**: rebuilt after the final
  sidebar (20:29) and playbar (18:41) edits; user ran the dnf lines.
  Step 8 complete.

---

### Step 8c — PlayBar critique fixes — ✅ (from the 2026-08-27 critique, 30/40; re-critiqued 35/40)

Snapshot: `.impeccable/critique/2026-08-28T00-36-27Z__src-components-playbar-svelte.md`.
Work top-down, one gate pass after each:
1. ✅ **P2 popover mutual exclusion** (2026-08-29): EQ and play-next queue popovers share
   the same anchor (`right:14px; bottom:calc(100%+10px)`) and toggle
   independently → they overlap. Opening one must close the other.
2. ✅ **P2 standard popover dismissal** (2026-08-29): Escape + outside-click
   close the popovers — the ContextMenu idiom (`svelte:window` pointerdown +
   `el.contains`). GOTCHA: the toggle button's own pointerdown must be
   exempted or the popover becomes open-only (pointerdown closes, then the
   click re-toggles it back open). Popovers now carry `role=dialog` +
   aria-label.
3. ✅ **P2 volume mute target** (2026-08-29): ~19px → 28px box (invisible at
   rest; same bar as the sidebar back-chevron).
4. ✅ **P2 in-app keyboard transport** (2026-08-29, shaped per impeccable
   `shape` + apple-design): Space = play/pause, ←/→ = seek ±5s. NO ↑/↓
   volume (restraint: critique's own scope; volume has slider + media keys).
   Inert rule: acts only when focus is on the body — any interactive target
   (input, select, textarea, button, a, [contenteditable]) keeps native key
   behavior; Space ignores auto-repeat, arrows repeat on purpose (like
   dragging the seekbar). Advertised via `title` tooltips ("Play — Space",
   "Seek — ←/→ = ±5s"), same hover-only micro-help voice as the sidebar
   hints. No collision with `/`/`s` (different keys; those stay inert in
   inputs — same rule shape).
5. ✅ **P3 document + drift** (2026-08-29): DESIGN.md gained the
   "Instrument" type tier (400, 8–9.5px tabular — EQ dB/freq readouts and
   mode-button badges only; smallest tier, nothing else may use it) in the
   YAML frontmatter AND the prose hierarchy; EQ preset select radius
   6px → 8px (the control rung — 6px was the last off-ladder radius in the
   popovers).

After the items land: re-critique the playbar (DONE 2026-08-29: 30 → 35/40,
Good 87.5%, detector 0 findings/exit 0; snapshot in .impeccable/), then
rebuild the RPM (user installs). The three surviving P3s all landed same
day:
6. ✅ **P3 popover materialize entrance** (2026-08-29, apple-design
   "materialize, don't just fade"): 140ms scale 0.98→1 + opacity,
   `cubic-bezier(0.22, 1, 0.36, 1)`, `transform-origin: 100% 100%` (popovers
   sit above their trigger, so they grow from the shared bottom-right
   corner); transform+opacity only; the global reduced-motion switch
   collapses it. Entry only (unmount = instant exit, P3 scope).
7. ✅ **P3 clear-queue-all** (2026-08-29): "Clear" text button in the queue
   popover header when non-empty; new `playback_queue_clear` command
   (mpv.rs `queue_clear` mirrors `queue_remove`: clear → rebuild tail while
   playing → `queue-changed` mirror).
8. ✅ **P3 "ALL" badge jargon** (2026-08-29): "ALL" → "All" (plain word, not
   shouty jargon; the hover title carries the full explanation).
Detector config: `design-system-font-size` ignore-values 8px + 8.5px =
documented Instrument-tier exceptions (.impeccable/config.json, same route
as the sidebar colorMeta exceptions); the two 16px close glyphs were fixed
for real (→ 14px body, on-ramp) rather than ignored. Gates: check 0/0,
vitest 53, cargo 65, build OK, detector exit 0, screenshot clean.

**Icon consistency pass (impeccable polish, 2026-08-29)**: audited the
playbar's three glyph sources (pixel-cropped evidence) and landed:
- P1 track-skip icons carried a DEAD bar subpath (fill-only path, zero
  area — the bar never rendered; the triangle-only look was accidental).
  Now intentional: bare triangle = track step, bar+triangle = album jump
  (weight encodes jump size, doesn't rely on tooltips); code + comment
  match the pixels.
- P2 popover close × and queue-row × were FONT glyphs (a third family
  beside the SVGs) → utility-family stroked X (24-box, stroke 2, 14px).
- P2 volume waves 1.2 / mute cross 1.3 → 1.6 (the media-family stroke
  weight; 1.6 at 15px render = the Lucide-equivalent optical weight).
- P3 transport SVGs gained aria-hidden (a11y parity with mode/volume).
- DESIGN.md gained an **Iconography** subsection: the two-family system
  (media = 16-box fills + 1.6 strokes; utility = Lucide 24-box stroke 2)
  and the no-typographic-glyph rule.
Gates: check 0/0, vitest 53, build OK, detector exit 0, cropped screenshot
verified (incl. a live red-gradient-album playbar state).

**Grid critique pass (impeccable, 2026-08-29, commit 18d141d)**: album
grid + expansion panel critiqued as ONE surface (the expanded panel is a
grid row, not an overlay). First run 32/40 (Good); all five findings
landed same day:
- P2 selection without consequence (user-shaped as "b" + Enter): the
  row WAS a button that selected and then did nothing. Now: single
  click selects (wash + tooltip "Click again to play (or press
  Enter)"), a SECOND click on the selected row plays it, and Enter
  plays the selection from anywhere. The two paths can't double-fire
  (focused row's native Enter = click = second-click path; the window
  handler ignores interactive targets) and both stand down for inputs,
  the tag editor, and open context menus; a selected MISSING track
  locates on Enter instead. Selection clears when playback starts.
  A visible ▶-on-selected was rejected: button-in-button is invalid
  HTML and the per-row layout pop was ugly.
- P2 press convention skipped by the app's largest pressable families:
  tiles + track rows now `cursor: pointer` and take the documented
  `--active` `:active` wash (tile = a `::after` overlay, NEVER a
  filter/transform on the cover — WebKit layer churn = blank-cover
  flash).
- P2 inert grid empty state → clear-search action, same voice as the
  sidebar's `.empty`.
- P3 `.edit-album` 26 → 28px (PRODUCT.md hit-target bar); play-all
  gained `title="Play"`.
- P3 a11y: alert-triangle SVGs + the ▶/❚❚ status glyphs are
  `aria-hidden` with an sr-only "Now playing" sibling (new `.sr-only`
  utility in app.css).
Detector: the panel's `transition: height` is the documented WebKit
dual-clock fix — registered as a file-scoped exception
(layout-transition=* for ExpandedPanel.svelte); detector back to
exit 0. Snapshot: .impeccable/critique/2026-08-29T19-41-14Z__src-components-albumgrid-svelte.md.
Gates: check 0/0, vitest 53, build OK.

**Expansion choreography + the 20px-line reframe (2026-08-29)**:
the grid↔panel expansion surface, redesigned from user critique. Files:
AlbumGrid.svelte, ExpandedPanel.svelte.
- **Tile press ring** (user feedback "the wash should use the selected
  border"): the `::after` overlay wash is gone; `:active` lights the
  cover's EXISTING outline to the accent (`outline-color: var(--accent)`) —
  the outline is the album's state language (dim = hover, accent =
  playing / expanded / pressed). `.tile` gained 8px bottom padding
  (caption breathing).
- **Expansion state machine** (panel): the ad-hoc rAF chain became an
  explicit machine — `displayId` (lagging content; deliberately NOT
  tracking the `album` prop), `generation` counter for interruption,
  `move()` pins `auto`→px before every transition (a transition cannot
  interpolate from `auto` — an unpinned shrink jumps to 0 instead of
  animating), `settleToAuto()` drops the inline height in one no-jank
  reflow, `grow()` measures the natural height. Constants: grow 360 /
  collapse 280ms, same-row swap = 160ms content fade, no height
  motion. `cubic-bezier(0.22,1,0.36,1)` throughout.
- **Host-relocation architecture** (grid): the grid owns WHERE the panel
  row lives (`songHostId` / `albumHostId`), the panel owns what it shows
  (`album` prop = host row, `targetId` prop = the store's expanded album
  WHEN IT IS THIS ALBUM, else null = this instance closes). Same-row
  switch = host flips now, in-place content swap + fade (instance
  persists — the row key's two-value token does NOT flip). Cross-row
  switch = collapse-then-expand (see decisions log 2026-08-30): host
  lags the store through the outgoing panel's plain collapse, and its
  `onClosed` consumes the pending open — token flip (fresh mount at the
  destination), host flip, `await tick()` (row list flushed before the
  glide measures), glide + grow. A safety effect re-anchors a stale host
  (its row vanished from a search/artist change mid-collapse) to the
  store's target and clears the pending open (the unmount can't fire
  onClosed) — without it every later switch deadlocks.
- **reveal() auto-scroll: removed** — no app-initiated viewport movement
  in the expansion machinery; replaced by the reframe below.
- **The 20px-line reframe** (user-designed invariant — see decisions
  log): every open / collapse / switch ends with the focused row at the
  20px line, or the final layout's bottom edge (the browser's scrollTop
  clamp does the bottom limit for free — e.g. the last row of a
  200-track album). The travel is an ANCHOR, not a predicted tween: each
  frame it nudges scrollTop by however far the row's LIVE viewport
  position is from the desired one (ease-out cubic glide to the line,
  240–380ms — viewport-bounded, since only on-screen rows are clickable —
  then a hold). The live read makes it immune to mid-flight layout
  change: the panel grows BELOW the pinned row while the glide runs, so
  the row's motion is one continuous glide, then still. The first
  iteration aimed a predicted tween at the final layout and produced a
  visible stall→snap (the row physically can't reach the final position
  while the old slot is still open) — that's why the anchor exists. The
  anchor contract: it only ever tracks rows whose layout ABOVE them is
  stable (panels grow and close below their row); anything that animates
  layout above the row fights its one-paint-late scrollTop on this
  WebKitGTK (the reason cross-row switches collapse-then-expand — see
  decisions log 2026-08-30). Cancellation: wheel / touch / keys take the
  view over (height animations finish on their own); reduced motion
  disables the reframe entirely (zero viewport movement — the stricter
  a11y reading); ≤4px deltas don't travel.
  Verified via instrumented traces (Vite middleware sink →
  /tmp/songstress-diag.log, all instrumentation stripped after use).
Gates: check 0/0, vitest 53, build OK.

**Remaining: RPM rebuild (user installs) — deliberate pause 2026-08-29,
other work takes the queue.**

---

