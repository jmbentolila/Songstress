# Tag Editor Redesign (plan archive)

Moved out of `PLAN.md` on 2026-10-24 so the plan file stays readable in one pass.
This is the record, not the state: current state lives in `PLAN.md`, and this
file is the evidence behind it. Sections are kept verbatim, in their original
order. The index of every section is `README.md` next to this file.

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

## Tag editor suggestions: DB vocabulary + our own menu (2026-09-26, owner ask)

The tag editor had exactly one completion: a native `<datalist>` on the Album
field, fed from `library.albums`. It rendered WebKit's own popup (visibly not
our UI), and Tab could not take the highlighted suggestion — the browser only
accepts on Enter/click. The ask was three things: suggestions on **Artist,
Album, Album artist, Genre, Composer, Label and Grouping** wherever they appear;
**Tab completes the selected suggestion**; and the list wears **our context
menu's surface**.

**The blocker that shaped the design: the DB never stored those tags.**
`tracks` held `id, album_id, disc, track, title, duration_sec, path, mtime_ns,
size, staged, artist` — genre/composer/label/grouping existed only inside the
files, read on demand by `tags.rs` when a modal opened. Building a suggestion
list by reading tags would cost a full lofty pass per editor open. Owner chose
the schema route over an on-demand read/cache:

- **Migration v6** (`db.rs`): nullable `genre`, `composer`, `label`, `grouping`,
  `album_artist` on `tracks`. Additive, so it auto-upgrades — the same class as
  v3 (`tracks.artist`), which shipped without a major bump. **Nullable matters**:
  incremental scans skip unchanged files, so pre-v6 rows read NULL until a
  **Full Rescan** (or any tag save, which changes mtime) backfills them. Until
  then the four tag fields simply offer nothing — honest, not broken.
- **The scan persists what it already read** (`scan.rs`): `genre` via the
  accessor, `composer`/`label`/`grouping`/`album_artist` via `ItemKey` — the same
  keys `tags::write_file` writes, so the round trip is exact. The insert row was
  a seven-wide tuple at the point this started and would have hit twelve, so it
  became the `TrackRow` struct.
- **`tag_vocabulary`** (`library/vocab.rs`): distinct values per field with
  counts, capped at 400. Values are trimmed, blanks dropped, **ASCII case
  variants fold into one suggestion wearing the most-used spelling** (ties go to
  the ASCII-first spelling, so the order is reproducible), and **diacritics stay
  distinct** — `Motörhead` and `Motorhead` are two suggestions because they are
  two spellings the user may hold on purpose; their *matching* still folds, so
  typing `motor` finds both.
- **Ranking is pure** (`lib/suggest.ts`, 21 vitest cases): folded prefix beats
  folded substring, and nothing else counts — no subsequence fuzz, because a
  field that offers `Meliora` for `mlr` is guessing about the user's own data.
  An **exact folded match leads its class ahead of everything**, including the
  preference list: owner report the day it shipped — typing `DAWN` offered
  `Dawn of Victory` first (more tracks, so more popular) and the highlight rule
  then armed Tab/Enter to replace a correct value with a different album. Then
  `prefer` (the album door's same-album-artist-first order, carried over intact
  from the datalist), then **closeness: earliest match position, then shortest
  value**. Popularity is not part of the order at all (owner ruling the same
  day: "it's not about popularity") — the counts still travel from the DB, but
  they only describe the library and no longer decide what the top suggestion
  is. Ranking by how much of the library a value happens to name made the list
  depend on library size instead of on what was typed.
- **Keyboard contract (owner-set, twice: 2026-09-26)**: ↓/↑ walk the list;
  **Tab completes the highlighted value and keeps focus** (a second Tab then
  moves on — the trap and the accelerators live on `window` in TagSurface, so a
  handled key stops the event there); **Enter completes only when something is
  highlighted, otherwise it stays the editors' Save**; Escape closes the LIST,
  not the modal; picking is on pointer-down with the default prevented so the
  input never blurs. **The first row is always armed**, including when it equals
  the query: the list is the library's answer to what was typed, and the case,
  decoration or spacing the DB holds is precisely what completing should give
  back (owner case: type `dawn`, the library holds `DAWN` — refusing to highlight
  it left no keyboard way to take the stored spelling). Ranking puts an exact
  match first, so Tab/Enter take THAT spelling; a second Tab then moves on, and
  with a list open the first Enter completes rather than saves.
- **Surface**: the context menu's rules moved to app.css
  (`.menu-surface`/`.menu-item`/`.menu-sep`) and its entrance to
  `lib/menuPop.ts`, and both the menu and the list now use them — "looks like
  our menus" is true by construction instead of by two copies of the numbers.
  The list is `position: absolute` INSIDE the field cell: a `fixed` child of a
  `.glass` panel anchors to the **panel**, because backdrop-filter is a
  containing block, and the modal body both scrolls and clips. It flips above
  the field when it would run past the panel's visible bottom.
- **Values in browser dev** fall back to the fake library's knowledge (album
  titles, artist names) with the four tag fields quiet — no command exists there.

Gates: cargo 123 (incl. an end-to-end test that tags a real fixture file, scans
it, and reads the vocabulary back), vitest 117 (13 new), svelte-check 0, build
ok. Live-verified in the dev instance through the devtools bridge: the list sits
exactly under its input (same left/width, +4px), prefixes outrank substrings,
`aria-expanded`/`aria-activedescendant` track the highlight, Tab commits and
keeps focus, Escape closes the list then the modal, and `tag_vocabulary` returns
the real library's albums/artists (the four tag fields read 0 until a rescan).

### Marquee parks at the loop point, not mid-roll (2026-09-26, owner ask)

Pausing used to freeze the playbar marquee wherever it stood — `.playbar.paused
.mq-in { animation-play-state: paused }` — which cut a long title off mid-word
and left it parked on a half-scrolled string. The owner's ask: on pause/stop the
line should **finish rolling to the next dwelling point and stop there**.

The action owns it now (`lib/marquee.ts`), because CSS cannot wait for a point in
a loop:

- The pause signal rides the action's parameter (`{ key, pause }` from PlayBar's
  `!playback.isPlaying`); `update()` compares values and only re-measures when
  the KEY changes, so PlayBar's frequent re-renders (progress, transport) cannot
  restart the roll.
- On pause it waits for `animationiteration` — the loop point, where the strip is
  back at the head and the keyframes hold it for the next quarter anyway — and
  parks there by adding the JS-owned `is-held` class the stylesheet reads. The
  landing spot is exactly where the cycle rests, so the parked line looks like
  every dwell.
- If the strip has not begun rolling yet (still inside its 1.2s arrival delay),
  it parks at once: the strip is already at the head, so rolling a whole lap to
  "finish" would be motion with no job. A text change while parked stays parked
  (the class outlives the re-measure); resuming unpauses in place.
- Cost, measured live: a 560px line (19.9s loop) kept rolling 17.6s after the
  click, then stopped at the head (`x = -1`, `is-over`/`is-held` true,
  `animation-play-state: paused`) and stayed there across further samples.
  **Worst case is therefore one loop** (≤ the 24s cap) — inherent to "finish the
  roll" on a very long line. If that ever reads as too long a tail, the lever is
  a cap on the remaining travel (park immediately when the loop point is more
  than N seconds away), not the mechanism.

Gates: svelte-check 0, vitest 125, build ok. No version bump of its own — this
rides the 0.14.0 batch with the tag-editor suggestions.

