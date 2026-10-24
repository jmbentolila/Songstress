# Motion audit (plan archive)

Moved out of `PLAN.md` on 2026-10-24 so the plan file stays readable in one pass.
This is the record, not the state: current state lives in `PLAN.md`, and this
file is the evidence behind it. Sections are kept verbatim, in their original
order. The index of every section is `README.md` next to this file.

## Motion audit — token consolidation + polish ✅ (2026-08-30, improve-animations skill)

Full audit of the motion surface (8 categories). Verdict: **the core
choreography is right** — no HIGH/MEDIUM feel-breaking findings; the
360/280ms panel moves, the 240–380ms anchor glide, the gap-synced slot
margins, the 320ms drawer stack and the instant high-frequency hovers
are all correct (and user-verified — untouched). What landed:
- **Motion tokens** (app.css `:root`): `--ease-out:
  cubic-bezier(0.22, 1, 0.36, 1)` (the user-verified panel curve — now
  referenced, not re-typed) and `--ease-drawer: cubic-bezier(0.32, 0.72,
  0, 1)`. All 7 hand-typed curve sites replaced, incl. ExpandedPanel's
  JS `CURVE` const (var() resolves fine in inline styles — verified by
  frame trace). Duration spellings normalized to ms.
- **PlayBar gradient fade** 400ms `ease` → 280ms var(--ease-out) (UI
  budget is <300ms). *(Standing exception extended 2026-10-09: the panel
  surface already spends past the budget by design — 500ms grow cap since
  0.16.14 — and the retune extends the exception to the 380ms content
  travel. Deliberate disclosure motion, not high-frequency UI.)*
- **EmptyState progress fill** animates `width` (layout) → full-width
  element + `transform: scaleX(pct/100)` from the left edge, 250ms
  var(--ease-out) — compositor-only per progress tick.
- **Dead `translate` transition on .tile .cover** removed (the hover lift
  it served went away in Step 0c).
- **Finding #4 (pop-in keyframes on the EQ/queue popovers) is VOID —
  not implemented**: both popovers `{#if}`-unmount on close, so a
  re-open is a fresh element for keyframes AND transitions alike; the
  keyframes-restart concern only applies to re-triggered mounted
  elements. The 140ms pop-in (scale 0.98 from the shared edge with the
  trigger) is correct as-is.
- Parked with the modal work: TagEditor entrance (see *Up next*).

## Settings menu tree — sidebar-shaped redesign ✅ (2026-08-30, impeccable critique, user options C/A/A)

Critique of the settings menu tree scored 25/40 (snapshot
`.impeccable/critique/2026-08-30T21-44-56Z__src-components-sidebar-svelte.md`;
degraded single-context run — no sub-agent tool in this harness). Core
finding: the tree was the Rust global menu bar rendered verbatim
(Playback/Library/View/Help + grafted Appearance) — View was a one-item
pane duplicating Appearance's theme button, Help was a one-item pane of a
NO-OP ("About Songstress" did nothing on either surface), and the root
rows had no drill-down affordance. The sidebar now has its own tree
(Global Menu / menu.rs untouched — it stays a menu bar):
- **Root:** Appearance / Playback / Library with trailing › chevrons + a
  dim **About Songstress footer row** at the bottom (a footer, not a nav
  pane) opening the new in-glass **About dialog** (About.svelte, App-level
  like TagEditor: name, version via `getVersion()` so it can't drift from
  the RPM, one-line description; Esc/backdrop/✕ close; focus lands inside
  on open; the sidebar's Escape handler yields to it via `ui.aboutOpen`).
- **Playback pane:** drops its 6 transport rows (the PlayBar owns them) —
  modes + EQ only. **Library pane:** "Save imported music" hides when
  nothing is staged instead of standing as a dead row.
- **ONE row language** (option A): menu rows join the artist-row rhythm
  (`--sidebar-row-size` height, 13px, same label column); the left check
  column is gone — ✓ and › are trailing glyphs in the count column, labels
  align with the pane title. Arrow keys walk the focused layer's buttons
  (menu convention; Tab unchanged; ranges keep native arrow behavior).
- **Glass controls** (option A): the OS-default checkbox (the only
  non-glass control in the app) is a hidden native input driving a 16px
  rounded box — accent fill + check in luminance-aware `--accent-text`
  (a white check on the user's white accent would vanish), 160ms;
  sliders are appearance-none with an inline accent fill gradient sized
  to the value + accent-dot thumb.
- DESIGN.md updated (row language, tree shape, slider/checkbox
  components); the detector's one advisory (checkbox 5px radius off the
  rounded scale) is documented as deliberate there.

## Settings panes — critique pass C: layout + quieter ✅ (2026-08-31, impeccable critique 24/40)

Second critique of the sidebar Settings stack (`src/components/Sidebar.svelte`)
scored **24/40** (`.impeccable/critique/2026-08-31T18-30-27Z__src-components-sidebar-svelte.md`);
user chose the order **C (layout+quieter) → A (clarify) → B (audit)**, scope
"everything". This is pass C.

**Diagnosis:** the Appearance pane did not feel crammed because of density
(content stopped 231px above the pane floor — 40% of the column empty) but
because five heterogeneous controls ran on one uniform 16px cadence with no
group seams: the eye got one block. Cognitive load failed 4 of 8 checklist
items (chunking, grouping, ≤4 options at the accent decision, progressive
disclosure). Detector: 1 advisory (`design-system-radius` on the documented
5px checkbox radius) — false positive; browser overlay: 1 finding (`.app`
clips a positioned child — the intentional window clip).

**Landed:**
- **Uneven rhythm**: `.panebody` gap 30px between groups, `.group` gap 10px
  inside; group seams use DESIGN.md's **Label** tier (11.5/600/+0.09em
  uppercase), which the pane never used. Control labels moved to List tier at
  full `--text` (the old 12px/400 dim was a tier DESIGN.md does not declare).
