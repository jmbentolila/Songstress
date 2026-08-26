# PHASE 2 — Real Library (SQLite + lofty): Design

Status: **draft for review** — no code yet. Decisions locked here become the
implementation contract; change the doc first, then code.

Ground rules from AGENTS.md still apply: Svelte 5 runes stores, no CSS
framework, `npm run check`/`vitest`/`cargo test --lib` green before done,
HMR rots (cold restart), debug Rust panics on overflow.

---

## 1. Goals

- Replace `fakeLibrary.ts` with a real music library scanned from disk into
  SQLite, without rewriting any UI component.
- Compute expensive derived data **once, during scan**: panel colors
  (kills the current first-expand gradient lag at the correct layer),
  thumbnails, sort names.
- Persist app settings in SQLite (single source of truth), migrating the
  existing localStorage keys.
- Keep the fake library available as a UI-dev mode behind a flag.
- Stay out of Phase 3's lane: no MPV work here, but don't paint us into a
  corner (stable IDs, duration as f64 seconds, path always stored).

## 2. Non-goals (deferred, do not build)

- Live filesystem watching (rescan is manual/button + on-launch).
- Multiple library roots, network mount special-casing.
- Playlists, ratings, play counts, "play next" queue.
- ReplayGain *application* (Phase 3 concern); we only preserve tags we find.

## 3. Current state (what we're replacing)

- `src/lib/types.ts`: `Artist{id,name,sortName}`, `Album{id,artistId,title,year,cover}`,
  `Track{id,albumId,disc,track,title,durationSec}`, `PlaybackContext{albumId,trackIndex}`.
- `src/lib/stores/library.svelte.ts`: thin facade `{artists, albums, tracksOf(),
  artistOf()}` over static arrays — **this facade is the seam**; components never
  touch fakeLibrary directly.
- Colors: frontend `artColors.ts` invokes Rust `album_colors(file)` per cover;
  path-keyed `COLOR_CACHE` in Rust dedupes per session; nothing survives restart.
- Covers are `public/covers/*.jpg`, referenced by URL string on `Album.cover`.
- Playback is a fake timer engine in `playback.svelte.ts` driven by
  `{albumId, trackIndex}`.

## 4. Rust-side architecture

New module tree under `src-tauri/src/`:

```
library/
  mod.rs      — public API used by commands: scan(), get_library(), settings
  db.rs       — connection management, migrations, helpers
  scan.rs     — walkdir + lofty tagging + grouping/upsert logic
  thumbs.rs   — thumbnail generation (image crate → WebP)
  colors.rs   — MOVED from lib.rs verbatim (same algorithm/tests)
```

- **rusqlite (bundled feature)**, sync API, all calls inside
  `spawn_blocking`. One connection guarded by a Mutex; WAL mode. No sqlx —
  we don't want compile-time SQL checks bad enough to add an async runtime
  dependency to commands that are already fine on the blocking pool.
- Migrations: hand-rolled `schema_migrations(version INTEGER PK, applied_at)`
  + ordered `MIGRATIONS: &[&str]` applied in a transaction on open. No extra
  crate.
- DB location: `app.path().app_data_dir()/songstress.db`.

### 4.1 Schema

```sql
CREATE TABLE artists (
  id   TEXT PRIMARY KEY,        -- "ar-" || blake3_64(normalized name)
  name TEXT NOT NULL,
  sort_name TEXT NOT NULL
);
CREATE TABLE albums (
  id        TEXT PRIMARY KEY,   -- "al-" || blake3_64(albumartist|title|year)
  artist_id TEXT NOT NULL REFERENCES artists(id),
  title     TEXT NOT NULL,
  year      INTEGER,
  cover     TEXT,               -- thumb:// id or NULL (see §6)
  color_c1  TEXT,               -- "rrggbb", NULL until computed
  color_c2  TEXT,
  UNIQUE(artist_id, title, year)
);
CREATE TABLE tracks (
  id           TEXT PRIMARY KEY, -- "tr-" || blake3_64(path)
  album_id     TEXT NOT NULL REFERENCES albums(id),
  disc         INTEGER NOT NULL DEFAULT 1,
  track        INTEGER,
  title        TEXT NOT NULL,
  duration_sec REAL NOT NULL,
  path         TEXT NOT NULL UNIQUE,
  mtime_ns     INTEGER NOT NULL, -- scan bookkeeping
  size         INTEGER NOT NULL
);
CREATE INDEX idx_tracks_album ON tracks(album_id, disc, track);
CREATE TABLE settings (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL            -- JSON
);
```

