# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Users

Primary: the owner (single user, personal use first). The app is built for
his own Fedora 44 / KDE Plasma 6.7 (Wayland) desktop and his local music
collection. Audience may expand: Songstress is intended to be publishable as
a public KDE music-player alternative (RPM packaging is maintained), so
defaults must stay opinionated for one user while edge cases are handled as
if strangers would hit them.

## Product Purpose

A local music player that browses the collection as an **album grid**
(MusicBee-style) rather than a track list, and plays it through MPV
(gapless, ReplayGain, equalizer). Success = the owner's daily listening and
library management happens in Songstress, and a KDE user installing the RPM
finds a player that feels like it belongs on Plasma.

## Positioning

Album-grid-first local playback for KDE: the grid *is* the interface —
expanding a tile reveals the tracklist in place, chrome recedes into
frosted glass, and the system's own decoration language (Klassy palette,
button order, Global Menu, KWin force blur) is mirrored by the app.
Neighbors (track-list players, generic WebKit media apps) cannot truthfully
copy the "designed into Plasma" mechanism without doing the per-system
mirroring work.

## Operating Context

- Fedora 44, KDE Plasma 6.7, Wayland; output 3840×2160 @ scale 1.6;
  WebKitGTK 4.1 (2.52) webview inside a Tauri 2 shell.
- Music lives in one or more local roots (default ~/Music); inotify-watched
  for live changes; kdialog for all pickers; MPV child process over JSON IPC
  spawned with `--no-config`; SQLite + lofty for metadata.
- Dev loop runs as a systemd user unit (`songstress-dev`); production
  installs via RPM (`dnf`/`rpm -Uvh`).
- KWin "Better Blur DX" force blur is the wallpaper material the glass
  floats over — a system dependency of the visual identity, not of function.

## Capabilities and Constraints

Capabilities: album grid with in-place expansion, artist sidebar, unified
library search, tag editing (file + album), two-step import staging with
save/discard, multi-root library, live file watching, MPV playback with
shuffle/repeat/album nav, play-next queue, 10-band EQ + presets, MPRIS
(Media Player 2), Plasma Global Menu, glassmorphism chrome with traffic
lights in the sidebar header (no titlebar), user-tunable tile/row sizes,
accent color, dark/light/system themes.

Constraints (confirmed, durable): no GTK dialogs (kdialog only); mpv must
run `--no-config`; WebKitGTK `backdrop-filter` does not blur in-window
content (probe-verified) — in-window frost is content-side (clip, fade
shadows, `filter: blur` on rows); user-tuned UI alphas (grid 0.8, chrome
0.7, expanded-panel gradient 0.36 dark / 0.30 light) are identity, not
defaults; Svelte 5 runes stores; hand-rolled CSS, no framework; no router.
Open: public release scope is a possibility, not a commitment yet.

## Brand Commitments

- Name: **Songstress** (binary, window class, app-id `com.yossi.songstress`).
- Glassmorphism is the owner's primary style and is binding.
- No titlebar; traffic lights in the sidebar are a must.
- KDE is the target framework/platform family: mirror the user's system
  (KWin decoration palette + button order, KWin force blur, Plasma Global
  Menu, KDE-native dialogs) rather than importing foreign UI conventions.

## Evidence on Hand

- Real owner library (~247 albums) scanned in `~/Music`; fixture library in
  `src-tauri/fixtures/`.
- Curated album artwork in `public/covers/` (real folder.jpg files).
- `PLAN.md` — full implementation record, decisions log, and hard-won gotchas
  (authoritative for history); `AGENTS.md` — conventions and pitfalls.
- Working RPM packaging (Step 7b); screenshot-based verification workflow
  (`spectacle` + KWin WindowsRunner; no click-injection tooling installed).

## Product Principles

1. **The collection is the hero.** Chrome recedes; artwork and tracklists
   lead. Every surface decision asks "does this push content or chrome
   forward?"
2. **A native KDE citizen, not a guest.** Mirror the user's system —
   decoration, blur, dialogs, global menu. The app should be indistinguishable
   from Plasma's own design language at a glance.
3. **Personal-first, public-capable.** Optimize for one owner's daily use;
   keep opinionated defaults, but handle stranger edge cases as real.
4. **Motion is meaning.** Slides, fades, and reveals explain spatial
   structure (menu stack, panel expansion, search) — never decoration for
   its own sake; reduced-motion users get instant states.
5. **Invisible robustness.** The hard-won pitfalls (HMR rot, WebKit quirks,
   mpv IPC discipline, inotify self-feeding) are load-bearing; verify with
   the full gate list before claiming done.

## Accessibility & Inclusion

`prefers-reduced-motion` is honored globally (all transitions collapse to
instant); explicit `:focus-visible` accent rings on interactive chrome;
minimum ~28px hit targets on menu controls. No product-specific
accessibility commitments beyond these have been made.
