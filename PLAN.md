# PLAN.md — Songstress: state, decisions, next

The current state of the app, the owner-confirmed decisions that must not be
re-litigated, and the short list of open work. The landed-work record (Steps
0-8, every feature write-up, every measurement) lives in `docs/plan-archive/`,
indexed by `docs/plan-archive/README.md`. Read this file first; open an archive
file only for the area you are working on.

Companion: `AGENTS.md` (conventions, commands, hard-won gotchas). Read that
before changing anything.

**Resume protocol.** Pick one item from Next up, do only that item, run its
verification, then record it as a dated `## YYYY-MM-DD — title` section at the
top of `docs/plan-archive/work-log.md` and update its row in Status if it
shipped. This file carries state only: no narrative, no measurements, no log
entries. Commit nothing unless asked.

## Next up

Nothing here is pre-approved; each item needs the owner's go.

1. **KWin blur upstream close-out** (Step 0b is the one open 🔶; full analysis
   in `docs/plan-archive/remaining-work.md`):
   - File with `xarblu/kwin-effects-better-blur-dx`: the suspect line is theirs,
     the API that dropped the alignment step is KWin's `BackgroundEffectItem`.
     Cross-link KWin if the conversion belongs compositor-side. Issue creation
     was blocked on the owner's GitHub account; it is not drafted yet.
   - Two cheap strengtheners, neither done: 150% display scale (proves
     fractional-ness rather than 1.6 specifically) and one other translucent
     window (Konsole at ~90% with `BlurNonMatching=true`), which would prove it
     is not Songstress, Tauri or WebKitGTK.
   - Ask infinality for a COPR snapshot bump: the NVR stamps a September build
     date on a June commit, so `dnf update` looks like it contains fixes that
     are not there.
   - Box state: running a source build of `728307f`, the RPM intentionally
     removed. A KWin or Plasma upgrade breaks blur until it is rebuilt against
     the new KWin.
2. **Screen dropper live check** (🔶 2026-09-15): the portal `PickColor`
   crosshair shipped but has never been exercised. One live pick and one Esc in
   the app closes it.
3. **Settings panes P3 critique re-run** (⬜ since 2026-08-31): the polish pass
   landed, the re-run never happened.

## Deferred, do not start without asking

- Artwork editing in the tag editor (Step 1 decision; specced 2026-09-03 in
  `docs/plan-archive/tag-editor.md`).
- Crossfade option, the escape hatch for the Phase 3 encoder-padding seam at
  Epicus Furor to Emerald Sword.
- MPRIS-initiated volume changes do not write through to the settings DB
  (acceptable for now; revisit if it annoys).

## Status

