# Songstress

> Agent instructions: [AGENTS.md](AGENTS.md) · roadmap + implementation record: [PLAN.md](PLAN.md)

Album-grid music player for Fedora KDE (Wayland). Tauri 2 + Svelte 5 frontend, Rust backend,
MPV (JSON IPC) as audio engine, SQLite library, lofty tags.

## Development

```sh
npm install
npm run tauri dev      # full app
npm run check          # svelte-check
npm test               # vitest (row model, sorting)
```

First Rust build takes a few minutes. Requires the Tauri Linux prerequisites:
`webkit2gtk4.1-devel gtk3-devel librsvg2-devel patchelf` plus `nodejs npm` and rustup.

## Wallpaper blur (KWin)

Wayland clients cannot request compositor blur-behind, and stock KWin blur only
honors client-requested blur regions. The frosted-wallpaper look therefore needs the
community force-blur effect **Better Blur DX** (actively maintained successor of Better
Blur, supports Plasma 6.7):

```sh
sudo dnf copr enable infinality/kwin-effects-better-blur-dx
sudo dnf install --refresh kwin-effects-better-blur-dx
```

`~/.config/kwinrc` is already configured for it:

- `[Plugins] betterBlurDxEnabled=true`, `[Plugins] blurEnabled=false` (DX replaces stock blur)
- `[Effect-better-blur-dx]` (dashes!) — `WindowClasses=songstress` (single line only; multi-entry lists break matching in this build), `BlurMatching=true`, `CornerRadius=14`

After installing, run `qdbus-qt6 org.kde.KWin /KWin reconfigure` (no relogin needed on
first install). The effect must match the exact installed KWin version — if a Plasma
upgrade breaks it, reinstall the package.

**Auto-load on boot**: KWin occasionally boots without loading the effect despite
`betterBlurDxEnabled=true` (silent failure, seen twice: 2026-08-22 and 2026-08-23).
The `kwin-blur-load.service` user unit (`~/.config/systemd/user/`, script in
`~/.local/bin/kwin-blur-load.sh`) runs after `plasma-kwin_wayland.service`, checks
`isEffectLoaded` and force-loads over DBus with a 2-minute retry. Enabled and tested;
no manual step needed after reboots anymore.

Without the effect the app degrades gracefully: panels keep their in-app
`backdrop-filter` (frosting content scrolled beneath them) and the tuned surface
alphas keep everything legible over unblurred wallpaper.

## Notes

- The in-app `backdrop-filter` on sidebar/playbar/expanded panel works unconditionally;
  it only frosts page content, never the wallpaper.
- `decorations: false` — custom traffic lights + drag regions; resize edges come from
  GTK CSD.
- mpv is spawned with `--no-config`; the user's `~/.config/mpv/mpv.conf` is not read.
