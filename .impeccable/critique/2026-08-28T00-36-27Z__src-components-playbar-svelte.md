---
target: playbar
total_score: 30
max_score: 40
na_heuristics: 
p0_count: 0
p1_count: 0
timestamp: 2026-08-28T00-36-27Z
slug: src-components-playbar-svelte
---
# Critique snapshot: src/components/PlayBar.svelte (degraded single-context)

## Heuristics (10 scored, none n/a; max 40)
1 Visibility of System Status: 4 — badges/titles encode every state
2 Match System/Real World: 3 — custom album-skip glyphs (tooltipped)
3 User Control and Freedom: 2 — popovers: no Escape, no outside-click
4 Consistency and Standards: 3 — dismissal contract contradicts ContextMenu/titlebar-menu idioms
5 Error Prevention: 3
6 Recognition Rather Than Recall: 3 — "ALL" badge jargon until hover
7 Flexibility and Efficiency: 2 — no in-app keyboard transport; no clear-queue
8 Aesthetic and Minimalist Design: 4
9 Error Recovery: 3
10 Help and Documentation: 3 — strong micro-help, hover-only
TOTAL 30/40 (Good, 75%)

## Detector: 9 advisories, all drift — EQ instrument tier 8-10px below documented 11px micro; select 6px radius; 16px close glyph

## Priority issues
P2 EQ + queue popovers share anchor (right:14px bottom:calc(100%+10px)) and toggle independently -> overlap. Fix: mutual exclusion. -> /impeccable harden
P2 No standard popover dismissal (Escape/outside-click); keyboard users tab through 12 sliders to the ×. -> /impeccable harden
P2 Volume mute target ~19px < 28px bar. -> /impeccable polish
P2 No in-app keyboard transport. Proposed: Space=play/pause, arrows=seek ±5s, inert while typing/slider focused. -> /impeccable shape
P3 Instrument type drift: document named "Instrument" tier (8-10px, EQ/badges only) in DESIGN.md; select 6px->8px. -> /impeccable document

## Personas
Alex: no clear-queue, no keyboard transport; media keys OK via MPRIS.
Sam: no-Escape popover = tab through 12 sliders to close; otherwise exemplary labels/tooltips.
Riley: EQ+Queue overlap; mute target; "ALL" badge.

## Strengths
Status density (badges not glows); honest transport (wash play button, disabled-at-idle 0.35, album-pair as extension); micro-help that teaches affordances ("double-click to zero").
