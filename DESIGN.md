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

Generous but quiet radii: window 14px, panels 12px, covers 10px, controls
8px, icon buttons 7px, pills 999px. The radius ladder decreases with
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
- **Sidebar menu stack:** three absolute layers (home → Settings root → pane)
  sliding transform-only on `cubic-bezier(0.32, 0.72, 0, 1)` @ 320ms;
  root recedes -100% while a pane enters from the right; ✕ pops all
  levels as one conveyor (home in from left, root out left in parallel,
  pane out right) and teleports the root back to its entry side off-screen.
  Arrow keys walk the focused layer's buttons (Tab still works). The
  sidebar's tree is SIDEBAR-shaped, not a mirror of the Global Menu
  (menu-bar shape): Appearance / Playback / Library + a dim "About
  Songstress" footer row. View's theme item lives in Appearance; Playback
  drops its transport rows (the PlayBar owns them); "Save imported music"
  hides when nothing is staged. The About footer opens an in-glass dialog
  (name, Tauri version, one-line description — no native dialogs).

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
  (no native confirm — GTK dialogs are banned).

### Inputs / Fields
- **Search fields (sidebar 30px, titlebar-less era):** 8px radius, 1px
  Glass Line stroke, hover-wash fill, 14px leading icon, placeholder in
  dim; focus = accent stroke only.
- **Sliders:** native range inputs, appearance-none: 4px track (hover-wash
  base with an inline accent fill gradient sized to the value) + 14px
  accent-dot thumb; focus = accent ring. The equalizer's vertical sliders
  are horizontal inputs rotated -90° in fixed slots (WebKitGTK ignores
  `writing-mode` on ranges).
- **Checkbox:** hidden native input (stays focusable) + 16px rounded-square
  box (5px radius — deliberate, below the control tier for a 16px object):
  hover-wash fill + Glass Line at rest, accent fill + check in
  `--accent-text` (luminance-aware: white on dark accents, dark on light
  ones) when checked; 160ms fill/check-in; focus = inset accent ring.
- **Color picker:** hidden native input inside a conic-gradient swatch;
  preset swatches are 20px circles, selected = 2px Chalk outline.

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
- **Do** mirror the system: KWin palette for traffic lights, button order
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
