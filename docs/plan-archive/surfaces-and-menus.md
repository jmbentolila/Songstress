# The surface template: dismissal, and one press = one verb (2026-09-01) (plan archive)

Moved out of `PLAN.md` on 2026-10-24 so the plan file stays readable in one pass.
This is the record, not the state: current state lives in `PLAN.md`, and this
file is the evidence behind it. Sections are kept verbatim, in their original
order. The index of every section is `README.md` next to this file.

## The surface template: dismissal, and one press = one verb (2026-09-01)

Started as "remove the red dot from the imported-music modal, put an ✕ top-right"; it turned
into the rule that was missing, because the same objection applies to every floating surface.

- **Two dismissal families, one verb each.** The traffic-light dot is the WINDOW's close — the
  user's own KWin close colour, in the window's own circle — and it now lives only in the sidebar
  header, where it is true. Modals AND popovers dismiss with `SurfaceClose`, rebuilt as the boxed
  ✕ at the head's right edge: 28×28, radius 7, the sidebar gear's own glyph (16-unit box, 8-unit
  cross, 1.4 stroke, round caps, drawn SVG — a text ✕ is the third typographic family Music
  folders was already caught using), dim → hover wash → `--active` press, inset accent ring, no
  transition. The settings stack had been written up as "the exception" that kept this shape; the
  exception was the rule and the dot was the deviation.
- **Why the dot was wrong, in one line:** it claimed a verb the surface doesn't have. Dismissing
  the queue does not quit the app, and a circle in the close colour at the close spot says
  otherwise. Fidelity to Plasma means mirroring what the platform does for a *dialog*, and Plasma
  dialogs do not carry a second window button.
- **Flush, not floating:** the box's right edge is the content's right edge (measured 17px = 1px
  border + 16px padding), and its centre is the title's (119.5 / 119.5). About has no head row, so
  it takes the corner (8px) instead. `Clear` in the queue header moved off the far edge for the
  same reason it used to be there: a destructive verb must not neighbour dismissal.
- **Rolled out to all six surfaces** (Manage imports, Music folders, Tag editor, About, the
  equalizer popover, the queue popover); `.q-x` stays a BARE ✕ — boxed + head + always-visible vs
  bare + row + hover-revealed is what tells "put this away" from "remove this row" now that both
  are crosses.
- **One press, one verb.** Escape (and the scrim, and the ✕) does exactly one thing: while a modal,
  popover or the context menu is up, that surface answers and the sidebar's global key router
  stands down — so closing a modal opened from the Library pane no longer pops the pane off the
  stack too. Measured: open → Esc → modal gone, Library pane still active; Esc → root; Esc →
  closed. The router's OTHER keys are gated by the same predicate, which also ends `s` pushing the
  settings stack in behind the About scrim. The context menu had no Escape owner at all (the key
  popped a settings level instead of dismissing the menu on screen) and now has one.
- **Why a predicate and not `stopPropagation`:** all six handlers listen on `window`, and
  same-node listeners all run regardless — only `stopImmediatePropagation` cuts them short, and
  Svelte re-attaches them on update so order is not guaranteed. `surfaceOpen()`
  (`src/lib/stores/surfaces.svelte.ts`) makes the precedence a fact each handler checks. This raced
  twice before it was named: the earlier fix moved About's handler INTO the router, which made
  About correct and left everyone else wrong.
- **A layer holds its own content while it animates out** (his bug report: accent swatches
  flashing on the way back from the EQ preset submenu). Not a timing bug — an identity bug: the sub
  layer is `{#if menuSub === "eq-preset"} … {:else} <accent picker> {/if}` and `menuSub` flips to
  null at t=0 of the back, so the layer re-renders into the wrong branch while still on screen. The
  mirror case flashed the preset list, and the detail layer had the same flaw as a blank panel
  sweeping across. `heldDetail`/`heldSub` + `shownDetail`/`shownSub` drive rendering, titles and
  branch conditions; `atDetail`/`atSub`/`inert`/`aria-hidden`/`class:active` still read the live
  state, so the hold is paint-only. Measured after the fix: back out of eq-preset → 0 swatches, 10
  preset rows (was 12 swatches, 0 rows).