Why text IDs from hashed natural keys:
- Stable across rescans (rows are upserted, never delete+reinsert while the
  natural key lives) → safe as MPRIS trackids in Phase 4 and as Vue… er,
  Svelte each-keys today.
- No AUTOINCREMENT churn; re-scans converge instead of duplicating.
- `blake3` (or sha2 if we want zero new deps — decide at implementation,
  either is fine; only stability matters, not cryptographic strength).

### 4.2 Settings keys (migrated from localStorage)

`musicDir`, `theme`, `tileSize`, `sidebarRowSize`, `volume`,
(replaygain mode lands in Phase 3). Migration: on first DB open the frontend
POSTs its current localStorage values via `init_settings`; DB wins from then
on. `ui.svelte.ts` hydrates from `get_settings` at startup and writes through
(debounced 300ms) via `set_setting`. localStorage stays as a write-through
mirror purely so the very first paint before hydration matches.

## 5. Scan pipeline

Command: `scan(started_by_user: bool)` → runs on blocking pool; a scan
already running makes a second request a no-op returning the current job.

1. Resolve root from settings (`musicDir`, default `~/Music`).
2. walkdir; extensions: mp3, flac, m4a, aiff, ogg, opus, wav (wav = sparse
   tags, accept filename-derived titles).
3. Skip unchanged: `path` present in DB with same `mtime_ns`+`size` → leave
   row untouched (incremental; first scan touches everything).
4. Tag with lofty: title, track, disc, album, albumartist, artist, year,
   duration (from properties), embedded picture (largest), replaygain tags
   (stored raw in `tracks` later if needed — not in v1 schema above; YAGNI).
5. Grouping (REVISED 2026-08-22 — directory-authoritative; supersedes the
   original (release_artist, album, year) key, which split real albums when
   `albumartist` was missing on some tracks and misfiled guest-tracked
   albums under Various Artists):
   - **Same folder ⇒ same album.** Tracks bucket per
     (parent directory, normalized album title); differing titles inside one
     directory still separate two real albums.
   - Per-bucket metadata is CONSENSUS, not per-track: release artist = most
     common non-null `albumartist`; title = most common album-tag spelling;
     year = most common non-null year. One stray/missing tag can never split
     an album; year is stored metadata, never identity.
   - Buckets merge across directories (multi-disc trees) when normalized
     titles match and release artists agree (unknown matches anything);
     distinct known artists with the same title stay separate.
   - Artist assignment trusts `albumartist`: any albumartist found ⇒ album
     belongs to that artist even when track artists differ (guests stay
     guests). NO albumartist anywhere + identical track artists ⇒ that
     artist. NO albumartist + differing track artists ⇒ Various Artists
     (id fixed `"ar-various"`).
   - artist rows keyed by normalized name (lowercase, diacritic-folded —
     port `compareByName`'s folding to Rust; reuse for `sort_name`).
6. Upserts via `ON CONFLICT DO UPDATE`; collect seen paths; afterwards mark
   rows whose path wasn't seen as deleted (v1: hard delete; tombstoning
   deferred until playlists exist).
7. Post-pass (same job, lower priority): for albums with `cover IS NULL` or
   colors NULL → extract/write thumbs + colors (§6). Emits progress events.
8. Events to frontend: `scan-progress {phase, done, total}` and
   `scan-finished {added, updated, removed, errors[]}`. Errors are per-file,
   collected, non-fatal.

Cancellation: drop the job handle on app exit; explicit cancel button is
non-goal v1 (scans of personal libraries are minutes at worst).

## 6. Artwork: covers, thumbnails, colors

