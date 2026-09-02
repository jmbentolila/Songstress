---
target: the imported-music modal (ManageImports)
total_score: 30
max_score: 40
na_heuristics: 
p0_count: 0
p1_count: 3
timestamp: 2026-09-02T00-40-17Z
slug: src-components-manageimports-svelte
---
Target: src/components/ManageImports.svelte ("Imported music" modal), assessed loaded with a mock pile:
5 artist cards / 8 albums / 53 tracks, incl. a 50-char album title, diacritics (Björk, Skeletá), a
merge, a byte-identical duplicate, an already-indexed file, marked states, an Apply receipt, and the
legacy Infestissumam pile. Method: DEGRADED single-context (no sub-agent tool exposed) — Assessment A
(design review, live webview via the devtools bridge + spectacle) then Assessment B (detect.mjs).

Heuristic scores (0-4): 1) Visibility 3  2) Real world 3  3) Control & freedom 4  4) Consistency 2
5) Error prevention 4  6) Recognition 3  7) Flexibility 3  8) Aesthetic/minimal 3  9) Error recovery 3
10) Help 2   TOTAL 30/40 (Good). No n/a heuristics.

Cognitive load: 2 checklist failures (hierarchy inversion between the receipt and the "already" block;
working-memory demand — both key messages require scrolling, and the receipt does not name its own
operation). Decision points stay <=2 options per row; the screen carries 19 buttons total.

P1 issues
1. The two messages this window exists to deliver are below the fold. "Already in your library" and the
   Apply receipt render AFTER the album cards inside the scroller. Measured: body clientHeight 389px vs
   scrollHeight 864px = 475px hidden; the held block's top is off-screen on open. PLAN.md's own rule
   ("import reports what it did, in the window the import opens"; "a ring that ends with the screen
   unchanged reads as a failure") is broken by placement, not copy. Fix: take both out of the scroller —
   a report band directly under the head, and/or pin the receipt into the footer line that currently says
   "Mark an album to apply" (the footer is already the what-just-happened/what-happens-next slot).
2. The destination — the reason the window exists — truncates first. Long album title renders
   "merges into Rite Here Rite Now (Original Motion Picture Soundtrack) · M…": the path collapses to one
   character, and the full path is only in a `title` tooltip (mouse-only). Fix: let the rule (the gloss)
   truncate before the path (the evidence); allow the destination to wrap to a second line; expose the
   full path on keyboard focus, not hover.
3. `aria-modal="true"` with no containment. Nothing sets `inert`/`aria-hidden` on the background, so Tab
   walks the sidebar, grid and playbar behind the scrim — the same failure the settings stack was fixed
   for. Fix: inert the stage while any surface is open (one place: App.svelte, driven by surfaceOpen())
   plus a Tab ring inside the dialog.

P2 issues
4. Palette/type drift (detector: 5 findings). `#ff8f8f` + 2 alphas as the destructive tint is a second
   hue DESIGN.md does not have (its only non-accent is Amber Caution #f2a33c, reserved for missing-file
   glyphs); MusicFolders spends the same red on "Remove", so this is a system decision, not a local bug.
   `border-radius: 4px` on the focus ring is off the shapes ladder (7/8/10/12). `.mi-path` is set in
   SFMono/Consolas/Menlo while DESIGN.md's typography says "Label/Mono Font: none". Decide once: either
   promote a documented Destructive tier (used in both components) or express the mark without a new hue;
   focus radius to 8; either amend DESIGN.md to allow mono for filesystem paths or set the path in Inter.
5. "ALREADY IN YOUR LIBRARY" is styled as an artist: it reuses `.mi-glabel` (11.5px, +0.09em, uppercase,
   0.64 alpha) and sits directly under the last artist card, so the first read is an artist by that name.
   Give the report band its own voice (sentence case, no tracking).

Minor
- Partial imports show non-contiguous track numbers (1,3,4,5,7,8) with no explanation — reads as data
  loss; say "6 of 8 tracks" or mark the gap.
- Overlay scrollbar (11px, themed) paints over the cards' right edge (measured: card right == body right,
  offsetWidth-clientWidth == 0). `scrollbar-gutter: stable`, or 4px padding-right.
- With a White accent, three solid-accent pills compete (2 marked Saves + Apply). Consider the mark taking
  the accent WASH and reserving solid for the one verb that commits.
- `saveImports(albumId, trackId)` still supports per-track scope in store+backend; this window only marks
  per album. Expose it or delete it.

Working
- Mark-then-apply with reversible marks and a footer that states the FILES ("2 albums to save · 1 album to
  discard") rather than the UI.
- Destination in plain words per row, resolved by the same cascade the save follows.
- Error prevention: byte-identical duplicate rule, never staging what the library holds, never removing a
  folder it did not write, Apply disabled until marked, receipt naming which file went and which stayed.

Peak-end: the peak is Apply (files move, one of them possibly deleted); the end of that moment is a
sentence below the fold. The highest-stakes message in the app is the least likely to be read.

Detector vs review: detector caught the 4px radius and the mono face before the review did; the red it
reported three times is one decision. No false positives material.
