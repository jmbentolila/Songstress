<div align="center">

<img src="assets/app-icon.svg" alt="Songstress" width="128" />

# Songstress

**A local-first music player for Linux desktops.**
Album-grid centric, MusicBee-style browsing, glassmorphism chrome —
built for Fedora (Plasma and GNOME on Wayland), shaped by one very deafeningly opinionated listener.

*Tauri 2 · Svelte 5 · Rust · MPV · SQLite · lofty*

</div>

---

## What it looks like from the inside

- **Album grid first** — artists in a sidebar, albums as covers, an expanded
  panel under the selected album with its full track list
- **Real audio engine** — MPV over JSON IPC: gapless playback, 4-band EQ,
  shuffle pools that stay fast at thousands of tracks
- **A tag editor that respects you** — album + track editors, art picker,
  disputed-tag awareness, and an import flow that *moves* your files where
  they belong and shows you the receipt
- **Desktop-native integration** — Global Menu over DBus on Plasma (with live
  transport glyphs), MPRIS, inotify library watching, blur via KWin, dialogs via
  kdialog — with zenity + in-titlebar fallbacks on GNOME
- **Glass, not frameworks** — hand-rolled CSS with theme tokens; two-tier
  translucency tuned to the compositor, zero CSS dependencies

## Development

```sh
npm install
npm run tauri dev      # the full app (needs mpv + kdialog/zenity)
npm run check          # svelte-check
npm test               # vitest
cd src-tauri && cargo test --lib
```

Rust ≥ 1.8x (lofty), Node ≥ 20, WebKitGTK (Tauri 2 defaults), `mpv` on PATH.

## The docs triangle

| Doc | What it is |
|---|---|
| **[AGENTS.md](AGENTS.md)** | The working contract: gotchas learned the hard way, verification gates, versioning rules |
| **[PLAN.md](PLAN.md)** | The entire roadmap *and* implementation log — every decision, every dead end, dated |
| **[DESIGN.md](DESIGN.md) / [PRODUCT.md](PRODUCT.md)** | The design brief: tuned alphas, motion tokens, the No-Lift Rule |

## Built with a machine

**This app was vibecoded** — designed, written, reviewed, and relentlessly
refactored by a human and an AI agent working the same keyboard. The taste is
human's; a lot of the typing is the machine's; the arguments about both are
recorded, dated, in [PLAN.md](PLAN.md). If you're an agent reading this:
read AGENTS.md first, it is scarred so you don't have to be.
