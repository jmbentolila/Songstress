# Library pane, rebuilt (2026-08-31) (plan archive)

Moved out of `PLAN.md` on 2026-10-24 so the plan file stays readable in one pass.
This is the record, not the state: current state lives in `PLAN.md`, and this
file is the evidence behind it. Sections are kept verbatim, in their original
order. The index of every section is `README.md` next to this file.

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
- ~~**Per-album sidecar file** (owner idea, 2026-09-26)~~ — **SHIPPED** the same
  day as `.songstress.json`; the format, the adoption rules and the shared-folder
  finding are in "Per-album display state travels" below. The open questions
  this note used to carry are answered there: the DB stays the source of truth
  at runtime with the file as the portable half (a scan adopts, an in-app change
  mirrors), one file per FOLDER with a map of album id → state, and an album
  spanning several folders keeps its entry in the folder holding most of its
  tracks (the same rule-1 cascade the import destination uses).

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

