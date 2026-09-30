---
target: playbar waveform prototype
total_score: 32
max_score: 40
na_heuristics: 
p0_count: 0
p1_count: 2
timestamp: 2026-09-28T03-35-21Z
slug: src-components-playbar-svelte
---
# Critique: src/components/PlayBar.svelte — waveform prototype

Mode: Operate. Brief pin: the waveform stays; this reviews its expression.

## Heuristics (10 scored, none n/a; max 40)
1 Visibility of System Status: 3 — position by hue only; no playhead; pending-peaks state undesigned
2 Match System/Real World: 4 — waveform is the most faithful real-world picture of a track
3 User Control and Freedom: 4 — seeking, ±5s, drag, native escape intact
4 Consistency and Standards: 3 — focus ring off house inset style; 6px radius off-scale (detector)
5 Error Prevention: 3 — clamped seeks, disabled at idle
6 Recognition Rather Than Recall: 3 — lane draggability signalled only by tooltip
7 Flexibility and Efficiency: 4 — keyboard/MPRIS/±5s retained
8 Aesthetic and Minimalist Design: 2 — permanent 32px full-width amplitude texture out-weighs the transport in a dim-until-touched bar
9 Error Recovery: 3 — unchanged
10 Help and Documentation: 3 — hover-only tooltip
TOTAL 32/40 (Good, 80%) — previous 35/40 (2026-08-29). Drop is items 1 and 8 (expression, not architecture).

## Detector
1 finding, exit 2: .wave-input:focus-visible border-radius 6px (line 944), design-system-radius, advisory. True positive (scale = 7px icon / 8px control).

## Design specificity
Idea is category-interchangeable (any player could wear a waveform unchanged); execution is authored — house tokens only, glass shelf, dim-until-touched grammar. Risk: first playbar element that is neither system-mirrored nor collection identity.

## Strengths
- Native <input type=range> kept as transparent control layer: keyboard/ARIA/drag/←→/tooltip inherited, ~no a11y regression.
- Token-only color (--accent played, --text-dim unplayed): light/dark + accent picker flow through.
- Cheap progress: bars/path built once per track id; only clip-rect width moves at 10 Hz.
- Explicit containment: min(32px, calc(--playbar-h - 58px)) makes transport collision impossible.

## Priority issues
P1 Position conveyed by color alone — no playhead. Over a varying silhouette the accent/dim boundary stops reading as a boundary; fails color-alone (WCAG/Sam). Fix: persistent 2px full-lane playhead at the boundary. [$impeccable polish]
P1 Waveform weight fights "the collection is the hero". Unplayed bars at text-dim×0.4, 32px tall, full width, in every state incl. stopped. Fix: drop unplayed tier to near-texture; consider drawing only with a track loaded; keep played portion full accent so progress is the loud thing. [$impeccable quieter]
P2 Control with no affordance — no hover/press feedback on a draggable lane (Apple §1). Fix: hover lifts playhead/contrast; :active takes --active wash. [$impeccable animate / polish]
P2 Pending + idle states undesigned. Real pipeline has a no-peaks moment; stopped draws a 0.06 flat line that reads as broken. Fix: defined loading lane + calm idle lane. [$impeccable harden]
P3 Off-scale radius + off-house focus ring. Fix: border-radius: var(--radius-control); inset ring (outline-offset: -2px). [$impeccable polish]

## Persona red flags
Alex (Power User): control intact, but waveform is pure decoration to him and costs a grid row at 660px.
Sam (Accessibility): color-only position; no aria-valuetext ("1:23 of 4:56"); unplayed text-dim×0.4 likely under 3:1; light theme inverts text-dim to dark ink → dark bars on light chrome, heavier than intended. Waveform correctly aria-hidden; focus ring visible but off-house.
Riley (Stress): idle 0.06 flat lane looks like a render failure; retag mid-play swaps trackId → silhouette changes with no transition.

## Minor observations
- No reduced-motion handling needed (lane doesn't animate) — a quiet win; keep true if a playhead transition is added.
- Static mirrored waveform can read as a live meter for the first second; the playhead also fixes this.
- Elapsed + total only; no remaining-time option.
- 80 bars ≈ 3.6px at ~460px — good definition, no change needed.

## Questions to consider
- If unplayed bars became pure texture and only the played portion kept color, is that more or less waveform?
- What does this look like in light theme, where --text-dim inverts to ink? Untested half.
- Does the waveform belong at idle at all, or should the lane only materialize with a track?