- **Accent folded one level deeper**: the stack gained a 4th layer (`.layer
  sub`, z-index 3). Appearance now shows an `Accent · <name> · ›` summary row;
  the 12-dot grid lives in the sub-pane as a 6-column grid (11 presets + the
  custom picker = two even rows, was a ragged 7 + 5). `ui.menuSub` is the
  state; Escape pops sub → pane → root → home; `openDetail` clears it.
- **Full push, then teleport**: a pushed sub-pane takes the detail layer a
  FULL width left (`translateX(-100%)`, matching root — the user rejected
  parallax peeking on 2026-08-27 and a flat glass column has no room for it).
  On a full pop the receded detail would sweep the column on its way back to
  its +100% entry slot, so `.layer.detail.no-anim` reuses the root's one-frame
  transition suppressor. **Frame-trace verified** (`devctl sample` on
  `.layer.detail` x during the close): 22 frames at -235, then a single frame
  at +235 — no intermediate values, so no sweep.
- **Per-pane footers** (user-approved content): Appearance → `Reset appearance`
  (tile 180 / rows 36 / theme system / stock accent / gradient off; verified
  live: `--tile-size` 220px → 180px on click), Playback → `Reset equalizer`
  (Flat preset + preamp 0), Library → status line `248 albums · 4434 tracks`
  (+ ` · scanned HH:MM` once a scan has run — `ui.lastScan`, persisted through
  the settings table from `scan-finished`; the clause is omitted rather than
  lying "never scanned" on a library built before the readout existed).
  Footer ACTIONS render at 0.82 of `--text` ≈ 4.6:1 over the worst-case bright
  wallpaper under the 0.7 chrome tier; `--text-dim` stays caption-only there.
- Splitting the shared `.appearance label` rule removed the
  `.toggle { flex-direction: row !important }` hack.

**Verification**: `npm run check` 0 errors · `npm test` 57 green ·
`npm run build` ok · `detect.mjs --scope layout` clean (exit 0) · full
detector unchanged (1 documented advisory) · screenshots of Appearance, the
Accent sub-pane, Playback and Library footers through the devtools bridge.

**Open (passes A and B, in that order):**
- **A — clarify**: 3-segment `Light | Dark | System` (theme is currently a
  ONE-WAY DOOR: `ui.theme` ships as `system`, `cycleTheme()` destroys it, the
  Rust model carries `theme_dark: bool`); slider value readouts in the count
  column (`tabular-nums`, `aria-valuetext`); copy: "Add music folder…" vs
  "Music folders…", "Full Rescan (rebuild)".
- **B — audit**: focus stays on the pushed-out row (measured:
  `document.activeElement` inside an `aria-hidden` layer, 5 focusables → Tab
  walks 4 invisible rows before the visible pane) — hand focus to the incoming
  layer and set `inert` on inactive layers; `.swatch:focus-visible` (none
  exists today); drill glyph at `opacity:.6` on `--text-dim` ≈2.3:1 worst case.
