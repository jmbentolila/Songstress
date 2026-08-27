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

## Layout

```
src/
  App.svelte              shell: titlebar / stage (grid+sidebar+playbar overlay) 
  app.css                 theme tokens, .glass, scrollbar, noise garnish
  components/             TitleBar, Sidebar, AlbumGrid, ExpandedPanel, PlayBar, TagEditor, MusicFolders, ContextMenu, EmptyState
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
- **Component CSS can be emitted unscoped.** TitleBar once emitted a global sheet,
  leaking `.title { text-align: center }` into every component. All TitleBar classes
  are now `tb-*` namespaced. If styles leak again, namespace the classes — don't
  fight the compiler. Audit served sheets:
  `curl 'http://localhost:1420/src/components/X.svelte?svelte&type=style&lang.css'`
  and check every selector carries a `.s-…` hash.
- **Debug Rust panics on integer overflow** and a panic inside a Tauri command
  aborts the whole app. Use `u16`/`saturating_*` for pixel math (see the `+8`
  bucket-center incident in git-less history / PLAN.md).
- **Covers live in `public/covers/`** (plain Vite — `static/` is a SvelteKitism).
  `tauri.conf.json` `frontendDist` is `../dist`.
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
- **Scan errors are near-invisible.** `scan_library` failures surface nowhere in
  the journal — check `[scan]` eprintln lines for success, and remember
  incremental scans SKIP unchanged files entirely (they never re-enter
  grouping). After changing grouping/tag-consensus logic, use Library →
  Full Rescan (rebuild) or nudge `tracks.mtime_ns` in the DB.
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
- **UI alphas** (user-tuned): grid backdrop 0.8 (`--bg-grid`), chrome 0.7
  (`--bg-chrome`), expanded panel gradient alpha 0.36 dark / 0.30 light. Don't
  drift from these without being asked.
- **Switch animation contract**: album switch = instant content swap + 160 ms
  fade-in of the NEW album; height changes in ONE step (no per-frame relayout).
  The outgoing album must NEVER render at the new row (user calls it "ghost").
  Expand/collapse keeps the eased `grid-template-rows` animation.
- **Cover decode is expensive** (some art is 3000px). Panel art uses
  `decoding="async"`; `AlbumGrid.toggleExpand` pre-decodes via `img.decode()`.

## Verification before saying done

1. `npm run check` → 0 errors
2. `npm test` → all green
3. `cargo test --lib` (in src-tauri) → all green
4. `npm run build` → succeeds
5. Visual pass if UI touched (raise window, screenshot, actually look)
6. Update PLAN.md (status table + implementation log)
