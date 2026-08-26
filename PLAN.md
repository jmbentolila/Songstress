# PLAN.md — Songstress roadmap (interruption-resilient)

Companion to **AGENTS.md** (conventions/gotchas) and **PROGRESS.md** (landed work).
This file is the working plan: check boxes off as work lands, never lose context
to an interrupted session. Each item is written so a FRESH session can pick it
up with no prior conversation: every decision is recorded, every step is small
and independently verifiable.

**Resume protocol**: pick the first unchecked item, do only that item, run its
verification, tick it, commit nothing unless asked, update PROGRESS.md when a
whole step lands. If context was lost, re-read AGENTS.md + this file; that is
enough.

Status legend: ⬜ todo · 🔶 in progress · ✅ done

---

## Step 0 — Bug fixes (BEFORE all features)

### 0a. Tracklist phantom hover ✅
**Symptom**: hovering below the last track of column 1 highlights the first
track of column 2.
**Root cause (confirmed)**: `.tracklist.two` uses CSS multicol (`columns: 2` +
`break-inside: avoid`, ExpandedPanel.svelte:527-534) — WebKit multicol
hit-testing maps dead-zone points into the next fragmentainer.
**Fix**: replace multicol with explicit grid in ExpandedPanel.svelte:
`display: grid; grid-auto-flow: column; grid-template-columns: 1fr 1fr;
column-gap: 24px; grid-template-rows: repeat(⌈n/2⌉, auto)` — row count
computed in the component (`Math.ceil(list.length / 2)`). DOM order and
visuals identical; applies to BOTH the single-tracklist branch (line 361)
and the per-disc branch (line 331).
**Verify**: hover-probe the dead zone below column 1 in single-disc and
multi-disc albums; full AGENTS.md gates.

### 0b. Frozen border artifacts on window move 🔶 (diagnosed: upstream effect bug)
**Symptom**: fast side-to-side window moves leave a ~1px stale vertical line
at the old window border (captured via screen recording; both left and right
borders seen). Vanishes on any forced redraw. ALSO: switching to the app
shows stale translucency for a frame (glass backdrop shows the previous
background until a repaint) — same BlurCache staleness family, include in
the upstream report.
**Diagnosis COMPLETE (2026-08-23)**:
- ✅ Blur A/B (both directions): artifacts GONE with `better_blur_dx` unloaded,
  back when loaded. Effect is the culprit.
- ✅ RoundedCornersPass ruled out: `CornerRadius=0` + reload → artifacts still
  appear. Config restored to 14.
- User's build: kwin-effects-better-blur-dx 2.5.1 (git 20260808 e8475d0,
  xarblu fork — taj-ny's original ARCHIVED Nov 2025). Build already contains
  upstream's July/Aug cache+scissor+ceil fixes ("fixup glScissor for small
  dirtyRegions", "ceil glWidth/glHeight", "expand contentsRect by 1px").
- Leading theory: stale/short glScissor in the BlurCache draw path clips the
  exposed-region repaint at fractional scale (1.6×) — the final device-pixel
  column of the old window rect is never repainted. Upstream territory.
- Upstream refs: xarblu/kwin-effects-better-blur-dx #92 (blur caching stale
  transparency — closed by adding fixes, not a toggle), #14 (move artifacts,
  closed), taj-ny #143 (artifacts at smallest blur strength, open).
**Action**:
- ⬜ File upstream issue with the screen recording (repo is active, similar
  issues fixed within weeks). NOTE: GitHub showed "issue creation is
  restricted" — user may need to comment on #14/#92 or file via KDE bugzilla.
- ⬜ Re-test after `dnf update kwin-effects-better-blur-dx` (user runs dnf).
- Cosmetic meanwhile: artifact clears on any redraw over that area.
- App-side fix impossible: the stale pixel is outside the moved window — only
  the compositor/effect can repaint it.

### 0c. Artwork blank-frame flash ✅
**Symptom**: fast mouse sweeps over grid artworks, or returning to the app
after idle → a cover goes blank for one frame.
**Root cause (confirmed by bisect)**: hover `translate` promotes a compositor
layer per hover-in/out; WebKitGTK paints the texture create/destroy churn as
a one-frame blank.
**Bisect history** (all tested live):
- drop `loading="lazy"` alone: flash persisted (kept anyway — idle eviction
  protection, 246 thumbs are cheap)
