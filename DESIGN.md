---
name: Songstress
description: "Album-grid music player for KDE — lucid glass, the collection is the hero"
colors:
  slate-night: "rgba(22, 22, 28, 0.8)"
  slate-night-chrome: "rgba(22, 22, 28, 0.7)"
  porcelain-mist: "rgba(246, 246, 249, 0.8)"
  porcelain-mist-chrome: "rgba(246, 246, 249, 0.7)"
  panel-strong: "rgba(26, 26, 33, 0.94)"
  panel-strong-light: "rgba(252, 252, 254, 0.95)"
  stock-orchid: "#a78bfa"
  stock-orchid-light: "#7c58f0"
  stock-orchid-wash: "rgba(167, 139, 250, 0.18)"
  stock-orchid-wash-light: "rgba(124, 88, 240, 0.14)"
  chalk: "#f4f4f6"
  chalk-dim: "rgba(244, 244, 246, 0.64)"
  ink: "#232329"
  ink-dim: "rgba(35, 35, 41, 0.64)"
  glass-line: "rgba(255, 255, 255, 0.1)"
  glass-line-light: "rgba(30, 30, 40, 0.12)"
  hover-wash: "rgba(255, 255, 255, 0.07)"
  hover-wash-light: "rgba(30, 30, 40, 0.06)"
  amber-caution: "#f2a33c"
typography:
  headline:
    fontFamily: "Inter, 'Segoe UI', system-ui, sans-serif"
    fontSize: "22px"
    fontWeight: 700
    letterSpacing: "-0.01em"
    lineHeight: 1.1
  display:
    fontFamily: "Inter, 'Segoe UI', system-ui, sans-serif"
    fontSize: "20px"
    fontWeight: 700
    letterSpacing: "-0.01em"
    lineHeight: 1.15
  title:
    fontFamily: "Inter, 'Segoe UI', system-ui, sans-serif"
    fontSize: "15px"
    fontWeight: 700
    letterSpacing: "0.01em"
    lineHeight: 1.2
  menu:
    fontFamily: "Inter, 'Segoe UI', system-ui, sans-serif"
    fontSize: "13.5px"
    fontWeight: 400
    lineHeight: 1.2
  body:
    fontFamily: "Inter, 'Segoe UI', system-ui, sans-serif"
    fontSize: "14px"
    fontWeight: 400
    lineHeight: 1.4
  list:
    fontFamily: "Inter, 'Segoe UI', system-ui, sans-serif"
    fontSize: "13px"
    fontWeight: 400
    lineHeight: 1.2
  label:
    fontFamily: "Inter, 'Segoe UI', system-ui, sans-serif"
    fontSize: "11.5px"
    fontWeight: 600
    letterSpacing: "0.09em"
    lineHeight: 1.2
  micro:
    fontFamily: "Inter, 'Segoe UI', system-ui, sans-serif"
    fontSize: "11px"
    fontWeight: 400
    letterSpacing: "normal"
  instrument:
    fontFamily: "Inter, 'Segoe UI', system-ui, sans-serif"
    fontSize: "9.5px"
    fontWeight: 400
    letterSpacing: "normal"
    lineHeight: 1
rounded:
  window: "14px"
  panel: "12px"
  cover: "10px"
  control: "8px"
  icon: "7px"
  pill: "999px"
spacing:
  xxs: "4px"
  xs: "8px"
  sm: "12px"
  md: "16px"
  gap: "20px"
  lg: "24px"
components:
  menu-row:
    textColor: "{colors.chalk}"
    typography: "{typography.menu}"
    rounded: "{rounded.control}"
    padding: "0 10px"
    height: "40px"
  menu-row-hover:
    textColor: "{colors.chalk}"
    backgroundColor: "{colors.hover-wash}"
    typography: "{typography.menu}"
    rounded: "{rounded.control}"
    padding: "0 10px"
    height: "40px"
  menu-row-pressed:
    textColor: "{colors.chalk}"
    backgroundColor: "{colors.stock-orchid-wash}"
    typography: "{typography.menu}"
    rounded: "{rounded.control}"
    padding: "0 10px"
    height: "40px"
  search-field:
    textColor: "{colors.chalk}"
    backgroundColor: "{colors.hover-wash}"
    typography: "{typography.list}"
    rounded: "{rounded.control}"
    padding: "0 10px 0 30px"
    height: "30px"
  play-pause-button:
    backgroundColor: "{colors.stock-orchid-wash}"
    textColor: "{colors.chalk}"
    rounded: "{rounded.pill}"
    size: "34px"
  play-all-button:
    backgroundColor: "{colors.stock-orchid-wash}"
    textColor: "{colors.stock-orchid}"
    rounded: "{rounded.pill}"
    size: "34px"
  badge-pill:
    backgroundColor: "{colors.stock-orchid-wash}"
    textColor: "{colors.stock-orchid}"
    typography: "{typography.label}"
    rounded: "{rounded.pill}"
    padding: "3px 9px"
