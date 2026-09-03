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
- **Amber Caution** (`#f2a33c` dark / `#b26a12` light — the light value is the
  same hue taken down to clear 4.5:1 on the porcelain glass): **the system's one
  caution hue**, spent on exactly three things — a missing-file alert glyph in a
  tracklist, a destructive *mark* ("Discard this album", "Remove this folder"), and
  error text. It ships as a set so a mark never half-adopts the hue:
  `--caution` (text), `--caution-line` (0.55 border), `--caution-wash` (0.14 fill).
  The only non-accent hue in the app. It was documented as exclusive to alert glyphs
  while two components improvised an undocumented salmon (`#ff8f8f`) for destructive
  state; the drift is closed by naming the tier, not by adding a second one.
- **Hue is never the only carrier.** Every caution state also changes border, fill
  and weight, so the meaning survives a colour-blind eye and a grayscale capture.

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
**Label/Mono Font:** none — and that includes filesystem paths. `Music
Files/Ghost/Impera` is set in Inter at 11.5px with `overflow-wrap: anywhere`;
a mono face was tried and rejected, because a second family in a one-voice
system reads as "technical" rather than as "where this is going", and the
break-anywhere rule only fires on a token too long for the line.

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
  above what it gates. The equalizer popover now agrees with it — the head carries
  the gate and the preset name (identity + state, next to dismissal), the curve is
  the body, and the route out to the Playback pane sits at the base under the seam,
  because leaving belongs with the exit and not with what a surface IS. It used to
  be written up as the deliberate inverse ("an instrument surface keeps its master
  at the base"); that bought nothing a head does not already buy, and it made the
  popover the only surface whose first row was navigation.
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

### Loading is a shape, not a message
While the library is being built the surfaces that will hold it hold **their own
shape with the content taken out**: the album grid shows real tile geometry (same
column count, same 20px grid gap, square cover at `--radius-cover`, same
13px cover→caption gap and caption line heights), and the artist list shows real
rows (`--sidebar-row-size` height, `.row`'s 10px inset and 8px radius) with a
name pill and a count pill. No headline, no bar, no numbers — the placeholder IS
the status. When the library lands it **fills** the shapes rather than reflowing
them, and the swap is instant, per the switch contract; only the skeleton's own
160ms fade-in is motion, and it is CSS (a `svelte/transition` would run at full
speed for users who asked for none).

- **One material, defined once.** `.sk` is global in `app.css` beside `.glass` —
  base `--sk-base`, sweep peak `--sk-hi`, its own token pair per theme, because
  `--hover` (a wash) and `--border` (a hairline) are tuned for other jobs. A
  shared *component* would leak: a class forwarded into a child is unscoped.
- **The sweep is a `translateX` on a pseudo-element**, `1400ms linear infinite`,
  with `animation-delay: calc(var(--i) * -180ms)` — negative, so the light is
  already mid-shelf on frame 1 instead of every placeholder starting in lockstep
  (which reads as one big flash, not as a surface waiting). No lift, no scale:
  the No-Lift Rule applies doubly to something that must never look interactive.
- **Never over content.** A re-scan of a populated library keeps its grid and the
  sidebar's dim "Updating library…" note; a skeleton that covers 251 real albums
  is a lie about the library, not about the wait.
- **Never a manufactured wait.** The boot dump is tens of milliseconds, so the
  placeholder waits `BOOT_GRACE_MS` before it may appear at all (the same rule the
  import manager documents for its `staged_plan` fetch). And it never *scrolls*
  past its own surface: the sidebar counts rows with `floor`, the grid with
  `ceil` capped at 4, because a placeholder you can scroll into is an infinite
  empty page.
- **Pill widths vary** — a uniform bar reads as a table, not as names — but from a
  fixed pattern array, never `Math.random()`: the node re-renders on every
  progress event, roughly every 6ms during a scan, and fresh random widths would
  shimmer on their own.
- `aria-busy` on the two regions, `aria-hidden` on the placeholders: the wait is
  announced once, not twenty-six times.

- **The arrival entrance belongs to the wait, not to the data.** When a placeholder
  gives way to content the shapes fade up into position — **320ms** `--ease-out`,
  **12px** of rise, **70ms** per item capped at **420ms** (~750ms end to end, past
  the 300ms UI ceiling on purpose: this fires on a first run, not a hundred times a
  day). **The spread is the animation, not the duration** — `--ease-out` spends 90%
  of the distance in the first ~37% of the time, so lengthening the duration alone is
  invisible settling (measured: 90% opacity at 74ms of a 200ms cut, which is why the
  first two versions both read as "too fast").
  **The staggered unit is per surface, because what has a position differs.** The
  artist list animates its ROWS (`--i` = index); the album grid animates TILES along
  their **DIAGONAL** (`--i = row + column`) — at a 295px tile barely one and a half
  rows fit on screen, so a row-level ladder there is a wall arriving late, and a wall
  is the opposite of unfolding. Strict reading order (`row * cols + col`) marches
  left-to-right and reads as a typewriter; the diagonal reads as a surface opening.
  Dials are custom properties declared on the **animating element itself** (inheriting
  them from the container costs a style recalc per child, and a declaration on the
  container is anyway overridden for children by the shared rule), and `--i` is inline
  per child.
  It is armed by `library.load()` when the dump it just wrote replaces an EMPTY library
  after a scan (or after a boot slow enough to have shown the skeleton), and by nothing
  else: a warm launch whose dump lands in 46ms must not animate, and an artist switch
  or a search keystroke re-mounts rows, so a permanent class would turn typing into
  confetti — the frequency tier where motion is disqualified. It is CSS so the
  `prefers-reduced-motion` kill switch can reach it (a `svelte/transition` would run at
  full speed for the users who asked for none); the reduced variant pauses the motion
  and zeroes the ladder, keeping the fade, because a sequence of zero-duration
  arrivals is worse than no motion. Past `:nth-child(24)` — and past the grid's 9th
  row — there is no animation at all: only what can be seen enters, since everything
  past a cut appears at t=0, which reads as the bottom of the list arriving first.

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

### Dismissal — two families, one verb each
- **The window's close is a dot; a surface's close is a box.** The traffic-light
  dot (close/min/maximize in the sidebar header) is the only thing in the app that
  wears the KWin close colour, because it is the only thing that quits the window.
  Every floating **surface** — modal *and* popover — dismisses with `SurfaceClose`,
  the boxed ✕ at the head's right edge. The earlier design put the dot on surfaces
  too ("one family, two dots"); that was fidelity taken one step past the truth:
  a circle in the user's close colour, in the close spot, on a surface that cannot
  close the app, claims a verb it does not have.
- **The box is not new.** It is the sidebar gear's own ✕ — the glyph the gear
  morphs into when settings open — same 16-unit box, 8-unit cross, 1.4 stroke,
  round caps, drawn as SVG. So "put this away" is one dialect in the app, and it is
  the dialect the user already presses twice a day. A text ✕ is banned (a third
  typographic family in a glass surface); `.q-x`'s glyph is drawn for the same reason.
- **Spec:** 28×28 (this app's icon-button rung, and its own ~28px minimum hit
  target met by construction — a floating surface has no drag region forgiving a
  smaller one), radius 7, `--text-dim` at rest → `--hover` wash + full-strength
  glyph on hover → `--active` wash on press, inset 2px accent ring on
  `:focus-visible`. **No transition**: the wash answers on pointer-down, and a
  dismissal that fades in reads as a delay. The glyph is absolutely centered
  (`translate: -50% -50%`), never `place-items` on a native `<button>` — that
  drifts ~1px down on this WebKitGTK.
- **Dismissal sits top-right, flush with the content edge** (measured: 17px from
  the panel's outer edge = 1px border + 16px padding, so the box's right edge is
  the same line the rows' right edges end on). A head therefore reads
  *what this is → what to do about it*, left to right, and the title's optical
  centre and the glyph's are the same line (measured: 119.5 / 119.5). About has no
  head row, so it places the box in the corner instead (`top/right: 8`). The
  settings stack was always in this family, as an exception; it is now the rule.
- **A destructive verb never neighbours dismissal.** The queue header used to put
  `Clear` at the far edge "opposite the close dot"; with dismissal at the far edge,
  `Clear` keeps the left group (title · count · Clear) and the whole slack of the
  row separates it from the ✕.
- **`.q-x` is a bare ✕, and that is now load-bearing.** With dismissal also an ✕,
  the two verbs are told apart by everything around the glyph: boxed + in the head
  + always visible = put this surface away; bare + inside the row + revealed on row
  hover = remove this row. Never give a row's ✕ a box, and never put one in a head.
- **Hit area (window cluster):** in the titlebar the dot grows its own box by half
  the configured gap (`::before { inset: calc(var(--tb-gap) / -2) }`), which makes
  the target exactly the cluster's pitch for *whatever* the decoration asks for.
  One more pixel and neighbouring targets overlap, and the later button steals the
  shared strip — which would bury the close dot's east edge inside minimize. Tune
  the decoration, and the hit area follows by construction: a cluster whose target
  is always its pitch cannot be resized into a miss.
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

### One press, one verb
- **The topmost surface owns the keypress.** Escape (and the scrim, and the ✕) does
  exactly ONE thing: while a modal, popover or the context menu is up, that surface
  answers and the sidebar's global key router stands down entirely — so closing a
  modal opened from the Library pane no longer pops the pane off the stack too, and
  `s` cannot push the stack in behind the About scrim. A second press then does the
  next thing: pop one menu level (sub → detail → root → home).
- **Precedence is a fact, not an order.** Every one of those handlers listens on
  `window`, and same-node listeners all run regardless of `stopPropagation` (only
  `stopImmediatePropagation` cuts them short, and Svelte re-attaches them on update
  so attachment order is not guaranteed). So `surfaceOpen()`
  (`src/lib/stores/surfaces.svelte.ts`) is what each handler checks. This raced
  twice before it was named: the first fix moved About's handler *into* the router,
  which made About correct and everyone else still wrong.
- **A layer keeps showing its OWN content while it animates out.** The stack's
  branch conditions read `shownDetail`/`shownSub` — the last non-null id — not the
  live `menuDetail`/`menuSub`, which flip to null at t=0 of a back. Rendering for
  the null state is either a blank panel sweeping across (detail) or, worse, the
  WRONG pane: the sub layer's `{:else}` branch is the accent picker, so backing out
  of the EQ preset list printed colour swatches over the drawer on its way out.
  Every gate that decides what the user can *act on* (`atDetail`, `atSub`, `inert`,
  `aria-hidden`, `class:active`) still reads the live state — the hold is paint-only.

### A floating surface: arrival, containment, and what must not scroll
- **It materializes, and it leaves the same way.** `.scrim` fades its dim over 200ms
  and the panel it carries rises **12px** out of `opacity: 0` while leaning in from
  `scale(.99)` over 240ms (origin centre — a modal is not anchored to a trigger, so it
  arrives from where it is). The dim leads by 40ms: the background settles, then the
  object lands. Those are distances and durations, not a factor — this rule used to say
  "scales in from `.97`", and a factor is a different experience per surface: the same
  declaration gave the 340×128 About card about 5px of edge travel and the 620×520
  import window a zoom. **The exit is the mirror on a shorter clock**: panel 150ms, dim
  190ms, and the lead reversed, because going out the object leaves first and then the
  world brightens.
- **The exit is CSS, and that is the whole reason it exists.** An animated outro needs
  the surface to stay mounted for its length, and the obvious tools — `svelte/transition`
  (WAAPI) or a `setTimeout` hold — are outside the reach of the global
  `prefers-reduced-motion` kill switch, which is why this section used to document "the
  exit is still instant" as a deliberate trade. A CSS `animation` ends at 0.01ms under
  that switch and still fires its end event, so the hold can end on `animationend` with
  no timer to be wrong. The mechanism is six lines per modal and **no new shared state**:
  `close()` starts the exit and the open flag stays true until the animation ends, which
  makes the flag mean *visible* — so `modalOpen()` (and therefore `inert`) keeps telling
  the truth while a dialog is still on screen. A second Escape / ✕ / scrim press
  force-closes, which is the backstop for an event that never arrives.
- **Opacity is applied inline, not from an attribute rule.** On this engine a value that
  changes through a `[data-open]`-style selector in the same batch as the `transition`
  string gets no transition at all for `opacity` (measured with `getAnimations()`:
  `height` and `transform` appear, `opacity` does not) — the fade dies while the box
  still moves, silently. Anything that animates opacity as part of a JS-driven move
  writes it on the element.
- **A modal makes the rest of the window inert.** `role="dialog" aria-modal="true"`
  is a claim about the a11y tree; without `inert` on the background it is a claim the
  code does not honour, and Tab walks the grid, the sidebar and the playbar behind a
  scrim that says otherwise. `App.svelte` sets `inert` on `.stage` from `modalOpen()`
  and each dialog rings its own Tab (`lib/focusTrap.ts`, edge-only, so DOM order
  inside the panel is untouched). **Popovers and the context menu are excluded on
  purpose**: their trigger is part of the interaction (`aria-expanded`, focus
  returning to `eqBtn`/`qBtn`), and inerting the playbar would blur the button
  mid-transaction. Dragging the window is not lost — the scrim already intercepts it.
- **What the surface exists to say is never below the fold.** A status or receipt
  that renders after a scrolling list is unread, and "unread" is the same failure as
  "unsaid". The imported-music window's two statements ("already in your library",
  "just applied") now sit in a band under the head's seam, outside the scroller. The
  band is `flex: none` and bounded (3 names, then a count) precisely because it does
  not scroll: it trades list height for certainty, so it has to stay short.
- **The newest fact gets the most authority.** Before this, the Apply receipt was a
  bare sentence while the older "already" list had a bordered card. In a band, order
  and label do the work: receipt first, each block named by what it is
  (`Just applied`, `Already in your library`).

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

## Settings fidelity is the point

The Settings panes borrow KDE's own vocabulary, so their details are not
decorative: a traffic-light dot is a Klassy button (`--tb-radius`, never a
pill), its fill is the KWin palette colour, and it takes a hover before it
shows a glyph, because an icon that is always drawn is chrome and one that
appears is an affordance. Size and spacing are measured from the platform,
not guessed: `kde_window_decoration` reads the user's Klassy config and the
effective stylesheet and publishes `--tb-dot` / `--tb-gap` / `--tb-margin`,
which the cluster and the panes' left edge consume, so we match a theme we
did not write and a DPI we did not choose. The dot's hit area is the full
pitch (`gap / -2` each side), so adjacent targets never overlap and every
target clears 24 px while staying under the 41 px of a window button.

A row of options is a **row of options**, not a list of cards: 40 px min
height (the user's own `--sidebar-row-size`), no borders, no background, the
whole row being the click target — 24 px of label plus 16 px of padding is a
40 px target wearing padding. Inset translucent cards on the 0.7-alpha glass
read as a surface with a hole cut in it, and every pane that does it makes
the stack look like a settings screen from a different app.

Spacing is a ladder of three rungs — **6 / 12 / 30** — and the ladder carries
all of the hierarchy, so no rule, caption or background is needed: content
under its own label 6 (a caption wants to be touching), a group's label to
its first row 12 (a heading wants to be near its subject, not tight against
it: 4–6 px reads as a caption of the row), between groups 30. A
multi-part group (Playback's three checkboxes) puts each part's control
*touching* its own label and separates the parts by the group gap — 6px/12px
on, 6px/12px off, 6px/12px crossfade — which reads as one thing made of
three parts. The previous shape, 6 px above a label and 12 px below it, made
every heading a caption of the row beneath and turned a stack of groups into
a stack of rows.

The sidebar's bottom row is a **cluster** — Save, Rescan, Settings,
side by side, one visual unit. A door to a settings screen does not deserve
an entire row of a sidebar whose whole job is the collection.

## Import staging, and the window that owns it

Staging is a **state of the row**, not a location on disk: a file the app is
holding a decision about is flagged, wherever it sits, so a pending import
gets cover art, a waveform, play counts and scan-safety the moment it is
imported instead of after the user remembers to press Save. The
consequence that justifies the machinery is failure: an interrupted copy
left a half-written duplicate that only a hash check could find, and an
app that cannot tell you which of two identical files it is holding has no
business deleting either.

**Files you already own are a receipt, not a no-op.** The honest answer to
pointing at a folder the library already indexes is "those 12 tracks are
already here, in Ghostlights", and the import window opens to say it — with
the destination, since the one question a staging UI must answer before you
commit is *where will this end up*. The window opens on import because the
alternative is a progress ring that ends and a library that looks exactly
the size it was, which is how "nothing happened" looks even when the app
correctly did nothing.

**A report that can be missed is a report that wasn't made.** The window opens to
say it, and the saying happens in a band under the head's seam — outside the
scroller. It used to render after the album cards, inside the list: measured on a
loaded pile, 389px of an 864px body was visible, which meant an eight-album import
opened with its own explanation off-screen, and the Apply receipt — the one message
about files that may have been deleted — was the furthest thing from view. Same
reason the band is bounded to three names plus a count: it does not scroll, so it
spends list height it has to earn back by staying short.

**The destination is the row's answer, so the destination survives.** Each row
resolves and states where the files will go — the rule in words (`merges into
Impera`), the path as evidence — and that ordering is now enforced in the layout:
the gloss ellipsizes first and the path wraps instead of truncating, because the
question this window exists to answer is *where*, and a long album title used to
cost the path everything but one character (`· M…`) with the full text available
only in a hover tooltip. A row that grows 14px is cheaper than a fact that
disappears.

**A pile is a set of albums, and albums group by artist.** One card per
artist, one row per album, rows touching and divided by a rule: the card
*is* the group, so no gap is wasted to say "these belong together". A gap is
used once for one job — 30 px between cards means separate artists, 12 px
under a card's label means that label owns everything below it, a 1 px rule
between touching rows means sibling. The same gap doing two jobs at once is
how a list stops reading as a structure.

**The window is a frame, not a shrink-wrap.** `height: calc(100vh - 140px)`, always: the
pile does not decide how tall the decision is. A content-sized window moved the floor
under the Apply buttons every time a row expanded or the report band appeared, and made
every open a different shape. Inside the frame the album list is the scroll port
(`flex: 1 1 auto` + `min-height: 0` — without that zero the long pile pushes the frame
open instead of scrolling inside it) and the footer is `flex: none`: when there is not
enough room, what gives is the list, never the buttons.

**The card's summary is the control.** One disclosure button per album, covering the
title line, the meta and the destination — not the caret alone. It used to be TWO
buttons carrying the same `aria-expanded`: one state announced twice, the list's
arrow-walk stopping twice per album, and a destination line dead to a click that landed
6px below a live title. The chevron is an indicator inside the button, `aria-hidden`,
because `aria-expanded` is what carries the state; the file list is the button's
**sibling** and never its child, or every track title gets swallowed into the control's
accessible name. Whole-summary is a safe target here specifically because expanding is
*reading* — the verbs that commit live in the other grid column, outside the expander.

**Save/Discard centre on the album's first two lines, not on the card.** The decision
column is a band of title line (18px) + the summary's own hug (6px) + ONE destination
line (15px), pinned to the top of its row. Centring it on the summary *block* would sit
the buttons lower on every album whose path needs two lines — and the path wraps on
purpose — so the decision column would arrive ragged. Those two line heights are
declared rather than inherited, because the band is arithmetic made of them.

**The window's shape is a function of the pile.** One album waiting → its file list
opens unfolded: it is the whole subject of the window, and the list is the reassurance
the decision needs. More than one → all collapsed, and the caret is there to be asked.
Either way the state resets on open, so reopening never shows whatever the user happened
to leave open last time.

**A file list unfolds; it does not jump.** Its height is measured in px and transitioned
— 200ms out, 150ms back, `auto` at rest — because this webview has no `interpolate-size`
(`CSS.supports` is false, measured), so `height: 0 → auto` cannot be animated in CSS at
all, and a fade over a box that snaps to its new height is precisely the motion that
reads as quick. Rest-closed is `display: none`, not `height: 0`: a zero-height grid item
still occupies its row, and every collapsed album would keep a 6px hole under its own
summary forever.

**Every pending thing is a floor under the button that decides it.** An
album's Save/Discard pair sits in the row it belongs to — the association is
layout, not memory — and the album's folder, which cannot be changed there,
is *shown* there. A control that is not bound to the thing it acts on is the
mistake the old sidebar made twice: an album-scoped button that saved
everything, and a global Save label that actually saved one album.

**The footer is an action, not a status light.** Its counts are of the
pending pile; `Apply` runs each marked row's own decision, which is why the
verb is Apply and not Save — one button doing two things is fine when the
button says "the decisions above", not when it says "Save" and means it
sometimes. Per-row buttons are marked rather than executed so the decision
stays reversible up to the commit; the ring fills from the real number of
albums already applied, so it cannot start from a stale count or spin in
place.

**Nothing here can be undone later, so it is decided now.** A file whose
resolved name the destination folder already holds byte for byte is a
duplicate at Apply: **the file the user just pointed at is deleted** and the
library's copy stays, and the window's receipt states it, because a silent
deletion of someone's own file is not acceptable even when it is a copy of
one they own. (Same name, different bytes gets ` (2)`; the rule is about the
destination, not about the whole library — the same song on a compilation and
on an album is a thing people own on purpose.) A file the library
already indexes is never staged. And the app removes folders it wrote in
its own staging area and nothing else — the folder you imported from stays
where it is, empty or not, because it is yours.

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
  Escape pops menu levels and clears-then-blurs the search field — and
  always belongs to the topmost floating surface first (one press, one verb).
- **Do** mirror the system: KWin palette for the **window's** traffic lights
  (`--tb-*`, and only for the window's — surfaces dismiss with the boxed ✕),
  button order
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
