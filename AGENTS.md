# AGENTS.md — Songstress

Instructions for AI agents (and humans) working on this codebase. Read this fully
before changing anything. The roadmap + full implementation record lives in
**PLAN.md** — read that too, and update it when you finish work.

## What this is

Songstress — a local music player for Fedora 44 / KDE Plasma 6.7 (Wayland).
Album-grid-centric, MusicBee-style browsing, glassmorphism UI. Personal use first.

Stack: **Tauri 2 (Rust) + Svelte 5 + TypeScript** frontend, **MPV via JSON IPC** as
audio engine (Phase 3), **SQLite + lofty** for the library (Phase 2). No router, no
CSS framework — hand-rolled CSS with custom properties.

## Commands

```sh
npm run check          # svelte-check (must be 0 errors before finishing)
npm test               # vitest — row model, sorting, (rust: see below)
npm run build          # production build (also validates CSS/TS end to end)
npm run tauri dev      # full app (prefer the systemd unit below)
cd src-tauri && cargo test --lib   # Rust color-extraction tests
```

Dev server runs as a **systemd user unit** (survives agent shells):

```sh
systemctl --user restart songstress-dev     # start/refresh the app
journalctl --user -u songstress-dev -f      # logs
```

Closing the app window kills `tauri dev` by design — restart the unit.

**Restart protocol (owner-set, 2026-09-01).** A restart brings the window back
CENTERED, which is not where he keeps it — his tile is the **bottom-left quarter**
(last measured: `x=0 y=690`, 1200×660 logical, in a 3-part grid: terminal + app in
the left column, browser in the right). Nothing can put it back programmatically:
a Wayland client cannot see its own position (measured — Tauri's `outer_position()`
returns `(0,0)` while KWin reports `x=560 y=290`), and KWin exposes no DBus
move/resize on Plasma 6.7. So after ANY restart: **stop, tell him the window is
misplaced, and wait for his green light that he has moved it back** before driving
or measuring anything in it. Do NOT add a KWin window rule, seed
`.window-state.json`, or call `set_position` to work around this — he declined all
three; the manual move IS the protocol.

Nothing persists the geometry, on purpose (2026-09-01): `tauri-plugin-window-state` was added,
measured and **reverted**. A Wayland client cannot see where the compositor put it — Tauri's
`outer_position()` returns `(0,0)` while KWin reports the real `x/y` — so a client-side
"remember position" is not just unavailable, saving `(0,0)` and restoring it would shove the
window into the corner. Size *could* be remembered, but the plugin writes only on a clean exit
(a `systemctl restart` / `tauri dev` rebuild kills the process, nothing is saved), and his
tiling clips **and resizes** on drop — one drag fixes both axes. So: no dependency, no KWin
rule, no seeded state file (all three offered, all three declined). Do not re-add one without a
case where the size is not already free.

Frontend work needs no restart at all: `tauri dev` hot-reloads Svelte components, stores and
CSS. Restart only for Rust changes (cargo rebuilds anyway) or `contain:`/containing-block
changes — and remember the **version files are watched too**: bumping `tauri.conf.json` /
`Cargo.toml` restarts the app and costs him a drag. Bump before he places the window, or warn
him.

**Reviewing a loading state without a scan.** `node tools/devctl.mjs eval "__skel(true)"`
holds the grid + artist-list skeletons in place (`main.ts`, DEV-only, sets
`library.devLoading`; `__skel(false)` releases them). Do NOT try to re-enter the state by
clicking Rescan after a reload: a *parked* scan (`SONGSTRESS_SCAN_STALL_MS`, PLAN.md) still
holds the backend's `SCAN_RUNNING`, so the retry fails instantly with "scan already running"
and only the console knows. Re-parking needs a unit restart — i.e. his window drag again — so
park once, then iterate with `__skel`.

## Layout

