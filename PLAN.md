# PLAN.md — Songstress: roadmap + implementation record

The single file to read the app's implementation in: decisions, remaining
work, and the full landed-work log (the former PROGRESS.md was merged into
this file 2026-08-26). Companion to **AGENTS.md** (conventions & hard-won
gotchas — read that too before changing anything).

**Resume protocol**: pick the first unchecked item under *Remaining work*,
do only that item, run its verification, tick it, commit nothing unless
asked, and append to the *Implementation log* when a whole step lands. If
context was lost, re-read AGENTS.md + this file; that is enough.

Status legend: ⬜ todo · 🔶 in progress · ✅ done

## Status

| Work | Status |
|---|---|
| Phase 0 — env + scaffold | ✅ 2026-08-21 |
| Phase 1 — UI on fake data | ✅ (several polish iterations) |
| Phase 2 — library scanner (SQLite) | ✅ real scan verified 2026-08-22 |
| Phase 3 — MPV JSON IPC playback | ✅ 2026-08-22 |
| Phase 4 — MPRIS | ✅ 2026-08-23 |
| Step 0 — bug fixes (before all features) | 0a ✅ · 0b 🔶 upstream · 0c ✅ |
| Step 1 — tag editor | ✅ 2026-08-23 (+ fix round 2026-08-24) |
| Step 2a — import music (two-step staging) | ✅ 2026-08-24 |
| Step 2b — playbar gradient toggle | ✅ 2026-08-24 |
| Step 3 — menu bar + Plasma Global Menu + identity | ✅ 2026-08-24 |
| Step 4 — accent color picker | ✅ 2026-08-24 |
| Step 5 — playback controls (shuffle/repeat/album nav) | ✅ 2026-08-25 |
| Step 6 — equalizer | ✅ 2026-08-25 |
| Step 7a — "play next" queue | ✅ 2026-08-25 |
| Step 7d — live library watching | ✅ 2026-08-25 |
| Step 7c — multi-library roots | ✅ 2026-08-26 |
| Step 7b — packaging (RPM) | ✅ 2026-08-26 · rpm-only targets + 0.6.0 built 2026-09-01 |
| Step 8 — titlebar elimination (chrome into sidebar) | ✅ 2026-08-27 |
| Step 8c — playbar critique fixes | ✅ 2026-08-29 (re-critique 35/40) |
| Expansion choreography + 20px-line reframe | ✅ 2026-08-29 |
| Cross-row choreography REMOVED (collapse-then-expand) | ✅ 2026-08-30 |
| Cross-row: parallel collapse + expand (travel drains) | ✅ 2026-08-30 (reimplemented + trace-verified) |
| Motion audit: tokens + polish (impeccable) | ✅ 2026-08-30 |
| Settings menu tree: sidebar-shaped + glass controls | ✅ 2026-08-30 (RPM 0.2.0-1 built 2026-08-30) |
| Settings panes: critique pass C (layout + quieter) | ✅ 2026-08-31 |
| Settings panes: pass A (clarify: theme segments, slider values, copy) | ✅ 2026-08-31 |
| Settings panes: pass B (audit: focus handoff, inert, contrast) | ✅ 2026-08-31 |
| Settings panes: polish (P3s) | ✅ 2026-08-31 · critique re-run ⬜ |
| Settings stack: one dismissal family, platform geometry, spacing ladder | ✅ 2026-08-31 |
| Manage-imports window (one card per artist, per-album Apply) | ✅ 2026-08-31 |
| Import staging: cache copy → in-place row flag (migration v2) | ✅ 2026-08-31 |
| Imported-music modal: critique 30/40 → all 6 issues fixed (3 P1 + 3 P2) | ✅ 2026-09-02 |
| Surface template: boxed ✕ top-right for every modal AND popover | ✅ 2026-09-01 |
| Escape: one press, one verb (the surface owns it) | ✅ 2026-09-01 |
| Stack: a layer holds its own content while it animates out | ✅ 2026-09-01 |
| Window geometry across restarts: `window-state` tried, **reverted** (Wayland can't, and the tiling does it) | ❌ 2026-09-01 |
| First-scan repro harness (park + sandbox data dir) + EmptyState bar fix | ✅ 2026-09-02 |
| Loading skeletons: grid + artist list, no scan bar in the stage | ✅ 2026-09-02 · **0.8.0** |
| Failed scans always emit their reason + the empty state renders it | ✅ 2026-09-02 · **0.8.0** |
| Arrival entrance: list cascades, grid unfolds on its diagonal | ✅ 2026-09-02 · **0.8.0** |
| Playbar volume slider: glass styling, native chrome gone | ✅ 2026-09-03 · **0.9.0** |
| Global Menu rebuild: the broadcast mirrors the panes | ✅ 2026-09-03 · **0.9.0** |
| Context-menu pass: one album-menu builder, `reveal_container`, artist pending dot | ✅ 2026-09-03 · **0.9.0** |
| Tag Editor redesign Phase A (Rust) + B (album modal + picker) + C (track modal, split, stepper) | ✅ 2026-09-03 · **0.9.0** · D ✅ **0.9.2** · E ✅ **0.9.3** |
| Audit pass: aria-live announcer (WCAG 4.1.3) + radii on-scale + --on-cover | ✅ 2026-09-03 · **0.9.1** |

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

## Remaining work

### Step 7b — Packaging (RPM) — ✅ (landed 2026-08-26)

DECIDED (2026-08-25, go-ahead 2026-08-26): `tauri build` → native RPM
(dnf install/remove). AppImage rejected (not native to Fedora/KDE, no
package-manager integration); Flatpak deferred (decisions log). Release
profile: lto="thin", codegen-units=1, strip=true. Bundler generates the
system desktop entry + icons; remove the user-level
~/.local/share/applications/com.yossi.songstress.desktop after install.
Binary name MUST stay `songstress` (resourceClass + dev unit). Data stays
at ~/.local/share/com.yossi.songstress (settings/library survive the
dev→packaged switch). Version 0.1.0. NOT in scope: auto-update, Flathub,
external repo. Add `Requires: mpv kdialog` to the spec (mpv is spawned, not
linked — the bundler can't see it); better-blur-dx stays optional (COPR,
graceful degradation). sudo needs a password → the USER runs dnf install
lines; the agent builds, hands the command over, and does post-install
verification. Verify: install → krunner launch → Global Menu + MPRIS +
blur → dnf remove clean.

**Landed so far (2026-08-26)**:
- `Cargo.toml [profile.release]`: lto="thin", codegen-units=1, strip=true.
- `tauri.conf.json`: productName "Songstress" (display name only — the
  binary name comes from the Cargo package name, so `songstress` /
  resourceClass is untouched), category **"Music"** (maps to freedesktop
  `AudioVideo;Audio;Music;`), license MIT, short/long description,
  `bundle.linux.rpm = { release: "1", depends: ["mpv", "kdialog"] }`.
- **Category gotcha**: the bundler's AppCategory is the macOS/GNOME-style
  name list ("Music", "Video", "Game", …) — NOT freedesktop strings
  ("AudioVideo" / "Audio" / "Audio;Video;" all → "invalid category", and
  the failure surfaces only AFTER the full cargo release build, ~2 min of
  nothing). The error has no did-you-mean when confidence < 0.8.
- **rpmbuild NOT required** — the tauri bundler assembles the RPM itself
  (cpio+gzip); only `rpm` is needed to inspect.

**`bundle.targets` is `["rpm"]`, not `"all"` (2026-09-01, owner confirmed).** With
`"all"` the build writes a good RPM and THEN dies on the AppImage step — `linuxdeploy`
cannot fetch its tools here — so every release build ended in a failure that meant
nothing. The config is strict JSON (no comment can carry the reason), so the reason
lives here: RPM is the packaging format, AppImage was rejected (not native to
Fedora/KDE, no package-manager integration), Flatpak stays deferred.
- Built: `src-tauri/target/release/bundle/rpm/Songstress-0.1.0-1.x86_64.rpm`
  (~11 MB). rpm -qip/-ql/-qp --requires verified: Name songstress 0.1.0-1,
  MIT, /usr/bin/songstress, /usr/share/applications/Songstress.desktop
  (Name=Songstress, Exec=songstress, StartupWMClass=songstress,
  Icon=songstress, Categories=AudioVideo;Audio;Music;), hicolor icons
  32/128/256@2, Requires mpv + kdialog (+ webkit2gtk/gtk3).
- Dev unit stopped before install (MPRIS name / appmenu registration are
  single-instance per name).
- **Installed + verified from KRunner (2026-08-26, user ran the dnf lines)**:
  `/usr/bin/songstress` running (pgrep); full library grid rendered from the
  SHARED ~/.local/share/com.yossi.songstress DB (dev→packaged data survives
  ✓); MPRIS root + Player interfaces live (Metadata {}, PlaybackStatus
  Stopped — both on /org/mpris/MediaPlayer2, as built in Phase 4); Global
  Menu GetLayout returns Playback/Library/View/Help; KWin getWindowInfo:
  resourceClass `songstress`, desktopFile `songstress` (→ Better Blur DX
  WindowClasses match; isEffectLoaded(better_blur_dx)=true). Screenshot
  verified. NOTE: DNF 5 has no `-U` for `install` — the RPM path alone
  upgrades-or-installs.

**Remaining (user, then agent)**:
1. ~~USER: close the app, `sudo dnf remove songstress`.~~ DONE 2026-08-26 —
   AGENT verified clean: `rpm -q` → not installed, no /usr/bin/songstress,
   no system Songstress.desktop, hicolor icons gone, **data intact**
   (songstress.db 1.7 MB untouched) → dev unit restarted, watcher armed,
   zero scans (the 7c loop fix holds in dev too). User-level dev-identity
   entry still present (dev workflow restored).
2. ~~⬜ USER decision (only matters when the RPM is installed again): remove
   ~/.local/share/applications/com.yossi.songstress.desktop to avoid a
   duplicate KRunner entry — or keep it if dev work resumes (the dev
   app's panel identity comes from it).~~ RESOLVED 2026-08-26: user
   reinstalled the RPM and removed the user-level entry (kbuildsycoca6
   re-run); single launchable = /usr/share/applications/Songstress.desktop.
   To resume dev-only work later, recreate the user-level entry (Step 3
   notes: StartupWMClass=songstress).

### Step 0b — Frozen border artifacts on window move — 🔶 diagnosed: upstream effect bug

**Symptom**: fast side-to-side window moves leave a ~1px stale vertical line
at the old window border (captured via screen recording; both left and right
borders seen). Vanishes on any forced redraw. ALSO: switching to the app
shows stale translucency for a frame (glass backdrop shows the previous
background until a repaint) — same BlurCache staleness family, include in
the upstream report.
**Diagnosis COMPLETE (2026-08-23)**:
- ✅ Blur A/B (both directions): artifacts GONE with `better_blur_dx`
  unloaded, back when loaded. Effect is the culprit.
- ✅ RoundedCornersPass ruled out: `CornerRadius=0` + reload → artifacts
  still appear. Config restored to 14.
- User's build: kwin-effects-better-blur-dx 2.5.1 (git 20260808 e8475d0,
  xarblu fork — taj-ny's original ARCHIVED Nov 2025). Build already contains
  upstream's July/Aug cache+scissor+ceil fixes ("fixup glScissor for small
  dirtyRegions", "ceil glWidth/glHeight", "expand contentsRect by 1px").
- Leading theory: stale/short glScissor in the BlurCache draw path clips
  the exposed-region repaint at fractional scale (1.6×) — the final
  device-pixel column of the old window rect is never repainted. Upstream
  territory.
- Upstream refs: xarblu/kwin-effects-better-blur-dx #92 (blur caching stale
  transparency — closed by adding fixes, not a toggle), #14 (move
  artifacts, closed), taj-ny #143 (artifacts at smallest blur strength,
  open).
**Action**:
- ⬜ File upstream issue with the screen recording (repo is active, similar
  issues fixed within weeks). NOTE: GitHub showed "issue creation is
  restricted" — user may need to comment on #14/#92 or file via KDE
  bugzilla.
- ⬜ Re-test after `dnf update kwin-effects-better-blur-dx` (user runs dnf).
- Cosmetic meanwhile: artifact clears on any redraw over that area.
- App-side fix impossible: the stale pixel is outside the moved window —
  only the compositor/effect can repaint it.

### Up next (user-confirmed 2026-08-30, after the motion-audit fixes)

- **Skeleton loaders for the grid + artist list** — ✅ shipped 2026-09-02, and the
  EmptyState progress bar did NOT stay: the stage is now textless while loading
  (see the implementation log). Two loose ends left on purpose, both needing the
  owner's live library rather than the sandbox:
  1. ~~`BOOT_GRACE_MS` is a guess~~ — **measured 2026-09-02** on the live library:
     `get_library` over 251 albums / 4464 tracks is **46–50ms**, so the 150ms grace
     stays (≈3x, never flashes on a warm launch). The note's grace was measured too:
     a watcher rescan of the same library ran 83ms and `.scan-note` never appeared
     across 357 sampled frames. STILL UNVERIFIED: the note *appearing* on a genuinely
     long scan (Library → Full Rescan, ~25s, proven safe — the track upsert does not
     touch the `staged` column) — it needs the owner's eyes, not another agent run.
  2. `.sk-cover` carries no cover-mass shadow. The real tile does. Call: whether a
     placeholder should already weigh 18px of shadow, or whether weight arrives
     with the artwork. One line either way.
- **Modal work (later, user-owned)**: TagEditor's entrance. The audit noted
  it mounts with zero entrance motion (TagEditor.svelte:251 `{#if open}`) —
  a ~180ms scale(0.97)+fade from center is the planned direction
  (transform+opacity only; center origin is correct for a centered modal).
  The first real modal of this round — the imported-music manager — shipped
  without entrance motion too, matching MusicFolders/About on purpose.

### Explicitly deferred

- Artwork editing in the tag editor (Step 1 decision). — specced 2026-09-03,
  see **Tag Editor Redesign — working spec** at the end of this file.
- Crossfade option — the eventual escape hatch for the Phase 3 audible seam
  at Epicus Furor→Emerald Sword (source-file MP3 encoder-padding estimation;
  not fixable app-side).
