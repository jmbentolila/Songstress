# Per-album display state travels: `.songstress.json` (2026-09-26, owner ask) (plan archive)

Moved out of `PLAN.md` on 2026-10-24 so the plan file stays readable in one pass.
This is the record, not the state: current state lives in `PLAN.md`, and this
file is the evidence behind it. Sections are kept verbatim, in their original
order. The index of every section is `README.md` next to this file.

## Per-album display state travels: `.songstress.json` (2026-09-26, owner ask)

Owner's goal: "the important thing is that the user is able to easily go from one
device to another and keep its library having the exact same behavior inside
Songstress". Per-album state lived only in the SQLite `settings` table, keyed by
album id — exactly what a fresh install does not have — so the portable half is
now a file in the album's own folder.

**Shape (v1), one file per FOLDER, one entry per album:**

```json
{
  "version": 1,
  "albums": {
    "al-cac01388c68b989f": {
      "colors":   ["cee4de", "086ba0"],
      "gradient": ["cee4de", "086ba0"]
    }
  }
}
```

- **Why a map and not one object per file:** a folder can hold several albums.
  Measured in this library: `~/Music/Compilations/Proyecto Anison Latino` is a
  flat folder whose files carry THREE different album tags (11/14/2 tracks), so
  the scan correctly groups three albums that all point at one folder. A flat
  object had them overwrite each other (251 albums → 249 files, last writer
  wins); the map keeps an entry each. The album id is the key because it is
  tag-derived and therefore stable across machines.
- **`gradient`** (the panel override, mirroring the new per-album setting
  `albumGradient:<id>`) is a real override: **on a scan the file wins, always**.
- **`colors`** (the artwork-derived pair, mirroring `albums.color_c1/c2`) is a
  **fallback only**: the artwork stays the live source, so picking new art and
  pressing "use album colors" derives from the art you actually loaded, and the
  file's pair answers only the case where there is nothing to derive from
  (copied library, lost `folder.jpg`, unreadable format). Delete the key to let
  the app re-derive. The owner's refinement, after the first cut had it win
  outright: "keep that value for the specific case of missing artwork".
- **Direction is explicit in the code** (`sync_album`/`sync_entry` take `adopt`).
  A scan adopts; an in-app change mirrors. The first implementation re-adopted on
  the write path, so setting a gradient wrote the new value to the DB and then
  put the OLD one back into both — caught live, regression-tested.
- **Per-album overrides moved out of the `panelGradients` blob** into one key
  per album, migrated once at `init_settings` (never clobbering a key a sidecar
  already adopted). A blob would have let the app's pre-scan in-memory map wipe
  an adopted value on its next write. The frontend re-reads those keys after
  every scan (`refreshPanelGradients`).
- **Written for every album's folder**, version-only when there is nothing to
  carry ("just in case", owner) — and never rewritten when the bytes match, so
  the file's mtime does not churn. Stale entries (album retagged/removed) are
  pruned by the owner of that folder's file.
- **The inotify watcher ignores it** (`marks_library_dirty` → all paths ours →
  not a library change). Without that, every gradient edit would schedule a
  rescan and the scan's own writes could re-arm it — the same self-feeding shape
  as the read-traffic bug. Verified live: no scan in the 12s after an in-app
  gradient write.
- **Multi-folder albums** get ONE entry in the folder holding most of their
  tracks (rule-1, the import-destination cascade). Measured: `Nightfall In
  Middle-Earth` spans a regular and a special-edition folder; `Anison no Kokoro`
  spans a Downloads folder and a Music one.

Live verification in the dev instance against the real library: first scan wrote
249 files over 251 albums (the shared folder explains the difference), carried
the owner's pre-existing DAWN override out of the legacy blob into the file, the
shared folder came out with three entries, an in-app `set_album_gradient` moved
only that album's `gradient` key (colors untouched), and a second scan wrote 0.

Gates: cargo 133 (7 sidecar tests incl. the shared-folder and pruning cases),
vitest 125, svelte-check 0, build ok.

### The playbar survives a retag that moves the playing row (2026-09-26, owner report)

Symptom, after importing and playing: edit the tags of the playing file so it
belongs to a different album, and the audio keeps going while the UI goes blank —
playbar reads "Nothing playing" (no gradient, no details, no active transport),
and the destination album's rows show no playing state.

Cause: `tracks.album_id` is grouping output, not identity. A retag rewrites the
file, the watcher's incremental scan re-groups it, and the row moves to another
album — while the PATH, and therefore the track id (`stable_id("tr", path)`),
stays. The frontend resolved the playing track through
`library.tracksOf(playback.current.albumId)[...]`, i.e. through the album the
engine had named when playback started. `tracksOf(stale)` is `[]`, so
`currentTrack()` returned null and every surface that hangs off it (playbar
details, artwork gradient, `.cover.ring` on the album tile, the active state on
track rows) went quiet at once — including the destination album's rows, which
had no reason to light up because nothing said the track lived there now.

Fix, all frontend (the engine is fine — see below):
- `library.trackById(id)` — a `#byId` index built with every dump, so a row can
  be found library-wide in O(1) (`albumOfTrack` rides on it).
- `currentTrack()` resolves the context's `trackId` **library-wide first**, and
  only falls back to the album's own list for a context that never had a track
  id (the note this replaced still said "trackId is authoritative", which was
  true only inside the album it started in).