```
src/
  App.svelte              shell: stage (grid+sidebar+playbar overlay + drag strip)
  app.css                 theme tokens, .glass, scrollbar, noise garnish
  components/             Sidebar, AlbumGrid, ExpandedPanel, PlayBar, TagEditor, MusicFolders, ContextMenu, EmptyState
  lib/
    types.ts              Artist/Album/Track/PlaybackContext/Theme
    buildRows.ts          row-model grid (pure, vitest-covered)
    sort.ts               sortKey (article-stripping, diacritic folding)
    fakeLibrary.ts        Phase-1 data (seeds incl. multi-disc + compilation)
    artColors.ts          invoke wrapper -> Rust album_colors
    stores/*.svelte.ts    runes stores: ui, library, playback
    window.ts             isTauri guard + window control helpers
src-tauri/src/library/    db, scan, artwork, tags, import, settings (Rust)
src-tauri/src/watcher.rs  inotify live watching (Step 7d; multi-root Step 7c)
src-tauri/src/lib.rs      commands: album_colors (dominant-color extraction)
public/covers/            album art for the fake library (real folder.jpg files)
```

## Hard-won gotchas — do not rediscover these

- **Svelte 5 runes stores** live in `*.svelte.ts`. `$effect`/`$derived` rules apply.
- **HMR rots.** After a long series of hot edits the webview degrades: stale
  `bind:clientWidth`, dropped component CSS, phantom layouts. Non-atomic writes to
  watched files (sed/python rewrite of app.css) make it worse. Fix = cold restart:
  `systemctl --user restart songstress-dev`. Don't debug phantoms before restarting.
- **A COLD start can also drop component CSS — and a reload will not fix it.**
  The webview can request `X.svelte?svelte&type=style&lang.css` before the plugin
  has compiled its parent; the plugin has no metadata for that id, Vite falls
  through to the static-file handler (which ignores the query) and the "CSS" is
  served as the component's RAW SOURCE, so the style tag never injects. Result:
  the shell renders layout-less (full-width grid, playbar stacked vertically)
  while components requested a moment later are fine — it reads as "only one
  pane broke". Diagnose: `curl -s 'localhost:1420/src/App.svelte?svelte&type=style&lang.css' | head -c 200`
  — raw `<script>` means uncached. Fix: `server.warmup.clientFiles` (added to
  vite.config.js); `touch`ing the file is the manual version. `location.reload()`
  does NOTHING here — the id stays uncached, so don't waste a restart cycle on it.
  **The global sheet rots the same way, silently:** `src/app.css` can keep serving a
  module whose `__vite__css` is the EMPTY string (2026-09-02, after a long edit
  series). Nothing 404s and the components still look styled, but every theme token
  reads empty — the tell is `getComputedStyle(document.documentElement)
  .getPropertyValue('--gap')` coming back `""`, or a `.sk`/`.glass` probe returning
  nothing. `touch src/app.css` re-emits it in one second; a screenshot would take a
  restart cycle and show you a page that merely looks a bit flat.
- **`overflow: hidden` is still a scroll port.** Programmatic `.focus()` on an
  element inside a layer that is `translateX(100%)` SCROLLS the clipper
  (`.stack.scrollLeft` → ~202) and nothing ever scrolls it back: every layer
  shifts sideways at once, so pane titles print over each other and the deeper
  layer's controls leak into view. The 4-layer stack therefore uses
  `overflow: clip` AND `focus({ preventScroll: true })` (`focusFirst`) — the
  layer is animating into place anyway, so it never needs the scroll. If the
  settings panes ever look "printed twice", read `.stack.scrollLeft` first.
- **Component CSS can be emitted unscoped.** TitleBar once emitted a global sheet,
  leaking `.title { text-align: center }` into every component. (TitleBar was
  deleted in Step 8 — its `tb-*` traffic lights now live in Sidebar; the
  namespace lesson stands.) If styles leak again, namespace the classes — don't
  fight the compiler. Audit served sheets:
  `curl 'http://localhost:1420/src/components/X.svelte?svelte&type=style&lang.css'`
  and check every selector carries a `.s-…` hash.
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
- **Probes never compute paths on owner-owned files.** A scratch scheme derived
  from the target's own path (`path.with_extension("tmp")` → a sibling that can
  collide with the original, cleanup that deletes "the temp") cost the owner a
  real mp3 in 2026-09-04 (backup saved it; a temp test's rename landed on the
  original and its cleanup deleted it). Rule: copy the file to /tmp FIRST and
  probe the copy; never rename onto, write in, or remove_file a path under
  ~/Music from test or debug code. Same energy as the panic gotchas: user data
  gets character ops, char boundaries, and temp+rename — always.