- permanent `will-change: translate` on all covers: flash GONE but 246
  permanent layers made 0b's move artifacts MUCH worse → reverted
- `:has()` batch promotion while grid hovered: flash worse → reverted
- **FINAL FIX**: removed the hover lift entirely; hover cue is now a
  paint-only outline ring (`outline-color: var(--text-dim)`) — no transform,
  no layer promotion, no churn. User verified flash gone.
**Note**: if the lift microinteraction is missed later, revisit only when
WebKitGTK fixes layer-churn blanking.

---

## Step 1 — Tag editor ✅

Full MusicBee-style editor. Motivation: real albums misbehave from source tag
issues (wrong albumartist, wrong year, split albums).

**Rust — `src-tauri/src/library/tags.rs` (new) — DONE 2026-08-23**:
- ✅ `get_track_tags(track_id)`: path from DB → `lofty::read_from_path` →
  fields: title, artist, album artist, album, year, track n + total, disc
  n + total, genre, composer, label (ItemKey::Label), comments, grouping
  (ItemKey::ContentGroup). Tagless files → empty fields.
- ✅ `get_album_tags(album_id)`: album-level consensus (first non-empty per
  field across tracks, flagged when tracks disagree) + per-track list.
- ✅ `save_track_tags` / `save_album_tags`: `primary_tag_mut().or(first_tag_mut())`
  → `remove_key` + `insert_text` (fixture-test API, scan.rs:573-614),
  `save_to_path` with `WriteOptions::default()`. Tagless file → create tag
  (`insert_tag(Tag::new(...))` via `FileType::primary_tag_type()`).
- ✅ After save: frontend triggers the incremental `scan_library` (edited
  mtimes guarantee re-parse through real grouping; `scan-finished` auto-
  reloads the frontend). NO parallel DB-update path, NO migration.
- ✅ Tests: write/read roundtrip incl. clearing (empty string/None removes
  keys — NOTE: clearing year must purge BOTH `ItemKey::Year` AND
  `ItemKey::RecordingDate`; the year() accessor falls back to RecordingDate
  and set_year rewrites one in place), synthetic two-frame untagged MP3
  (single frame fails lofty validation), and the "fix albumartist merges
  split albums" case. GOTCHA: stripping tags from a FLAC via
  `TaggedFile::remove` does NOT persist — FlacFile::write_to re-writes its
  internally-held VorbisComments; only editing existing/created tags works.

**Frontend** — DONE 2026-08-23:
- ✅ `TagEditor.svelte`: glass modal (`--panel-bg-strong`, `te-*` namespaced
  CSS), MusicBee-style label+field grid (year / track-n-of / disc-n-of rows).
  Album mode adds per-track editable list (title, n, disc). Dirty tracking →
  Save only on change; Esc cancels.
- ✅ `ContextMenu.svelte` (first context menu; `contextMenu.svelte.ts` store +
  TitleBar `.tb-pop` look): right-click track row → "Edit tags…", album
  header → "Edit album tags…".
- ✅ Album header: small always-visible edit button next to play-all.
- ✅ State: `ui.tagEditor = { open, albumId?, trackId? }`; modal in App.svelte.
- ✅ Regroup edge: $effect watches library.albums while open — if the edited
  album vanishes (merge/split), the editor closes and the expansion collapses
  gracefully.

**Verify**: cargo tests + gates + visual pass editing a real misbehaving album.
Gates passed 2026-08-23 (check 0/0, vitest 10, cargo 27, build OK); visual pass
on real albums DONE 2026-08-24 after the fix round below — user-verified live
(Stereopony stray merged into Anison no Kokoro, Edge numbers restored, album
modal clean).

### Step 1 fix round — DONE 2026-08-24 (first real-library use exposed 3 bugs)

1. **serde case mismatch** (root cause of "save dropped my albumartist"):
   structs serialized snake_case (`album_artist`) while the frontend spoke
   camelCase (`albumArtist`). Unknown keys are ignored + `#[serde(default)]`
   ⇒ multi-word fields were silently EMPTY on save (albumartist wiped!) and
   showed "undef"/blank in the editor. Single-word fields (album, composer)
   worked — which is why the composer save seemed fine. FIX:
   `#[serde(rename_all = "camelCase")]` on TrackTags/AlbumTags.