| Work | Status |
|---|---|
| Phase 0 — env + scaffold | ✅ 2026-08-21 |
| Phase 1 — album grid on fake data | ✅ 2026-08-21..22 |
| Phase 2 — library scanner (SQLite) | ✅ real scan 2026-08-22 |
| Phase 3 — MPV JSON IPC playback | ✅ 2026-08-22 |
| Phase 4 — MPRIS | ✅ 2026-08-23 |
| Step 0 — bug fixes before features | 0a ✅ · 0b 🔶 upstream · 0c ✅ |
| Step 1 — tag editor | ✅ 2026-08-23 · fix round 2026-08-24 |
| Step 2a — import music, two-step staging | ✅ 2026-08-24 · in place since 2026-08-31 (migration v2) |
| Step 2b — playbar gradient toggle | ✅ 2026-08-24 |
| Step 3 — menu bar, Plasma Global Menu, window identity | ✅ 2026-08-24 |
| Step 4 — accent color picker | ✅ 2026-08-24 |
| Step 5 — playback controls (shuffle, repeat, album nav) | ✅ 2026-08-25 |
| Step 6 — equalizer | ✅ 2026-08-25 |
| Step 7a — play-next queue | ✅ 2026-08-25 |
| Step 7d — live library watching | ✅ 2026-08-25 |
| Step 7c — multi-library roots | ✅ 2026-08-26 |
| Step 7b — RPM packaging | ✅ 2026-08-26 · 0.6.0 built 2026-09-01 |
| Step 8 — titlebar elimination, chrome into sidebar | ✅ 2026-08-27 |
| Motion audit, settings tree and panes, dismissal family, surface template | ✅ 2026-08-29..09-03 · 0.9.0 |
| Window size memory (position plugin tried, reverted) | ✅ 2026-09-01 · redone 2026-09-05 · 0.9.15 |
| Loading skeletons, failed-scan reason, arrival entrance | ✅ 2026-09-02 · 0.8.0 |
| Tag editor redesign, phases A to E | ✅ 2026-09-03 · 0.9.0 to 0.9.3 |
| Accessibility pass, perf audits | ✅ 2026-09-06 (content-visibility tried and reverted) |
| Per-album gradients, screen dropper | 🔶 2026-09-15 · dropper needs one live pick |
| GNOME portability, opaque chrome, traffic lights | ✅ 2026-09-21..23 · 0.12.0, 0.12.1 |
| Dev/prod instance split, empty-state mark, Genre to Track# cascade | ✅ 2026-09-24 |
| WebKitGTK 2.54 artwork and stutter | ✅ 2026-09-26 · stutter is upstream; box pinned to 2.52.1 |
| Matte grain fields and level dial | ✅ 2026-09-26 · 0.13.3, 0.13.4 |
| Scrim press-dismiss, tag suggestions, `.songstress.json`, playbar re-anchor | ✅ 2026-09-26 · 0.14.0 |
| Playbar progress style, dropper overlay, 0.16.0 batch | ✅ 2026-09-28 · 0.15.0, 0.16.0 |
| Scan ring, waveform backfill, light contrast, tag-editor open | ✅ 2026-09-29 · 0.16.1, 0.16.2 |
| GNOME window frame and window shadow | ✅ 2026-10-02, 2026-10-04 · 0.16.3, 0.16.6 |
| Playbar drag, transport gap, edge shadows, corner resize | ✅ 2026-10-05 · 0.16.7 to 0.16.11 |
| Expanded-panel choreography and retune | ✅ 2026-10-09 · shipped as 0.16.14 (the 0.16.15 to 0.16.17 chain was consolidated into it, `0161cf4`) |
| Relink moves the track id with the path; migration-failure journal names its half | ✅ 2026-10-30 · no version bump (version files are watched) |

## Archive

The record: `docs/plan-archive/README.md` indexes every moved section and the
step map. Quick pointers: `steps-0-7.md`, `step-8-titlebar.md`, `tag-editor.md`,
`tag-editor-phase-c.md`, `playbar-progress.md`, `webkitgtk-254.md`,
`display-state.md`, `library-and-import.md`, `settings-and-surfaces.md`,
`surfaces-and-menus.md`, `remaining-work.md`, `environment.md`,
`spec-summary.md`, `open-threads.md`, `work-log.md`.

## Decisions log (user-confirmed, do not re-litigate)

- Bug fixes come BEFORE all features (Step 0 first).
- Blur A/B test approved; display-scale change requires asking first.
- Tag editor: full MusicBee field set (incl. track/disc totals; conductor/
  lyrics/ratings excluded), artwork editing DEFERRED, right-click menus +
  always-visible album-header edit button.
- Import: TWO-STEP staging flow (import-to-listen vs save-to-library);
  staging persists until saved/discarded; badge in grid; save = per album /
  per track / global save-all; kdialog pickers; drag-and-drop included.
- Menu bar (Global Menu XOR in-titlebar bar); 4 menus (Playback/Library/View/Help);
  static window title; desktop-entry identity slice pulled forward.
- Accent picker: presets + custom + reset; lands after menus.
- Shuffle/repeat: reshuffle-on-wrap; stages persist; after accent picker (before EQ).
- EQ: 10-band + preamp; playbar popover AND Playback menu; after shuffle/repeat.
- Album nav buttons: global grid order, wrap, no-op when stopped.
- Track ordering: unnumbered tracks alphabetical BEFORE numbered (per disc).
- Step 7 order: play-next queue → live watching → multi-roots → RPM
  packaging; MPRIS Next/Previous respect the queue; STOP before packaging
  and check with the user first. (7b started 2026-08-26 — user gave the go.)
