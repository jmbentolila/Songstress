# Open threads / known issues (plan archive)

Moved out of `PLAN.md` on 2026-10-24 so the plan file stays readable in one pass.
This is the record, not the state: current state lives in `PLAN.md`, and this
file is the evidence behind it. Sections are kept verbatim, in their original
order. The index of every section is `README.md` next to this file.

## Open threads / known issues

- **Stale dev-unit drop-in — REMOVED (2026-08-31)**: deleted
  `~/.config/systemd/user/songstress-dev.service.d/inspector.conf`
  (`Environment=WEBKIT_INSPECTOR_SERVER=127.0.0.1:9301`) + `daemon-reload` +
  unit restart. The env var made WebKitGTK listen on loopback:9301, but the
  remote-inspector handshake was never identified (see *Environment quick
  facts* — `tools/inspect.mjs` kept for the record), so nothing consumed it.
  Verified after restart: `DropInPaths` = distro `10-timeout-abort.conf`
  only, `Environment=` empty, port 9301 no longer listening, app boots clean
  (Global Menu registered, watcher armed, 248 tiles rendered). Recreate with
  `systemctl --user edit songstress-dev` if the inspector protocol is ever
  pinned down. The 2026-08-29 note below ("drop-in REMOVED") had covered the
  DMABUF/compositing experiments, not this one; the agent visual loop goes
  through the devtools bridge (`src/lib/devtools.ts` + `tools/devctl.mjs`),
  which needs no drop-in at all.

- **WebKitGTK viewport glitch, VARIANT 2 (2026-08-29)**: webview surface
  clipped to the top ~40px band, rest of the window black; layout was FULL
  size (frontend viewport matched the window — the 60s guard correctly saw
  no mismatch and could never fire). Triggered after the user dragged the
  window into a side-by-side split (journal: size churn 2740×1780 →
  2560×1600 → 2400×2132 right after launch). EXHAUSTED without effect:
  KWin reconfigure, minimize/restore, maximize/unmaximize, FOUR real 1px
  resize nudges (incl. during the churn), blur-effect unload/reload,
  full page reload (Vite entry touch). KWin scripting is dead on 6.7
  (loadScript/loadDeclarativeScript return -1 for any path or name —
  installed-script dir too; the /WindowsRunner Match/Run interface is
  `org.kde.krunner1`, NOT `org.kde.KWin.WindowsRunner`).

  Environment experiments (all via a songstress-dev.service.d drop-in,
  since 2026-08-29; drop-in REMOVED same evening — dev unit runs stock
  env):
  - `WEBKIT_DISABLE_DMABUF_RENDERER=1`: clip gone, UI perfect — but the
    user confirmed the glass layers break (search bar / mode buttons /
    panels read as inconsistent backgrounds; the wallpaper frost is
    gone). Rejected for daily use.
  - `WEBKIT_DISABLE_COMPOSITING_MODE=1`: transparency also degraded
    (same family of "sections with different backgrounds"). Rejected.
  - KILLING `WebKitWebProcess` as a runtime recovery: FATAL —
    WebKitGTK 2.52 does not respawn it in the Tauri setup; the window
    goes solid black and only a unit restart recovers. NEVER do this.
  - No upstream fix available: already on webkit2gtk4.1 2.52.5-1.fc44
    (latest Fedora build).

  Intermittency (stock env): launches at 15:17 and 15:28 were
  band-clipped (persisted 8+ min, unresponsive to everything above);
  15:33 launched clean; a follow-up restart showed a TRANSIENT
  "blurred wallpaper only" state (no content yet) that self-recovered
  within seconds — pre-first-paint, not the clip. Practical recovery:
  give a fresh launch ~10s; if the band persists, `systemctl --user
  restart songstress-dev` and retry — the race does not always hit.
  Trigger correlation: the window had just been dragged into a
  side-by-side split (size churn in journal). Open: a durable fix
  needs the WebKitGTK/Fedora side (DMABUF buffer pin on resize).
- **WebKitGTK viewport glitch** (FIXED 2026-08-22, kept for reference):
  on some launches the webview rendered at a stale smaller size inside the
  correctly-sized GTK window. `WEBKIT_DISABLE_DMABUF_RENDERER=1` fixed the
  glitch but killed backdrop transparency — rejected. Fix: Rust setup
  hook nudges the window 1px down+back at ~0.6s/~2.5s after launch,
  forcing a reconfigure, PLUS a self-healing guard
  (`src/lib/viewportGuard.ts` + `fix_viewport` command): for the first 60s
  after load the frontend compares its viewport against the real window
  size (8px tolerance) and re-nudges on mismatch. Verified across
  consecutive relaunches incl. one that reglitched post-nudge.
- **Expansion animation** (rewritten 2026-08-22): WebKitGTK animates the
  grid track and the content clip on DIFFERENT clocks — first expand
  snapped, collapse left an empty shell band. Now `.inner` animates
  explicit px height (rest states "0px"/"auto", settle timer 400ms) so
  clip edge, panel edge and shadow move in lockstep; verified frame-by-
  frame in slow motion.
- **Switch lag** (addressed 2026-08-22): album_colors was a SYNC tauri
  command (full-res JPEG decode on the main thread per switch) — now
  async + spawn_blocking + path-keyed COLOR_CACHE. Remaining suspects if
  lag persists: DX blur re-frost per frame, webview raster cost.
- **Expanded panel shadow** on `.expander`, not `.panel` (2026-08-22): the
  inner wrapper must stay overflow:hidden for the height animation and
  clipped the panel's shadow into sharp corners; expander is never
  clipped and tracks the animated box.
- **Grid scrollbar hidden permanently** (2026-08-22, user request):
  `scrollbar-width: none` + `::-webkit-scrollbar { display: none }` on the
  `.content` scroller; wheel/trackpad scrolling unaffected.
- **User's ~/.config/mpv/mpv.conf is broken** (d3d11 / C:\ fonts) —
  unrelated to the app (mpv is spawned `--no-config`) but worth telling
  them once more.
- **KWin DX multi-line WindowClasses** silently fails to match — keep one
  entry (see README/AGENTS).