2. **Stacked-ID3v2 silent-write no-op** (why "nothing happens on save"): some
   real MP3s (Lavf52-era muxers) carry SEVERAL ID3v2 blocks. The reader
   MERGES them into one view; `save_to_path` rewrites only the FIRST block
   and returns Ok — a later block's stale frames win on reread, so edits
   vanish silently. Diagnosed by hexdump (tag #1 = 187 KB holding just TCMP,
   tag #2 = 370 B holding the real frames). FIX: `write_file` now VERIFIES
   by rereading after every save; on mismatch `repair_stacked_tags` strips
   every on-disk tag of that type and rewrites the merged+edited tag as the
   only one (lossless — the merged view already is the union). Regression
   test builds a synthetic stacked MP3 and proves plain lofty save loses the
   edit while our path lands it.
3. **Album modal per-track list removed** (user decision — numbers showed
   "undef" pre-fix and album saves have no business touching per-track
   fields): `save_album_tags` lost its `tracks` param and NEVER writes
   title/artist/numbers; per-track edits are track-mode only (hint added).
   TrackRow struct deleted.
4. **Retag-adoption in scan** (`scan.rs`): a changed group whose (artist,
   norm title) matches an EXISTING album adopts its id regardless of year —
   year is metadata, never identity (documented intent, now actually true).
   Needed because the incremental scan skips the target album's own files,
   so pass-3 cross-dir merge can't see them. This is what let the Stereopony
   fix join the 218-track compilation (file year 2012 vs album 2019).
5. **Track ordering rule** (user decision): within an album disc, tracks
   with NO track number sort alphabetically BEFORE numbered tracks.
   Implemented in SQL (`ORDER BY album_id, disc, (track IS NOT NULL), track,
   title`) in get_library dump, play_album (playback order = display order)
   and tags::album_rows. Test: unnumbered_tracks_sort_alphabetically_first.
6. **Data restoration**: the buggy album save had wiped track numbers on all
   12 Edge of Tomorrow files (rows serialized as null → remove_key).
   Restored from the `(NN)` filename prefixes via one-off write_file run;
   user verified in-app after rescan.

Gates 2026-08-24: cargo 30 (3 new tests), check 0/0, vitest 10, build OK.

---

## Step 2a — Import music ✅ (redesigned 2026-08-24: two-step staging flow)

**Flow (user decision)**: importing is TWO separate steps —
1. **Import** = copy into a staging area; tracks are playable immediately but
   live OUTSIDE the library dir. For listening/trying out.
2. **Save to library** = copy staged files into `<musicDir>/…` (per album,
   per track, or a global "save everything staged"). Until saved, staged
   files PERSIST (survive restarts); "Discard" deletes them.

**Decided**:
- Staging root `<app_cache_dir>/import/<sourceFolderName>/` (source folder
  name preserved; loose files keep their parent dir's name). Import COPIES
  (originals stay at source). Collision: same-size ⇒ skip; different size ⇒
  " (2)" suffix (never overwrite).
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
- Cross-root album merge (pass 3) stays ENABLED: re-importing an album that's
  already in the library merges into it; the badge then marks the staged
  files. Save copies staged tracks out + deletes staging originals + rescan
  (no duplicate-track window).
- Save granularity: per album (context menu), per track (context menu), and
  global "Save imported music" in the titlebar menu (saves ALL staged —
  "scan current content, save what isn't in the library dir"). Same-name-
  same-size files in musicDir are skipped (dedupe).
- Discard granularity mirrors save (album / track / all).
- Pickers: kdialog ONLY (AGENTS rule) — `--getopenfilename --multiple` for
  files, `--getexistingdirectory` for folders. Drag-and-drop onto the grid
  as a second import path (Tauri file-drop), same copy logic.
- Copy emits `scan-progress` (phase "import"); then incremental scan →
  `scan-finished` refresh.

- ✅ Rust: `import_dir()` helper; `run_scan_roots` multi-root; staged flag
  computed in the dump; `import_music(paths)` (files + folders, collision-
  safe: same-size skip / " (N)" suffix); `choose_import_files` +
  `choose_import_folder` (kdialog, `--separate-output` for multi-file).
- ✅ Rust: `save_imports(album_id?, track_id?)` / `discard_imports(...)`;
  save re-roots the staging relpath into `<musicDir>`, deletes staged
  originals, prunes empty staging dirs. Tests: layout, collision, save+dedupe,
  discard (cargo 34).
- ✅ UI: menu "Add music files…" / "Add music folder…" / "Save imported
  music" (disabled when nothing staged); Imported badge (grid tile + panel
  header); context-menu Save/Discard on staged albums + tracks; Track type
  now `track: number | null` (unnumbered tracks render blank).
- ✅ Drag-and-drop import (`onDragDropEvent` → same staging flow).
- ✅ Save layout: `<musicDir>/<Artist>/<Album>/` (user decision 2026-08-24).
- ✅ Import skips files already known (library DB rows that still exist on
  disk + staging contents) — stale rows of deleted files never block.
- ✅ play_album refuses to ghost-play a track whose file is missing (clean
  error instead of mpv error-skipping past a stale UI state).
- ✅ MISSING-FILE LIFECYCLE (user decision 2026-08-24): the scan NEVER
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
- ⬜ Visual pass on the real library (user).

## Step 2b — Playbar gradient toggle ✅ (2026-08-24, user-verified)

- ✅ Sidebar gear setting "Playbar artwork gradient" (checkbox; persisted via
  settings write-through + localStorage mirror, default OFF).
- ✅ When on + a track loaded: playbar backdrop = 135° two-stop gradient from
  current album's colorC1/C2 via new shared `src/lib/gradient.ts`
  (`artGradientContrast`, same mix-to-surface math the panel uses — extracted
  from ExpandedPanel, which now imports it), alpha 0.7 = chrome exactly.
  Falls back to plain chrome when stopped. `transition: background 400ms`.
- ✅ CONTRAST GUARANTEE (user request): every stop is lightness-clamped (hue/sat
  preserved) until the stop composited at 0.7 over the theme surface clears
  WCAG 4.5:1 vs `--text` AND 3:1 vs the dimmed variant (`--text-dim` @ 0.64).
  Dark theme pushes stops darker, light theme lighter; passing stops untouched.
  `gradient.test.ts` (12 vitest cases: extreme covers × both themes).
- Gates 2026-08-24: check 0/0, vitest 22, cargo 35, build OK. User verified
  live (gradient follows albums, text always legible).

---

## Step 3 — Menu bar + Plasma Global Menu + window identity ✅

**Menu model**: Rust-OWNED (source of truth), frontend fetches via command +
`menu-action` events keep it fresh. Menus: **Playback / Library / View /
Help** (DECIDED).
- Playback: Play/Pause (dynamic label), Stop, Previous, Next
- Library: Rescan Library, Add Music… (step 2a), Choose Music Folder…
- View: theme toggle (checkable)
- Help: About Songstress

**Renderers**:
- ✅ In-titlebar menu bar (proper text menus) — landed 2026-08-24; hamburger
  REMOVED later that day (user call: Global Menu XOR titlebar bar, no fallback).
- ✅ Plasma Global Menu widget: `com.canonical.dbusmenu` served over zbus at
  `/org/songstress/Menu` as `com.yossi.songstress` + KWin Wayland registration
  via `org_kde_kwin_appmenu` — LANDED 2026-08-24, user-verified in the panel.
- ✅ dbusmenu methods + LayoutUpdated signal; activation funnels through
  `menu_activate` (playback native in Rust, rest as `menu-action`).
- ✅ Registrar fallback: NOT NEEDED — the Wayland path works.
- **War stories (do not rediscover)**:
  - KWin links an appmenu object to its window ONLY at `appMenuCreated` time
    and only if the window is already MAPPED (`WaylandServer::findWindow`
    searches mapped windows). At launch we race the first buffer attach →
    create/release/retry loop, stopping as soon as a dbusmenu consumer queries
    us (LAST_QUERY_MS touched in GetLayout/AboutToShow) — each re-register
    otherwise flaps the panel widget via KWin's transient
    `application_menu("service","")` two-step emissions.
  - Wire format MUST match Qt's exporter byte-for-byte: layout struct is
    `(i, a{sv}, av)` with EACH CHILD VARIANT-WRAPPED (`av` of `v`), and the
    GetLayout reply's second out arg must be the STRUCT itself, not a variant
    (`(u(ia{sv}av))`, NOT `(uv)`). KDE's DBusMenuImporter demarshals exactly
    that; anything else yields zero children → the applet brands us a
    "naughty app" (menuUpdated with empty menu → menuAvailable false).
  - dbusmenu GetLayout depth semantics: 0 = item only, 1 = item + one level
    of children (Qt's importer fetches depth 1), negative = all.
  - AboutToShow MUST return true when the subtree is stale (revision changed
    since the consumer last fetched that id): KDE's importer only refreshes a
    submenu on open when AboutToShow says so or the menu is empty — a constant
    false freezes item enabled/checked state at first import (Playback stayed
    grayed while playing).
  - Debugging path that worked: WAYLAND_DEBUG=1 on our unit (wire truth),
    dbus-monitor on destination=our-service (import attempts), plasmashell
    unit journal with WAYLAND_DEBUG (what the panel receives), busctl
    side-by-side GetLayout against a working app (Konsole) for format diff.

### Step 3 slice 1 — menu model + titlebar menu bar (landed 2026-08-24)
- **`src-tauri/src/menu.rs`**: Rust-owned model (Playback/Library/View/Help;
  ids `playback.play-pause/-stop/-previous/-next`, `library.rescan/
  add-files/add-folder/save-imports/choose-folder`, `view.theme` (checkable),
  `help.about`). Dynamic bits arrive via `set_menu_state` (MenuState:
  playing/hasTrack/scanning/anyStaged/themeDark — pushed by an $effect in
  App.svelte from the stores); every push re-emits `menu-changed` with the
  rebuilt model. `menu_activate` = ONE activation path: playback ids handled
  natively via the engine (toggle/stop/jump), everything else re-emitted to
  the frontend as `menu-action`. 3 cargo tests (label gating, scan/staged
  gating, checkable theme).
- **`src/lib/stores/menu.svelte.ts`**: mirrors the model (`menu-changed`),
  dispatches `menu-action` to scanner/ui stores, `activateMenuItem()` for
  rendered items. **TitleBar.svelte**: text menu bar after the traffic lights
  (hamburger later REMOVED — Global Menu XOR titlebar bar); classic
  hover-switches-while-open behavior, ✓
  checkmarks, disabled gating, Esc/focusout close.
- Gates: check 0/0, vitest 22, cargo 38, build OK; menu bar visible in
  screenshot.

**Window identity** (so the widget renders icon + "Songstress" like Konsole):
- ✅ User-level desktop entry `~/.local/share/applications/com.yossi.songstress.desktop`
  with `StartupWMClass=songstress` (resourceClass is the binary name — AGENTS.md);
  desktop-file-validate clean, kbuildsycoca6 rebuilt.
- ✅ Icons: 128x128 + 256x256 (from 128x128@2x) → `~/.local/share/icons/hicolor/.../apps/songstress.png`;
  gtk-update-icon-cache run. (Icon art itself = Tauri placeholder for now.)
- ✅ Window title stays STATIC "Songstress" (DECIDED — no now-playing title).
- ✅ Verified: panel renders icon + "Songstress" + Playback/Library/View/Help
  (screenshot ident2). Step 3 COMPLETE — user verified Global Menu activation
  (Playback items track play state).

---

## Step 4 — Accent color picker ✅ (2026-08-24)

- ✅ Sidebar gear popover: preset swatch row (stock purple #a78bfa/#7c58f0 +
  curated set working in both themes: magenta, red, orange, amber, green,
  teal, cyan, blue) + native `<input type="color">` custom + reset-to-default.
  (DECIDED: presets + custom.)
- ✅ Store ONE hex in settings (`accentColor`, existing write-through).
- ✅ Effect in App.svelte watches theme + accentColor → inline overrides on
  `<html>` (pure math in src/lib/accent.ts, vitest-covered: HSL round-trip,
  lightness clamps dark [0.70,0.80] / light [0.45,0.60], WCAG luminance
text flip, theme alphas 0.18/0.14 preserved): `--accent` (light theme = darkened variant, dark = lightened —
  small HSL lightness clamp), `--active` (accent at theme alpha 0.18 dark /
  0.14 light — user-tuned, preserve), `--accent-text` (white/black by
  luminance).
- ✅ No app.css token changes; unset = stock purple everywhere.

---

## Step 5 — Playback controls package ✅ (2026-08-25)

### 5a. Shuffle & Repeat ✅ (landed 2026-08-25)
DECIDED: shuffle 4 stages (disabled / album / artist / all artists); repeat
3 stages (disabled / album / track); reshuffle on wrap; stages PERSIST.
- ✅ Generalize `PlayState` to an ordered `(trackId, path)` play order:
  shuffle off = album order (today's behavior); album = shuffled album
  (clicked track first, remainder shuffled); artist/all = pool from DB
  (current track's artist across albums / whole library), shuffled.
  Current album for gradients/panel = playing track's album; `{albumId,
  trackIndex}` event contract UNCHANGED.
- ✅ Repeat track = mpv `loop-file=inf` (native, cleared on stage change).
  Repeat album = restart order at 0 on last-track eof; RESHUFFLE on wrap
  when shuffle active.
- ✅ PlayBar buttons LEFT of the speaker icon: shuffle (4 stages, badge dot +
  tooltip) and repeat (3 stages, "1" overlay for track).
- ✅ Persist stages in settings; MPRIS `Shuffle`/`LoopStatus` become real
  (currently hardcoded false/"None"); Playback menu checkable items.

### 5b. Prev/next album buttons ✅ (landed 2026-08-25)
DECIDED: global grid order (artist A→Z then year), wrap-around, no-op when
stopped. Frontend-only: compute adjacent album, call existing `play_album`
at track 0.
- ✅ PlayBar center cluster: `[⏮album] [prev] [play/pause] [next] [⏭album]`.
- ✅ Previous/Next album items in Playback menu (frontend-handled).

---

## Step 6 — Equalizer ✅ (landed 2026-08-25)

DECIDED: 10-band graphic + preamp; controls BOTH in playbar popover AND
Playback menu; lands after the playback controls package.
- ✅ `playback_eq` command `{ enabled, preampDb, gains[10] }` → lavfi chain:
  `volume=<preamp>dB` + 10 chained `equalizer` filters (octave-width peaking)
  → `set_property("af", ...)` live on mpv (gapless-safe, composes with
  ReplayGain). Disabled/flat → clear `af`.
- ✅ Pure filter-string builder fn + unit tests (band mapping, ±12 dB clamp).
- ✅ Persist (settings JSON: enabled + preamp + gains + preset name);
  re-apply on launch (Rust re-applies to mpv; frontend hydrates UI from
  settings — DB wins over the localStorage mirror, like ui.* settings).
- ✅ PlayBar: EQ button → glass popover: preamp slider, 10 vertical band
  sliders ±12 dB (double-click zeroes), preset dropdown, on/off toggle;
  divergence → "Custom". NOTE: vertical sliders are horizontal ranges
  rotated -90° in a fixed-size slot — WebKitGTK 2.52 ignores
  writing-mode/direction on range inputs (thumbs never move).
- ✅ Playback menu: no submenu support in the menu model → flat items
  appended to the Playback menu instead: "Equalizer: On/Off" (checkable),
  "EQ Preset: <name>" (cycling item, gated on enabled), "Customize
  Equalizer…" (opens the popover).
- ✅ Presets: Flat, Rock, Pop, Jazz, Classical, Electronic, Hip-Hop, Bass
  Boost, Treble Boost, Vocal Boost.

---

## Step 7a — "Play next" queue ✅ (landed 2026-08-25)

DECIDED (2026-08-25): queue lives in Rust `PlayState` (Rust owns playback
truth). Queue entries play BEFORE the album order resumes; the queue itself
is NEVER shuffled; session-only (no persistence). MPRIS Next/Previous
RESPECT the queue (user call).
- ✅ `PlayState.queue: VecDeque<OrderItem>` + `from_queue` flag. eof decision
  extracted to PURE `eof_advance(&mut PlayState) -> Option<EofAction>`
  (unit-tested): promote queue front into `order` at the current slot
  (events/MPRIS metadata stay correct), consume it at its own eof, NO window
  top-up on promotion (mpv consumed a queue entry — the order window didn't
  slide), normal top-up on consumption. Consuming the LAST order entry:
  promote the pre-armed wrap pass or stop. `load_queue` rebuilds the tail as
  [current, queue…, order window, (wrap pass)] — queue entries appended
  between current and window, armed pass re-appended (single append path).
- ✅ Interplay: repeat=track untouched (native loop); repeat=album wrap arms
  only when the queue is empty (`pre_arm_if_last` + set_repeat guard);
  `arm_wrap` excludes a playing queue entry; shuffle reshuffles the order
  only; `jump(+1)` consumes/promotes queue entries (falls into the order
  when drained), `jump(-1)` from a queue entry returns to the order entry
  before it; album click (`play_order`) and stop clear the queue.
- ✅ Context menus: track → "Play next" / "Add to queue"; album header →
  "Play album next" (non-missing tracks, album order).
- ✅ Commands: `playback_queue {trackIds, front}`, `playback_queue_remove
  {pos}`, `playback_queue_jump {pos}` (+ `tracks_by_ids` helper: input order
  preserved, per-album canonical album_index). PlayBar queue button
  (badge = count) → glass popover: queued rows (click = play now + discard
  earlier entries, × = remove) + dimmed "UP NEXT" order preview (3 entries).
  Queue mirrored via `queue-changed` {queue, upNext}.
- ✅ MPRIS Next/Previous respect the queue for free (they call `jump`).
- ✅ Tests: 8 new eof_advance tests (promote/consume/chain/top-up/pre-arm/
  wrapped-promotion/stop-clears/never-stops-with-queue) — caught a real
  out-of-bounds edge (queue entry as last order entry). Gates: check 0/0,
  vitest 53, cargo 56, build OK. Live-verified: popover renders real
  queue-changed data during playback.

## Step 7d — Live library watching (after 7a) ✅ (landed 2026-08-25)

DECIDED (2026-08-25): `notify` v6 (inotify), one recursive watcher per root,
only while the app runs. Events → 2s debounce → incremental rescan
(reuses SCAN_RUNNING via a dirty flag, rescan once after). Minimum 5s
between rescans so bulk copies don't thrash. Silent auto-refresh via the
existing scan-progress/scan-finished events; staged imports need no
special-casing (they end in a scan). Network mounts unsupported (inotify is
local-only) — manual Rescan remains; document it.
- ✅ `watcher.rs`: notify recommended_watcher on a named thread; any event
  kind counts; quiet-period debounce (recv_timeout loop); failing inotify
  (network mount) logs and disables watching. Unit test: burst coalesces to
  one callback (real FS events).
- ✅ lib.rs wiring: watcher callback only sets WATCH_DIRTY; a 500ms polling
  async loop scans when dirty AND scan idle AND ≥5s since the last one —
  events during a scan keep the flag set → exactly one follow-up scan.
  Watched root = the one at launch (a folder change resumes watching after
  the next app start — documented).
- ✅ Verified live: touch → "[watch] library changed on disk — rescanning"
  → [scan] 61ms / 4416 skipped / 0 changed. Gates: cargo 57 (incl. watcher
  debounce test), check/vitest/build unaffected (no frontend change).

## Step 7c — Multi-library roots (after 7d)

DECIDED (2026-08-25): migrate `musicDir` → `musicDirs` (JSON array), reading
old `musicDir` as migration fallback on first launch. Scanner loops all
roots (blake3 IDs → no cross-root collisions; the deletion sweep's
"remove paths that NO root contains" already handles root removal).
Library menu → "Music folders…" glass modal: roots list, Add (kdialog
picker) / Remove per row; removing a root drops its tracks from the library,
files NEVER touched; removing the last root returns to EmptyState. Staged
imports keep targeting the primary root; folder picker starts at the most
recent root → ~/Music. Tests: multi-root merge, orphan sweep on root
removal, migration.

## Step 7b — Packaging (RPM) — HOLD: check with user before starting

DECIDED (2026-08-25): `tauri build` → native RPM (dnf install/remove).
AppImage rejected (not native to Fedora/KDE, no package-manager
integration). Release profile: lto="thin", codegen-units=1, strip=true.
Bundler generates the system desktop entry + icons; remove the user-level
~/.local/share/applications/com.yossi.songstress.desktop after install.
Binary name MUST stay `songstress` (resourceClass + dev unit). Data stays
at ~/.local/share/com.yossi.songstress (settings/library survive the
dev→packaged switch). Version 0.1.0. NOT in scope: auto-update, Flathub,
external repo. Verify: install → krunner launch → Global Menu + MPRIS +
blur → dnf remove clean.

---

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
  and check with the user first.

## Environment quick facts (for fresh sessions)

- Dev unit: `systemctl --user restart songstress-dev`; logs
  `journalctl --user -u songstress-dev -f`. HMR rot → cold restart first.
- Raise window: WindowsRunner DBus Match + Run(matchId, "activate")
  (qdbus-qt6, `--literal` for Match output; matchId like `0_{uuid}`).
- playerctl NOT installed — verify MPRIS via busctl/gdbus/qdbus-qt6.
- grim NOT installed; screenshots via `spectacle -b -n -a -o <file>`
  (active window) / `-f` fullscreen; background captures for artifacts.
- Display 3840×2160 @ 1.6 scale (fractional — relevant to 0b).
