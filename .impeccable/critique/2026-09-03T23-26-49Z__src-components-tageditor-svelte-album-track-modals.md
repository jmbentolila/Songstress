---
target: Edit tags modals (album + track)
total_score: 34
max_score: 40
na_heuristics: 
p0_count: 0
p1_count: 0
timestamp: 2026-09-03T23-26-49Z
slug: src-components-tageditor-svelte-album-track-modals
---
Method: degraded single-context (no sub-agent tool exposed). Second run on the Edit tags surface (album + track modals) after the owner-priority fix batch.

Heuristics: 1) Visibility 3 — unchanged strong; new: auto-open fold was falsely springing open (found & fixed this run). 2) Real world 4 — footer caption ("13 files associated to this album") in the identity slot; orphan summary gone. 3) Control/freedom 3 — unchanged. 4) Consistency 4 — topline, fold, footer caption slot, hint copy now shared language in both modals; one sheet (TagSurface) owns the family. 5) Error prevention 3. 6) Recognition 3. 7) Flexibility 3 — Enter=Save both modals, ←/→ stepper verified live (1/13→2/13); no bulk-in-track (by design). 8) Minimalism 4 — album modal lost four always-empty rows to the fold; whole album editor fits 660px without scroll. 9) Error recovery 3. 10) Help 4 — the promise line is always visible in both doors; hints tightened.
Total: 34/40 (Good, upper band).

Cognitive load: 0–1 checklist failures (low). Album mode's seven same-weight rows collapsed to three + fold; reassurance up front.

Specificity: even more authored; detector 0 findings again (exit 0). Browser evidence: activation screenshots of both modals at the owner's 1200x660 + DOM probes.

Fixed since run 1 (29/40): P1 promise below fold (topline both modals), P2 orphan caption (footer caption, blast takes the slot while dirty), P2 keyboard accelerators (quickKey on the shared surface; typing rules), P3 hint copy, Sam focus-restore (opener capture/restore, guarded). Declined by owner: always-shown Disc of-pair (stable geometry), 11.5px dim contrast.

New issue found and fixed in-run: [P2] album fold auto-open fired on every album — conflicts_for() returns a census entry per field (presence included), the condition treated membership as dispute; now keyed on the disputed flag, same criterion that lights chips. Verified: DAWN (no hidden dispute) stays folded.

Remaining watch: P4 selection two-color (dot KWin-green, rings/lightbox accent) — deliberate. P3 optional: "Written ✓" could name what was written (title/genre…) — probably noise. Track modal album-switch while artLb lightbox open — unprobed edge.

Questions: whether to commit 0.9.0 now; whether the receipt should list per-field writes; whether album modal needs its own lightbox-Esc edge test.
