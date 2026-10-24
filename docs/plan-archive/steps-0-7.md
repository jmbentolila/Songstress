# Step 0 (plan archive)

Moved out of `PLAN.md` on 2026-10-24 so the plan file stays readable in one pass.
This is the record, not the state: current state lives in `PLAN.md`, and this
file is the evidence behind it. Sections are kept verbatim, in their original
order. The index of every section is `README.md` next to this file.

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