- **Flatpak deferred until distribution is a real goal (2026-08-26)**:
  RPM is the only packaging format for now. All packaging work is additive
  (release profile, bundler config), so a future flatpak is a second build
  pipeline (flatpak-tauri/flatpak-builder, KDE runtime, mpv + kdialog
  bundled in-sandbox) on top of the SAME binary — nothing to rip out. Keep
  settings/data in Tauri's app_data_dir (maps to the sandboxed
  ~/.var/app/.../data automatically). Note: `kde_window_decoration` reads
  ~/.config/kwinrc + klassyrc directly — in a sandbox that is NOT exposed,
  so it will silently fall back to its built-in defaults (fine, remember).
- **UI revamp planned (2026-08-26)**: until the revamp, logic-first —
  build and fix features before the UI pass; don't churn cosmetics inside
  feature work.
- **The 20px line (user-designed, 2026-08-29)**: every open/collapse/
  switch ends with the focused album's row 20px (the scroller's top
  padding) under the window top — or as low as the final layout's bottom
  edge allows. Travel is an ANCHOR (live row position, eased glide +
  hold), never a predicted tween (prediction produced a visible
  stall→snap on cross-row switches). No app-initiated auto-scroll on
  expand (reveal removed); wheel/keys/touch cancel the glide; reduced
  motion disables the reframe entirely.
- **Cross-row switches are COLLAPSE-THEN-EXPAND (user decision,
  2026-08-30 — choreography removed)**: the host stays at the outgoing
  album while it plays the plain 280ms collapse; `onClosed` consumes the
  pending open (token flip → fresh mount at the destination, glide,
  360ms grow). Only the plain animations exist now: fresh expand,
  collapse, same-row in-place swap + 160ms fade. WHY the bespoke
  cross-row design went away: the parallel version (outgoing panel
  ghosted at its row, closed under the 20px-line pin while the
  destination grew) is only clean when the close plays BELOW the pinned
  row — scrollTop only tracks layout ABOVE the row, so an upward
  switch's close costs the pin nothing, but a downward switch's close
  moves layout above the row while the anchor writes a scrollTop that
  this WebKitGTK applies one PAINT late. That combination was never
  fully tameable: instrumented traces (per-frame row position, ghost
  height, scrollTop before/after) showed the pin holding the row at
  20±1px while the user still reported motion in the 20px strip and a
  final rest above the line — a measurement/observation gap no amount
  of prediction closed (tried: catch-up anchor, next-paint prediction,
  frame-count-domain close with exact per-paint shrink, predicted
  drop-off, composite-position probe). Lesson: never animate layout
  ABOVE the pinned row on this target; sequence instead.
  (Removed machinery: ghost rows, two-value tokens for close hand-off,
  grid-driven/frame-count close, predictive pin, `lib/ease.ts`.
  The host token SURVIVES as a mount-identity mechanism: a cross-row
  flip re-keys the destination row so it mounts fresh instead of
  re-keying the outgoing instance there (teleport).)
  FOLLOW-UP (same day, user requested): the two ALBUM animations run
  IN PARALLEL — the outgoing panel becomes a ghost row at its own row
  (fresh mount at the measured height — the seamless hand-off trick —
  plain 280ms CSS close, no pin/prediction of any kind) while the host
  flips to the destination at t=0 (token flip → fresh mount → grow).
  Only the VIEW'S TRAVEL waits: a per-section glide queue drains when
  the ghost list is empty (drainGlide), because the travel is the one
  thing that conflicts with a close above the line — and a close above
  the row never moves the row ON the line (its close plays BELOW that
  row, the row's own panel slot), so the window is pin-free and
  bob-free in both directions. Upward switches read as natural
  accordion reflow (the new panel growing above pushes the old row down
  while it closes); the glide then lands on an already-grown panel —
  the smoothest glide of all the variants. The heavy pin/predict
  machinery stayed removed: what came back is only measured spawn +
  plain CSS close + a queue.
- **WebKitGTK rides 2.52.1 for now (owner, 2026-09-26)**: 2.54.0's Skia
  compositor stutters on this box and 2.52.1 does not (owner-verified A/B).
  The pin is manual, so `dnf upgrade` undoes it and the stutter returns;
  revisit on 2.54.1 rather than re-litigating the downgrade in the meantime.
