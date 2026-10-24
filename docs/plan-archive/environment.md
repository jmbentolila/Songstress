# Environment quick facts (for fresh sessions) (plan archive)

Moved out of `PLAN.md` on 2026-10-24 so the plan file stays readable in one pass.
This is the record, not the state: current state lives in `PLAN.md`, and this
file is the evidence behind it. Sections are kept verbatim, in their original
order. The index of every section is `README.md` next to this file.

## Environment quick facts (for fresh sessions)

- Fedora 44, **GNOME 50.5 (Mutter) Wayland** — corrected 2026-09-26: this line
  said Plasma 6.7.4, which is what the box ran when it was written. The KDE
  notes all over this file predate that move; AGENTS.md carries the current DE
  rules (portal-first pickers, in-titlebar menu, matte fields, GNOME screenshot
  tooling absent). Output HDMI-2 (LG HDR 4K) 3840×2160 @ scale 1.5 → logical
  2560×1440; the webview reports **dpr 2**, because GTK4 rounds to an integer
  scale under GNOME's fractional scaling (a DOM probe therefore reads
  2560×1440 CSS px). GPU: AMD Radeon 780M / Phoenix1 (radeonsi, Mesa 26.2.3);
  WebKitGTK composites via EGL/gbm.
- mpv 0.41 at /usr/bin/mpv. sudo requires password (user runs dnf lines).
- **WebKitGTK is pinned to 2.52.1 by hand** (`webkit2gtk4.1`, `-devel`,
  `webkitgtk6.0`): 2.54.0's Skia compositor stutters 3–47 FPS on
  hover/populate on this box, 2.52.1 does not (owner-verified A/B, 2026-09-26).
  The pin is NOT enforced — `dnf upgrade` silently restores 2.54 and the
  stutter with it, so check `rpm -q webkit2gtk4.1` first whenever the UI goes
  janky again.
- Dev unit: `systemctl --user restart songstress-dev`; logs `journalctl
  --user -u songstress-dev -f`. HMR rot → cold restart first.
- Raise window: WindowsRunner DBus Match + Run(matchId, "activate")
  (qdbus-qt6, `--literal` for Match output; matchId like `0_{uuid}`).
- **The user's desktop is a 3-part grid: terminal + the app in the LEFT column, a
  browser in the right.** The dev window is hand-placed, not maximized — measured
  2026-09-01: `x=0 y=690`, **1200×660**, dpr 2 (min is 960×600, default 1280×800,
  so it sits *below* the default on purpose). Therefore: **activate only — never
  resize, move, maximize, or "widen it to see the whole layout"** (a resize is also
  the known trigger of the band-clip glitch above, and it breaks his grid). Verify
  the UI **at 1200×660**: 236px sidebar ⇒ ~940px of grid ⇒ **4 columns at the 220px
  tile**, and 660 − 84 (playbar) = **576px of scroll port**; modals (ManageImports,
  Music Folders, About, Tag editor) must read at that height. `spectacle -b -n -a`
  (active window) is the right crop — it keeps the terminal and browser out of the
  frame; prefer a `devctl eval` DOM probe over a screenshot whenever state, not
  pixels, is the question (a screenshot steals focus, an eval does not).
- playerctl NOT installed — verify MPRIS via busctl/gdbus/qdbus-qt6.
- grim NOT installed; screenshots via `spectacle -b -n -a -o <file>`
  (active window) / `-f` fullscreen; background captures for artifacts.
- UI verification loop: the DEV-ONLY devtools bridge (commit e3f127b) —
  `node tools/devctl.mjs click|eval|sample|tail` drives the running webview
  (tile clicks, page-scope eval, rAF frame traces) and reads its
  console/errors via logs/devtools.log. No desktop input needed.
  (The WebKit remote-inspector drop-in opened a port but the handshake
  protocol was never identified — drop-in removed 2026-08-31, the
  tools/inspect.mjs probe kept for the record.
  Svelte scoped CSS silently drops cross-component :has() — use explicit
  state for cross-component CSS hooks.)

