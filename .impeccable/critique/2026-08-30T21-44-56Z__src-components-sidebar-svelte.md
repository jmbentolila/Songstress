---
target: settings menu tree (sidebar)
total_score: 25
max_score: 40
na_heuristics: 
p0_count: 0
p1_count: 2
timestamp: 2026-08-30T21-44-56Z
slug: src-components-sidebar-svelte
---
# Critique: settings menu tree (sidebar stack) — src/components/Sidebar.svelte

Method: DEGRADED single-context (no sub-agent tool exposed). A: source + live
inspection (devtools bridge state-verified layer walks + active-window captures
of all 5 layers). B: detect.mjs --json → 0 findings (exit 0). No injectable
overlay (no harness browser automation); fallback signal = live captures.

## Heuristics (Operate mode, all 10 apply)

| # | Heuristic | Score | Key issue |
|---|-----------|-------|-----------|
| 1 | Visibility of System Status | 3 | Inline state labels excellent; dead "About" reads enabled |
| 2 | Match System / Real World | 3 | "Add music folder…" vs "Music folders…" near-identical, different actions |
| 3 | User Control and Freedom | 4 | back + ✕ conveyor pop + Escape + `s` — never trapped |
| 4 | Consistency and Standards | 2 | three row dialects in one container; "Equalizer" vs "EQ"; theme in two places |
| 5 | Error Prevention | 2 | no-op About; adjacent folder-label mis-click pair; always-visible disabled "Save imported music" |
| 6 | Recognition Rather Than Recall | 2 | nothing signals root rows drill down; empty check-column indent noise |
| 7 | Flexibility and Efficiency | 3 | s/Escape// accelerators; no arrow-key nav in stack |
| 8 | Aesthetic and Minimalist Design | 2 | two degenerate panes, dead transport wall, OS checkbox on glass |
| 9 | Error Recovery | 3 | no real error paths; scan-state disables correctly |
| 10 | Help and Documentation | 1 | Help pane is 100% decorative (no-op About) |
| **Total** | | **25/40** | Acceptable (62.5%, top of band) |

Cognitive load: 5/8 checklist fails (chunking, grouping, visual hierarchy within
panes, minimal choices at root, progressive disclosure) → HIGH; chunking/
grouping/disclosure share one root cause: the tree is the Rust global menu bar
rendered verbatim.

## Design specificity

LLM: the stack's motion and shell are product-specific (conveyor pop, 320ms
drawer, home/search integration), but the TREE is category-interchangeable —
Playback/Library/View/Help is menu.rs rendered as-is with Appearance grafted.
Could sit unchanged in any Tauri app. Sidebar-specific IA unexplored.
Detector: 0 findings (structural issues are below its mechanical reach).

## What's working
1. Exit architecture (score 4): back chevron per level, ✕ pops the whole stack
   as one conveyor motion, Escape pops, `s` toggles.
2. Inline state labels: "Shuffle: Off", "EQ Preset: Rock", "✓ Equalizer: On" —
   state visible without clicking.
3. Appearance pane proves the target model: real controls in the sidebar's own
   language (sliders, washed theme button, swatch grid).

## Priority issues
1. [P1] Tree is the global menu bar, not a sidebar IA. View = 1 item that
   duplicates Appearance's Theme button; Help = 1 item that is a no-op; root =
   5 rows, 2 leading nowhere. Fix: sidebar-shaped tree — fold theme into
   Appearance (drop View pane), move About out of the nav (quiet footer row),
   leaving 3 real panes. /impeccable distill
2. [P1] Dead control that reads enabled ("About Songstress" no-op, frontend
   handler empty, Rust doesn't handle it) + duplicated theme control. Fix: make
   About real (in-glass popover — native dialogs banned) or remove from the
   sidebar tree; single theme home in Appearance. /impeccable clarify
3. [P2] No drill-down affordance on root rows (plain text; standard convention
   = chevron). Fix: dimmed › chevron right of each mrow, utility family.
   /impeccable layout
4. [P2] Three row dialects in one container + empty 14px check column indents
   irow labels away from the pane title. Fix: unify menu rows to the sidebar
   row language (label column aligned with artist rows; check as right-side
   state glyph or only where state exists). /impeccable layout
5. [P2] OS-default controls on glass: the "Playbar artwork gradient" checkbox
   is the only non-glass control in the app; slider chrome is native. Fix:
   custom 16px rounded-square checkbox (accent fill, white check, inset focus
   ring); styled range thumb. /impeccable polish

## Minor
- "EQ Preset" vs "Equalizer" terminology mix in one pane.
- "Add music folder…" (import) vs "Music folders…" (roots): adjacent
  near-identical labels, real mis-click risk (PLAN.md already flags it).
- "Save imported music" visible-but-always-disabled — hide when nothing staged.
- 6 transport rows duplicate the PlayBar; arguably global-menu-only content.
- No arrow-key navigation within the open stack (Tab works).
- Disabled-text contrast on glass borderline (dim on dim) — check when restyling.

## Persona red flags
- Alex (power): `s`/Escape good; no arrow nav; About click = silence; 6 dead
  rows at top of Playback while idle.
- Jordan (first-timer): no chevron → doesn't know rows drill down; About
  expecting info gets nothing; two near-identical "…folder…" rows.
- Sam (a11y): tab order + focus rings work; disabled rows unfocusable (good);
  check glyph conveys state without color; swatches labeled. Contrast of
  disabled text is the one real flag.