- **Restart geometry: what a Wayland client can and cannot remember — and why nothing was
  added.** `tauri-plugin-window-state` was tried with `SIZE|MAXIMIZED` and deliberately NOT
  `POSITION`, because a Wayland client never learns where the compositor put it: measured on one
  window, Tauri's `outer_position()` = `(0,0)` while KWin reports `x=560 y=290`, and
  `inner_size()` = `2560×1600` physical for a 1280×800 CSS window (Tauri scale 2 — the plugin's
  units are its own and self-consistent, which is why SIZE round-trips and POSITION does not).
  Saving position would have stored `(0,0)` and `set_position(0,0)` on next launch, moving the
  window to the top-left corner: worse than nothing. And SIZE does not survive the case that
  matters either — the plugin writes on `RunEvent::Exit`, which a kill never reaches (verified:
  after three rebuild-kills the state file still did not exist and the window came back
  1280×800). **Reverted the same day:** the owner's tiling clips and resizes on drop, so ONE
  drag fixes both axes, and a debounced save on `tauri://resize` to cover the kill path was
  declined as a background thread + disk writes for nothing. A KWin window rule and a seeded
  state file were offered and declined too. The protocol is manual and lives in AGENTS.md — do
  not re-add the dependency without a case where the size is not already free.
- **Version files are watched.** Bumping `tauri.conf.json` / `Cargo.toml` makes `tauri dev`
  rebuild and RESTART the app — which, per the rule above, costs the owner a window re-place. Do
  the bump before he places the window, or tell him it is coming.
- Gates: `npm run check` 0 · 73 vitest · 81 `cargo test --lib` · `npm run build` clean. DESIGN.md's
  dismissal section is rewritten ("Two families, one verb each" + a new "One press, one verb") and
  is now the authority; the old "one family, two dots" text is gone.

### The equalizer popover's head, rebuilt (2026-09-01, same template pass)

The head now reads **identity + state + dismissal** — `✓ Equalizer` at the content's left edge,
the preset name (read-only, Micro) hugging the ✕ at the right — the curve stays the body, and
`Playback settings ›` moved to the BASE, under the seam.

- Why: the route out used to be the head's first item, which made this the only surface in the
  app whose first row was *navigation* rather than a name. And the old justification for the
  inverse ordering ("an instrument surface keeps its master at the base") bought nothing the head
  grammar does not already buy — leaving belongs with the exit, and what-a-surface-is belongs at
  the top. DESIGN.md's "Ordering is genre, not law" passage is rewritten to say so; the popover
  and the Playback pane now agree instead of deliberately disagreeing.
- The seam's job changed with it: it used to separate the work from its *state*, now it separates
  the work from the *exit*. `.own` lost `cursor: default` (it is actionable again) and the route's
  negative margins were re-aimed: `margin: 0` on the button, `-6px` on the footer row so the wash
  bleeds to the padding edge while the label stays aligned with the head's left edge.
- No `justify-content` on the head: the preset's own `margin-left: auto` spends the slack, so the
  grouping is owned by the item that forms it rather than distributed between three.
- Measured at 1200×660 (DOM probe, not a screenshot): toggle x=772 (content edge), preset ends
  1129, ✕ 1141–1169 with 17px to the popover's outer edge (1px border + 16px padding — flush, as
  every surface now is), 12px preset→✕; route 766,533 against a seam at 526 and a head bottom of
  355. Popover 431×255.


## The playbar volume slider sheds its native chrome (2026-09-03)

