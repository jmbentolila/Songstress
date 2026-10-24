# WebKitGTK gotchas

What this engine does and does not do, measured: CSS dialect it ignores, `backdrop-filter`, and the 2.54 breakage.

Moved out of `AGENTS.md` on 2026-10-24 to keep the injected file under
the 20,000-char context cap. Rules stay in `AGENTS.md`; this file carries the
measurements and the reproduction.

- **This WebKitGTK ignores some modern CSS that computes cleanly.** Measured
  2026-09-05: unprefixed `user-select` is a no-op (every `none` in the app was
  dead code — `-webkit-user-select` is mandatory), and the INDIVIDUAL transform
  properties (`translate:`/`rotate:`/`scale:`) mis-resolve percentage
  centering on some svg elements — getComputedStyle reports the right value,
  layout is 4px off (the search-clear ×; traffic lights using the same rule
  measured fine, so it is per-element, no rule of thumb). Dialect that never
  lies on this engine: prefixed `user-select`, plain `transform`. Audit with
  `grep 'user-select' src/ ; grep -rnE '^\s*(translate|rotate|scale):' src/`.
  **`transform` on a RANGE INPUT is a third bite (2026-09-28):** the straight-line
  seek had `top: 50%` + `translateY(-50%)`; getComputedStyle reported the matrix
  while the box ignored it, so the line sat 2px low. Center such a box with
  `top: 0; bottom: 0; margin: auto 0`, never a transform.
- **WebKitGTK backdrop-filter does NOT blur in-window content** (verified
  2026-08-27, 2.52/DMABUF): a 32px-bold probe element under the playbar
  stayed pixel-sharp through `blur(80px)` — radius changes are invisible.
  All the visible frost is KWin's force blur on the wallpaper (on GNOME there is
  no compositor blur at all — the glass is unblurred translucency). Consequence:
  don't try to make scrolling content blur under the playbar via
  backdrop-filter; dim it with the `.grid-fade` veil in App.svelte instead.
  If you ever test backdrop-filter again, probe with content UNDER the glass,
  never with the wallpaper. **Re-verify on 2.54**: the Skia compositor notes
  claim filters no longer need offscreen surfaces, so in-window blur may now
  be real (this claim was measured on 2.52 only).
- **WebKitGTK 2.54 (installed 2026-09-25) broke two things, unrelated to each
  other.** (1) `thumb://` subresources from `http://` pages: the Rust handler
  is never called (probe-verified; `tauri://` origins are fine, so the RPM
  never lost artwork). Dev fix: `src/lib/artSrc.ts` + the `vite.thumb-dev.js`
  middleware (`/thumb-http/…` served from the cache dir) — never "fix" this
  by touching the Rust protocol handler. (2) The new Skia compositor stutters
  on this box (radeonsi + GNOME/Mutter): FPS 3–47 on hover/populate in BOTH
  dev and the RPM, main thread clean (rAF probe). Every env lever
  (`WEBKIT_DISABLE_DMABUF_RENDERER`, `…_COMPOSITING_MODE`,
  `WEBKIT_USE_SKIA_FOR_COMPOSITION=0`) MISPAINTS worse — black window except
  the damage rect around the pointer. Do not chase these: (2) is
  **confirmed 2.54 and fixed by 2.52.1** — the box is pinned by hand, so a
  plain `dnf upgrade` brings the stutter back; check `rpm -q webkit2gtk4.1`
  before blaming the app, and revert/upgrade deliberately with
  `sudo dnf upgrade webkit2gtk4.1 webkit2gtk4.1-devel webkitgtk6.0`.
  Instrument trap when re-measuring: 2.52 names the FPS var
  `WEBKIT_SHOW_FPS`, 2.54 `WEBKIT_DRAW_FPS` — the wrong name prints nothing
  (a blind counter reads as "no drops"). See PLAN.md 2026-09-26.
  A long-running instance is a fossil: `rpm -q --last webkit2gtk4.1` dated
  the outage; "the RPM was fine yesterday" meant a process started Sep 24.
