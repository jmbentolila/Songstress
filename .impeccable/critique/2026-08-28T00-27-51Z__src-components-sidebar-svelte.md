---
target: sidebar
total_score: 33
max_score: 36
na_heuristics: 10
p0_count: 0
p1_count: 0
timestamp: 2026-08-28T00-27-51Z
slug: src-components-sidebar-svelte
---
# Re-critique snapshot: src/components/Sidebar.svelte (degraded single-context)

## Heuristics (9 scored, #10 Help n/a; max 36)
1 Visibility of System Status: 3 — no populated-library scanning indicator
2 Match System/Real World: 4 (up) — gear="Settings" restores convention
3 User Control and Freedom: 4 (up) — clear-×, All Artists always, Escape dual-duty, /, s
4 Consistency and Standards: 4 (up) — one navrow pattern, one matching rule, on-scale radii, detector clean
5 Error Prevention: 3 — remedies exist; the note itself is inert
6 Recognition Rather Than Recall: 3 (up) — accelerators invisible
7 Flexibility and Efficiency: 3 (up) — real accelerators, no hints
8 Aesthetic and Minimalist Design: 4
9 Error Recovery: 3 (up) — zero-match no longer a dead end
10 Help: n/a
TOTAL 33/36 (Good, 92%) — previous 25/36

## Detector: 0 findings, exit 0 (was 6 advisories)

## Priority issues (all P3)
P3 Accelerators undiscoverable: / and s unadvertised. Fix: title-attribute hover hints (keep root at 5 rows). -> /impeccable clarify
P3 "No artists match." inert: make it an action ("Clear search"). -> /impeccable clarify
P3 No scanning feedback in populated sidebar: 12px dim "Updating library…" under field while scanner.running. -> /impeccable polish

## Personas
Alex: finds / by habit, never s without a hint. Sam: no red flags remain (best surface for her). Riley: hint absence + new tooltip surface are her regression vectors.

## Verified fixed since last run
P1 search dead end (All Artists + clear-×); P2 diacritics (fold()); P2 aria-label; P2 Settings rename; P3 radius 7px; P3 sidecar exceptions; navrow divider consistency; search focus = accent stroke.
