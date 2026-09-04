---
target: the expanded album panel (ExpandedPanel)
total_score: 30
max_score: 40
na_heuristics: 
p0_count: 0
p1_count: 1
timestamp: 2026-09-04T00-05-34Z
slug: src-components-expandedpanel-svelte
---
# Critique — ExpandedPanel (album row) — 2026-09-04

⚠ DEGRADED: single-context (no sub-agent tool exposed; A inline, then B isolated).
Score 30/40 (Good). n/a: none. P0:0 P1:1 P2:3 P3:1.

Heuristics: 1:3 2:3 3:3 4:2 5:3 6:3 7:4 8:3 9:3 10:3.
Cognitive load: 1 checklist failure (state legibility). Chunking exemplary.

Specificity: authored — height state machine (retargeted transitions, ghost
hand-off, margin-synced slot), artwork gradient 0.36/0.30, playback verbs by
commitment order. Detector: 0 findings, exit 0. No overlay (no browser-automation
harness; devctl DOM probes as evidence channel).

P1 focus-ring family missing here: .track/.staged-badge/.edit-album/.play-all
have no :focus-visible rule; UA generic ring observed live on a focused row —
two focus dialects on one screen vs sidebar/import/tag-editor accent rings.
Fix: one :is(...) outline rule joining the family.

P2 Imported badge: button since 0.7 but pill geometry ~20px < 28px control bar
(PRODUCT.md). Give it vertical padding; keep the pill.

P2 Missing-file alert hardcodes #f2a33c while --caution exists (same value).
Use the token.

P2 State collision: .track:active and .track.selected share --active; press on
a selected row is invisible, press vs selection indistinguishable. Keep neutral
selection, give .selected a 1px inset --border ring; press stays the wash.

P3 Header comment still says "expand, scroll, flash" — flash removed 0.9.3.

Personas: Sam — focus dialect change on the app's primary content; Alex — 20px
pill beside 28px pencil, misclick; Jordan — clean, tooltip model.

Minor: disc/plain 2-col thresholds differ (deliberate, commented); ❚❚ glyph
font-dependent; footer can sit under playbar veil at 1200x660 (fine).

Question: header cluster = three controls, three weights — play as the only
accent thing, badge/pencil deliberately quieter siblings?