- **mpv must be spawned with `--no-config`** — the user's `~/.config/mpv/mpv.conf`
  is a broken Windows config (d3d11, C:\ fonts).
- **No GTK dialogs.** File/folder pickers must go through `kdialog` (see the
  `choose_music_folder` command) so they render native KDE style. Don't reintroduce
  tauri-plugin-dialog / rfd — its GTK chooser looks GNOME-ish (user rejected it).
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
- **NEVER await IPC commands in the mpv reader task** (mpv.rs `spawn_reader`).
  It is the task that reads responses off the socket — awaiting a command
  there stalls all reads until the 10s timeout fires (mpv executed instantly;
  only we were blind). Reader-task side effects are fire-and-forget (`fire()`).
- **mpv playlist is a rolling 32-track window** (`PLAYLIST_WINDOW`), never the
  whole play order — all-artists shuffle pools are thousands of entries and
  per-entry IPC floods mpv (later commands queue past their timeout).
- **KWin force blur** = Better Blur DX (COPR `infinality/kwin-effects-better-blur-dx`).
  Config group is `[Effect-better-blur-dx]` — **dashes, not underscores** (plugin id
  is `better_blur_dx`; the mismatch cost hours). `WindowClasses` must be a **single
  line** (`songstress`); multi-entry newline lists silently never match. After config
  changes: unload+load the effect over DBus, not just `reconfigure`:
  `busctl --user call org.kde.KWin /Effects org.kde.kwin.Effects unloadEffect s better_blur_dx`
  (then `loadEffect`). Stock blur must stay disabled. **Unloading the effect kills
  blur SYSTEM-WIDE** (it's a global KWin effect) — always `loadEffect` it back
  immediately after diagnosis; the user will notice within minutes.
- **Window resourceClass is `songstress`** (binary name, NOT the app-id). Verify with
  `qdbus-qt6 org.kde.KWin /KWin org.kde.KWin.getWindowInfo <uuid>` (uuid via
  `/WindowsRunner` `Match`). KWin scripting console is useless on 6.7 (no output).
- **Screenshots lie when the game is fullscreen.** Raise our window first:
  WindowsRunner `Match` → `Run(matchId, "activate")`. Never minimize/kill the
  user's windows or change their wallpaper without asking.
- **This WebKitGTK ignores some modern CSS that computes cleanly.** Measured
  2026-09-05: unprefixed `user-select` is a no-op (every `none` in the app was
  dead code — `-webkit-user-select` is mandatory), and the INDIVIDUAL transform
  properties (`translate:`/`rotate:`/`scale:`) mis-resolve percentage
  centering on some svg elements — getComputedStyle reports the right value,
  layout is 4px off (the search-clear ×; traffic lights using the same rule
  measured fine, so it is per-element, no rule of thumb). Dialect that never
  lies on this engine: prefixed `user-select`, plain `transform`. Audit with
  `grep 'user-select' src/ ; grep -rnE '^\s*(translate|rotate|scale):' src/`.
- **WebKitGTK backdrop-filter does NOT blur in-window content** (verified
  2026-08-27, 2.52/DMABUF): a 32px-bold probe element under the playbar
  stayed pixel-sharp through `blur(80px)` — radius changes are invisible.
  All the visible frost is KWin's force blur on the wallpaper. Consequence:
  don't try to make scrolling content blur under the playbar via
  backdrop-filter; dim it with the `.grid-fade` veil in App.svelte instead.
  If you ever test backdrop-filter again, probe with content UNDER the glass,
  never with the wallpaper.
- **UI alphas** (user-tuned): grid backdrop 0.8 (`--bg-grid`), chrome 0.7
  (`--bg-chrome`), expanded panel gradient alpha 0.36 dark / 0.30 light. Don't
  drift from these without being asked.
- **Switch animation contract**: album switch = instant content swap + 160 ms
  fade-in of the NEW album; height changes in ONE step (no per-frame relayout).
  The outgoing album must NEVER render at the new row (user calls it "ghost").
  Expand/collapse keeps the eased `grid-template-rows` animation.
- **Cover decode is expensive** (some art is 3000px). Panel art uses
  `decoding="async"`; `AlbumGrid.toggleExpand` pre-decodes via `img.decode()`.

## Import staging (read before touching `library/import.rs`)

- **Staging is a row flag, not a folder.** `tracks.staged` (migration v2) is the entire definition of "pending": `import_music` indexes the
  user's files **where they are** — no copy, no cache dir, no second scan root — so a pending import gets artwork, waveforms, play counts and
  scan-safety the moment it lands, and importing a 400-file folder costs a scan instead of 400 writes. The scan flags exactly the files that
  were requested (`run_scan_files(.., only: Some(&set), ..)`), which is why importing one file out of a 400-file folder stages one.
- The flow: import flags → the user decides per album → `save_imports` **moves** the file into its resolved folder and re-points the row
  (`scan::relink_track`, keeping play history; id derives from path, so id and path change together or the next re-import hits UNIQUE),
  `discard_imports` **forgets the row and deletes nothing**. `album_destination` resolves the folder: the album's own folder → the artist's
  layout → the directory the majority of the library's albums live in → the primary root.
- **Three rules, all about ownership.** At Apply, a file whose resolved name the destination folder already holds **byte for byte** is a
  duplicate: **the file the user just pointed at is deleted**, the library's copy stays, and the receipt says so — a silent deletion of
  someone's file is not acceptable even when it is a copy of one they own. (Same name, different bytes ⇒ a ` (2)` name, never an overwrite.
  This is a collision rule, not library-wide content dedupe: the same track on a compilation and on an album is legitimate, so a hash column
  would delete files the user wants twice over.) A file the library already indexes is **not staged at all** and is reported under
  "Already in your library". And the app **never removes a folder it did not write**: it prunes only directories under its own staging root,
  and the folder you imported from is left alone even when it is now empty.
