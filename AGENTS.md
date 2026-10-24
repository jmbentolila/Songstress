# AGENTS.md — Songstress

Instructions for AI agents (and humans) working on this codebase. Read this file
fully before changing anything. The measurements behind its rules live in
`docs/gotchas/` and are read on demand from the index at the bottom. `PLAN.md`
carries the current state, the owner decisions and the next-up list; the
landed-work record sits in `docs/plan-archive/`, indexed by its README. Read
PLAN.md before starting and update it when you finish.

This file must stay under 18,000 chars. Check with `wc -c` before a large
addition; overflow goes to `docs/gotchas/<area>.md` and one line stays here.

## What this is

Songstress, a local music player for Fedora 44 (Plasma 6.7 and GNOME, Wayland).
Album-grid-centric, MusicBee-style browsing, glassmorphism UI. Personal use first.

Stack: Tauri 2 (Rust) + Svelte 5 + TypeScript, MPV via JSON IPC as the audio
engine, SQLite + lofty for the library. No router, no CSS framework, hand-rolled
CSS with custom properties.

One RPM serves both desktops (2026-09-23). DE differences are runtime fallbacks,
never build variants. Plasma is the reference desktop (kdialog pickers, Global
Menu, KWin force-blur). On GNOME the same portal-first picker takes over, the
menu bar moves into the titlebar (`wayland_appmenu` skips with STATE=2), the
chrome goes opaque (`html[data-de="gnome"]` pins the glass tokens solid) and
GRAIN replaces glass on every opaque field, with "The Matte Rule" in DESIGN.md.

RPM deps: **`mpv` only** (`rpm -qpR`, verified 2026-09-26 on 0.13.4-1: mpv +
libwebkit2gtk-4.1 + libgtk-3, nothing else). Pickers are XDG-portal-first
(`portal_files.rs`) with a `kdialog` fallback, so no picker package is required;
there is no `zenity` path. Waveform peaks decode with **symphonia** (pure Rust,
statically linked, adds no system package), so `depends = ["mpv"]` still holds:
do not reach for ffmpeg or gstreamer for decode.

This box is GNOME-only right now: the kdialog branch is verified by argv
identity, never by a live pick. Say so, do not claim it.

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

Closing the app window kills `tauri dev` by design; restart the unit.

**Restart protocol (owner-set, 2026-09-01).** A restart brings the window back
at the remembered SIZE (`window_state.rs`) at a KWin-picked position. It is not
where he keeps it: his tile is the bottom-left quarter of a three-part grid, and
nothing can put it back programmatically, because a Wayland client cannot see
its own position and KWin exposes no move/resize on Plasma 6.7. So after ANY
restart: **stop, tell him the window is misplaced, and wait for his green light
that he has moved it back** before driving or measuring anything in it. Do NOT
add a KWin window rule, seed `.window-state.json`, or call `set_position`; he
declined all three and the manual move IS the protocol. Never save position;
size is saved on every Resized event. Measurements and history:
`docs/gotchas/desktop.md`.

Frontend work needs no restart: `tauri dev` hot-reloads Svelte, stores and CSS.
Restart only for Rust changes or `contain:` changes, and remember the version
files are watched too: bumping `tauri.conf.json` or `Cargo.toml` restarts the app
and costs him a drag. Bump before he places the window, or warn him.

**Reviewing a loading state without a scan.** `node tools/devctl.mjs eval "__skel(true)"`
holds the grid and artist-list skeletons (`main.ts`, DEV-only, sets
`library.devLoading`); `__skel(false)` releases them. Do not re-enter the state
by clicking Rescan after a reload: a parked scan (`SONGSTRESS_SCAN_STALL_MS`)
still holds `SCAN_RUNNING`, and re-parking needs a unit restart, which costs a
window drag. Park once, then iterate with `__skel`.

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

## Rules that bite

Data and user files. **Probes never compute paths on owner-owned files.** Copy
the file to `/tmp` first and probe the copy; never rename onto, write in, or
`remove_file` a path under `~/Music` from test or debug code. A temp scheme
derived from the target path (a `with_extension("tmp")` sibling that collided
with the original) deleted a real mp3 on 2026-09-04. User data gets
character ops, char boundaries and temp+rename, always.
**lofty 0.22.4's `write_id3v1` PANICS** on a byte cut through a multi-byte char
(`end byte index 28 is not a char boundary`): blame the v1 WRITER, not the
reader. `tags::write_file` is armored with temp+rename under `catch_unwind`, a
char-boundary pre-trim and retry-once-without-v1. Upstream fix is lofty >=0.23.
**Rust panics on integer overflow**, and a panic inside a Tauri command aborts
the app: use `u16` and `saturating_*` for pixel math.

Engine. Spawn mpv with **`--no-config`** (the user's `mpv.conf` is a broken
Windows config). **NEVER await IPC commands in the mpv reader task**: it reads
responses off the socket, so awaiting stalls all reads until the 10s timeout.
Side effects there are fire-and-forget (`fire()`). The playlist is a **rolling
32-track window** (`PLAYLIST_WINDOW`), never the whole play order.

