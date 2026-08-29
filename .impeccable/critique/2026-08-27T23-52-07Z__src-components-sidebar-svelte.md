---
target: sidebar
total_score: 25
max_score: 36
na_heuristics: 10
p0_count: 0
p1_count: 1
timestamp: 2026-08-27T23-52-07Z
slug: src-components-sidebar-svelte
---
# Critique snapshot: src/components/Sidebar.svelte (degraded single-context run)

## Heuristics (9 scored, #10 Help n/a; max 36)
1 Visibility of System Status: 3 — no populated-library scanning indicator in sidebar
2 Match System/Real World: 3 — gear="Menu" breaks trained gear=settings expectation
3 User Control and Freedom: 3 — three menu-stack exits; zero-match search has no clear affordance
4 Consistency and Standards: 3 — unified state language; diacritic mismatch list vs grid; 6px radius drift
5 Error Prevention: 3 — zero-match state exists, offers no remedy
6 Recognition Rather Than Recall: 2 — Appearance one push behind five opaque rows; search-clear not visible
7 Flexibility and Efficiency: 2 — no keyboard path to search/menu
8 Aesthetic and Minimalist Design: 4 — one decision per layer
9 Error Recovery: 2 — dead-end empty state; split diacritic behavior
10 Help: n/a
TOTAL 25/36 (Acceptable, 69%)

## Priority issues
P1 Search dead end at zero matches: "All Artists" row filtered out with the list; no clear (x) in field. Fix: always-visible All Artists + clear button (old titlebar pattern). -> /impeccable clarify
P2 Two search surfaces, two rules: sidebar toLowerCase().includes vs grid fold() (diacritics). "bjork" split result. Fix: fold() the artist filter. -> /impeccable harden
P2 Settings discoverability regression: gear was Appearance, now "Menu" with Appearance one push in. Options: accept / dual affordance. -> /impeccable shape
P2 A11y: sidebar search input lacks aria-label (placeholder-only). -> /impeccable audit
P3 Drift: .backchev 6px -> 7px; document traffic-light border rgba(0,0,0,0.18) + conic swatch colors as sidecar exceptions. -> /impeccable polish

## Detector (6 advisories, all design-system-* drift)
rgba(0,0,0,0.18) traffic border [legit]; 6px backchev radius [drift]; #e5484d #ffb224 #46a758 #00a2c7 conic swatch [legit rainbow picker]

## Personas
Alex: no keyboard shortcuts (search focus / menu open); rescan two pushes deep.
Sam: focus rings + targets good; fails on unlabeled search input and zero-match dead end (no visible control in nav region).
Riley: "bjork" vs "björk" split between list and grid; stack held up under interrupt/escape cases.

## Cognitive load
1 checklist failure (working memory: gear=menu-vs-settings recall). Menu root = 5 options, at the boundary.

## Strengths
Unified search (one query -> list + grid sections); menu-stack wayfinding (titled layers, dual exits, conveyor pop, one state language); passive status (counts/active-wash, no noise).
