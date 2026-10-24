# Spec summary (from the brief) (plan archive)

Moved out of `PLAN.md` on 2026-10-24 so the plan file stays readable in one pass.
This is the record, not the state: current state lives in `PLAN.md`, and this
file is the evidence behind it. Sections are kept verbatim, in their original
order. The index of every section is `README.md` next to this file.

## Spec summary (from the brief)

- Fedora 44 KDE/Wayland, album-grid player modeled on MusicBee browsing.
- Sidebar = flat artist list, click filters grid ("All Artists" state).
  Sorting: artists A→Z, albums by year ascending.
- Grid = rows of N tiles OR one full-width expanded tracklist panel.
  Row-model, never expand "inside a tile".
- Playback state is `{ albumId, trackIndex }` — no queue. Click track →
  play from there through the album; another album replaces context
  entirely. Gapless via mpv append-play + `--gapless-audio=yes`.
  ReplayGain: album mode default (configurable later).
- Tile size + sidebar row size sliders drive CSS custom properties.
- Thumbnails at ~3 sizes, never downscale full-res at render time (Phase 2).
- Glassmorphism, light/dark; `backdrop-filter` on chrome surfaces only;
  KWin blur-behind needs Better Blur DX. (Aug note said "Wayland clients can't request
  it" — the reason changed, the conclusion didn't: `ext-background-effect-v1` is live in
  KWin (MR 4890, merged 2026-01-28) and in GNOME 51's mutter, but the request path is
  GTK/KF6-owned, so a Tauri/WebKitGTK surface still cannot reach it. Force blur remains
  the only lever for this window.)
- `decorations: false`, custom macOS traffic lights.
- Build order: UI on fake data → scanner → mpv IPC → MPRIS (early).