---

# Design System: Songstress

## Overview

**Creative North Star: "The Listening Room"**

Songstress is a dim room behind glass. The album collection is displayed,
not listed: artwork glows at tile scale, captions sit a step below it in
volume, and every control dims until touched — then answers with a wash,
never a glow. The interface recedes so the music's pictures lead. The
aesthetic is lucid, tactile, and restrained; luminous over busy. There is
no hero moment for the chrome itself: if a screen makes you look at the
buttons instead of the albums, it has failed.

The material is glass, and the glass is honest about where it comes from.
The window is transparent; KWin's force blur frosts the wallpaper behind
it, and every pane (grid 0.8, chrome 0.7, strong panels 0.94/0.95) is an
alpha tier of that same frosted field. The app also mirrors the user's own
system decoration — traffic-light palette, button order, and the Plasma
Global Menu come from KWin, so the player reads as Plasma furniture rather
than a guest with a skin.

**Key Characteristics:**
- Album grid is the interface; expanding a tile reveals its tracklist in place
- Glass = alpha-tiered transparency over KWin-frosted wallpaper, never solid fills
- Controls dim at rest (`--text-dim`), answer on press with a translucent accent wash
- One accent (Stock Orchid, user-overridable) used sparingly: state, selection, focus
- No titlebar: window chrome (traffic lights, menu gear) lives in the sidebar header
- Motion explains spatial structure (stack slides, panel height) — it is never decoration

## Colors

A single-hue neutral glass field with one orchid accent and one amber
caution color; the entire palette is expressed as alpha over the frosted
wallpaper rather than opaque fills.

### Primary
- **Stock Orchid** (`#a78bfa` dark / `#7c58f0` light): the only brand color —
  focus rings, selected states, the play button wash, accent text on wash
  backgrounds, the back-chevron. It is user-overridable at runtime
  (accent picker), so any surface using it must consume the `--accent`
  variable, never a literal.

### Tertiary
- **Amber Caution** (`#f2a33c`): reserved exclusively for missing-file
  alert glyphs in tracklists. The only non-accent hue in the app.

### Neutral
- **Slate Night** (`rgba(22,22,28,·)`): the dark-theme glass field. Two
  tiers: grid backdrop at **0.8** and chrome (sidebar, playbar, popovers)
  at **0.7** — both are user-tuned alphas, not defaults. Light theme
  mirrors as **Porcelain Mist** (`rgba(246,246,249,·)`).
- **Chalk & Ink** (`#f4f4f6` / `#232329`): text at full strength; at
  **0.64 alpha** (`chalk-dim` / `ink-dim`) for secondary captions,
  counts, and at-rest controls.
- **Glass Line** (`rgba(255,255,255,0.1)` / `rgba(30,30,40,0.12)`): the
  1px border language — sidebar edge, panel edges, field strokes, the
  single seam line above the playbar.
- **Hover Wash** (`rgba(255,255,255,0.07)` / `rgba(30,30,40,0.06)`): the
  universal attention state for rows, fields, and icon buttons.
- **Panel Strong** (`rgba(26,26,33,0.94)` / `rgba(252,252,254,0.95)`):
  near-opaque tier for popovers and modals that must isolate from content.

### Named Rules

**The Two-Tier Glass Rule.** Grid sits at 0.8 alpha, chrome at 0.7. The
chrome is *more* transparent than the field it floats over because the
field's darker tier already separates content from wallpaper. Do not
flatten the tiers or swap the alphas without the owner's sign-off — they
are tuned, not derived.

**The One Accent Rule.** Stock Orchid appears on ≤10% of any screen.
Washes (0.18/0.14) carry selection and primary action; the solid accent
carries text/icons only when sitting on its own wash. Never a gradient
accent, never a second brand hue.

## Typography

**Display Font:** Inter (with Segoe UI / system-ui fallbacks)
**Body Font:** same family — one voice, no pairing
**Label/Mono Font:** none; numeric readouts use `font-variant-numeric:
tabular-nums` instead of a mono face

**Character:** A single humanist sans at a compact 11–22px range;
hierarchy is built from weight (400/600/700) and dim-ness (full vs 0.64
alpha), not from size drama. Nothing is set in a face you can't blame.

### Hierarchy
- **Headline** (700, 22px, -0.01em, 1.1): EmptyState page headlines only.
- **Display** (700, 20px, -0.01em, 1.15): expanded album title — the one
  large text in the app, earning it as the album's name.
- **Title** (700, 15px, +0.01em): nav-row titles ("Settings", pane names).
- **Menu** (400, 13.5px): sidebar menu rows — slightly above list size to
  mark the menu as its own cadence.
