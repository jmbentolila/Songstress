---
target: sidebar settings menu stack (root + Appearance/Playback/Library panes)
total_score: 24
max_score: 40
na_heuristics: 
p0_count: 0
p1_count: 3
timestamp: 2026-08-31T18-30-27Z
slug: src-components-sidebar-svelte
---
⚠️ DEGRADED: single-context (no sub-agent/Task tool exposed in this harness — Assessment A and B ran sequentially inline, A first).

Target: the sidebar Settings stack — root layer + Appearance / Playback / Library panes (`src/components/Sidebar.svelte`, model from `src-tauri/src/menu.rs`). Mode: Operate. Live inspection through the devtools bridge + spectacle; measurements are from the running dev app.

## Design Health Score

| # | Heuristic | Score | Key Issue |
|---|-----------|-------|-----------|
| 1 | Visibility of System Status | 3 | Location is excellent (title + back ‹ + push direction), state is read back in labels ("Shuffle: Off", "Scanning…"). Gaps: the sliders never show their value; while scanning, Library rows only dim — the progress bar lives in EmptyState, not here. |
| 2 | Match System / Real World | 2 | "Add music folder…" (import staging) vs "Music folders…" (library roots) are 2 words apart and mean different things; "Full Rescan (rebuild)" is a parenthetical the user has to interpret. |
| 3 | User Control and Freedom | 3 | Escape pops one level, ✕ pops all, `s` toggles, back chevron on every pane. One-way door: touching Theme permanently destroys "system", the shipped default. |
| 4 | Consistency and Standards | 2 | `.theme` hovers with the accent wash (`--active`) while every row hovers neutral and reserves accent wash for press; it is also the only permanently filled + bordered control, so it reads as a text field. Control labels are 12px/400 — a tier DESIGN.md does not declare. |
| 5 | Error Prevention | 3 | Clamped, stepped ranges; lightness clamping protects Black/White accents; rows disable during a scan. No confirmation needed anywhere in the pane. |
| 6 | Recognition Rather Than Recall | 2 | "Theme: Dark" hides both that it cycles and that a third option exists; cycling rows ("EQ Preset: Flat") don't say they cycle; swatches show the raw hex while the applied accent is theme-clamped, so the promise ≠ the result. |
| 7 | Flexibility and Efficiency | 3 | `s` opens the stack, arrow keys walk the focused layer, presets for accent and EQ, focus-visible rings on rows. |
| 8 | Aesthetic and Minimalist Design | 2 | Five heterogeneous controls on one uniform 16px cadence, with the 12-dot accent block (the pane's highest element count, lowest-frequency setting) inside the first 250px — the user's own read: "crammed, tucked to the top". |
| 9 | Error Recognition and Recovery | 2 | Settings writes are `invoke(...).catch(() => {})` — a failed persist is invisible forever; there is no error surface at all in this pane, so nothing to diagnose. |
| 10 | Help and Documentation | 2 | About exists (name/version/description) but nothing contextual; only the accent presets carry a tooltip (`title`), on a surface where the risky items are theme and the two rescans. |
| **Total** | | **24/40** | **Acceptable — significant improvements needed before this reads as the rest of the app** |

n/a heuristics: none.

## Design Specificity Verdict

**LLM assessment:** The stack as a whole is authored, not category-interchangeable: the 3-layer drawer with a receding root, the one-row-language (name left / trailing glyph in the count column), wash-not-glow states and the KDE mirror are recognizably Songstress. The Appearance pane is the one place the app stops sounding like itself. Two bare native range inputs, a generic bordered button, a checkbox and a raw swatch grid is the dialect of every settings panel on every platform; meanwhile the PlayBar three zones away uses micro-badges ("ALL", "1", queue count), `tabular-nums` readouts and accent washes. The pane with the most numbers in it is the only pane that shows none, and the app's own Tabular Rule ("anything that changes while you watch is tabular") has zero instances here because there are no digits.

**Deterministic scan (`detect.mjs --json src/components/Sidebar.svelte`, exit 2):** 1 finding — `design-system-radius` advisory at `Sidebar.svelte:1030` (`border-radius: 5px` on the glass checkbox). This is a documented deliberate exception in DESIGN.md ("5px radius — deliberate, below the control tier for a 16px object"), so it is a false positive against the design system, not against the design. Nothing else: no contrast, tap-target, or state-coverage hits from the mechanical pass.

**Browser overlays:** injected `detect.js` into the live app (devtools bridge, live-server on :8400, since stopped; page reloaded afterward so no overlays remain). One console finding: `clipped-overflow-container — div.app clips a positioned child`. Also a false positive by intent: `.app { overflow: hidden }` + `border-radius: 14px` is the window clip that lets the wallpaper show in the corners.

## Overall Impression

The structure is right and the parts are mostly right; what's missing is articulation. The pane presents five decisions in one voice at one rhythm, so the eye reads a block instead of five choices — that is the "crammed" feeling, and it is not density (content stops 231px above the pane floor; 40% of the column is empty below). Two behavioral defects sit underneath the cosmetics and matter more than the layout: the theme control silently destroys its own default, and pushing a pane leaves keyboard focus in a now-`aria-hidden` layer.

Biggest opportunity: make the Appearance pane answer "what is it set to, and what are the other options?" for every control. That single question fixes the sliders, the theme control, and the accent row at once, and the rhythm fix comes almost free with it.

## What's Working

- **The drawer's spatial contract.** Root enters from the right, detail enters from the right, `‹` returns left, ✕ pops all three as one conveyor, and each layer's `aria-hidden` follows its active state. Location is never in doubt — heuristic 1 earns its 3 here.
- **The one-row language.** Menu rows, artist rows and pane rows share `--sidebar-row-size`, 13px, and the count column for trailing glyphs. It is why the sidebar reads as one list rather than three widgets.
- **Trailing-glyph state, not decoration.** ✓ for checked items, dim › for drill rows, a micro-badge rather than a glow on the PlayBar's mode buttons. State is encoded structurally, consistent with the One Accent Rule.

## Priority Issues

**[P1] "System" theme is a one-way door.**
`ui.theme` ships as `"system"`, but `.theme` calls `cycleTheme()`, which resolves system to a fixed value, and the Rust model carries only `theme_dark: bool` (`menu.rs:38,179`). The moment the control is touched, follow-the-OS is gone with no path back from any surface.
*Fix:* a 3-segment `Light | Dark | System` control in the Appearance pane, and a `theme: "light"|"dark"|"system"` string in the menu state so the Global Menu can offer it too.
*Suggested command:* `/impeccable clarify` (with the state model change).

**[P1] Keyboard focus is orphaned when a pane is pushed.**
Measured live after opening Appearance: `document.activeElement` is still the root-layer `.mrow`, whose layer is now `aria-hidden="true"`, and 5 focusable buttons remain inside it. Focus sits inside a hidden subtree (ARIA violation, no ring), and Tab walks the 4 covered root rows — Appearance, Playback, Library, About — before reaching a single control in the pane the user is looking at.
*Fix:* on `openDetail`/`back`, move focus to the incoming layer's first control (`.navtitle` container or the first control), and set `inert` on inactive layers so hidden controls leave the tab order.
*Suggested command:* `/impeccable audit`.

**[P1] The sliders never say what they are set to.**
"Tile size" (120–320) and "Sidebar rows" (28–52) show a track and nothing else: no value, no end labels. This is the pane whose entire job is choosing a number, chosen blind; and it is the only surface where the Tabular Rule has no instance.
*Fix:* label row becomes `name … value` right-aligned in the count column with `tabular-nums`, live on `oninput` (the same name-left/count-right column the artist rows use). Add `aria-valuetext` so it is announced.
*Suggested command:* `/impeccable layout`.

**[P2] Uniform cadence + a 12-dot block = the "crammed" read.**
`gap: 16px` everywhere, so the header→content gap equals slider→slider equals button→swatches; nothing marks where one decision ends. Cognitive-load checklist fails 4 of 8 (chunking: 5 ungrouped items; grouping: no proximity signal; minimal choices: 12 visible options at the accent decision; progressive disclosure: absent). 4+ failures is high load.
*Fix:* uneven rhythm (8px within a group, 28–32px between), Label-tier headers (11.5/600/+0.09em — the tier DESIGN.md reserves for exactly this and which the pane never uses), and the accent picker folded to a single row (current swatch + name) that pushes a 4th layer or discloses in place. No inset cards: a second translucent layer on the 0.7 chrome glass breaks both DESIGN.md's no-double-border rule and the "never stack translucent on translucent" material rule.
*Suggested command:* `/impeccable layout` then `/impeccable quieter`.

**[P2] The Theme button breaks the row language and the accent budget.**
`.theme:hover { background: var(--active) }` spends the accent wash on hover where every row hovers `--hover` and reserves accent wash for press/selection (One Accent Rule, ≤10% per screen), and its permanent `--hover` fill + 1px border at 29px tall makes it look like a disabled text field, not the pane's main action.
*Fix:* hover = `--hover`, press = `--active`, no resting fill; or replace it with the 3-segment control from P1, which removes the question.
*Suggested command:* `/impeccable polish`.

## Persona Red Flags

**Alex (Power User):** Presses `s`, arrow-keys to Appearance, Enter — then Tab has to clear 4 phantom rows before the first slider. Theme is one click but yields 2 states when 3 exist, and cycling is the only way to reach them. No way to restore "System" without editing the SQLite `settings` row by hand.

**Jordan (First-Timer):** In the Library pane, "Add music files…", "Add music folder…" and "Music folders…" sit three rows apart and all plausibly mean "add my music"; the first two stage an import, the third edits library roots. "Rescan Library" vs "Full Rescan (rebuild)" — the difference is invisible, and picking wrong costs a full re-read of 247 albums. "Theme: Dark" gives no hint that clicking changes anything.

**Sam (Accessibility-dependent):** Focus stays inside the `aria-hidden` pushed-out layer after opening a pane (WCAG 4.1.2 / ARIA: focus must not be inside `aria-hidden`); inactive layers are not `inert`, so 5 hidden controls stay in the tab order. Accent swatches have no `:focus-visible` rule at all (the outline channel is used by hover/selected) and the `<input type=color>` inside is `opacity: 0` — the picker is unreachable by keyboard. Contrast computed worst case over a white wallpaper under the 0.7 chrome tier: About footer text ≈3.5:1 and the `›` drill at `opacity: .6` on `--text-dim` ≈2.3:1 — both under AA (4.5:1 for text, 3:1 for meaningful graphics).

**Riley (Stress tester):** Pick the "Black" preset in the dark theme: the swatch is a black dot, the accent becomes light grey (`CLAMP` in `accent.ts:96` clamps dark-theme L to 0.7–0.8), and the custom rainbow swatch never shows the custom color you chose — the control's promise and its result disagree. Kill the app mid-`set_setting` and the DB silently keeps the old value; localStorage will repaint the old setting on next launch with no indication anything was lost.

## Minor Observations

- 12 swatches wrap 7+5 in the 236px column (20px + 7px gap fits 7) — a ragged pair of rows; 6 per row would be even, or a fold per P2.
- `.toggle { flex-direction: row !important }` exists only to fight the shared `.appearance label` rule; splitting that rule for the group restructure removes the `!important`.
- Detail layers have no footer, the root layer does (About). If a "Reset to defaults" row is ever added, that's where it goes, and the inconsistency resolves itself.
- The pane's only write path is fire-and-forget; a single failed toast-equivalent inside the glass would close heuristic 9's gap.
- `Full Rescan (rebuild)` is the one place in the sidebar where a parenthetical explains a button; rename per copy pass.

## Questions to Consider

- What if the Appearance pane answered "what is it set to?" for every row — would the pane still feel crowded at 4 rows with values instead of 5 rows of bars?
- Should the 12-dot accent picker exist at all in the primary column, given it is a once-a-year decision competing with a per-session decision (tile size)?
- Is the sidebar the right place for shuffle/repeat/EQ at all, now that the PlayBar owns those controls and encodes them differently (badge vs ✓)? What would the Playback pane have to become to justify its second encoding — or should it hold only what the PlayBar cannot (EQ presets, Customize)?