- **Import reports what it did.** `import_music` returns `ImportReport { staged, already }`; `imports.report` renders it in the window the
  import opens, and `imports.applied` renders Save/Discard counts after Apply — including the deletion above. A progress ring that ends with
  the screen unchanged reads as a failure, and "turns out you already own this" is the most likely outcome of importing music.
- Legacy: rows whose path sits under the cache `import/` dir are flagged at startup, so a copy-era pile survives the upgrade and its next Save
  is a rename out of the app's own directory. The copy importer (`import_paths`, `KnownFiles`, name+size dedupe) is gone; identity is **path**
  (import), **blake3 hash** (save), and `(artist, album, disc, track)` (scan), each covered by a test.

## Svelte 5 gotchas learned the hard way

- **A `${}` in a plain attribute is not an interpolation.** Svelte reads the `$` as
  text and only the braces as the tag, so `style:transform="scaleX(${pct/100})"`
  emits the declaration `scaleX($0.4)` — invalid CSS, dropped with no error, and
  `el.style` ends up empty so the stylesheet's `scaleX(0)` wins. Interpolated
  attribute values need a template literal: ``style:transform={`scaleX(${pct/100})`}``.
  The tell (this is how EmptyState's first-scan bar sat still for whole scans):
  the counter *text* next to it is right, `style` is `""`, console says nothing.
- **A `$derived` that early-returns on a plain `let` never wakes up.** Svelte 5
  tracks only what a derived body ACTUALLY reads before it returns. A blast-
  radius derived opened with `if (!snapshot) return null` where `snapshot` was
  a plain script `let` — first pass (before the load finished) early-returned,
  so `touched`/`census` were never registered as dependencies, and the derived
  stayed null through every later edit. Symptom: every input is right, the
  value is `null`, no error. The gate variable of a derived must be `$state`
  (here: `snapshotJson = $state("")`), full stop.
- **A class forwarded into a child component is unscoped.** `<SurfaceClose
  class="ab-close" />` puts the class on an element compiled in another file, so
  the parent's scoped rule `.ab-close { … }` is reported as *unused* and does
  nothing. Use `:global(.ab-close)` — and give it one more class of specificity
  (`:global(.ab .ab-close)`) when the child sets the same property, because the
  child's stylesheet loads later and an equal-specificity tie goes to it.
- **A parent's scoped CSS does not reach a child component's root element.** Two
  bites: About needed `:global(.ab .ab-close)` to position `<SurfaceClose>`, and a
  `.group > * + * { margin-top }` rule silently gave `<Toggle>` no separation
  (0px, no warning). Put box/spacing rules on the **container** (`gap` works
  whoever compiled the child) and use `:global()` only for per-element offsets.
- **Child effects run before parent effects.** Never capture `document.activeElement`
  in a parent `$effect` to remember "who opened this" — by then the child's
  autofocus has already moved focus inside. Record the trigger at open time
  (`openAbout(e.currentTarget)`), and read it back on unmount.

## UI work: impeccable + apple-design are the default tools

Any UI tweak goes through the **impeccable** skill (run its `context.mjs`
setup once per session — it loads PRODUCT.md + DESIGN.md as the brief; then
the one playbook for the command at hand: critique / polish / animate /
harden / new-work…) and **apple-design** for the motion + material rules
( respond on pointer-down; animate from the presentation value; enter and
exit along the same path; springs over fixed-duration tweens where a gesture
can interrupt; size-specific tracking; translucency conveys hierarchy).

Two overrides, because this repo is older and more constrained than the
skills' defaults:
- **DESIGN.md's tuned values win** (two-tier glass 0.8/0.7, panel gradient
  0.36/0.30, one accent, the motion tokens `--ease-out` / `--ease-drawer`,
  the No-Lift Rule). Where apple-design says `backdrop-filter` for in-window
  frost, we clip + cast light instead — see the WebKitGTK gotcha above.
