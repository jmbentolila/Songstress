# Tag Editor Redesign (plan archive)

Moved out of `PLAN.md` on 2026-10-24 so the plan file stays readable in one pass.
This is the record, not the state: current state lives in `PLAN.md`, and this
file is the evidence behind it. Sections are kept verbatim, in their original
order. The index of every section is `README.md` next to this file.

## Tag Editor Redesign — Phase C: the track modal (2026-09-03)

- **The split.** `TagEditor.svelte` is a dispatcher now (trackId → track
  modal, albumId → album modal). The modal CONTRACT — backdrop, the
  `.scrim > .glass` pair, the `scrim-out` outro, Escape ownership with the
  lightbox veto, the focus trap, the `waiting` hold while fields load —
  lives in `TagSurface.svelte`, with the shared footer family in a `.te-*`
  namespaced global sheet (AGENTS.md: a class rendered across a file
  boundary is unscoped). Field rendering lives in `FieldGrid.svelte` —
  dispute chips, inline "of N" totals, the album datalist — driven by data,
  not by mode branches. The field model, census types and numeric
  validation moved to `lib/tagFields.ts`, the one file both editors agree
  on so the pair cannot drift.
- **The track modal** (`TrackTagsEditor.svelte`) is the surgical one:
  - *Identity row*: the file's name, its folder `~`-shortened (full path as
    the tooltip), and a reveal button — paths travel as DISPLAY strings
    only, revealing stays `reveal_container`'s job. Rust: `tags::track_file`
    + the `get_track_file` command (+1 test: name, album id, staged flag).
  - *Header stepper* `◂ n/N ▸` walks the album in display order
    (`library.tracksOf`, the panel's own order). It DISABLES while unsaved —
    it will not eat your edits silently — and a track save STAYS OPEN
    (footer reads `Written ✓`, the fields re-read from disk): that is what
    the walk is for, five fixes without the window ever closing.
  - *Retag door*: the Album field is a `datalist` of existing titles,
    same-album-artist matches first.
  - *Artwork is per-file*: the album's candidates, applied to this file
    (`ArtSelector` with `trackId` — the seam Phase A/B already carried).
- **The stays-behind caution**: retagging a LIBRARY (non-pending) file to a
  different album/album-artist moves the row, not the file — the modal says
  so in the caution hue and names the folder it stays in until an import.
  The clean-merge door (Phase E's picker) remains the pending-only path.
- **Same body as the album modal (owner ruling)**: the track modal wears the
  SAME two-column shape — `.te-cols` (artwork column left, stacked real
  previews, fields right in the single label+field rhythm). Only the APPLY
  target differs: this file, not the album. `.te-cols` moved into
  TagSurface's shared global sheet; `layout` on FieldGrid now selects ROWS
  only, the grid SHAPE is one rhythm for both.
- check 0/0 · vitest 84 · cargo 89/89 (+1) · build ok. Owner eyes the
  track-editor walk next (window re-centered by the cargo restart).
- Owner layout rulings, same day: (1) the track modal wears the SAME body
  as the album modal — `.te-cols` moved into TagSurface's shared sheet, and
  FieldGrid's `layout` selects ROWS only, the grid SHAPE is one rhythm;
  (2) small fields share rows — the grid is four shared tracks
  (label|field|label|field), one-field rows span the right side, and the
  pairs are Genre+Year (both modals) and Track#+Disc# (track modal);
  year/number inputs are fixed-width (`half`), "of N" totals capped at the
  same measure; (3) same day — the four shared tracks reserved a full column
  for a four-digit year, so pairs moved INSIDE the field cell (grid is back
  to label|field): genre stretches and pushes Year to the edge, the number
  pair packs left as one unit, and dispute chips render after the row's
  inputs so a wide chip never wraps the pair onto its own line; (4) the disc
  cell fills the row's remainder (split evenly, ends at the column edge);
  paired labels keep the SAME label→field distance the column labels have
  (grp label margin-right 4 → 10px total, probe: discGap/yearGap = 10);
  (5) **"More tag fields" fold on the track modal** (owner ruling): the
  rarely-touched Composer/Label/Grouping/Comment behind a disclosure row
  (the row is the button, chevron the indicator, instant swap — the
  ManageImports family), collapsed by default, and the header ACCOUNTS for
  the fold: "N fields filled" / "edited" — hidden state never silent.
  FieldGrid gained track-core/track-more layouts; the album modal keeps
  every field visible (the census promise — disputed fields may not sleep
  behind a fold).
  (6) SAME EVENING, critique-driven (impeccable critique of both modals,
  29→34/40; snapshots under .impeccable/critique/): the FIELD ORDER swapped
  to Album→Album artist; small-field geometry became MEASURED, not guessed
  (the grid observes the disc tail — label, box, "of", total — with one
  ResizeObserver, borderBoxSize only: getBoundingClientRect reads the modal's
  scale-in transform and pinned Year short; hidden "of" pairs claim no row
  room and must not be summed; every term exact layout px or the label sits
  0.12px adrift): the track modal's Year label stands under "Disc #" and its
  box under the disc box, the year box filling the mirrored cluster; the
  album modal's Genre|Year row is symmetric by the measured year-label width
  (--b = half of column minus label minus 16px separations; .te-grp needs
  border-box or the label margin inflates the set width). CORRIDOR: modal
  content is centered — body pays the 11px scrollbar gutter on the left too
  (overflow-y: scroll makes the gutter permanently REAL: WebKitGTK reserves
  ZERO with stable+auto while content fits), head/footer carry 13px margins.
  SELECTION dot filled with the KWin decoration green (--tb-a, live from
  decoration.svelte.ts; accent fallback for the pre-token frame). SHARED
  SURFACE additions in TagSurface: .te-topline (the promise line, first
  under the rule in BOTH modals — the bulk surface must never scroll its
  reassurance away), the .te-more fold family + slide transition (svelte/
  transition, prefersReducedMotion.current → instant, Svelte 5.9's is a
  reactive value, NOT a store, NOT callable), and the disclosure styles.
  ALBUM MODAL: orphan "N files" caption → FOOTER identity caption ("13 files
  associated to this album"; blast message takes the slot while dirty, same
  slot semantics as the track modal's path), and the SAME fold for
  Composer/Label/Grouping/Comment — with the census promise held: AUTO-OPENS
  when a hidden field is disputed, keyed on the DISPUTED FLAG, not on
  conflicts-membership (conflicts_for returns a census entry for EVERY
  field — membership test sprang the fold on every album; found by the
  second critique run's own probe). KEYBOARD (critique: Flexibility 2→3):
  quickKey prop on the surface — Enter=Save in both doors (form convention
  inside fields, native activation on buttons), ←/→ walk the track stepper
  outside text fields only; step() dismisses the artwork lightbox (probe:
  it lingered a caption behind a moved context). FOCUS: the surface captures
  its opener and restores focus on close (guarded: detached context-menu
  openers, body). ArtSelector hint tightened to "Click to enlarge · the dot
  marks the cover."
  (7) APP-WIDE AUDIT (impeccable, 16/20 Good; the two detector
  layout-transition warnings VERIFIED intentional — the .panel-slot margin
  rides the expansion's own reflow, documented in-component). The one real
  gap, fixed in 0.9.1: **status messages were silent** (WCAG 4.1.3) —
  src/lib/stores/announcer.svelte.ts + ONE visually-hidden
  role=status/aria-live=polite region in App.svelte, deliberately OUTSIDE
  the inert stage so a modal cannot mute it; fed by scan-finished (success
  AND failure), both tag editors' save paths (success and failure), the
  same sentences the visual layer shows; say() grows a space tail on equal
  consecutive messages because equal text = no node change = no
  announcement. ManageImports and the sidebar scan-note already own their
  polite regions — nothing double-announces. Verified by screenshotting the
  region with its sr-only temporarily stripped ("Library updated" — the
  watcher rescan after a file save, narrated). The two off-scale radii are
  on-scale (--radius-icon on the toggle box, --radius-pill on the scrollbar
  thumb — 5px wide, it was always a pill), and cover-overlay white is now
  var(--on-cover) (3 sites). Detector residue: only the two intentional
  warnings.

### Critique run — ExpandedPanel, 2026-09-04 (30/40 Good, first for this slug)

  The last unreviewed surface (sidebar/playbar/grid/import/tag-modals all had
  runs). One systemic finding, all four items fixed the same evening:
  **P1** the app's keyboard-focus family (accent outline — sidebar `.mrow`,
  import `.mi-disc`, tag inputs) stopped at this door: `.track`,
  `.staged-badge`, `.edit-album`, `.play-all` fell back to the UA's generic
  ring (observed live — two focus dialects on one screen); one four-selector
  rule joins the family, ring follows each control's own rounding.
  **P2** the Imported badge became a door in 0.7 but kept label geometry —
  padding-block 3→7px lands it at exactly 28.0px measured (PRODUCT.md bar),
  and per the cluster ruling it is now a QUIET sibling: transparent fill,
  text-dim, accent only on hover — Play is the header's only accent thing.
  **P2** missing-file alert `#f2a33c` → `var(--caution)` (token existed,
  value identical). **P2** state collision: `.track.selected` and `:active`
  shared the wash exactly, so press-on-selected showed nothing and a
  mid-press read as selection — selection is now wash + 1px inset `--border`
  ring (measured live: `inset 0 0 0 1px rgba(255,255,255,.1)`), the wash
  stays the press moment. Dev-only P3: header comment still promised the
  0.9.3-deleted arrival "flash". Detector: 0 findings. Snapshot:
  `.impeccable/critique/2026-09-04T00-05-34Z__src-components-expandedpanel-svelte.md`
  (scored BEFORE the fixes; the P1–P2s above are why it says 30).

### Playbar gradient crossfades, 2026-09-04

  The artwork backdrop teleported: one linear-gradient cannot interpolate
  into another (the old `transition: background 280ms` on .playbar could
  only ever lie). The gradient now lives in stacked `.pb-bg-l` opacity
  layers, one per gradient in flight: an album switch fades the NEW layer
  IN while the OLD fades OUT — both stacked fills, so the opacity
  crossfade IS the gradient morph (owner ruling after a first attempt that
  faded through chrome: reading it as off/on, and it was). Fade IN alone
  at first play / toggle-on, fade OUT alone at stop / toggle-off, old
  layers removed at the floor (300ms TTL, invisible-duplicate guard for
  skip-back-mid-crossfade, which also revives a layer whose gradient comes
  back). 700ms symmetric ease (cubic-bezier(0.4,0,0.2,1)) — the owner ruled the
  first cut (200ms) "much slower"; a slow dissolve is scenery, not
  feedback, and deliberately spends itself outside the 300ms state-response
  budget; a front-loaded curve would undo the slowness the duration buys.
  Reduced-motion killed by the stylesheet. State
  traps paid: mutate layers only through the $state proxy (raw-object
  writes notify nobody and the class never flips); `transition:fade`
  inside {#key} compiles with intro|outro flags and NEVER RUNS (verified
  live, cold reload) — CSS opacity cannot be skipped. Live traces: mount
  0.62→0.95→1; switch saw old 100→0 and new 0→100 SIMULTANEOUSLY with both
  gradients present in the DOM.

### find-animation-opportunities sweep + mirrored connective tissue, 2026-09-04

  The sweep's honest headline: most seams were already animated (panel
  contract, accordion, drawers, playbar dissolve, even the boot grid's
  .grid.enter row ladder — 320ms, 90ms/row, 540 cap, first 8 rows). What
  shipped is the leftover connective tissue, all of it MIRRORS (owner
  rule: every entrance owes its exit along the same path, same length):

  - ContextMenu: local transition, 125ms cubicOut, opacity + scale(0.97),
    transform-origin pinned by the clamp effect to the corner the click
    actually anchors (flip decides: left top / right bottom / …). Grows
    out of the click point, shrinks back into it.
  - EQ + queue popovers: the pop-in KEYFRAME becomes the popUp
    TRANSITION — 150ms cubicOut, opacity + 6px rise + 0.97 scale, origin
    bottom-right at the anchor button. Unfolds up from the button, folds
    back down. One dialect with the menu (same scale, same easing).
  - ExpandedPanel rows: transition:fade|local 160ms on the row BUTTON
    (not the <li> — the list is grid-auto-flow:column and an in-flow
    outro cell must not move; the fading button holds its slot, the gap
    closes as the deliberate second beat). Rows discarded/removed fade
    out in place; watcher-adopted rows fade in the same beat; `local:`
    keeps boot silent (the grid entrance owns it).
  - Lightbox: as-lb-in keyframe replaced by fade|local 140ms — entered
    on a fade, now LEAVES on one (it used to vanish).
  - Skeleton exit: the tiles rise in on library arrival; the shimmer now
    fades out over 200ms instead of blinking (wrapper div — transitions
    may not sit on component tags; grid-column: 1/-1 keeps its measure).

  Syntax tax paid: Svelte 5 modifiers are `fade|local`, not the old
  `local:fade`; JS-driven transitions read matchMedia/MediaQuery at fire
  time (the stylesheet kill switch cannot reach them). Live-verified:
  menu mid-flight opacity .71 / scale .991 / origin pinned to the click
  corner, outro in flight at +60ms; popover rising at .76, still mounted
  folding on close. Rejections logged with their gate: play/pause glyph,
  seek, stepper (100+/day), hold-to-confirm on discard/remove
  (recoverable, tens/day), tile hover, search re-stagger.

### The real-flow rehearsal happened, 2026-09-04

  The owner imported Ghost's Infestissumam (12 files) through the album
  badge modal with a real Save — the first end-to-end run of staging →
  saveImports → relink_track → album_destination against his library.
  Verified after: staged=0, import cache empty, all 12 files MOVED (birth
  mtimes intact) into `~/Music/Music Files/Ghost B.C./Infestissumam/`,
  folder created by the save itself, 12 rows re-pointed (play history
  kept), the following scan skipped 4464/added 0 — exactly what a clean
  relink looks like from the outside. Destination note: the files carry
  ALBUM ARTIST "Ghost B.C." (the 2011 release), so the layout rule put
  them under Ghost B.C., next door to the Ghost catalog — the split is
  the files' truth, not the app's. Every deferred verification item is
  now closed; nothing in the plan remains open.

### Settings persistence speaks when it fails, 2026-09-04

  Critique re-run's single P2, wired (owner: "who knows if someone else
  ends up using it"): pushSetting's `catch(() => {})` — the one place a
  failure stayed invisible FOREVER — now hands the announcer one sentence
  ("Could not save a setting — your change may not stick next launch."),
  and initSettings' catch says its own on Tauri only (browser dev keeps
  its silent localStorage fallback). No visual syntax for a failure mode
  nobody should see; just no more silent lying. Announcer has zero
  imports — no cycle. Re-runs persisted for all four surfaces (sidebar
  24→32, modals 34→36, panel 30→34, imports 30→35; audit ≈18/20, residue
  = the two intentional panel-slot warnings).

### The app icon becomes ours, 2026-09-04 — Version 0.9.7

  Shipped since day one: the TAURI DEFAULT LOGO. The revision ran the
  full dreaded loop — glyph families (monogram/note/wave), dioramas
  (mini-stage/sleeves), a generated-diva detour, potrace plans — and
  stalled on two diagnoses the owner named himself: too abstract (the S
  wore Tauri's bones) or too clunky (freehand beziers; the honest hand
  was mine). The reference he produced (a stencil mic with a face in the
  negative space) set the language: STENCIL DISCIPLINE — bold broken
  geometry where gaps are the message and the composition, not the
  curves, carries the mark. The keeper: dark stage (the app's actual
  material), user-accent glow, stencil mic tilted up-right with grille
  slits as true mask cutouts, voice arcs riding its axis into the light;
  reads clean at 32px. The hidden-profile cameo failed honestly (edge
  notches read as chips, not a face — the reference's face works because
  it IS the composition). Master: assets/app-icon.svg →
  `npx tauri icon assets/app-icon.svg` regenerates the full set;
  hicolor 128/256 refreshed, gtk-cache + sycoca rebuilt. The next
  revision starts from the SVG, not from archaeology.

#### Icon deployment footnotes (2026-09-04, dev-only)

- Dock/KRunner landed via: user-shadow `Songstress.desktop` with an
  absolute-path `Icon=` (KDE does NOT consult the user-dir hicolor
  overlay; system copies were stale RPM-owned art), plus a
  `NoDisplay=true` `com.yossi.songstress.desktop` alias (app-id match).
- **KWin's overview on 6.7 stayed generic even with `set_icon()` pushing
  the icon over the KDE extension** (undecorated GTK client quirk?).
  Accepted: dock + KRunner carry the mark; the overview is KWin's
  business. `window.set_icon` stays — it is the correct app-side push.
- `kbuildsycoca6 --nosignal` is GONE on Plasma 6.7 (rc=1, quiet) — the
  silent no-op that hid this whole hunt. Plain `kbuildsycoca6`.

### Cache hygiene: thumbnails no longer outlive albums — 0.9.8

  Orphan cleanup at scan end was DB-side only; per-album WebP dirs under
  the cache leaked whenever an album died (discards, retags to a new
  key). Measured 301 dirs / 251 rows on the live library.
  artwork::prune_orphan_thumbs now runs after every scan's artwork
  post-pass; the live sweep reclaimed 50 dirs (155→129 MB). New DEV
  door `__scan()` in main.ts (devctl has no __TAURI__; the API is
  bundle-imported). Doc note: PHASE2.md deleted as dead weight — every
  root doc earns its place now (README/AGENTS/PLAN + the two impeccable
  briefs).

### Global Menu glyphs + the stencil as the app's resting face — 0.9.9

  Playback menu rows now carry freedesktop icon names rendered by
  Plasma's own Breeze set (dbusmenu "icon" prop): transport, album-skip
  (go-*-skip), and state-honest repeat/shuffle/EQ glyphs — the icons
  track the stage the label describes (test asserts play-pause flips
  with state). Missing artwork everywhere (grid tile, expanded panel,
  playbar square when nothing is loaded) now shows StencilMark.svelte:
  the app-icon microphone glyph-only, currentColor, ~30% opacity —
  "no cover" reads as the app's own quiet room. The stencil now lives
  in three materializations: the icon (master SVG), the window push
  (set_icon), and the UI placeholder (component); one geometry.

### Wide-window panel fixes: row spacing + switch "sort glitch" (2026-09-05, dev report)

  Two sightings only visible once the window left its bottom-left tile:

  1. **Air between tracklist rows** (Dead Ringer screenshot). `.tracklist`
     carries `flex: 1`, and the panel is as tall as its cover art (the
     `clamp(170px, 24%, 300px)` width grows with the window → 300px square).
     A short two-column list in that box leaves the grid's `auto` row tracks
     to absorb the slack — 8 tracks stretched over 300px. Fix:
     `align-content: start` on `.tracklist.two`; the slack belongs below the
     list, not between its rows.
  2. **Tracklist "sort glitch" when switching albums on the same row**
     (notably → Welcome To The Neighborhood). The same-row host flip swaps
     `tracks`, and the keyed each sees remove-all + add-all: every outgoing
     row plays its 160 ms outro fade IN THE SAME `grid-auto-flow: column`
     grid, so both albums' rows occupy cells for 160 ms — phantom extra
     columns, scrambled order — then settle. Fix: `{#key displayAlbum.id}`
     around the tracklist block → the swap is atomic (the outro of a
     `|local` transition does not play on ancestor-block destruction), while
     store-driven row adds/removes **within** one album still fade, because
     the key doesn't change and those stay local each-block deltas.

  Frontend-only — hot-reloaded, no restart, window untouched. Gates:
  svelte-check 0, vitest 84 green, build green.

  Follow-up the same session (owner evidence: Bat Out of Hell 7-track single
  vs Dead Ringer 8-track 4+4, side by side on one line): the 8/5 thresholds
  put the SHAPE CHANGE between adjacent albums — one extra track and a full
  list became two half-width stumps in the art-tall panel. An unbalanced
  band (6+2) was considered and rejected: it keeps the flip exactly where
  the eye compares. New rule: SPLIT_MIN = 11, single column through 10,
  balanced halves from 11, ONE threshold for the main list and per-disc
  blocks (old disc threshold was 5). 11+ splits read as "big album"; the
  10→11 cliff is the least-traveled spot in the library.

  Amended after living with it: SPLIT_MIN settled at 9 (single column
  through 8 — the Bat-Out-of-Hell shape he endorsed — balanced halves from
  9; 9 splits 5+4). One constant, main list + disc blocks, unchanged.

  Final shape of the rule (owner, same session): split from 9, BALANCED
  halves only from 11. 9 and 10 render a dominant column with the last two
  tracks carved to a short tail — 7+2 / 8+2 — so the band reads as a full
  list with overflow instead of two stumps. splitRows(n): n>=11 ? ceil(n/2)
  : n-2; single constant pair, main list + disc blocks.

  Rule as settled (owner): >8 splits; below the balance minimum (11) the
  head column caps at 7 and the rest spills — 9 → 7+2, 10 → 7+3; 11+
  balanced halves. The 7-cap makes 9s and 10s the same panel height, gives
  column 2 enough mass to read as a forming column, and keeps the closer
  company in the tail. splitRows(n) = n>=11 ? ceil(n/2) : min(7, n).

  Amended (owner ruling, 2026-09-05, same day): the BALANCED shape is now
  keyed on the ALBUM, not the count — any MULTI-DISC album with >8 tracks
  splits balanced (ceil(n/2)), because a multi-disc record is genuinely
  big and two even columns match its weight. The dominant-column rule
  (7-cap, spill) is reserved for SINGLE-disc albums: 9 → 7+2, 10 → 7+3,
  12 → 7+5 (a lone disc's long list stays "full list with overflow",
  never two stumps). BALANCE_MIN (11) is gone — replaced by a multiDisc
  flag threaded from `hasMultipleDiscs` into splitRows; per-disc blocks
  live only in the multi-disc branch, so they take the balanced shape.
  splitRows(n, multiDisc) = multiDisc ? ceil(n/2) : min(7, n).

### Selection is chrome's no: the prefixed user-select (2026-09-05)

  Owner: dragging across grid/sidebar/playbar painted selections. The
  intent was already in app.css (body none, input text) plus three
  component rules — all DEAD CODE on this engine: WebKitGTK here (UA
  reports Version/60.5) ignores the unprefixed `user-select`; measured
  body's computed `-webkit-user-select` = UA `text`. Fix in the global
  sheet: `-webkit-user-select: none` on body (inherits to every surface,
  grid/panels/sidebar/playbar alike), editable exception widened from
  bare `input` to `input, textarea, [contenteditable="true"]` with both
  spellings. The four component `none`s are now redundant belts under the
  body inheritance — left in place. Probe after edit: body `none`,
  input `text`.

### The × that sat 4px high: no standalone transform properties (2026-09-05)

  Owner: search-field clear button off-center. Measured: button box
  centered fine; the svg inside it rode 4px high. Root cause: this
  WebKitGTK misapplies the INDIVIDUAL transform properties
  (`translate`/`rotate`/`scale`) on at least some svg elements —
  computed values look right, layout disagrees (same silence family as
  unprefixed `user-select` above). Traffic lights and the button itself,
  using the same properties, measured correct — so it is per-element,
  which is worse: no rule of thumb, only the dialect that never lies.
  Policy adopted: centering uses `transform: translate(...)` (the
  magnifier icon proved it) everywhere; converted every standalone
  translate/rotate/scale in the app — Sidebar (.search-clear button +
  svg, .tb-light svg, .gear .icon — its morph now transitions a combined
  `transform: translate(-50%,-50%) rotate(±90deg)`), SurfaceClose .sc
  svg, ExpandedPanel .play-all svg nudge, ProgressRing -90deg.
  `grep '^\s*(translate|rotate|scale):' src/` is now empty. Re-measured
  after: .tb-light 3.5/3.5, .search-clear 5/5 — exact.

### Tag saves now re-index the files they wrote (2026-09-05, owner report)

  Xandria's staged Sacrificium Bonus CD was retitled via the tag editor
  (files confirmed: TALB=Sacrificium) but the expanded view never moved.
  Diagnosis: staged albums stay WHERE THEY WERE IMPORTED (BT pile under
  ~/Downloads, outside every library root) — and an incremental scan only
  revisits files under a root, while the watcher is blind out there too.
  `save_album_tags` wrote the files and nothing ever re-read them: the DB
  carried the pre-edit tags forever (the journal's `missing 13` WAS those
  rows — the scan could not see them; every path verified to exist).
  Library-root albums only appeared to work because the watcher rescans
  them a beat after the save.

  Fix: both save commands now end with `rescan_written(conn, app, paths)`
  (lib.rs) — the import's own mechanism: roots = the written files'
  parents, only = the written files, no removal sweep — so the rows are
  re-parsed, re-grouped (album identity follows the new tags: merge,
  move, split) and `scan-finished` is emitted so the dump reloads like
  any scan. Journal: `[scan] retag-rescan: added U updated N (asked K)`
  on every save, FAILED variant included.
  Regression test: tags.rs::relabel_then_targeted_rescan_regroups_rows.
  Live verification: re-fired the owner's exact save via devctl invoke —
  `[scan] retag-rescan: added 0 updated 13 (asked 13)`, DB now shows ONE
  Sacrificium, 25 rows across discs 1–2; the Bonus CD tile dissolved
  into a real disc 2, staged flags intact (13 still pending, they still
  live in Downloads until Save). Also: Cargo.lock version 0.9.13 caught
  up (missed by the bump — the lock is a fourth place, not three).

### Phase E accordion: fixed width + why the joined album isn't offered (2026-09-05)

  Width bug (owner: "it should stay full width no matter the string of
  the item inside"): .te-pick sat as a row-wrap FLEX ITEM of .te-join —
  content-based width, so typing in the search literally resized the
  walls. Fix: flex:1 1 100% (the promise line's own dialect) — own line,
  always full width, options ellipsize inside.

  Sacrificium missing from the list was CORRECT: after the owner's album
  retag + the new retag-rescan, the file's own tag already says
  "Sacrificium" and its row ALREADY IS in that album (disc 2, staged) —
  and the picker deliberately hides "the album the file already points
  at" (joining where you already are is a no-op). It shows as the
  album's own disc-2 block, not as a join target.

### ArtChange::Hash was a struct variant: candidate picks died at the IPC wall
  (2026-09-05, owner screenshot: `invalid args 'art' … invalid type:
  string "2d4afec97…"` while setting a cover on the ASMR compilation)

  The webview's documented wire shape (src/lib/artChange.ts) is
  externally tagged: "keep" | "clear" | {"hash":"<hex>"} | {"upload":{…}}.
  Rust declared Hash as a STRUCT variant (`Hash { hash }`), whose tagged
  JSON is DOUBLY nested — {"hash":{"hash":…}} — so every pick from the
  candidate grid failed serde before the command ran. Upload { image,
  mime } happens to match the contract (its payload IS a map of those
  fields) and unit variants are plain strings, so uploading a new image
  always worked and only "use one of the album's OWN covers" broke —
  which is precisely the verb a compilation modal is built around (10
  files, 10 pictures). Fix: Hash is a newtype variant now; the comment
  records the wire contract where it lives. Regression test asserts the
  WIRE both ways (from_str/to_str on the literal JSON), not the
  constructor — the constructor never lies about a serde shape.
  Verified live against the very album: save_album_tags with
  art={"hash": <current>} returned {written:10, total:10,
  receipt:["replaced Folder.jpg"]}. (The hash in the owner's error is no
  longer among ASMR's candidates — one of the album's pictures changed
  with his earlier field edits, so the failed save's target had since
  retired from the pile.)

### Cleared year stayed on the tile + "duplicate" artwork rows (2026-09-05, owner questions)

  Year: the album save wrote the files (TDRC verified gone) but the
  albums ROW is only ever INSERTed (ON CONFLICT DO NOTHING; retag-adopt
  matches artist+title on purpose), so a cleared/changed year never
  reached the DB and the grid printed the fossil forever. Fix:
  save_album_tags, on a clean write and touched.year, UPDATEs
  albums.year directly — title/artist stay the scan's to re-key (identity),
  year is stored metadata; the TRACK save deliberately doesn't touch the
  row (one file's year is no album consensus). Test:
  year_edit_updates_the_album_row (set AND clear). Live: Anison no
  Kokoro's row is NULL now; owner's own save, completed by the fix.

  Artwork rows: NOT a bug. The 218-file Anison compilation embeds at
  least 6 byte-distinct encodings of the same 1000x1000 cover (71.8k…
  101k) plus a folder Cover.jpg that is a 7th; the inventory dedupes by
  blake3, so every real image is shown with its honest file count
  ("in 156 files", "in 20 files"). Candidate UX idea (NOT built):
  perceptual grouping of byte-distinct look-alikes.

### Artwork clear: ghost the victim, not the graveyard (2026-09-05)

  Owner: "Remove artwork… if I press it all artworks get a disabled
  look." The clear-decision preview dimmed EVERY candidate tile
  (as-ghost on cleared), which reads "the picker broke" while the note
  below the strip correctly promised removal from every file. Now only
  the album's CURRENT cover ghosts — the single image the decision
  removes — with its caption swapped to "removing on Save" in the
  caution hue; every other candidate keeps full contrast, because
  clicking one IS the recovery path and should look like it. Undo
  remains the header's Keep artwork / re-picking anything. The ✕
  living only on the selected tile is by design (removal is a verb on
  THE cover — it strips the picture from every file plus folder art),
  now backed by the captioned ghost so the state has a subject.

### The red arc: inline `background` shorthand repainted the border band (2026-09-05)

  Owner: "that solid line of red looks a bit weird" along Of Time and
  Parallels' rounded top edge (the album whose extracted accent is a 0.1%
  red logo stamp). DOM probe exonerated everything (border neutral
  rgba(255,255,255,.1), no outline/pseudo/shadow) — the culprit was the
  inline `style:background={<gradient>}` SHORTHAND: it silently reset
  background-clip to border-box, so the artwork gradient painted under
  the translucent rounded border; a translucent border over a saturated
  1px sliver prints a hard oversaturated rim, reddest where the 135°
  ramp is reddest. (The same shorthand had also been wiping .panel's own
  --panel-bg base all along.) Fixed with per-layer layers:
  `${gradient} padding-box padding-box, var(--panel-bg) border-box` —
  gradient inside the border, neutral base under it. Verified by owner
  screenshot + pixel scan (the line measured rgb(102..128,49..55,66..72)
  before; gone). Note the grammar trap: a single <box> keyword on a
  NON-final layer is ORIGIN, not clip — the doubled padding-box is
  load-bearing. Screenshot capture note: WindowsRunner Match returns an
  empty list on this KWin session and queryWindowInfo times out — manual
  owner screenshots are the eyes until that DBus path is debugged.

### Artwork picker: real placeholders for both slow phases (2026-09-05)

  Owner: Anison's cover grid took a while after opening. Two phases were
  un-cushioned: (1) get_art_candidates parses EVERY file of the album
  (218 for Anison) and the existing skeleton was invisible in the album
  modal — `.as-stack .as-sk { height: auto }` collapsed it to zero, i.e.
  the loader died in the exact modal that needs it; and (2) after the
  inventory arrived, each preview <img> streamed from disk and POPPED
  per-tile. Now: stack skeleton matches the real stack tile (172px) and
  wears --sk-base (app.css's dedicated placeholder pair — the tile's own
  bg was --hover, a token its own comment warns against reusing for
  placeholders); and every tile image reveals through a 220 ms opacity
  fade via a cache-safe reveal action (complete+naturalWidth checked at
  connect; error also reveals). Layout never moves; covers arrive INTO
  their boxes. A pop is the jarring event, the fade is not — no separate
  reduced-motion branch.

### Picker order: the worn cover leads (2026-09-05)

  Candidates are ordered by file count server-side, which on a
  multi-encoding album buried "which one is the cover" in scavenger
  hunt. orderedCandidates promotes the tile the save will wear (the
  user's fresh pick if any, else the album's current; nothing while the
  clear decision is pending, since then nothing is worn; nothing for
  uploads, whose tile is the arrival already at the head) to position 1
  — position is the loudest ordering cue, dot and ring keep saying it
  too. Keyed each: picking a different tile MOVES it to the head (node
  moves, image persists) — deliberate: the jump is feedback.

### The artwork-removal banner was lofty's ID3v1 writer (2026-09-05)

  Owner screenshot: red banner mid-save on Road To The Unknown — `task
  906 panicked … end byte index 28 is not a char boundary; inside 'í'`.
  First diagnosis (this entry) blamed parse_ini's `line[1..len-1]` —
  WRONG: that slice only runs on lines ending in ASCII `]`, so its end
  index is always a char boundary. The real backtrace (second
  occurrence, full symbols in the journal) is `write_id3v1 →
  Id3v1TagRef::write_to → save_to_path → tags::write_file`: lofty
  0.22.4 truncates each legacy ID3v1 field with `val.split_at(BYTES)`,
  and the COMMENT field's budget is 28 — inside 'í' (bytes 27..29).
  Reproduced byte-exactly: "Walla Walla.mp3" carries a cp1251 comment
  ("Collected by Тарантиныч"); lofty's v1 reader decodes one char per
  byte, and the split at 28 lands inside the 'í'. Every save of that
  file — artwork clear included — died there, and the first (unarmed)
  occurrence left the tail half-written until the next scan.

  The writer is now armored in layers (tags.rs write_file):
  the save goes to a sibling temp under catch_unwind (a panic cannot
  touch the original); every ID3v1 text field is pre-trimmed to its
  exact budget at a char boundary, which removes the known panic at
  its source; if the writer panics anyway (a field we don't map), the
  save retries ONCE with the ID3v1 tag dropped — a 1997-era duplicate
  of fields we just wrote into v2 is worth less than a file that saves
  at all. repair_stacked_tags got the same treatment, and now runs on
  a same-extension sibling (lofty probes the format by extension — a
  `.songstress-tmp` re-READs as UnknownFormat, measured). Regression
  test: write_file_survives_id3v1_multibyte_truncation plants a
  hand-built legacy tag at EOF (lofty's own writer can't produce the
  shape — producing it is the panic) and asserts the save lands AND
  the v1 survives char-trimmed. The parse_ini strip_prefix/suffix fix
  and its test stay — character ops on user data is the rule — but
  that parser was innocent. Verified live: re-save of the whole album
  against the dev instance writes Walla Walla (and the other 66) with
  the v1 intact.

  NOTE, lost data: while probing, a temp test renamed its copy onto
  `Where Is The Love_.mp3` (the sibling name collided with the
  original via with_extension) and its cleanup DELETED the original.
  The file is gone from the collection (not in Trash, no snapshot);
  the DB row (tr-a3ecc4f2c5c3e69d) now points at a missing path —
  Restore the file or prune the row, owner's call.

### Relink landed outside the album folder (2026-09-05)

  Owner restored Where Is The Love_.mp3 from backup and double-clicked the
  missing row to relink. The command COPIED it (correct — the file was
  outside every scan root, and rows outside the roots are swept) but into
  its own naive `<primary root>/<Artist>/<Album>` layout, orphaning it a
  directory away from the 66 files it belongs to. relink_track predates
  import staging and never learned about `album_destination`, whose rule
  1 is exactly this case: the album already lives somewhere → join that
  folder. Now wired to it (folder also checked against ALL roots, not
  just the primary — secondary-root files no longer copy unnecessarily).
  The stray copy was moved into the album folder with its row re-pointed
  (same id, mtime zeroed → the watcher re-parsed it back into the album);
  the two folders relink had created were pruned (the app wrote them, so
  the never-delete-what-you-didnt-write rule is satisfied).

### App icon: padded to the theme's visual weight (2026-09-05)

  Owner: "ours looks larger". Taskbar measurement said equal (all tiles
  76x82 @2x) — but the KRunner/app-launcher list prints the hicolor PNG
  as art, measured 50x51 against Reversal's 43-45: our tile was
  full-bleed (99% canvas), the theme's app tiles carry ~14% margin, so
  the launcher drew us genuinely bigger than the row. Fix in the SVG
  master: the whole tile wrapped in scale(0.86) at the center — corners
  and stencil scale with it, nothing redrawn. Regenerated via
  `npx tauri icon assets/app-icon.svg -o src-tauri/icons`, installed to
  every hicolor size the desktop files reference (48/512 rasterized
  from the SVG with ImageMagick), icon cache refreshed. Version 0.9.14.

### Tracklist split: the balanced shape is a multi-disc thing (2026-09-05)

  Owner ruling: balanced halves for ANY album over 8 tracks that has
  multiple discs; the 7-cap dominant-column form (9 → 7+2, 10 → 7+3) is
  reserved for SINGLE-disc albums. The count-keyed BALANCE_MIN (11) is
  gone — splitRows(n, multiDisc) = multiDisc ? ceil(n/2) : min(7, n),
  the flag threaded from the panel's existing `hasMultipleDiscs`. The
  visible change vs yesterday's rule: single-disc 11+ no longer halves
  (12 → 7+5 now), multi-disc 9–10 no longer caps (→ 5+4 / 5+5), and a
  fat disc block always halves. Rationale on the record: a multi-disc
  record is genuinely big, two even columns match its weight; a lone
  disc's long list stays "full list with overflow", never two stumps.

### Window size memory: hand-rolled, size-only (2026-09-05, owner ask)

  Owner: "bothersome to resize it every time i open the app". The
  2026-09-01 revert note said do-not-re-add the plugin without a case
  where size is not already free — this is that case, asked for in his
  own words. NOT the plugin again: it writes only on RunEvent::Exit,
  which the dev unit's SIGTERM and every `tauri dev` rebuild-kill never
  reach (measured then: three kills, no state file). New `window_state`
  module instead: save on EVERY `WindowEvent::Resized` (plus
  CloseRequested) as logical px — a few-hundred-byte tempfile+rename
  into the app CONFIG dir, which always holds the size the compositor
  last agreed to, so a kill before close loses at most an ongoing drag;
  restore in setup with a 600×400 sanity floor (any bad/missing file is
  simply "no memory"). POSITION is still never saved or restored — the
  Wayland client can't see it (AGENTS.md protocol: he places the window
  himself), and the restart protocol is unchanged except that now ONLY
  position is his job. Window-state file:
  <config dir>/window-size.json. Version 0.9.15 with the split-rule
  change. Gates: cargo 96, vitest 84, svelte-check 0, build green.
  Post-shots the same day (owner: "it spawned closer to the bottom right",
  twice): the config-declared window maps at its baked 1280×800 and KWin
  places on FIRST MAP; a later client resize grows from the TOP-LEFT, so
  the remembered size visibly walked off-center. `visible:false` + resize
  + `show()` did NOT heal it (measured: KWin still placed for 1280×800),
  so the window is now BUILT in setup (`WebviewWindowBuilder`, label
  "main", title/min-size/decorations/transparent carried from the deleted
  config `windows` block — do not re-declare it there) with the restored
  size. Measured after: spawns AT the remembered size, position still
  KWin's call (~center, biased down-right; xdg cannot reposition after
  map and `.center()` is a hint). AGENTS.md's restart protocol reworded:
  restart = remembered size, misplaced position, manual place as ever.

### "From disk…" now eats audio files: extract the cover from the mp3 (2026-09-05)

  Owner ask: choose a MUSIC file as the artwork source and take the
  picture embedded in it. One verb, not a new surface: the picker tile's
  kdialog gained an Audio filter, and `read_image` grew a branch — an
  AUDIO path yields its LARGEST embedded picture (the scan's own
  `artwork::embedded_art`, now public, called with one file), anything
  else keeps the raw read. The 25 MB cap moved onto the RETURNED bytes,
  so a 60 MB FLAC whose picture is 300 KB passes while a 400 MB "image"
  still never crosses IPC; audio carrying no picture errors plainly into
  the picker's error line. Drag-drop picks up the behavior for free
  (same `fromPaths`, regex widened to scan::EXTENSIONS — now public so
  the picker cannot drift from what the scanner indexes). Save rides the
  existing ArtChange::Upload wire: extract → arrive as a tile → Save
  writes it like any brought-in image. Version 0.9.15.

### The 7-cap grew a four-column octopus (2026-09-05, same day as the rule)

  Owner screenshot: Queen "Forever" (23 rows, ALL tagged disc 1 — one
  album, no disc blocks) rendered 7+7+7+2 in FOUR columns. Cause: the
  same-day rule removed the balance fallback from the single-disc branch,
  and `grid-auto-flow: column` with a 7-row template spills EVERY 7-item
  chunk into a NEW column — the cap was only ever sound for the 9–10
  band it was designed on. Restored (measured owner-side): single-disc
  BALANCES from 11 like before; the 7-cap lives at 9–10 only; multi-disc
  balances from 9. The panel's column budget is TWO, always. (The album's
  skipped numbers 5/19 were innocent: its tags genuinely jump, the count
  label agreed.) Version 0.9.16.

### "No format could be determined": lofty's probe vs a fat stacked tag (2026-09-05)

  Owner joined a track into its album and the save died with
  `"19 Flash.mp3": No format could be determined from the provided file`.
  Diagnosis on a /tmp copy (owner-path rule honored): the file carries an
  ID3v2.3 whose declared region is 399 KB (a 387 KB APIC inside), an
  ID3v2.4 starting EXACTLY at that region's end, MPEG frames only after
  it, and a v1 at EOF. lofty READS this fine (the .mp3 extension is the
  hint) but its WRITE probe goes blind — it cannot place the audio
  past the first block — so `save_to_path` fails on EVERY path (temp,
  original, any extension), and write_file's chain never reaches the
  verify/repair step, which only exists behind a successful save.
  Mutants measured: removing EITHER v2 block heals the probe; removing
  only the v1 does not; synthetic small-stacks do NOT reproduce it (the
  fat declared region matters) — so no synthetic re-summons the exact
  refusal, and the tests pin the mechanism instead of the ghost.
  Fix: (a) new `cut_leading_tags` — strips leading ID3v2 blocks by the
  headers' own size fields (v2.2/v2.3/v2.4 arithmetic, refuses to eat
  more than the file holds), zero lofty involvement; (b)
  repair_stacked_tags runs its copy through it first, so the strip loop
  has a probeable file; (c) write_file's `Ok(Err(..))` branch routes
  "No format could be determined" into repair_stacked_tags (error string
  carries both halves if the repair also fails). Losslessness unchanged:
  `expected` is the merged view (union of the blocks) plus the user's
  edit, and it becomes the file's single tag. New tests 97–98:
  cut_leading_tags arithmetic (incl. the no-ID3 no-op) and the repair's
  contract. The owner's file self-heals on the next save. Version 0.9.17.

### The modal "glitches its dimensions and returns": one skeleton per track (2026-09-05)

  Owner: stepping the tag editor (or saving) made the modal briefly
  distort and snap back. Reproduced live via the devtools bridge: every
  step and every save fired the ArtSelector skeleton (3 pulsing `as-sk`
  tiles swapped in over the candidate column, then out). Cause: the
  component's load $effect transitively READ `trackId` (it is an invoke
  argument), and Svelte 5 registers every signal an effect's synchronous
  run touches — so the per-FILE signal re-ran the per-ALBUM fetch, and
  `loading=true` traded real tiles for the skeleton. The census itself
  never changes while stepping (it is the album's picture pile); a save
  CAN change it (new embedded art), but that wants a SILENT refresh, not
  a skeleton over content mid-dialog.
  Fix (ArtSelector): the effect now keys on `loadAlbum` ($state) and only
  fetches when albumId actually changes; a new `refreshSeq` prop, bumped
  by both hosts after a successful save, re-fetches quietly (tiles stay;
  no `loading` flip); trackId remains request context, never a trigger.
  Verified live: animationstart probe across a step and a save = zero
  skeleton activations, dialog geometry unmeasurable-flat (98 rAF frames
  at 680×525 through the earlier step). Owner's track title touched by
  the drive-in tests was restored through the app's own save. 0.9.18.

### The loader lives IN the standard box (2026-09-05, owner ruling)

  The first-open jump survived the skeleton fix because the surface did
  the opposite of what a modal should: `waiting` hid the panel until the
  fields existed, then revealed it sized to whatever loaded FIRST — and
  the stack's 3-tile artwork skeleton (3×172px) was TALLER than the
  finished picker, so opening a track measured 749 → 525: inflate, then
  shrink. Owner's rule: the loader sits inside the modal's standard
  dimensions and content may only ever EXPAND the box.
  TagSurface: `.te { min-height: 525px }` (the track modal's measured
  resting height; the album modal floors there too and grows from its
  own content), the invisible-waiting trick is gone — the panel enters
  on the click's frame, at final width and the standard height, showing
  "Reading file tags…" inside it. ArtSelector: stack mode reserves ONE
  placeholder tile sized exactly like a real one (strip keeps three —
  sideways, height-neutral). Re-measured open: gone → 674×520 →
  678×523 → 680×525 — the 520→525 leg is the entrance scale itself; the
  LAYOUT box is 680×525 from its first frame. Many-candidate albums now
  can only expand, per the rule. 0.9.19.

### Tracklist split: disc 1 leads the whole album (2026-09-06, owner ruling)

  Owner disliked the mixed look of a multi-disc album whose disc 1 splits
  5+5 while the disc below sits one-column-8 (Ira Dei) or splits at a
  different row (Human. :II: Nature.: 5+4 / single-8 / 5+4). His rule:
  the first disc's split point is the MINIMUM split point for every
  subsequent disc; plus the corollary that if any disc splits, disc 1
  splits too (balanced halves, even below the threshold) — a lone stump
  beside a split neighbor reads out of place. Traced against all 64
  multi-disc albums in the live DB first: no 1-item columns would appear,
  no saws, max edge drift one row.
  Contract now lives in `src/lib/discSplit.ts` (`discSplitPlan`, vitest-
  covered with the real library shapes): nothing splits if no disc
  reaches SPLIT_MIN; disc 1 halves (Electric Castle's 7-track disc 1
  splits 4+3 and LEADS at 4, so disc 2 stays a proper 5+5); a follower
  with more than the lead splits at max(lead, own balance) — the minimum,
  not fixed: Legends 11/23 → 6+5 · 12+11, the only visible fallback in
  the library; a follower ≤ lead stays single, its one column already
  lead-wide (RotK's disc 4). ExpandedPanel: discGroups carries `rows`
  per block (template no longer computes), splitRows/halfRows are the
  single-disc branch only (7-cap band untouched). New shapes: Ira Dei
  5+5 · 5+3, Human. 5+4 · 5+3 · 5+4. Frontend-only; hot-reloaded.
  Amendment (owner eyes, same day — Legacy of the Dark Lands 24/12/12/20):
  rule 4 over-shot. Discs 2/3 (12 tracks each, lead 12) rendered single
  "because 12 ≤ lead" — a 12-row column, too long, and grammar-inconsistent
  under a split disc 1. The minimum that governs a follower is not always
  disc 1's: at or below the lead, a disc ≥ SPLIT_MIN splits at its OWN
  balanced halves (Legacy → 12+12 · 6+6 · 6+6 · 12+8). Single-column now
  requires n < SPLIT_MIN AND n ≤ lead (RotK's 8-track tail unchanged); the
  whole 64-album census re-traced — Legacy is the only album the amendment
  changes. 9 new/updated vitest cases green, check + build green.
  Amendment 2 (same day — 01011001 8/7, Twilight Dementia 7/6): multi-disc
  blocks get their own gate, MD_SPLIT_MIN = 7. The owner's reason for 9
  ("the cover square pins the panel height") only anchors the FIRST
  panel content; disc blocks after the first add pure incremental height
  with a void right — so stacked discs split from 7. Contract shape
  unchanged (lead / minimum / own-balance); only the two `SPLIT_MIN`
  gates in discSplitPlan become MD_SPLIT_MIN; interlude discs ≤ 6 stay
  single (Omega untouched). Smallest reachable split: 4+2. Whole-DB
  re-trace: exactly 4 blocks change — 01011001 → 4+4 · 4+3, Twilight →
  4+3 · 4+2, RotK d4 and Lion King d2 (8-track singles at/below lead) →
  4+4 each; the other 60 albums bit-identical. 95 vitest green.
  Simplification (owner-invited, same day): the "disc 1 always splits"
  exception is gone. Rule 3's first branch hands disc 1 exactly the
  lead on its own (n1 > lead for every n1 ≥ 2), and no owned album has
  a disc 1 under the gate, so the clause only ever explained itself.
  A hypothetical [2,9] now renders 1+1 · 5+4 (owner: a 2-track disc
  splitting under a big one "would look fine"). The contract is now:
  a gate (any disc ≥ 7), a definition (lead = half of disc 1), and ONE
  per-disc formula with no positional exceptions — 5-line function,
  every clause tied to a named album. 64-album re-trace: 0 outcome
  diffs; 96 vitest green.

### The boot skeleton is a right, not a bet (2026-09-06, owner ruling)

  The owner opened the 0.9.19 RPM (installed 19:38 — same
  com.yossi.songstress data dir as dev, so a WARM launch of the
  populated library) and saw no loader at all: content in one silent
  swap, "stuff just pops as it loads". Diagnosis: nothing broken —
  BOOT_GRACE_MS (150ms) suppressed the placeholder on every dump as
  fast as the measured 46–50ms, by design. Owner overturned the
  design: a launch dump IS a load, repeated or cached or not, and
  hiding it reads as a glitch. New rule: skeleton on from the first
  frame of every boot; the STORE holds the first dump until
  MIN_SKELETON_MS=250 has been served (flicker rule: a 50ms
  placeholder is a manufactured delay), then hands over WITH the
  entrance cascade (filledFromPlaceholder gains `firstDump`;
  mid-session refreshes over visible content still swap silently,
  the 400ms note unchanged). libraryLoading is now four lines, boot
  branch = `!ready`; bootSlow/grace deleted everywhere. Verified
  live by rAF-sampling a dev reload: pills for the floor's ~10
  frames, hand-over mid-cascade. RPM must be rebuilt to carry this.

### EQ silenced every 32 kHz track: Nyquist-unstable 16 kHz band (2026-09-06)

  Owner report: tracks like My Dearest / Break up! / Butter-Fly (Anison no
  Kokoro) played as silence with a moving progress bar, plus a pop on every
  switch-to, switch-from, and pause/resume — but ONLY with the EQ on.
  Diagnosis, in order: (1) all affected files probed at 32000 Hz while the
  rest of the library is 44.1/48 kHz — files themselves decode perfectly
  (ffmpeg volumedetect −10 dB mean), so engine-side; (2) no ReplayGain tags,
  bare mpv plays them fine natively at 32 kHz; (3) owner proved the EQ is
  the trigger by toggling it off; (4) `gapless-audio=weak` (was `yes`)
  changed nothing — owner retested, same behavior, so the hidden-resampler
  theory died; (5) NUMERIC PROOF: the Rock preset's `equalizer=f=16000`
  band sits exactly on Nyquist at 32 kHz, goes unstable, and rails ALL
  output samples to 0 dBFS (measured: mean/max 0.0 dB, 64000/64000 clipped;
  same chain at 48 kHz or without the band: −22.7 dB healthy). A railed/DC
  stream reads as silence downstream with a transient on every graph
  start/stop — the entire symptom set. First fix attempt (rate-aware chain
  + `audio-params/samplerate` observer rebuilding per track) cured the
  silence but left 2 thumps on first entry — the observer only learns the
  rate AFTER the file loads, so it can never win that race. Final fix, in
  `eq.rs` only: the chain opens with `aresample=48000`, pinning the EQ's
  working rate for every track — no band is ever near Nyquist, the graph
  never rebuilds between files, the AO never reconfigures. Observer
  machinery reverted; `mpv.rs` diff is just the `weak` flag (kept: still
  the honest setting for format changes). Library-wide ffprobe sweep
  (4,527 files): exactly 23 at 32 kHz — 14 Anison no Kokoro (incl. Beat
  Hit!, missed in the first sweep) + 9 Kingdom Hearts orchestral (Of Time
  And Parallels); owner listened through the list, all clean. Gates:
  svelte-check 0, vitest 97, cargo 99 (incl. new pinned-rate test), build
  ok. NOT version-bumped / committed / RPM-built — owner deferred.

### UI showed track 1–2 while audio played tracks N-1–N at the album-repeat wrap (2026-09-06, owner report, unreproduced)

  Owner, passive listening, repeat=album, "Avalon – The Land of New Hope"
  (10 tracks, all numbered, all present, all plain 44.1 kHz MP3 — data
  ruled out first): at the wrap the UI indicated track 1 while track 9
  played, then track 2 while track 10 played — a stable +2 UI-ahead-of-audio
  offset. Audit trail: (1) UI shows Rust's st.index, audio is mpv's real
  playlist position — any extra Rust advance desyncs them; (2) live IPC
  probe proved all programmatic commands benign: loadfile-replace emits
  end-file reason "stop" (ignored), playlist-clear emits nothing,
  appending a missing path emits nothing eagerly — so clicks, Reload wraps
  and bad appends cannot self-corrupt; (3) N=10 excludes the >32-track
  duplicate bug below. Remaining explanation: duplicate natural end-file
  events for real boundaries (mpv-side quirk), unproven without a repro.
  Mitigations shipped: every eof/error decision now logs one journal line
  (`[mpv] end-file reason=… idx=A->B action=…`) so a recurrence is provable
  from `journalctl --user -u songstress-dev`; if it recurs, grab the log
  around the wrap before touching anything. Deliberately NOT shipped: a
  ground-truth resync off observed `path` — behavior risk with no repro to
  validate against; revisit with logs in hand.

### Wrap desync recurred with proof — path resync shipped (2026-09-11, Avantasia "Angel Of Babylon")

  The Sept-6 Avalon ghost came back, this time with the new eof logging to convict with. Passive listening, repeat=album, 11-track album: at the wrap the playbar showed Stargazers at 0:56 / 3:52 — but Stargazers is 571s and 3:52 is Your Love Is Evil (232.8s, DB-verified). `upNext` (Angel / Your Love / Death) proved Rust sat at index 0 while mpv played a 232s file starting at the wrap instant. The eof trail is unambiguous: single `10->0` at 16:50:55, NO `0->1`/`1->2` after — Rust never double-advanced, so mpv's playlist diverged from Rust's order with zero eof trail. The Sept-6 top-up fix is exonerated (N=11 < 32, and the trail shows no duplicate). What the trail still cannot show is the mpv-side playlist CONTENT at the wrap (pre-arm is logged only as a count) — the divergence mechanism itself remains unproven (candidates: stale fire-and-forget appends landing after a later playlist-clear during rapid rebuilds; a toggle re-arm; none confirmed).
  Shipped the item deliberately deferred on Sept 6, now justified: (1) ground-truth resync — mpv `path` is now an observed property (id 5); every change schedules a 500ms delayed recheck (both eof-first and path-first orderings settle by then), and a lingering mismatch self-heals Rust's index to the observed file with a loud `[mpv] RESYNC` line, or `[mpv] PATH-MISMATCH` when the file is unknown (log, never guess). Normal advances recheck clean and stay silent. (2) The silent-path blind spot is closed: one journal line per index-changing op (`play_order`, `jump`, `queue_jump`, `shuffle-rebuild`, `repeat-arm` with first file), and every end-file line now carries `now=` (current file) + `pre_first=` (what the armed pass starts with) — the exact evidence missing today. A second `10->0` 7 min after the screenshot needs index 10 just before, reachable only via those now-logged silent paths (owner's own post-screenshot clicking is the working theory, unconfirmed). Pure-fn `resync_index` + 3 unit tests. Gates: cargo 103, check 0/0, vitest 97, build ok. RPM `Songstress-0.10.1-1.x86_64.rpm` built + verified (Name songstress, Requires mpv + kdialog) — handed to owner for `dnf install` + window-drag protocol. Version 0.10.1 (patch: wrap-desync resync + op logging).

### Repeat-wrap appended track #32 twice on >32-track albums (2026-09-06, found during the above audit)

  At every gapless/queue-drained wrap promote, eof_advance appended
  order[31] as top-up — but the pre-armed pass (32 entries) already fills
  mpv's window behind the new current, so track #32 played twice and the UI
  ran +1 ahead of the audio for the rest of the pass (self-healed at the
  next wrap). Hit every repeat listen of big compilations (Anison no Kokoro:
  219 tracks). Fix: top_up=None at all three promote sites; the existing
  unit test enshrined the duplicate (`/t31.mp3` "fresh pass top-up") and now
  asserts None. Same family as the Avalon report, different mechanism —
  and excluded as its cause (N=10 < 32).

### Playbar bottom line prefers the track artist (2026-09-10, owner ask)

  Was always album artist + album title, so guest/feats read wrong. The scan
  always parsed the per-track artist but never stored it — new `tracks.artist`
  column (migration v3, nullable), written on insert + updated on conflict,
  exposed as `artist` in `get_library`. Frontend: `Track.artist?`, playbar
  shows `track.artist.trim() || albumArtist` — NULL/empty (pre-v3 rows,
  untagged files) falls back to the old line, never blank. Existing rows need
  a Full Rescan to backfill (incremental skips unchanged files). Fake library:
  one guest spot on the Various Artists comp so the branch has data in fake
  mode. Gates: svelte-check 0, vitest 97, cargo 100, build ok.

### Playbar overlong lines marquee instead of truncating (2026-09-10, owner ask)

  Long titles/subs ("Wake Up To The Moon" case) ellipsized with no way to read
  them. Both playbar lines now run a seamless CSS marquee, ONLY on overflow:
  a Svelte action measures single-copy vs viewport (ResizeObserver +
  track/text key), appends one aria-hidden twin + 48px gap on overflow, and
  loops the strip by exactly copy+gap so the wrap is invisible. transform-only,
  linear, duration scaled to distance (~32px/s, clamped 4–18s) + 1.2s first-loop
  delay to read the head. Pauses when the music pauses (cycling on a stopped
  player is motion with no job) and on hover (reading); reduced-motion keeps
  the old static ellipsis + native title tooltip. Same :global() hatch as
  ArtSelector's as-load for the JS-added class. Gates: check 0/0, vitest 97,
  build ok, detector clean; live probe showed the current short track correctly
  staying static — the real overflow proof is his Ghostlights track.
  Lag check (owner: "is it choppy?"): 4s rAF sample on the live marquee —
  239 frames, mean 17.6ms, p95 24ms, ZERO frames over 34ms, steady -0.7px/frame
  advance. No jank; the loop is healthy (very long lines hit the 18s clamp so
  his Ghostlights sub runs ~40px/s, not 32). Speed bumped to ~56px/s on his
  ask (near 1px/frame — live-confirmed 12.82s loop); he judges the feel.
  Still steppy at 56 → ruled out restarts (getAnimations: same startTime,
  currentTime advancing wall-clock) and drops (earlier 4s sample clean), so it
  is raster-level, not scheduling. Now trying 37.5px/s = exactly 1 device px
  per frame at 60Hz × scale 1.6 (live-confirmed 15.89s loop = true 37.5).
  If that fails, next lever is fade-paging (no continuous motion at all).
  Verdict (owner): smoother, maybe not 100% gone — good enough to ship;
  37.5px/s + 24s clamp stay.

### Seek bar gets the volume bar's glass treatment (2026-09-10, owner ask)

  Same bug the volume slider just had: native accent-color range insets the
  thumb's travel but not its track, so near 100% a sliver of unfilled bar sat
  past the thumb. Same fix — appearance:none, the gradient IS the track
  (accent to seekPct%, hover past it), 14px accent thumb, accent focus ring,
  dimmed when nothing's loaded. Live-confirmed gradient + 4px + none;
  detector clean.

### Shared tooltip system, volume style everywhere (2026-09-11, owner ask)

  All 49 native titles across 11 files converted to `use:tooltip` — one
  singleton pill in the volume readout's material (11px, hairline, --active),
  hover 350ms / focus 150ms, flips near viewport top, hides on leave/blur/
  scroll/Escape/pointerdown, opacity-only, instant under reduced-motion.
  A11y: title captured at mount and removed (no doubles), linked back via
  aria-describedby while visible. Conversion exposed two icon-only controls
  whose sole name was title (reveal button, cover badges) — now real
  aria-labels. Live: Seek pill raises on focus with exact text + describedby.

### Tracklist arrow-key navigation + Enter (2026-09-11, owner ask)

  After selecting a track, Up/Down walk the highlight through the visible
  tracklist (listbox behavior) instead of scrolling; Enter already played
  and is untouched. Tight scope: body or track-row focus only (search,
  sliders, other controls keep native keys), same panel-open/menu guards,
  no wrap at edges, roving focus with preventScroll + nearest-scroll.

### Volume % readout riding the thumb (2026-09-10, owner ask)

  Pill above the volume thumb showing live `playback.volume`%, positioned
  with the half-thumb correction and clamped inside the track. Pure CSS
  show logic (:hover covers hover + drag, :focus-within covers keyboard),
  opacity-only 150ms entrance, aria-hidden (native range announces itself).
  Live: reads 33%, parked invisible at rest.
  Refinements (owner): fixed 4ch centered pill (no reflow 7%↔100%),
  unclamped thumb-true tracking edge to edge, 1s idle fade (timer on
  pointermove/input, wins over hover/focus via .idle). Handlers live on
  the input itself (span-level pointer handlers trip the a11y rule).
  Width fix: global border-box made min-width cap padding INCLUDED, so
  "100%" overflowed the floor — .vol-tip is content-box now, floor holds.
  Stuck-tip fix: :focus-within kept the pill up forever on mouse-focus
  residue after a drag (focus stays until window blur — exactly his symptom).
  Focus now counts only when :focus-visible (keyboard) via volKey.

### Exit-time WebKitWebProcess SEGV (2026-09-10, owner report — NOT our bug)

  Crash dialog after closing: SEGV in WebKitWebProcess during exit() —
  DRMDeviceManager teardown → _gbm_device_destroy → Mesa dri_destroy → heap
  corruption in free(). Our binary nowhere in the stack; we never touch
  GBM/EGL/DRM. coredumpctl shows the same renderer crashing since Sep 4
  (SEGV + SIGABRT, plus a Brave SIGTRAP Sep 9) — systemic, predates tonight's
  work. No data risk: it fires post-close in renderer teardown; DB/settings/
  window-size all persist incrementally. Advice: ignore, or confirm upstream
  via Epiphany-on-close. Do NOT set WEBKIT_DISABLE_DMABUF_RENDERER to dodge
  it — software compositing would undo the marquee smoothness work above.

### Ambient-blob prototype on the expanded panel (2026-09-10, owner idea)

  Two art-color globs drifting behind the panel content (26s/31s alternate
  loops, 8% opacity, colors = raw scan colors in the artwork's hue family).
  Cheap-by-construction: softness is a baked radial-gradient, never
  filter:blur (re-blurring per frame is the most expensive thing on this
  engine) — the loop moves cached layers via transform only. Content pinned
  above via z-index; reduced-motion freezes the drift. Live: both loops
  running, 3s frame sample mean 16.66ms / p95 17ms / zero drops — costs
  nothing measurable. Owner judges the LOOK; playbar port follows if it sings.
  V1 (55%/8%/26-31s) read as static smudges, motion invisible — same-hue wash
  + ultra-soft falloff. V2 (40%/16%/22-27s, 60% falloff, bigger travel):
  owner sees it. Awaiting taste verdict.
  V3: breathe removed (translation-only keyframes), travel ~45% of blob size
  each way, 18s/23s loops — owner: movement still too subtle at V2.
  Artifact report: diagonal lines = gradient banding (135° base + blob wash
  pushed it over the visible threshold; the panel never had .glass's grain).
  Fix: .panel::after feTurbulence dither at 0.035 (glass uses 0.02).
  Owner redirect: stops ARE the globs — linear ramp deleted, two 90% washes
  at the 135° corners (panel alpha) drifting ±12%, 20s/26s. One layer, less
  banding by construction. Live: ramp gone, washes running, frames still
  16.66/17/zero. Awaiting taste verdict v2.
  V2 verdict: washes too faint + RADIAL banding (rings). Owner redirect #2:
  ramp restored full-bleed on an oversized layer rocking ±9° via transform
  (18s alternate) — source points swing, paint rasterizes once. Blobs gone.
  Panel clips via overflow:hidden; grain stays.
  V3 bug (owner screenshot): layer edge swept into view — -25% inset assumed
  a square panel; at ~6:1 wide the ±9° rock needs ~45%-of-height margin.
  Bled to -60% (live: 207px vertical margin vs ~128 needed).
  Rock killed same night: barely readable. Owner spec v4: sources patrol —
  two fading LINEAR washes (135° c1 top-left slides ±16% panel width,
  315° c2 bottom-right mirrored), same 20s clock opposite phase. Live.
  Rings screenshot: radial-blob era (linear can't ring) — predated hot-swap.
  Fresh screenshot on linears: STILL banded (arcs — origin unclear, but
  hiding washes => "smooth as can be", so washes implicated). Static
  classic ramp test (via eval): mild diagonal banding too. Grain 0.035→0.07:
  WORSE (finer, more noticeable) — dither path dead. Verdict: full revert to
  shipped static ramp, grain experiment removed with it. Better Blur DX
  exonerated (isolation test never touched it; bands tracked our art hues).
  Open fork: keep classic ramp (mild banding, original identity) or go flat
  base (perfectly smooth, loses art color). Playbar port shelved meanwhile.
  PROTOTYPE DEAD (owner: "lost the fight"). Panel is byte-for-byte the
  shipped static ramp again. Lessons banked: transform motion itself held
  60fps clean all night — the killer was gradient QUANTIZATION on this
  engine (any large wash bands; dither grain made it worse, not better).
  Ambient motion on gradient surfaces: do not retry without a dither story.

### Sidebar artist select resets the grid to top (2026-09-10, owner ask)

  Re-clicking the active artist (or All Artists) collapsed the panel but kept
  scrollTop, stranding the view mid-grid. select() now zeroes main.content's
  scrollTop on every pick — instant, not smooth (rows are being replaced;
  a glide would chase the collapsing layout).

### Tracklist arrow-key navigation + Enter (2026-09-11, owner ask)

  After selecting a track, Up/Down walk the highlight through the visible
  tracklist (listbox behavior) instead of scrolling; Enter already played
  and is untouched. Tight scope: body or track-row focus only (search,
  sliders, other controls keep native keys), same panel-open/menu guards,
  no wrap at edges, roving focus with preventScroll + nearest-scroll.
  Follow-ups same night: Enter keeps the highlight (was clearing it, so
  arrows restarted at the album head) — mouse second-click keeps its toggle
  off-ramp via event.detail, keyboard Enter (detail 0) plays and holds.
  Left/Right step ±one column-height within the containing list (rows-only
  scope: body Left/Right stay PlayBar seek). Id-anchored focus (data-tid +
  CSS.escape), group-local math, single-column falls through.
  Bug fix same night: the detail-0 early return skipped playTrack — keyboard
  Enter on a focused row played nothing (the "highlight gone" downstream
  was the mouse toggle doing its job). Now detail 0 plays AND holds.

### Shared tooltip system, volume style everywhere (2026-09-11, owner ask)

  All 49 native titles across 11 files converted to `use:tooltip` — one
  singleton pill in the volume readout's material, hover/focus delays,
  viewport flip + clamp, hides on leave/blur/scroll/Escape/pointerdown,
  aria-describedby while visible. Two icon-only controls gained real
  aria-labels (their sole name was title).
  Gap sweep same night: four icon-only controls never had tips — Prev/Next
  track triangles, mute button (mirrors Unmute/Mute), search-clear ✕.
  Text-labeled buttons and the deliberately-bare row ✕ left alone.
  Flip fix: pill went ABOVE near top chrome (traffic-light overlap) — flip
  threshold 8px → 80px. Frost ask: backdrop blur added as progressive
  enhancement (engine can't blur in-window content — measured — so shadow +
  denser fill carry legibility everywhere); mirrored onto vol-tip.
  "Broken UI" same night: NOT the tooltips — global-sheet HMR rot (--gap
  computed "", all theme tokens dead after the long hot-edit series).
  Fixed with `touch src/app.css` (seconds, zero restarts); `--gap` back.

### Stuck seek tooltip after minimize + external-volume thumb sync (2026-09-13, owner report)

  Two reports, one session. (1) Restoring the minimized window with the
  pointer over the seek bar left its "Seek" tip up until another anchor
  stole it: minimize fires no mouseleave, so the 350ms show-timer popped
  while hidden and greeted the restore. Fix: tooltip hides + cancels on
  window blur and on document hidden (tooltip.ts ensure()). (2) Output
  level dropping on its own (thumb unmoved, master max; once mid-game):
  any MPRIS client (KDE media widget, playerctl) lowering mpv's volume
  never reached the thumb — Rust observed it but told nobody, so the UI
  kept showing the old value. Fix: the volume observer now emits
  `playback-volume` + a `[mpv] volume a -> b` journal line, and the
  playback store mirrors it into thumb + localStorage + settings (echoes
  of our own sets ignored by the rounding guard, no loop). NOT covered:
  replaygain=album (per-track gain, volume property untouched — a quiet
  master on one album is expected) and OS-level ducking (PipeWire
  role-duck lowers the sink outside mpv entirely; nothing to mirror).

  Follow-up (same day): owner confirms the dip was MID-TRACK — rules out
  replaygain (per-track gain). System probe finds PipeWire role-based
  ducking live (`linking.role-based.duck-level = 0.3`): a game stream ducks
  music to 30% below mpv entirely — no property changes, nothing to mirror,
  master untouched. Verdict: the game dip was the audio server, not the
  app. The new thumb-mirror + journal line now separate the two worlds:
  thumb moves = something drove mpv (logged); thumb still + quiet = the
  server ducked it.

### RPM release bump 0.10.1-1 → -2 (2026-09-13)

  0.10.1-1 was already built + installed from the intermediate tree, so the
  fixed RPM read as "already installed" (same EVR, no upgrade). Rule: a
  rebuild with an unchanged version bumps the rpm `release` instead.
  The three version files stay 0.10.1 — release is packaging revision.

### Stuck seek tip, round 2 (2026-09-13, still sticking after the blur fix)

  The blur/visibility fix only cancelled a *pending* timer — the restore has
  two hotter paths: a SYNTHETIC mouseenter (window remaps under a stationary
  pointer, no matching leave ever comes) and refired focus (a dragged slider
  parks DOM focus; the cycle re-arms it with the pointer nowhere near).
  Fix in tooltip.ts: a `seenMove` gate — blur/hide/activation disarm hover,
  only a real pointermove re-arms — plus focus tips require `:focus-visible`
  (keyboard-driven focus; mouse users get the hover tip). Probe hole found
  live: the wiring lived in `ensure()` (first-show), so a session whose FIRST
  trigger was the synthetic enter had nothing disarmed — guards now run at
  module load (window flag dedupes HMR re-imports). Verified on a cold unit
  via devctl: synthetic enter → no tip, programmatic focus → no tip, real
  move + enter → tip shows. Earlier false-negative on the positive control
  was HMR-mixed module state, cleared by the restart.

### Artist-switch bridge (2026-09-14, owner request)

  Every artist tab switch now replays the population animation: the current
  grid sinks + fades out (160 ms, whole-grid transition) and the new grid
  rises + fades in on the boot cascade's diagonal (260 ms, 40 ms step capped
  at 240 ms, 12 px travel). The boot dials (320 ms + 540 ms cap) are the
  rare-tier budget; a tab switch fires tens of times a day, so this runs the
  fast/subtle variant (animate-skill gate: tens/day = near-imperceptible).
  Mechanism: `displayedArtistId` in AlbumGrid holds the old list through the
  exit and flips at the bottom; the exit is a transition so rapid re-clicks
  retarget from the presentation value. Sidebar's select() leaves scrollTop
  alone on a real switch — the reset lands at the swap — and still pops to
  the top on a same-artist re-click. Search keystrokes never enter the
  bridge (keystroke tier stays silent). Reduced motion: instant swap, no
  travel, via the existing global kill switch + matching reduce specificity.
  Caption sub-label reads displayedArtistId so the fading rows keep the right
  voice (year vs artist name). Gates: check 0 errors, vitest 97 green, build
  OK. Feel-check open: play it in the app — if the 130 ms exit reads as a
  blink rather than a hand-over, raise EXIT to ~160 ms.

### Album panel gradients: per-album overrides + global switch (2026-09-15, owner request)

  Feasibility question ("let the user change the hex gradient points?") →
  yes, shipped same session plus the two follow-ups: a global kill switch
  like the playbar's, and the editor living in the space below the artwork
  column of the album edit modal.
  - Store: `ui.albumGradient` (default true) + `ui.panelGradients`
    (album id → {c1, c2} in DB hex form) in ui.svelte.ts — localStorage
    mirror + SQLite via the existing App effect/initSettings plumbing;
    `setPanelGradient`/`clearPanelGradient` helpers; Reset appearance
    restores both (toggle → true, overrides → {}).
  - Panel: ExpandedPanel's gradient effect checks the kill switch first
    (off = plain --panel-bg, overrides included — no half-state), then the
    override, then scan colors, then extraction. Corrupt override entries
    fall through to artwork colors instead of blanking the panel.
  - Editor: new PanelGradient.svelte under the ArtSelector in a `.te-artcol`
    stack — preview strip, two swatch+hex rows (native color inputs synced
    to free hex text via `normalizeHex` in gradient.ts: #rrggbb/rrggbb/#rgb),
    "Use artwork colors" reset. Applies instantly (display state, not file
    tags — never waits for Save); half-typed text writes nothing, the panel
    keeps the last good pair. Your hex, your problem: no contrast clamp.
    Off-state shows the Appearance pointer note.
  - Surfaces: Appearance pane Toggle ("Album artwork gradient") + Global
    Menu `appearance.album-gradient` row (menu.rs field with default-true,
    menu.svelte.ts handler + pushMenuState, new Rust test).
  - Gates: check 0 errors, vitest 99 green (2 new normalizeHex), cargo 104
    green (1 new), build OK. No visual pass taken — owner drives the UI.

### Screen dropper in the gradient editor (2026-09-15, owner request)

  "Can we get a dropper icon?" — the web EyeDropper API is ABSENT in this
  WebKitGTK (probed live: `'EyeDropper' in window` → false), so the dropper
  goes through the compositor instead: new `pick_screen_color` command
  (`pick_color.rs`) calls the xdg-desktop-portal `Screenshot.PickColor`
  method over the existing `zbus` (handle_token per call, empty parent —
  the picker is a fullscreen crosshair, not our dialog) and waits on the
  Request's Response signal (0 = color doubles → hex, 1/2 = silent None).
  No new D-Bus crate; only `futures-util` added (was already in the cargo
  cache). PanelGradient rows each gained a 28px pipette tool (hidden in
  browser dev, where there is no portal); a pick fills that row's hex,
  Esc cancels silently, real failures announce once. Portal interface
  verified present (`PickColor` introspected); the live click-through is
  inherently interactive — owner to pick once + Esc once in the app.
  Gates: check 0, vitest 99, cargo 104, build OK.

### Dropper fix: wrong portal interface (2026-09-15, same session)

  Click did nothing and said nothing — two bugs. (1) The proxy named the
  interface `org.freedesktop.portal.Desktop`; that object carries no such
  interface (the per-portal interfaces live there: Screenshot, FileChooser,
  …), so the call died with UnknownMethod. The earlier busctl introspection
  had used the RIGHT name all along; the code just didn't match it. Fixed
  to `org.freedesktop.portal.Screenshot`. (2) The failure was invisible:
  announcer-only + console. The editor now prints picker failures in a
  caution line under the rows — a click owes an answer. Lesson for portal
  work: verify the interface name against introspection, and busctl arg
  encoding for dicts is fiddly enough to skip (verify through the app).
  A stray `cargo test` failure in the same run passed twice on re-run
  (watcher timing flake under load, unrelated to this change).

### Dropper mark II: our own overlay picker (2026-09-28, owner request)

  Mutter's PickColor crosshair shows no hotspot, so aiming it is a guess —
  and its look is compositor-owned, unchangeable. The portal crosshair is
  gone (`pick_color.rs` deleted): `begin_screen_pick` screenshots through
  the portal (non-interactive, no compositor UI), stashes the PNG in memory,
  and opens a fullscreen overlay window (`index.html#screen-pick`) rendering
  the shot under a standard crosshair cursor with a magnifier loupe (11px
  patch at 12x, centre tile reticuled, hex readout). Click lifts the exact
  pixel via canvas; Escape/right-click cancels silently. Three commands are
  the pick's three beats (begin / take image / finish with hex-or-null);
  the hex arrives on the main window as a `screen-picked` event. Known
  limit: the shot can span monitors while the overlay covers one. Live
  click-through is inherently interactive — owner to pick once + Esc once.

### Playbar follows panel overrides (2026-09-15, owner request)

  The playbar read scan colors only, so a customized album snapped back to
  its artwork hues the moment it played. Its backdrop now checks
  `ui.panelGradients` first (same override-wins + fall-through promise as
  the panel). One deliberate asymmetry kept: the playbar keeps its
  contrast clamp on custom hex (controls must stay readable); the panel
  stays raw, per the earlier ruling. The album switch already crossfades
  gradient layers, so override edits made mid-playback glide in.

### Override orphan prune (2026-09-15, owner request)

  `library.load()` drops `ui.panelGradients` entries whose album id is gone
  from the dump — boot and every rescan, since regroups orphan mid-session
  too. Guarded to live dumps (fake-library ids must never wipe real
  overrides in dev) and no-op assigns (quiet loads don't churn settings;
  persistence rides the existing App effect). Gates: check 0, vitest 99.

### Extraction rework: two named hues, never the average (2026-09-15, owner request)

  Standardized the owner's hand-pick rule (Step 9b): hot stop = most vivid
  significant family, anchor = most colorful distant family
  (distance × chroma over regions covering >= 2% of pixels), ordered
  quiet-first. Separate bars on purpose: the old single 25%-weight bar
  starved dull-sky complements 10:1 under a vivid star (measured on the
  real thumbs: Dragons came out red-on-red, Moonflower brown-to-red).
  Findings while tuning: merge radius 48 fragments Moonflower's forest
  into khaki — 64 stays (its transitivity already protects gradients);
  thin vivid ribbons that merge away (the teal portal → sage) are accepted
  as honest acreage voice. Multiplied score demotes white title text
  without banning black grounds. v4 migration NULLs the color columns so
  the next ordinary scan refills them (covers untouched; overrides immune).
  Tests: anison pin rewritten (teal water → amber glow, same hues, new
  order) + 3 synthetic (mud rejection, quiet-first, mono fallback). Two
  u8-overflow panics caught en route — both in TEST arithmetic (`c+20`),
  fixed with casts; the habit is now documented at the pin.
  Gates: cargo 107, check 0, vitest 99, build OK.

### Extraction round 2: hue opposition (2026-09-15, owner verdict: hit or miss)

  The v4 rule (distance × chroma) fixed Dragons (navy → vermilion in the
  DB, near the owner's steel blue) but Moonflower came out TAN — a warm
  khaki fragment outscoring the teal 5% on RGB distance. Diagnosis: RGB
  distance conflates lightness with opposition, so bright mush beats dim
  complements. The anchor score is now hue-angle distance × chroma: mush
  is hue-adjacent to every hot star by construction and can never win;
  white title text dies on chroma; black grounds/snow still win by
  default when nothing colorful stands apart. Verified on the three
  target covers + six repo covers (eyeballed the two near-mono ones —
  beige-on-beige and white-on-white stay honest). v5 migration NULLs the
  colors for one more ordinary rescan. Methodology lesson learned twice:
  thumb probes ≠ full-res pipeline results (downscale blur changes
  buckets) — tune against DB outcomes, and a misplaced floor (applied to
  the winner instead of the pool) reads as "nothing qualifies".

### Version 0.11.0 (2026-09-15): panel gradients go personal

  Minor bump (new user-visible capability, no breaking changes): per-album
  gradient overrides in the album edit modal (preview, swatch+hex rows,
  portal-crosshair dropper per stop, instant apply) + global Album artwork
  gradient switch (Appearance + Global Menu) + playbar follows overrides
  (contrast-clamped) + orphan prune on every library load + extraction
  rework across two rounds (vivid-significant hot stop, hue-opposition
  anchor, quiet-first ordering; v4/v5 color refills). Nine owner customs
  already in settings; anison pin rewritten, 4 synthetic color tests.

### Import fix: dual-year TYER no longer fails the file (2026-09-15, owner report)

  Importing `~/Desktop/Remastered Albums` staged nothing: all 51 tracks
  failed lofty's default parse — their v2.3 TYER holds TWO years
  ("2003 / 2013", original + remaster), which `Timestamp::parse` rejects
  ("non-digit characters") and fails the whole file over. Sibling folders
  with normal tags imported fine, which is why the base-directory import
  "grabbed the other files without issues". Fix: `library::read_tagged`
  (mod.rs) tries the default read, falls back to Relaxed (bad frame
  skipped, everything else kept — artist/album/title/disc/track survive,
  only the ambiguous year reads empty). Routed through it: scan
  `parse_file`, tag-editor read + write, and the two artwork-browsing
  skips (embedded covers in such files are now found). Regression test
  `dual_year_tyer_does_not_fail_the_file` crafts raw v2.3 bytes and runs
  an import-shaped scan (parents as roots + `only` set): two discs in
  sibling folders land as one album, zero errors.
  Gates: cargo 109, check 0, vitest 99, build OK.

### RPM release bump 0.11.1-1 → -2 (2026-09-19)

  Step 0b re-framing landed as `a9c013e` (docs + the Cargo.lock 0.11.0→0.11.1 sync),
  which the release build then reproduced **byte-for-byte** — the bundled
  `/usr/bin/songstress` hashed identical to the installed one (`0eade7e0a3fb2c56…`,
  21,506,792 bytes). No code shipped, so the version stays 0.11.1 in all three files;
  per the 2026-09-13 rule the rebuild bumps rpm `release` instead, because 0.11.1-1 is
  already installed and a same-EVR RPM reads as "already installed" with no upgrade path.
  Release is packaging revision, not content.

### Fresh-box setup + the 0.12.0 surprise (2026-09-23, **0.12.1**)

  Fresh Fedora Workstation (GNOME, Wayland, GNOME-only box); this checkout was
  clean at 0.11.1 with no toolchain, so the evening opened with rustup
  stable-minimal + the owner-run dnf line (Tauri Fedora prereqs) + a recreated
  `songstress-dev.service` (wiped by the format) + an SSH deploy key so the
  agent can push. Wrong turn, honestly logged: assuming 0.12.0's source was
  lost in the format, the session "reconstructed" it from the installed binary
  (descriptions + appmenu skip matched) and built a kdialog→zenity fallback
  stack with a `(zenity or kdialog)` rich dep — including a real miss (only 2
  of 4 kdialog call sites converted at first; the binary-strings probe HAD
  counted 4 — enumerate dialog sites from SOURCE, never from strings). The
  first push attempt then failed on a moved remote: 0.12.0 (`ce17612`) had been
  pushed from the old machine all along — portal-first pickers, Adwaita stock
  accents, the same appmenu skip, the hamburger. The zenity stack was REVERTED
  (portal-first is the approved architecture: compositor-native on both DEs,
  zero picker deps in the RPM) and tonight's UI batch merged onto origin/main.
  Read the superseded details below as the detour, not the design. Gates after
  the dnf line: cargo 109, check 0, vitest 99, build OK; pre-rebase release
  build green (3m29s — its rich-dep verification is moot after the revert).
  Prod 0.12.0 kept running alongside throughout (MPRIS/bus-name conflicts are
  logged-and-swallowed by design). Open question: the portal FileChooser path
  was never clicked on this box — owner to test file import against it.

### GNOME portability pass (2026-09-21, unreleased — owner is moving to Fedora GNOME)

  Four changes, all verified (cargo 113, check 0, vitest 99, build OK):

  1. **Portal-first pickers** (`src-tauri/src/portal_files.rs`, new): every
     file/folder picker (add-music-folder, import files/folder, relink, cover
     art) now opens the xdg-desktop-portal FileChooser — GTK-native on GNOME,
     Qt-native on Plasma — with the old kdialog command kept as a fallback
     when the portal is unreachable (`lib.rs`: shared `AUDIO_GLOBS` so both
     backends spell the same set; portal `None` = cancel, only `Err` falls
     back). RPM `depends` drops `kdialog` (keeps `mpv`): a fresh GNOME install
     no longer pulls KDE Frameworks. Tests: `uri_to_path` (plain/percent/
     non-file) + `classify_de` (gnome/kde/mixed/empty).
  2. **Adwaita stock accent** (`app.css` + `desktop_environment` command +
     `App.svelte` once-at-startup `html[data-de]`): GNOME sessions get
     `--accent #3584e4` (dark) / `#1c71d8` (light) + matching `--active`
     wash. STOCK values only — a chosen accent writes inline vars and wins,
     and the artwork gradients (playbar/panel) never read `--accent`, so they
     keep album colors exactly as asked. Traffic lights untouched (owner will
     theme GNOME himself).
  3. **Appmenu skip on GNOME** (`wayland_appmenu.rs`): `register()` returns
     early with state=2 when `classify_de == "gnome"` — no registry
     roundtrip against a compositor that cannot answer. In-app sidebar menu
     is unaffected (it never branched on this).
  4. **Hamburger menu glyph** (`Sidebar.svelte`): the Feather cog is now
     three bars in the app's icon geometry (16-box, 1.4 stroke, round caps —
     the ✕ morph is shape-agnostic). aria-label/tooltip say "menu", not
     "settings". The `.gear` button class stays as the internal hook.

### GNOME opaque chrome (2026-09-23, unreleased 0.12.1)

  Owner decision: embrace GNOME instead of forcing glass — translucency goes on
  GNOME, KDE keeps the tuned 0.8/0.7 glass untouched. Single binary: 0.12.0's
  `desktop_environment` command feeds `html[data-de]` ("gnome" on GNOME-derived
  sessions — everything else stays glass, the reference look). app.css pins the four backdrop-revealing
  tokens (--bg-grid, --bg-chrome, --panel-bg, --panel-bg-strong) to their solid
  hues in both themes + solid fill for floating pills (.app-tip, .vol-tip);
  hairlines/washes/scrims/shadows sit ON surfaces and are untouched. Coverage by
  token census: every structural surface (stage, sidebar, playbar, pops, all
  modals, panel) goes through those tokens; component-level rgba is scrims over
  artwork/content (still correct opaque). Panel art gradient needs nothing
  (layers over the token base → becomes a tint). Window keeps transparent(true)
  — opaque content composites opaque. DOM probe (no screenshot):
  dataset.de="gnome", --bg-grid computes rgb(22, 22, 28). Gates: cargo 109,
  check 0. Owner to eyeball the result (he drives UI).

### Sidebar traffic lights → system spec, both desktops (2026-09-23, 0.12.1)

  Owner: read ~/.config/gtk-{3.0,4.0}/gtk.css + windows-assets (system traffic
  lights, gtk3/gtk4 sets identical) and wear their dot + glyph colors in-app.
  Scope verified before building: the sidebar trio is the ONLY in-app traffic
  light (tb-* classes and --tb-dot have no other consumers) — the owner's
  read was right, no correction needed. Applied to BOTH desktops (Klassy
  mirror bypassed for the sidebar): #FF5F57/#FEBD2E/#28C840, hover = rest
  (glyph appearance is the feedback), pressed #E04C44/#DC9E22/#1FAA33, glyphs
  rgba(0,0,0,.62). Geometry left alone (15px/11px vs system's 16px/6px —
  colors were the ask). --tb-* vars now serve only the SurfaceClose family.
  Hamburger + modal ✕ stay white per owner (different families from traffic
  glyphs): hamburger already --text, SurfaceClose --text-dim → --text (plus a
  stale gear comment fixed). Probe: close/max/glyph computed values match the
  SVG spec. Gates: check 0. Hover/pressed need owner eyes. (Follow-up: glyphs
  8→9→10px at owner request; 10px overshoots the system 60% proportion
  deliberately — owner-approved.)

### Bundled Inter as the standard font (2026-09-23, 0.12.1)

  The stack said Inter but the box rendered Noto Sans (never bundled, never
  installed). Owner call: bundle it, standard on both desktops (a GNOME
  variant only if he asks later). `@fontsource-variable/inter` (one variable
  file covers the used 400/600/700), imported in main.ts, stack leads with
  "Inter Variable". Offline-safe, no dnf. Probe: 600-weight check true,
  body computes "Inter Variable". Gates: check 0, build OK (woff2 subsets
  in dist).

### Global +2px type scale (2026-09-23, 0.12.1)

  Owner: +2pt everywhere. Mechanical pass: all 120 `font-size: Npx`
  declarations across 17 files (src .svelte + app.css only — no .ts) bumped
  by script, diff-verified pure (no other line touched). .5 steps preserved
  (11.5→13.5). Fixed heights/rows/gaps untouched: 15px rows and tight
  microcopy spots may clip — owner eyeballing (he drives UI). Gates:
  check 0.

### Sidebar artist marquee + loop dwell (2026-09-23, 0.12.1)

  Owner: overlong artist names slide on hover (playbar behavior), and EVERY
  text marquee dwells at the loop point before restarting. PlayBar's `marquee`
  action extracted to src/lib/marquee.ts unchanged + `hover` mode (measure on
  mouseenter, teardown on leave; rest state stays pure ellipsis). Strip +
  keyframes moved to app.css as the shared language; per-consumer triggers stay
  scoped (playbar: always-on w/ 1.2s first-loop delay + paused-park; sidebar:
  hover-gated, no delay). Dwell: keyframes hold the last 25% at the loop point
  (twin makes the jump invisible); duration = travel/0.75 so travel keeps the
  37.5px/s device-pixel speed and the rest scales with the journey. Live-row probe:
  is-over + twin + dur 18.49s for a 520px run (math checks), teardown on leave
  restores ellipsis. Caveats found while probing: none of his 66 artist rows
  overflow at 260px/15px (verified synthetically), and the first `.row` is the
  "All Artists" button (different template) — future probes must skip index 0.

### Full-bleed app icon for the Hatter dash (2026-09-23, 0.12.1)

  Owner: our dash tile reads small next to Hatter's. Measured at 48px: Hatter
  tiles are full-bleed (ink 0,0→48,48 on all 5 sampled), ours floated at
  (3,3)→(45,45) — the 86% ratio from the 2026-09-05 KRunner tuning (full-bleed
  printed over KDE's tiled rows). GNOME+Hatter is the daily driver now, so the
  margin group is out of `assets/app-icon.svg` and PNGs regenerated via
  `npx tauri icon`: new ink bbox is full 48. Same squircle language, same size
  now. If KDE tiling complains later, the SVG comment names the lever. RPM
  rebuilt from the merged tree (the earlier 0.12.1-1 was cut from the zenity
  tree — stale, never installed, same EVR reused).

### Icon glyph airs out (2026-09-23, 0.12.1, owner screenshot)

  Full-bleed overshot the other way: the near-white mic filled the tile edge
  to edge and read oversize next to Hatter's aired-out glyphs (screenshot:
  ours dominating calculator/terminal/settings). Tile stays edge to edge;
  the stencil artwork scales 0.85 about center instead (mic 253×283 → 215×240
  of 512, ~42×47% of tile). Corners untouched. PNGs regenerated, RPM rebuilt.

### Icon tile measured, not eyeballed (2026-09-23, 0.12.1, owner screenshots)

  The mic shrinks were chasing the wrong variable: measuring the screenshots'
  rendered pixels showed neighbors at 63px and ours at 72px — the dash renders
  our hicolor PNG larger per canvas px than Hatter SVGs, so full-bleed overshot
  by exactly the old margin (63/72 = 0.875 ≈ the original 0.86, which had been
  accidentally right all along). Tile scale set to measured 0.875, mic back to
  filling the tile 1:1 (predicted render: 63px = neighbors). Lesson, logged
  for the next icon-size complaint: measure RENDERED pixels (screenshot), never
  the asset — compositor/theme scaling is part of the system, and the first
  two rounds "fixed" a tile that PNG-math said was small without ever checking
  the screen. If the mic reads small now, that's glyph contrast — different
  lever, do not touch the 0.875 without re-measuring.

### Dev/prod instance separation (2026-09-24, dev-only — no version bump)

  Owner ask: use the RPM for listening while the dev instance restarts under
  it. Both builds shared `com.yossi.songstress`, hence one DB, one mpv
  socket, one MPRIS name, one appmenu name. Fix is the dev overlay
  `src-tauri/tauri.dev.conf.json` (merged via `tauri dev --config`, a
  documented flavors mechanism): identifier `com.yossi.songstress.dev`,
  productName `Songstress (dev)`. Tauri derives data/config/cache dirs from
  the identifier, so the DB, window-size, thumbs and import staging isolate
  with zero code — verified in the journal (`data_dir=...
  .com.yossi.songstress.dev`) and on the bus (both
  `org.mpris.MediaPlayer2.songstress` AND `...songstress.dev` claimed at
  once, prod untouched). What is NOT derived lives in the new
  `profile.rs` (one `is_dev(identifier)` predicate, `!= PROD` so unknown
  flavors isolate rather than collide): mpv socket dir (`songstress-dev`,
  threaded through spawn/reap/kill/socket_path), MPRIS bus/identity/
  desktop_entry, Global Menu service name (this one would have stolen prod's
  menu slot on Plasma — the hardcoded `SERVICE_NAME` const is gone).
  Dev unit ExecStart now passes the overlay; plain `tauri dev` without it
  keeps the old shared behavior, so dev runs go through the unit. Dev library
  starts EMPTY — add a small test folder once (re-do after DB migrations).
  Frontend: `document.title = "Songstress — dev"` under `import.meta.env.DEV`
  so tiled windows are distinguishable without checking the dock.
  No version bump: prod behavior is byte-identical unless the identifier
  differs, and the RPM build never sees the overlay.

### Mono dev icon (2026-09-24, dev-only)

  `src-tauri/icons/icon-dev.png`: `magick icon.png -colorspace Gray` —
  grayscale, same squircle+mic, unmistakable next to prod purple. Deliberately
  OUTSIDE the SVG → `npx tauri icon` pipeline (dev-only; regenerating icons
  will not refresh it — re-run the magick line if icon.png changes). Referenced
  only by the user-local `~/.local/share/applications/
  com.yossi.songstress.dev.desktop` (`Icon=` absolute path, `Exec=systemctl
  --user start songstress-dev` so the dash tile boots the unit,
  `StartupWMClass=` the dev id). Runtime icon setting was not an option:
  Wayland has no per-window icon protocol, the compositor looks it up from
  the app-id — which is why the icon needed the identifier overlay, and why
  the overlay doubles as the isolation mechanism.

### Dev dock entry, second half (2026-09-24, dev-only)

  The overlay identifier alone did NOT separate the dock tile: dev showed as
  a second window under the prod icon. Root cause, read from the sources
  (not guessed): Tauri only sets the GTK/Wayland app-id when
  `app.enableGTKAppId` is true, and it defaults to FALSE (tauri-utils
  config.rs) — so both instances reported the same fallback id and Mutter
  grouped them. Fix: `"app": { "enableGTKAppId": true }` in the DEV overlay
  only (schema key is `enableGTKAppId`, all-caps GTK). Prod untouched — its
  dash matching works as before. Follow-on collision, caught by reading tao
  0.35.3 (it `app.register()`s the id): the dev GTK application now owns
  `com.yossi.songstress.dev` on the bus, which is exactly the name the
  appmenu claimed — so the dev menu service moved to the `.menu` sibling
  (`profile::appmenu_service_name`; KWin is handed the pair explicitly, the
  name itself is arbitrary). Proof without a screenshot: the bus now shows
  `com.yossi.songstress` (prod GTK), `com.yossi.songstress.dev` (dev GTK —
  nothing else claims it anymore), `...dev.menu` (dev menu object), and both
  MPRIS names. Compositor has a distinct app-id + a basename-matching desktop
  file with the mono icon, which is the full recipe for a separate entry.
  Known quirk: a lingering dev process now holds its GTK name, so a second
  dev launch may attach to the old instance instead of opening fresh — kill
  the old one first (the unit restart already does).

### Dev dock entry, root cause + fix (2026-09-24, dev-only)

  enableGTKAppId alone did NOT separate the dock: still one purple icon,
  two dots. Root cause, read from gnome-shell's `shell-window-tracker.c`
  (fetched, not guessed): `get_app_for_window` checks WM_CLASS *first* —
  "canonical if it does" — and only then sandbox id, app-id, pid,
  startup-notification, group. The class defaults to the binary basename,
  "songstress" for BOTH builds, and the RPM desktop claims
  `StartupWMClass=songstress`, so the dev window matched prod before the
  (correct) app-id was ever consulted. Proven by elimination, not just
  code-reading: the bus showed the dev GTK id correctly claimed while the
  dock stayed grouped, and the pid/startup/group paths cannot return prod's
  app for a systemd-launched foreign binary — WM_CLASS was the only path
  that could produce what the screenshots showed.
  Fix: `glib::set_prgname(Some("songstress-dev"))` in `main.rs`, gated on
  the TAURI_CONFIG identifier (the merged JSON the CLI injects — same
  object the app boots from, no second mechanism; RPM environ verified to
  carry zero TAURI_ vars, missing/unparseable defaults to prod). Must run
  before GTK init. New `glib = "0.18"` dep (already in the tree via the GTK
  stack — ~zero build cost; note its `set_prgname` takes Option, E0308 on
  the first try). Dev desktop's StartupWMClass restored as
  `songstress-dev` — now descriptive, belt-and-suspenders with the app-id
  path, which finally gets its turn and resolves the mono-icon desktop.
  Both entries confirmed separated in the dock; bus shows prod GTK, dev
  GTK, dev `.menu` object, both MPRIS names.

### Empty-state app mark (2026-09-24, owner ask, two rounds)

  The welcome screen gets the stencil mic above "Welcome to Songstress"
  at 75% opacity — glyph only, no squircle (then bumped 96px → 144px on
  request). Geometry mirrors the #stencil mask group in assets/app-icon.svg
  (white = paint, black = knockout); if the mic ever changes there, mirror
  it in EmptyState.svelte. Two gotchas, both caught before shipping: the
  first cut referenced a standalone asset file via <img>, which died twice —
  currentColor cannot cross the image-document boundary (the glyph would
  have fallen back to CanvasText, near-invisible on the dark glass), so the
  SVG is INLINED and the asset file deleted; and the paint is currentColor
  (via `color: var(--text)`) rather than the tile's fixed near-white, which
  would wash out on the light theme. Error branch untouched: a failed scan
  is a different state with its own screen.
  Buttons share one width: measured live (177 vs 94px), `min-width: 180px`
  on the shared rule equalizes by construction — verified 180/180 via the
  devctl bridge.
  Ink rhythm + italic path: the 10px box gaps were uniform but the
  headline's line slack + descender read ~4px airier, so `.primary` carries
  `margin-top: 4px` (measured live: 14px below vs 10px above); the
  single-folder path renders in `<em>` via a split branch, with
  `break-word` (not `anywhere`, which would rewrap the sentence) so long
  paths wrap instead of overflowing.

### Track modal Genre→Track# cascade after the +2px scale (2026-09-24, owner screenshot)

  Symptom (screenshot): disc-total box wrapped full-width onto its own
  line, Year box giant, horizontal scrollbar in the modal body. One root,
  three symptoms in cascade: the +2px scale pushed the Track# row's minimum
  (four 56px-min boxes + wider labels + ofs + gaps) a hair past the field
  column, so the disc total wrapped; the --cluster mirror then measured the
  WRAPPED boxes and concretized that width; the year row rendered it (giant
  box) and overflowed the column (x-scroll). Fix is one number in
  FieldGrid.svelte: nums boxes `min-width: 56px → 48px`. 48 still clears 3
  digits at 17px with air, and flex grows the boxes whenever room exists, so
  healthy layouts render pixel-identically — the change only moves the wrap
  cliff 32px further out, with a tripwire comment naming the cascade for the
  next scale bump. Gaps untouched (ruled measures). Heals live over HMR: the
  ResizeObserver re-fires on the reflow and re-measures the healthy row.

### Music folders modal: narrower + centered Add (2026-09-24, owner ask)

  Width 560px → 440px (a path list + one button never earned the wide
  box; long paths ellipsize either way). The footer already SAID
  `justify-content: flex-end` but had no `display: flex`, so the button
  sat left — now a real flex row, centered per the ask.

### Modal pass (2026-09-24, owner-driven)

  Track tag editor: +2px cascade fixed (one number, see entry above).
  Music folders: 560px → 440px, Add centered (footer's justify-content was
  dead without display:flex). Remaining modals (album editor, imports,
  About) + settings stack reviewed by owner on screen — fine, no changes.

### Single instance (2026-09-24, owner report from KDE, **0.12.2**)

  Report: opening the app on a second desktop/activities while one ran did
  not crash the first, but mpv warned and stayed silent until restart.
  Mechanism (no guessing needed): no singleton existed, so the second
  process ran full setup — `reap_previous` killed the FIRST instance's mpv
  (pid guard passes: same cmdline shape), rebound the SAME socket path,
  and the first engine's reader died on the closed socket. Every later
  playback command failed; only a restart re-spawned its engine.
  Fix: first-party `tauri-plugin-single-instance` (v2.4.5), registered FIRST
  per the plugin docs — the second process exits in plugin setup, before our
  DB/mpv setup ever runs, after handing off; the first focuses its window
  (unminimize+show+focus, best-effort under Wayland focus rules). Singleton
  key defaults to the bundle identifier, so dev (.dev) and RPM stay separate
  singletons and coexist — verified dev boots clean; prod gets it with the
  0.12.2 RPM rebuild. No live second-launch test here: it would steal his
  focus mid-session. Verify by launching a second copy (should focus, never
  duplicate) — on this GNOME box now, on KDE/d second desktop later.

### Post-import landing replaces the auto modal (2026-09-24, owner ask)

  Importing no longer opens Manage Imports by itself: the app switches to
  the first imported album's artist tab and expands that album in the grid
  (staged badge + per-album doors are the receipt now). Modal opens only on
  direct invocation (menu, badge, album menu), where the report band still
  renders. Mechanics: `importMusic` reads the staged plan (exact albumIds,
  plan order = "first on the list"), waits for the post-scan dump to carry
  the row (the scan finished inside the invoke but the dump refresh races
  our return — 15s poll, quiet give-up), then `revealInGrid` replicates the
  sidebar tab switch (menu layers closed, search cleared, expansions
  collapsed) and fires `ui.gridReveal` 260ms later — after the tab's 160ms
  switch bridge — guarded so it never yanks the user back if they moved on.
  The grid consumes the request through `toggleExpand(id, "albums", true)`:
  an open-only flag reusing the full choreography (ghosts, glide, cover
  warm) instead of duplicating it; a re-import onto the open album just
  travels. Pure "already own this" imports land on the existing album with
  the sentence heard via the announcer. No live end-to-end test here (needs
  real files + driving his window) — verify by importing: tab switch,
  expanded album, modal stays shut.

### Sidebar pending-imports filter row (2026-09-24, owner ask)

  "Songs pending import" row between search and All Artists, only while the
  staged pile exists: name + pending-song count (staged albums' tracks) with
  the same accent dot the dotted rows wear. Click toggles `ui.pendingOnly`
  (session-only, aria-pressed, tooltip both ways), ANDing with search; the
  filter gates on pile non-emptiness AND an auto-clear effect follows the
  pile, so an emptied pile can neither blank the list for a frame nor
  re-filter the next pile by surprise. Selecting any artist clears it (the
  filter finds, the selection shows — same contract as search), and the
  post-import landing clears it too. No new CSS: .row/.name/.count/.pending.

### Pending filter: tab → caption (2026-09-24, owner correction)

  First cut was a full `.row` tab; owner: not a tab — the quiet `.empty`
  voice, pinned under the search it filters with (margin borrows scanNote's
  negative-top idiom: nearer the search than the list). Same behavior
  (toggle, aria-pressed, tooltips, auto-clear, select-clears), accent text
  for the ON state instead of the row wash.

### 0.13.0 shipped (2026-09-24): commit bfefb8a, RPM built

  Minor (not patch): import landing + pending filter are user-visible
  features; single instance the fix; dev isolation rides along (prod
  behavior identical). Committed + pushed to main; note GitHub moved the
  repo to jmbentolila/Songstress (capital S) — local remote still points
  at the lowercase URL, pushes land via redirect, update it when convenient.
  RPM: src-tauri/target/release/bundle/rpm/Songstress-0.13.0-1.x86_64.rpm
  (built, NOT installed — install is his call; it carries the singleton,
  so first launch after install owns the name).

### Cold-press play: context-aware start (2026-09-25, owner ask)

  Cold play (nothing current: Space / playbar / MPRIS-less press) always
  started `library.albums[0]` — the oldest album overall, Bat Out of Hell
  (1977) — whatever the tab. Now `coldStart()` in playback.svelte.ts:
  shuffle on (any stage) → uniform random track across the visible context
  (All Artists = whole library, artist tab = that artist; the backend still
  anchors it and builds the stage pool behind it); shuffle off → track 1 of
  the first album on the grid (All Artists = `globalAlbumOrder`, verified
  identical to the grid comparator; artist tab = that artist's oldest).
  Frontend-only, single call site, hot-reloads — no restart, no Rust.

