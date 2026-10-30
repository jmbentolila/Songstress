# Library gotchas (scan, watcher, tags, album state)

The measured detail behind the library rules in `AGENTS.md`.

Moved out of `AGENTS.md` on 2026-10-24 to keep the injected file under
the 20,000-char context cap. Rules stay in `AGENTS.md`; this file carries the
measurements and the reproduction.

- **Debug Rust panics on integer overflow** and a panic inside a Tauri command
  aborts the whole app. Use `u16`/`saturating_*` for pixel math (see the `+8`
  bucket-center incident in git-less history / PLAN.md).
- **Covers live in `public/covers/`** (plain Vite — `static/` is a SvelteKitism).
  `tauri.conf.json` `frontendDist` is `../dist`.
- **lofty 0.22.4's `write_id3v1` PANICS truncating legacy fields** — it cuts
  title/artist/album at 30 bytes and comment at 28 with `split_at` (a BYTE
  slice), so any v1 field whose cut lands inside a multi-byte char kills the
  save task mid-write (owner data: a cp1251 ID3v1 comment, album Road To The
  Unknown). The message reads `end byte index 28 is not a char boundary` —
  blame the ID3v1 WRITER, not the string's reader (the first diagnosis here
  fingered parse_ini; wrong — see PLAN.md 2026-09-05). `tags::write_file` is
  armored: temp+rename under catch_unwind, char-boundary pre-trim of v1
  fields, retry-once-with-v1-dropped. Upstream fix is lofty ≥0.23.
- **A cancelled kdialog folder pick stores JSON `null` as musicDir.** `music_root`
  must treat null/empty as unset (→ ~/Music) — the old `unwrap_or(v)` made the
  scan root a RELATIVE path `null`, so every scan failed instantly and silently
  ("music root null does not exist"): no progress events, no error, menu clicks
  "did nothing". Scan command errors only reach the webview console.
- **Scan errors are near-invisible — but not any more, on one condition.**
  `scan_inner` now emits `scan-finished` on EVERY path, carrying `error: Some(..)`
  (2026-09-02), and the empty state renders that string in the caution hue. Before
  it, the emit sat below a `?`, so a failed run announced itself only in the
  webview console — and because `scan-finished` is the SOLE thing that clears
  `library.scanning`, a failed FIRST scan shimmered its skeleton forever. Any new
  early-return in that function must still emit. The journal side is now greppable:
  `[scan] FAILED: <reason>` (and `[scan] files …` for success). Remember
  incremental scans SKIP unchanged files entirely (they never re-enter grouping).
  After changing grouping/tag-consensus logic, use Library → Full Rescan (rebuild)
  or nudge `tracks.mtime_ns` in the DB.
- **The inotify watcher must ignore read traffic.** notify's inotify backend
  watches IN_OPEN too, so the scan's own walkdir reads arrive as
  Access(Open)/Access(Close(Read)) events — counting them made the watcher
  feed on itself (one 60ms rescan every 5s, 24h straight; 7723 in a day).
  Filter through `watcher::marks_library_dirty`: Access is noise; Create/
  Remove/Name/Data/Metadata count (a plain `touch` = Metadata, and the scan
  keys on mtime+size). Also: DROPping a WatchHandle stops its thread (the
  channel Disconnect IS the stop signal) — `rewatch` relies on exactly that.
- **A track's ALBUM is grouping output, not identity.** A retag rewrites the
  file, the scan re-groups it, and the row moves to another album while its
  PATH (and so its id, `stable_id("tr", path)`) stays put. So never resolve "the
  playing track" through `playback.current.albumId` — `currentTrack()` looks the
  `trackId` up library-wide via `library.trackById`, and `reanchorCurrent()`
  re-points the context after each dump (`library.dumpVersion`, wired in
  App.svelte). Resolving through the album is how the playbar read "Nothing
  playing" over audio that was still going (2026-09-26). The engine is
  unaffected: mpv's order is materialized by path at album start.
- **A path write moves the id with it.** A track id IS `stable_id("tr", path)`,
  which is the id a scan derives for that file, so a relink that wrote only
  `path` left the old id behind and the next scan insert for that path died on
  the PRIMARY KEY — the upsert only handles `ON CONFLICT(path)` (relink fixed
  2026-10-30; the row write is `relink_row` in `lib.rs`, derived from the exact
  stored path).
- **`.songstress.json` is per-album display state, not library content.** One
  file per FOLDER, a map of album id → `{colors, gradient}` (a flat folder can
  hold several albums — measured: three album tags in one compilation folder).
  A SCAN adopts it (gradient always wins; `colors` only fills in when there is
  no artwork to derive from), while an in-app change MIRRORS the DB into it —
  `sync_album`/`sync_entry` take an explicit `adopt` flag, and getting that
  direction backwards silently reverts the user's edit (it did, once). The
  watcher ignores the filename: without that, every colour edit schedules a
  rescan. Per-album gradient overrides are settings keys
  `albumGradient:<album id>` (one per album, not the retired `panelGradients`
  blob), written together with the file by `set_album_gradient`, and re-read by
  `refreshPanelGradients()` after each scan. Never write into an album folder by
  any name other than this one (plus its own temp) — see the 2026-09-04 rule.
- **Tag editor suggestions are DB-backed, and the DB only knows what a scan
  stored.** The seven completable fields read `tag_vocabulary`
  (`library/vocab.rs`, migration v6), so `genre`/`composer`/`label`/`grouping`
  return **nothing until a Full Rescan** has backfilled the columns —
  incremental scans skip unchanged files, and a tag SAVE only fills its own row
  (mtime changed). Don't "fix" an empty genre list by re-reading files at editor
  open: that was the design the schema route replaced. Also: counts arrive from
  the DB but do NOT rank the list (closeness does: earliest inclusion, then
  shortest value — owner ruling); the **first row is always armed**, even when
  it equals the query (type `dawn`, the library holds `DAWN`: completing must be
  able to give back the stored spelling), so with a list open the first Tab/Enter
  completes and the second moves on / saves; Tab/Enter complete only a
  **highlighted** item (Enter
  otherwise stays Save); Escape closes the list, not the modal; and the list is
  `position: absolute` inside the field cell because a `fixed` child of a
  `.glass` panel anchors to the PANEL (backdrop-filter is a containing block)
  while the modal body also clips. Ranking rules live in `lib/suggest.ts` with
  their tests — change the order there, not in the component.


