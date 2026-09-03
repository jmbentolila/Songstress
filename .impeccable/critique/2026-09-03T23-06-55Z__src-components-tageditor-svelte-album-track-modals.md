---
target: Edit tags modals (album + track)
total_score: 29
max_score: 40
na_heuristics: 
p0_count: 0
p1_count: 1
timestamp: 2026-09-03T23-06-55Z
slug: src-components-tageditor-svelte-album-track-modals
---
Method: degraded single-context (no sub-agent tool exposed). Target: TagEditor surface (album + track modals), TagSurface/FieldGrid/ArtSelector included.

Heuristics (0-4): 1) Visibility 3 — save-disabled-until-dirty, "Written ✓", accordion filled/edited ledger, removed-ghost state; error string only reaches the footer path slot. 2) Real world 3 — plain tag vocabulary throughout; the floating "13 files" caption is the one orphaned fragment. 3) Control/freedom 3 — Cancel/Esc, dirty-blocked stepper with reason, artwork removal reversible until Save. 4) Consistency 3 — one shared surface/corridor/caption system; track footer identity vs album's bare footer is the one asymmetry. 5) Error prevention 3 — numeric inputmodes, dirty-block, unedited-fields-not-written contract. 6) Recognition 3 — datalist, count-chips, tooltips. 7) Flexibility 2 — no Enter-to-save, no arrow-key stepper for the 13-file sequential path. 8) Minimalism 3 — accordion on track mode; album mode carries 5 equal-weight mostly-empty rows. 9) Error recovery 3 — caution-hue messages in place, edits preserved. 10) Help 3 — contextual one-liners, but the reassurance line sits below the fold on the album modal at 660px.
Total: 29/40 (Good).

Cognitive load: 2 failures (album reassurance copy below fold; 7 same-weight field rows + artwork column competing at first glance). Moderate.

Specificity: strongly authored — the KWin-green selection dot, bidi-safe identity footer, census chips, and the measured Year/Disc column continuity are not category-interchangeable. Detector: 0 findings across the five component files (exit 0). Browser overlay injection skipped: the only page access is the devctl eval bridge (no tab automation); evidence came from DOM probes + activation screenshots instead.

Priority issues:
P1 Album modal hides its safety promise: "Fields you do not edit are not written" is below the fold at the owner's 1200x660 window — the bulk surface scrolls its reassurance away exactly when stakes are highest (13 files). Fix: move it into the header subtitle zone or the footer caption slot. Command: layout.
P2 "13 files" orphan caption: floats top-left of the album body, duplicates the tile's own "in 13 files", belongs to neither column. Fix: fold into the header ("Edit album tags — DAWN · 13 files") or drop it. Command: clarify.
P3 No keyboard accelerators on the track modal: Enter=Save, Left/Right = stepper when clean. The stepper exists precisely for walking N files; walking it with the mouse is the long path. Command: harden.
P3 Hint copy "Click an image to see it fully" reads off; wraps to 3 ragged lines in the 190px column. Fix: "Click to enlarge · the dot marks the cover". Command: clarify.
P4 (watch) Selection speaks two colors: dot = KWin green, lightbox "Use as cover" + focus rings = accent. Deliberate today; revisit if it ever reads as inconsistency.

Persona red flags:
Alex (power user): no Enter-to-save, no arrow-key stepper; otherwise the cleanest path in the app.
Sam (a11y): 11.5px dim captions for label-column and hints are borderline contrast at arm's length; verify focus trap + focus restoration on close (not probed this run).
Riley (stress): single-disc albums always render "Disc # [ ] of [ ]" noise; empty-of pair is dead furniture on most files.

Minor: album footer lacks the identity caption its track twin has (justified — albums have no path, but a "DAWN · Aldaria · 2015" caption would cost nothing); "Written ✓" disappears on next edit (correct); accordion accounting is excellent state design.

Questions to consider: Does the album modal need Label/Grouping/Composer/Comment visible on first open, or would the same census-promise reasoning allow a fold once the census proves them empty across the library? Should the track stepper announce "unsaved changes" as an inline nudge rather than a disabled button?