- `reanchorCurrent()` points `playback.current` at the row's new home
  (album id + index), so everything that reads `albumId` stays coherent —
  queue building, the fake-mode advance, the marquee key. Called from the
  engine's own `playback-changed` events (their album label can go stale too)
  and from an App.svelte effect watching the new `library.dumpVersion`, which is
  the moment a moved row actually lands.

Verified live in the dev instance, by moving a row between albums in the DB and
forcing a dump (no files touched): playing "MOON RIVER -prologue-" out of DAWN,
moving its row to `Impera`, then scanning — the playbar kept the title and the
Pause state, and the playing ring MOVED to `Impera`. Moving it back put the ring
back on DAWN with the playbar untouched. 4 new vitest cases pin the rule
(library-wide resolution, re-anchor, unknown id left alone, no-id fallback).

Why the engine needs no change: mpv's play order is materialized at album start
(a list of album_id + path), so the currently-playing file and the following
tracks keep playing — the file did not move, only its album label did. Two
adjacent cases remain, both unverified and out of scope here: moving a file on
disk while it plays (import Save re-points the row to a NEW id in a NEW path, so
the engine's stored path goes stale and there is no local old→new mapping), and
a retag that renames the artist so the album's own id changes — the row
survives, but a *queue* built earlier still names the old album.

Gates: svelte-check 0, vitest 129, build ok (no Rust change, so no cargo run).

### The import landing targets the batch, not the pending pile (2026-09-26, owner report)

The landing itself is wanted and stays: an import switches the artist tab and
expands the album it staged ("the landing IS the receipt", 2026-09-24). What was
wrong is WHERE it landed. `importMusic` used `imports.plan[0]?.albumId` — the
first entry of the whole pending pile (`staged_import_plan`, DB order) — so
importing album B while an older album A was still awaiting import landed on
**A**, an album the user had not touched. The owner's rule: "it should land on
the first item of the most recent import batch".

- `ImportReport.staged` already carried only this batch, but as
  `{artist, title, tracks}` — no id, so the frontend could only have matched by
  artist+title, which picks the wrong album when two share a title.
  `ImportedAlbum` now carries `album_id` (lib.rs passes the same structs through;
  the only construction site is `album_groups`).
- `album_groups` now orders by the FIRST path that names each album rather than
  alphabetically: the report is a receipt for the files the user picked, and
  "the first album of this batch" means the album of the first file picked. (The
  receipt list in the modal reads in picked order now too — a side effect worth
  having.)
- `landingAlbumId(report)` (importPlan.ts, 4 vitest cases) is the whole rule:
  first staged album of the batch, else null so the grid stays put — the
  existing "everything was already in your library" fallback (which uses
  `report.already`, also this batch) is untouched, and `imports.plan` still
  holds every pending album for the modal and the badge. Only the landing moved.

Gates: cargo 134 (new `album_groups` order/id test), vitest 133, svelte-check 0,
build ok. Live verification of the landing would need a real import action (the
unit tests cover the decision); not exercised in the dev instance.

Note for the record: writing the new test appended to `src/lib/importPlan.test.ts`
initially CLOBBERED that file (it already existed with 16 tests — writing tool,
not git, so the restore came from HEAD). Caught by the test count dropping from
129 to 117, restored from `git show`, then appended: 20 tests in that file, 0
deletions in the diff. Check for an existing file before writing one.

### Artwork delete: resilience, the spinner, and the surface it drops on (2026-09-26, owner-driven)

Four refinements landed together after the first end-to-end use of the delete:

- **A file the tag writer cannot read is SKIPPED, not fatal.** The first version
  walked the album's files and aborted on the first refusal, which surfaced as a
  raw error *and* left a silent partial delete. `ArtDeleteReport.skipped` now
  carries the paths, the reason goes to the journal (`[art] could not strip ...`),
  the delete continues through the rest, and the panel says it in plain words
  ("2 files could not be updated (unreadable tags): <name>") beside the
  announcer's "removed it from 21 files". Re-running the same delete is safe:
  already-stripped files report "nothing to do".
- **The trigger for that: `Barairo no Sekai (GetBackers).mp3`** in *Anison no
  Kokoro*. Measured with a byte dump: one ID3v2.3 tag (186,368-byte payload)
  ending at 186,378, then **1,044 bytes of NUL**, then the first MPEG frame
  (`FF FB E0 04`) at 187,422. `file(1)` agrees with lofty — it reports only the
  ID3 header. This is NOT the stacked-tag shape the existing repair handles
  (that expects a second `ID3` header at the declared end), so that repair
  cannot fix it. **Owner decision: leave it** — the app skips it and the album's
  artwork stays editable on the other files. A "find the first frame after the
  tag and rewrite" repair was offered and declined.
- **The in-flight Delete button is a spinner, not a word**, and the label never
  leaves the DOM (it is hidden): the button's width is the LABEL's width in both
  states, so the row cannot twitch. Measured 75px → 75px. `ProgressRing` stays
  determinate on purpose (a scan says how far it got); a delete is a short
  indeterminate verb, which is the one case a spinner is honest about.
- **The confirmation drops from the INNERMOST surface that raised it** — the
  modal normally, the lightbox when the expanded view asked (a box on the panel
  behind the image read as a bug). Inside the lightbox there is no veil: its own
  0.7 dim already is the dim. Escape and a backdrop press both resolve the
  confirmation before the surface itself (capture-phase Escape, since the modal's
  listener is on `window` too). The general rule is in DESIGN.md ("The In-Modal
  Confirmation Rule") and AGENTS.md.

Gates: cargo 135, vitest 133, svelte-check 0, build ok.

---

