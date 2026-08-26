# PROGRESS.md — Songstress implementation tracker

Last updated: 2026-08-25 (Step 7d live library watching LANDED — inotify via
notify, 2s debounce, dirty-flag scan loop with 5s min interval, verified
live. Earlier same day: Step 7a play-next queue (Rust-side user queue, pure
tested eof choreography, context menus + playbar queue popover, MPRIS
Next/Previous respect the queue), Step 6 equalizer, Step 5 playback
controls, Step 4 accent picker, Step 3 menu+identity, titlebar search,
scanner grouping fix, scan-killer bug. Earlier still: Step 2b, 2a, 1, MPRIS)
Companion to AGENTS.md (conventions & gotchas). Update the status table and the
"Next steps" section whenever work lands.

**NEXT UP: see PLAN.md** — the working roadmap (Step 0 bug fixes, then tag
editor → import/gradient → menus/global-menu → accent → EQ → shuffle/repeat).
It is interruption-resilient: every decision is recorded there; check items
off as they land and move completed steps into this file's status/notes.

## Spec summary (from the brief)

- Fedora 44 KDE/Wayland, album-grid player modeled on MusicBee browsing.
- Sidebar = flat artist list, click filters grid ("All Artists" state). Sorting:
  artists A→Z, albums by year ascending.
- Grid = rows of N tiles OR one full-width expanded tracklist panel. Row-model,
  never expand "inside a tile".
- Playback state is `{ albumId, trackIndex }` — no queue. Click track → play from
  there through the album; another album replaces context entirely. Gapless via
  mpv append-play + `--gapless-audio=yes`. ReplayGain: album mode default
  (configurable later).
