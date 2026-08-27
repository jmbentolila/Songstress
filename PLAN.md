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
| Step 7b — packaging (RPM) | ✅ 2026-08-26 |

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

### Explicitly deferred

- Artwork editing in the tag editor (Step 1 decision).
- Crossfade option — the eventual escape hatch for the Phase 3 audible seam
  at Epicus Furor→Emerald Sword (source-file MP3 encoder-padding estimation;
  not fixable app-side).
- MPRIS-initiated volume changes do NOT write through to the settings DB
  (the frontend owns persistence and doesn't hear about them) — acceptable
  for now; revisit if it annoys.
- Visual pass on the import flow on the real library (user) — ⬜.
- Menu label ambiguity: "Add music folder…" (import staging, Step 2a) vs
  "Music folders…" (library roots, Step 7c) sit in the same Library menu
  and are easily confused — rename candidate for the UI revamp
  (noted 2026-08-26, logic-first per decisions).

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
- ⬜ Visual pass on the real library (user) — still open, see *Deferred*.

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

## Open threads / known issues

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

## Environment quick facts (for fresh sessions)

- Fedora 44, Plasma 6.7.4 Wayland, output 3840×2160 @ scale 1.6 (logical
  2400×1350; fractional — relevant to 0b). GPU: AMD Radeon 780M
  (radeonsi, Mesa 26.1.7); WebKitGTK composites via EGL/gbm.
- mpv 0.41 at /usr/bin/mpv. sudo requires password (user runs dnf lines).
- Dev unit: `systemctl --user restart songstress-dev`; logs `journalctl
  --user -u songstress-dev -f`. HMR rot → cold restart first.
- Raise window: WindowsRunner DBus Match + Run(matchId, "activate")
  (qdbus-qt6, `--literal` for Match output; matchId like `0_{uuid}`).
- playerctl NOT installed — verify MPRIS via busctl/gdbus/qdbus-qt6.
- grim NOT installed; screenshots via `spectacle -b -n -a -o <file>`
  (active window) / `-f` fullscreen; background captures for artifacts.
- No click-injection tooling (no ydotool/wtype) — UI verification is
  menu-driven over DBus + screenshots + unit tests.
