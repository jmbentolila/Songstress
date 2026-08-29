---
target: album grid + expansion panel
total_score: 32
max_score: 40
na_heuristics: 
p0_count: 0
p1_count: 0
timestamp: 2026-08-29T19-41-14Z
slug: src-components-albumgrid-svelte
---
# Critique snapshot: album grid + expansion panel (degraded single-context)

## Heuristics (10 scored, none n/a; max 40)
1 Visibility of System Status: 4 — rings/badges/alerts encode every state
2 Match System / Real World: 3 — double-click-to-play undiscovered
3 User Control and Freedom: 3
4 Consistency and Standards: 3 — tiles + track rows skip the documented :active wash, cursor:default
5 Error Prevention: 3
6 Recognition Rather Than Recall: 3 — single-click selection has no consequence or hint
7 Flexibility and Efficiency: 3 — rich context menus; no tile keyboard roving
8 Aesthetic and Minimalist Design: 4
9 Error Recovery: 3 — grid search empty state inert
10 Help and Documentation: 3 — strong microcopy; play affordance unadvertised
TOTAL 32/40 (Good, 80%)

## Detector: 0 findings, exit 0
One warning found and resolved as documented exception: `transition: height`
on .inner is the WebKitGTK dual-clock fix (expand/collapse keeps the eased
height; the single-step constraint is the album-SWITCH path). Registered
file-scoped: layout-transition=* for src/components/ExpandedPanel.svelte.

## Priority issues
P2 Tiles + track rows skip the documented press convention (no :active
wash, cursor:default) while the rest of the chrome follows it. -> polish
P2 Selection without consequence: single-click washes a track, then it
does nothing; play path (double-click) hinted only on missing rows;
play-all icon-only without title. -> shape (user decision)
P2 Grid search empty state is an inert sentence; sidebar's is an action.
-> clarify
P3 .edit-album 26px target -> 28px bar. -> polish
P3 a11y: alert SVG + ▶/❚❚ glyphs need aria-hidden + sr-only "now
playing". -> audit

## Personas
Alex: right-click paths exemplary; no arrow-key roving; Space correctly
leaves focused tiles to native activation.
Riley: long titles / 2-col / multi-disc / missing / staged all hold;
zero-track album = degenerate empty panel ("0 tracks · 0:00").
Sam: best a11y surface after sidebar; only the two P3 glyph nits.

## Strengths
Row-model grid (expansion IS a row); art-tinted panel at tuned alpha;
Songs/Albums search sectioning; back-cover two-column tracklist (also a
WebKit hit-testing fix); forgiving failure paths (locate / bulk-remove /
staged save+discard).