- **Body** (400, 14px, 1.4): base UI text.
- **List** (400, 13px): artist rows, captions, track titles, search fields.
- **Label** (600, 11.5px, +0.09em, UPPERCASE): section labels ("Songs",
  "Albums"), badges ("Imported"), disc titles.
- **Micro** (400, 11px, tabular): counts, times, durations.
- **Instrument** (400, 8–9.5px, tabular): the EQ's dB/frequency readouts
  and the mode-button state badges (A / R / ALL / 1 / queue count) only —
  text functioning as instrument scale, not content. The smallest tier;
  nothing else may use it.

### Named Rules

**The Tabular Rule.** Anything that changes while you watch — positions,
durations, track counts — is set `tabular-nums` so digits don't jitter.

## Layout

Fixed borderless window (min 960×600, default 1280×800) with a 14px
corner radius and transparent body; the wallpaper shows in the corners.
Three zones: a 236px sidebar (left), the album grid (fills the rest),
and an 84px playbar (full-width bottom). There is **no titlebar** — window
controls sit in the sidebar header.

The grid is a row model: column count derives from available width ÷ tile
size (user-tunable 120–320px, default 180px) with 20px gaps; an expanded
album becomes a full-width panel row that pushes subsequent rows down.
Sidebar artist rows are user-tunable height (28–52px, default 36px);
menu rows are fixed 40px.

The grid scroller clips at the playbar's top line: rows slide out *under*
the shelf rather than passing behind its glass, softened by cast-light
gradients (26px @ 0.22 at the window top, 34px @ 0.30 above the playbar).

## Elevation & Depth

**"Glass Before Shadow."** Depth is the alpha tiers of the glass field —
0.8 grid, 0.7 chrome, 0.94/0.95 strong panels — floating over KWin's
force-blurred wallpaper. WebKit's `backdrop-filter` does **not** blur
in-window content on this engine (probe-verified), so no in-app frost is
ever faked by it; in-window softening is done by clipping content out
from under chrome and by cast-light gradients. Box shadows play only two
supporting roles: anchoring physical objects, and casting light.

### Shadow Vocabulary
- **Structural anchor** (`box-shadow: 0 10px 32px rgba(0,0,0,0.45)` dark /
  `0.16` light): expanded panels and their expander shell — the one
  surface that must read as an object placed on the glass.
- **Cover mass** (`0 4px 18px rgba(0,0,0,0.35)`): album artwork (grid
  tiles, expanded art, playbar thumbnail) — gives each cover physical
  weight without lifting it.
- **Popover seat** (`0 8px 28px rgba(0,0,0,0.35)`): menu/popover surfaces.
- **Cast light** (linear gradients, not shadows): 26px @ 0.22 from the
  window top and 34px @ 0.30 from the playbar shelf, onto the grid —
  chrome casting onto content so the glass reads as floating.

### Named Rules

**The No-Lift Rule.** Grid tiles never translateY, scale, or promote a
compositor layer on hover (WebKit paints that churn as a one-frame blank
cover flash). Hover = outline ring; press = wash. Motion belongs to
structure (panels, stack), not to tiles.

## Shapes

Generous but quiet radii: window 14px (`--radius-window`), panels 12px, covers
10px, controls 8px, icon buttons 7px, pills 999px. The window rung is a token,
not a literal, because the modal scrim (`.scrim` in app.css — About, Music
folders, Tag editor all use it) has to be clipped to it: `overflow: hidden` on
`.app` does not clip a `position: fixed` descendant, so an unrounded scrim paints
square corners over the window's transparent ones. Two defenses, both load-bearing:
the scrim carries `border-radius: var(--radius-window)`, and `.app` carries
`contain: paint`, which makes it the containing block for fixed descendants so
**every** overlay is clipped to the window shape — a future surface cannot
re-introduce the artifact even if it forgets the radius. Any future full-window
overlay takes `.scrim`, never a private copy of it. The radius ladder decreases with
object size — the biggest surface gets the softest corner. Borders are
always the 1px Glass Line; nothing has a double border, and the sidebar
deliberately has **no** bottom border (the playbar's top line is the only
seam).

## Components

Controls dim at rest, wash on hover, take an accent wash on press —
lucid and quiet.

### Buttons
- **Shape:** 7–8px radius icon squares (28px), 34px circles for primary
  transport, pill badges.
- **Transport (play/pause):** translucent orchid wash
  (`--active`, 0.18/0.14) with full-strength Chalk glyph — the wash is
  translucent, so text color stays normal; solid accent is reserved for
  glyphs on wash.
- **Mode buttons (shuffle/repeat/EQ/queue):** dim at rest (opacity 0.75,
  `text-dim`); full Chalk when active *plus* a micro badge (8px accent
  text, e.g. "A", "1", count) — the badge, not a glow, signals state.
- **Press:** every pressable takes `--active` wash on `:active` instantly;
  no scale animation (list cadence, 100+ uses/day).
