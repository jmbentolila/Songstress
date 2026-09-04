---
target: sidebar + settings panes
total_score: 32
max_score: 40
na_heuristics: 
p0_count: 0
p1_count: 1
timestamp: 2026-09-04T00-32-10Z
slug: src-components-sidebar-svelte
---
# Re-run 2026-09-04 (degraded single-context; detector + live settings-pane probe)
24 -> 32/40 (Good). Detector: 0 findings (the old 5px checkbox advisory is
gone — tokenized). Live probe: 12 pane controls, 0 unlabeled; 17
focus-visible rules, accent family throughout.
Delta since baseline: theme is a three-segment control with SYSTEM RESTORED
(the one-way door is gone), sliders show values (.val readouts, tabular),
theme hover neutral like the family, focus rings everywhere, copy pass
 landed. H3 4, H4 4, H6 3, H2 3.
NEW TOP FINDING (only real one): settings persistence is still
invoke("set_setting").catch(() => {}) — a failed write is invisible
forever; the announcer exists and is not wired here. P2, one-liner.
Deliberate, not counted: KDE mirror of the drawer, alpha values, the .app
clip advisory (window corner contract).