- MPRIS-initiated volume changes do NOT write through to the settings DB
  (the frontend owns persistence and doesn't hear about them) — acceptable
  for now; revisit if it annoys.
- Visual pass on the import flow on the real library (user) — ✅ 2026-09-04 — full flow run for real (see log entry below 0.9.5).
- Menu label ambiguity: "Add music folder…" (import staging, Step 2a) vs
  "Music folders…" (library roots, Step 7c) sit in the same Library menu
  and are easily confused — **resolved 2026-08-31 (pass A)**: the staging pair
  is now "Import music files…" / "Import music folder…" (ids unchanged),
  "Music folders…" (roots) keeps its name, and "Full Rescan (rebuild)" became
  "Full rescan (all files)". EmptyState's "Add music folder…" stays — it opens
  the ROOTS flow, where that name is the correct one.
- ~~**Global Menu: theme is 2-state there while the app has 3 modes**~~ —
  **resolved 2026-09-03** by the full Global Menu pass (see "The broadcast
  mirrors the pane" at the end of this log): `MenuState.theme` is now the MODE,
  the Appearance menu carries three radio items, and every `pushMenuState()`
  field was re-checked for bool-vs-mode flattening (`stagedCount` carries the
  sidebar door's count; `playbarGradient` joined the state; shuffle/repeat/eq
  were already honest strings or true bools).

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
  KWin blur-behind needs Better Blur DX (Wayland clients can't request it).
- `decorations: false`, custom macOS traffic lights.
- Build order: UI on fake data → scanner → mpv IPC → MPRIS (early).

## Implementation log (build order)

### Phase 0 — done (2026-08-21)
- rustup stable (1.98), nodejs+npm via dnf, Tauri Linux deps installed by user.
- Scaffolded from create-tauri-app svelte-ts template, **then stripped
  SvelteKit → plain Vite+Svelte** (no router needed). vitest added.
  svelte-check wired.
- Window: 1280×800 min 960×600, `decorations:false`, `transparent:true`,
  client-side 14px border-radius. App-id `com.yossi.songstress`, binary
  `songstress`, project `~/Projects/Songstress`.
- Capabilities: window minimize/toggle-maximize/close/start-dragging/is-maximized.

### Phase 1 — done (several polish iterations)
- Row-model grid: `buildRows(albums, cols, expandedId)` pure fn + tests;
  column count mirrors CSS auto-fill math (`columnCount`); rows keyed
  stably (`r-<firstAlbumId>` / constant `"expanded"` so the expander
  survives relocation).
- ExpandedPanel: mount-closed + double-rAF flip (first-open animates);
  open/close animates `grid-template-rows 0fr→1fr`; switch = instant
  content swap + 160 ms fade-in of new content, single-step height change
  (no ghost of previous album, no per-frame relayout). Artwork-gradient
  background (see below). Header: title + circular play-all button (plays
  track index 0) top-right; artist · year under it; meta footer
  bottom-right ("N tracks · total"). Tracklist ≥10 tracks → 2 CSS columns;
  multi-disc albums get uppercase "Disc n" dividers + per-disc numbering.
  Track click → `playTrack(albumId, flatIndex)`; current row shows ▶/❚❚.
- Sidebar: All Artists pinned + A→Z with counts, quick-filter box, gear
  popover = tile-size slider (120–320), sidebar-row slider (28–52), theme
  toggle. Sidebar ends above playbar (`bottom: var(--playbar-h)`) so the
  popover can't be overlapped.
- Overlay layout: `.stage::before` paints grid backdrop (own alpha),
  content scrolls UNDER sidebar/playbar so their backdrop-filter frosts
  scrolling art.
- Alphas (user-tuned): grid 0.8 / chrome 0.7 / panel gradient 0.36 dark ·
  0.30 light.
- Themes: dark/light tokens in app.css; theme toggle persists; noise
  garnish on .glass only.
- TitleBar: tb-* namespaced classes, traffic lights, drag regions,
  dblclick maximize handled natively by Tauri drag-region.
- PlayBar: fake playback engine (ticking position, advance-through-album,
  skip, seek, volume persisted).
- Fake library: 14 albums / 9 artists incl. **compilation** (Anison no
  Kokoro, 34 tracks, Various Artists) and **multi-disc** (Keepers Live, 22
  tracks / 2 discs). Covers are real folder.jpg files copied into
  public/covers/.
- Blur integration: Better Blur DX via COPR, kwinrc preconfigured (see
  README.md + AGENTS.md gotchas). Verified working visually.

**Gradient algorithm** (`src-tauri/src/lib.rs` `colors::`):
- Decode cover (image crate) → thumbnail 48×48 → weighted buckets (weight =
  4 + chroma²/16, ALL pixels count so primary = whole-artwork average).
- Primary c1 = weight-average color. Buckets merged greedily into clusters
  (manhattan < 64). Accent c2 = cluster with max distance from c1 among
  clusters ≥25% of heaviest weight; fallback = lighten(c1).
- Frontend mixes each 30% toward theme surface + lifts (+26 floor, ×0.85),
  alpha per theme, 135° two-stop gradient, cached per cover (successes only).
- Tests: every-cover no-overflow + `anison_accent_is_the_blue_hair`.

### Phase 2 — done (real library verified 2026-08-22)

**Original design (as specced, 2026-08-21)**: settings table in SQLite
(music dir, ui prefs, replaygain, volume) migrating localStorage keys over;
scan pipeline = walkdir over the configured root (default ~/Music/Music
Files) → lofty tags (mp3/flac/m4a/aiff; wav sparse-tags acceptable) →
group by albumartist→artist→"Various Artists" → albums keyed
(albumartist,title,year) → tracks ordered (disc,track) → sort_name
computed at insert (Rust port of src/lib/sort.ts); thumbnails 96/256/512
WebP into app cache via the `colors::extract` reuse, served by a `thumb://`
protocol, panel colors stored in DB (kills per-open extraction); progress
events `scan-progress`; empty-state screen before first scan; frontend swap
behind a flag, keep fake mode for UI dev. Schema (v1, as shipped):
```sql
artists(id TEXT PK, name TEXT, sort_name TEXT)
albums(id TEXT PK, artist_id TEXT, title TEXT, year INT NULL,
       UNIQUE(artist_id, title, year))
tracks(id TEXT PK, album_id TEXT, disc INT, track INT, title TEXT,
       duration_sec REAL, path TEXT UNIQUE)
meta/settings(key TEXT PK, value TEXT)
```

**M1 — db + settings (2026-08-22)**: `src-tauri/src/library/{mod,db,settings}.rs`
— rusqlite (bundled) + WAL, atomic migration framework (v1 = full schema
incl. color/thumb columns), settings get/set/init commands with
INSERT-OR-IGNORE localStorage seeding, blake3 `stable_id()` helper. DB
verified created+migrated on launch at `app_data_dir/songstress.db`.
10/10 cargo tests.

**M2 — scanner (2026-08-22)**: `library/scan.rs` — walkdir+lofty, grouping
(albumartist→artist→Unknown; mixed-artist albums → Various Artists),
sort_name = Rust port of sort.ts, stable blake3 IDs, incremental via
mtime_ns+size, deletion + orphan sweep. `scan_library` command (own
connection, concurrency-guarded) emits `scan-progress`/`scan-finished`.
Fixtures committed under `src-tauri/fixtures/library/` (multi-disc,
compilation, diacritics, The-prefix, 7 formats). 13/13 cargo tests, zero
warnings. No UI entry point yet — arrives with M4.

**M3 — artwork (2026-08-22)**: `library/artwork.rs` post-pass — folder art
> largest embedded picture, one decode → 96/256/512 WebP thumbs in
`app_cache_dir()/thumbs/<id>/` + colors stored in albums rows;
`colors::extract_image` refactor; `thumb://` protocol with size fallback +
traversal guard; wired into `scan_library`. Albums without art stay NULL
(placeholder). 14/14 cargo tests.

**M4 — frontend live (2026-08-22)**: live-by-default in Tauri
(`VITE_LIB=fake` opts out; browser always fake), class-based library store
hydrated by new `get_library` dump command + refreshed on `scan-finished`;
panel gradient reads scan-computed colorC1/C2 directly; settings hydrate
from SQLite (init_settings seeding → get_settings wins, debounced
write-through); EmptyState with folder picker and scan progress;
title-bar hamburger menu = Rescan / Choose music folder… / About (the
hamburger was later REMOVED in Step 3 — menu bar XOR Global Menu). Folder
picker = native KDE `kdialog --getexistingdirectory` via our own
`choose_music_folder` command (starts at saved musicDir → ~/Music → $HOME;
cancel returns null) — tauri-plugin-dialog removed: its GTK chooser looked
GNOME-ish on this Plasma-only personal app.

**Fix round 2 (2026-08-22, after the user's real scan exposed grouping bugs)**:
- **Grouping rewritten** (`scan.rs`): directory-authoritative — same
  folder ⇒ same album. Tracks bucket per (parent dir, norm title);
  artist/title/year are per-bucket CONSENSUS values; buckets merge across
  dirs (multi-disc) when titles match and release artists agree.
  albumartist is TRUSTED: differing track artists are guests, not Various
  Artists; VA only for albumartist-less folders with mixed artists. Year
  left the identity key entirely. 5 new staged-fixture tests (retagged
  copies via lofty `insert_text`/`remove_key` + `WriteOptions`).
- **Case-insensitive folder art** (`artwork.rs folder_art`): one read_dir →
  lowercase map, FOLDER_ART priority preserved; `Cover.JPG` etc. now
  match. Unit + refresh tests.
- **Dev-build dep optimization** (`[profile.dev.package."*"] opt-level=2`):
  our crate stays opt-0. Test suite 6.6s → 0.33s; full scan of 4405 files
  7.5s.
- **Rayon-parallel `artwork::refresh`**: serial prep (all DB access) →
  parallel decode/thumbs/colors (atomic counter, scoped poller thread
  emits progress) → serial UPDATEs. Real-library artwork phase: minutes →
  5.1s.
- **`[scan]` timing log** in `scan_library`: files/artwork durations +
  counts to stderr.
- **Year sort everywhere**: store-level `byYear()` (nulls last, sortKey
  tie-break) applied to live dumps AND fake data.
- **Expansion scroll cap**: `reveal()` target clamped to aligning the
  expander's top border with the scroller top (oversized panels no longer
  scroll past their start).
- **Column thresholds**: disc tracklists split at ≥5, single tracklists ≥8.
- **Titlebar menu z-index fix**: `.stage` creates no stacking context, so
  Sidebar (z-10, later in DOM) painted OVER the whole titlebar incl. the
  app menu → items unclickable. `.tb-root` now z-40.
- **KWin blur outage** (2026-08-23, RECURRED after reboot): better_blur_dx
  wasn't loaded (silent failure — kwinrc correct, no journal error). Fixed
  for good: `kwin-blur-load.service` systemd user unit
  (`~/.config/systemd/user/`, script `~/.local/bin/kwin-blur-load.sh`),
  ordered `After=plasma-kwin_wayland.service`, WantedBy
  graphical-session.target; retries `loadEffect` over DBus up to 2 min.
  Enabled + tested (unload → start unit → loaded). Retires the
  "KWin Force-Blur script automation" deferred item. Also removed the
  stray `[Effect-better_blur_dx]` underscores group from kwinrc (the
  AGENTS-documented footgun; backup `~/.config/kwinrc.bak-blurfix`).
- **One-time DB reset** to apply the new grouping (incremental scan skips
  unchanged files by mtime+size, so old split rows survived a normal
  rescan): old DB kept as `songstress.db.bak-splitbug`; musicDir manually
  re-inserted (it is NOT localStorage-mirrored). Verified: 4405 added,
  ZERO duplicate album-title rows remain. Remaining odd albums are source
  tag issues — tag editor planned (landed as Step 1).
- **Decoration-aware titlebar** (`kde_window_decoration` command): parses
  `kwinrc` `[org.kde.kdecoration2]` (button layout) + `klassy/klassyrc`
  `[ButtonColors]` (palette/opacity) with hand-rolled INI parsing (no new
  crates) and serde_json for Klassy's override blobs; falls back to
  built-in defaults when absent. Frontend `stores/decoration.svelte.ts` +
  TitleBar render layout from `ButtonsOnLeft` filtered to CSD-honorable
  letters (X/I/A; Shade etc. skipped) and colors from the parsed palette.
  Verified pixel-exact vs the user's Klassy config
  (`#fd5256/#fbc006/#58fb3f` @85%). This is the "read the user's system
  config" seam — reusable pattern. (Flatpak note: ~/.config is not exposed
  in a sandbox → built-in defaults.)
- **Tracks play on DOUBLE-click**, not single click (ExpandedPanel `.track`
  buttons use `ondblclick`; single click = selection highlight, resets on
  album switch). Titlebar glyphs are white (Klassy titlebar-text style)
  and appear only on the hovered button, not the whole group.

### Phase 3 — done (2026-08-22)
- **Engine** (`src-tauri/src/mpv.rs`): one mpv spawned at setup
  (`--no-config --idle=yes --no-video --gapless-audio=yes --replaygain=album
  --input-ipc-server=$XDG_RUNTIME_DIR/songstress/mpv.sock`), tokio
  UnixStream, single writer task, reader task routes responses by
  `request_id` (global registry — NOT thread-local) and dispatches events.
  Properties are OBSERVED explicitly (`time-pos`/`duration`/`pause`) — mpv
  pushes nothing unobserved. Position throttled to ~10 Hz. Orphaned
  engines from previous sessions are reaped via a pidfile + /proc cmdline
  guard (`idle=yes` otherwise stacks silent mpvs forever).
- **Rust owns `{albumId, trackIndex}`**: `play_album` = playlist-clear →
  loadfile replace → append tail (native gapless). NOTE: `playlist-clear`
  takes NO argument (passing "current" errors out — cost one debugging
  round). `jump()` rebuilds the queue (played entries are consumed; a
  plain replace desyncs the tail) and WRAPS at album edges (prev on first
  track → last, next past end → first). eof→advance, error→skip, end of
  album→stop.
- **IPC values are TYPED** (`command(json!(...))`): mpv rejects
  stringified booleans ("unsupported format") and observe ids must be JSON
  ints ("invalid parameter") — both silently killed UI updates until caught
  by driving the socket directly.
- **Commands**: play_album / playback_toggle / playback_pause /
  playback_jump / playback_seek / playback_volume / playback_stop; managed
  `Engine` handle; spawn failure degrades to a detached engine whose
  commands error cleanly.
- **Frontend** (`playback.svelte.ts`): dual-engine behind `library.live` —
  live mode mirrors Rust state via events (`playback-changed/-paused/-
  position/-stopped`); fake timer engine kept for browser/fake dev. Same
  API, components untouched.
- Verified live: real audio through the user's library; UI tracks
  position/pause/advance. PlayBar: prev/next always enabled (wrap-around);
  volume speaker icon = mute/unmute. All Artists grid orders albums by
  artist sort_name then year (artist view stays year-ascending). One
  audible seam at Epicus Furor→Emerald Sword is source-file MP3
  encoder-padding estimation (both files same codec/rate) — not fixable
  app-side; crossfade option is the eventual escape hatch (deferred).

### Phase 4 — done (2026-08-23)
- **Server** (`src-tauri/src/mpris.rs`, zbus 5 tokio feature): serves
  `org.mpris.MediaPlayer2` + `.Player` at `/org/mpris/MediaPlayer2` as
  `org.mpris.MediaPlayer2.songstress`, backed by the SAME `PlayState` the
  frontend mirrors — UI and external controllers (playerctl, KDE widget)
  can never disagree. Server failure (busy name, no bus) is logged and
  swallowed; playback never depends on it.
- **Metadata** (9 fields): trackid/url/title/length/album/artist/
  albumArtist/contentCreated/artUrl, built per-read from the library DB
  (own connection — microseconds, never the shared AppState mutex). Two
  gotchas cost a debug round: D-Bus object paths only allow `[A-Za-z0-9_]`
  (blake3 ids carry hyphens → fold to `_` in `track_path()`), and
  `thumb_path()` already appends `thumbs` (pass the cache ROOT, not
  `<cache>/thumbs`, or artUrl's `exists()` silently fails).
- **Position/Volume**: `POS_BITS`/`VOL_BITS` statics in mpv.rs. Position
  is stored on EVERY mpv time-pos tick (the ~10 Hz throttle is
  frontend-emit only) and polled via the Position property — spec forbids
  pushing Position through PropertiesChanged. Volume is now an OBSERVED
  mpv property (observe id 4) so slider/MPRIS/mpv-internal changes all
  stay in sync.
- **Property changes**: a 5 Hz watcher task diffs a PlayState snapshot and
  emits PropertiesChanged only for what changed (status, metadata, Can*
  flags, volume). No notification hooks threaded through mpv.rs mutations.
- **Seeked**: emitted from `seek_to` (covers Seek AND SetPosition, as the
  spec wants); SetPosition validates the trackid against the CURRENT track
  and ignores stale ones. Seek/SetPosition clamp to [0, duration].
- **Verified live over D-Bus**: full introspection surface, metadata for
  real albums, Position polling, Pause/Play, Next/Previous (album wrap),
  Seek +30s with Seeked(int64) signal, SetPosition, Volume get/set +
  PropertiesChanged (metadata/status/can-*/volume all observed firing).
---

## Step 0 — Bug fixes (BEFORE all features)

### 0a. Tracklist phantom hover ✅
**Symptom**: hovering below the last track of column 1 highlights the first
track of column 2.
**Root cause (confirmed)**: `.tracklist.two` uses CSS multicol (`columns: 2`
+ `break-inside: avoid`, ExpandedPanel.svelte) — WebKit multicol
hit-testing maps dead-zone points into the next fragmentainer.
**Fix**: replace multicol with explicit grid in ExpandedPanel.svelte:
`display: grid; grid-auto-flow: column; grid-template-columns: 1fr 1fr;
column-gap: 24px; grid-template-rows: repeat(⌈n/2⌉, auto)` — row count
computed in the component (`Math.ceil(list.length / 2)`). DOM order and
visuals identical; applies to BOTH the single-tracklist branch and the
per-disc branch.
**Verify**: hover-probe the dead zone below column 1 in single-disc and
multi-disc albums; full AGENTS.md gates.

### 0c. Artwork blank-frame flash ✅
**Symptom**: fast mouse sweeps over grid artworks, or returning to the app
after idle → a cover goes blank for one frame.
**Root cause (confirmed by bisect)**: hover `translate` promotes a
compositor layer per hover-in/out; WebKitGTK paints the texture
create/destroy churn as a one-frame blank.
**Bisect history** (all tested live):
- drop `loading="lazy"` alone: flash persisted (kept anyway — idle
  eviction protection, 246 thumbs are cheap)
- permanent `will-change: translate` on all covers: flash GONE but 246
  permanent layers made 0b's move artifacts MUCH worse → reverted
- `:has()` batch promotion while grid hovered: flash worse → reverted
- **FINAL FIX**: removed the hover lift entirely; hover cue is now a
  paint-only outline ring (`outline-color: var(--text-dim)`) — no
  transform, no layer promotion, no churn. User verified flash gone.
**Note**: if the lift microinteraction is missed later, revisit only when
WebKitGTK fixes layer-churn blanking.

(0b remains open — see *Remaining work*; diagnosis complete, upstream
effect bug.)

---

## Step 1 — Tag editor ✅ (2026-08-23, fix round 2026-08-24)

Full MusicBee-style editor. Motivation: real albums misbehave from source
tag issues (wrong albumartist, wrong year, split albums).

**Rust — `src-tauri/src/library/tags.rs` (new)**:
- `get_track_tags(track_id)`: path from DB → `lofty::read_from_path` →
  fields: title, artist, album artist, album, year, track n + total, disc
  n + total, genre, composer, label (ItemKey::Label), comments, grouping
  (ItemKey::ContentGroup). Tagless files → empty fields.
- `get_album_tags(album_id)`: album-level consensus (first non-empty per
  field across tracks, flagged when tracks disagree) + per-track list.
- `save_track_tags` / `save_album_tags`: `primary_tag_mut()
  .or(first_tag_mut())` → `remove_key` + `insert_text`, `save_to_path`
  with `WriteOptions::default()`. Tagless file → create tag via
  `FileType::primary_tag_type()`.
- After save: frontend triggers the incremental `scan_library` (edited
  mtimes guarantee re-parse through real grouping; `scan-finished`
  auto-reloads the frontend). NO parallel DB-update path, NO migration.
- Tests: write/read roundtrip incl. clearing (empty string/None removes
  keys — NOTE: clearing year must purge BOTH `ItemKey::Year` AND
  `ItemKey::RecordingDate`; the year() accessor falls back to
  RecordingDate), synthetic two-frame untagged MP3 (single frame fails
  lofty validation), and the "fix albumartist merges split albums" case.
  GOTCHA: stripping tags from a FLAC via `TaggedFile::remove` does NOT
  persist — FlacFile::write_to re-writes its internally-held
  VorbisComments; only editing existing/created tags works.

**Frontend**:
- `TagEditor.svelte`: glass modal (`--panel-bg-strong`, `te-*` namespaced
  CSS), MusicBee-style label+field grid (year / track-n-of / disc-n-of
  rows), disputed "•" markers. Dirty tracking → Save only on change; Esc
  cancels.
- `ContextMenu.svelte` (first context menu; `contextMenu.svelte.ts` store
  + TitleBar `.tb-pop` look): right-click track row → "Edit tags…", album
  header → "Edit album tags…".
- Album header: small always-visible edit button next to play-all.
- State: `ui.tagEditor = { open, albumId?, trackId? }`; modal in
  App.svelte.
- Regroup edge: $effect watches library.albums while open — if the edited
  album vanishes (merge/split), the editor closes and the expansion
  collapses gracefully.

**Fix round — 2026-08-24 (first real-library use exposed 3 bugs)**:
1. **serde case mismatch** (root cause of "save dropped my albumartist"):
   structs serialized snake_case (`album_artist`) while the frontend spoke
   camelCase (`albumArtist`). Unknown keys are ignored + `#[serde(default)]`
   ⇒ multi-word fields were silently EMPTY on save (albumartist wiped!)
   and showed "undef"/blank in the editor. Single-word fields (album,
   composer) worked — which is why the composer save seemed fine. FIX:
   `#[serde(rename_all = "camelCase")]` on TrackTags/AlbumTags.
2. **Stacked-ID3v2 silent-write no-op** (why "nothing happens on save"):
   some real MP3s (Lavf52-era muxers) carry SEVERAL ID3v2 blocks. The
   reader MERGES them into one view; `save_to_path` rewrites only the
   FIRST block and returns Ok — a later block's stale frames win on reread,
   so edits vanish silently. Diagnosed by hexdump (tag #1 = 187 KB holding
   just TCMP, tag #2 = 370 B holding the real frames). FIX: `write_file`
   now VERIFIES by rereading after every save; on mismatch
   `repair_stacked_tags` strips every on-disk tag of that type and
   rewrites the merged+edited tag as the only one (lossless — the merged
   view already is the union). Regression test builds a synthetic stacked
   MP3 and proves plain lofty save loses the edit while our path lands it.
3. **Album modal per-track list removed** (user decision — numbers showed
   "undef" pre-fix and album saves have no business touching per-track
   fields): `save_album_tags` lost its `tracks` param and NEVER writes
   title/artist/numbers; per-track edits are track-mode only (hint added).
   TrackRow struct deleted.
4. **Retag-adoption in scan** (`scan.rs`): a changed group whose (artist,
   norm title) matches an EXISTING album adopts its id regardless of year
   — year is metadata, never identity (documented intent, now actually
   true). Needed because the incremental scan skips the target album's own
   files, so pass-3 cross-dir merge can't see them. This is what let the
   Stereopony fix join the 218-track compilation (file year 2012 vs album
   2019). Test: `retagged_stray_adopts_existing_album_regardless_of_year`.
5. **Track ordering rule** (user decision): within an album disc, tracks
   with NO track number sort alphabetically BEFORE numbered tracks.
   Implemented in SQL (`ORDER BY album_id, disc, (track IS NOT NULL),
   track, title`) in get_library dump, play_album (playback order = display
   order) and tags::album_rows. Test:
   unnumbered_tracks_sort_alphabetically_first.
6. **Data restoration**: the buggy album save had wiped track numbers on
   all 12 Edge of Tomorrow files (rows serialized as null → remove_key).
   Restored from the `(NN)` filename prefixes via one-off write_file run;
   user verified in-app after rescan. User verified everything live:
   Stereopony stray merged into Anison no Kokoro (218 tracks), numbers
   back, modal clean. Gates: cargo 30, check 0/0, vitest 10, build OK.

---

## Step 2a — Import music ✅ (redesigned 2026-08-24: two-step staging flow)

> **Redesigned 2026-08-31: staging is a row flag, not a folder.** Everything below that describes a copy into `<cache>/import/` is the
> copy era; the two-step decision (import to listen, save to own) is unchanged, the *where* is not. `import_music` now indexes the user's
> files **where they are** and sets `tracks.staged` (migration v2), so a pending album is playable *with* cover art, waveform and play
> counts instead of being an alien under the cache dir — and a scan leaves it alone rather than mourning it as missing. Save **moves** the
> file into its resolved folder and re-points the row (`scan::relink_track`, history intact); Discard **forgets the row and deletes no
> file**, because deleting a file the user never asked us to delete is how 28 ghost rows were born in one session. Three ownership rules, all
> user-decided: a file that lands on a name the destination folder already holds byte for byte deletes the copy the user just pointed at and
> says so (a collision rule, not library-wide dedupe — see AGENTS.md); a file already indexed is never staged; a folder the app did not write
> is never removed. `import_music` returns `ImportReport { staged, already }` and the window opens to show it. The staging root survives only
> as the place legacy rows are flagged at startup and pruned from; `import_paths` and `KnownFiles` are deleted. Details: **AGENTS.md**
> "Import staging", **DESIGN.md** "Import staging, and the window that owns it", and the "Settings stack…" / "Import staging moved…" sections
> at the foot of this file.

**Flow (user decision)**: importing is TWO separate steps —
1. **Import** = copy into a staging area; tracks are playable immediately
   but live OUTSIDE the library dir. For listening/trying out.
2. **Save to library** = copy staged files into `<musicDir>/…` (per album,
   per track, or a global "save everything staged"). Until saved, staged
   files PERSIST (survive restarts); "Discard" deletes them.

**Decided**:
- Staging root `<app_cache_dir>/import/<sourceFolderName>/` (source folder
  name preserved; loose files keep their parent dir's name). Import COPIES
  (originals stay at source). Collision: same-size ⇒ skip; different size
  ⇒ " (2)" suffix (never overwrite).
- Import skips files whose (lowercased name, size) already exists in the
  library DB (existing files only — stale rows of deleted files never
  block) or the staging area ⇒ "add file then add its folder" can't dup.
- Save layout (user decision 2026-08-24): `<musicDir>/<Artist>/<Album>/`
  (artist/album from the DB row; '/' sanitized; deeper staging structure
  kept). Same-name-same-size dedupe unchanged.
- Scanner walks musicDir + import root in ONE run (`run_scan` takes roots).
- "Staged" is COMPUTED at dump time (any track path under import root) —
  no schema migration. Albums with staged tracks get an "Imported" badge;
  staged albums render mixed in the grid (user decision).
- Cross-root album merge (pass 3) stays ENABLED: re-importing an album
  that's already in the library merges into it; the badge then marks the
  staged files. Save copies staged tracks out + deletes staging originals
  + rescan (no duplicate-track window).
- Save granularity: per album (context menu), per track (context menu),
  and global "Save imported music" in the menu (saves ALL staged — "scan
  current content, save what isn't in the library dir"). Same-name-
  same-size files in musicDir are skipped (dedupe).
- Discard granularity mirrors save (album / track / all).
- Pickers: kdialog ONLY (AGENTS rule) — `--getopenfilename --multiple` for
  files, `--getexistingdirectory` for folders. Drag-and-drop onto the grid
  as a second import path (Tauri file-drop), same copy logic.
- Copy emits `scan-progress` (phase "import"); then incremental scan →
  `scan-finished` refresh.

**Landed**:
- Rust: `import_dir()` helper; `run_scan_roots` multi-root; staged flag
  computed in the dump; `import_music(paths)` (files + folders,
  collision-safe: same-size skip / " (N)" suffix); `choose_import_files` +
  `choose_import_folder` (kdialog, `--separate-output` for multi-file).
- Rust: `save_imports(album_id?, track_id?)` / `discard_imports(...)`;
  save re-roots the staging relpath into `<musicDir>`, deletes staged
  originals, prunes empty staging dirs. Tests: layout, collision,
  save+dedupe, discard (cargo 34).
- UI: menu "Add music files…" / "Add music folder…" / "Save imported
  music" (disabled when nothing staged); Imported badge (grid tile +
  panel header); context-menu Save/Discard on staged albums + tracks;
  Track type now `track: number | null` (unnumbered tracks render blank).
- Drag-and-drop import (`onDragDropEvent` → same staging flow).
- play_album refuses to ghost-play a track whose file is missing (clean
  error instead of mpv error-skipping past a stale UI state).
- **MISSING-FILE LIFECYCLE** (user decision 2026-08-24): the scan NEVER
  auto-deletes rows whose files vanished — they are flagged `missing` in
  the dump (existence check), shown with an amber alert icon in the track
  number slot. Double-click (or right-click → "Locate file…") opens a
  kdialog picker → `relink_track` re-links the row (files outside the
  library dir are copied into `<musicDir>/<Artist>/<Album>/` first; mtime
  zeroed so the next scan re-parses tags). "Remove from library" (track
  menu, Delete key) and "Remove missing tracks" (album menu) delete rows
  explicitly. ScanCounts gained `missing`; `removed` is now always 0.
  Play failures on vanished files (pre-rescan) flag the row missing
  immediately and open the locate dialog directly — no silent no-op.
  USER-VERIFIED 2026-08-24 (import/save/discard/relink/missing flows).
- ✅ 2026-09-04 (badge-modal Save, 12 files) Visual pass on the real library (user) — still open, see *Deferred*.

---

## Step 2b — Playbar gradient toggle ✅ (2026-08-24, user-verified)
- Sidebar gear setting "Playbar artwork gradient" (checkbox; persisted via
  settings write-through + localStorage mirror, default OFF).
- When on + a track loaded: playbar backdrop = 135° two-stop gradient from
  current album's colorC1/C2 via new shared `src/lib/gradient.ts`
  (`artGradientContrast`, same mix-to-surface math the panel uses —
  extracted from ExpandedPanel, which now imports it), alpha 0.7 = chrome
  exactly. Falls back to plain chrome when stopped.
  `transition: background 400ms`.
- **CONTRAST GUARANTEE** (user request): every stop is lightness-clamped
  (hue/sat preserved) until the stop composited at 0.7 over the theme
  surface clears WCAG 4.5:1 vs `--text` AND 3:1 vs the dimmed variant
  (`--text-dim` @ 0.64). Dark theme pushes stops darker, light theme
  lighter; passing stops untouched. `gradient.test.ts` (12 vitest cases:
  extreme covers × both themes).
- Gates 2026-08-24: check 0/0, vitest 22, cargo 35, build OK. User
  verified live (gradient follows albums, text always legible).
---

## Step 3 — Menu bar + Plasma Global Menu + window identity ✅ (2026-08-24)

**Menu model**: Rust-OWNED (source of truth), frontend fetches via command
+ `menu-action` events keep it fresh. Menus: **Playback / Library / View /
Help** (DECIDED).
- Playback: Play/Pause (dynamic label), Stop, Previous, Next
- Library: Rescan Library, Add Music… (step 2a), Choose Music Folder…
- View: theme toggle (checkable)
- Help: About Songstress

**Landed**:
- `src-tauri/src/menu.rs`: Rust-owned model (ids `playback.play-pause/-
  stop/-previous/-next`, `library.rescan/rescan-full/add-files/add-folder/
  save-imports/choose-folder`, `view.theme` (checkable), `help.about`).
  Dynamic bits arrive via `set_menu_state` (MenuState:
  playing/hasTrack/scanning/anyStaged/themeDark — pushed by an $effect in
  App.svelte from the stores); every push re-emits `menu-changed` with the
  rebuilt model. `menu_activate` = ONE activation path: playback ids
  handled natively via the engine (toggle/stop/jump), everything else
  re-emitted to the frontend as `menu-action`. 3 cargo tests.
- `src/lib/stores/menu.svelte.ts`: mirrors the model (`menu-changed`),
  dispatches `menu-action` to scanner/ui stores. **TitleBar.svelte**: text
  menu bar after the traffic lights (classic hover-switches-while-open, ✓
  checkmarks, disabled gating, Esc/focusout close). **The menubar HIDES
  once the Global Menu registers** (poll `appmenu_active` +
  `appmenu-registered` event, either order wins); hidden FROM THE START
  (globalMenuActive defaults true — no startup flash) and only APPEARS if
  registration definitively fails (`appmenu_state` 2, polled at 1s).
  Hamburger REMOVED entirely (user call — Global Menu or titlebar bar,
  never both). Gotcha: TitleBar has TWO `{#if library.live}` blocks — the
  menubar nav and the hamburger popover; an early hide-edit landed in the
  wrong one.
- **Plasma Global Menu** (`menu_dbus.rs` + `wayland_appmenu.rs`): serves
  `com.canonical.dbusmenu` at `/org/songstress/Menu` as
  `com.yossi.songstress` (zbus); registers with KWin via
  `org_kde_kwin_appmenu` Wayland protocol — foreign wl_display/wl_surface
  adopted from raw-window-handle in guest mode
  (`Backend::from_foreign_display` + `ObjectId::from_ptr`), parked thread
  keeps the binding alive. Panel widget shows Playback/Library/View/Help
  (+ the applet's Search entry) — user-verified.
- **War stories (do not rediscover)**:
  - KWin links an appmenu object to its window ONLY at `appMenuCreated`
    time and only if the window is already MAPPED. At launch we race the
    first buffer attach → create/release/retry loop, stopping as soon as a
    dbusmenu consumer queries us (LAST_QUERY_MS touched in GetLayout/
    AboutToShow) — each re-register otherwise flaps the panel widget via
    KWin's transient `application_menu("service","")` two-step emissions.
  - Wire format MUST match Qt's exporter byte-for-byte: layout struct is
    `(i, a{sv}, av)` with EACH CHILD VARIANT-WRAPPED (`av` of `v`), and the
    GetLayout reply's second out arg must be the STRUCT itself, not a
    variant (`(u(ia{sv}av))`, NOT `(uv)`). KDE's DBusMenuImporter
    demarshals exactly that; anything else yields zero children → the
    applet brands us a "naughty app".
  - dbusmenu GetLayout depth semantics: 0 = item only, 1 = item + one
    level of children (Qt's importer fetches depth 1), negative = all.
  - AboutToShow MUST return true when the subtree is stale (revision
    changed since the consumer last fetched that id): KDE's importer only
    refreshes a submenu on open when AboutToShow says so or the menu is
    empty — a constant false freezes item enabled/checked state at first
    import (Playback stayed grayed while playing).
  - Debugging path that worked: WAYLAND_DEBUG=1 on our unit (wire truth),
    dbus-monitor on destination=our-service (import attempts), plasmashell
    unit journal with WAYLAND_DEBUG (what the panel receives), busctl
    side-by-side GetLayout against a working app (Konsole) for format diff.
  - Activation from outside the app (agent verification): `dbusmenu Event
    <id> "clicked" uint32 0 variant` (dbus-send: int32/string/variant/uint32
    order per the zbus signature `(isvu)`); there is NO `Activate` method.
- **Window identity** (so the widget renders icon + "Songstress" like
  Konsole): user-level desktop entry
  `~/.local/share/applications/com.yossi.songstress.desktop` with
  `StartupWMClass=songstress` (resourceClass is the binary name);
  desktop-file-validate clean, kbuildsycoca6 rebuilt. Icons 128/256 →
  `~/.local/share/icons/hicolor/.../apps/songstress.png`;
  gtk-update-icon-cache run. (Icon art itself = Tauri placeholder for now.)
  Window title stays STATIC "Songstress" (DECIDED — no now-playing
  title). Verified: panel renders icon + name + menus; Global Menu
  activation user-verified (Playback items track play state).

**Titlebar albums/songs search** (post-Step-3 feature): right-end field in
TitleBar (after the drag spacer — flex:1 on the spacer beats auto margins,
so the field must come AFTER it in DOM). `ui.mediaFilter` (session-only).
Results split into two labeled sections: **Songs** (albums containing
matching tracks; expanding one shows ONLY the matching songs via
ExpandedPanel's `visibleTrackIds` — playback indices stay anchored to the
full track list) and **Albums** (album-title match). An album can appear
in both; each section expands INDEPENDENTLY — `ui.expandedAlbum.
{songs, albums}` (two panels can be open simultaneously). Matchers in
`src/lib/search.ts` (`fold`, `albumTitleMatches`, `albumTrackMatches`,
`albumMatches`; vitest-covered, diacritic-insensitive). Esc or × clears;
"No albums or songs match" empty state. Gotcha: `.tb-balance` flex:1
absorbs free space before auto margins get any — don't rely on
margin-left:auto in the titlebar.
- Search scope fix: the Albums section matches ALBUM TITLES ONLY (artist
  names excluded — "hello" must not surface Helloween; artists have the
  sidebar search). Songs section matches track titles only.

---

## Step 4 — Accent color picker ✅ (2026-08-24)
- Gear popover "Accent" row: 9 swatches (stock = split purple gradient,
  selecting it stores null = reset) + custom via native `<input
  type=color>` styled as a conic-gradient swatch; selected ring = outline.
- ONE hex persisted (`ui.accentColor`, localStorage mirror + `accentColor`
  SQLite setting through the existing write-through).
- Pure math in `src/lib/accent.ts` (11 vitest cases): hex↔HSL, WCAG
  relative luminance, `accentVariants(hex, theme)` → lightness clamped
  dark [0.70,0.80] / light [0.45,0.60], `--active` at user-tuned alphas
  0.18 dark / 0.14 light, `--accent-text` white/#1b1b1f by luminance.
- App.svelte effect sets/removes `--accent`, `--active`, `--accent-text`
  on `<html>`; unset → removeProperty → stock purple from app.css. No
  token changes in app.css.
- User fix: `--accent-text` is for SOLID accent fills only — on
  translucent `--active` washes (playbar play/pause, sidebar active row)
  the glyph must stay `--text` (black accent-text on a faint wash read as
  "glyph turned black").
- Gates: check 0/0, vitest 43, cargo 39, build OK.

**Landed alongside (scanner, same session)**:
- **Grouping fix ("The Singles" bug)**: Edguy's folder tags albumartist,
  Phil Collins' tags none → Pass 3's "unknown matches anything" merged
  both into one 59-track Edguy album. Fix: subgroup release artist falls
  back to track-artist consensus ONLY when no file tags albumartist AND
  all track artists agree (mixed artists stay None → Various Artists).
  Regression test
  `same_title_untagged_folder_does_not_join_tagged_artists_album`.
- **Full Rescan (rebuild) menu item** (Library): `scan_library { full:
  true }` reparses every file — needed after grouping/consensus logic
  changes since skipped files never re-enter grouping. `run_scan_roots`
  gained a `full` flag.
- **Silent scan-killer bug**: a cancelled kdialog folder pick stored the
  JSON literal "null" as musicDir; `music_root`'s `unwrap_or(v)` turned it
  into a relative path "null" → every scan failed INSTANTLY with "music
  root null does not exist" — no progress events, no error surfaced, menu
  clicks "did nothing". music_root now treats null/empty as unset (→
  ~/Music). Also: SCAN_RUNNING flag reset now happens before the `?` on
  the join result (a panicking scan task can no longer wedge all future
  scans).

---

## Step 5 — Playback controls package ✅ (2026-08-25)

**5a. Shuffle & Repeat** — DECIDED: shuffle 4 stages (disabled / album /
artist / all artists); repeat 3 stages (disabled / album / track);
reshuffle on wrap; stages PERSIST.
**5b. Prev/next album buttons** — DECIDED: global grid order (artist A→Z
then year), wrap-around, no-op when stopped. Frontend-only: compute
adjacent album, call existing `play_album` at track 0.

**Landed**:
- **Play-order engine** (mpv.rs): `PlayState.order: Vec<OrderItem{trackId,
  path, albumId, albumIndex}>` replaces the parallel track/path arrays;
  the `{albumId, trackIndex, trackId}` event contract is preserved
  (albumIndex = position within the track's OWN album, so UI mapping
  survives pools that span albums). Order built per shuffle stage: off =
  album order; album/artist/all = clicked track first + Fisher-Yates
  shuffle (tiny xorshift RNG, no rand dep) over the album / the artist's
  albums / the whole library.
- **Repeat**: track = native mpv `loop-file=inf` (cleared on stage
  change); album = eof at last order entry restarts at 0, RESHUFFLED when
  shuffle is active; off = stop (stages survive stop — they're modes).
- **Mid-playback stage change**: `playback_set_shuffle` rebuilds the queue
  anchored at the current track WITHOUT restarting it (playlist-clear
  keeps the playing file, tail re-appended; repeat-off returns to plain
  album order positioned at the current track).
- **Persistence**: stages in settings (`shuffleStage`/`repeatStage`) +
  localStorage mirror; hydrated at launch in setup (incl. re-arming
  loop-file). MPRIS `Shuffle`/`LoopStatus` are now REAL and settable
  (Shuffle true → album stage if off; LoopStatus Track/Playlist map to
  the repeat stages) with property-changed signals.
- **UI**: PlayBar — album prev/next buttons flanking the transport
  (⏮album/album⏭, global grid order, wrap-around, no-op stopped, frontend
  computed via `lib/albumOrder.ts` + tests); shuffle (4-stage, badge dot)
  and repeat (3-stage, "1" overlay) buttons left of the volume; Playback
  menu gained Shuffle:/Repeat: cycling checkable items + Previous/Next
  album items (frontend-handled via menu-action).
- **Post-landing fixes (same day)**:
  - mpv playlist is a rolling 32-track WINDOW (PLAYLIST_WINDOW), not the
    whole order — all-artists pools are ~4.4k entries and one IPC command
    per entry floods mpv (later commands queue past the 10s timeout →
    "mpv response timeout" crash look). eof advance tops the window up by
    1.
  - **NEVER await IPC commands in the mpv reader task** — it is the task
    that reads responses; awaiting one stalls all reads until timeout (the
    repeat-album eof reload took 20s = 2 chained timeouts while mpv had
    executed instantly). All reader-task side effects are fire-and-forget
    (`fire()`); only command-side tasks await.
  - Repeat-album wrap is PRE-ARMED: entering the last track (natural eof,
    direct click via play_order, or set_repeat while on the last track)
    appends the next pass (reshuffled when shuffle is on) so mpv wraps
    GAPLESSLY — it never goes idle, so the audio device never reopens
    (the reopen was the 3-4s gap). Promoted to the live order at the last
    track's eof; jump/set_shuffle discard or promote the armed pass.
  - Mode buttons (shuffle/repeat): ON = full-brightness glyph (hover
    color) + stage badge (A/R/ALL, "1" for track repeat), OFF = dimmed;
    user found the accent-wash background indistinct and the first
    hand-drawn glyphs broken — Feather icon geometry instead.
- Gotchas: std MutexGuard is not Send — the mpv eof handler decides inside
  a scope and acts AFTER the guard ends (generator Send bound);
  `play_order` must read stages BEFORE taking the write lock (no
  reentrant locks).
- Gates: check 0/0, vitest 48, cargo 41, build OK.

---

## Step 6 — Equalizer ✅ (2026-08-25)
DECIDED: 10-band graphic + preamp; controls BOTH in playbar popover AND
Playback menu.
- **Backend** (`src-tauri/src/eq.rs`): pure `af_chain(&Eq) ->
  Option<String>` (disabled or all-flat → None → clears mpv's `af`;
  preamp `volume=<n>dB` only when significant, then 10 octave-width
  peaking `equalizer` filters, ±12 dB clamp, 0.1 dB significance
  threshold, `{:.1}` formatting — mpv accepts "4.0"). `Eq { enabled,
  preamp_db, gains[10], preset }`, serde camelCase. 7 unit tests.
- **Command** `playback_eq(state, engine, eq)`: clamps → pushes chain to
  mpv (`Mpv::set_af`, "" = cleared) → persists the clamped state in
  settings key `"equalizer"` → returns the normalized state.
- **Launch re-apply**: Rust setup reads the settings key and re-applies to
  mpv (after the shuffle/repeat restore block); the frontend ALSO hydrates
  the UI via new `initEq()` in playback.svelte.ts (DB wins over the
  localStorage mirror, same rule as ui.* settings) — without it the
  popover showed Flat while mpv had the chain (caught by seeded-value
  screenshot).
- **UI**: PlayBar EQ button (sliders glyph, left of shuffle) → glass
  popover `.eq-pop`: on/off toggle, preset `<select>` (divergence →
  "Custom"), preamp column + divider + 10 vertical band columns (dB
  readout above, freq label below, double-click zeroes). Vertical sliders
  = horizontal ranges rotated -90° in a fixed 26×120 `.slot` — **WebKitGTK
  2.52 ignores writing-mode/direction on range inputs** (thumbs render but
  never move; cost one debugging round).
- **Menu**: the menu model has no submenus → three flat items appended to
  Playback: "Equalizer: On/Off" (checkable), "EQ Preset: <name>" (cycling,
  gated on enabled), "Customize Equalizer…" (opens the popover via
  `ui.eqOpen`, session-only flag). MenuState gained `eq_enabled`/
  `eq_preset`; App.svelte $effect tracks both. `eq_items_follow_state`
  cargo test.
- **Presets** (`src/lib/eq.ts`, 10): Flat, Rock, Pop, Jazz, Classical,
  Electronic, Hip-Hop, Bass Boost, Treble Boost, Vocal Boost;
  `matchingPreset` → "Custom" on divergence. 6 vitest cases. Preset
  recheck (user request): Rock was Winamp's aggressive scooped-mids curve
  (−8 @250, +11 top) → gentler modern V [5,4,2.5,0,-2,-1,1.5,3.5,5,5.5];
  Pop de-honked (mid-hump → gentle smile); Hip-Hop's boxy +3 @250 removed.
  Jazz/Classical/Electronic kept (Winamp shapes, still standard).
- Verified end-to-end: seeded non-flat EQ → restart → mpv reports exactly
  `volume=2.0dB,equalizer=f=31:...g=6.0,...f=16000:...g=-4.0`; popover
  renders hydrated values with thumbs off-center; reset to disabled/flat →
  `af` empty again. Gates: check 0/0, vitest 53, cargo 48, build OK.
- Not hand-verified (no click-injection tools): slider dragging, preset
  select, menu item clicks — logic covered by tests + screenshots.

---

## Step 7a — "Play next" queue ✅ (2026-08-25)
DECIDED: queue lives in Rust `PlayState` (Rust owns playback truth).
Entries play BEFORE the album order resumes; the queue is NEVER shuffled;
session-only (no persistence). MPRIS Next/Previous RESPECT the queue
(user call). Album clicks and stop clear it.
- **Design**: `PlayState.queue: VecDeque<OrderItem>` + a `from_queue`
  flag — while a queued entry plays it is PROMOTED into `order` at the
  current slot (so `playback-changed`/MPRIS metadata stay correct) and
  consumed from `order` at its own eof; `queue` only holds not-yet-played
  entries.
- **Pure eof choreography** (mpv.rs `eof_advance`): the end-file handler
  decision was extracted out of the reader task into a pure fn returning
  an `EofAction` — 8 unit tests cover promote/consume/chain/window-
  top-up/pre-arm/wrapped-promotion. Playlist invariant:
  `[current] ++ queue ++ order[index+1..index+31] (++ wrap pass)`.
  Promotion does NO window top-up (mpv consumed a queue entry, the order
  window didn't slide — appending would DUPLICATE the last entry); the
  eof test for the 40-track case pins this. Edge the tests caught: a
  queue entry as the LAST order entry leaves the index out of bounds on
  consumption → promote the pre-armed wrap pass or stop.
- **Rebuild path**: `load_queue` appends queue entries BETWEEN the current
  track and the order window and re-appends an armed wrap pass (single
  append path — play_order now arms BEFORE load_queue instead of firing
  duplicates after). Enqueue/remove/jump-to-queue all go through this
  rebuild; eof promote/consume stay surgical (0–1 appends, fire-and-
  forget).
- **Jump semantics**: Next consumes the current queue entry (if any) and
  promotes the front; falls into the order when drained; Previous from a
  queue entry returns to the order entry before it. MPRIS Next/Previous
  respect the queue for free (they call `jump`).
- **Interplay**: repeat=track untouched (native loop); repeat=album wrap
  arms only when the queue is empty (`pre_arm_if_last` + set_repeat
  guard); `arm_wrap` excludes a playing queue entry; shuffle reshuffles
  the order only.
- **Commands/UI**: `playback_queue {trackIds, front}` / `_remove` /
  `_jump` (+ `tracks_by_ids` — input order preserved, per-album canonical
  album_index); context menus (track: Play next / Add to queue; album
  header: Play album next, non-missing tracks, album order); PlayBar
  queue button (count badge) → glass popover with queued rows (click =
  play now, discarding earlier entries; × = remove) + dimmed "UP NEXT"
  preview; `queue-changed` event {queue, upNext} mirrors it. Enqueue
  while STOPPED just stores (never auto-plays; shown in the popover).
- Gates: check 0/0, vitest 53, cargo 56, build OK; live-verified (popover
  rendered real queue-changed data mid-playback). Not hand-verified (no
  click injection): enqueue/remove/jump clicks — logic is unit-tested.

---

## Step 7d — Live library watching ✅ (2026-08-25)
DECIDED: `notify` (inotify), one recursive watcher per root, only while
the app runs. Events → 2s debounce → incremental rescan (reuses
SCAN_RUNNING via a dirty flag, rescan once after). Minimum 5s between
rescans so bulk copies don't thrash. Silent auto-refresh via the existing
scan-progress/scan-finished events; staged imports need no special-casing
(they end in a scan). Network mounts unsupported (inotify is local-only)
— manual Rescan remains; document it.
- `src-tauri/src/watcher.rs`: notify recommended_watcher on a named
  thread; quiet-period debounce (recv_timeout loop keeps extending while a
  burst pours in). A failing inotify backend (network mounts) logs and
  disables watching — manual Rescan stays the path there.
- lib.rs wiring: the callback only sets `WATCH_DIRTY`; a 500ms polling
  async task scans when dirty AND `SCAN_RUNNING` clear AND ≥5s since the
  last watcher scan (`WATCH_MIN_INTERVAL`). Events during a scan keep the
  flag set → exactly one follow-up scan after it settles. Scans are
  incremental + silent.
- Watched root = music root at launch; a folder change resumes watching
  after the next app start (superseded by Step 7c's `rewatch`).
- Verified live: touch → rescan in ~2s (61ms scan, 4416 skipped, 0
  changed). Baloo may re-touch xattrs after an mtime change → one extra
  spaced rescan (min-interval handles it; each costs ~60ms).
- ⚠ The callback originally counted ANY event kind — that self-feed loop
  was found and fixed during Step 7c verification (2026-08-26); see 7c
  below and the AGENTS.md gotcha.

---

## Step 7c — Multi-library roots ✅ (Rust 2026-08-25, frontend + verification 2026-08-26)
DECIDED (2026-08-25): migrate `musicDir` → `musicDirs` (JSON array),
reading old `musicDir` as migration fallback on first launch. Scanner
loops all roots (blake3 IDs → no cross-root collisions; the deletion
sweep's "remove paths that NO root contains" already handles root
removal). Library menu → "Music folders…" glass modal: roots list, Add
(kdialog picker) / Remove per row; removing a root drops its tracks from
the library, files NEVER touched; removing the last root returns to
EmptyState. Staged imports keep targeting the primary root; folder picker
starts at the most recent root → ~/Music. Tests: multi-root merge, orphan
sweep on root removal, migration.

**Rust (2026-08-25)**: `musicDirs` AUTHORITATIVE settings key + one-time
setup migration from legacy `musicDir`; `get_music_folders` /
`add_music_folder` (kdialog when path=None, `validate_new_root` rejects
duplicate/nested roots) / `remove_music_folder` (drops the root's tracks,
deletion sweep reaps empty rows); `rewatch()` re-arms the watcher on root
changes; 4 cargo tests (`roots_from_settings_migration_and_multi_root`,
`validate_new_root_rejects_overlap`, `persist_roots_round_trips_through_
settings`, `root_removal_drops_only_tracks_under_it`).

**Frontend (2026-08-26 — finished a half-landed wiring; svelte-check had 7
errors)**:
- `App.svelte` now renders `<MusicFolders />` (the modal existed but
  nothing opened it — `Library → Music folders…` set the flag, nothing
  listened).
- `menu.svelte.ts` dropped its unused root imports (roots are managed in
  the modal, not the menu — `library.add-folder` is the Step 2a IMPORT
  path, a different feature with a confusingly similar label).
- `EmptyState.svelte` migrated off the removed `ui.musicDir` /
  `chooseMusicFolder` onto `ui.musicFolders` + `addMusicFolderRoot`
  (fresh install = "Pick the folder that holds your music…"; one folder =
  names it; several = "your N music folders"; Rescan ghost only with
  roots; buttons disable while a scan runs).
- Add/remove errors route through `notifyError` so `validate_new_root`
  rejections are visible (they previously hit the console only).
- **In-glass remove confirm**: the modal's native `confirm()` would have
  rendered a GTK dialog (AGENTS.md forbids). ✕ on a row arms an inline
  confirm row (Remove / Cancel); it disarms on close or on any list change
  so a stale confirm can never fire on the wrong folder.

**Watcher bugs found while verifying (2026-08-26)**:
- **Self-feed loop (live, 24h old)**: one pointless rescan every 5s, 24h
  straight (7723 rescans, ~60ms each, zero changes). Raw inotify capture
  (all 322 dirs, IN_ALL_EVENTS) proved every event was OPEN/ACCESS/
  CLOSE_NOWRITE — the scan's own walkdir reads: notify's inotify backend
  watches IN_OPEN, and the callback counted ANY kind. `watcher.rs` now
  filters through `marks_library_dirty`: Access events are noise (reads
  never change the library); Create/Remove/Name/Data/Metadata count
  (Metadata is where a plain `touch` lands — the documented poke — since
  the scan keys on mtime+size); Any/Other fail open. Verified live: 0
  scans in 2 min after the fix (was 24/min).
- **Dropped WatchHandle never stopped its thread**: the loop only matched
  `Ok` on the stop channel, so a dropped handle (what `rewatch` does on
  every root add/remove) leaked a thread and a full recursive inotify
  watch. Disconnected now exits the thread; the `stop()` method is
  retired, tests drop the handle.
- Tests: `read_traffic_does_not_rearm_the_watcher` (walk+read loop must
  stay silent, ends with a real write as the armed-control) and
  `access_is_noise_everything_mutating_counts`.
- Gates: check 0/0, vitest 53, cargo 65 (0 warnings), build OK.
- Live-verified: Global Menu → Library → "Music folders…" driven over DBus
  (dbusmenu `Event 205 "clicked"`) → modal rendered with the real root +
  "Add folder…" (screenshot; no click-injection tools). NOT verified by
  hand: the confirm row (arms on ✕ click) and EmptyState with an empty
  library — user check: Music folders… → Remove all → EmptyState
  headline/buttons → re-add.

---

## Step 8 — Titlebar elimination: chrome moves into the sidebar 🔶 (2026-08-27)
DECIDED (2026-08-27, supersedes the Step 3 "Global Menu XOR in-titlebar bar"
row for the in-app surface): the 44px titlebar is gone. New arrangement:
- Traffic lights (KWin-palette Klassy dots) → sidebar header, TOP-LEFT;
  gear button TOP-RIGHT, lights and gear side by side.
- App title ("Songstress") removed from the sidebar.
- ONE search field ("Search library...") in the sidebar: filters the artist
  list AND drives the grid's Songs/Albums sections (old `ui.filter` +
  `ui.mediaFilter` merged into `ui.search`; clicking an artist row clears
  it). Titlebar's media search deleted.
- Menu: gear opens the app menu (Playback/Library/View/Help + Appearance
  will live inside it). **Gear popover is a placeholder for now** — the real
  menu UI is the next task. Plasma Global Menu stays the live menu and is
  unaffected. The old in-titlebar menubar fallback rendering is gone.
- Window drag regions: the sidebar header + a 20px strip over the grid's
  top padding (`.drag-strip` in App.svelte) — covers the grid's 20px
  content padding; scrolled content can't be grabbed in its top 20px.

**Landed (2026-08-27)**: `TitleBar.svelte` deleted; its traffic-light /
chromeVars/layout code moved into `Sidebar.svelte` (classes keep the
`tb-*` namespace, styles re-scoped there); `Sidebar` header rebuilt
(lights left, gear right, `data-tauri-drag-region` on the header, gear
toggles a glass placeholder popover, closes on outside pointerdown /
Escape); `App.svelte` loses `<TitleBar />`, gains `.drag-strip` (z-5 over
the grid, below sidebar/playbar z-10); `ui.svelte.ts` merges `filter` +
`mediaFilter` → `search`; `AlbumGrid` reads `ui.search`; `--titlebar-h`
token dropped from `app.css`. Gates: check 0/0, vitest 53, build OK;
cold-restart + screenshot verified (lights, gear, search, full-height
grid).

**Menu stack (Step 8b, 2026-08-27)**: the gear's placeholder popover is
replaced by an iOS-Settings-style stack in the sidebar (apple-design skill:
transform-only slide, standard curve, parallax recede). Three absolute
layers in `Sidebar .stack`: home (search + artists) → root ("Menu" navrow +
Appearance/Playback/Library/View/Help rows with chevrons) → detail (navrow
with accent back-chevron + title; Appearance pane = the old gear-popover
settings; other panes render the live Rust `menu.menus` model via
`menu_activate`). Gear (icon flips to ✕) opens/closes; back-chevron or
Escape pops one level. State: session-only `ui.menuOpen` / `ui.menuDetail`
(`"appearance"` or a menu id). Root recedes -30% + 0.85 opacity behind an
open detail. Gates: check 0/0, vitest 53; root + closed states verified by
screenshot (detail-pane visuals + the push/pop motion are for the user's
hands). Swipe-back gesture deferred (CSS transitions aren't grabbable;
would need a spring-driven drag layer).

**Menu iteration (user-feedback driven, same day)**:
- parallax recede -30% → full push: on a flat glass sidebar the peeking
  root items read as clutter ("I can still see Appearance").
- root divider removed; left back-chevron added to the root title; right
  chevrons removed from rows (user spec).
- ✕ semantics: one-level pop → ALL levels; then two-phase pop (root
  centered intermediate) → single conveyor: home in from left, root out
  LEFT in parallel (same speed, no crossing), detail out right. Root
  teleports -200%→+100% off-screen (one no-anim frame) so re-open always
  enters from the right. Close timer 340ms.
- **BUG (found by user): home layer had no pointer-events grant — the
  artist list was click-dead.** `.layer` defaults to none; grant it to
  every on-screen layer (`.layer:not(.off)`), not just menu `.active`s.
- Polish batch (emil-design-eng audit): 400→320ms layers, `:active`
  press wash on rows/chevrons/gear, gear↔✕ crossfade+90° rotate 160ms,
  back-chevron hit target 18px→28px, explicit `:focus-visible` accent
  rings, menu rows 38→40px (deliberate cadence), dead opacity transition
  removed. Stagger-on-entry audited but deferred (could read busy at 236px).

**Critique pass (impeccable, 2026-08-27): sidebar scored 25/36 Acceptable**
(first run, snapshot in `.impeccable/critique/`). Fixed in sequence:
- P1 search dead end: "All Artists" row is now ALWAYS visible (it's a
  control, not a result) + clear (×) button in the field when non-empty
  (old titlebar pattern; × glyph absolute-centered — WebKitGTK button
  place-items drift, same fix as the traffic lights).
- P2 diacritics: artist filter now uses `fold()` like the grid ("bjork"
  finds Björk in both surfaces).
- P2 a11y: search input `aria-label="Search library"`.
- P2 gear IA: user chose option A + rename — the root pane is titled
  "Settings" (not "Menu"); Appearance stays its first row; aria-labels
  follow ("Open settings"/"Back to settings"); DESIGN.md updated.
- P3 (landed): `.backchev` 6px→7px (on-scale); sidecar colorMeta gained
  `traffic-light-border` (rgba(0,0,0,0.18), system-mirror chrome) and the
  four `picker-spectrum-*` conic colors (functional rainbow) as documented
  exceptions — detect.mjs on Sidebar is now clean (exit 0).
- P3 (landed, round 2): keyboard accelerators — `/` focuses the library
  search (inert while typing, while the Settings stack is open, or with
  modifier keys), `s` toggles the Settings stack, Escape in the search
  field clears the query then blurs (old titlebar behavior, preserved
  through the merge). Re-critique: 25 → 33/36 (Good), detector 0 findings.
- P3 (landed, round 3 — the re-critique's three items): accelerator hints
  as `title` tooltips (search input: "/ to focus — Escape clears"; gear:
  "Settings — press s to toggle"), "No artists match." upgraded from inert
  caption to a clear-search action button (`.empty` voice), and a transient
  11px dim "Updating library…" note under the field while `scanner.running`
  with a populated library (aria-live polite). Sidebar re-critique: 33/36
  (Good), detector 0 findings.
- **PlayBar critique (2026-08-27, same evening)**: 30/40 (Good). Empty-state
  copy fixed ("Pick a track"), then the critique ran. Findings carried into
  Step 8c below — NOT yet fixed.
- **RPM rebuild + install (user, 2026-08-27 20:47)**: rebuilt after the final
  sidebar (20:29) and playbar (18:41) edits; user ran the dnf lines.
  Step 8 complete.

---

### Step 8c — PlayBar critique fixes — ✅ (from the 2026-08-27 critique, 30/40; re-critiqued 35/40)

Snapshot: `.impeccable/critique/2026-08-28T00-36-27Z__src-components-playbar-svelte.md`.
Work top-down, one gate pass after each:
1. ✅ **P2 popover mutual exclusion** (2026-08-29): EQ and play-next queue popovers share
   the same anchor (`right:14px; bottom:calc(100%+10px)`) and toggle
   independently → they overlap. Opening one must close the other.
2. ✅ **P2 standard popover dismissal** (2026-08-29): Escape + outside-click
   close the popovers — the ContextMenu idiom (`svelte:window` pointerdown +
   `el.contains`). GOTCHA: the toggle button's own pointerdown must be
   exempted or the popover becomes open-only (pointerdown closes, then the
   click re-toggles it back open). Popovers now carry `role=dialog` +
   aria-label.
3. ✅ **P2 volume mute target** (2026-08-29): ~19px → 28px box (invisible at
   rest; same bar as the sidebar back-chevron).
4. ✅ **P2 in-app keyboard transport** (2026-08-29, shaped per impeccable
   `shape` + apple-design): Space = play/pause, ←/→ = seek ±5s. NO ↑/↓
   volume (restraint: critique's own scope; volume has slider + media keys).
   Inert rule: acts only when focus is on the body — any interactive target
   (input, select, textarea, button, a, [contenteditable]) keeps native key
   behavior; Space ignores auto-repeat, arrows repeat on purpose (like
   dragging the seekbar). Advertised via `title` tooltips ("Play — Space",
   "Seek — ←/→ = ±5s"), same hover-only micro-help voice as the sidebar
   hints. No collision with `/`/`s` (different keys; those stay inert in
   inputs — same rule shape).
5. ✅ **P3 document + drift** (2026-08-29): DESIGN.md gained the
   "Instrument" type tier (400, 8–9.5px tabular — EQ dB/freq readouts and
   mode-button badges only; smallest tier, nothing else may use it) in the
   YAML frontmatter AND the prose hierarchy; EQ preset select radius
   6px → 8px (the control rung — 6px was the last off-ladder radius in the
   popovers).

After the items land: re-critique the playbar (DONE 2026-08-29: 30 → 35/40,
Good 87.5%, detector 0 findings/exit 0; snapshot in .impeccable/), then
rebuild the RPM (user installs). The three surviving P3s all landed same
day:
6. ✅ **P3 popover materialize entrance** (2026-08-29, apple-design
   "materialize, don't just fade"): 140ms scale 0.98→1 + opacity,
   `cubic-bezier(0.22, 1, 0.36, 1)`, `transform-origin: 100% 100%` (popovers
   sit above their trigger, so they grow from the shared bottom-right
   corner); transform+opacity only; the global reduced-motion switch
   collapses it. Entry only (unmount = instant exit, P3 scope).
7. ✅ **P3 clear-queue-all** (2026-08-29): "Clear" text button in the queue
   popover header when non-empty; new `playback_queue_clear` command
   (mpv.rs `queue_clear` mirrors `queue_remove`: clear → rebuild tail while
   playing → `queue-changed` mirror).
8. ✅ **P3 "ALL" badge jargon** (2026-08-29): "ALL" → "All" (plain word, not
   shouty jargon; the hover title carries the full explanation).
Detector config: `design-system-font-size` ignore-values 8px + 8.5px =
documented Instrument-tier exceptions (.impeccable/config.json, same route
as the sidebar colorMeta exceptions); the two 16px close glyphs were fixed
for real (→ 14px body, on-ramp) rather than ignored. Gates: check 0/0,
vitest 53, cargo 65, build OK, detector exit 0, screenshot clean.

**Icon consistency pass (impeccable polish, 2026-08-29)**: audited the
playbar's three glyph sources (pixel-cropped evidence) and landed:
- P1 track-skip icons carried a DEAD bar subpath (fill-only path, zero
  area — the bar never rendered; the triangle-only look was accidental).
  Now intentional: bare triangle = track step, bar+triangle = album jump
  (weight encodes jump size, doesn't rely on tooltips); code + comment
  match the pixels.
- P2 popover close × and queue-row × were FONT glyphs (a third family
  beside the SVGs) → utility-family stroked X (24-box, stroke 2, 14px).
- P2 volume waves 1.2 / mute cross 1.3 → 1.6 (the media-family stroke
  weight; 1.6 at 15px render = the Lucide-equivalent optical weight).
- P3 transport SVGs gained aria-hidden (a11y parity with mode/volume).
- DESIGN.md gained an **Iconography** subsection: the two-family system
  (media = 16-box fills + 1.6 strokes; utility = Lucide 24-box stroke 2)
  and the no-typographic-glyph rule.
Gates: check 0/0, vitest 53, build OK, detector exit 0, cropped screenshot
verified (incl. a live red-gradient-album playbar state).

**Grid critique pass (impeccable, 2026-08-29, commit 18d141d)**: album
grid + expansion panel critiqued as ONE surface (the expanded panel is a
grid row, not an overlay). First run 32/40 (Good); all five findings
landed same day:
- P2 selection without consequence (user-shaped as "b" + Enter): the
  row WAS a button that selected and then did nothing. Now: single
  click selects (wash + tooltip "Click again to play (or press
  Enter)"), a SECOND click on the selected row plays it, and Enter
  plays the selection from anywhere. The two paths can't double-fire
  (focused row's native Enter = click = second-click path; the window
  handler ignores interactive targets) and both stand down for inputs,
  the tag editor, and open context menus; a selected MISSING track
  locates on Enter instead. Selection clears when playback starts.
  A visible ▶-on-selected was rejected: button-in-button is invalid
  HTML and the per-row layout pop was ugly.
- P2 press convention skipped by the app's largest pressable families:
  tiles + track rows now `cursor: pointer` and take the documented
  `--active` `:active` wash (tile = a `::after` overlay, NEVER a
  filter/transform on the cover — WebKit layer churn = blank-cover
  flash).
- P2 inert grid empty state → clear-search action, same voice as the
  sidebar's `.empty`.
- P3 `.edit-album` 26 → 28px (PRODUCT.md hit-target bar); play-all
  gained `title="Play"`.
- P3 a11y: alert-triangle SVGs + the ▶/❚❚ status glyphs are
  `aria-hidden` with an sr-only "Now playing" sibling (new `.sr-only`
  utility in app.css).
Detector: the panel's `transition: height` is the documented WebKit
dual-clock fix — registered as a file-scoped exception
(layout-transition=* for ExpandedPanel.svelte); detector back to
exit 0. Snapshot: .impeccable/critique/2026-08-29T19-41-14Z__src-components-albumgrid-svelte.md.
Gates: check 0/0, vitest 53, build OK.

**Expansion choreography + the 20px-line reframe (2026-08-29)**:
the grid↔panel expansion surface, redesigned from user critique. Files:
AlbumGrid.svelte, ExpandedPanel.svelte.
- **Tile press ring** (user feedback "the wash should use the selected
  border"): the `::after` overlay wash is gone; `:active` lights the
  cover's EXISTING outline to the accent (`outline-color: var(--accent)`) —
  the outline is the album's state language (dim = hover, accent =
  playing / expanded / pressed). `.tile` gained 8px bottom padding
  (caption breathing).
- **Expansion state machine** (panel): the ad-hoc rAF chain became an
  explicit machine — `displayId` (lagging content; deliberately NOT
  tracking the `album` prop), `generation` counter for interruption,
  `move()` pins `auto`→px before every transition (a transition cannot
  interpolate from `auto` — an unpinned shrink jumps to 0 instead of
  animating), `settleToAuto()` drops the inline height in one no-jank
  reflow, `grow()` measures the natural height. Constants: grow 360 /
  collapse 280ms, same-row swap = 160ms content fade, no height
  motion. `cubic-bezier(0.22,1,0.36,1)` throughout.
- **Host-relocation architecture** (grid): the grid owns WHERE the panel
  row lives (`songHostId` / `albumHostId`), the panel owns what it shows
  (`album` prop = host row, `targetId` prop = the store's expanded album
  WHEN IT IS THIS ALBUM, else null = this instance closes). Same-row
  switch = host flips now, in-place content swap + fade (instance
  persists — the row key's two-value token does NOT flip). Cross-row
  switch = collapse-then-expand (see decisions log 2026-08-30): host
  lags the store through the outgoing panel's plain collapse, and its
  `onClosed` consumes the pending open — token flip (fresh mount at the
  destination), host flip, `await tick()` (row list flushed before the
  glide measures), glide + grow. A safety effect re-anchors a stale host
  (its row vanished from a search/artist change mid-collapse) to the
  store's target and clears the pending open (the unmount can't fire
  onClosed) — without it every later switch deadlocks.
- **reveal() auto-scroll: removed** — no app-initiated viewport movement
  in the expansion machinery; replaced by the reframe below.
- **The 20px-line reframe** (user-designed invariant — see decisions
  log): every open / collapse / switch ends with the focused row at the
  20px line, or the final layout's bottom edge (the browser's scrollTop
  clamp does the bottom limit for free — e.g. the last row of a
  200-track album). The travel is an ANCHOR, not a predicted tween: each
  frame it nudges scrollTop by however far the row's LIVE viewport
  position is from the desired one (ease-out cubic glide to the line,
  240–380ms — viewport-bounded, since only on-screen rows are clickable —
  then a hold). The live read makes it immune to mid-flight layout
  change: the panel grows BELOW the pinned row while the glide runs, so
  the row's motion is one continuous glide, then still. The first
  iteration aimed a predicted tween at the final layout and produced a
  visible stall→snap (the row physically can't reach the final position
  while the old slot is still open) — that's why the anchor exists. The
  anchor contract: it only ever tracks rows whose layout ABOVE them is
  stable (panels grow and close below their row); anything that animates
  layout above the row fights its one-paint-late scrollTop on this
  WebKitGTK (the reason cross-row switches collapse-then-expand — see
  decisions log 2026-08-30). Cancellation: wheel / touch / keys take the
  view over (height animations finish on their own); reduced motion
  disables the reframe entirely (zero viewport movement — the stricter
  a11y reading); ≤4px deltas don't travel.
  Verified via instrumented traces (Vite middleware sink →
  /tmp/songstress-diag.log, all instrumentation stripped after use).
Gates: check 0/0, vitest 53, build OK.

**Remaining: RPM rebuild (user installs) — deliberate pause 2026-08-29,
other work takes the queue.**

---

## Motion audit — token consolidation + polish ✅ (2026-08-30, improve-animations skill)

Full audit of the motion surface (8 categories). Verdict: **the core
choreography is right** — no HIGH/MEDIUM feel-breaking findings; the
360/280ms panel moves, the 240–380ms anchor glide, the gap-synced slot
margins, the 320ms drawer stack and the instant high-frequency hovers
are all correct (and user-verified — untouched). What landed:
- **Motion tokens** (app.css `:root`): `--ease-out:
  cubic-bezier(0.22, 1, 0.36, 1)` (the user-verified panel curve — now
  referenced, not re-typed) and `--ease-drawer: cubic-bezier(0.32, 0.72,
  0, 1)`. All 7 hand-typed curve sites replaced, incl. ExpandedPanel's
  JS `CURVE` const (var() resolves fine in inline styles — verified by
  frame trace). Duration spellings normalized to ms.
- **PlayBar gradient fade** 400ms `ease` → 280ms var(--ease-out) (UI
  budget is <300ms).
- **EmptyState progress fill** animates `width` (layout) → full-width
  element + `transform: scaleX(pct/100)` from the left edge, 250ms
  var(--ease-out) — compositor-only per progress tick.
- **Dead `translate` transition on .tile .cover** removed (the hover lift
  it served went away in Step 0c).
- **Finding #4 (pop-in keyframes on the EQ/queue popovers) is VOID —
  not implemented**: both popovers `{#if}`-unmount on close, so a
  re-open is a fresh element for keyframes AND transitions alike; the
  keyframes-restart concern only applies to re-triggered mounted
  elements. The 140ms pop-in (scale 0.98 from the shared edge with the
  trigger) is correct as-is.
- Parked with the modal work: TagEditor entrance (see *Up next*).

## Open threads / known issues

- **Stale dev-unit drop-in — REMOVED (2026-08-31)**: deleted
  `~/.config/systemd/user/songstress-dev.service.d/inspector.conf`
  (`Environment=WEBKIT_INSPECTOR_SERVER=127.0.0.1:9301`) + `daemon-reload` +
  unit restart. The env var made WebKitGTK listen on loopback:9301, but the
  remote-inspector handshake was never identified (see *Environment quick
  facts* — `tools/inspect.mjs` kept for the record), so nothing consumed it.
  Verified after restart: `DropInPaths` = distro `10-timeout-abort.conf`
  only, `Environment=` empty, port 9301 no longer listening, app boots clean
  (Global Menu registered, watcher armed, 248 tiles rendered). Recreate with
  `systemctl --user edit songstress-dev` if the inspector protocol is ever
  pinned down. The 2026-08-29 note below ("drop-in REMOVED") had covered the
  DMABUF/compositing experiments, not this one; the agent visual loop goes
  through the devtools bridge (`src/lib/devtools.ts` + `tools/devctl.mjs`),
  which needs no drop-in at all.

- **WebKitGTK viewport glitch, VARIANT 2 (2026-08-29)**: webview surface
  clipped to the top ~40px band, rest of the window black; layout was FULL
  size (frontend viewport matched the window — the 60s guard correctly saw
  no mismatch and could never fire). Triggered after the user dragged the
  window into a side-by-side split (journal: size churn 2740×1780 →
  2560×1600 → 2400×2132 right after launch). EXHAUSTED without effect:
  KWin reconfigure, minimize/restore, maximize/unmaximize, FOUR real 1px
  resize nudges (incl. during the churn), blur-effect unload/reload,
  full page reload (Vite entry touch). KWin scripting is dead on 6.7
  (loadScript/loadDeclarativeScript return -1 for any path or name —
  installed-script dir too; the /WindowsRunner Match/Run interface is
  `org.kde.krunner1`, NOT `org.kde.KWin.WindowsRunner`).

  Environment experiments (all via a songstress-dev.service.d drop-in,
  since 2026-08-29; drop-in REMOVED same evening — dev unit runs stock
  env):
  - `WEBKIT_DISABLE_DMABUF_RENDERER=1`: clip gone, UI perfect — but the
    user confirmed the glass layers break (search bar / mode buttons /
    panels read as inconsistent backgrounds; the wallpaper frost is
    gone). Rejected for daily use.
  - `WEBKIT_DISABLE_COMPOSITING_MODE=1`: transparency also degraded
    (same family of "sections with different backgrounds"). Rejected.
  - KILLING `WebKitWebProcess` as a runtime recovery: FATAL —
    WebKitGTK 2.52 does not respawn it in the Tauri setup; the window
    goes solid black and only a unit restart recovers. NEVER do this.
  - No upstream fix available: already on webkit2gtk4.1 2.52.5-1.fc44
    (latest Fedora build).

  Intermittency (stock env): launches at 15:17 and 15:28 were
  band-clipped (persisted 8+ min, unresponsive to everything above);
  15:33 launched clean; a follow-up restart showed a TRANSIENT
  "blurred wallpaper only" state (no content yet) that self-recovered
  within seconds — pre-first-paint, not the clip. Practical recovery:
  give a fresh launch ~10s; if the band persists, `systemctl --user
  restart songstress-dev` and retry — the race does not always hit.
  Trigger correlation: the window had just been dragged into a
  side-by-side split (size churn in journal). Open: a durable fix
  needs the WebKitGTK/Fedora side (DMABUF buffer pin on resize).
- **WebKitGTK viewport glitch** (FIXED 2026-08-22, kept for reference):
  on some launches the webview rendered at a stale smaller size inside the
  correctly-sized GTK window. `WEBKIT_DISABLE_DMABUF_RENDERER=1` fixed the
  glitch but killed backdrop transparency — rejected. Fix: Rust setup
  hook nudges the window 1px down+back at ~0.6s/~2.5s after launch,
  forcing a reconfigure, PLUS a self-healing guard
  (`src/lib/viewportGuard.ts` + `fix_viewport` command): for the first 60s
  after load the frontend compares its viewport against the real window
  size (8px tolerance) and re-nudges on mismatch. Verified across
  consecutive relaunches incl. one that reglitched post-nudge.
- **Expansion animation** (rewritten 2026-08-22): WebKitGTK animates the
  grid track and the content clip on DIFFERENT clocks — first expand
  snapped, collapse left an empty shell band. Now `.inner` animates
  explicit px height (rest states "0px"/"auto", settle timer 400ms) so
  clip edge, panel edge and shadow move in lockstep; verified frame-by-
  frame in slow motion.
- **Switch lag** (addressed 2026-08-22): album_colors was a SYNC tauri
  command (full-res JPEG decode on the main thread per switch) — now
  async + spawn_blocking + path-keyed COLOR_CACHE. Remaining suspects if
  lag persists: DX blur re-frost per frame, webview raster cost.
- **Expanded panel shadow** on `.expander`, not `.panel` (2026-08-22): the
  inner wrapper must stay overflow:hidden for the height animation and
  clipped the panel's shadow into sharp corners; expander is never
  clipped and tracks the animated box.
- **Grid scrollbar hidden permanently** (2026-08-22, user request):
  `scrollbar-width: none` + `::-webkit-scrollbar { display: none }` on the
  `.content` scroller; wheel/trackpad scrolling unaffected.
- **User's ~/.config/mpv/mpv.conf is broken** (d3d11 / C:\ fonts) —
  unrelated to the app (mpv is spawned `--no-config`) but worth telling
  them once more.
- **KWin DX multi-line WindowClasses** silently fails to match — keep one
  entry (see README/AGENTS).

## Settings menu tree — sidebar-shaped redesign ✅ (2026-08-30, impeccable critique, user options C/A/A)

Critique of the settings menu tree scored 25/40 (snapshot
`.impeccable/critique/2026-08-30T21-44-56Z__src-components-sidebar-svelte.md`;
degraded single-context run — no sub-agent tool in this harness). Core
finding: the tree was the Rust global menu bar rendered verbatim
(Playback/Library/View/Help + grafted Appearance) — View was a one-item
pane duplicating Appearance's theme button, Help was a one-item pane of a
NO-OP ("About Songstress" did nothing on either surface), and the root
rows had no drill-down affordance. The sidebar now has its own tree
(Global Menu / menu.rs untouched — it stays a menu bar):
- **Root:** Appearance / Playback / Library with trailing › chevrons + a
  dim **About Songstress footer row** at the bottom (a footer, not a nav
  pane) opening the new in-glass **About dialog** (About.svelte, App-level
  like TagEditor: name, version via `getVersion()` so it can't drift from
  the RPM, one-line description; Esc/backdrop/✕ close; focus lands inside
  on open; the sidebar's Escape handler yields to it via `ui.aboutOpen`).
- **Playback pane:** drops its 6 transport rows (the PlayBar owns them) —
  modes + EQ only. **Library pane:** "Save imported music" hides when
  nothing is staged instead of standing as a dead row.
- **ONE row language** (option A): menu rows join the artist-row rhythm
  (`--sidebar-row-size` height, 13px, same label column); the left check
  column is gone — ✓ and › are trailing glyphs in the count column, labels
  align with the pane title. Arrow keys walk the focused layer's buttons
  (menu convention; Tab unchanged; ranges keep native arrow behavior).
- **Glass controls** (option A): the OS-default checkbox (the only
  non-glass control in the app) is a hidden native input driving a 16px
  rounded box — accent fill + check in luminance-aware `--accent-text`
  (a white check on the user's white accent would vanish), 160ms;
  sliders are appearance-none with an inline accent fill gradient sized
  to the value + accent-dot thumb.
- DESIGN.md updated (row language, tree shape, slider/checkbox
  components); the detector's one advisory (checkbox 5px radius off the
  rounded scale) is documented as deliberate there.

## Settings panes — critique pass C: layout + quieter ✅ (2026-08-31, impeccable critique 24/40)

Second critique of the sidebar Settings stack (`src/components/Sidebar.svelte`)
scored **24/40** (`.impeccable/critique/2026-08-31T18-30-27Z__src-components-sidebar-svelte.md`);
user chose the order **C (layout+quieter) → A (clarify) → B (audit)**, scope
"everything". This is pass C.

**Diagnosis:** the Appearance pane did not feel crammed because of density
(content stopped 231px above the pane floor — 40% of the column empty) but
because five heterogeneous controls ran on one uniform 16px cadence with no
group seams: the eye got one block. Cognitive load failed 4 of 8 checklist
items (chunking, grouping, ≤4 options at the accent decision, progressive
disclosure). Detector: 1 advisory (`design-system-radius` on the documented
5px checkbox radius) — false positive; browser overlay: 1 finding (`.app`
clips a positioned child — the intentional window clip).

**Landed:**
- **Uneven rhythm**: `.panebody` gap 30px between groups, `.group` gap 10px
  inside; group seams use DESIGN.md's **Label** tier (11.5/600/+0.09em
  uppercase), which the pane never used. Control labels moved to List tier at
  full `--text` (the old 12px/400 dim was a tier DESIGN.md does not declare).
- **Accent folded one level deeper**: the stack gained a 4th layer (`.layer
  sub`, z-index 3). Appearance now shows an `Accent · <name> · ›` summary row;
  the 12-dot grid lives in the sub-pane as a 6-column grid (11 presets + the
  custom picker = two even rows, was a ragged 7 + 5). `ui.menuSub` is the
  state; Escape pops sub → pane → root → home; `openDetail` clears it.
- **Full push, then teleport**: a pushed sub-pane takes the detail layer a
  FULL width left (`translateX(-100%)`, matching root — the user rejected
  parallax peeking on 2026-08-27 and a flat glass column has no room for it).
  On a full pop the receded detail would sweep the column on its way back to
  its +100% entry slot, so `.layer.detail.no-anim` reuses the root's one-frame
  transition suppressor. **Frame-trace verified** (`devctl sample` on
  `.layer.detail` x during the close): 22 frames at -235, then a single frame
  at +235 — no intermediate values, so no sweep.
- **Per-pane footers** (user-approved content): Appearance → `Reset appearance`
  (tile 180 / rows 36 / theme system / stock accent / gradient off; verified
  live: `--tile-size` 220px → 180px on click), Playback → `Reset equalizer`
  (Flat preset + preamp 0), Library → status line `248 albums · 4434 tracks`
  (+ ` · scanned HH:MM` once a scan has run — `ui.lastScan`, persisted through
  the settings table from `scan-finished`; the clause is omitted rather than
  lying "never scanned" on a library built before the readout existed).
  Footer ACTIONS render at 0.82 of `--text` ≈ 4.6:1 over the worst-case bright
  wallpaper under the 0.7 chrome tier; `--text-dim` stays caption-only there.
- Splitting the shared `.appearance label` rule removed the
  `.toggle { flex-direction: row !important }` hack.

**Verification**: `npm run check` 0 errors · `npm test` 57 green ·
`npm run build` ok · `detect.mjs --scope layout` clean (exit 0) · full
detector unchanged (1 documented advisory) · screenshots of Appearance, the
Accent sub-pane, Playback and Library footers through the devtools bridge.

**Open (passes A and B, in that order):**
- **A — clarify**: 3-segment `Light | Dark | System` (theme is currently a
  ONE-WAY DOOR: `ui.theme` ships as `system`, `cycleTheme()` destroys it, the
  Rust model carries `theme_dark: bool`); slider value readouts in the count
  column (`tabular-nums`, `aria-valuetext`); copy: "Add music folder…" vs
  "Music folders…", "Full Rescan (rebuild)".
- **B — audit**: focus stays on the pushed-out row (measured:
  `document.activeElement` inside an `aria-hidden` layer, 5 focusables → Tab
  walks 4 invisible rows before the visible pane) — hand focus to the incoming
  layer and set `inert` on inactive layers; `.swatch:focus-visible` (none
  exists today); drill glyph at `opacity:.6` on `--text-dim` ≈2.3:1 worst case.
- **Polish**: swatches should render the theme-clamped derived accent (Black in
  dark theme becomes light grey — `accent.ts:96` CLAMP), the custom swatch
  should show the picked color instead of a permanent rainbow.

## Settings panes — critique pass A: clarify ✅ (2026-08-31)

The P1 behavioral findings, all in the Appearance pane + the Library menu copy.

- **Theme: 3-segment `Light | Dark | System`** replaces the cycling
  `Theme: Dark` button. `ui.theme` SHIPS as `"system"` and `cycleTheme()`
  resolves it to a fixed value, so the default was a one-way door with no path
  back from any surface; the segmented control writes the mode directly
  (`ui.theme = t.id`, the $effect persists + repaints as usual). DESIGN.md
  gained the component spec (32px, Glass Line trough, `--hover` segments,
  active = accent wash + 600, hover stays NEUTRAL — the old button spent the
  accent wash on hover, breaking the row language and the One Accent Rule).
  Verified live: `aria-checked` reports `System` (the true stored mode — the
  old button displayed "Dark", i.e. the RESOLVED theme, hiding the state).
  **Global Menu left untouched on purpose** — see *Explicitly deferred* for the
  `theme_dark: bool` → `theme: string` migration it needs.
- **Slider value readouts** (`180px` / `36px`) in the label row's right column,
  Micro tier + `tabular-nums` + `aria-valuetext`, live while dragging. The pane
  whose entire job is choosing a number was choosing it blind, and the Tabular
  Rule had zero instances in it because it showed no digits.
- **Copy (menu.rs labels, ids untouched):** "Add music files…" → **Import music
  files…**, "Add music folder…" → **Import music folder…** (they stage an
  import — Step 2a — and sat three rows above "Music folders…", which edits
  library roots); "Full Rescan (rebuild)" → **Full rescan (all files)**.
  EmptyState's "Add music folder…" deliberately KEEPS its name: it opens the
  roots flow, where that is the correct label.

**Verification**: `npm run check` 0 errors · `npm test` 57 green ·
`cargo test --lib` 65 green (no label assertions touched) · `npm run build` ok ·
`detect.mjs --scope layout` clean · screenshot `/tmp/a1-appearance.png` +
live DOM read of the new labels. Side-effect note: the pass-C "Reset
appearance" click during verification cleared the owner's accent + playbar
gradient; both were restored (accent = White, gradient on) through the UI.

## Settings panes — critique pass B (audit) + polish ✅ (2026-08-31)

**B — the keyboard/ARIA findings:**
- **Focus handoff.** Pushing a layer used to leave focus on the row that
  triggered it — inside a layer that had just become `aria-hidden`, so the ring
  vanished and Tab walked the covered rows first. `focusFirst()` now moves focus
  to the incoming layer's back affordance (pane/sub-pane → `.backchev`, root →
  its first row, close → the gear). Verified live: after popping the accent
  sub-pane, `document.activeElement` is the detail `.backchev`.
- **`inert` on inactive layers** (home when the stack is open, root/detail/sub
  whenever they aren't the active layer). The 5 hidden controls left the tab
  order — the "4 phantom stops" finding is structurally gone, not commented
  around. Verified in the DOM: with the sub-pane open,
  `home/root/detail = inert`, `sub = not`.
- **Radiogroup keys for the segmented control.** `role="radio"` implies arrow
  keys: `segKeydown` (Left/Right/Up/Down/Home/End) + roving `tabindex` (group is
  ONE tab stop, the checked segment carries `tabindex=0`; the group itself is
  `tabindex=-1`, which also silences svelte's `a11y_interactive_supports_focus`).
  Verified: ArrowRight moves aria-checked, focus AND tabindex together.
  Gotcha found: `event.currentTarget` is null inside a `requestAnimationFrame`
  callback, so the first version moved aria-checked while leaving the focus ring
  on the previous segment — grab the node list synchronously.
- **Swatch focus.** `.swatch:focus-visible` + `.swatch.custom:focus-within`
  (the `<input type=color>` is `opacity: 0` inside its label, so the label's
  focus-within is its only ring) → accent outline.
- **Drill glyph contrast.** dropped the `opacity: .6` on top of `--text-dim`
  (≈2.3:1 worst case under the 0.7 chrome tier; a meaningful graphic needs
  3:1) — `--text-dim` alone is ≈3.5:1 — and it now resolves to `--text` on
  `:focus-visible`, not just on hover.
- Deliberate non-fix: `--text-dim` Micro-tier captions (`.fstat`, `.curname`,
  artist counts, track durations) sit ≈3.5:1 over the worst-case wallpaper
  app-WIDE. Fixing it per-pane would fracture the caption tier; it needs one
  global decision (raise the tier's alpha, or darken the chrome tier), which
  touches every surface — logged, not acted on.

**Polish (P3s):**
- **Clamp ring on accent presets.** `accentVariants` clamps accent lightness per
  theme (`accent.ts` CLAMP), so picking **Black** renders `#b3b3b3` in the dark
  theme while the swatch was a black dot. Swatches keep the raw hex (Black and
  White must stay tellable apart) and gain a 2px ring in the color they actually
  produce — only where the lightness shift exceeds 0.12 — plus a tooltip
  ("Black — renders #b3b3b3 in dark theme"). Verified: rings on Black/White
  only.
- **Custom swatch** shows the picked color with a 9px conic corner instead of a
  permanent rainbow.
- The Accent summary chip shows the DERIVED color, not the stored hex.
- Segment radius 6px → 7px (icon tier): a 2px inset inside an 8px trough
  computes to 6, which is off the radius ladder and indistinguishable at this
  size, so no documented exception was needed. Detector back to its single
  known false positive (the documented 5px checkbox radius).

**Verification**: `npm run check` 0/0 · `npm test` 57 · `cargo test --lib` 65 ·
`npm run build` ok · detector: 1 advisory (documented) · live DOM probes above ·
`/tmp/p2-accent.png` shows the Black preset's ring.

**Still open**: critique re-run for the trend line (expect the P1s to clear);
`--text-dim` caption-tier decision; Global Menu theme-state migration (deferred).

## Post-pass regression: the stack painted "twice" (2026-08-31, found by the user)

Cold restart after pass B/polish, and the user reported the sidebar menu broken.
Two INDEPENDENT causes, both now fixed and verified live:

1. **Cold-start dropped CSS** (not my code): the webview requested
   `App/AlbumGrid/PlayBar.svelte?svelte&type=style&lang.css` before the svelte
   plugin had compiled their parents, the plugin had no metadata for the id, Vite
   fell through to the static-file handler — which ignores the query — so those
   "style modules" were served as the component's RAW SOURCE. No style tag, no
   layout: 1280-wide grid rows, `.playbar` block instead of grid, and it did NOT
   self-heal on reload (only on `touch`). Fixed in vite.config.js with
   `server.warmup.clientFiles = ["./src/main.ts"]`; verified by curling all four
   style modules immediately after a cold boot — all return compiled CSS now.
2. **Pass B's focus handoff scrolled the stack sideways** (my bug): `.stack` was
   `overflow: hidden`, which is STILL a scroll port. `focusFirst()` focused the
   incoming layer's back chevron while that layer sat at `translateX(100%)`, so
   the browser scrolled the clipper to `scrollLeft: 202` to reveal it — and
   nothing ever scrolled it back. Every layer shifted left at once (home -437,
   root.active -202, detail +33), which is why "Settings" printed over
   "Appearance" and the sub layer's swatches showed through the root pane.
   Computed styles looked perfect while `getBoundingClientRect()` lied — the
   tell is to read `scrollLeft`, not the transforms. Fixed twice over: `.stack`
   is `overflow: clip` (`hidden` kept as the fallback line) and `focusFirst` /
   the segmented-control focus pass `{ preventScroll: true }`.

**Verified after the fix** (DOM probes, all with `scrollLeft: 0`):
open → home -235 / root.active 0 / detail+sub +235, focus `mrow`;
Appearance → root.receded -235 / detail.active 0, focus "Back to settings";
Accent → detail.receded / sub.active 0, focus "Back to appearance";
pop → detail.active, focus "Back to settings"; pop → root.active, focus `mrow`.
Screenshots `/tmp/x-appr-crop.png` (segmented control on System, 180px/36px
readouts, Accent · White drill) and `/tmp/x-sub-crop.png` (6-across presets,
clamp rings on Black/White) show the panes as designed.
Also caught while verifying: the custom swatch inherited the *preset's* derived
color when the accent was a preset — it now stays the rainbow unless the color
is genuinely custom (`hasCustom ? chipColor : undefined`).

**Screenshot note for future sessions**: this window is 1280x800 CSS and
spectacle captures 2048x1280 — scale is **1.6**, not the dpr the page reports
(2). Dividing device px by 2 made the correctly-rendered sidebar look 40px too
narrow for half of this investigation.

## User-feedback round (2026-08-31, after the pass B/polish report)

Three requests, all shipped:

1. **`System | Light | Dark`.** User's reasoning: reading left to right, the
   leftmost segment is the default option, and System IS the default. Reversed
   the previous order — and it matches `resetAppearance()`'s target and the
   shipped default, so the control now says "default" where the eye lands
   first. `THEMES` order also drives the radiogroup's arrow/Home/End order, so
   Home=System. Verified: `["System*","Light","Dark"]`.
2. **The accent sub-pane looked broken — three separate reasons, all real.**
   - The selected dot drew TWO near-identical concentric rings (white 2px
     outline + grey clamp ring around White). Now `swatchRing()` suppresses the
     clamp ring while that preset is selected; the tooltip still carries the
     promise. Verified computed: Black `box-shadow: 0 0 0 2px #b3b3b3`; White
     `box-shadow: none`, `outline 2px @2px` — one ring each.
   - The custom swatch rendered SMALLER and low in its cell: WebKit draws its
     own ~14px color-well inside `<input type=color>`, and `inset: -4px` on the
     label couldn't contain it. The input is now `appearance: none; inset: 0;
     width/height: 100%` and invisible, so only the 20px conic disc paints.
   - Row rhythm: the outline reaches 4px past a dot and the grid had no row gap,
     so an outlined dot touched the dot above. First fix was `gap: 14px 20px` —
     which the user immediately caught, because a minimum COLUMN gap on fixed
     20px tracks spread with `space-between` overflows the grid's own box
     (6×20 + 5×20 = 220 in a 207px pane) and shoved the last dot into the
     sidebar border (measured: dot right edge 234, content edge 235, border 236
     — with the 4px outline beyond that). Final rule is `gap: 14px 0`: row gap
     only, horizontal spacing left fluid, so both rows sit flush on the pane's
     14px padding (measured 14→221, outline clears the border by 10px) and the
     ragged 11-item row stays column-aligned.
3. **Custom color moved out of PRESETS** into its own `CUSTOM` group, in the
   pane's existing readout-row language:
   `Pick any color · not set / #rrggbb · [20px picker]`. An editor, not a
   twelfth named choice — and as a dot in the grid it had a worse bug than the
   semantics: `hasCustom` is false whenever the accent equals a preset, so it
   silently showed the PRESET's derived color. The whole row is the `<label>`,
   so the text opens the chooser too. Verified end-to-end: pick `#12ab56` → row
   reads `#12ab56`, swatch shows the DERIVED accent (mint in dark theme) with a
   conic corner, the Appearance drill reads `Accent #12ab56`, the back chevron's
   focus ring turns green (live accent), the presets deselect; restore to White
   → drill reads `Accent White`. (`/tmp/n3-2x.png`, `/tmp/n2-sub-2x.png`)

Also while in there: `resetAppearance()` now resets `playbarGradient` — it is
the pane's other visible control and the button says "Reset appearance".

## Playback pane: flat cycling rows → enable + reveal (2026-08-31)

User brief: "the three of them get a checkbox; Repeat and Random enable a
horizontal switch like the theme one; Eq enables both preset and custom fields;
the footer should reset the 3 options, not just the eq."

**Before:** five look-alike rows from the flattened menu model — `Shuffle: Off`,
`Repeat: Off`, `Equalizer: On` ✓, `EQ Preset: Rock` (dead until the equalizer
was on), `Customize Equalizer…` — no seams, values welded into labels, cycling
invisible, one checkmark meaning three different things.

**After** (bespoke, store-driven — like Appearance):
```
☑ Repeat        →  [ Album | Track ]
☑ Shuffle       →  [ Album | Artist | All ]
☑ Equalizer     →  Preset        Rock ›        (sub layer, 10 radio rows)
                   CUSTOM
                   Preamp   0 dB   [slider]
                   31       +5     [slider]    × 10 bands, dbl-click zeroes
                   ── Reset playback
```
- "Off" is owned by the checkbox, so it is NOT a segment: 2-up repeat / 3-up
  shuffle, and the two controls can never disagree. `.seg` now takes
  `--seg-n`; `segKeys(ids, get, set)` is the shared radiogroup key handler
  (Theme uses the same one now).
- `setShuffleStage` / `setRepeatStage` are the single door (state + localStorage
  + SQLite write-through + `playback_set_*`); `cycleShuffle` / `cycleRepeat` are
  now thin wrappers over them, so the **Global Menu keeps cycling and both
  surfaces push the same state**. `setShuffleOn/off` remember the last non-off
  stage in module memory (not persisted, not pushed — a UI memory), so
  un-checking and re-checking restores Track rather than silently picking
  Album. Verified: pick Track → uncheck → recheck → `Track*`.
- Reveals mount with the checkbox, indented 12px (no card — double-translucency
  ban), fade+rise 200ms; the EQ reveal holds the preset drill + preamp + 10
  bands, which is why `.panebody` was already scrollable.
- The 10 presets live in the sub layer (`ui.menuSub = "eq-preset"`, the accent
  picker's mechanism, now branched): `.irow` + `.ok` in the count column,
  `aria-current`, and a `Custom curve — adjust the sliders in Playback` status
  line when `preset === null` (editing a band clears the name, and the pane has
  to be able to say so). Sub `navtitle` and the back button's label are derived
  now (`Back to ${detailTitle}`) instead of hardcoded "appearance".
- Footer renamed and widened in scope: `Reset playback` clears repeat, shuffle,
  eq enable, preset (Flat) and preamp.
- `Customize Equalizer…` left the pane: the sliders are right here, and the
  playbar popover is still one tap away. Transport rows stay hidden (PlayBar
  owns them).

**Verified live** (`/tmp/pb5-2x.png`, `/tmp/pb6-2x.png`): 3 toggles read the real
state (Repeat off, Shuffle off, Equalizer on); enabling shows 1 then 3 reveals;
segments report `["Album*","Track"]` / `["Album*","Artist","All"]` with
`grid-template-columns: 88px 88px` on the 2-up; band values match the Rock
preset (+5/+4…); editing band 0 flips the sub layer to the Custom status line;
tapping Rock re-applies it and restores +5; every state restored afterwards
(Repeat off, Shuffle off, Equalizer on, Rock). `npm run check` 0/0 · 57 tests ·
build ok · detector unchanged.

**Open:** the equalizer's vertical mini-EQ in the playbar popover and these
horizontal rows are now two surfaces for one thing — fine for now, but the
popover should get the same `dB` readout honesty next time it's touched.

## Equalizer popover trimmed + one Toggle component (2026-08-31)

Chose option **b** (popover keeps the curve, pane owns enable + preset), with
the user's addition: the playbar's EQ button must still reach everything.

- **`components/Toggle.svelte`** now owns the app's checkbox (glass box,
  luminance-aware check, accent ring on focus) — 4 call sites in the sidebar
  (Playbar gradient, Repeat, Shuffle, Equalizer) and the popover. The popover's
  bare native checkbox was the last one outside the design system, and copying
  its CSS into a second component would have been the worse answer.
- **The native `<select>` is gone**, replaced by the preset NAME in Micro tier
  (`Rock` / `Custom`). Its dropdown was the toolkit's own chrome — AGENTS.md
  already bans GTK-styled pickers for looking foreign; a WebKit select menu is
  the same category of problem. Removing it also deleted the popover's second,
  weaker copy of the preset picker.
- **`Playback settings ›`** footer row in the popover (same 0.82-of-`--text`
  action tier as the panes' reset rows, neutral hover wash, accent ring) closes
  the popover, opens the stack on the Playback pane and moves focus to the
  incoming layer's back chevron with `preventScroll` — the popover is the fast
  path for nudging bands; the pane is the way to the rest, and the button the
  user just pressed is where the route starts.
- Verified live (`/tmp/eq1-crop.png`, `/tmp/eq2-crop.png`): popover reports
  `{toggle: true, checked: true, preset: "Rock", select: false, sliders: 11,
  foot: "Playback settings"}`; clicking the footer yields
  `{popoverClosed: true, stackOpen: true, detail: true, focus:
  "backchev|Back to settings", sliders: 11}` and the four Toggle instances read
  the real state. Rock's curve matches band-for-band (+5/+4/+2.5/0/-2/-1/+1.5/
  +3.5/+5/+5.5). svelte-check flags no unused selectors after the dead
  `.eq-pop .toggle` / `.eq-pop select` rules were removed.

## Dismissal standardised on the traffic-light family (2026-08-31)

User's proposal, adopted with two amendments: every floating surface dismisses
with a red traffic-light dot at **top-left**, and the EQ popover's rows were
reordered so the header is chrome, the middle is the curve, the base is
ownership (`[✓] Equalizer …… Rock`).

- **`components/SurfaceClose.svelte`** — 14px dot (was 13; the titlebar dots grew
  to 14 and the gap to 10, making the cluster 24px on center — bigger dots, bigger
  hit area, and the family keeps one size), `--tb-x` (the user's KWin
  close colour, now published on `<html>` by `decoVars()` in the decoration
  store, shared with the titlebar instead of duplicated), 26px transparent hit
  box, glyph on hover AND keyboard focus, press = brightness 0.82, **red only**.
  Applied to: EQ popover, queue popover, About modal, Music folders modal (whose
  dismissal was a font-glyph `✕` — a third typographic family inside glass).
  **The settings stack keeps its boxed ✕**: its header row is the same row as the
  real window dots, and two red dots 200px apart with different scopes is worse
  than the shape difference removed. `.q-x` (remove row) also stays an ✕.
- **Focus discipline:** dismissal returns focus to the opener. The opener is
  recorded at OPEN (`openAbout(from)`, `ui.aboutOpener`; the popovers hold
  `eqBtn`/`qBtn`) — reading `activeElement` at unmount gives the dialog's own
  close button, because child effects run before parent effects and the autofocus
  has already moved focus inside.
- **EQ popover restructure:** header = dot + `Playback settings ›` route (0.82
  action tier); `.bands` = the curve, `.asleep` when off (`opacity: .45` on
  readouts + slots, band labels and the checkbox row keep full contrast —
  dimming the gate hides the only way out, and the curve stays editable while
  off: shape it first, then switch on); footer `.own` = Toggle + preset name over
  a `--border` seam, `cursor: default` because it is inert now.
- Queue popover: dot first, `Clear` pushed to the far edge (`margin-left: auto`)
  so the destructive action sits opposite dismissal. Titlebar dots got a 21px
  hit area via `::before { inset: -4px }`, capped by the 8px gap.
- Verified live: `--tb-x` on `:root` = `rgba(253, 82, 86, 1)` (the real KWin
  colour, not the fallback) and the dot computes to `rgb(253, 82, 86)`; popover
  probe `{header: [sc, route], footer: [Equalizer, Rock], asleep: false}`,
  off-state `{asleep: true, dbOpacity: .45, hzOpacity: 1, toggleOpacity: 1}`;
  close returns focus to `aria-label="Equalizer"`; About autofocuses the dot,
  `position: absolute` at 15px inset, and closing returns focus to the
  `About Songstress` row (before the open-time capture it landed on `<body>`).
  Screenshots: `/tmp/tl1-crop.png` (curve on), `/tmp/tl2-crop.png` (asleep),
  `/tmp/tl3-crop.png` (queue), `/tmp/tl4-crop.png` (About).
- **Two Svelte 5 gotchas for AGENTS.md:** a class forwarded into a child
  component does NOT receive the parent's scope id (`:global(.ab-close)` is
  required), and the parent rule also needs one extra class of specificity
  (`:global(.ab .ab-close)`) because the child's `.sc { position: relative }` is
  an equal-specificity rule in a later stylesheet. Music folders modal verified
  by structure only (it opens from the parked Global Menu).

## Modal scrim now respects the window radius (2026-08-31)

User catch: the About overlay squared off the window's rounded corners. Cause —
`.app` draws the 14px window radius with `overflow: hidden`, but `overflow` does
not clip a `position: fixed` descendant (its containing block is the viewport),
so `.ab-backdrop` / `.mf-backdrop` painted `rgba(0,0,0,.35)` into the transparent
corners.

- Added the missing ladder rung as a token (`--radius-window: 14px`), used it in
  `.app` (was a literal), and introduced **`.scrim`** in app.css carrying the
  whole overlay tier — fixed/inset-0, `z-index: 150`, the 0.35 wash, centering
  grid, and `border-radius: var(--radius-window)`. Both dialogs now use
  `class="…-backdrop scrim"`; their private copies of the same seven declarations
  are gone (z-index and wash strength were drifting candidates).
- Side effect worth knowing: `border-radius` affects hit-testing, so a click in
  the outer 14px corner triangle falls through the scrim to the grid instead of
  dismissing the dialog. Correct — that area is not visually part of the wash.
- Verified: computed `border-radius: 14px` on the scrim and on `.app`; corners in
  `/tmp/sc1-tl.png` / `/tmp/sc1-br.png` show the wash stopping on the curve.

## Scrim, take two: Tag editor was the third modal + containment (2026-08-31)

User: still square on my view. Two findings.

- **A modal was missed.** `TagEditor.svelte` carries a third private copy of the
  scrim (`.te-backdrop`: same fixed/inset-0/z-150/grid/0.35) and its dismissal was
  a **font-glyph `✕`** — the exact two things the previous two commits removed
  elsewhere. It was missed because the sweep grepped for the class names it knew
  (`ab-close`, `mf-close`, `close`) instead of for the pattern. Now
  `class="te-backdrop scrim"` + `SurfaceClose` first in `.te-head` (the header's
  `h2 { flex: 1 }` pushed the dot to the right until the markup was reordered —
  the dot goes LEFT, like every other surface).
- **Structural fix:** `.app { contain: paint }`. `overflow: hidden` does not clip
  fixed descendants; containment makes `.app` their containing block, so the
  window's rounded shape clips every overlay regardless of what it forgets. The
  radius on `.scrim` stays (explicit, survives someone dropping the containment).
- **Screenshot caveat that cost time:** the window capture's geometry is not
  stable — it was 2048×1280 (client × 1.6) earlier and 2128×1264 later, i.e. the
  decoration/shadow region is included sometimes, and the alpha channel makes the
  transparent margin read as white in viewers; a full-screen grab put *the
  terminal running this session* under the crop. So corner verification moved from
  pixels to the DOM: `elementFromPoint(1,1)` → `.head` (the click falls through the
  clipped corner triangle) and `elementFromPoint(40,40)` → `.ab-backdrop.scrim`
  (inside the arc). Computed: `.app` contain `paint`, all three scrims
  `border-radius: 14px`, Tag editor header `[BUTTON.sc, H2]`, dot
  `rgb(253, 82, 86)`, no `✕` glyph left in its text.
- If a user still sees a square corner in the wash, the candidate is the compositor
  rounding the window at a radius ≠ our 14px clip (then: read the radius from the
  decoration instead of hardcoding the rung).

## Library pane, rebuilt (2026-08-31)

The last pane still rendering the flattened menu model: six rows at one
undifferentiated tier, in an order that put the rarest action first, with the
one live fact (two albums sitting staged) buried fifth and unstated.

- Bespoke branch like Appearance and Playback, three groups: **Import** (files /
  folder, then a reveal that exists only while anything is staged — the amount as
  a state line, then `Save imported music` and `Discard imported music`),
  **Scan** (`Scan for changes`, `Re-read all files` + the live progress line),
  **Storage** (`Music folders…` with the folder count as readout). Footer keeps
  the counts and adds the last-scan time on a subordinate second line.
- **`.gstat`** (in-group status, caption tier, tabular nums): the live scan
  progress — `Scanning 1,810 of 4,434` — inside the group whose rows it explains,
  `role="status"`. The footer stopped carrying `scanning…`: the footer is what the
  library IS, the group is what is happening to it.
- **`Discard imported music`** is a new door, not a new mechanism:
  `discardImports()` with no scope already existed and was reachable only as a
  Delete keypress on one staged track, so a pile of staged imports had no way to
  be dropped at once. No dialog — the label asks (`Discard 2 albums now?`),
  leaving the pane disarms, no timeout, no color (the documented caution amber is
  still not a token).
- `.subrow .tail/.curname/.cur` became `.panebody .tail/…`: a row that carries a
  readout is not necessarily a drill. Rows now share one `.irow:disabled,
  .mrow:disabled` dim.
- menu.rs labels moved to match the pane ("Scan for changes", "Re-read all
  files") — "Full rescan (all files)" and "Rescan Library" both undersold what
  the incremental scan does (it sweeps orphans too). Rust tests unaffected (65
  green; they assert enabled-flags and the scanning label, not these two).
- Verified live: `{groups: [Import, Scan, Storage], rows: [… "Save imported music
  2 albums" …], foot: "248 albums · 4,434 tracks", reveal: true}`; arming the
  discard → `Discard 2 albums now?`; leaving and re-entering the pane → disarmed;
  real "Scan for changes" completed in <0.6s and the footer gained
  `last scan 04:07 PM`; fake progress through the live store module → `.gstat`
  "Scanning 1,810 of 4,434", role=status, 6 rows disabled. Screenshots
  `/tmp/lib0-crop.png` (before), `/tmp/lib1-crop.png` (armed),
  `/tmp/lib2-crop.png` (scanning), `/tmp/lib3-crop.png` (final).
### Row separation fix (user catch)

First render put the action rows directly in `.group`, so the group's 10px gap
fell between them: 50px pitch where every row list in the app is exactly
`--sidebar-row-size` (the user's setting is 40px, measured root layer 98/138/178).
Now runs of rows live in a `.rows` wrapper with no gap (measured Library
136/176, 250/290, 384/424 — 40px pitch, and the reveal's rows at x=26 = the
14px panebody pad + 12px indent). `/tmp/lib4-crop.png`.

Not changed, and logged as a real question rather than an oversight: Appearance
and Playback are forms, so their children keep the 10px group gap (Appearance
toggle @298 / Accent @324; Playback Repeat @112, seg @138, Shuffle @200,
Equalizer @246, Preset @272). The candidate improvement there is proximity by
relationship — an owner row and its own reveal closer together than two sibling
toggles — not a blanket gap change.

### Ownership spacing: 6 / 12 / 30 (the fix I said I'd offer)

Done in the same pass, in Appearance and Playback (the panes whose content is
mixed controls). `.group` went from `gap: 10px` to a ladder expressed as sibling
margins — `* + *` 12, `.glabel + *` 6, `.reveal` 6, and inside a reveal the label
hug `-6` — so the tightest distance in a pane is now the ownership relation.
Measured after (bottom → top): Repeat 128→134 (6), subject→subject 166→196 and
212→242 (30), Equalizer 258→Preset 264 (6), Preset 304→CUSTOM 316 (12), CUSTOM
330→Preamp 336 (6), slider→slider 364→376 (12). `/tmp/pb9-crop.png`.

`.reveal`'s internal gap moved 10 → 12 so the ladder has three rungs and not
three-and-a-half; the EQ block grows ~22px taller for eleven sliders, which is
inside a pane that already scrolls.

Side effect of the measurement session: **Repeat is now ON (Album)** in the user's
settings — left over from the pass-B keyboard test (clicking a segment enables its
subject through `setRepeatStage`, which is the single door working as designed).
Reported rather than silently reverted, since it changes playback behaviour.

### Second catch: the checkbox was not a row (Appearance spacing)

The ladder was right and Appearance still looked off, because `Toggle.svelte` was
a 16px label: the checkbox row sat as a caption between two 40px rows, so its
pane's rhythm could not match its neighbours' no matter what the gaps said. Fix is
genus, not spacing — `.toggle { min-height: var(--sidebar-row-size) }` (the
compact popover variant opts out; the popover is still 250px tall and its toggle
16px), and the two row-height children in Appearance's Theme group (the checkbox,
the Accent drill) went into a `.rows` run so they touch.

Measured after: Appearance — label→slider 6, slider→slider 12, group→group 30,
label→segment 6, segment→row-run 12, checkbox 294–334 → Accent 334–374 (touching,
40 pitch, same as the root layer). Playback — Repeat 112–152 → segment 158 (6),
segment → Shuffle 30, Equalizer → Preset 6 with both rows 40 tall, CUSTOM → its
sliders 6. `/tmp/ap3-crop.png`, `/tmp/pb10-crop.png`.

The general lesson for this codebase: when a pane "feels misaligned" but the gap
ladder checks out, measure the *children's heights*, not just the gaps — the row
height is the unit every list is built on, and one component opting out of it
silently makes the whole pane read as though the spacing were wrong.

### Third catch: a group title needs air (user, plain words this time)

"SIZES and its slider are basically right after each other." I had made every
label hug its content at 6px, which is the right distance for a *caption* and the
wrong one for a *heading*: a title's job is to stand slightly apart and name what
follows. The label hug is gone — group labels and the `CUSTOM` label now sit at
the 12px neighbour distance, and 6px survives for exactly one relation: a row and
the fields it owns. Measured: `SIZES` 112–126 → first slider 138 (12); slider→slider
12; group→group 30; `THEME` 250–262 → segment 12; rows touching at 40.
`/tmp/ap5-crop.png`. Fewer rules, two rungs of real meaning instead of three
competing ones — the user's complaint was the design over-reaching, not the
numbers being wrong.

### Fourth catch: three checkbox rows are one subject, not three (user)

"Too much separation between the three checkbox rows." Correct, and the cause was
mine: Playback had one `.group` per toggle, so the 30px *between-subjects* rung
separated Repeat from Shuffle. Those three are not subjects, they are three rows
of the one subject the pane is named after. Now a `.subject` wrapper (owner + what
it owns, gap 6) sits inside a single `.group` (peers, gap 12), and the 30px rung is
reserved for panes that actually have titled subjects — Appearance's two, Library's
three.

Measured: Repeat 112–152 → segment 158 (6); segment → Shuffle 202 (12, was 30);
Shuffle → Equalizer 254 (12); Equalizer 294 → Preset 300 (6); Library's staged
reveal, being group state rather than a row's own field, sits at 12 (was the
removed 6px hug). `/tmp/pb11-crop.png`.

The ladder also lost its only negative margin: `.group > .reveal { margin-top:
-6px }` became `.subject { gap: 6px }`, which is the same distance expressed the
way the rest of the ladder already worked — on the container, where it reaches
component roots too. Two user-visible corrections in a row were caused by
expressing a distance on the *element* instead of the *container* that owns the
relationship.

### Fifth catch: "did it according to my klassy settings"

I had set gap 10 and dot 14 by eye and by argument. The honest answer was no — the
app already read Klassy's *colours* and titlebar opacity, but not its geometry, so
the cluster was an imitation with the right palette. Now `WindowDecoration` carries
`dotSize`, `buttonGap`, `marginLeft`, `cornerRadius`, published as `--tb-dot`,
`--tb-gap`, `--tb-margin`, `--tb-radius`, and the CSS consumes them (`.head`'s left
padding, `.tb-traffic`'s gap, `.tb-light`'s size, `--radius-window` itself).

Calibrated against the user's actual render rather than Klassy's docs — screen
scale 1.6, Konsole's buttons 24x24 px (15 logical) on a 42 px pitch (26 logical),
which is button rect 16 (KDE Small icon tier) + `ButtonSpacingLeft=10`, the
small-circle shape filling the rect minus 1px. So dot = tier size − 1, gap =
spacing + slack. After: **pixel identical** to Konsole in one screenshot — 24 px
dots, 42 px pitch, `/tmp/cmp_dots.png`. Two tests in `deco_geometry` lock it.

The lesson is narrower than "read the config": the app already had a pipeline for
one kind of Klassy truth (colour) and I kept hand-tuning the other kind (size) as
if it were a design decision. Where the user has already declared a value, the
design decision is *which* values are ours to make.

### Scan progress moves onto the row, as a filling arc (user)

The text loader under the Scan group grew the pane mid-action and shoved the rows
below it downward while the user was watching them. Now the two Scan rows carry
their own progress: a determinate ring in the row tail (`ProgressRing.svelte`,
16px, `stroke-dasharray` on a fixed 16-unit viewBox so the stroke scales with the
box, sweep starts at 12 o'clock, butt caps so 0% draws nothing, 120ms linear on
the offset because the value is data and an ease would lag the truth it reports).
`scanner.kind` is the plumbing: set by the operation, read by the row. Live during
a full re-read: `aria-valuenow` 37 → 67 → 98, `aria-valuetext` "Scanning 1,677 of
4,487" — the sentence the arc replaced, kept for assistive tech. `/tmp/ring-pane.png`,
`/tmp/r3z.png` (8x).

Two judgement calls worth keeping: the accent is the wrong-looking token at first
(`--accent-text` is #1b1b1f, built for glyphs *on* accent fills, not on chrome), so
the arc takes `--accent`, the same token as the seek fill — the app has one idea of
what "your colour is working" looks like; and the track is 20% of `--text`, which
with a White accent is faint, deliberately — the arc is the message, the track is
only the promise of how much is left.

### The arc died at 85% (user: "check their fills")

Two tail bugs, one mechanism — the ring was unmounting in the same frame the run
ended, and the run ended wherever the last progress event happened to land:
* started full and unwound, because `done`/`total` outlive a finished run → every
  operation now calls `begin(kind)`, which clears the counts and the phase;
* vanished before completing, because the last event was 85%, not 100% → `end()`
  holds that row's arc at full for 300ms (`heldKind`), and only for runs that
  reported progress at all.

Sampled in-page, 200ms apart, after the fix: `200ms:25 401ms:54 601ms:83
801ms:100 1001ms:100 1201ms:gone`, and for the incremental scan, which finishes
almost at once: `348ms:100 496ms:100 648ms:100 797ms:gone`. The rule the pane
follows now: the ring's only direction is forward, and it is allowed to say
"finished".

- Not touched: the footer time follows the browser locale (`04:07 PM`), which is
  not KDE's default 24h convention — left as-is rather than imposing `h23`.

## Import destinations: cascade, not template (2026-08-31, user's library as the witness)

- The old save rebuilt `<musicDir>/<Artist>/<Album>` from tags, which is a guess
  about a layout the user already has. Real library proved it wrong: the
  collection lives one level below the configured root (248 of 250 albums under
  `<root>/Music Files/`), so template saves wrote `Music/Ghost/Impera` beside it.
- `album_destination()` now READS the library, three rules, tested in
  `library/import.rs` (74 → 77 tests over the pass):
  1. **merge** — the album is already on disk: use the folder holding most of its
     tracks (a staged addition to an indexed album joins the album, no sibling).
  2. **artist** — new album, known artist: the parent of that artist's existing
     album folders, by count.
  3. **new** — new artist: `<collection home>/<Artist>/<Album>`, where the home is
     the folder holding more albums than any other (grandparent of each album
     folder, counted by DISTINCT album so one 68-file box set cannot swing it:
     `Blind Guardian/Legacy Of The Dark Lands/CD1, Album/`). Empty library → the
     primary root. Ties: shallowest, then lexicographic.
- The `al.id != staged` exclusion in rule 1 was a bug the user's library caught:
  staged files are folded into the album row their tags name, so the album
  excluded ITSELF, rule 1 saw nothing, rule 2 built a sibling folder and split
  `03. Spillways.mp3` from its ten siblings. The staging-path filter is the only
  exclusion needed. `an_addition_to_an_indexed_album_joins_the_album_it_belongs_to`
  fails on the old code (`"artist"` vs `"merge"`).
- `save_to_library` MOVES (rename, copy+remove fallback), and a moved track takes
  its id with it (`UPDATE tracks SET id = stable_id("tr", new), path = new`),
  because id = f(path) — otherwise re-importing the same file collides on UNIQUE.
- Duplicate = blake3 hash (size only when nothing matches by size), so a move
  never deletes a file on a byte count. `SaveReport { moved, duplicates, vanished }`.
- Rows whose file vanished are FORGOTTEN and reported, which healed the 28
  copy-era ghosts (Impera 21 → 11, Rite Here Rite Now 36 → 18) the first time
  Save ran. Two stray albums later moved by hand into `Music Files/Ghost/` at
  the user's request; the whole library is now one regime (0 rows outside it).

## "Manage imported music": the staged pile gets a modal (2026-08-31)

- Two verbs that acted on the whole pile before saying where it would go are
  replaced by ONE door row (`Manage imported music…`, tail = `N albums`), which
  opens `ManageImports.svelte` (sibling of MusicFolders: same `.scrim`, same
  glass tier, SurfaceClose in the head).
- Per album: artist group label, title + year + count, the resolved destination
  in plain words (`merges into Ghostlights  Music Files/Avantasia/Ghostlights`,
  short path + full path in `title`), and the file list (auto-expanded when one
  album is staged — the list is the reassurance the decision wants).
- Decisions are MARKED, not executed: Save / Discard per album (toggle off by
  pressing again), `Save all` / `Discard all` in the footer (press again to clear
  the pile's marks), and one **Apply**. An undecided album stays staged — closing
  the window loses nothing, which is what makes it safe to leave halfway.
- Footer states the files, not the UI: `1 album to save · 1 album to discard`,
  `Mark an album to apply`; Apply is disabled while nothing is marked. While
  running, the footer's arc is albums-applied + the backend's copy fraction
  (`phase == import` only; the scan after a save has its own totals).
- Pure logic in `src/lib/importPlan.ts` (+16 tests, 57 → 73): grouping, the
  order Apply follows (on-screen order, so the list and the operations cannot
  disagree), path shortening, the wording, durations.
- `discard()` now DELETES THE ROWS too. Deleting only the file left the row for
  the scan, whose policy for a vanished file is to keep it as missing — right for
  a file the user moved, a lie about one they threw away: it still points into
  the staging dir, so the album reads as staged and the freshly emptied door
  lights up again. That is how the 28 ghosts were born. `a_discard_leaves_no_row_behind`.
- Live round-trips, both verified against the real library and cleaned up after
  (Ghostlights 36/36, Popestar 5/5, 4,439 rows, 0 rows missing on disk): two
  unmodified copies → hash duplicates → moved 0, no phantom rows; a modified copy
  → moved into the album folder (37 files, 37 rows); a discarded copy → gone
  without a trace. Modal closes itself when the pile empties, and focus returns
  to the door row (`[data-imports-door]`) unless the door is gone with the pile.
- Spacing: ONE CARD PER ARTIST, albums as touching rows inside it divided by a
  hairline (`border-top`, the panes' `.rows` idiom at content height). That
  resolved a rung ambiguity the first version had: with a card per album, the 6px
  binding an album to its artist label competed with the 12px separating
  neighbours and the reader had to guess which distance meant what. Now the
  grouping is structural — the card IS the artist — and every gap means one
  thing: 6 between an album and its destination/file list (the hug), 0 plus a
  divider between albums, 12 between artists. Card is `overflow: clip`, not
  hidden, so focusing a row cannot scroll the card. Verified live with two Ghost
  albums under one card beside Ghost B.C.'s (`/tmp/mi6-crop.png`).
- Found while screenshotting: five accent-filled buttons hardcoded `color: #fff`,
  which with a light accent (the user runs White) is white on white. All now use
  the luminance-aware `--accent-text` (ManageImports, MusicFolders, TagEditor,
  EmptyState, ExpandedPanel).

### Still open on this feature

- Staging without the copy: stage in place (index the original path), so Save
  moves the file the user actually imported and the cache import dir retires.
- Multi-disc merge into an album that keeps a folder per disc: rule 1 targets the
  folder holding most of its tracks; matching on the disc number is the
  refinement. No album in this library needs it today.
- TagEditor entrance motion, parked with the modal work above.

## Environment quick facts (for fresh sessions)

- Fedora 44, Plasma 6.7.4 Wayland, output 3840×2160 @ scale 1.6 (logical
  2400×1350; fractional — relevant to 0b). GPU: AMD Radeon 780M
  (radeonsi, Mesa 26.1.7); WebKitGTK composites via EGL/gbm.
- mpv 0.41 at /usr/bin/mpv. sudo requires password (user runs dnf lines).
- Dev unit: `systemctl --user restart songstress-dev`; logs `journalctl
  --user -u songstress-dev -f`. HMR rot → cold restart first.
- Raise window: WindowsRunner DBus Match + Run(matchId, "activate")
  (qdbus-qt6, `--literal` for Match output; matchId like `0_{uuid}`).
- **The user's desktop is a 3-part grid: terminal + the app in the LEFT column, a
  browser in the right.** The dev window is hand-placed, not maximized — measured
  2026-09-01: `x=0 y=690`, **1200×660**, dpr 2 (min is 960×600, default 1280×800,
  so it sits *below* the default on purpose). Therefore: **activate only — never
  resize, move, maximize, or "widen it to see the whole layout"** (a resize is also
  the known trigger of the band-clip glitch above, and it breaks his grid). Verify
  the UI **at 1200×660**: 236px sidebar ⇒ ~940px of grid ⇒ **4 columns at the 220px
  tile**, and 660 − 84 (playbar) = **576px of scroll port**; modals (ManageImports,
  Music Folders, About, Tag editor) must read at that height. `spectacle -b -n -a`
  (active window) is the right crop — it keeps the terminal and browser out of the
  frame; prefer a `devctl eval` DOM probe over a screenshot whenever state, not
  pixels, is the question (a screenshot steals focus, an eval does not).
- playerctl NOT installed — verify MPRIS via busctl/gdbus/qdbus-qt6.
- grim NOT installed; screenshots via `spectacle -b -n -a -o <file>`
  (active window) / `-f` fullscreen; background captures for artifacts.
- UI verification loop: the DEV-ONLY devtools bridge (commit e3f127b) —
  `node tools/devctl.mjs click|eval|sample|tail` drives the running webview
  (tile clicks, page-scope eval, rAF frame traces) and reads its
  console/errors via logs/devtools.log. No desktop input needed.
  (The WebKit remote-inspector drop-in opened a port but the handshake
  protocol was never identified — drop-in removed 2026-08-31, the
  tools/inspect.mjs probe kept for the record.
  Svelte scoped CSS silently drops cross-component :has() — use explicit
  state for cross-component CSS hooks.)

## Settings stack rebuilt around KDE fidelity (2026-08-31)

Four panes were four dialects of "settings screen"; they are one dialect now, and the details are borrowed from the platform rather than
invented, which is the point of a KDE-styled DE app:

- **One dismissal family.** `SurfaceClose` is the Klassy dot (fill = KWin's own palette colour via `deco.buttonColors`, corner radius =
  `--tb-radius`, glyph on hover/focus only, 26 px hit box at the platform pitch), used by every floating surface; popovers also dismiss on
  outside-pointer-down and Escape, panes on the dot. The About footer opens an in-glass dialog instead of a native one, so no surface in the
  stack is a different windowing paradigm from its neighbours.
- **Geometry is measured, not eyeballed.** `kde_window_decoration` reads Klassy's own config (and `kdeglobals` for the palette) and publishes
  `--tb-dot` / `--tb-gap` / `--tb-margin` / `--tb-radius` on `<html>`, so the cluster and the panes' left edge match a theme we do not
  implement and a DPI we do not choose. Measured against Konsole's Klassy-rendered buttons: 15 px dots, 11 px gap, 41 px pitch, 41 px targets.
- **One row rhythm.** `Toggle` is the row primitive and takes `--sidebar-row-size` (40) as its min-height — the user's own setting, so the
  stack obeys a change made in Appearance. No borders, no inset cards (translucent cards on 0.7 glass read as a surface with a hole in it).
- **A three-rung spacing ladder carries all the hierarchy**: 6 content-under-own-label / 12 label-to-first-row / 30 between groups, so no
  rule, caption or background is needed. Playback's three options are one group of three label+control blocks: touching inside, ladder outside.
- **The sidebar's bottom row is a cluster** (Save, Rescan, Settings): a door to a settings screen does not get a full row of the sidebar whose
  job is the collection.

**Modal containment (the rule that took three tries).** `contain: paint` on `.app` makes it the containing block of fixed descendants, so
`overflow: clip` + `--radius-window` on the app frame rounds *everything in the window* to the window's own shape — set `--radius-window` from
KWin-reported `--tb-radius`. A scrim inside the frame must never paint its own radius or a square corner escapes the clip; the shared `.scrim`
in `app.css` is the only place the dim exists, and each surface clips itself because a child's own paint containment clips its border-box.
Verifying a change to containment needs a **cold restart**: HMR re-injects styles without recomputing the containment tree, and reloading does
not fix it.

## Import staging moved out of the cache directory (2026-08-31)

Staging was a place: `import_music` copied files into `<cache>/import/`, and a scan indexed that root with no album identity of its own. Being
a place, it cost more than it bought — an interrupted copy left a half-written duplicate only a hash check could find; a staged album could not
show the cover sitting two directories away from its own files; a user-moved file came back as an undeletable ghost; a 400-file import meant
400 writes before the first note played. Staging is now a **state of the row** (`tracks.staged`, migration v2): the import indexes the files the
user pointed at, exactly those files, wherever they are.

- **Identity is separated by verb, not shared by rule**: **path** at import (already a row ⇒ nothing to decide, reported), **blake3 hash on
  the colliding name** at save (identical bytes ⇒ the file just pointed at is deleted and the library's copy stays; different bytes ⇒ a
  ` (2)` name), **(artist, album, disc, track)** at scan. The old name+size dedupe collapsed all three into one guess, and "same name,
  different size" silently created an album twin.
- **Save moves, and the row moves with it.** `relink_track` re-points path and id (id is derived from path; if the id did not travel, the next
  re-import of the same file would hit the UNIQUE constraint and lose the play history that was the point of keeping the row).
- **Discard deletes nothing** outside our own staging root — including the row, which the old code left behind for a scan to find, and which is
  exactly how a discarded album came back as an undeletable "missing" ghost.
- **We never remove a folder we did not write.** The app prunes directories under its own root; the folder you imported from stays, empty or
  not. It is yours.
- **Import reports itself.** `ImportReport { staged, already }` renders in the window the import opens ("Already in your library", with the
  album named), and the Apply receipt reports the moved / deleted-as-duplicate / discarded / vanished counts. 93 tests: 4 new, 6 retired
  together with the code they covered (a test for a deleted rule is deleted with it).

## The surface template: dismissal, and one press = one verb (2026-09-01)

Started as "remove the red dot from the imported-music modal, put an ✕ top-right"; it turned
into the rule that was missing, because the same objection applies to every floating surface.

- **Two dismissal families, one verb each.** The traffic-light dot is the WINDOW's close — the
  user's own KWin close colour, in the window's own circle — and it now lives only in the sidebar
  header, where it is true. Modals AND popovers dismiss with `SurfaceClose`, rebuilt as the boxed
  ✕ at the head's right edge: 28×28, radius 7, the sidebar gear's own glyph (16-unit box, 8-unit
  cross, 1.4 stroke, round caps, drawn SVG — a text ✕ is the third typographic family Music
  folders was already caught using), dim → hover wash → `--active` press, inset accent ring, no
  transition. The settings stack had been written up as "the exception" that kept this shape; the
  exception was the rule and the dot was the deviation.
- **Why the dot was wrong, in one line:** it claimed a verb the surface doesn't have. Dismissing
  the queue does not quit the app, and a circle in the close colour at the close spot says
  otherwise. Fidelity to Plasma means mirroring what the platform does for a *dialog*, and Plasma
  dialogs do not carry a second window button.
- **Flush, not floating:** the box's right edge is the content's right edge (measured 17px = 1px
  border + 16px padding), and its centre is the title's (119.5 / 119.5). About has no head row, so
  it takes the corner (8px) instead. `Clear` in the queue header moved off the far edge for the
  same reason it used to be there: a destructive verb must not neighbour dismissal.
- **Rolled out to all six surfaces** (Manage imports, Music folders, Tag editor, About, the
  equalizer popover, the queue popover); `.q-x` stays a BARE ✕ — boxed + head + always-visible vs
  bare + row + hover-revealed is what tells "put this away" from "remove this row" now that both
  are crosses.
- **One press, one verb.** Escape (and the scrim, and the ✕) does exactly one thing: while a modal,
  popover or the context menu is up, that surface answers and the sidebar's global key router
  stands down — so closing a modal opened from the Library pane no longer pops the pane off the
  stack too. Measured: open → Esc → modal gone, Library pane still active; Esc → root; Esc →
  closed. The router's OTHER keys are gated by the same predicate, which also ends `s` pushing the
  settings stack in behind the About scrim. The context menu had no Escape owner at all (the key
  popped a settings level instead of dismissing the menu on screen) and now has one.
- **Why a predicate and not `stopPropagation`:** all six handlers listen on `window`, and
  same-node listeners all run regardless — only `stopImmediatePropagation` cuts them short, and
  Svelte re-attaches them on update so order is not guaranteed. `surfaceOpen()`
  (`src/lib/stores/surfaces.svelte.ts`) makes the precedence a fact each handler checks. This raced
  twice before it was named: the earlier fix moved About's handler INTO the router, which made
  About correct and left everyone else wrong.
- **A layer holds its own content while it animates out** (his bug report: accent swatches
  flashing on the way back from the EQ preset submenu). Not a timing bug — an identity bug: the sub
  layer is `{#if menuSub === "eq-preset"} … {:else} <accent picker> {/if}` and `menuSub` flips to
  null at t=0 of the back, so the layer re-renders into the wrong branch while still on screen. The
  mirror case flashed the preset list, and the detail layer had the same flaw as a blank panel
  sweeping across. `heldDetail`/`heldSub` + `shownDetail`/`shownSub` drive rendering, titles and
  branch conditions; `atDetail`/`atSub`/`inert`/`aria-hidden`/`class:active` still read the live
  state, so the hold is paint-only. Measured after the fix: back out of eq-preset → 0 swatches, 10
  preset rows (was 12 swatches, 0 rows).
- **Restart geometry: what a Wayland client can and cannot remember — and why nothing was
  added.** `tauri-plugin-window-state` was tried with `SIZE|MAXIMIZED` and deliberately NOT
  `POSITION`, because a Wayland client never learns where the compositor put it: measured on one
  window, Tauri's `outer_position()` = `(0,0)` while KWin reports `x=560 y=290`, and
  `inner_size()` = `2560×1600` physical for a 1280×800 CSS window (Tauri scale 2 — the plugin's
  units are its own and self-consistent, which is why SIZE round-trips and POSITION does not).
  Saving position would have stored `(0,0)` and `set_position(0,0)` on next launch, moving the
  window to the top-left corner: worse than nothing. And SIZE does not survive the case that
  matters either — the plugin writes on `RunEvent::Exit`, which a kill never reaches (verified:
  after three rebuild-kills the state file still did not exist and the window came back
  1280×800). **Reverted the same day:** the owner's tiling clips and resizes on drop, so ONE
  drag fixes both axes, and a debounced save on `tauri://resize` to cover the kill path was
  declined as a background thread + disk writes for nothing. A KWin window rule and a seeded
  state file were offered and declined too. The protocol is manual and lives in AGENTS.md — do
  not re-add the dependency without a case where the size is not already free.
- **Version files are watched.** Bumping `tauri.conf.json` / `Cargo.toml` makes `tauri dev`
  rebuild and RESTART the app — which, per the rule above, costs the owner a window re-place. Do
  the bump before he places the window, or tell him it is coming.
- Gates: `npm run check` 0 · 73 vitest · 81 `cargo test --lib` · `npm run build` clean. DESIGN.md's
  dismissal section is rewritten ("Two families, one verb each" + a new "One press, one verb") and
  is now the authority; the old "one family, two dots" text is gone.

### The equalizer popover's head, rebuilt (2026-09-01, same template pass)

The head now reads **identity + state + dismissal** — `✓ Equalizer` at the content's left edge,
the preset name (read-only, Micro) hugging the ✕ at the right — the curve stays the body, and
`Playback settings ›` moved to the BASE, under the seam.

- Why: the route out used to be the head's first item, which made this the only surface in the
  app whose first row was *navigation* rather than a name. And the old justification for the
  inverse ordering ("an instrument surface keeps its master at the base") bought nothing the head
  grammar does not already buy — leaving belongs with the exit, and what-a-surface-is belongs at
  the top. DESIGN.md's "Ordering is genre, not law" passage is rewritten to say so; the popover
  and the Playback pane now agree instead of deliberately disagreeing.
- The seam's job changed with it: it used to separate the work from its *state*, now it separates
  the work from the *exit*. `.own` lost `cursor: default` (it is actionable again) and the route's
  negative margins were re-aimed: `margin: 0` on the button, `-6px` on the footer row so the wash
  bleeds to the padding edge while the label stays aligned with the head's left edge.
- No `justify-content` on the head: the preset's own `margin-left: auto` spends the slack, so the
  grouping is owned by the item that forms it rather than distributed between three.
- Measured at 1200×660 (DOM probe, not a screenshot): toggle x=772 (content edge), preset ends
  1129, ✕ 1141–1169 with 17px to the popover's outer edge (1px border + 16px padding — flush, as
  every surface now is), 12px preset→✕; route 766,533 against a seam at 526 and a head bottom of
  355. Popover 431×255.


## Imported-music modal: critique → fix pass (2026-09-02, all six findings)

Critiqued loaded, not empty: a mock pile through the app's own import store — 5 artist
cards / 8 albums / 53 tracks, incl. a 50-char album title, diacritics, a merge, a
byte-identical duplicate, an already-indexed file, marked states, a real Apply and its
receipt. Baseline **30/40**, 3 P1 + 3 P2. All six fixed; snapshot
`.impeccable/critique/2026-09-02T00-40-17Z__src-components-manageimports-svelte.md`.

- **[P1] The report band.** "Already in your library" and the Apply receipt rendered
  AFTER the album cards, inside the scroller — measured 389px shown of 864px content,
  475px hidden, so the two statements the window exists to make were off-screen on
  open and the receipt (the one about deleted files) was the furthest from view. Both
  moved into a `flex: none` band under the head's seam, outside the scroller, receipt
  first, each block labelled (`Just applied` / `Already in your library`) so the newest
  fact has the most authority — previously the receipt was a bare sentence while the
  older list had a bordered card. Band is bounded (3 names, then `and N more`, full
  list in `title`) precisely because it does not scroll. The bordered inset box is gone:
  it was a third container dialect in a window of filled cards and borderless rows, and
  its label reused `.mi-glabel`, so the first read was "an artist called *Already in
  your library*". Now sentence case, no tracking — the app speaking, not a group heading.
  Measured after: band 60px, `aboveBody: true`, only 34px hidden on a 4-album pile.
- **[P1] The destination survives.** The gloss (`merges into …`) ellipsizes FIRST and
  the path wraps instead of truncating (`flex-wrap: wrap`, `.mi-rule { flex: 0 1 auto }`,
  `.mi-path { flex: 1 1 auto; overflow-wrap: anywhere }`). Before, a long album title
  collapsed the path to `· M…` with the full text only in a hover `title` — mouse-only,
  for the row whose whole job is answering *where will this end up*. Measured after:
  every path `scrollWidth == clientWidth` (the 399px one included), the row grows to
  3 lines instead of lying.
- **[P1] Containment.** `App.svelte` sets `inert` on `.stage` from a new `modalOpen()`
  (modals only — popovers and the context menu keep their trigger live, or the
  focus-handoff to `eqBtn`/`qBtn` breaks); `src/lib/focusTrap.ts` gives every dialog an
  edge-only Tab ring, wired into all four. `aria-modal="true"` had been a claim nothing
  honoured: Tab walked the grid, sidebar and playbar behind the scrim — the same failure
  the settings stack was fixed for. Verified: `stage.inert === true` with the modal up,
  and Tab from the last control wraps to the ✕.
- **[P2] Drift collapsed into the system, not documented around it.** Added the
  `--caution` / `--caution-line` / `--caution-wash` set (amber, light-theme value
  `#b26a12` for contrast) and moved the discard mark, Music folders' Remove +
  confirmation row, and the Tag editor's error text onto it — the undocumented
  `#ff8f8f` family is gone (6 literals). `.mi-path` drops the mono stack (DESIGN.md:
  one voice, no pairing). Focus radius 4 → 8; `.mf-remove` 5 → 7. Detector on the six
  touched files: **5 findings → 0**.
- **[P2] A surface materializes.** `.scrim` fades the dim over 140ms and
  `.scrim > .glass` scales in from `.97` over 180ms — one pair of rules, four modals,
  and CSS on purpose: the global `prefers-reduced-motion` kill switch cannot reach a
  `svelte/transition` (WAAPI), so a JS entrance would run at full speed for the users
  who asked for none. The exit is still instant; symmetric outro would need a
  two-phase close per surface, and that trade is recorded in DESIGN.md rather than
  silently made.
- **Mock discipline:** staged through `importMusic()` (the real flow, so the receipt
  rendered), never Save — discard only, so no file moved and nothing of the owner's was
  touched; his legacy Infestissumam was explicitly un-marked before Apply. Verified
  after cleanup: `4464 rows / 12 staged / 251 albums`, 12 cache rows intact, `/tmp/mock`
  deleted.
- **Corrections to the critique itself:** it claimed "no keyboard route into the
  manager" — wrong, `s` opens the stack, arrows walk the pane, Enter opens the door.
  Three minors were left unfixed on purpose: `scrollbar-gutter` does nothing for an
  overlay scrollbar (the fix would cost the cards' flush-right alignment), the `▸/▾`
  caret is still a font glyph, and `saveImports(…, trackId)` is still a per-track API
  with no UI.

## First-scan repro harness, and the progress bar that never moved (2026-09-02)

Wanted: the dev instance *held* on the first-launch screen — scan in flight, grid
still empty — so that state can be looked at rather than caught mid-blink. Built
as two independent, reversible pieces; nothing of his library was touched.

- **The park (Rust, `lib.rs::scan_park` + one guard in the `scan_inner` progress
  closure).** `SONGSTRESS_SCAN_STALL_MS=<ms>` sleeps the *blocking scan task* the
  first time the file loop crosses `SONGSTRESS_SCAN_STALL_AT` percent (default 40,
  0-100) and then carries on. The position is the whole point: the park is inside
  `run_scan_files`' per-file loop, **before** grouping and before any upsert, so
  while it holds the DB has nothing and the grid is *provably* empty — the same
  screen a first launch shows, not a UI that was told to lie. Unset ⇒ `scan_park()`
  returns `None`, one branch, zero cost. `SCAN_RUNNING` stays set the while, so a
  second scan answers "scan already running" and the watcher just keeps its dirty
  flag: an honest stuck state, including its side effects. At 3 600 000 ms it also
  self-heals an hour later.
- **The fresh library (no schema wipe, no backup-and-pray).** systemd drop-in
  `~/.config/systemd/user/songstress-dev.service.d/first-scan.conf` sets
  `XDG_DATA_HOME=/tmp/songstress-first-run`, and Tauri's `app_data_dir()` follows
  XDG — so the whole app (DB *and* WebKit's localstorage/CacheStorage) relocates.
  Its `songstress.db` is `sqlite3 ".backup"` of the real one with
  `tracks/albums/artists` emptied and `lastScan` cleared: **his prefs, roots,
  accent and tile size intact, zero albums** — a configured install that has never
  scanned, which is exactly the moment before the grid populates. Real DB verified
  afterwards: 4464 rows, untouched mtime. Revert = delete the drop-in,
  `daemon-reload`, restart.
- **[P1 bug found by holding the state] The first-scan progress bar never moved.**
  `EmptyState.svelte` had `style:transform="scaleX(${pct / 100})"` — in a *plain*
  (non-backticked) attribute Svelte does not treat `${…}` as an interpolation: it
  emits the literal `$` and the `{expr}`, so the runtime declaration was
  `scaleX($0.4)` — invalid CSS, dropped without a word, `style=""` on the node, and
  `.fill` sitting at the stylesheet's `scaleX(0)` for the entire scan. The only
  feedback the screen actually gave was the `Scanning — 1806/4517` text line. Fix:
  backtick the template literal. Measured parked at 40 %: `style.transform =
  scaleX(0.4)`, fill 128px of a 320px bar. Lesson generalizes — see AGENTS.md.
  *(The bar itself is gone as of the entry below — the lesson is what stayed.)*
- **Gates:** svelte-check 0/0 · vitest 73/73 · cargo --lib 81/81 · build ok.

## Loading skeletons: the grid and the artist list (2026-09-02)

Decision that shaped it: **no progress bar on the first scan at all.** The stage
shows the shape of what is coming until files land, or — if the scan ends on an
empty library — the welcome state, which is a decision and not a wait. The park
harness from the entry above held that screen open for the whole build.

- **One predicate, two surfaces.** `src/lib/loadingState.ts::libraryLoading` is
  the single definition of "nothing to show, and something in flight"; the grid
  and the sidebar each build the same `LoadingFacts` object, and the type makes
  both of them answer for every fact rather than let one surface quietly stop
  checking one. Facts, not stores: the module stays pure and vitest-covered.
  Rules: never over content (`albums > 0` ⇒ false, so a re-scan keeps the real
  grid and the sidebar's "Updating library…" note); a scan in flight is
  immediate; boot gets a grace (`BOOT_GRACE_MS = 150`).
  `library.scanning` is load-bearing next to `scanner.running` — a
  **watcher**-triggered first scan never passes through the frontend's `begin()`,
  so `running` is false and only the progress listener knows the scan is up.
  Without it the grid would flash "Pick the folder that holds your music" mid-scan.
- **`library.scanning` was also the last reader of two dead fields:**
  `library.scanDone`/`scanTotal` had no consumer anywhere (EmptyState read
  `scanner`, the rings read `scanner`) — deleted with the bar, not before.
  `scanner.done/total/phase` stay: the Library-pane rings and ManageImports use
  them, so the numbers live on as chrome, not as a stage.
- **Geometry is the point.** `GridSkeleton.svelte` re-derives the real tile:
  `columnCount(gridWidth, ui.tileSize, 20)`, `repeat(var(--cols), minmax(0,1fr))`,
  `gap: 12px var(--gap)`, square cover at `--radius-cover`, 13px cover→caption,
  8px bottom padding. `skeletonRows` = `ceil(stage / (rowH + gap))` capped at 4 —
  `TILE_STACK = 52` is the *real* tile's non-cover height (the skeleton's own CSS
  lands 3px short, which is not a reflow). Measured parked at 1200×660 / `tileSize
  220`: cols 3, cover `295×295` at `(256,20)`, 2 rows, `.content 747/576` (the
  second row runs under the playbar shelf, which is what the first real row does).
  The sidebar counts with `floor` minus the real "All Artists" row: `nav 470/470`,
  **zero scrollbar** — the one surface with a visible `::-webkit-scrollbar`.
- **Deleted:** EmptyState's whole `{#if library.scanning || scanner.running}`
  branch — `Building your library…`, the `Starting scan…` fallback, the 320×6
  bar and its (just-fixed) `scaleX` binding, plus `.bar`/`.fill`. It is
  welcome-only now, which also retires the centered-column/collision problem:
  the skeleton is in flow inside `.grid`, so the absolute `.empty` never has to
  share the stage with it.
- **New to the codebase: an `infinite` animation.** The kill switch is
  `* { animation-duration: 0.01ms !important }`, which on an infinite loop is not
  "reduced" but ~100k cycles/sec. `.sk::after { animation-name: none }` inside the
  media query wins the `!important` tie on specificity **and** touches a different
  longhand than the global rule owns, so the sweep stops instead of racing. The
  static material still reads as waiting; `aria-busy` says so out loud.
- **Search is disabled while loading** (owner's call): a query typed into two
  placeholder surfaces can only match nothing, and a field that takes the typing
  and answers with silence is the worse lie. The magnifier dims with it via
  `.search:has(input:disabled) > svg` (`:has` because the icon precedes the
  input), and the `title` swaps from the `/` tip to "Search once your library is
  built".
- **Dev hook, because the park can't be re-entered:** `window.__skel(true|false)`
  (`main.ts`, `import.meta.env.DEV`-gated → `library.devLoading`). After a page
  reload — which any store-module edit causes — the backend still holds
  `SCAN_RUNNING`, so clicking Rescan dies with "scan already running" and the
  loading state becomes unreviewable without a Rust rebuild (and a window drag).
  This is what the skeleton was verified with.
- **Known consequences of "no bar", stated once:** a hung scan is now a shimmer
  that never ends (the harness proves it — an hour of it); the fill is one instant
  at `scan-finished`, because grouping and every upsert happen after the file
  loop, so a progressive resolve would need per-group commits and a rethink of the
  removal sweep. Both accepted, neither designed around.
- **Gates:** svelte-check 0/0 · vitest 84/84 (11 new) · cargo --lib 81/81 · build ok.

## A failed scan now says so, and always ends its own shimmer (2026-09-02)

The skeleton made an old hole load-bearing. `scan_inner` had **one** terminal
emit and it sat *below* the error propagation (`lib.rs:523` —
`let counts = result…??;`), so a run that died on a missing root, a DB error or a
panicked blocking task reset `SCAN_RUNNING`, returned `Err` to the caller, and
emitted nothing. The webview's `library.scanning` is cleared by `scan-finished`
and by nothing else → **a failed first scan shimmered forever.** (Before the
skeleton that same bug read as "Building your library… / Starting scan…" stuck —
same failure, less convincing. It is exactly the shape of the `music root null`
bug AGENTS.md records.)

- **Fix at the cause, not the symptom.** No watchdog, no timeout: the emit moved
  above the `?` and `ScanSummary` grew `error: Option<String>`, so the terminal
  event now fires on **every** path — success, hard failure, panic (`JoinError`
  folds into the same match with the others). Returning `Err` is kept: callers
  that chain a scan (`add_music_folder`, `save_imports`) must still abort, and
  `rescan()`'s `catch` still logs. Additive field; `scan-finished` has exactly one
  frontend reader, which ignored the payload entirely until now.
- **The UI half:** `library.scanError` ($state) ← the event's `error`; cleared by
  `begin()` so a retry can't leave the old reason on screen behind the new
  skeleton; a failed run does NOT stamp `lastScan` ("scanned 14:02" for a scan
  that didn't happen) and does not re-read a dump that cannot have changed.
  `EmptyState` gained its failure branch: *"Your library wasn't updated"* + the
  reassurance line (nothing was moved, changed or deleted) + the scanner's own
  sentence in `--caution` — the tint the tag editor's failed SAVE already uses, so
  a failed scan doesn't invent a second error colour — and the ghost button
  relabels *Rescan → Try again*.
- **Verified end to end on the sandbox**, not argued: `invoke('scan_library',
  {root:'/tmp/definitely-not-here'})` from the bridge → journal `[scan] FAILED:
  music root /tmp/definitely-not-here does not exist` → screen reads "Your library
  wasn't updated" with that exact string at `rgb(242,163,60)` and
  `aria-busy="false"` → *Try again* clears it → real first scan → skeleton (6
  covers / 10 rows) → parked at `1807/4517`. A failure, a retry and the stuck
  state, in one chain.
- **Also here: the sidebar's scan note had the same blind spot.** It gated on
  `scanner.running` — "the frontend asked" — so a watcher-triggered rescan was
  invisible there too; it now reads `running || scanning` like `libraryLoading`.
  Paired with `NOTE_GRACE_MS = 400`, because the union is only safe if a ~56ms
  background pass cannot mount a line for a single frame (a flicker, not news).
  Both grace constants live in `loadingState.ts` now — one rule about waits, one
  place to tune them.
- **Gates:** svelte-check 0/0 · vitest 84/84 · cargo --lib 81/81 · build ok.

## The arrival entrance: rows fade up when a placeholder gives way (2026-09-02)

`animate` skill, run in order. **Gate:** the transition happens on a first scan and
on a boot slow enough to have shown a placeholder — rare, first-run tier, so the
delight budget applies. The same motion on a warm launch or an artist switch would
have failed the gate outright (100+/day, keyboard-adjacent), which is why the
trigger is the interesting part of this change, not the keyframe. **Purpose:**
preventing a jarring change — 251 covers otherwise teleport in over an empty grid.

- **Armed by the store, in the same write batch as the data** (`library.load()` →
  `entering` for `ENTER_MS = 360`). Deliberately NOT a component `$effect`: an
  effect runs *after* the DOM update, so the class would land a frame late and every
  row would paint at rest, jump back to the start of its animation, and blink. The
  condition is `albums was 0 → dump has albums → (fromScan || scanning || bootSlow)`
  — "a placeholder was replaced", not "content arrived". `scan-finished` passes
  `load(true)` because it clears `scanning` before calling.
- **Ingredients:** CSS `@keyframes` (chosen over WAAPI/`svelte/transition` — it runs
  off the main thread while covers decode and `album_colors` IPC fires, and the
  global reduced-motion kill switch can reach it; PLAN.md's modal note is the same
  lesson), `opacity` + `transform: translateY(8px)` only, `--ease-out` — the
  codebase's verified curve, not the skill's generic `cubic-bezier(0.23,1,0.32,1)`,
  because extending existing tokens beats forking them — 200ms, 30ms per-row ladder
  capped at 150ms. Per ROW, not per tile: the row owns the position, the tile owns a
  cover, and this WebKit paints layer churn as blank covers (No-Lift Rule).
- **Two things that would have shipped wrong if I hadn't measured them.** The cut
  `:nth-child(n + 8) { animation: none }` (for the 335 surfaces a 84-row grid +
  251-row list would otherwise composite) landed *inside* the sidebar's visible
  rows, and everything past a cut appears at t=0 — i.e. below the cascade, arriving
  early. The boundary is now `n + 24`, below the fold of the tallest list. And the
  sidebar's first child is `All Artists`, a control that never left: `animation`
  changing on a persistent node re-runs it, so the entrance would have blinked a
  button the user might be reaching for. Excluded by a scoped rule.
- **Reduced motion:** the kill switch collapses the duration anyway, so what had to
  be disarmed by hand was the LADDER — a sequence of zero-duration arrivals is worse
  than no motion. `.enter > :nth-child(n) { animation-delay: 0s }`; the
  pseudo-class is load-bearing, `.enter > *` loses the specificity tie to the ladder.
- **Revised after the owner watched it: too fast.** The trace was the diagnosis, not
  just the taste note — row 1 reached **90% opacity at 74ms** of a nominal 200ms,
  because `--ease-out` spends 90% of the distance in the first ~37% of the duration.
  So the duration was mostly invisible settling and the only real lever was the
  SPREAD. Now 250ms / 10px rise / 50ms ladder capped at 300ms (~440ms for the front
  six rows, ~550ms total), with `ENTER_MS` raised to 580 — it must exceed duration +
  cap, or a row still holding at `opacity: 0` inside its delay snaps visible when the
  class drops. Measured after: `103ms 93,73,12,0,0,0 → 205ms 100,99,93,73,12,0 →
  439ms all 100`. DESIGN.md carries the new numbers (it is the authority, not the log).
  Repro note: the probe read `100,100,…` flat once — because `app.css` had gone
  empty-serving AGAIN after a non-atomic rewrite, so there was no CSS to animate. The
  tell is `--gap` reading `""`; `touch src/app.css` fixes it. Always check the token
  before believing a CSS measurement.
- **Measured, in this WebKit:** injected ladder probe — t0 all armed rows at
  `opacity 0 / matrix(1,0,0,1,0,8)`, t+90ms child0 `0.95 / 0.44px`, child1 (30ms
  behind) `0.84 / 1.30px`, child5 (at the 150ms cap) still at rest, children 24+
  `none`; t+510ms all `1 / 0`. On the live nodes after the restore: grid rows
  animate, `.scroll`'s first child reads `none` ✓, its second `enter-row` at
  `0.55 / 3.57px` mid-flight. Warm boot on the real library: 251 tiles,
  `animationName: none`, no `enter` class ✓ — the gate holds.
- **Dev-cycle cost, recorded because it bit:** mid-task the dev server was serving
  `src/app.css` as a module with `__vite__css = ""`, i.e. the page had NO global CSS
  (every token empty, `.glass` blur `none`) while all component styles looked fine.
  `touch src/app.css` healed it in ~1s. That is the AGENTS.md cold-start-CSS class
  of failure, in its global-sheet form, and a synthetic-CSS probe found it in one
  call where a screenshot would have sent me chasing a layout bug that did not exist.
- **Third pass: slower, and the grid unfolds rather than marches.** "A bit slower"
  moved the ladder to 70ms/420ms cap over a 320ms fade (ENTER_MS → 900, since it must
  exceed duration + cap or a delayed row snaps in). The unfold request exposed that
  the ROW was the wrong unit for a grid: at a 295px tile barely 1.5 rows are on
  screen, so a row ladder there is a wall arriving late. The grid now animates TILES
  with `--i = row + column` — a DIAGONAL, because `row * cols + column` marches
  left-to-right and reads as a typewriter — with its own dials on `.tile` (declared on
  the animated element: a `--enter-*` on `.grid.enter` would be overridden for every
  child by the shared `.enter > *` declaration, which is exactly the trap the sidebar
  comment now records), rows opting out so a cover doesn't travel twice, and a cut at
  the 9th row so 75 off-screen covers don't composite. Reduce pauses it via
  `animation-play-state`, not `animation-name` — the latter is not the longhand the
  global `!important` kill switch owns, so the two mechanisms don't fight.
  Traced on the live grid: `135ms → 93,55,0,55,0,0`, `247ms → 100,96,69,96,69,0`,
  all settled by 545ms; the equal pairs ARE the diagonal (tiles (0,1) and (1,0) share
  `--i` 1). Sidebar: delays 0.07/0.14/0.21/0.28s from inline `--i`, All Artists `none`.
- **Gates:** svelte-check 0/0 · vitest 84/84 · cargo --lib 81/81 · build ok.
- **Landed:** all of 2026-09-02's loading work — harness, skeletons, scan-error
  emit, entrance — is one commit, **Version 0.8.0**. No RPM rebuilt (owner's call);
  the next `npm run tauri build` / RPM target will carry 0.8.0.

## The playbar volume slider sheds its native chrome (2026-09-03)

- User finding: at 100 the knob sat flush but a sliver of unfilled track stayed
  visible to its right. The cause was shape, not math — `.volume input` was the last
  natively-rendered range in the app (`appearance: auto` + `accent-color` only), and
  WebKitGTK's native paint insets the thumb's travel inside a full-width track. The
  settings panes had already solved this (glass slider: `appearance:none`, 4px rounded
  track, value-sized `accent → hover` gradient, accent dot). The volume row gets the
  same treatment, inline `style:background` gradient on `playback.volume` — at 100 the
  fill is the whole box and there is no sliver to be. The seek bar is still native and
  carries the same quirk at track end; deliberately not touched (owner's call pending).
- **Gates:** check 0/0 · vitest 84/84 · build ok. No restart (hot-reloaded).

## The broadcast mirrors the pane — Global Menu rebuild (2026-09-03)

- The ask: bring `menu.rs` (which now feeds ONLY the Plasma Global Menu — every
  sidebar pane is bespoke) up to date with what the sidebar actually offers, sections
  and all. Owner decisions, all five: sliders stay in-window (with a door back),
  manage-imports opens the modal, footer resets join, View renames to Appearance,
  and the parked 3-state theme fix rides along.
- **Separators are a model concept now.** `MenuItem` gains `separator` (and `radio`)
  with serde defaults; `menu.rs::build()` emits breaks that mirror the panes' group
  gaps — Playback: transport | repeat/shuffle | equalizer | reset (3), Library:
  import | scan | storage (2), Appearance: theme | gradient+accent | more+reset (2).
  `menu_dbus.rs` exports them as `type:"separator"` (visible, never enabled — a
  stray Event hits the `!enabled` early-return), and checkables report
  `toggle-type: "radio"` for the theme trio (KDE's importer degrades an ungrouped
  radio to a checkable, so this is the safe way to say "mutually exclusive").
- **`MenuState` de-flattened:** `themeDark: bool` → `theme: "light"|"dark"|"system"`
  — the MODE, pushed from `ui.theme`, not `resolvedTheme()`; the App.svelte effect
  tracks the mode + `playbarGradient`. `anyStaged` → `stagedCount` (albums, same
  reduce the sidebar door's tail uses — it gates the row; the owner declined
  putting the count in the label, so the label stays static). Re-checked
  the rest: shuffle/repeat/eq were already honest; playing's Option<bool> is a
  genuine tri-state feeding a binary label, fine.
- **New doors, same setters.** `appearance.more` / `appearance.accent` open the
  sidebar stack at the Appearance pane / its accent sub layer (state-only —
  `openSettingsLayer`; the window typically lacks keyboard focus when the panel is
  clicked, so there is no focus to hand off). `library.manage-imports` mirrors the
  sidebar's door into the modal; `library.save-imports` is gone from the model (the
  modal owns Save/Discard now; ExpandedPanel's context menu keeps calling
  `saveImports()` directly — that store function stays). `help.about` was a silent
  no-op in the dispatcher (`() => {}`) — it now calls `openAbout()`; the null
  opener fallback covers "no trigger element to restore focus to."
- **Resets moved to the stores** (`resetAppearance` in ui.svelte.ts, `resetPlayback`
  in playback.svelte.ts) — the footers and the Global Menu rows now call one
  function each; "the two surfaces cannot drift" is the repo's own principle and
  this was the drift. `cycleTheme` deleted (nothing cycles a tri-state when the
  menu shows all three).
- Sidebar fallout: `rootPanes` filter now also drops the Rust-owned `"appearance"`
  (the constant and the menu id collide by design — `openDetail("appearance")`
  renders the bespoke pane); `detailItems` filters separators out and renames its
  save-imports clause to manage-imports. The generic irow fallback loop stays
  correct-by-filtering even though no pane currently uses it.
- Verified on the wire (dbus-send depth 2): all new labels present, 2 separators in
  Library's export, `3× "radio" + 1× "checkmark"` in Appearance's. Two owner
  corrections followed: manage-imports keeps its STATIC label (the count lives only
  in the sidebar door's tail — the owner declined it in the menu row, so the test
  now asserts no count), and `help.about` was a silent no-op in the dispatcher —
  it now calls `openAbout()`. About-open verified end to end: a dbusmenu
  `Event(400,"clicked",…)` brought `.ab` into the DOM (`true`), and `.ab-close`
  took it back out (`false`). Note `menu_activate` is the Rust-native path for
  playback ids only; `help.about` re-emits to the frontend as `menu-action`.
- **Gates:** check 0/0 · vitest 84/84 · cargo --lib 83/83 (4 menu tests rewritten to
  look items up by id — the positional-index tests were exactly what broke today;
  new tests pin the separator counts, the one-checked radio invariant, and the
  mode-not-resolution theme report) · build ok. Rust change → cargo rebuilt and the
  app restarted (cost: one window drag).

## Right-click everywhere: the context-menu pass (2026-09-03)

- **Grid tiles got a menu.** Albums already had one on the expanded panel's
  header; the tile was a dead right-click. Extracted ONE builder —
  `src/lib/albumMenu.ts` — and both doors (tile, panel header) call it; what
  used to be two hand-maintained copies is one list again, unable to drift
  (same principle as the store-owned resets).
- **"Open containing folder" — and paths never enter the webview.** New Rust
  command `reveal_container(albumId | trackId | artistId)`: the menu says
  WHICH row, the DB says WHERE. A track resolves to its FILE (`dolphin
  --select` → folder open, file highlighted); an album to artwork's
  `dominant_dir` (majority rule, multi-disc safe); an artist to the DEEPEST
  folder holding all of its files (`common_parent`, component-wise — the
  `/m/b` vs `/m/bee` string-prefix trap is pinned by a test). Vanished file →
  parent dir; vanished dir → honest error; no dolphin → xdg-open. Verified
  live on all three levels.
- **Bug the artist work caught: the Dir demote.** The first cut branched on
  `target.is_file()`, so an album (whose resolution is ALREADY a directory)
  got demoted to its PARENT — revealing the artist folder above the album,
  and looking plausible enough that the first live check missed it. The
  resolver now returns `Container::File | Container::Dir` and only a FILE
  target is ever demoted. The artist-level check doubles as the regression:
  the same row now opens the album folder itself, not one above it.
- **Separators are a context-menu concept too.** `MenuItem.separator` +
  shared `SEP` (same vocabulary as the Rust Global Menu model); the renderer
  draws an inset hairline and keys the each-block BY INDEX — SEP is one
  shared object and Svelte's default identity keys would collide the moment
  a menu holds two breaks. Album: Edit | playback pair | container | import-
  state (the last break only when there IS an import state). Track: Edit |
  play verbs | container.
- **Playback pairs replace single verbs**: album `Play album next` became
  `Play album` (first file that exists — a missing opener must not make the
  row a dead click) + `Add album to queue` (front, the old behavior); tracks
  got `Play now` above `Play next` / `Add to queue` — same
  `playTrack(albumId, indexById)` call the row-click path uses, so the
  search-filtered view cannot play the wrong offset.
- **Right-click selects (tracks)**: the menu acts on what you pointed at,
  and the keyboard verbs (Enter/Delete) come alive on that row. Artists
  deliberately do NOT select on right-click — selecting (a heavy filter
  change) just to look at a menu is a worse surprise than on a track row.
- **Artist rows got the minimal menu** — one item (an artist has no tags of
  its own and nothing singular to play). "All Artists" is left without one:
  it's a control, not a folder, and "the container of all artists" is a
  claim the row shouldn't silently make.
- **Dismissal grew up**: the menu now dies on scroll (captured on the whole
  document — the grid scrolls in its own container), `focusin` outside the
  box (Tab walks away), and window `blur` (a panel-menu click sends no DOM
  event; without blur the menu floats over a library you walked away from).
  No open-time race: the opener's mousedown focus lands BEFORE the
  contextmenu event registers the listeners.
- **Dev-loop notes:** plain `.ts` module chains (albumMenu.ts, the stores)
  can HMR half-dead — the menu kept rendering the pre-separator list until a
  `location.reload()`; when a DOM probe contradicts a just-landed edit,
  reload before debugging. Synthetic `blur`/`scroll` fired in the SAME tick
  as the open missed the not-yet-attached listeners — dismiss probes need
  open and the trigger in separate evals.
- **The staged-album doors all point at the modal.** The album menu's direct
  `Save to library`/`Discard import` became one `Manage imported music…`
  opening the modal AT that album, and the expanded panel's `Imported` pill
  became a BUTTON doing the same (the tile's corner badge stays a label — it
  lives inside the tile's own button). `openImportManager(albumId?)` gained
  the address: the row expands even in a tall pile, `scrollIntoView`s, and
  wears a 1.6 s accent ring (`mi-attn`, driven by STATE — a classList string
  is invisible to Svelte's CSS scanner and rightly warned). The focus flag
  is consumed on read, and close clears it. Rationale is the repo's own:
  the DESTINATION is what the decision is about, and only the modal states
  it per album — a menu applying the move blind skipped the one fact the
  user was deciding on.
- **Gates:** cargo --lib 85/85 (new: artist arm of container_target,
  common_parent incl. the string-prefix trap) · check 0/0 · vitest 84/84 ·
  build ok. Rust change → cargo rebuilt → one more app restart (one more
  window drag).

### The pending-import dot on artist rows (2026-09-03, same pass)

- An artist with staged albums now wears an accent dot after the count in
  the sidebar list (`.count.pending::after`, the app's "a decision lives
  here" tier, same badge language as the mode buttons), with a hover title
  spelling the claim: "N albums — M awaiting import". Trailing placement so
  the count keeps its column alignment. Derived once as `stagedCounts`
  (artistId → staged albums) next to `albumCounts`. "All Artists" stays
  undotted — the pile is the artist's story, and the Library pane already
  owns the global number.
- Probe note: the staged pile is a MOVEABLE target during a session — the
  owner's own testing moved it off Aimer mid-verification, and the first
  probe dutifully "failed" against a stale assumption. Dump the actual
  state (every row's class + the staged tile's caption) before debugging a
  "missing" indicator.

## Tag Editor Redesign — working spec (2026-09-03, approved; phases A+B+C built, D–E remain)

Agreed shape (owner + agent, one long design chat; every rule below is a
decision, not a default). Two SEPARATE modals with deliberately different
jobs — the album modal is a bulk surface (never a track list: a 300-track
compilation makes any list a scroll-punishment), the track modal is a
surgical one (plus a header stepper `◂ ▸` to walk the album's tracks: the
listless way to fix five files without closing the window).

### The promise (both modals)

**Fields you did not touch are never written.** A field the user did not
edit goes to no file, even if the modal "knows better". Corollaries: a
disputed field left alone stays disputed (no silent flattening); the write
set is files differing in ≥1 *touched* field; and the modal states its blast
radius before Save — footer reads `writes 4 of 18 files · cover`. Bulk tools
that hide their reach are scary tools. (This is the answer to MusicBee's
apply-checkboxes, achieved at file-diff granularity without the checkbox
clutter.)

### Album modal

- No track list, ever. One summary line: `18 files · 2 discs`.
- **Disputed fields resolve for real**: the `•` expands to chips of the
  competing values with counts (`Power Metal · 12   Progressive · 6`);
  click to adopt, or type. Rust must return the VALUES (today
  `get_album_tags` computes the disagreement and throws the values away):
  `conflicts: {field: [{value, count}]}`.
- Smart save: only touched fields, only differing files. Fewer mtime bumps,
  fewer re-parses. Year/number fields validate (4-digit, integers) — inline
  caution, not today's silent `junk → null → cleared`.

### Track modal

- All fields; shows the file's name and folder (you are editing a file;
  say which) with a reveal affordance.
- Album field: `datalist` of existing albums (same-album-artist first).
  When the edit would retag the track into another album while the FILE
  stays behind (non-pending tracks), a caution note says so — see Phase E.
- Split `TagEditor.svelte` → `AlbumTagsEditor.svelte` + `TrackTagsEditor.svelte`
  sharing one field-grid piece; the mode branches already fight in one file.

### Artwork — a first-class field, chosen not uploaded (MusicBee's model)

- Both modals carry an **artwork selector**: candidate tiles, deduplicated
  by hash — every distinct embedded picture with its census (`in 18 files`)
  and every folder-art file by name. The resolved cover wears the selection
  ring; clicking a tile adjudicates. Track modal candidates include the
  album's other files' pictures too ("pull the real cover onto this one
  file" is a compilation-fixing verb). New images arrive via kdialog picker,
  **Ctrl+V** (clipboard → base64 → IPC; WebKitGTK image-paste to be verified
  early, disk picker is the fallback), or drag-drop on the strip.
- **Selecting is an instruction**: album mode = every file ends up carrying
  exactly the chosen picture (files already equal are untouched — that IS
  "reflected on each track", stated honestly); the folder-art winner's bytes
  are replaced in place, or `cover.jpg` is created in the album dir (app
  wrote it ⇒ app owns it). A written file ends with EXACTLY the chosen
  picture — stray extra embedded images are normalized away, MusicBee-style.
  Unwritten files keep what they had. Track mode: same rule, one file.
- **Remove** = nothing selected: embedded cleared everywhere, folder-art
  winner DELETED (explicit instruction; the save receipt names it —
  `removed folder.jpg`), imports' honesty rule extended.
- Ripple: cover change forces re-extraction of THAT album's thumbs and panel
  colors (`process_album` already does both) — the album re-tints live.
  Needs: targeted `refresh_album(album_id)` (runs even when a cover exists;
  `refresh()` today only processes cover-less albums) and **hashed thumb
  URLs** (the `thumb://…/512.webp` URL has no cache-buster today).

### Phase E — "Add to existing album…" (pending tracks only)

Ground truth from `scan.rs`: album identity is `stable_id(artist, title,
year)` + an adopt pass matching (artist, norm-title) — so same album-artist
+ same album title ⇒ the track is APPENDED even from another folder.
Directory-authoritative only means a folder is never SPLIT, and folders are
where files live. Therefore the picker is a *retag*, not a mover: retag →
scan adopts the row → later Save moves the file home via `album_destination`
("the album's own folder" = the existing album's). Library (non-pending)
tracks get only the datalist retag — and it warns, because the file stays
behind in a split-folder album; the clean-merge promise holds only for
pending tracks, hence the gate. The door: searchable album picker (that
artist's albums first), shows *"will become track 12 of 14"* with the next
free number prefilled + editable, writes album/album_artist (and adopts the
target's year — cosmetic hygiene; year cannot split a row, adopt matches on
title alone).

### Phases

- **A — Rust**: conflicts-as-values; `ArtChange{Keep,Use(selector),Clear}` +
  picture-diff in the write-set math; `list_art_candidates`; folder-art
  write/delete; `refresh_album`; hashed thumb URLs; numeric validation.
  Pure functions, tested (`cargo test --lib`).
- **B — Album modal**: chips, summary + blast-radius line, artwork selector,
  validation, save receipts.
- **C — Track modal**: component split, stepper, per-file well with album
  candidates, datalist + stays-behind caution, filename/reveal.
- **D**: entrances/dismissal carry-over (scrim contract, vanish guards), copy.
- **E**: the pending-track picker door.

Status: **Phase A executed 2026-09-03** (cargo --lib 88/88, +3 tests:
conflicts-as-values w/ counts & majority order, diff-aware write set incl.
the honest zero-save, full artwork lifecycle upload→hash-adjudicate→
folder-JPEG replace→inventory→clear→refresh_one cover-NULL). Landed shapes:
`AlbumTags.conflicts`, `TouchedFields` (omitted = legacy all),
`ArtChange keep/clear/hash/upload`, `SaveReport {written,total,receipt}`
(receipts name the user's `folder.jpg` when replaced/removed),
`get_art_candidates(albumId, trackId?)` → census + `thumb://art-<hash8>/
256.webp` previews + `current`/`folderArt`/`trackPics`; cover URLs now
`…/512-<hash8>.webp` (handler parses size before the dash, legacy URLs
resolve); `artwork::refresh_one` re-extracts one album and NULLs the cover
when nothing is left. Uploads: base64, sniffed-mime (never trusted), 25 MB
cap, webp/gif/tiff embeddable (folder art always JPEG). One behavior the
tests pinned: the inventory is the album's CURRENT truth — an adjudicated
image replaces what it displaced, no history tiles.
**Phase B executed 2026-09-03** — the album modal, `ArtSelector` and the
lightbox went through three owner-feedback rounds; see the Phase B sections
below for what actually landed. **Phase C executed 2026-09-03** (the track
modal + the component split — see the Phase C section at the end of this log).
Phase D (entrances/copy carry-over) executed 2026-09-03 (0.9.2): the modal
family contract verified compliant (scrim-in/surface-in, `.out` outro with
the animationend backstop, `.waiting` re-triggering the entrance when the
fields arrive — no per-editor reinvention); vanish guards confirmed on both
doors (album: the closing effect; track: everything is derived from the
live store, so a regroup heals the window instead of breaking it); copy
pass: numeric errors now grammatical for one or many ("Year must be a
number" / "Year, Track # must be numbers" — one `badMsg` derived per
editor), and the 0.9.1 accelerators got their affordances in tooltips —
Save says "Write this file (Enter)" / "Write the changed files (Enter)" /
"Nothing to write", Cancel "Close without writing (Esc)", the stepper
"Next track (→)". Phase E (pending-track picker) remains — and executed
the same evening:

**Phase E executed 2026-09-03** (the redesign's last door, `TrackTagsEditor`
carries it alone): "Add to existing album…" renders ONLY for staged files
(`meta.staged`). It opens a fixed-position searchable popover (fixed, not
absolute — the body scrolls and clips, and a field-anchored popover must
not fight the clipper; no svelte:window either — the whole template lives
inside the children snippet and window tags may not sit in a block, so
click-outside runs from an $effect bound to the open state). Candidates:
the file's artist's albums first, then the rest by the grid's own order
(article-stripping sortKey, artist then title); the album the file already
points at is excluded; each row wears its CONSEQUENCE — "track N of M"
(next free = max trackNo + 1, gaps respected; total = files + 1). Applying
is a RETAG only — album, album artist, adopted target year (cosmetic
hygiene: the adopt pass matches on title alone), next free number
prefilled into the editable Track # — plus the promise line ("will join X
as track 12 of 12 — Save writes it; Import moves the file") and its
announcer sentence. No new Rust: scan.rs's adopt pass does the append on
the post-save rescan, and Import's album_destination does the move.
Escape belongs to the picker while it is open (innermost-first, the
lightbox's law). Live-probed against the owner's 12 real pending Ghost
files: open → apply "Land of Light" → fields 12/12/2017 → promise line +
announcement → Escape abandoned unsaved, staged count unchanged.

## Tag Editor Redesign — Phase B: the album modal executes the spec (2026-09-03)

- **`TagEditor.svelte` album mode rebuilt around the promise.** The census
  (`get_album_tags.conflicts` + `fileCount`) drives everything: a summary
  line (`13 files · 2 discs · folder.jpg ranks first`), dispute CHIPS under
  each field showing the competing values with counts (click = adopt, on
  press), and the footer's pre-save promise — `writes 13 of 13 files ·
  cover`, "up to" when several fields contribute (the union across fields
  is not exact without per-file rows; the receipt prints the exact truth).
- **`ArtSelector.svelte`** (shared, album-scoped — track mode hosts it in
  Phase C): candidate tiles with census labels, ring on the adjudicated
  choice, three arrival doors — kdialog (`pick_image`), Ctrl+V (clipboard →
  base64, magic-byte sniffed), and webview drag-drop (tauri's own drop
  events, so no HTML5 path dance) — plus `Remove artwork` with its promise
  line ("removed from every file and folder.jpg deleted"). Arrivals render
  as data-URL tiles; the `+` tile is the honest empty state.
- **The save switched from "stamp everything" to the real `touched` diff**
  (computed against the load snapshot) and the live `ArtChange`; on success
  the footer BECOMES the receipt: `Wrote 4 of 18 files` + the folder-art
  lines named, Done focused, rescan refreshing behind it. The track branch
  keeps its old exit (its blast radius is one file, stated by its own
  existence). Validation moved in front: junk numbers never reach Save —
  caution border + footer naming the offending fields.
- **Gotcha paid (see AGENTS.md):** the blast derived gated on a plain `let
  snapshot`, early-returned its first pass, and therefore never subscribed
  to `touched` — null forever. Gate variables of deriveds must be `$state`.
- Rust side of the phase: census now covers QUIET fields too (a single-value
  field reports its count; a disputed one reports each) plus `fileCount`, and
  `ArtCandidate.count` counts FILES only (`folder` names the file) with
  `filesWithArt` on the inventory — every clause of the blast line is
  computable client-side, and the tests pin it (88/88).
- Verified live (DOM probes, no save against his library): summary line,
  tiles render, edit genre → `writes 13 of 13 files`; gates: cargo 88/88 ·
  check 0/0 · vitest 84/84 · build ok.
- **Left in Phase B deliberately:** the receipt-after-save visual (probe-only
  session did not write to his collection — it is exercised the first time
  he saves), chips on a genuinely disputed album (his library is clean).

### Phase B, second look — two columns, no per-file numbering in album mode (2026-09-03, owner feedback)

- Album modal is now ONE row of two columns: artwork left (190px, candidates
  STACKED as real previews — one or two images earn to be seen, not queued
  in a strip), fields right on a single label+input rhythm. Panel 620→680px.
  Track mode keeps the compact double-column grid (it has no artwork column
  yet; Phase C decides its layout too).
- **Track/disc numbering left the album editor entirely.** The numbers belong
  to the files, and the album row inherits them; the hint line says so. The
  fields are unrendered → never touched → never written (the promise needs
  no special case); Rust keeps `track_total`/`disc_total` in the shared
  struct for callers that still mean them.
- Mechanism: a `{#snippet fields()}` rendered into either layout — one loop,
  two containers, no duplication. Verified by geometry probe (art x=279 w=190,
  grid x=487, `te-trackNo` absent, year present), check 0/0, build ok.

### Phase B, third look — the artwork interaction contract (2026-09-03, owner feedback)

- **Click an image → see the image.** Tiles expand into a lightbox (the
  candidate's 512px preview — `ArtCandidate.full`, written beside the 256 at
  first sight; originals stay in the files, not in a cache copy). The
  lightbox carries the artwork verdicts in context: `Use as cover` when the
  viewed one is not the cover, `Remove artwork` / `Keep artwork`.
- **Choosing moved to a corner dot** on each tile (iOS photo-picker language:
  the image opens, the dot decides) — keyboard reachable (role=checkbox,
  Enter/Space), answers on press with a scale dip.
- **Remove moved onto the image it dismisses**: a corner ✕ on the COVER tile
  only (hover-revealed, caution on hover), and the header's text button is
  gone. Removal of the chosen art is a property of the chosen art.
- Escape is layered honestly: lightbox owns the first press (the parent sets
  `artLb=false`; the selector's `$effect(!open → view=null)` is the single
  close path), the dialog owns the second.
- Summary line: the body gained a real top measure (18px) and 12px under —
  a caption of the whole editor must not sit flush on the header's rule.
- Probe-verified: expand → lightbox with the 512 URL, dot-select, Escape×1
  closes only the lightbox. check 0/0 · cargo 88/88 · build ok.

### Lightbox first-open sizing + Close weight (2026-09-03, owner feedback)

- **The small-first expansion was a fallback lie, not a size decision.** The
  thumb protocol's lenient chain (512→256→96) answered a 512 URL with 256
  bytes and an IMMUTABLE cache label — so the first open of a cache written
  before 512s existed served small, and only the next open (which writes the
  real 512) corrected it. Artwork previews (`art-<hash>`) are now served
  STRICTLY — the inventory advertises only sizes it writes, and a mismatch
  must be visible (404), never papered over with wrong bytes under a
  forever-cache header. Album covers keep the lenient chain (older albums
  may legitimately lack a size until their next refresh). His stale dirs
  already healed: every `art-*` cache holds both sizes now.
- **Close is the primary button in the lightbox** — more often than not the
  image stays. `Use as cover` steps down to an accented ghost (present,
  colored, one weight below); `Remove artwork` stays plain.
- cargo 88/88 · check 0/0 · build ok.
- Round two on the lightbox image: the mislabeled response lived in **WebKit's
  disk cache**, outliving restarts under `immutable`. The 512 URL is now
  content-addressed (`512-<hash8>.webp` — the handler ignores the suffix;
  physical files keep their plain names), so a lying cached response is
  unreachable by construction and `immutable` is finally honest. Verified:
  first expansion of the session, `naturalWidth = 512`. The lightbox Close
  takes a solid accent fill (over the scrim, the panel's translucent wash
  reads disabled — a primary must read primary everywhere).
- Round three was the truth: the disk and protocol were innocent (every
  `512.webp` verified 512×512) — the lightbox `img` had NO definite width
  (`max-width: 100%` against an auto-width figure is a circular percentage),
  so the first view — async decode, no intrinsic size yet — collapsed, and
  every later view laid out from the cache-served decode. An img sized by
  viewport math (`width: min(512px, 100vw-140px, 100vh-220px)`) is correct
  before a single byte decodes. Probe on a cold-cache session: first
  expansion renders at the full box. The general law: anything that fades
  in with async media needs a definite box, never an intrinsic one.
- Lightbox final form: 384px base (75% of before), the dim behind it at 0.70,
  and the card carries the MODAL's own radius token with `overflow: hidden`
  doing the clipping — an `img` may paint its replaced content over its own
  border-radius on WebKitGTK, so the clip belongs on the card, not the image.
  It is now a sibling card of the dialog: same radius, border, material.
  Geometry probe: w=384, radius token honored, overflow hidden, scrim 0.7.
- One-rhythm follow-up (owner): the buttons row sits 13px from the card's outer
  bottom border (12 padding + the border's own pixel), so the image→row `gap`
  went 12 → 13 — measured to the visible border, every side of the mat now
  spaces its content identically. Probe: `gapImgRow:13, gapRowBottom:13`.

## Tag Editor Redesign — Phase C: the track modal (2026-09-03)

- **The split.** `TagEditor.svelte` is a dispatcher now (trackId → track
  modal, albumId → album modal). The modal CONTRACT — backdrop, the
  `.scrim > .glass` pair, the `scrim-out` outro, Escape ownership with the
  lightbox veto, the focus trap, the `waiting` hold while fields load —
  lives in `TagSurface.svelte`, with the shared footer family in a `.te-*`
  namespaced global sheet (AGENTS.md: a class rendered across a file
  boundary is unscoped). Field rendering lives in `FieldGrid.svelte` —
  dispute chips, inline "of N" totals, the album datalist — driven by data,
  not by mode branches. The field model, census types and numeric
  validation moved to `lib/tagFields.ts`, the one file both editors agree
  on so the pair cannot drift.
- **The track modal** (`TrackTagsEditor.svelte`) is the surgical one:
  - *Identity row*: the file's name, its folder `~`-shortened (full path as
    the tooltip), and a reveal button — paths travel as DISPLAY strings
    only, revealing stays `reveal_container`'s job. Rust: `tags::track_file`
    + the `get_track_file` command (+1 test: name, album id, staged flag).
  - *Header stepper* `◂ n/N ▸` walks the album in display order
    (`library.tracksOf`, the panel's own order). It DISABLES while unsaved —
    it will not eat your edits silently — and a track save STAYS OPEN
    (footer reads `Written ✓`, the fields re-read from disk): that is what
    the walk is for, five fixes without the window ever closing.
  - *Retag door*: the Album field is a `datalist` of existing titles,
    same-album-artist matches first.
  - *Artwork is per-file*: the album's candidates, applied to this file
    (`ArtSelector` with `trackId` — the seam Phase A/B already carried).
- **The stays-behind caution**: retagging a LIBRARY (non-pending) file to a
  different album/album-artist moves the row, not the file — the modal says
  so in the caution hue and names the folder it stays in until an import.
  The clean-merge door (Phase E's picker) remains the pending-only path.
- **Same body as the album modal (owner ruling)**: the track modal wears the
  SAME two-column shape — `.te-cols` (artwork column left, stacked real
  previews, fields right in the single label+field rhythm). Only the APPLY
  target differs: this file, not the album. `.te-cols` moved into
  TagSurface's shared global sheet; `layout` on FieldGrid now selects ROWS
  only, the grid SHAPE is one rhythm for both.
- check 0/0 · vitest 84 · cargo 89/89 (+1) · build ok. Owner eyes the
  track-editor walk next (window re-centered by the cargo restart).
- Owner layout rulings, same day: (1) the track modal wears the SAME body
  as the album modal — `.te-cols` moved into TagSurface's shared sheet, and
  FieldGrid's `layout` selects ROWS only, the grid SHAPE is one rhythm;
  (2) small fields share rows — the grid is four shared tracks
  (label|field|label|field), one-field rows span the right side, and the
  pairs are Genre+Year (both modals) and Track#+Disc# (track modal);
  year/number inputs are fixed-width (`half`), "of N" totals capped at the
  same measure; (3) same day — the four shared tracks reserved a full column
  for a four-digit year, so pairs moved INSIDE the field cell (grid is back
  to label|field): genre stretches and pushes Year to the edge, the number
  pair packs left as one unit, and dispute chips render after the row's
  inputs so a wide chip never wraps the pair onto its own line; (4) the disc
  cell fills the row's remainder (split evenly, ends at the column edge);
  paired labels keep the SAME label→field distance the column labels have
  (grp label margin-right 4 → 10px total, probe: discGap/yearGap = 10);
  (5) **"More tag fields" fold on the track modal** (owner ruling): the
  rarely-touched Composer/Label/Grouping/Comment behind a disclosure row
  (the row is the button, chevron the indicator, instant swap — the
  ManageImports family), collapsed by default, and the header ACCOUNTS for
  the fold: "N fields filled" / "edited" — hidden state never silent.
  FieldGrid gained track-core/track-more layouts; the album modal keeps
  every field visible (the census promise — disputed fields may not sleep
  behind a fold).
  (6) SAME EVENING, critique-driven (impeccable critique of both modals,
  29→34/40; snapshots under .impeccable/critique/): the FIELD ORDER swapped
  to Album→Album artist; small-field geometry became MEASURED, not guessed
  (the grid observes the disc tail — label, box, "of", total — with one
  ResizeObserver, borderBoxSize only: getBoundingClientRect reads the modal's
  scale-in transform and pinned Year short; hidden "of" pairs claim no row
  room and must not be summed; every term exact layout px or the label sits
  0.12px adrift): the track modal's Year label stands under "Disc #" and its
  box under the disc box, the year box filling the mirrored cluster; the
  album modal's Genre|Year row is symmetric by the measured year-label width
  (--b = half of column minus label minus 16px separations; .te-grp needs
  border-box or the label margin inflates the set width). CORRIDOR: modal
  content is centered — body pays the 11px scrollbar gutter on the left too
  (overflow-y: scroll makes the gutter permanently REAL: WebKitGTK reserves
  ZERO with stable+auto while content fits), head/footer carry 13px margins.
  SELECTION dot filled with the KWin decoration green (--tb-a, live from
  decoration.svelte.ts; accent fallback for the pre-token frame). SHARED
  SURFACE additions in TagSurface: .te-topline (the promise line, first
  under the rule in BOTH modals — the bulk surface must never scroll its
  reassurance away), the .te-more fold family + slide transition (svelte/
  transition, prefersReducedMotion.current → instant, Svelte 5.9's is a
  reactive value, NOT a store, NOT callable), and the disclosure styles.
  ALBUM MODAL: orphan "N files" caption → FOOTER identity caption ("13 files
  associated to this album"; blast message takes the slot while dirty, same
  slot semantics as the track modal's path), and the SAME fold for
  Composer/Label/Grouping/Comment — with the census promise held: AUTO-OPENS
  when a hidden field is disputed, keyed on the DISPUTED FLAG, not on
  conflicts-membership (conflicts_for returns a census entry for EVERY
  field — membership test sprang the fold on every album; found by the
  second critique run's own probe). KEYBOARD (critique: Flexibility 2→3):
  quickKey prop on the surface — Enter=Save in both doors (form convention
  inside fields, native activation on buttons), ←/→ walk the track stepper
  outside text fields only; step() dismisses the artwork lightbox (probe:
  it lingered a caption behind a moved context). FOCUS: the surface captures
  its opener and restores focus on close (guarded: detached context-menu
  openers, body). ArtSelector hint tightened to "Click to enlarge · the dot
  marks the cover."
  (7) APP-WIDE AUDIT (impeccable, 16/20 Good; the two detector
  layout-transition warnings VERIFIED intentional — the .panel-slot margin
  rides the expansion's own reflow, documented in-component). The one real
  gap, fixed in 0.9.1: **status messages were silent** (WCAG 4.1.3) —
  src/lib/stores/announcer.svelte.ts + ONE visually-hidden
  role=status/aria-live=polite region in App.svelte, deliberately OUTSIDE
  the inert stage so a modal cannot mute it; fed by scan-finished (success
  AND failure), both tag editors' save paths (success and failure), the
  same sentences the visual layer shows; say() grows a space tail on equal
  consecutive messages because equal text = no node change = no
  announcement. ManageImports and the sidebar scan-note already own their
  polite regions — nothing double-announces. Verified by screenshotting the
  region with its sr-only temporarily stripped ("Library updated" — the
  watcher rescan after a file save, narrated). The two off-scale radii are
  on-scale (--radius-icon on the toggle box, --radius-pill on the scrollbar
  thumb — 5px wide, it was always a pill), and cover-overlay white is now
  var(--on-cover) (3 sites). Detector residue: only the two intentional
  warnings.

### Critique run — ExpandedPanel, 2026-09-04 (30/40 Good, first for this slug)

  The last unreviewed surface (sidebar/playbar/grid/import/tag-modals all had
  runs). One systemic finding, all four items fixed the same evening:
  **P1** the app's keyboard-focus family (accent outline — sidebar `.mrow`,
  import `.mi-disc`, tag inputs) stopped at this door: `.track`,
  `.staged-badge`, `.edit-album`, `.play-all` fell back to the UA's generic
  ring (observed live — two focus dialects on one screen); one four-selector
  rule joins the family, ring follows each control's own rounding.
  **P2** the Imported badge became a door in 0.7 but kept label geometry —
  padding-block 3→7px lands it at exactly 28.0px measured (PRODUCT.md bar),
  and per the cluster ruling it is now a QUIET sibling: transparent fill,
  text-dim, accent only on hover — Play is the header's only accent thing.
  **P2** missing-file alert `#f2a33c` → `var(--caution)` (token existed,
  value identical). **P2** state collision: `.track.selected` and `:active`
  shared the wash exactly, so press-on-selected showed nothing and a
  mid-press read as selection — selection is now wash + 1px inset `--border`
  ring (measured live: `inset 0 0 0 1px rgba(255,255,255,.1)`), the wash
  stays the press moment. Dev-only P3: header comment still promised the
  0.9.3-deleted arrival "flash". Detector: 0 findings. Snapshot:
  `.impeccable/critique/2026-09-04T00-05-34Z__src-components-expandedpanel-svelte.md`
  (scored BEFORE the fixes; the P1–P2s above are why it says 30).

### Playbar gradient crossfades, 2026-09-04

  The artwork backdrop teleported: one linear-gradient cannot interpolate
  into another (the old `transition: background 280ms` on .playbar could
  only ever lie). The gradient now lives in stacked `.pb-bg-l` opacity
  layers, one per gradient in flight: an album switch fades the NEW layer
  IN while the OLD fades OUT — both stacked fills, so the opacity
  crossfade IS the gradient morph (owner ruling after a first attempt that
  faded through chrome: reading it as off/on, and it was). Fade IN alone
  at first play / toggle-on, fade OUT alone at stop / toggle-off, old
  layers removed at the floor (300ms TTL, invisible-duplicate guard for
  skip-back-mid-crossfade, which also revives a layer whose gradient comes
  back). 700ms symmetric ease (cubic-bezier(0.4,0,0.2,1)) — the owner ruled the
  first cut (200ms) "much slower"; a slow dissolve is scenery, not
  feedback, and deliberately spends itself outside the 300ms state-response
  budget; a front-loaded curve would undo the slowness the duration buys.
  Reduced-motion killed by the stylesheet. State
  traps paid: mutate layers only through the $state proxy (raw-object
  writes notify nobody and the class never flips); `transition:fade`
  inside {#key} compiles with intro|outro flags and NEVER RUNS (verified
  live, cold reload) — CSS opacity cannot be skipped. Live traces: mount
  0.62→0.95→1; switch saw old 100→0 and new 0→100 SIMULTANEOUSLY with both
  gradients present in the DOM.

### find-animation-opportunities sweep + mirrored connective tissue, 2026-09-04

  The sweep's honest headline: most seams were already animated (panel
  contract, accordion, drawers, playbar dissolve, even the boot grid's
  .grid.enter row ladder — 320ms, 90ms/row, 540 cap, first 8 rows). What
  shipped is the leftover connective tissue, all of it MIRRORS (owner
  rule: every entrance owes its exit along the same path, same length):

  - ContextMenu: local transition, 125ms cubicOut, opacity + scale(0.97),
    transform-origin pinned by the clamp effect to the corner the click
    actually anchors (flip decides: left top / right bottom / …). Grows
    out of the click point, shrinks back into it.
  - EQ + queue popovers: the pop-in KEYFRAME becomes the popUp
    TRANSITION — 150ms cubicOut, opacity + 6px rise + 0.97 scale, origin
    bottom-right at the anchor button. Unfolds up from the button, folds
    back down. One dialect with the menu (same scale, same easing).
  - ExpandedPanel rows: transition:fade|local 160ms on the row BUTTON
    (not the <li> — the list is grid-auto-flow:column and an in-flow
    outro cell must not move; the fading button holds its slot, the gap
    closes as the deliberate second beat). Rows discarded/removed fade
    out in place; watcher-adopted rows fade in the same beat; `local:`
    keeps boot silent (the grid entrance owns it).
  - Lightbox: as-lb-in keyframe replaced by fade|local 140ms — entered
    on a fade, now LEAVES on one (it used to vanish).
  - Skeleton exit: the tiles rise in on library arrival; the shimmer now
    fades out over 200ms instead of blinking (wrapper div — transitions
    may not sit on component tags; grid-column: 1/-1 keeps its measure).

  Syntax tax paid: Svelte 5 modifiers are `fade|local`, not the old
  `local:fade`; JS-driven transitions read matchMedia/MediaQuery at fire
  time (the stylesheet kill switch cannot reach them). Live-verified:
  menu mid-flight opacity .71 / scale .991 / origin pinned to the click
  corner, outro in flight at +60ms; popover rising at .76, still mounted
  folding on close. Rejections logged with their gate: play/pause glyph,
  seek, stepper (100+/day), hold-to-confirm on discard/remove
  (recoverable, tens/day), tile hover, search re-stagger.

### The real-flow rehearsal happened, 2026-09-04

  The owner imported Ghost's Infestissumam (12 files) through the album
  badge modal with a real Save — the first end-to-end run of staging →
  saveImports → relink_track → album_destination against his library.
  Verified after: staged=0, import cache empty, all 12 files MOVED (birth
  mtimes intact) into `~/Music/Music Files/Ghost B.C./Infestissumam/`,
  folder created by the save itself, 12 rows re-pointed (play history
  kept), the following scan skipped 4464/added 0 — exactly what a clean
  relink looks like from the outside. Destination note: the files carry
  ALBUM ARTIST "Ghost B.C." (the 2011 release), so the layout rule put
  them under Ghost B.C., next door to the Ghost catalog — the split is
  the files' truth, not the app's. Every deferred verification item is
  now closed; nothing in the plan remains open.

### Settings persistence speaks when it fails, 2026-09-04

  Critique re-run's single P2, wired (owner: "who knows if someone else
  ends up using it"): pushSetting's `catch(() => {})` — the one place a
  failure stayed invisible FOREVER — now hands the announcer one sentence
  ("Could not save a setting — your change may not stick next launch."),
  and initSettings' catch says its own on Tauri only (browser dev keeps
  its silent localStorage fallback). No visual syntax for a failure mode
  nobody should see; just no more silent lying. Announcer has zero
  imports — no cycle. Re-runs persisted for all four surfaces (sidebar
  24→32, modals 34→36, panel 30→34, imports 30→35; audit ≈18/20, residue
  = the two intentional panel-slot warnings).

### The app icon becomes ours, 2026-09-04 — Version 0.9.7

  Shipped since day one: the TAURI DEFAULT LOGO. The revision ran the
  full dreaded loop — glyph families (monogram/note/wave), dioramas
  (mini-stage/sleeves), a generated-diva detour, potrace plans — and
  stalled on two diagnoses the owner named himself: too abstract (the S
  wore Tauri's bones) or too clunky (freehand beziers; the honest hand
  was mine). The reference he produced (a stencil mic with a face in the
  negative space) set the language: STENCIL DISCIPLINE — bold broken
  geometry where gaps are the message and the composition, not the
  curves, carries the mark. The keeper: dark stage (the app's actual
  material), user-accent glow, stencil mic tilted up-right with grille
  slits as true mask cutouts, voice arcs riding its axis into the light;
  reads clean at 32px. The hidden-profile cameo failed honestly (edge
  notches read as chips, not a face — the reference's face works because
  it IS the composition). Master: assets/app-icon.svg →
  `npx tauri icon assets/app-icon.svg` regenerates the full set;
  hicolor 128/256 refreshed, gtk-cache + sycoca rebuilt. The next
  revision starts from the SVG, not from archaeology.

#### Icon deployment footnotes (2026-09-04, dev-only)

- Dock/KRunner landed via: user-shadow `Songstress.desktop` with an
  absolute-path `Icon=` (KDE does NOT consult the user-dir hicolor
  overlay; system copies were stale RPM-owned art), plus a
  `NoDisplay=true` `com.yossi.songstress.desktop` alias (app-id match).
- **KWin's overview on 6.7 stayed generic even with `set_icon()` pushing
  the icon over the KDE extension** (undecorated GTK client quirk?).
  Accepted: dock + KRunner carry the mark; the overview is KWin's
  business. `window.set_icon` stays — it is the correct app-side push.
- `kbuildsycoca6 --nosignal` is GONE on Plasma 6.7 (rc=1, quiet) — the
  silent no-op that hid this whole hunt. Plain `kbuildsycoca6`.
