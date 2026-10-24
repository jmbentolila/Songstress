# Desktop and window gotchas (KWin blur, placement, screenshots)

Plasma blur build and config, the window restart and placement measurements, picker choices, and the screenshot traps.

Moved out of `AGENTS.md` on 2026-10-24 to keep the injected file under
the 20,000-char context cap. Rules stay in `AGENTS.md`; this file carries the
measurements and the reproduction.

- **No GTK dialogs. Pickers are portal-first, then `kdialog`.** `pick_directory`/
  `pick_image` call the XDG desktop portal (`portal_files.rs`), so the picker is
  compositor-native on BOTH desktops; if the portal is unavailable they fall back
  to `kdialog` (spawn-NotFound only), cancel stays Ok(None) on either. Don't
  reintroduce tauri-plugin-dialog / rfd — its GTK chooser looks GNOME-ish (user
  rejected it). There is **no `zenity` path** (verified 2026-09-26:
  `grep -rn zenity src-tauri/src` → nothing): the 0.12.1 "zenity picker stack"
  was a detour built on a wrong assumption and reverted by the 0.12.0 merge
  `a0e03ea` — PLAN.md keeps it as history, not as design. Packaging consequence:
  the RPM carries no picker dependency (`depends = ["mpv"]`), so a picker
  failure surfaces as an error naming the missing helper rather than being
  unreachable by construction.
- **KWin force blur** (Plasma only — no Mutter equivalent; on GNOME the glass
  simply loses its wallpaper blur) = Better Blur DX, **now a source build of `728307f`, not the COPR
  package** (`infinality/kwin-effects-better-blur-dx` was deliberately removed; `dnf update`
  will not restore it, and a kwin/Plasma upgrade breaks blur until rebuilt against the exact
  running kwin). Config group is `[Effect-better-blur-dx]` — **dashes, not underscores**
  (plugin id is `better_blur_dx`; the mismatch cost hours). `WindowClasses` must be a
  **single line** (`songstress`); multi-entry newline lists silently never match — a KCM
  write preserved it on 6.7.5, verified 2026-09-19. Stock blur must stay disabled
  (`blurEnabled=false`).
  **Config changes need only `reconfigureEffect` — NOT unload+load** (the old advice here
  caused avoidable blackouts): `qdbus-qt6 org.kde.KWin /Effects
  org.kde.kwin.Effects.reconfigureEffect better_blur_dx`. Proven to re-read kwinrc. Helper:
  `~/src/bbdx-ab.sh {show|A|B|C|D|set|unset}`.
  **Unloading still kills blur SYSTEM-WIDE**, and it cannot swap a rebuilt `.so` anyway —
  that needs **logout+login** (KWin holds the mapping; Wayland has no `--replace`).
  `kwriteconfig6` wants `--key K --delete`, never `--delete K` (the latter exits 0 having
  done nothing). Blur is **intentionally absent** in the Overview and during
  minimize/maximize — upstream design, not a bug to chase (see PLAN.md Step 0b S3).
- **Window resourceClass is `songstress`** (binary name, NOT the app-id). Verify with
  `qdbus-qt6 org.kde.KWin /KWin org.kde.KWin.getWindowInfo <uuid>` (uuid via
  `/WindowsRunner` `Match` — which implements **`org.kde.krunner1`**, NOT
  `org.kde.KWin.WindowsRunner`; and `Match` needs `--literal` to print its struct array).
  KWin scripting console is useless on 6.7 (no output).
- **Screenshots lie when the game is fullscreen.** Raise our window first:
  WindowsRunner `Match` → `Run(matchId, "activate")`. Never minimize/kill the
  user's windows or change their wallpaper without asking.

## Restart and placement, full measurements

**Restart protocol (owner-set, 2026-09-01; size-restored 2026-09-05).** A restart brings
the window back AT THE REMEMBERED SIZE (see below) at a KWin-picked position — measured
2026-09-05: center-ish, biased down-right (~150 logical px off true center; the spawn is
anchored as if still 1280×800 because KWin places from the client's early commit and the
xdg protocol cannot reposition after map — the builder's `.center()` is a hint, not a
placement). Not where he keeps it — his tile is the **bottom-left quarter**
(last measured: `x=0 y=690`, 1200×660 logical, in a 3-part grid: terminal + app in
the left column, browser in the right). Nothing can put it back programmatically:
a Wayland client cannot see its own position (measured — Tauri's `outer_position()`
returns `(0,0)` while KWin reports `x=560 y=290`), and KWin exposes no DBus
move/resize on Plasma 6.7. So after ANY restart: **stop, tell him the window is
misplaced, and wait for his green light that he has moved it back** before driving
or measuring anything in it. Do NOT add a KWin window rule, seed
`.window-state.json`, or call `set_position` to work around this — he declined all
three; the manual move IS the protocol.

Nothing persists the POSITION, on purpose (2026-09-01): `tauri-plugin-window-state` was added,
measured and **reverted**. A Wayland client cannot see where the compositor put it — Tauri's
`outer_position()` returns `(0,0)` while KWin reports the real `x/y` — so a client-side
"remember position" is not just unavailable, saving `(0,0)` and restoring it would shove the
window into the corner. The plugin also writes only on a clean exit (a `systemctl restart` /
`tauri dev` rebuild kills the process, nothing is saved). No KWin rule, no seeded state file
(both offered, both declined; they still are for position). SIZE, however, the owner asked for
(2026-09-05) — that ships as `window_state.rs`: the last size is saved on every `Resized` event
(tempfile+rename, survives the kill) and restored at startup. Do not save position, and do not
swap the module back to the plugin.

## GNOME-on-this-box notes

   GNOME (current box; the 2026-09-23 note is STALE — re-measured 2026-09-26:
   `songstress-dev` IS loaded and running, and `tools/devctl.mjs eval`+`tail`
   bridge into the page fine, so DOM/texture probes work on GNOME too. Of the
   screenshot tooling, `spectacle`, `qdbus-qt6` AND `gnome-screenshot` are all
   absent (`gdbus` is present), so there is no established screenshot path here —
   the DOM probe is the tool that works; if a shot is ever needed, the Mutter
   focus/stale-frame rule is UNVERIFIED (it was measured on KWin) and the portal
   route would have to be probed first. The restart protocol above is
   KWin-specific; Mutter centers new windows — re-measure placement/size behavior
   before writing a GNOME protocol here.