- User finding: at 100 the knob sat flush but a sliver of unfilled track stayed
  visible to its right. The cause was shape, not math — `.volume input` was the last
  natively-rendered range in the app (`appearance: auto` + `accent-color` only), and
  WebKitGTK's native paint insets the thumb's travel inside a full-width track. The
  settings panes had already solved this (glass slider: `appearance:none`, 4px rounded
  track, value-sized `accent → hover` gradient, accent dot). The volume row gets the
  same treatment, inline `style:background` gradient on `playback.volume` — at 100 the
  fill is the whole box and there is no sliver to be. The seek bar is still native and
  carries the same quirk at track end; deliberately not touched (owner's call pending).
- **Gates:** check 0/0 · vitest 84/84 · build ok. No restart (hot-reloaded).

## The broadcast mirrors the pane — Global Menu rebuild (2026-09-03)

- The ask: bring `menu.rs` (which now feeds ONLY the Plasma Global Menu — every
  sidebar pane is bespoke) up to date with what the sidebar actually offers, sections
  and all. Owner decisions, all five: sliders stay in-window (with a door back),
  manage-imports opens the modal, footer resets join, View renames to Appearance,
  and the parked 3-state theme fix rides along.
- **Separators are a model concept now.** `MenuItem` gains `separator` (and `radio`)
  with serde defaults; `menu.rs::build()` emits breaks that mirror the panes' group
  gaps — Playback: transport | repeat/shuffle | equalizer | reset (3), Library:
  import | scan | storage (2), Appearance: theme | gradient+accent | more+reset (2).
  `menu_dbus.rs` exports them as `type:"separator"` (visible, never enabled — a
  stray Event hits the `!enabled` early-return), and checkables report
  `toggle-type: "radio"` for the theme trio (KDE's importer degrades an ungrouped
  radio to a checkable, so this is the safe way to say "mutually exclusive").
- **`MenuState` de-flattened:** `themeDark: bool` → `theme: "light"|"dark"|"system"`
  — the MODE, pushed from `ui.theme`, not `resolvedTheme()`; the App.svelte effect
  tracks the mode + `playbarGradient`. `anyStaged` → `stagedCount` (albums, same
  reduce the sidebar door's tail uses — it gates the row; the owner declined
  putting the count in the label, so the label stays static). Re-checked
  the rest: shuffle/repeat/eq were already honest; playing's Option<bool> is a
  genuine tri-state feeding a binary label, fine.
- **New doors, same setters.** `appearance.more` / `appearance.accent` open the
  sidebar stack at the Appearance pane / its accent sub layer (state-only —
  `openSettingsLayer`; the window typically lacks keyboard focus when the panel is
  clicked, so there is no focus to hand off). `library.manage-imports` mirrors the
  sidebar's door into the modal; `library.save-imports` is gone from the model (the
  modal owns Save/Discard now; ExpandedPanel's context menu keeps calling
  `saveImports()` directly — that store function stays). `help.about` was a silent
  no-op in the dispatcher (`() => {}`) — it now calls `openAbout()`; the null
  opener fallback covers "no trigger element to restore focus to."
- **Resets moved to the stores** (`resetAppearance` in ui.svelte.ts, `resetPlayback`
  in playback.svelte.ts) — the footers and the Global Menu rows now call one
  function each; "the two surfaces cannot drift" is the repo's own principle and
  this was the drift. `cycleTheme` deleted (nothing cycles a tri-state when the
  menu shows all three).
- Sidebar fallout: `rootPanes` filter now also drops the Rust-owned `"appearance"`
  (the constant and the menu id collide by design — `openDetail("appearance")`
  renders the bespoke pane); `detailItems` filters separators out and renames its
  save-imports clause to manage-imports. The generic irow fallback loop stays
  correct-by-filtering even though no pane currently uses it.
- Verified on the wire (dbus-send depth 2): all new labels present, 2 separators in
  Library's export, `3× "radio" + 1× "checkmark"` in Appearance's. Two owner
  corrections followed: manage-imports keeps its STATIC label (the count lives only
  in the sidebar door's tail — the owner declined it in the menu row, so the test
  now asserts no count), and `help.about` was a silent no-op in the dispatcher —
  it now calls `openAbout()`. About-open verified end to end: a dbusmenu
  `Event(400,"clicked",…)` brought `.ab` into the DOM (`true`), and `.ab-close`
  took it back out (`false`). Note `menu_activate` is the Rust-native path for
  playback ids only; `help.about` re-emits to the frontend as `menu-action`.
- **Gates:** check 0/0 · vitest 84/84 · cargo --lib 83/83 (4 menu tests rewritten to
  look items up by id — the positional-index tests were exactly what broke today;
  new tests pin the separator counts, the one-checked radio invariant, and the
  mode-not-resolution theme report) · build ok. Rust change → cargo rebuilt and the
  app restarted (cost: one window drag).

## Right-click everywhere: the context-menu pass (2026-09-03)

- **Grid tiles got a menu.** Albums already had one on the expanded panel's
  header; the tile was a dead right-click. Extracted ONE builder —
  `src/lib/albumMenu.ts` — and both doors (tile, panel header) call it; what
  used to be two hand-maintained copies is one list again, unable to drift
  (same principle as the store-owned resets).
- **"Open containing folder" — and paths never enter the webview.** New Rust
  command `reveal_container(albumId | trackId | artistId)`: the menu says
  WHICH row, the DB says WHERE. A track resolves to its FILE (`dolphin
  --select` → folder open, file highlighted); an album to artwork's
  `dominant_dir` (majority rule, multi-disc safe); an artist to the DEEPEST
  folder holding all of its files (`common_parent`, component-wise — the
  `/m/b` vs `/m/bee` string-prefix trap is pinned by a test). Vanished file →
  parent dir; vanished dir → honest error; no dolphin → xdg-open. Verified
  live on all three levels.