- **Polish**: swatches should render the theme-clamped derived accent (Black in
  dark theme becomes light grey — `accent.ts:96` CLAMP), the custom swatch
  should show the picked color instead of a permanent rainbow.

## Settings panes — critique pass A: clarify ✅ (2026-08-31)

The P1 behavioral findings, all in the Appearance pane + the Library menu copy.

- **Theme: 3-segment `Light | Dark | System`** replaces the cycling
  `Theme: Dark` button. `ui.theme` SHIPS as `"system"` and `cycleTheme()`
  resolves it to a fixed value, so the default was a one-way door with no path
  back from any surface; the segmented control writes the mode directly
  (`ui.theme = t.id`, the $effect persists + repaints as usual). DESIGN.md
  gained the component spec (32px, Glass Line trough, `--hover` segments,
  active = accent wash + 600, hover stays NEUTRAL — the old button spent the
  accent wash on hover, breaking the row language and the One Accent Rule).
  Verified live: `aria-checked` reports `System` (the true stored mode — the
  old button displayed "Dark", i.e. the RESOLVED theme, hiding the state).
  **Global Menu left untouched on purpose** — see *Explicitly deferred* for the
  `theme_dark: bool` → `theme: string` migration it needs.
- **Slider value readouts** (`180px` / `36px`) in the label row's right column,
  Micro tier + `tabular-nums` + `aria-valuetext`, live while dragging. The pane
  whose entire job is choosing a number was choosing it blind, and the Tabular
  Rule had zero instances in it because it showed no digits.
- **Copy (menu.rs labels, ids untouched):** "Add music files…" → **Import music
  files…**, "Add music folder…" → **Import music folder…** (they stage an
  import — Step 2a — and sat three rows above "Music folders…", which edits
  library roots); "Full Rescan (rebuild)" → **Full rescan (all files)**.
  EmptyState's "Add music folder…" deliberately KEEPS its name: it opens the
  roots flow, where that is the correct label.

**Verification**: `npm run check` 0 errors · `npm test` 57 green ·
`cargo test --lib` 65 green (no label assertions touched) · `npm run build` ok ·
`detect.mjs --scope layout` clean · screenshot `/tmp/a1-appearance.png` +
live DOM read of the new labels. Side-effect note: the pass-C "Reset
appearance" click during verification cleared the owner's accent + playbar
gradient; both were restored (accent = White, gradient on) through the UI.

## Settings panes — critique pass B (audit) + polish ✅ (2026-08-31)

**B — the keyboard/ARIA findings:**
- **Focus handoff.** Pushing a layer used to leave focus on the row that
  triggered it — inside a layer that had just become `aria-hidden`, so the ring
  vanished and Tab walked the covered rows first. `focusFirst()` now moves focus
  to the incoming layer's back affordance (pane/sub-pane → `.backchev`, root →
  its first row, close → the gear). Verified live: after popping the accent
  sub-pane, `document.activeElement` is the detail `.backchev`.