- **Focus:** inset 2px accent ring (`outline-offset: -2px`) — the app's
  only focus language.

### Iconography

Two families, split by role — every glyph in the chrome belongs to one:
- **Media family** (transport, volume): 16×16 viewBox, **filled shapes**
  (play/skip triangles, speaker body) plus **1.6 round-cap strokes** (skip
  bars, speaker waves, mute cross). Rendered 15–17px. 1.6 is the one stroke
  weight in this family — arcs and bars must not drift thinner or heavier.
- **Utility family** (mode buttons, popover close X, queue-row X):
  Lucide-style 24×24, **pure stroke 2**, round caps/joins. Rendered 14–16px.
- **Weight hierarchy inside the media family encodes jump size:**
  bar+triangle = album-level jump, bare triangle = track-level step. The
  icons differ in mass so the pair is distinguishable without tooltips.
- No typographic glyphs (×, −, +) in interactive chrome — the old font ×
  read as a third family beside the SVGs.

### Rows & Navigation
- **Menu row (one row language, 2026-08-30):** the sidebar has ONE row
  language — artist rows AND menu rows share `--sidebar-row-size` height,
  13px type, and the same label column. Trailing glyphs sit in the count
  column: a 14px accent ✓ for checked items, a dim › chevron on root rows
  that open a pane (full on hover). No left check column. Full-width, 8px
  radius, Glass hover wash, accent wash on press; disabled = dim, no wash.
- **Artist row (user height, default 36px):** name left (13px), count
  right (11px dim, tabular); active = `--active` wash + 600 weight.
- **Track row (expanded panel):** number (tabular, 12px dim) · title
  (13px, ellipsis) · duration (12px dim, tabular); current track = accent
  text + 600 weight with a ▶/❚❚ glyph replacing the number; missing file =
  Amber Caution alert triangle replacing the number.
- **Sidebar menu stack:** four absolute layers (home → Settings root → pane →
  sub-pane) sliding transform-only on `cubic-bezier(0.32, 0.72, 0, 1)` @ 320ms.
  Each push takes the layer behind it a FULL width left (no parallax peek in a
  flat glass column); ✕ pops all levels as one conveyor (home in from left,
  root out left in parallel, pane/sub-pane out right), with the off-screen
  layers teleporting back to their entry slots under a one-frame transition
  suppressor. Arrow keys walk the focused layer's buttons (Tab still works).
  The sidebar's tree is SIDEBAR-shaped, not a mirror of the Global Menu
  (menu-bar shape): Appearance / Playback / Library + a dim "About
  Songstress" footer row. View's theme item lives in Appearance; Playback
  drops its transport rows (the PlayBar owns them); "Save imported music"
  hides when nothing is staged. The About footer opens an in-glass dialog
  (name, Tauri version, one-line description — no native dialogs).
- **Pane bodies (Appearance, Playback):** grouped by meaning with Label-tier
  seams ("SIZES", "THEME", "PRESETS", "CUSTOM") and a deliberately UNEVEN
  rhythm — ~10px inside a group, 30px between groups (the old uniform 16px
  cadence read as one crammed block even though 40% of the column below was
  empty). A control's label is its row's primary content: List tier at full
  `--text`, never dim. Rare, high-option-count decisions sit one push deeper —
  the accent grid is a sub-pane behind an `Accent · <name> · ›` summary row
  (11 presets in a 6-track grid, the picker in its own group), the 10 EQ
  presets behind `Preset · <name> · ›`.
- **Enable checkbox owns "off" (Playback pane).** A setting whose value space
  is off + N modes is a checkbox plus a segmented control of the N modes, never
  one cycling row labelled `Repeat: Off` (the value welded into the label
  changes width while you watch and hides the state space) and never a segment
  that includes Off (two controls, one truth). The fields the checkbox owns
  reveal UNDERNEATH it, indented 12px — no card, because inset translucent
  panels on the 0.7 chrome glass are banned — and they mount and unmount with
  it, which is also how a dependent field stops being a dead row: `Preset`
  simply does not exist while the equalizer is off. Reveals fade+rise 4px over
  200ms rather than animating height (a height transition on a block that can
  hold 11 sliders needs a magic max-height). Un-checking remembers the stage, so
  re-checking restores what you had.
  Ordering is genre, not law: a pane is a form read top-down, so its gate sits
  above what it gates. The equalizer popover is an instrument surface, so the
  curve comes first and its master sits at the base, above the seam. Do not
  "fix" one toward the other.