Priority per album (first hit wins; folder-name matching is
case-insensitive — `Cover.JPG` counts):
1. `cover.jpg/folder.jpg/front.png` in the album's dominant directory
   (MusicBee-style libraries — the user's current setup).
2. Largest embedded picture among the album's tracks.
3. NULL → UI falls back to placeholder tile (already handled: `cover: null`).

Pipeline: pick source → decode once →
- thumbnails: 96 / 256 / 512 px WebP written to
  `app_cache_dir()/thumbs/<album_id>/{96,256,512}.webp`;
- colors: run the existing `colors::extract` on the same decoded buffer
  (refactor `extract` to take an `&DynamicImage` so scan and tests share it),
  store hex in `albums.color_c1/c2`.
- Serve via custom protocol: `register_uri_scheme_protocol("thumb")` →
  `thumb://<album_id>/<size>.webp` (stream from cache dir; 404 → fallback
  chain 512→256→96). Album.cover becomes `thumb://al-…/512.webp` strings —
  same "URL string" shape the UI already expects, so components don't change.
- Regeneration rule: thumb/color pass runs for albums missing outputs OR
  whose source mtime changed. Cache invalidation is keyed on source mtime
  stored alongside (in memory during the job; a `meta` row if it proves needed).

This retires the per-expand `album_colors` round-trip: the albums payload
carries colors, and `ExpandedPanel` renders the gradient synchronously from
`album.color_c1/c2` (fallback to today's invoke path only if null).

## 7. Command surface (Tauri)

```text
get_library() -> {artists:[], albums:[], tracks:[]}     // full dump, one-shot
scan(user_initiated: bool) -> {job_accepted: bool}
init_settings(map) / get_settings() -> map / set_setting(key, json)
// events: scan-progress, scan-finished
```

Full-dump rationale: personal-library scale (≤ ~100k tracks ≈ low-MB JSON)
makes pagination pure overhead; revisit only if dump exceeds ~20 MB.
`library.svelte.ts` keeps its exact facade shape but builds `tracksOf()` as a
Map lookup from the dump and refreshes on `scan-finished`.

## 8. Frontend integration

- `VITE_LIB=live` env flag (default `fake`) selects the library backend in
  `library.svelte.ts`. Fake mode untouched for UI dev.
- New empty-state screen when live mode has zero albums ("Choose your music
  folder" → invokes folder picker via tauri-plugin-dialog → triggers scan).
- **Title-bar menu** (agreed 2026-08-22): a menu button in `TitleBar` next to
  the traffic lights owns library/system actions — *Rescan library*,
  *Choose music folder…*, *About Songstress*. The sidebar gear keeps
  appearance-only scope (theme, tile sizes); don't merge the two. Rescans
  surface job state via the `scan-progress`/`scan-finished` events (§5).
- `ui.svelte.ts` switches to settings-backed hydration per §4.2.
- `artColors.ts` shrinks to a sync read of `album.color_c1/c2` with the old
  invoke path kept as fallback for NULL (pre-first-scan albums).
- types.ts gains optional fields (`colorC1?`, `colorC2?`) — additive only.

## 9. Testing

Rust:
- Fixture audio files committed under `src-tauri/fixtures/` (tiny: 1s silence
  flac + mp3 + m4a generated once via ffmpeg, checked in) with known tags —
  including a two-disc album, a compilation, and diacritic/„The“-prefix names.
- `scan.rs` tests: grouping, ordering (disc,track), sort_name folding,
  incremental re-scan (touch one file → exactly one update), deletion sweep.
- `db.rs` tests: migrations apply clean twice (idempotent), settings round-trip.
- `colors.rs`: existing tests move along unchanged (algorithm frozen).

Frontend:
- vitest row-model/sort suites untouched (pure functions, already covered).
- New: library store facade over a canned dump (Map lookup correctness,
  scan-finished refresh), settings hydration merge order
  (localStorage → DB override).

## 10. Rollout milestones

| # | Deliverable | Done when |
|---|---|---|
| M1 ✅ 2026-08-22 | db.rs + migrations + settings commands | cargo tests green; DB created + migrated on launch (`app_data_dir/songstress.db`); settings survive restart |
| M2 ✅ 2026-08-22 | scan.rs + fixtures + events | fixture library scans correctly, incremental verified |
| M3 ✅ 2026-08-22 | thumbs protocol + colors-in-scan | thumb:// serves in running app; albums carry colors |
| M4 ✅ 2026-08-22 | frontend live mode + empty state + settings migration | full UI runs on scanned library; fake mode still switchable |
| M5 | cleanup | album_colors invoke path removed from hot path; PROGRESS.md updated |

M2 implementation notes (2026-08-22):
- `scan_library` command: optional `root` param (else `musicDir` setting, else
  `~/Music`), runs on spawn_blocking with its OWN connection, guarded by an
  AtomicBool against concurrent scans; emits `scan-progress {done,total}` and
  `scan-finished {added,updated,removed,skipped,errors[]}`.
- Grouping per §5: albumartist → artist → Unknown Artist; mixed track artists
  → `ar-various`; sort_name via Rust port of sort.ts (incl. NFD-decomposed
  input); albums keyed (release-artist-norm, album-norm, year).
- Incremental: unchanged (mtime_ns+size) rows untouched; vanished paths
  deleted; orphan albums/artists swept (Various Artists pinned).
- Fixtures committed at `src-tauri/fixtures/library/` (232K): multi-disc,
  compilation, Björk diacritics, "The" prefix, flac/mp3/m4a/ogg/opus/wav;
  wav exercises filename-title fallback. Tests operate on a temp copy.
- 13/13 cargo tests, zero warnings.

Each milestone ships behind the flag; the app must stay usable in fake mode
throughout.

M4 implementation notes (2026-08-22):
- Flag semantics flipped to opt-OUT: `LIVE_LIBRARY = isTauri && VITE_LIB !==
  "fake"` — the Tauri app is live by default, browser dev always fake.
- `library.svelte.ts` became a class store: same facade shape (artists/
  albums/tracksOf/artistOf), live data hydrated via new `get_library` dump
  command (camelCase JSON matching types.ts), refreshed on `scan-finished`.
  Albums carry colorC1/colorC2; ExpandedPanel gradient reads them directly
  (invoke fallback only when NULL — first-expand gradient lag is gone in
  live mode).
- Settings hydration (`ui.svelte.ts`): init_settings seeds from localStorage,
  get_settings wins afterwards; writes go through debounced (300ms)
  pushSetting alongside the localStorage mirror. musicDir lives here too.
- Empty state (`EmptyState.svelte`) shows when live + no albums: folder
  picker (@tauri-apps/plugin-dialog, dialog:default capability) → persists
  musicDir → scan with progress bar (scan-progress listener).
- Title-bar menu (hamburger beside traffic lights): Rescan library (disabled
  while running, shows progress), Choose music folder…, About Songstress
  (getVersion). Shared orchestration in `stores/scanner.svelte.ts`.
- ⚠ Pending user verification: real scan of ~/Music/Music Files, thumb://
  rendering end-to-end, menu flows.

M3 implementation notes (2026-08-22):
- `library/artwork.rs`: post-pass over albums with cover/color NULL →
  dominant track directory → folder art (cover/folder/front .jpg/.png) else
  largest embedded picture → ONE decode → 96/256/512 WebP thumbs into
  `app_cache_dir()/thumbs/<album_id>/` + colors via refactored
  `colors::extract_image(&DynamicImage)`. No source → stays NULL (UI
  placeholder); later rescans retry. Idempotent.
- `thumb://` protocol registered with size fallback chain (requested →
  512 → 256 → 96), immutable cache headers, path-traversal guard.
  ⚠ Visual verification of protocol serving happens in M4 when the UI
  renders scanned albums.
- Fixtures extended: folder.jpg for Helloween (folder path), embedded art on
  both Synth Wars tracks (embedded path). Artwork test covers filled /
  placeholder / idempotency paths.
- scan_library now runs artwork refresh as a post-pass (same progress
  channel, phase:"artwork"). 14/14 cargo tests.

M1 implementation notes (2026-08-22):
- `src-tauri/src/library/{mod,db,settings}.rs`; `MIGRATIONS` v1 = full §4.1
  schema; migration + version row commit atomically.
- `AppState { db: Mutex<Connection> }` managed in setup; commands
  `get_settings` / `set_setting` / `init_settings` (INSERT-OR-IGNORE seeding
  semantics — DB never clobbers its own values from stale localStorage).
- `stable_id(prefix, parts)` helper (blake3, 16 hex chars) ready for M2.
- rusqlite bundled + blake3 added to Cargo.toml. 10/10 cargo tests.

## 11. Open questions — RESOLVED 2026-08-22

1. Default music root — **`~/Music`** (changeable later via title-bar menu/settings).
2. Cover priority — **folder art > embedded**, placeholder when neither.
3. Formats v1 — **mp3, flac, m4a, aiff, ogg, opus, wav** (wav: sparse tags,
   filename-derived titles acceptable).
4. ID hashing — **blake3**, 64-bit, hex-encoded with `ar-`/`al-`/`tr-` prefixes.