- Tile size + sidebar row size sliders drive CSS custom properties.
- Thumbnails at ~3 sizes, never downscale full-res at render time (Phase 2).
- Glassmorphism, light/dark; `backdrop-filter` on chrome surfaces only; KWin
  blur-behind needs Better Blur DX (Wayland clients can't request it).
- `decorations: false`, custom macOS traffic lights.
- Build order: UI on fake data → scanner → mpv IPC → MPRIS (early).

## Status

| Phase | Status |
|---|---|
| 0 — env + scaffold | ✅ done |
| 1 — UI on fake data | ✅ done (several polish iterations) |
| 2 — library scanner | ✅ done (real scan verified 2026-08-22) |
| 3 — MPV JSON IPC playback | ✅ done (2026-08-22) |
| 4 — MPRIS | ✅ done (2026-08-23) |

### Phase 0 — done
- rustup stable (1.98), nodejs+npm via dnf, Tauri Linux deps installed by user.
- Scaffolded from create-tauri-app svelte-ts template, **then stripped SvelteKit →
  plain Vite+Svelte** (no router needed). vitest added. svelte-check wired.
- Window: 1280×800 min 960×600, `decorations:false`, `transparent:true`,
  client-side 14px border-radius. App-id `com.yossi.songstress`, binary
  `songstress`, project `~/Projects/Songstress`.
- Capabilities: window minimize/toggle-maximize/close/start-dragging/is-maximized.

### Phase 1 — done
- Row-model grid: `buildRows(albums, cols, expandedId)` pure fn + tests; column
  count mirrors CSS auto-fill math (`columnCount`); rows keyed stably
  (`r-<firstAlbumId>` / constant `"expanded"` so the expander survives relocation).
- ExpandedPanel: mount-closed + double-rAF flip (first-open animates);
  open/close animates `grid-template-rows 0fr→1fr`; switch = instant content swap +
  160 ms fade-in of new content, single-step height change (no ghost of previous
  album, no per-frame relayout). Artwork-gradient background (see below). Header:
  title + circular play-all button (plays track index 0) top-right; artist · year
  under it; meta footer bottom-right ("N tracks · total"). Tracklist ≥10 tracks →
  2 CSS columns; multi-disc albums get uppercase "Disc n" dividers + per-disc
  numbering. Track click → `playTrack(albumId, flatIndex)`; current row shows ▶/❚❚.
- Sidebar: All Artists pinned + A→Z with counts, quick-filter box, gear popover =
  tile-size slider (120–320), sidebar-row slider (28–52), theme toggle. Sidebar ends
  above playbar (`bottom: var(--playbar-h)`) so popover can't be overlapped.
- Overlay layout: `.stage::before` paints grid backdrop (own alpha), content scrolls
  UNDER sidebar/playbar so their backdrop-filter frosts scrolling art.
- Alphas (user-tuned): grid 0.8 / chrome 0.7 / panel gradient 0.36 dark · 0.30 light.
- Themes: dark/light tokens in app.css; theme toggle persists; noise garnish on
  .glass only.
- TitleBar: tb-* namespaced classes, traffic lights, drag regions, dblclick
  maximize handled natively by Tauri drag-region.
- PlayBar: fake playback engine (ticking position, advance-through-album, skip,
  seek, volume persisted).
- Fake library: 14 albums / 9 artists incl. **compilation** (Anison no Kokoro, 34
  tracks, Various Artists) and **multi-disc** (Keepers Live, 22 tracks / 2 discs).
  Covers are real folder.jpg files copied into public/covers/.
- Blur integration: Better Blur DX via COPR, kwinrc preconfigured (see README.md +
  AGENTS.md gotchas). Verified working visually.

### Gradient algorithm (src-tauri/src/lib.rs `colors::`)
- Decode cover (image crate) → thumbnail 48×48 → weighted buckets (weight = 4 +
  chroma²/16, ALL pixels count so primary = whole-artwork average).
- Primary c1 = weight-average color. Buckets merged greedily into clusters
  (manhattan < 64). Accent c2 = cluster with max distance from c1 among clusters
  ≥25% of heaviest weight; fallback = lighten(c1).
- Frontend mixes each 30% toward theme surface + lifts (+26 floor, ×0.85),
  alpha per theme, 135° two-stop gradient, cached per cover (successes only).
- Tests: every-cover no-overflow + `anison_accent_is_the_blue_hair`.

## Next steps (in order)

### Phase 2 — done (real library verified 2026-08-22)
- **Fix round 2** (2026-08-22, after the user's real scan exposed grouping
  bugs):
  - **Grouping rewritten** (`scan.rs`): directory-authoritative — same folder
    ⇒ same album. Tracks bucket per (parent dir, norm title); artist/title/
    year are per-bucket CONSENSUS values; buckets merge across dirs
    (multi-disc) when titles match and release artists agree. albumartist is
    TRUSTED: differing track artists are guests, not Various Artists; VA only
    for albumartist-less folders with mixed artists. Year left the identity
    key entirely. 5 new staged-fixture tests (retagged copies via lofty
    `insert_text`/`remove_key` + `WriteOptions`). Contract updated in
    PHASE2.md §5.
  - **Case-insensitive folder art** (`artwork.rs folder_art`): one read_dir →
    lowercase map, FOLDER_ART priority preserved; `Cover.JPG` etc. now match.
    Unit + refresh tests.
  - **Dev-build dep optimization** (`[profile.dev.package."*"] opt-level=2`):
    our crate stays opt-0. Test suite 6.6s → 0.33s; full scan of 4405 files
    7.5s.
  - **Rayon-parallel `artwork::refresh`**: serial prep (all DB access) →
    parallel decode/thumbs/colors (atomic counter, scoped poller thread emits
    progress) → serial UPDATEs. Real-library artwork phase: minutes → 5.1s.
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
  - **KWin blur outage**: better_blur_dx wasn't loaded one session; manual
    `loadEffect` fixed it; removed the stray `[Effect-better_blur_dx]`
    underscores group from kwinrc (the AGENTS-documented footgun; backup at
    `~/.config/kwinrc.bak-blurfix`).
  - **Blur outage RECURRED 2026-08-23** after reboot (same silent failure —
    kwinrc correct, effect simply not loaded, no journal error). Fixed for
    good this time: `kwin-blur-load.service` systemd user unit
    (`~/.config/systemd/user/`, script `~/.local/bin/kwin-blur-load.sh`),
    ordered `After=plasma-kwin_wayland.service`, WantedBy
    `graphical-session.target`; retries `loadEffect` over DBus up to 2 min.
    Enabled + tested (unload → start unit → loaded). This retires the
    "KWin Force-Blur script automation" deferred item.
  - **One-time DB reset** to apply the new grouping (incremental scan skips
    unchanged files by mtime+size, so old split rows survived a normal
    rescan): old DB kept as `songstress.db.bak-splitbug`; musicDir manually
    re-inserted (it is NOT localStorage-mirrored). Verified: 4405 added,
    ZERO duplicate album-title rows remain. Remaining odd albums are source
    tag issues — tag editor planned (see deferred).
- **Phase 2 M4 done** (frontend live): live-by-default in Tauri (`VITE_LIB=fake`
  opts out; browser always fake), class-based library store hydrated by new
  `get_library` dump command + refreshed on `scan-finished`; panel gradient
  reads scan-computed colorC1/C2 directly; settings hydrate from SQLite
  (init_settings seeding → get_settings wins, debounced write-through);
  EmptyState with folder picker and scan progress;
  title-bar hamburger menu = Rescan / Choose music folder… / About.
  Folder picker = native KDE `kdialog --getexistingdirectory` via our own
  `choose_music_folder` command (starts at saved musicDir → ~/Music → $HOME;
  cancel returns null) — tauri-plugin-dialog removed: its GTK chooser looked
  GNOME-ish on this Plasma-only personal app.
- **Phase 2 M3 done** (artwork): `library/artwork.rs` post-pass — folder art
  > largest embedded picture, one decode → 96/256/512 WebP thumbs in
  `app_cache_dir()/thumbs/<id>/` + colors stored in albums rows;
  `colors::extract_image` refactor; `thumb://` protocol with size fallback +
  traversal guard; wired into `scan_library`. Albums without art stay NULL
  (placeholder). 14/14 cargo tests. ⚠ thumb:// serving gets visual check in M4.
- **Phase 2 M2 done** (scanner): `library/scan.rs` — walkdir+lofty, grouping
  (albumartist→artist→Unknown; mixed-artist albums → Various Artists),
  sort_name = Rust port of sort.ts, stable blake3 IDs, incremental via
  mtime_ns+size, deletion + orphan sweep. `scan_library` command (own
  connection, concurrency-guarded) emits `scan-progress`/`scan-finished`.
  Fixtures committed under `src-tauri/fixtures/library/` (multi-disc,
  compilation, diacritics, The-prefix, 7 formats). 13/13 cargo tests, zero
  warnings. No UI entry point yet — arrives with M4.
- **Phase 2 M1 done** (`PHASE2.md` §11 questions resolved: `~/Music` root,
  folder art > embedded, mp3/flac/m4a/aiff/ogg/opus/wav, blake3 IDs):
  `src-tauri/src/library/{mod,db,settings}.rs` — rusqlite (bundled) + WAL,
  atomic migration framework (v1 = full schema incl. color/thumb columns),
  settings get/set/init commands with INSERT-OR-IGNORE localStorage seeding,
  blake3 `stable_id()` helper. DB verified created+migrated on launch at
  `app_data_dir/songstress.db`. 10/10 cargo tests. Next: M2 scanner.
- **Decoration-aware titlebar**: new `kde_window_decoration` command parses
  `kwinrc` `[org.kde.kdecoration2]` (button layout) + `klassy/klassyrc`
  `[ButtonColors]` (palette/opacity) with hand-rolled INI parsing (no new
  crates) and serde_json for Klassy's override blobs; falls back to built-in
  defaults when absent. Frontend `stores/decoration.svelte.ts` + TitleBar
  render layout from `ButtonsOnLeft` filtered to CSD-honorable letters
  (X/I/A; Shade etc. skipped) and colors from the parsed palette. Verified
  pixel-exact vs the user's Klassy config (`#fd5256/#fbc006/#58fb3f` @85%).
  This is the "read the user's system config" seam — reusable pattern.
- Tracks now play on **double-click**, not single click (ExpandedPanel
  `.track` buttons use `ondblclick`; single click = selection highlight,
  resets on album switch). Titlebar glyphs are white (Klassy titlebar-text
  style) and appear only on the hovered button, not the whole group.

- Settings storage: `settings` table in SQLite (music dir, ui prefs, replaygain,
  volume) — migrate localStorage keys over.
- Scan pipeline: walkdir over configured root (default `~/Music/Music Files`) →
  lofty tags (mp3/flac/m4a/aiff; wav sparse-tags acceptable) → group by
  albumartist→artist→"Various Artists"; albums keyed (albumartist,title,year);
  tracks ordered (disc,track); sort_name computed at insert (reuse src/lib/sort.ts
  logic ported to Rust).
- Schema sketch:
  ```sql
  artists(id TEXT PK, name TEXT, sort_name TEXT)
  albums(id TEXT PK, artist_id TEXT, title TEXT, year INT NULL,
         UNIQUE(artist_id, title, year))
  tracks(id TEXT PK, album_id TEXT, disc INT, track INT, title TEXT,
         duration_sec REAL, path TEXT UNIQUE)
  meta/settings(key TEXT PK, value TEXT)
  ```
- Thumbnails: reuse `colors::extract`; add thumb generation (image crate) at
  ~96/256/512 WebP into app cache dir; serve via custom `thumb://` protocol.
  Store panel colors in DB too (kills per-open extraction).
- Progress events to frontend (Tauri event `scan-progress`); rescan command;
  empty-state screen before first scan.
- Frontend swap: replace fakeLibrary behind a flag; keep fake mode for UI dev.

### Phase 3 — done (2026-08-22)
- **Engine** (`src-tauri/src/mpv.rs`): one mpv spawned at setup
  (`--no-config --idle=yes --no-video --gapless-audio=yes --replaygain=album
  --input-ipc-server=$XDG_RUNTIME_DIR/songstress/mpv.sock`), tokio UnixStream,
  single writer task, reader task routes responses by `request_id` (global
  registry — NOT thread-local) and dispatches events. Properties are OBSERVED
  explicitly (`time-pos`/`duration`/`pause`) — mpv pushes nothing unobserved.
  Position throttled to ~10 Hz. Orphaned engines from previous sessions are
  reaped via a pidfile + /proc cmdline guard (`idle=yes` otherwise stacks
  silent mpvs forever).
- **Rust owns `{albumId, trackIndex}`**: `play_album` = playlist-clear →
  loadfile replace → append tail (native gapless). NOTE: `playlist-clear`
  takes NO argument (passing "current" errors out — cost one debugging round).
  `jump()` rebuilds the queue (played entries are consumed; a plain replace
  desyncs the tail) and WRAPS at album edges (prev on first track → last,
  next past end → first). eof→advance, error→skip, end of album→stop.
- **IPC values are TYPED** (`command(json!(...))`): mpv rejects stringified
  booleans ("unsupported format") and observe ids must be JSON ints
  ("invalid parameter") — both silently killed UI updates until caught by
  driving the socket directly.
- **Commands**: play_album / playback_toggle / playback_pause / playback_jump /
  playback_seek / playback_volume / playback_stop; managed `Engine` handle;
  spawn failure degrades to a detached engine whose commands error cleanly.
- **Frontend** (`playback.svelte.ts`): dual-engine behind `library.live` —
  live mode mirrors Rust state via events (`playback-changed/-paused/-
  position/-stopped`); fake timer engine kept for browser/fake dev. Same API,
  components untouched.
- Verified live: real audio through the user's library; UI tracks position/
  pause/advance. PlayBar: prev/next always enabled (wrap-around); volume
  speaker icon = mute/unmute. All Artists grid orders albums by artist
  sort_name then year (artist view stays year-ascending). One audible seam at
  Epicus Furor→Emerald Sword is source-file MP3 encoder-padding estimation
  (both files same codec/rate) — not fixable app-side; crossfade option is
  the eventual escape hatch.

### Phase 4 — done (2026-08-23)
- **Server** (`src-tauri/src/mpris.rs`, zbus 5 tokio feature): serves
  `org.mpris.MediaPlayer2` + `.Player` at `/org/mpris/MediaPlayer2` as
  `org.mpris.MediaPlayer2.songstress`, backed by the SAME `PlayState` the
  frontend mirrors — UI and external controllers (playerctl, KDE widget) can
  never disagree. Server failure (busy name, no bus) is logged and swallowed;
  playback never depends on it.
- **Metadata** (9 fields): trackid/url/title/length/album/artist/albumArtist/
  contentCreated/artUrl, built per-read from the library DB (own connection —
  microseconds, never the shared AppState mutex). Two gotchas cost a debug
  round: D-Bus object paths only allow `[A-Za-z0-9_]` (blake3 ids carry
  hyphens → fold to `_` in `track_path()`), and `thumb_path()` already
  appends `thumbs` (pass the cache ROOT, not `<cache>/thumbs`, or artUrl's
  `exists()` silently fails).
- **Position/Volume**: `POS_BITS`/`VOL_BITS` statics in mpv.rs. Position is
  stored on EVERY mpv time-pos tick (the ~10 Hz throttle is frontend-emit
  only) and polled via the Position property — spec forbids pushing Position
  through PropertiesChanged. Volume is now an OBSERVED mpv property (observe
  id 4) so slider/MPRIS/mpv-internal changes all stay in sync.
- **Property changes**: a 5 Hz watcher task diffs a PlayState snapshot and
  emits PropertiesChanged only for what changed (status, metadata, Can*
  flags, volume). No notification hooks threaded through mpv.rs mutations.
- **Seeked**: emitted from `seek_to` (covers Seek AND SetPosition, as the
  spec wants); SetPosition validates the trackid against the CURRENT track
  and ignores stale ones. Seek/SetPosition clamp to [0, duration].
- **Verified live over D-Bus**: full introspection surface, metadata for real
  albums, Position polling, Pause/Play, Next/Previous (album wrap), Seek
  +30s with Seeked(int64) signal, SetPosition, Volume get/set +
  PropertiesChanged (metadata/status/can-*/volume all observed firing).
- Note: MPRIS-initiated volume changes do NOT write through to the settings
  DB (the frontend owns persistence and doesn't hear about them) — saved
  volume only updates from the in-app slider. Acceptable for now; revisit if
  it annoys.

### Step 3 — Menu bar + Plasma Global Menu — COMPLETE (2026-08-24)- **Menu model** (`src-tauri/src/menu.rs`): Rust-owned source of truth —
  Playback/Library/View/Help; dynamic bits (playing/hasTrack/scanning/anyStaged/
  themeDark) pushed by an App.svelte $effect via `set_menu_state`, which
  rebuilds + emits `menu-changed`. `menu_activate` = one activation path:
  playback ids handled natively by the engine, rest re-emitted as
  `menu-action` to the frontend. 3 cargo tests.
- **Titlebar menu bar** (`TitleBar.svelte` + `stores/menu.svelte.ts`): text
  menus after the traffic lights; hover-switch-while-open, ✓ checkmarks,
  disabled gating, Esc/focusout close. User-verified. The
  menubar HIDES once the Global Menu registers (poll `appmenu_active` +
  `appmenu-registered` event, either order wins); hamburger REMOVED (user
  call). Gotcha:
  TitleBar has TWO `{#if library.live}` blocks — the menubar nav and the
  hamburger popover; the first hide-edit landed in the wrong one.
- **Plasma Global Menu** (`menu_dbus.rs` + `wayland_appmenu.rs`): serves
  `com.canonical.dbusmenu` at `/org/songstress/Menu` as `com.yossi.songstress`
  (zbus); registers with KWin via `org_kde_kwin_appmenu` Wayland protocol —
  foreign wl_display/wl_surface adopted from raw-window-handle in guest mode
  (`Backend::from_foreign_display` + `ObjectId::from_ptr`), parked thread
  keeps the binding alive. Panel widget shows Playback/Library/View/Help
  (+ the applet's Search entry) — user-verified.
  - **Gotchas that cost the debugging marathon** (full detail in PLAN.md):
    KWin links appmenu→window only at create time and only for MAPPED windows
    (→ create/release/retry until a consumer queries us); the layout wire
    format must be `(i,a{sv},av)` with variant-wrapped children and the reply
    `(u(ia{sv}av))` — NOT `(uv)` — or KDE's DBusMenuImporter yields an empty
    menu and hides the widget; GetLayout depth 1 must include one child level;
    AboutToShow must answer "stale" per-subtree (revision vs last fetch) or the
    importer never refreshes submenu items after first import (grayed Playback).
  - The in-titlebar menu bar is HIDDEN FROM THE START (globalMenuActive
    defaults true — no startup flash) and only APPEARS if registration
    definitively fails (`appmenu_state` 2, polled at 1s; `appmenu-registered`
    event on success); hamburger REMOVED entirely (user call) — Global Menu
    or titlebar bar, never both.
- Gates: check 0/0, vitest 22, cargo 38, build OK.
- **Window identity**: user-level desktop entry (`StartupWMClass=songstress`,
  validated) + hicolor 128/256 icons; panel renders icon + "Songstress" +
  menus; title static. User verified Global Menu activation end-to-end.
- **Titlebar albums/songs search** (post-Step-3 feature): right-end field in
  TitleBar (after the drag spacer — flex:1 on the spacer beats auto margins,
  so the field must come AFTER it in DOM). `ui.mediaFilter` (session-only).
  Results split into two labeled sections: **Songs** (albums containing
  matching tracks; expanding one shows ONLY the matching songs via
  ExpandedPanel's `visibleTrackIds` — playback indices stay anchored to the
  full track list) and **Albums** (title/artist match). An album can appear
  in both; each section expands INDEPENDENTLY — `ui.expandedAlbum.{songs,
  albums}` (Songs panel = matching tracks only, Albums panel = full album,
  two panels can be open simultaneously).
  Matchers in `src/lib/search.ts` (`fold`, `albumTitleMatches`,
  `albumTrackMatches`, `albumMatches`; vitest-covered, diacritic-insensitive).
  Esc or × clears; "No albums or songs match" empty state. Gotcha:
  `.tb-balance` flex:1 absorbs free space before auto margins get any —
  don't rely on margin-left:auto in the titlebar.
  - Search scope fix: the Albums section matches ALBUM TITLES ONLY (artist
    names excluded — "hello" must not surface Helloween; artists have the
    sidebar search). Songs section matches track titles only.

### Step 7d — Live library watching — done (2026-08-25)
- **`src-tauri/src/watcher.rs`**: notify v7 (inotify) recommended_watcher on
  a named thread; ANY event kind counts; quiet-period debounce — fires only
  after 2s with no events (recv_timeout loop keeps extending while a burst
  pours in). A failing inotify backend (network mounts) logs and disables
  watching — manual Rescan stays the path there. Unit test: 3 quick writes
  coalesce to ONE callback (real FS events).
- **lib.rs wiring**: the callback only sets `WATCH_DIRTY`; a 500ms polling
  async task scans when dirty AND `SCAN_RUNNING` clear AND ≥5s since the
  last watcher scan (`WATCH_MIN_INTERVAL`). Events during a scan keep the
  flag set → exactly one follow-up scan after it settles. Scans are
  incremental + silent (existing scan-progress/scan-finished drive the UI).
- Watched root = music root at launch; a folder change resumes watching
  after the next app start (documented in PLAN.md).
- Verified live: touch → rescan in ~2s (61ms scan, 4416 skipped, 0 changed).
  Baloo may re-touch xattrs after a mtime change → one extra spaced rescan
  (min-interval handles it; each costs ~60ms). Gates: cargo 57, check 0/0,
  vitest 53, build OK.
- **Step 7 order**: 7a ✅ → 7d ✅ → 7c multi-roots (next) → 7b RPM
  packaging (HOLD: check with user before starting).

### Step 7a — "Play next" queue — done (2026-08-25)
- **Design**: queue lives in `PlayState.queue: VecDeque<OrderItem>` + a
  `from_queue` flag — while a queued entry plays it is PROMOTED into
  `order` at the current slot (so `playback-changed`/MPRIS metadata stay
  correct) and consumed from `order` at its own eof; `queue` only holds
  not-yet-played entries. Session-only; never shuffled; album clicks and
  stop clear it.
- **Pure eof choreography** (mpv.rs `eof_advance`): the end-file handler
  decision was extracted out of the reader task into a pure fn returning an
  `EofAction` — 8 unit tests cover promote/consume/chain/window-top-up/
  pre-arm/wrapped-promotion. Playlist invariant:
  `[current] ++ queue ++ order[index+1..index+31] (++ wrap pass)`.
  Promotion does NO window top-up (mpv consumed a queue entry, the order
  window didn't slide — appending would DUPLICATE the last entry); the
  eof test for the 40-track case pins this. Edge the tests caught: a queue
  entry as the LAST order entry leaves the index out of bounds on
  consumption → promote the pre-armed wrap pass or stop.
- **Rebuild path**: `load_queue` appends queue entries BETWEEN the current
  track and the order window and re-appends an armed wrap pass (single
  append path — play_order now arms BEFORE load_queue instead of firing
  duplicates after). Enqueue/remove/jump-to-queue all go through this
  rebuild; eof promote/consume stay surgical (0–1 appends, fire-and-forget).
- **Jump semantics**: Next consumes the current queue entry (if any) and
  promotes the front; falls into the order when drained; Previous from a
  queue entry returns to the order entry before it. MPRIS Next/Previous
  respect the queue for free (they call `jump`).
- **Commands/UI**: `playback_queue {trackIds, front}` / `_remove` /
  `_jump` (+ `tracks_by_ids`); context menus (track: Play next / Add to
  queue; album header: Play album next); PlayBar queue button (count badge)
  → glass popover with queued rows (click = play now, discarding earlier
  entries; × = remove) + dimmed "UP NEXT" preview; `queue-changed` event
  {queue, upNext} mirrors it. Enqueue while STOPPED just stores (never
  auto-plays; shown in the popover).
- Gates: check 0/0, vitest 53, cargo 56, build OK; live-verified (popover
  rendered real queue-changed data mid-playback). Not hand-verified (no
  click injection): enqueue/remove/jump clicks — logic is unit-tested.
- **Step 7 order** (PLAN.md): 7a ✅ → 7d live watching → 7c multi-roots →
  7b RPM packaging (HOLD: check with user before starting).

### Step 6 — Equalizer — done (2026-08-25)
- **Backend** (`src-tauri/src/eq.rs`): pure `af_chain(&Eq) -> Option<String>`
  (disabled or all-flat → None → clears mpv's `af`; preamp `volume=<n>dB`
  only when significant, then 10 octave-width peaking `equalizer` filters,
  ±12 dB clamp, 0.1 dB significance threshold, `{:.1}` formatting — mpv
  accepts "4.0"). `Eq { enabled, preamp_db, gains[10], preset }`, serde
  camelCase. 7 unit tests.
- **Command** `playback_eq(state, engine, eq)`: clamps → pushes chain to mpv
  (`Mpv::set_af`, "" = cleared) → persists the clamped state in settings key
  `"equalizer"` → returns the normalized state.
- **Launch re-apply**: Rust setup reads the settings key and re-applies to
  mpv (after the shuffle/repeat restore block); the frontend ALSO hydrates
  the UI via new `initEq()` in playback.svelte.ts (DB wins over the
  localStorage mirror, same rule as ui.* settings) — without it the popover
  showed Flat while mpv had the chain (caught by seeded-value screenshot).
- **UI**: PlayBar EQ button (sliders glyph, left of shuffle) → glass popover
  `.eq-pop`: on/off toggle, preset `<select>` (divergence → "Custom"),
  preamp column + divider + 10 vertical band columns (dB readout above,
  freq label below, double-click zeroes). Vertical sliders = horizontal
  ranges rotated -90° in a fixed 26×120 `.slot` — **WebKitGTK 2.52 ignores
  writing-mode/direction on range inputs** (thumbs render but never move;
  cost one debugging round).
- **Menu**: the menu model has no submenus → three flat items appended to
  Playback: "Equalizer: On/Off" (checkable), "EQ Preset: <name>" (cycling,
  gated on enabled), "Customize Equalizer…" (opens the popover via
  `ui.eqOpen`, session-only flag). MenuState gained `eq_enabled`/`eq_preset`;
  App.svelte $effect tracks both. `eq_items_follow_state` cargo test.
- **Presets** (`src/lib/eq.ts`, 10): Flat, Rock, Pop, Jazz, Classical,
  Electronic, Hip-Hop, Bass Boost, Treble Boost, Vocal Boost; `matchingPreset`
  → "Custom" on divergence. 6 vitest cases. Preset recheck (user request):
  Rock was Winamp's aggressive scooped-mids curve (−8 @250, +11 top) →
  gentler modern V [5,4,2.5,0,-2,-1,1.5,3.5,5,5.5]; Pop de-honked
  (mid-hump → gentle smile); Hip-Hop's boxy +3 @250 removed. Jazz/Classical/
  Electronic kept (Winamp shapes, still standard).
- Verified end-to-end: seeded non-flat EQ → restart → mpv reports exactly
  `volume=2.0dB,equalizer=f=31:...g=6.0,...f=16000:...g=-4.0`; popover
  renders hydrated values with thumbs off-center; reset to disabled/flat →
  `af` empty again. Gates: check 0/0, vitest 53, cargo 48, build OK.
- Not hand-verified (no click-injection tools): slider dragging, preset
  select, menu item clicks — logic covered by tests + screenshots.

### Step 5 — Playback controls package — done (2026-08-25)
- **Play-order engine** (mpv.rs): `PlayState.order: Vec<OrderItem{trackId,
  path, albumId, albumIndex}>` replaces the parallel track/path arrays; the
  `{albumId, trackIndex, trackId}` event contract is preserved (albumIndex =
  position within the track's OWN album, so UI mapping survives pools that
  span albums). Order built in lib.rs per shuffle stage: off = album order;
  album/artist/all = clicked track first + Fisher-Yates shuffle (tiny xorshift
  RNG, no rand dep) over the album / the artist's albums / the whole library.
- **Repeat**: track = native mpv `loop-file=inf` (cleared on stage change);
  album = eof at last order entry restarts at 0, RESHUFFLED when shuffle is
  active; off = stop (stages survive stop — they're modes).
- **Mid-playback stage change**: `playback_set_shuffle` rebuilds the queue
  anchored at the current track WITHOUT restarting it (playlist-clear keeps
  the playing file, tail re-appended; repeat-off returns to plain album
  order positioned at the current track).
- **Persistence**: stages in settings (`shuffleStage`/`repeatStage`) +
  localStorage mirror; hydrated at launch in setup (incl. re-arming
  loop-file). MPRIS `Shuffle`/`LoopStatus` are now REAL and settable
  (Shuffle true → album stage if off; LoopStatus Track/Playlist map to the
  repeat stages) with property-changed signals from the 200ms poll.
- **UI**: PlayBar — album prev/next buttons flanking the transport
  (⏮album/album⏭, global grid order, wrap-around, no-op stopped, frontend
  computed via `lib/albumOrder.ts` + tests); shuffle (4-stage, badge dot)
  and repeat (3-stage, "1" overlay) buttons left of the volume; Playback
  menu gained Shuffle:/Repeat: cycling checkable items + Previous/Next
  album items (frontend-handled via menu-action).
- Gotchas: std MutexGuard is not Send — the mpv eof handler decides inside a
  scope and acts AFTER the guard ends (generator Send bound); `play_order`
  must read stages BEFORE taking the write lock (no reentrant locks).
- **Post-landing fixes (same day)**:
  - mpv playlist is a rolling 32-track WINDOW (PLAYLIST_WINDOW), not the
    whole order — all-artists pools are ~4.4k entries and one IPC command
    per entry floods mpv (later commands queue past the 10s timeout →
    "mpv response timeout" crash look). eof advance tops the window up by 1.
  - **NEVER await IPC commands in the mpv reader task** — it is the task
    that reads responses; awaiting one stalls all reads until timeout
    (the repeat-album eof reload took 20s = 2 chained timeouts while mpv
    had executed instantly). All reader-task side effects are fire-and-
    forget (`fire()`); only command-side tasks await.
  - Repeat-album wrap is PRE-ARMED: entering the last track (natural eof,
    direct click via play_order, or set_repeat while on the last track)
    appends the next pass (reshuffled when shuffle is on) so mpv wraps
    GAPLESSLY — it never goes idle, so the audio device never reopens
    (the reopen was the 3-4s gap). Promoted to the live order at the last
    track's eof; jump/set_shuffle discard or promote the armed pass.
  - Mode buttons (shuffle/repeat): ON = full-brightness glyph (hover color)
    + stage badge (A/R/ALL, "1" for track repeat), OFF = dimmed; user found
    the accent-wash background indistinct and the first hand-drawn glyphs
    broken — Feather icon geometry instead.
- Gates: check 0/0, vitest 48, cargo 41, build OK.

### Step 4 — Accent color picker — done (2026-08-24)
- Gear popover "Accent" row: 9 swatches (stock = split purple gradient,
  selecting it stores null = reset) + custom via native `<input type=color>`
  styled as a conic-gradient swatch; selected ring = outline.
- ONE hex persisted (`ui.accentColor`, localStorage mirror + `accentColor`
  SQLite setting through the existing write-through).
- Pure math in `src/lib/accent.ts` (11 vitest cases): hex↔HSL, WCAG
  relative luminance, `accentVariants(hex, theme)` → lightness clamped dark
  [0.70,0.80] / light [0.45,0.60], `--active` at user-tuned alphas 0.18
  dark / 0.14 light, `--accent-text` white/#1b1b1f by luminance.
- App.svelte effect sets/removes `--accent`, `--active`, `--accent-text` on
  `<html>`; unset → removeProperty → stock purple from app.css. No token
  changes in app.css.
- User fix: `--accent-text` is for SOLID accent fills only — on translucent
  `--active` washes (playbar play/pause, sidebar active row) the glyph must
  stay `--text` (black accent-text on a faint wash read as "glyph turned
  black").
- Gates: check 0/0, vitest 43, cargo 39, build OK.
- **Scanner grouping fix** ("The Singles" bug): Edguy's folder tags
  albumartist, Phil Collins' tags none → Pass 3's "unknown matches anything"
  merged both into one 59-track Edguy album. Fix: subgroup release artist
  falls back to track-artist consensus ONLY when no file tags albumartist AND
  all track artists agree (mixed artists stay None → Various Artists).
  Regression test `same_title_untagged_folder_does_not_join_tagged_artists_album`.
- **Full Rescan (rebuild)** menu item (Library): `scan_library { full: true }`
  reparses every file — needed after grouping/consensus logic changes since
  skipped files never re-enter grouping. `run_scan_roots` gained a `full` flag.
- **Silent scan-killer bug**: a cancelled kdialog folder pick stored the JSON
  literal "null" as musicDir; `music_root`'s `unwrap_or(v)` turned it into a
  relative path "null" → every scan failed INSTANTLY with "music root null
  does not exist" — no progress events, no error surfaced, menu clicks
  "did nothing". music_root now treats null/empty as unset (→ ~/Music).
  Also: SCAN_RUNNING flag reset now happens before the `?` on the join result
  (a panicking scan task can no longer wedge all future scans).

### Step 2b — Playbar gradient toggle — done (2026-08-24)
- Sidebar gear checkbox "Playbar artwork gradient" (`ui.playbarGradient`,
  localStorage mirror + SQLite settings write-through, default OFF).
- Shared `src/lib/gradient.ts`: ExpandedPanel's mix-to-surface math extracted;
  `artGradientContrast` builds the playbar backdrop (135°, album colorC1/C2,
  chrome alpha 0.7 exactly) with a WCAG contrast guarantee — each stop is
  lightness-clamped (hue/sat preserved) until composited contrast ≥4.5:1 vs
  `--text` and ≥3:1 vs the 0.64-alpha dim variant, in both themes. Stopped →
  plain chrome; 400ms background transition. 12 new vitest cases
  (`gradient.test.ts`). Gates: check 0/0, vitest 22, cargo 35, build OK;
  user-verified live.


### Step 1 — Tag editor — done (2026-08-23, fix round 2026-08-24)
- **Backend** (`src-tauri/src/library/tags.rs`): get/save track + album tags
  straight from/to the FILES via lofty (never the DB); saves rely on edited
  mtimes re-parsing through the real grouping path on the next scan. Album
  reads return consensus values + per-field disputed flags + per-track rows;
  album saves stamp ONLY album-level fields (per-track title/artist are never
  touched by them). Tagless files get a tag created via
  `FileType::primary_tag_type()`.
- **Gotchas that cost debugging rounds** (see PLAN.md Step 1 for detail):
  clearing year must purge `ItemKey::Year` AND `ItemKey::RecordingDate`
  (accessor falls back); `TaggedFile::remove` on a FLAC does not persist
  (FlacFile re-writes its held VorbisComments); a lone synthetic MPEG frame
  fails lofty validation — tests use two frames.
- **Frontend**: `TagEditor.svelte` glass modal (te-* namespaced, MusicBee-style
  grid, dirty-tracked Save, Esc cancels, disputed "•" markers); first context
  menu (`ContextMenu.svelte` + `contextMenu.svelte.ts` store) — right-click
  track → "Edit tags…", album header → "Edit album tags…"; always-visible
  pencil button next to play-all; regroup vanish-guard closes the editor and
  collapses the expansion if a save merges/splits the open album away.
- Save → `rescan()` → `scan-finished` refreshes the frontend. Gates: check
  0/0, vitest 10, cargo 27, build OK; user-verified live on the real library.
- **Fix round (2026-08-24, first real-library use — full detail in PLAN.md
  "Step 1 fix round")**:
  - serde camelCase mismatch silently dropped albumArtist (and numbers) on
    save → `rename_all = "camelCase"`; this was the "undef" + "save does
    nothing" root cause.
  - Stacked-ID3v2 MP3s (Lavf52-era): reader merges blocks, writer rewrites
    only the first → edits silently vanished. `write_file` now reread-
    verifies and collapses to a single merged tag on mismatch. cargo test
    `stacked_id3v2_save_actually_lands`.
  - Album modal: per-track list removed (user decision); `save_album_tags`
    never touches per-track fields.
  - Scan retag-adoption: changed group matching an existing album by
    (artist, norm title) adopts its id regardless of year — editor merges
    now work across the incremental-scan skip. Test:
    `retagged_stray_adopts_existing_album_regardless_of_year`.
  - Track order: unnumbered tracks alphabetical before numbered (per disc),
    in dump + play_album + album_rows. Test:
    `unnumbered_tracks_sort_alphabetically_first`.
  - Restored the 12 Edge of Tomorrow track numbers (wiped by the serde bug)
    from `(NN)` filename prefixes. User verified everything live: Stereopony
    stray merged into Anison no Kokoro (218 tracks), numbers back, modal
    clean. Gates: cargo 30, check 0/0, vitest 10, build OK.


### Phase 4 — MPRIS (superseded by the done section above)
- zbus server: org.mpris.MediaPlayer2 + Player; wire to the mpv `PlayState`
  (Rust already owns the album context); Next/Prev clamp within album; Seeked
  signal; metadata trackid/length/artUrl(cover).

## Explicitly deferred
"Play next" queue escape hatch · packaging/.desktop/icon · multi-library roots ·
live library watching. (KWin Force-Blur automation: DONE 2026-08-23 —
`kwin-blur-load.service` user unit, see Phase 2 fix-round notes.)
(Tag editor: DONE — see Step 1.)
- Title-bar menu (agreed 2026-08-22): menu button in `TitleBar` for *Rescan
  library* / *Choose music folder…* / *About Songstress*; sidebar gear stays
  appearance-only. Lands with Phase 2 M4 (see PHASE2.md §8) since rescan needs
  the scan backend to exist. ✅ landed.

## Known issues / open threads
- WebKitGTK viewport glitch: on some launches the webview rendered at a stale
  smaller size inside the correctly-sized GTK window (playbar mid-window, dead
  glass below); activation didn't heal it, restarts were a coin flip.
  `WEBKIT_DISABLE_DMABUF_RENDERER=1` fixed the glitch but killed backdrop
  transparency — rejected. Fix applied 2026-08-22: Rust setup hook nudges the
  window 1px down+back at ~0.6s/~2.5s after launch, forcing a reconfigure,
  PLUS a self-healing guard (`src/lib/viewportGuard.ts` + `fix_viewport`
  command): for the first 60s after load the frontend compares its viewport
  against the real window size (8px tolerance) and re-nudges on mismatch.
  Verified across consecutive relaunches incl. one that reglitched post-nudge.
- Expansion animation rewritten 2026-08-22 (was grid-template-rows 0fr→1fr):
  burst-screenshot diagnosis showed WebKitGTK animates the grid track and the
  content clip on DIFFERENT clocks — first expand snapped with no animation,
  collapse left an empty shell band below already-vanished content, and panels
  sometimes sat empty until close+reopen. Now `.inner` animates explicit px
  height (rest states "0px"/"auto", settle timer 400ms) so clip edge, panel
  edge and shadow move in lockstep; verified frame-by-frame in slow motion.
- Switch lag root causes addressed 2026-08-22: album_colors was a SYNC tauri
  command (full-res JPEG decode on the main thread per switch) — now async +
  spawn_blocking + path-keyed COLOR_CACHE. Remaining suspects if lag persists:
  DX blur re-frost per frame, webview raster cost.
- Expanded panel shadow moved from .panel to .expander (2026-08-22): the inner
  wrapper must stay overflow:hidden for the height animation and clipped the
  panel's shadow into sharp corners; expander is never clipped and tracks the
  animated box, so the shadow follows every frame (also kills the late shadow
  pop).
- Grid scrollbar hidden permanently (2026-08-22, user request):
  `scrollbar-width: none` + `::-webkit-scrollbar { display: none }` on the
  `.content` scroller; wheel/trackpad scrolling unaffected.
- User's ~/.config/mpv/mpv.conf broken (d3d11/C:\ fonts) — unrelated to app but
  worth telling them once more.
- KWin DX multi-line WindowClasses silently fails to match — keep one entry.

## Environment facts
- Fedora 44, Plasma 6.7.4 Wayland, output 3840×2160 @ scale 1.6 (logical 2400×1350).
- GPU: AMD Radeon 780M (radeonsi, Mesa 26.1.7); WebKitGTK composites via EGL/gbm.
- mpv 0.41 at /usr/bin/mpv. sudo requires password (user runs dnf lines).
- Raise/activate our window: WindowsRunner DBus Match+Run (see AGENTS.md).