- The immersive verification loop is the devtools bridge (`tools/devctl.mjs`)
  + spectacle, not a headless browser; prefer critique/audit snapshots under
  `.impeccable/critique/` over open-ended screenshot loops.

## Verification before saying done

1. `npm run check` → 0 errors
2. `npm test` → all green
3. `cargo test --lib` (in src-tauri) → all green
4. `npm run build` → succeeds
5. Visual pass if UI touched — **only when the owner asks for it, or when the change
   is not something he can check faster himself.** He drives the app and tests the UI
   (rule set 2026-09-01): do NOT walk the surfaces one screenshot at a time — activate,
   capture, crop, read. That is slower than him pressing the keys and it spends the
   session's budget. Do the mechanical gates (1-4), one cheap DOM probe if a measurement
   is the thing at issue (geometry, computed style — no screenshot), then hand it over
   with what to look at. When he DOES ask for a visual pass, the rules below are
   non-negotiable — they are why a screenshot can lie.
   RAISE IT FOR REAL: WebKit stops repainting the Songstress window while it is
   unfocused, so `spectacle` hands back a frame from BEFORE your last action and
   you will debug a UI bug that does not exist. Activate first
   (`qdbus-qt6 org.kde.KWin /WindowsRunner org.kde.krunner1.Run "$ID" activate`),
   then screenshot — and when a screenshot contradicts a DOM probe, trust the
   probe and re-shoot.
   ACTIVATE ONLY — never resize, move or maximize the window to make a layout
   "fit the screenshot". It lives in the user's 3-part grid (terminal + app in the
   left column, browser in the right) at **1200×660**, and a size change is both a
   broken desktop and the known trigger of the WebKitGTK band-clip glitch. Verify
   the UI at his size (see PLAN.md → Environment quick facts).
6. Update PLAN.md (status table + implementation log)

## Versioning at commit time

Every time a commit gets the green light: quantify the change since the last
bump (what shipped — fixes/polish vs. features vs. breaking; dev-only vs.
user-visible) and bump the version accordingly in ALL THREE places, together
(`package.json`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml` — they
drift if bumped independently):

- **patch** (0.x.y → 0.x.y+1): bug fixes, motion/polish, copy changes
- **minor** (0.x.y → 0.(x+1).0): a meaningful user-visible feature or a
  PLAN.md step that ships new capability
- **major**: breaking change (DB schema, config format, command interface)

Include the bump in the same commit as the work. If a batch of commits
already shipped, make a follow-up "Version 0.x.y" commit whose message
quantifies what the version contains. A commit batch that is dev-only
(absent from the prod build) does not need a bump — say so in the commit
message. Never leave the version files disagreeing with each other, and
never leave a bump uncommitted across work commits: the version must always
describe what HEAD contains.