- **Bug the artist work caught: the Dir demote.** The first cut branched on
  `target.is_file()`, so an album (whose resolution is ALREADY a directory)
  got demoted to its PARENT — revealing the artist folder above the album,
  and looking plausible enough that the first live check missed it. The
  resolver now returns `Container::File | Container::Dir` and only a FILE
  target is ever demoted. The artist-level check doubles as the regression:
  the same row now opens the album folder itself, not one above it.
- **Separators are a context-menu concept too.** `MenuItem.separator` +
  shared `SEP` (same vocabulary as the Rust Global Menu model); the renderer
  draws an inset hairline and keys the each-block BY INDEX — SEP is one
  shared object and Svelte's default identity keys would collide the moment
  a menu holds two breaks. Album: Edit | playback pair | container | import-
  state (the last break only when there IS an import state). Track: Edit |
  play verbs | container.
- **Playback pairs replace single verbs**: album `Play album next` became
  `Play album` (first file that exists — a missing opener must not make the
  row a dead click) + `Add album to queue` (front, the old behavior); tracks
  got `Play now` above `Play next` / `Add to queue` — same
  `playTrack(albumId, indexById)` call the row-click path uses, so the
  search-filtered view cannot play the wrong offset.
- **Right-click selects (tracks)**: the menu acts on what you pointed at,
  and the keyboard verbs (Enter/Delete) come alive on that row. Artists
  deliberately do NOT select on right-click — selecting (a heavy filter
  change) just to look at a menu is a worse surprise than on a track row.
- **Artist rows got the minimal menu** — one item (an artist has no tags of
  its own and nothing singular to play). "All Artists" is left without one:
  it's a control, not a folder, and "the container of all artists" is a
  claim the row shouldn't silently make.
- **Dismissal grew up**: the menu now dies on scroll (captured on the whole
  document — the grid scrolls in its own container), `focusin` outside the
  box (Tab walks away), and window `blur` (a panel-menu click sends no DOM
  event; without blur the menu floats over a library you walked away from).
  No open-time race: the opener's mousedown focus lands BEFORE the
  contextmenu event registers the listeners.
- **Dev-loop notes:** plain `.ts` module chains (albumMenu.ts, the stores)
  can HMR half-dead — the menu kept rendering the pre-separator list until a
  `location.reload()`; when a DOM probe contradicts a just-landed edit,
  reload before debugging. Synthetic `blur`/`scroll` fired in the SAME tick
  as the open missed the not-yet-attached listeners — dismiss probes need
  open and the trigger in separate evals.
- **The staged-album doors all point at the modal.** The album menu's direct
  `Save to library`/`Discard import` became one `Manage imported music…`
  opening the modal AT that album, and the expanded panel's `Imported` pill
  became a BUTTON doing the same (the tile's corner badge stays a label — it
  lives inside the tile's own button). `openImportManager(albumId?)` gained
  the address: the row expands even in a tall pile, `scrollIntoView`s, and
  wears a 1.6 s accent ring (`mi-attn`, driven by STATE — a classList string
  is invisible to Svelte's CSS scanner and rightly warned). The focus flag
  is consumed on read, and close clears it. Rationale is the repo's own:
  the DESTINATION is what the decision is about, and only the modal states
  it per album — a menu applying the move blind skipped the one fact the
  user was deciding on.
- **Gates:** cargo --lib 85/85 (new: artist arm of container_target,
  common_parent incl. the string-prefix trap) · check 0/0 · vitest 84/84 ·
  build ok. Rust change → cargo rebuilt → one more app restart (one more
  window drag).

### The pending-import dot on artist rows (2026-09-03, same pass)

- An artist with staged albums now wears an accent dot after the count in
  the sidebar list (`.count.pending::after`, the app's "a decision lives
  here" tier, same badge language as the mode buttons), with a hover title
  spelling the claim: "N albums — M awaiting import". Trailing placement so
  the count keeps its column alignment. Derived once as `stagedCounts`
  (artistId → staged albums) next to `albumCounts`. "All Artists" stays
  undotted — the pile is the artist's story, and the Library pane already
  owns the global number.
- Probe note: the staged pile is a MOVEABLE target during a session — the
  owner's own testing moved it off Aimer mid-verification, and the first
  probe dutifully "failed" against a stale assumption. Dump the actual
  state (every row's class + the staged tile's caption) before debugging a
  "missing" indicator.

