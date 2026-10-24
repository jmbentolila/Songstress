# mpv gotchas

The audio engine's three ways to hang or flood itself.

Moved out of `AGENTS.md` on 2026-10-24 to keep the injected file under
the 20,000-char context cap. Rules stay in `AGENTS.md`; this file carries the
measurements and the reproduction.

- **mpv must be spawned with `--no-config`** — the user's `~/.config/mpv/mpv.conf`
  is a broken Windows config (d3d11, C:\ fonts).
- **NEVER await IPC commands in the mpv reader task** (mpv.rs `spawn_reader`).
  It is the task that reads responses off the socket — awaiting a command
  there stalls all reads until the 10s timeout fires (mpv executed instantly;
  only we were blind). Reader-task side effects are fire-and-forget (`fire()`).
- **mpv playlist is a rolling 32-track window** (`PLAYLIST_WINDOW`), never the
  whole play order — all-artists shuffle pools are thousands of entries and
  per-entry IPC floods mpv (later commands queue past their timeout).