- **Pane footers:** every detail layer ends in one dim row — an action where a
  real reset exists (`Reset appearance`, `Reset playback`) or a status line
  where it doesn't (`248 albums · 4434 tracks · scanned 14:02`, tabular, and
  the time clause only appears once a scan has actually run). A footer action
  resets EVERYTHING its pane shows and is named for that scope — `Reset
  equalizer` under Repeat/Shuffle/Equalizer reset only the third. Footer actions
  sit at 0.82 of `--text` (≈4.6:1 over the worst case: bright wallpaper under
  the 0.7 chrome tier); `--text-dim` is caption-only there because it does not
  clear AA over glass.
- **Sidebar panes are store-driven, the Global Menu is menu-model-driven.**
  Appearance and Playback render from the Svelte stores; the menu bar keeps its
  flat cycling rows. Both write through the same setters (`setShuffleStage`,
  `setRepeatStage`, `setEqEnabled`, `applyEqPreset`), and `pushMenuState()`
  re-renders the menu from the same state, so the two surfaces can render
  differently and cannot disagree.

### Cards / Containers
- **Album tile:** square cover (10px radius, cover-mass shadow) + caption
  column (title 13px/600, sub 11.5px dim). Hover = 2px outline ring in
  `text-dim`; playing or expanded = ring in `--accent`. No transforms.
- **Expanded panel:** 12px radius, 1px Glass Line, panel-bg tier with an
  art-derived gradient at 0.36 (dark) / 0.30 (light) alpha; 20px padding;
  structural-anchor shadow on its expander shell (never on the panel —
  the shell must stay unclipped for the height animation).
- **Popovers/modals:** `panel-strong` near-opaque tier, 10–12px radius,
  popover-seat shadow; destructive confirmations render *inside* the glass
  (no native confirm — GTK dialogs are banned). **No native form widgets** in
  glass: a `<select>`'s open menu is the toolkit's own chrome (same category of
  problem as the banned GTK chooser) — the equalizer popover shows its preset
  NAME in Micro tier and routes the rest to the Playback pane through a
  `Playback settings ›` footer action, so the button that opened it is also the
  way to everything it no longer owns. The popover keeps the one thing worth
  having next to the transport: the curve, editable while something plays.

### Settings panes are bespoke
All three sidebar panes (Appearance, Playback, Library) are store-driven
components, not renderings of the Rust menu model. The generic row list is the
**fallback** for a pane without a branch — `rootPanes` still comes from the model,
so a new pane added in `menu.rs` appears in the root layer and renders from the
model until someone builds its branch. Actions inside a bespoke pane go through
`activateMenuItem`, so a click in the pane and a click in the Global Menu are the
same door into the same Rust command. The one exception so far: the Library
pane's `Manage imported music…` opens a modal, and a modal has no menu id to
carry — the Global Menu keeps its own save-everything verb.

- **Grouped lists, not card grids.** Inside the imported-music modal each artist
  is ONE card with its albums as touching rows divided by a hairline — the sidebar
  `.rows` idiom at content height. A card per album forces every distance in the
  window to argue about hierarchy; a card per subject makes the structure
  structural. Card bodies clip (`overflow: clip`) so focusing a row cannot scroll
  the card under the reader.
- **An amount is stated once, as a state** — never welded into a row label
  (`Manage imported music…` with `2 albums` in its tail, not `Save 2 albums`,
  which changes width while you watch and ellipsises the label). Same rule that
  killed `Repeat: Off`. Inside the imported-music modal the footer states the
  pile (`2 albums · 36 tracks`) and, separately, what Apply will do to it
  (`1 album to save · 1 album to discard`).
- **Progress rides on the row that caused it, and it fills.** The scan and
  re-read rows carry a determinate ring in their tail (`ProgressRing.svelte`) —
  no text line under the group, which grew the pane mid-action and pushed the
  rows below it around while the user was reading them. An operation that runs
  from a modal carries its arc in the modal's footer instead, and the pane's row
  ring lights up behind it because `scanner.kind` does not care where the verb
  came from. `scanner.kind` names the
  verb in flight, set by the operation and not the click, because a scan can
  start from the Global Menu, the empty state or a relink and the pane would
  otherwise be disabled with nothing to point at. Six rows, six kinds: import,
  save, discard, scan, full, folder. A ring is never decorative — if every row is
  disabled, exactly one of them says why.
  It is a sweep and not a spinner on purpose: rotation says "busy", an arc says
  "40 seconds in, 30 to go", and a scan you cannot see finishing reads as a hang.
  No events yet (total 0) draws an empty track, which is the honest "started, no
  news"; a fake halfway would be a lie that happens to look like progress — and
  that is not a hypothetical: because `done`/`total` outlive a finished run, the
  first version of this appeared on the row at the PREVIOUS run's 100%, unwound
  counter-clockwise, then started climbing, which read as an arc that did a lap
  before it began. Every operation therefore begins by clearing the counts
  (`begin(kind)`), so the arc's only direction is forward; and because a run's
  later phases carry their own totals, the arc is keyed on the phase — a new
  phase restarts the node instead of animating backwards through what it already
  reported. **And it lands full**: the backend's last progress event is short of
  the end, so unmounting on completion left the arc dying at 85% on the row that
  had just been asked to do something, which reads as an interruption. `end()`
  holds that row's arc at 100% for 300ms (`heldKind`) — long enough to register as
  finished, too short to feel like a stage — and only for runs that reported
  progress, so a no-op operation never flashes a ring it had no data for. Stroke
  is `--accent` — the same token the seek fill and the focus rings use, so the
  app's idea of "your colour is doing something" is one idea. The sentence the
  arc replaces ("Scanning 1,677 of 4,487") survives as `aria-valuetext`.