- **`inert` on inactive layers** (home when the stack is open, root/detail/sub
  whenever they aren't the active layer). The 5 hidden controls left the tab
  order — the "4 phantom stops" finding is structurally gone, not commented
  around. Verified in the DOM: with the sub-pane open,
  `home/root/detail = inert`, `sub = not`.
- **Radiogroup keys for the segmented control.** `role="radio"` implies arrow
  keys: `segKeydown` (Left/Right/Up/Down/Home/End) + roving `tabindex` (group is
  ONE tab stop, the checked segment carries `tabindex=0`; the group itself is
  `tabindex=-1`, which also silences svelte's `a11y_interactive_supports_focus`).
  Verified: ArrowRight moves aria-checked, focus AND tabindex together.
  Gotcha found: `event.currentTarget` is null inside a `requestAnimationFrame`
  callback, so the first version moved aria-checked while leaving the focus ring
  on the previous segment — grab the node list synchronously.
- **Swatch focus.** `.swatch:focus-visible` + `.swatch.custom:focus-within`
  (the `<input type=color>` is `opacity: 0` inside its label, so the label's
  focus-within is its only ring) → accent outline.
- **Drill glyph contrast.** dropped the `opacity: .6` on top of `--text-dim`
  (≈2.3:1 worst case under the 0.7 chrome tier; a meaningful graphic needs
  3:1) — `--text-dim` alone is ≈3.5:1 — and it now resolves to `--text` on
  `:focus-visible`, not just on hover.
- Deliberate non-fix: `--text-dim` Micro-tier captions (`.fstat`, `.curname`,
  artist counts, track durations) sit ≈3.5:1 over the worst-case wallpaper
  app-WIDE. Fixing it per-pane would fracture the caption tier; it needs one
  global decision (raise the tier's alpha, or darken the chrome tier), which
  touches every surface — logged, not acted on.

**Polish (P3s):**
- **Clamp ring on accent presets.** `accentVariants` clamps accent lightness per
  theme (`accent.ts` CLAMP), so picking **Black** renders `#b3b3b3` in the dark
  theme while the swatch was a black dot. Swatches keep the raw hex (Black and
  White must stay tellable apart) and gain a 2px ring in the color they actually
  produce — only where the lightness shift exceeds 0.12 — plus a tooltip
  ("Black — renders #b3b3b3 in dark theme"). Verified: rings on Black/White
  only.
- **Custom swatch** shows the picked color with a 9px conic corner instead of a
  permanent rainbow.
- The Accent summary chip shows the DERIVED color, not the stored hex.
- Segment radius 6px → 7px (icon tier): a 2px inset inside an 8px trough
  computes to 6, which is off the radius ladder and indistinguishable at this
  size, so no documented exception was needed. Detector back to its single
  known false positive (the documented 5px checkbox radius).

**Verification**: `npm run check` 0/0 · `npm test` 57 · `cargo test --lib` 65 ·
`npm run build` ok · detector: 1 advisory (documented) · live DOM probes above ·
`/tmp/p2-accent.png` shows the Black preset's ring.

**Still open**: critique re-run for the trend line (expect the P1s to clear);
`--text-dim` caption-tier decision; Global Menu theme-state migration (deferred).

## Post-pass regression: the stack painted "twice" (2026-08-31, found by the user)

Cold restart after pass B/polish, and the user reported the sidebar menu broken.
Two INDEPENDENT causes, both now fixed and verified live:

1. **Cold-start dropped CSS** (not my code): the webview requested
   `App/AlbumGrid/PlayBar.svelte?svelte&type=style&lang.css` before the svelte
   plugin had compiled their parents, the plugin had no metadata for the id, Vite
   fell through to the static-file handler — which ignores the query — so those
   "style modules" were served as the component's RAW SOURCE. No style tag, no
   layout: 1280-wide grid rows, `.playbar` block instead of grid, and it did NOT
   self-heal on reload (only on `touch`). Fixed in vite.config.js with
   `server.warmup.clientFiles = ["./src/main.ts"]`; verified by curling all four
   style modules immediately after a cold boot — all return compiled CSS now.
2. **Pass B's focus handoff scrolled the stack sideways** (my bug): `.stack` was
   `overflow: hidden`, which is STILL a scroll port. `focusFirst()` focused the
   incoming layer's back chevron while that layer sat at `translateX(100%)`, so
   the browser scrolled the clipper to `scrollLeft: 202` to reveal it — and
   nothing ever scrolled it back. Every layer shifted left at once (home -437,
   root.active -202, detail +33), which is why "Settings" printed over
   "Appearance" and the sub layer's swatches showed through the root pane.
   Computed styles looked perfect while `getBoundingClientRect()` lied — the
   tell is to read `scrollLeft`, not the transforms. Fixed twice over: `.stack`
   is `overflow: clip` (`hidden` kept as the fallback line) and `focusFirst` /
   the segmented-control focus pass `{ preventScroll: true }`.

**Verified after the fix** (DOM probes, all with `scrollLeft: 0`):
open → home -235 / root.active 0 / detail+sub +235, focus `mrow`;
Appearance → root.receded -235 / detail.active 0, focus "Back to settings";
Accent → detail.receded / sub.active 0, focus "Back to appearance";
pop → detail.active, focus "Back to settings"; pop → root.active, focus `mrow`.
Screenshots `/tmp/x-appr-crop.png` (segmented control on System, 180px/36px
readouts, Accent · White drill) and `/tmp/x-sub-crop.png` (6-across presets,
clamp rings on Black/White) show the panes as designed.
Also caught while verifying: the custom swatch inherited the *preset's* derived
color when the accent was a preset — it now stays the rainbow unless the color
is genuinely custom (`hasCustom ? chipColor : undefined`).

**Screenshot note for future sessions**: this window is 1280x800 CSS and
spectacle captures 2048x1280 — scale is **1.6**, not the dpr the page reports
(2). Dividing device px by 2 made the correctly-rendered sidebar look 40px too
narrow for half of this investigation.

## User-feedback round (2026-08-31, after the pass B/polish report)

Three requests, all shipped:

1. **`System | Light | Dark`.** User's reasoning: reading left to right, the
   leftmost segment is the default option, and System IS the default. Reversed
   the previous order — and it matches `resetAppearance()`'s target and the
   shipped default, so the control now says "default" where the eye lands
   first. `THEMES` order also drives the radiogroup's arrow/Home/End order, so
   Home=System. Verified: `["System*","Light","Dark"]`.
2. **The accent sub-pane looked broken — three separate reasons, all real.**
   - The selected dot drew TWO near-identical concentric rings (white 2px
     outline + grey clamp ring around White). Now `swatchRing()` suppresses the
     clamp ring while that preset is selected; the tooltip still carries the
     promise. Verified computed: Black `box-shadow: 0 0 0 2px #b3b3b3`; White
     `box-shadow: none`, `outline 2px @2px` — one ring each.
   - The custom swatch rendered SMALLER and low in its cell: WebKit draws its
     own ~14px color-well inside `<input type=color>`, and `inset: -4px` on the
     label couldn't contain it. The input is now `appearance: none; inset: 0;
     width/height: 100%` and invisible, so only the 20px conic disc paints.
   - Row rhythm: the outline reaches 4px past a dot and the grid had no row gap,
     so an outlined dot touched the dot above. First fix was `gap: 14px 20px` —
     which the user immediately caught, because a minimum COLUMN gap on fixed
     20px tracks spread with `space-between` overflows the grid's own box
     (6×20 + 5×20 = 220 in a 207px pane) and shoved the last dot into the
     sidebar border (measured: dot right edge 234, content edge 235, border 236
     — with the 4px outline beyond that). Final rule is `gap: 14px 0`: row gap
     only, horizontal spacing left fluid, so both rows sit flush on the pane's
     14px padding (measured 14→221, outline clears the border by 10px) and the
     ragged 11-item row stays column-aligned.
3. **Custom color moved out of PRESETS** into its own `CUSTOM` group, in the
   pane's existing readout-row language:
   `Pick any color · not set / #rrggbb · [20px picker]`. An editor, not a
   twelfth named choice — and as a dot in the grid it had a worse bug than the
   semantics: `hasCustom` is false whenever the accent equals a preset, so it
   silently showed the PRESET's derived color. The whole row is the `<label>`,
   so the text opens the chooser too. Verified end-to-end: pick `#12ab56` → row
   reads `#12ab56`, swatch shows the DERIVED accent (mint in dark theme) with a
   conic corner, the Appearance drill reads `Accent #12ab56`, the back chevron's
   focus ring turns green (live accent), the presets deselect; restore to White
   → drill reads `Accent White`. (`/tmp/n3-2x.png`, `/tmp/n2-sub-2x.png`)

Also while in there: `resetAppearance()` now resets `playbarGradient` — it is
the pane's other visible control and the button says "Reset appearance".

## Playback pane: flat cycling rows → enable + reveal (2026-08-31)

User brief: "the three of them get a checkbox; Repeat and Random enable a
horizontal switch like the theme one; Eq enables both preset and custom fields;
the footer should reset the 3 options, not just the eq."

**Before:** five look-alike rows from the flattened menu model — `Shuffle: Off`,
`Repeat: Off`, `Equalizer: On` ✓, `EQ Preset: Rock` (dead until the equalizer
was on), `Customize Equalizer…` — no seams, values welded into labels, cycling
invisible, one checkmark meaning three different things.

**After** (bespoke, store-driven — like Appearance):
```
☑ Repeat        →  [ Album | Track ]
☑ Shuffle       →  [ Album | Artist | All ]
☑ Equalizer     →  Preset        Rock ›        (sub layer, 10 radio rows)
                   CUSTOM
                   Preamp   0 dB   [slider]
                   31       +5     [slider]    × 10 bands, dbl-click zeroes
                   ── Reset playback
```
- "Off" is owned by the checkbox, so it is NOT a segment: 2-up repeat / 3-up
  shuffle, and the two controls can never disagree. `.seg` now takes
  `--seg-n`; `segKeys(ids, get, set)` is the shared radiogroup key handler
  (Theme uses the same one now).
- `setShuffleStage` / `setRepeatStage` are the single door (state + localStorage
  + SQLite write-through + `playback_set_*`); `cycleShuffle` / `cycleRepeat` are
  now thin wrappers over them, so the **Global Menu keeps cycling and both
  surfaces push the same state**. `setShuffleOn/off` remember the last non-off
  stage in module memory (not persisted, not pushed — a UI memory), so
  un-checking and re-checking restores Track rather than silently picking
  Album. Verified: pick Track → uncheck → recheck → `Track*`.
- Reveals mount with the checkbox, indented 12px (no card — double-translucency
  ban), fade+rise 200ms; the EQ reveal holds the preset drill + preamp + 10
  bands, which is why `.panebody` was already scrollable.
- The 10 presets live in the sub layer (`ui.menuSub = "eq-preset"`, the accent
  picker's mechanism, now branched): `.irow` + `.ok` in the count column,
  `aria-current`, and a `Custom curve — adjust the sliders in Playback` status
  line when `preset === null` (editing a band clears the name, and the pane has
  to be able to say so). Sub `navtitle` and the back button's label are derived
  now (`Back to ${detailTitle}`) instead of hardcoded "appearance".
- Footer renamed and widened in scope: `Reset playback` clears repeat, shuffle,
  eq enable, preset (Flat) and preamp.
- `Customize Equalizer…` left the pane: the sliders are right here, and the
  playbar popover is still one tap away. Transport rows stay hidden (PlayBar
  owns them).

**Verified live** (`/tmp/pb5-2x.png`, `/tmp/pb6-2x.png`): 3 toggles read the real
state (Repeat off, Shuffle off, Equalizer on); enabling shows 1 then 3 reveals;
segments report `["Album*","Track"]` / `["Album*","Artist","All"]` with
`grid-template-columns: 88px 88px` on the 2-up; band values match the Rock
preset (+5/+4…); editing band 0 flips the sub layer to the Custom status line;
tapping Rock re-applies it and restores +5; every state restored afterwards
(Repeat off, Shuffle off, Equalizer on, Rock). `npm run check` 0/0 · 57 tests ·
build ok · detector unchanged.

**Open:** the equalizer's vertical mini-EQ in the playbar popover and these
horizontal rows are now two surfaces for one thing — fine for now, but the
popover should get the same `dB` readout honesty next time it's touched.

## Equalizer popover trimmed + one Toggle component (2026-08-31)

Chose option **b** (popover keeps the curve, pane owns enable + preset), with
the user's addition: the playbar's EQ button must still reach everything.

- **`components/Toggle.svelte`** now owns the app's checkbox (glass box,
  luminance-aware check, accent ring on focus) — 4 call sites in the sidebar
  (Playbar gradient, Repeat, Shuffle, Equalizer) and the popover. The popover's
  bare native checkbox was the last one outside the design system, and copying
  its CSS into a second component would have been the worse answer.
- **The native `<select>` is gone**, replaced by the preset NAME in Micro tier
  (`Rock` / `Custom`). Its dropdown was the toolkit's own chrome — AGENTS.md
  already bans GTK-styled pickers for looking foreign; a WebKit select menu is
  the same category of problem. Removing it also deleted the popover's second,
  weaker copy of the preset picker.
- **`Playback settings ›`** footer row in the popover (same 0.82-of-`--text`
  action tier as the panes' reset rows, neutral hover wash, accent ring) closes
  the popover, opens the stack on the Playback pane and moves focus to the
  incoming layer's back chevron with `preventScroll` — the popover is the fast
  path for nudging bands; the pane is the way to the rest, and the button the
  user just pressed is where the route starts.
- Verified live (`/tmp/eq1-crop.png`, `/tmp/eq2-crop.png`): popover reports
  `{toggle: true, checked: true, preset: "Rock", select: false, sliders: 11,
  foot: "Playback settings"}`; clicking the footer yields
  `{popoverClosed: true, stackOpen: true, detail: true, focus:
  "backchev|Back to settings", sliders: 11}` and the four Toggle instances read
  the real state. Rock's curve matches band-for-band (+5/+4/+2.5/0/-2/-1/+1.5/
  +3.5/+5/+5.5). svelte-check flags no unused selectors after the dead
  `.eq-pop .toggle` / `.eq-pop select` rules were removed.

## Dismissal standardised on the traffic-light family (2026-08-31)

User's proposal, adopted with two amendments: every floating surface dismisses
with a red traffic-light dot at **top-left**, and the EQ popover's rows were
reordered so the header is chrome, the middle is the curve, the base is
ownership (`[✓] Equalizer …… Rock`).

- **`components/SurfaceClose.svelte`** — 14px dot (was 13; the titlebar dots grew
  to 14 and the gap to 10, making the cluster 24px on center — bigger dots, bigger
  hit area, and the family keeps one size), `--tb-x` (the user's KWin
  close colour, now published on `<html>` by `decoVars()` in the decoration
  store, shared with the titlebar instead of duplicated), 26px transparent hit
  box, glyph on hover AND keyboard focus, press = brightness 0.82, **red only**.
  Applied to: EQ popover, queue popover, About modal, Music folders modal (whose
  dismissal was a font-glyph `✕` — a third typographic family inside glass).
  **The settings stack keeps its boxed ✕**: its header row is the same row as the
  real window dots, and two red dots 200px apart with different scopes is worse
  than the shape difference removed. `.q-x` (remove row) also stays an ✕.
- **Focus discipline:** dismissal returns focus to the opener. The opener is
  recorded at OPEN (`openAbout(from)`, `ui.aboutOpener`; the popovers hold
  `eqBtn`/`qBtn`) — reading `activeElement` at unmount gives the dialog's own
  close button, because child effects run before parent effects and the autofocus
  has already moved focus inside.
- **EQ popover restructure:** header = dot + `Playback settings ›` route (0.82
  action tier); `.bands` = the curve, `.asleep` when off (`opacity: .45` on
  readouts + slots, band labels and the checkbox row keep full contrast —
  dimming the gate hides the only way out, and the curve stays editable while
  off: shape it first, then switch on); footer `.own` = Toggle + preset name over
  a `--border` seam, `cursor: default` because it is inert now.
- Queue popover: dot first, `Clear` pushed to the far edge (`margin-left: auto`)
  so the destructive action sits opposite dismissal. Titlebar dots got a 21px
  hit area via `::before { inset: -4px }`, capped by the 8px gap.
- Verified live: `--tb-x` on `:root` = `rgba(253, 82, 86, 1)` (the real KWin
  colour, not the fallback) and the dot computes to `rgb(253, 82, 86)`; popover
  probe `{header: [sc, route], footer: [Equalizer, Rock], asleep: false}`,
  off-state `{asleep: true, dbOpacity: .45, hzOpacity: 1, toggleOpacity: 1}`;
  close returns focus to `aria-label="Equalizer"`; About autofocuses the dot,
  `position: absolute` at 15px inset, and closing returns focus to the
  `About Songstress` row (before the open-time capture it landed on `<body>`).
  Screenshots: `/tmp/tl1-crop.png` (curve on), `/tmp/tl2-crop.png` (asleep),
  `/tmp/tl3-crop.png` (queue), `/tmp/tl4-crop.png` (About).
- **Two Svelte 5 gotchas for AGENTS.md:** a class forwarded into a child
  component does NOT receive the parent's scope id (`:global(.ab-close)` is
  required), and the parent rule also needs one extra class of specificity
  (`:global(.ab .ab-close)`) because the child's `.sc { position: relative }` is
  an equal-specificity rule in a later stylesheet. Music folders modal verified
  by structure only (it opens from the parked Global Menu).

## Modal scrim now respects the window radius (2026-08-31)

User catch: the About overlay squared off the window's rounded corners. Cause —
`.app` draws the 14px window radius with `overflow: hidden`, but `overflow` does
not clip a `position: fixed` descendant (its containing block is the viewport),
so `.ab-backdrop` / `.mf-backdrop` painted `rgba(0,0,0,.35)` into the transparent
corners.

- Added the missing ladder rung as a token (`--radius-window: 14px`), used it in
  `.app` (was a literal), and introduced **`.scrim`** in app.css carrying the
  whole overlay tier — fixed/inset-0, `z-index: 150`, the 0.35 wash, centering
  grid, and `border-radius: var(--radius-window)`. Both dialogs now use
  `class="…-backdrop scrim"`; their private copies of the same seven declarations
  are gone (z-index and wash strength were drifting candidates).
- Side effect worth knowing: `border-radius` affects hit-testing, so a click in
  the outer 14px corner triangle falls through the scrim to the grid instead of
  dismissing the dialog. Correct — that area is not visually part of the wash.
- Verified: computed `border-radius: 14px` on the scrim and on `.app`; corners in
  `/tmp/sc1-tl.png` / `/tmp/sc1-br.png` show the wash stopping on the curve.

## Scrim, take two: Tag editor was the third modal + containment (2026-08-31)

User: still square on my view. Two findings.

- **A modal was missed.** `TagEditor.svelte` carries a third private copy of the
  scrim (`.te-backdrop`: same fixed/inset-0/z-150/grid/0.35) and its dismissal was
  a **font-glyph `✕`** — the exact two things the previous two commits removed
  elsewhere. It was missed because the sweep grepped for the class names it knew
  (`ab-close`, `mf-close`, `close`) instead of for the pattern. Now
  `class="te-backdrop scrim"` + `SurfaceClose` first in `.te-head` (the header's
  `h2 { flex: 1 }` pushed the dot to the right until the markup was reordered —
  the dot goes LEFT, like every other surface).
- **Structural fix:** `.app { contain: paint }`. `overflow: hidden` does not clip
  fixed descendants; containment makes `.app` their containing block, so the
  window's rounded shape clips every overlay regardless of what it forgets. The
  radius on `.scrim` stays (explicit, survives someone dropping the containment).
- **Screenshot caveat that cost time:** the window capture's geometry is not
  stable — it was 2048×1280 (client × 1.6) earlier and 2128×1264 later, i.e. the
  decoration/shadow region is included sometimes, and the alpha channel makes the
  transparent margin read as white in viewers; a full-screen grab put *the
  terminal running this session* under the crop. So corner verification moved from
  pixels to the DOM: `elementFromPoint(1,1)` → `.head` (the click falls through the
  clipped corner triangle) and `elementFromPoint(40,40)` → `.ab-backdrop.scrim`
  (inside the arc). Computed: `.app` contain `paint`, all three scrims
  `border-radius: 14px`, Tag editor header `[BUTTON.sc, H2]`, dot
  `rgb(253, 82, 86)`, no `✕` glyph left in its text.
- If a user still sees a square corner in the wash, the candidate is the compositor
  rounding the window at a radius ≠ our 14px clip (then: read the radius from the
  decoration instead of hardcoding the rung).

## Settings stack rebuilt around KDE fidelity (2026-08-31)

Four panes were four dialects of "settings screen"; they are one dialect now, and the details are borrowed from the platform rather than
invented, which is the point of a KDE-styled DE app:

- **One dismissal family.** `SurfaceClose` is the Klassy dot (fill = KWin's own palette colour via `deco.buttonColors`, corner radius =
  `--tb-radius`, glyph on hover/focus only, 26 px hit box at the platform pitch), used by every floating surface; popovers also dismiss on
  outside-pointer-down and Escape, panes on the dot. The About footer opens an in-glass dialog instead of a native one, so no surface in the
  stack is a different windowing paradigm from its neighbours.
- **Geometry is measured, not eyeballed.** `kde_window_decoration` reads Klassy's own config (and `kdeglobals` for the palette) and publishes
  `--tb-dot` / `--tb-gap` / `--tb-margin` / `--tb-radius` on `<html>`, so the cluster and the panes' left edge match a theme we do not
  implement and a DPI we do not choose. Measured against Konsole's Klassy-rendered buttons: 15 px dots, 11 px gap, 41 px pitch, 41 px targets.
- **One row rhythm.** `Toggle` is the row primitive and takes `--sidebar-row-size` (40) as its min-height — the user's own setting, so the
  stack obeys a change made in Appearance. No borders, no inset cards (translucent cards on 0.7 glass read as a surface with a hole in it).
- **A three-rung spacing ladder carries all the hierarchy**: 6 content-under-own-label / 12 label-to-first-row / 30 between groups, so no
  rule, caption or background is needed. Playback's three options are one group of three label+control blocks: touching inside, ladder outside.
- **The sidebar's bottom row is a cluster** (Save, Rescan, Settings): a door to a settings screen does not get a full row of the sidebar whose
  job is the collection.

**Modal containment (the rule that took three tries).** `contain: paint` on `.app` makes it the containing block of fixed descendants, so
`overflow: clip` + `--radius-window` on the app frame rounds *everything in the window* to the window's own shape — set `--radius-window` from
KWin-reported `--tb-radius`. A scrim inside the frame must never paint its own radius or a square corner escapes the clip; the shared `.scrim`
in `app.css` is the only place the dim exists, and each surface clips itself because a child's own paint containment clips its border-box.
Verifying a change to containment needs a **cold restart**: HMR re-injects styles without recomputing the containment tree, and reloading does
not fix it.