Library and watcher. `scan_inner` must emit `scan-finished` on EVERY path,
carrying `error: Some(..)`: it is the only thing that clears `library.scanning`,
so an early return without it shimmers the skeleton forever. Incremental scans
skip unchanged files entirely, so after changing grouping use Full Rescan or
nudge `mtime_ns`. The **watcher must ignore Access events**
(`marks_library_dirty`): counting them made it feed on itself, 7,723 rescans in
a day. `music_root` treats null or empty as unset (→ `~/Music`); a cancelled
kdialog pick once stored the literal `null` and every scan failed silently. **A
track's ALBUM is grouping output, not identity**: resolve the playing track by
`trackId` (`currentTrack()`, `reanchorCurrent()`), never through `albumId`.
`.songstress.json` is per-folder display state: a scan adopts it, an in-app
change mirrors the DB into it (`adopt` flag), and getting the direction
backwards silently reverts the user's edit; the watcher ignores the filename.
Tag suggestions read `tag_vocabulary`, so genre, composer, label and grouping
stay empty until a Full Rescan backfills them; ranking lives in
`lib/suggest.ts`.

Frontend. Svelte 5 runes stores live in `*.svelte.ts`. A `${}` in a plain
attribute is not an interpolation (use a template literal), and a `$derived`
that early-returns on a plain `let` never wakes up (its gate must be `$state`).
**HMR rots**: stale `bind:clientWidth`, dropped CSS and phantom layouts after a
long edit series, so cold-restart the unit before debugging phantoms; a cold
start and `app.css` can also serve empty CSS, and the tells and one-second fixes
are in `docs/gotchas/frontend.md`. `opacity: 0` still hit-tests, and
`overflow: hidden` is still a scroll port: use `pointer-events: none`,
`overflow: clip` and `focus({ preventScroll: true })`. Unprefixed `user-select` is a no-op, the
individual `translate:`/`rotate:`/`scale:` properties mis-resolve percentage
centering, and `transform` on a range input is ignored: use prefixed
`user-select`, plain `transform`, and center with
`top: 0; bottom: 0; margin: auto 0`. Animating an INHERITED registered custom property costs the whole document
(~27fps): change the token instantly and let each consumer transition its own
concrete property. `backdrop-filter` does NOT blur in-window content on this
engine (measured on 2.52, re-verify on 2.54): the visible frost is KWin blur, on
GNOME the glass is unblurred, and scrolling content is dimmed with `.grid-fade`.
**WebKitGTK 2.54** broke `thumb://` subresources (dev fix in `artSrc.ts` plus
`vite.thumb-dev.js`, never touch the Rust handler) and stutters on this box,
fixed by 2.52.1: the box is pinned by hand, so check `rpm -q webkit2gtk4.1`
before blaming the app. UI alphas are user-tuned: grid 0.8 (`--bg-grid`), chrome 0.7
(`--bg-chrome`), panel gradient 0.36 dark / 0.30 light. Do not drift without
being asked.
Confirmations inside a modal drop from that surface's top edge, as `.glass`
boxes with the safe action first, and the surface dims under them; Escape is
handled in CAPTURE with `stopImmediatePropagation`, never modal-on-modal and
never a native dialog. Album switch is an instant content swap plus a 160 ms
fade of the new album (expanded-panel same-row swap: 320 ms fade / 380 ms travel
with a two-beat stagger), and the outgoing album never renders at the new row.
Cover decode is expensive: `decoding="async"` and pre-decode in
`AlbumGrid.toggleExpand` with `img.decode()`.

Desktop. KWin force blur is **Better Blur DX built from source at `728307f`**,
not the removed COPR package, and a kwin upgrade breaks it until rebuilt. Config
group is `[Effect-better-blur-dx]` (dashes), `WindowClasses` must be a **single
line** (`songstress`), stock blur stays off, and a config change needs only
`reconfigureEffect` (unload kills blur system-wide; a rebuilt `.so` needs
logout). `kwriteconfig6 --key K --delete`, never `--delete K`. The window
resourceClass is `songstress` (binary name, not app-id). Screenshots lie when a
game is fullscreen: activate our window first, and never minimize or kill his
windows or change his wallpaper without asking. No GTK dialogs: pickers are
portal-first then kdialog, and a picker failure surfaces as an error naming the
missing helper.

## Import staging

**Staging is a row flag, not a folder**: `tracks.staged` is the whole definition
of pending, and `import_music` indexes the user's files where they are, so there
is no copy and no second scan root. `save_imports` MOVES the file and re-points
the row (`relink_track`, id derives from path, so id and path move together);
`discard_imports` forgets the row and deletes nothing. At Apply, a file whose
resolved name the destination already holds **byte for byte** is a duplicate:
**the file the user just pointed at is deleted**, the library copy stays, and
the receipt says so. Same name, different bytes gets a ` (2)` name, never an
overwrite. The app **never removes a folder it did not write**, and the folder a
file came from is left alone even when empty. `ImportReport { staged, already }`
is rendered in the window the import opens, because a ring that ends unchanged
reads as failure. Identity is **path** for import, **blake3** for save and
`(artist, album, disc, track)` for scan. The full contract, including legacy
cache rows and the removed copy importer, is `docs/gotchas/import.md`.