- **`role="status"` goes where a number moves** (scan progress) — a process you
  cannot see finishing reads as a hang. `.gstat` is caption tier with tabular
  nums; it sits inside the group whose rows it explains, not in the footer.
- **The footer states what the library IS** (counts, last scan), never what it is
  doing — two lines, the second subordinate (`.fstat .fsub`), so growth in the
  counts can't wrap a joined line mid-clause.
- **A destructive action with no dialog asks in place**: the button's own label
  becomes the question, naming the amount (`Discard 2 albums now?`), and leaving
  the pane disarms it. No timeout — the question would vanish under the cursor.
  It spends no color: the caution amber is documented but not yet a token, and an
  armed button is a bigger claim than a label change needs.

### The pane spacing ladder — 6 / 12 / 30
Spacing inside a settings pane encodes a relationship, and there are exactly
three relationships to encode:

| gap | lives on | meaning |
|---|---|---|
| **6px** | `.subject` | a row and the fields **it owns** — the only hug in the app |
| **12px** | `.group`, `.reveal` | anything and its neighbour in one group, group labels included |
| **0** | `.rows` | a run of rows — pitch is the row height itself |
| **30px** | `.panebody` | one titled group against the next |

Every rung is a **container gap**, never a margin on the thing itself: a parent's
scoped CSS does not reach a child component's root element, so `Repeat` (a
`<Toggle>`) is invisible to `.group > *`. The one place a negative margin is
correct is where you must go *below* a container's gap, and after the second round
of this only that remained — the label hug came out, because an uppercase 11.5px
micro title 6px above a control reads as a caption glued to it, and headings want
air at any size.

**A pane is as many groups as it has titled subjects.** Playback's
Repeat / Shuffle / Equalizer are three rows of one subject, so they are one
`.group` of three `.subject` blocks at 12px, each hugging its own reveal at 6px —
not three `.group`s at 30px, which said "unrelated subjects" and made the pane
read as a list whose pages were falling out of it. Appearance (SIZES, THEME) and
Library (Import, Scan, Storage) do have titled subjects and keep the 30px.

Before all of this, one undifferentiated 10px gap did everything, so a toggle's own
reveal sat exactly as far from it as an unrelated subject did. Ownership is now the
*tightest* distance in the pane and the only special case, which is why no divider,
box or second indent level is needed to say what belongs to what.

### A checkbox row is a row
`Toggle.svelte` takes `min-height: var(--sidebar-row-size)`. It used to be a
16px label, which made every pane that contained one fall off the row rhythm —
Appearance's "Playbar artwork gradient" sat as a caption between two 40px rows,
so the pane's spacing looked unlike its neighbours' even while the gap ladder was
identical. Now: the whole row is the click target, and a run of checkbox + drill
rows touches at exactly the pitch of the root layer, the artist list and the
Library pane. (The compact variant used in the equalizer popover opts out — a
popover header is not a list.)


### Row rhythm: lists touch, forms breathe
Inside a pane group, consecutive action rows go in a `.rows` wrapper with **no
gap** — pitch exactly `--sidebar-row-size`, the same rhythm as the root layer, the
artist list, the queue and the model-driven list. A group's own 10px gap is air
for **mixed content** (a label, a slider, a segmented control, a reveal); letting
it fall between rows too is what made the Library pane's rows float apart from
every other list in the sidebar — its first render was 50px pitch against the
app's 40.

The boundary is genre, same as the enable-checkbox rule: a pane that is a **list
of things you can do** (Library) keeps the list rhythm; a pane that is a **form of
mixed controls** (Appearance, Playback) uses group air, because each control owns
the field that reveals under it. If a form's spacing starts reading as arbitrary,
the fix is proximity-by-relationship (owner→owned tighter than sibling→sibling),
not a global gap.

### Dismissal — the traffic-light family
- **One family, two dots.** The titlebar's close dot quits the **window**;
  `SurfaceClose` dismisses a **surface**. Same device on purpose: one circle, sized by `--tb-dot`,
  the user's own KWin close colour (`--tb-x`, published on `<html>` by
  `decoVars()` so titlebar and surfaces cannot drift), ✕ glyph revealed on hover
  *and* on keyboard focus, press = brightness 0.82. No transition: the Plasma
  dots change instantly and the family is one behaviour.
