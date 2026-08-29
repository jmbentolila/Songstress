---
target: playbar
total_score: 35
max_score: 40
na_heuristics: 
p0_count: 0
p1_count: 0
timestamp: 2026-08-29T19-05-05Z
slug: src-components-playbar-svelte
---
# Critique snapshot: src/components/PlayBar.svelte (degraded single-context)

## Heuristics (10 scored, none n/a; max 40)
1 Visibility of System Status: 4 — badges/titles encode every state
2 Match System/Real World: 3 — custom album-skip glyphs (tooltipped)
3 User Control and Freedom: 4 — Escape + outside-click dismissal, transport keys, toggle buttons
4 Consistency and Standards: 4 — dismissal now matches ContextMenu idiom; all radii/sizes on documented ladders
5 Error Prevention: 3 — clamped seeks/volume, mute icon flip, disabled-at-idle
6 Recognition Rather Than Recall: 3 — "ALL" badge jargon until hover
7 Flexibility and Efficiency: 4 — Space/←/→ in-app transport, MPRIS, accelerators
8 Aesthetic and Minimalist Design: 4
9 Error Recovery: 3
10 Help and Documentation: 3 — strong micro-help, hover-only
TOTAL 35/40 (Good, 87.5%) — previous 30/40

## Detector: 0 findings, exit 0 (was 9 advisories)
8px / 8.5px instrument-tier sizes registered as documented exceptions
(.impeccable/config.json, same route as the sidebar colorMeta
exceptions); the two 16px close glyphs fixed for real (→ 14px body,
on-ramp, reads closer to the 12px preset select beside them).

## Verified fixed since last run (30 → 35)
- P2 popover mutual exclusion (EQ/queue no longer overlap at the shared anchor)
- P2 standard dismissal: Escape + outside-click, ContextMenu idiom;
  toggle-button pointerdown exempted (open-only bug avoided)
- P2 volume mute target 19px → 28px
- P2 in-app keyboard transport: Space = play/pause (repeat-guarded),
  ←/→ = seek ±5s (repeat on purpose); inert on any interactive focus
  target; advertised via title tooltips
- P3 Instrument type tier documented in DESIGN.md (8–9.5px, EQ readouts
  + mode badges only); EQ preset select 6px → 8px (control rung)

## Priority issues (all P3)
P3 "ALL" shuffle badge is unexplained jargon until hover. Fix: hover
title already exists; either accept or shorten (e.g. "All"). -> /impeccable clarify
P3 Popovers pop in instantly — no materialize/entrance (apple-design:
materialize, don't just fade; anchor to trigger, transform-origin
bottom-right). Fix: ~120–160ms scale 0.98→1 + opacity, standard curve,
reduced-motion off-switch. -> /impeccable animate
P3 No clear-queue-all (per-row × only; power-user red flag). Fix: small
"Clear" text button in the queue popover header when non-empty. -> /impeccable clarify

## Personas
Alex: keyboard transport landed (Space/←/→, advertised); remaining gaps
are clear-all + no queue reorder (reorder is by design, not scope).
Sam: Escape dismissal + role=dialog + 28px mute target landed;
hover-only tooltips are her residual blind spot (WebKitGTK title
tooltips are spotty for SR).
Riley: empty states honest and actionable; arrows while stopped no-op
safely; Space while stopped plays first album — same as the button.

## Strengths
Status density (badges not glows); honest transport (wash play button,
disabled-at-idle 0.35, album-pair as extension); micro-help that
teaches affordances ("←/→ = ±5s", "double-click to zero"); dismissal
contract now uniform across the app.