## UI work: impeccable + apple-design are the default tools

Any UI tweak goes through the **impeccable** skill (run its `context.mjs` setup
once per session, it loads PRODUCT.md and DESIGN.md as the brief, then the
playbook for the command at hand) and **apple-design** for motion and material.
Two overrides: **DESIGN.md's tuned values win** (two-tier glass 0.8/0.7, panel
gradient 0.36/0.30, one accent, the motion tokens, the No-Lift Rule), and where
apple-design says `backdrop-filter` for in-window frost, this repo clips and
casts light instead. Verify through the devtools bridge (`tools/devctl.mjs`)
plus the DOM probe, not a headless browser, and keep critique snapshots under
`.impeccable/critique/`.

## Verification before saying done

1. `npm run check` → 0 errors
2. `npm test` → all green
3. `cargo test --lib` (in src-tauri) → all green
4. `npm run build` → succeeds
5. Visual pass **only when the owner asks for it, or when the change is not
   something he can check faster himself**. He drives the app and tests the UI
   (2026-09-01): do not walk the surfaces one screenshot at a time. Run the
   mechanical gates, take one cheap DOM probe if a measurement is at issue, then
   hand it over with what to look at. When he DOES ask, activate the window
   first (WebKit stops repainting while unfocused and a screenshot then shows a
   frame from before your action), trust a DOM probe over a contradicting
   screenshot, and **never resize or move the window** to make a layout fit: it
   lives at 1200×660 in his grid and a size change triggers the WebKitGTK
   band-clip glitch. The current box is GNOME, where `spectacle`, `qdbus-qt6`
   and `gnome-screenshot` are absent and the `devctl` DOM probe is the tool that
   works; the Mutter focus rule is unverified.
6. Update PLAN.md: its Status row, and a dated `## YYYY-MM-DD — title` entry at
the top of `docs/plan-archive/work-log.md`

### Claim hygiene — from the Step 0b post-mortem (2026-09-19)

1. **Never read a version string as a date.** A build stamp names build and
   commit, not release date; reading one as a release date once cost four weeks
   of the wrong investigation. Check the commit (`git log -1 --format=%cs <sha>`)
   and prove containment with `git merge-base --is-ancestor` or `compare/a...b`;
   author dates lie too, containment is proof.
2. **A check that returns 0 for everything proves nothing.** Run positive
   controls in the same command (`strings -el` for UTF-16 keys, `nm | wc -l`
   before trusting an absence).
3. **Absence of a record is not absence of a fact.** Ask and record the
   provenance; "owner-observed, no prior record" is a real category.
4. **Record the lever you ruled out, not just the one you used** (`unloadEffect`
   was documented and `reconfigureEffect` existed all along).
5. **Flag the assumption you did not verify.** An unstated assumption becomes a
   fact in the next pass's hands.

The full post-mortem, with the four-week cost, is in
`docs/plan-archive/remaining-work.md` (Step 0b).

## Versioning at commit time

Every time a commit gets the green light: quantify the change since the last
bump and bump the version in ALL THREE places together (`package.json`,
`src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml` — they drift if bumped
independently):

- **patch** (0.x.y → 0.x.y+1): bug fixes, motion/polish, copy changes
- **minor** (0.x.y → 0.(x+1).0): a meaningful user-visible feature or a PLAN.md
  step that ships new capability
- **major**: breaking change (DB schema, config format, command interface)

Include the bump in the same commit as the work. If a batch already shipped,
make a follow-up "Version 0.x.y" commit whose message quantifies what it
contains. A dev-only batch (absent from the prod build) needs no bump; say so in
the message. Never leave the version files disagreeing, and never leave a bump
uncommitted across work commits: the version must describe what HEAD contains.

## Shared memory (this repo has no memory of its own)

Vault: `/home/yossi/Projects/Arkadia` (private git, Obsidian-readable). Sessions
start in `~/Projects` behind the router there, so vault work is a deliberate
move: when a fix here is hard-won, go to the vault and file it the same session
(`agents/incidents/`, commit and file links), then update `agents/open-loops.md`,
`wiki/index.md` and `wiki/log.md`. Never re-derive what the vault knows.

## Derived detail (read the area file before working in it)

| before touching | read |
|---|---|
| any Svelte component, CSS, motion, modals | `docs/gotchas/frontend.md` |
| CSS that computes but does not paint, WebKit behaviour | `docs/gotchas/webkitgtk.md` |
| KWin blur, window placement, screenshots, pickers | `docs/gotchas/desktop.md` |
| `library/scan.rs`, `watcher.rs`, tags, album state | `docs/gotchas/library.md` |
| `library/import.rs` | `docs/gotchas/import.md` |
| mpv, playback, engine IPC | `docs/gotchas/mpv.md` |