- **Red only.** A lone dot never shows a yellow or green it cannot honour —
  a popover has no minimize and no zoom.
- **Dismissal sits top-left**, where the app has always put its close dot, in
  every floating surface: popovers, modals. The header row above the content *is*
  the surface's chrome.
- **The settings stack is the exception** and keeps its boxed ✕ top-right: its
  header row is the same row as the real window dots, and two red dots 200px
  apart — one closing a drawer, one closing the app — is a worse confusion than
  the shape difference being removed.
- **Hit area:** the button is 26px around the dot (a bare dot is under this
  app's own ~28px minimum, and a popover has no drag region forgiving it). In the
  titlebar the dot grows its own box by half the configured gap
  (`::before { inset: calc(var(--tb-gap) / -2) }`), which makes the target
  exactly the cluster's pitch for *whatever* the decoration asks for. One more
  pixel and neighbouring targets overlap, and the later button steals the shared
  strip — which would bury the close dot's east edge inside minimize. Tune the
  decoration, and the hit area follows by construction: a cluster whose target is
  always its pitch cannot be resized into a miss.
- **The geometry is read, not eyeballed.** `kde_window_decoration` publishes
  `--tb-dot`, `--tb-gap`, `--tb-margin`, `--tb-radius` from klassyrc
  (`ButtonSpacingLeft`, `TitleBarLeftMargin`, `WindowCornerRadius`, `IconSize`),
  because a cluster coloured like the user's decoration but sized like an
  imitation still reads as one. Calibration, measured off the user's own screen
  (scale 1.6, so divide the pixels by 1.6 for CSS): Konsole's Klassy buttons are
  24x24 px = **15 logical**, centers on a 42 px = **26 logical** pitch — button
  rect 16 (KDE's Small icon tier) + `ButtonSpacingLeft=10`, the small circle
  filling the rect minus 1. So: dot = standard icon size for the configured tier
  minus 1, visible gap = configured spacing + that 1px of slack.
  `cargo test`'s `deco_geometry` locks the mapping and the calibration.
  Verified after the change, in the same screenshot, our cluster: 24 px dots on a
  42 px pitch, starts 1945/1987/2029 against Konsole's 25/67/109 — **pixel
  identical**, `/tmp/cmp_dots.png`.
- **Known deviations, in case they matter later:** our titlebar is taller than
  Klassy's would be (`TitleBarTopMargin=10` + rect 16 + `TitleBarBottomMargin=10`
  = 36 logical; the head is ~44, because the gear and the stack's close need a
  28px target), the dot's rim is `rgba(0,0,0,.18)` where Klassy draws the same
  hue opaque (`VaryColorCloseOutlineActive=Opaque`), and the glyphs are ours, not
  Klassy's `StyleArk`. The fill is not a deviation: both draw at
  `ButtonBackgroundOpacityActive`, 85% on this system, applied by `decoVars`.
- **Dismissal returns focus to the opener** (recorded at open: `openAbout(from)`,
  the popover's own `eqBtn`/`qBtn`) so Tab continues where the user was. The
  opener is captured when the surface opens, not when it unmounts — by then the
  dialog's autofocus has already moved `activeElement` inside.
- **`.q-x` stays an ✕.** "Remove this row" is not dismissal; giving it the close
  colour would teach the wrong verb.

### Inputs / Fields
- **Search fields (sidebar 30px, titlebar-less era):** 8px radius, 1px
  Glass Line stroke, hover-wash fill, 14px leading icon, placeholder in
  dim; focus = accent stroke only.
- **Sliders:** native range inputs, appearance-none: 4px track (hover-wash
  base with an inline accent fill gradient sized to the value) + 14px
  accent-dot thumb; focus = accent ring. The equalizer's vertical sliders
  are horizontal inputs rotated -90° in fixed slots (WebKitGTK ignores
  `writing-mode` on ranges). Every slider carries its **value** — Micro tier,
  `tabular-nums`, dim, updating live while dragging (the Tabular Rule: nothing
  that changes while you watch may jitter or stay hidden) — in the row's right
  column in the panes, above the slot in the equalizer's 11-column popover,
  where there is no right column. `dB` is stated **once per group**, on its
  first row (Preamp), not repeated eleven times; `aria-valuetext` carries it in
  full for the screen reader either way.
- **Segmented control (2–4 exclusive modes; `--seg-n` sets the column count):**
  32px tall, 1px Glass Line stroke, `--hover` trough, 7px inner radius on the
  segments (icon tier — the nested 8px−2px arithmetic gives 6, which is off the
  ladder and indistinguishable here). Active segment takes the accent wash +
  600 weight (state, not hover); inactive segments are dim and hover NEUTRAL —
  hover never spends the accent. `role="radiogroup"` + `role="radio"` /
  `aria-checked`, one Tab stop with a roving tabindex, inset accent ring on
  focus, Left/Right/Home/End inside the group (one shared `segKeys` handler).
  Used for Theme (**System | Light | Dark**), Repeat and Shuffle.
  System goes LEFTMOST: reading left to right, the first segment is where the
  default belongs, and following the OS is the default for a desktop player
  (user decision, 2026-08-31). Segment order is also the group's arrow /
  Home / End order, so Home lands on the default.
- **Checkbox** (`components/Toggle.svelte`, the only checkbox in the app — four
  sidebar call sites and the equalizer popover; the popover's bare native
  checkbox was the last widget outside the system): 16px rounded-square
  box (5px radius — deliberate, below the control tier for a 16px object):
  hover-wash fill + Glass Line at rest, accent fill + check in
  `--accent-text` (luminance-aware: white on dark accents, dark on light
  ones) when checked; 160ms fill/check-in; focus = inset accent ring.
- **Color picker:** preset swatches are 20px circles in a 6-track grid,
  `gap: 14px 0` — 14px rows because the selection outline reaches 4px past a
  dot, and **zero column gap** because the tracks are fixed 20px and
  `justify-content: space-between` is what centers the row: any minimum column
  gap pushes the tracks past their own box (6×20 + 5×20 = 220 inside a 207px
  pane) and the last dot runs into the sidebar border. Spacing is then fluid
  (≈17px between dots, clearing the 4px outline twice over) and both edges sit
  on the pane's 14px padding. selected = 2px Chalk outline.
  The native color input lives in its OWN group under a `CUSTOM` seam, as a
  readout row (`Pick any color · not set / #rrggbb | conic swatch`) — the same
  label-left / Micro-value-right language as the slider rows, and the whole row
  is the label so the text opens the chooser. It is not a twelfth dot in the
  grid: the picker is an editor, not another named choice, and among named
  colors a permanent rainbow promised something the presets didn't need. Once a
  custom color is picked the swatch becomes that color with a 9px conic corner.
  Accent presets render their raw hex, and any preset whose lightness the theme
  clamps (`accent.ts` CLAMP — Black in the dark theme becomes light grey) wears
  a 2px ring in the color it actually produces, plus a tooltip naming it. The
  ring appears only where the promise and the result differ, and NOT on the
  selected dot: two near-identical concentric rings read as a rendering glitch
  and make the selected dot look disabled.

### Chips / Badges
- **Pill badges (999px radius):** 3×9 padding, 10px/600 UPPERCASE label,
  orchid wash + orchid text — used for "Imported" staging markers and
  queue counts. Staging badges on covers flip to solid `rgba(0,0,0,0.62)`
  + white because they sit on artwork, not glass.

## Do's and Don'ts

### Do:
- **Do** consume theme state through the `--*` custom properties
  (`--accent`, `--active`, `--text-dim`, tiers) so runtime accent and
  theme flips reach every surface.
- **Do** keep chrome alphas at the tuned values (grid 0.8, chrome 0.7,
  expanded-panel gradient 0.36/0.30) and let KWin's force blur do the
  wallpaper frosting.
- **Do** animate structure only: transform for the sidebar stack (320ms,
  `cubic-bezier(0.32, 0.72, 0, 1)`), px-height for panel expand
  (360ms, `cubic-bezier(0.22, 1, 0.36, 1)`), 160ms opacity for album
  switches — and respect the global `prefers-reduced-motion` kill switch.
- **Do** give every pressable an instant `:active` wash and an inset
  `:focus-visible` accent ring — except text fields, whose focus state is
  the accent stroke (an inset ring vanishes against a near-opaque fill).
- **Do** keep the keyboard accelerators: `/` focuses library search, `s`
  toggles the Settings stack (both inert while typing or with modifiers),
  Escape pops menu levels and clears-then-blurs the search field.
- **Do** mirror the system: KWin palette for traffic lights (and for every
  surface's close dot — same `--tb-*`), button order
  from the decoration config, kdialog for pickers, Plasma Global Menu as
  the primary menu surface.

### Don't:
- **Don't** trust `backdrop-filter` for in-window frost on this
  WebKitGTK — it is a no-op for content; use clipping and cast-light
  gradients instead.
- **Don't** put transforms, scale, or layer promotion on grid tile hover —
  WebKit paints the churn as blank cover flashes.
- **Don't** add a titlebar, a second brand hue, gradient text, glow
  shadows, or solid accent fills with accent-colored text on top (washes
  take Chalk text; solid accent takes white).
- **Don't** use native dialogs or confirm() — everything renders inside
  the glass in the app's own language.
- **Don't** drift the tuned alphas or the single-seam rule (sidebar has no
  bottom border; the playbar's top line is the only seam).
- **Don't** namespace-less CSS in components — leaked global selectors
  have broken this app before; prefix what you add.
