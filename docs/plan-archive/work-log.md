# Implementation log (build order) (plan archive)

Moved out of `PLAN.md` on 2026-10-24 so the plan file stays readable in one pass.
This is the record, not the state: current state lives in `PLAN.md`, and this
file is the evidence behind it. Sections are kept verbatim, in their original
order. The index of every section is `README.md` next to this file.

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

