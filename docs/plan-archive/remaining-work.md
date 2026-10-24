# Remaining work (plan archive)

Moved out of `PLAN.md` on 2026-10-24 so the plan file stays readable in one pass.
This is the record, not the state: current state lives in `PLAN.md`, and this
file is the evidence behind it. Sections are kept verbatim, in their original
order. The index of every section is `README.md` next to this file.

## Remaining work

### Step 7b — Packaging (RPM) — ✅ (landed 2026-08-26)

DECIDED (2026-08-25, go-ahead 2026-08-26): `tauri build` → native RPM
(dnf install/remove). AppImage rejected (not native to Fedora/KDE, no
package-manager integration); Flatpak deferred (decisions log). Release
profile: lto="thin", codegen-units=1, strip=true. Bundler generates the
system desktop entry + icons; remove the user-level
~/.local/share/applications/com.yossi.songstress.desktop after install.
Binary name MUST stay `songstress` (resourceClass + dev unit). Data stays
at ~/.local/share/com.yossi.songstress (settings/library survive the
dev→packaged switch). Version 0.1.0. NOT in scope: auto-update, Flathub,
external repo. Add `Requires: mpv kdialog` to the spec (mpv is spawned, not
linked — the bundler can't see it); better-blur-dx stays optional (COPR,
graceful degradation). sudo needs a password → the USER runs dnf install
lines; the agent builds, hands the command over, and does post-install
verification. Verify: install → krunner launch → Global Menu + MPRIS +
blur → dnf remove clean.

**Landed so far (2026-08-26)**:
- `Cargo.toml [profile.release]`: lto="thin", codegen-units=1, strip=true.
- `tauri.conf.json`: productName "Songstress" (display name only — the
  binary name comes from the Cargo package name, so `songstress` /
  resourceClass is untouched), category **"Music"** (maps to freedesktop
  `AudioVideo;Audio;Music;`), license MIT, short/long description,
  `bundle.linux.rpm = { release: "1", depends: ["mpv", "kdialog"] }`.
  (2026-09-23, GNOME move — single-RPM decision, owner-approved over two DE
  packages): `depends` was briefly `["mpv", "(zenity or kdialog)"]` and
  `release` reset to `"1"` under 0.12.1. **Correction (2026-09-26) — that rich
  dep is not in the tree and never shipped**: the zenity picker stack was
  reverted by the 0.12.0 merge (`a0e03ea`, portal-first), `tauri.conf.json`
  carries `depends: ["mpv"]`, and `rpm -qpR` on the 0.12.1, 0.13.0 and 0.13.4
  RPMs shows only `mpv` + webkit2gtk + gtk3 — so the "rich-dep passthrough by
  tauri-bundler" question left open below was never exercised (moot, not
  answered). Descriptions neutralized ("for Fedora / Plasma and GNOME"). If a
  picker helper is ever required again, settle it the way it was finally
  settled here: build, then `rpm -qpR`.
- **Category gotcha**: the bundler's AppCategory is the macOS/GNOME-style
  name list ("Music", "Video", "Game", …) — NOT freedesktop strings
  ("AudioVideo" / "Audio" / "Audio;Video;" all → "invalid category", and
  the failure surfaces only AFTER the full cargo release build, ~2 min of
  nothing). The error has no did-you-mean when confidence < 0.8.
- **rpmbuild NOT required** — the tauri bundler assembles the RPM itself
  (cpio+gzip); only `rpm` is needed to inspect.

**`bundle.targets` is `["rpm"]`, not `"all"` (2026-09-01, owner confirmed).** With
`"all"` the build writes a good RPM and THEN dies on the AppImage step — `linuxdeploy`
cannot fetch its tools here — so every release build ended in a failure that meant
nothing. The config is strict JSON (no comment can carry the reason), so the reason
lives here: RPM is the packaging format, AppImage was rejected (not native to
Fedora/KDE, no package-manager integration), Flatpak stays deferred.
- Built: `src-tauri/target/release/bundle/rpm/Songstress-0.1.0-1.x86_64.rpm`
  (~11 MB). rpm -qip/-ql/-qp --requires verified: Name songstress 0.1.0-1,
  MIT, /usr/bin/songstress, /usr/share/applications/Songstress.desktop
  (Name=Songstress, Exec=songstress, StartupWMClass=songstress,
  Icon=songstress, Categories=AudioVideo;Audio;Music;), hicolor icons
  32/128/256@2, Requires mpv + kdialog (+ webkit2gtk/gtk3).
- Dev unit stopped before install (MPRIS name / appmenu registration are
  single-instance per name).
- **Installed + verified from KRunner (2026-08-26, user ran the dnf lines)**:
  `/usr/bin/songstress` running (pgrep); full library grid rendered from the
  SHARED ~/.local/share/com.yossi.songstress DB (dev→packaged data survives
  ✓); MPRIS root + Player interfaces live (Metadata {}, PlaybackStatus
  Stopped — both on /org/mpris/MediaPlayer2, as built in Phase 4); Global
  Menu GetLayout returns Playback/Library/View/Help; KWin getWindowInfo:
  resourceClass `songstress`, desktopFile `songstress` (→ Better Blur DX
  WindowClasses match; isEffectLoaded(better_blur_dx)=true). Screenshot
  verified. NOTE: DNF 5 has no `-U` for `install` — the RPM path alone
  upgrades-or-installs.

**Remaining (user, then agent)**:
1. ~~USER: close the app, `sudo dnf remove songstress`.~~ DONE 2026-08-26 —
   AGENT verified clean: `rpm -q` → not installed, no /usr/bin/songstress,
   no system Songstress.desktop, hicolor icons gone, **data intact**
   (songstress.db 1.7 MB untouched) → dev unit restarted, watcher armed,
   zero scans (the 7c loop fix holds in dev too). User-level dev-identity
   entry still present (dev workflow restored).
2. ~~⬜ USER decision (only matters when the RPM is installed again): remove
   ~/.local/share/applications/com.yossi.songstress.desktop to avoid a
   duplicate KRunner entry — or keep it if dev work resumes (the dev
   app's panel identity comes from it).~~ RESOLVED 2026-08-26: user
   reinstalled the RPM and removed the user-level entry (kbuildsycoca6
   re-run); single launchable = /usr/share/applications/Songstress.desktop.
   To resume dev-only work later, recreate the user-level entry (Step 3
   notes: StartupWMClass=songstress).

### Step 0b — Frozen border artifacts on window move — 🔶 root cause demonstrated: fractional display scale

**Symptom**: fast side-to-side window moves leave a ~1px stale vertical line
at the old window border (captured via screen recording; both left and right
borders seen). Vanishes on any forced redraw. ALSO: switching to the app
shows stale translucency for a frame (glass backdrop shows the previous
background until a repaint) — same BlurCache staleness family, include in
the upstream report.
**Diagnosis RE-FRAMED 2026-09-19 — the August conclusion rested on a false premise.**
- ✅ Blur A/B (both directions, 2026-08-23): artifacts GONE with `better_blur_dx`
  unloaded, back when loaded. Effect is the culprit. Still true.
- ✅ RoundedCornersPass ruled out (2026-08-23): `CornerRadius=0` → artifacts still appear.
- ❌ **PREMISE VOID**: "Build already contains upstream's July/Aug cache+scissor+ceil
  fixes". `e8475d0` is the *"version 2.5.1"* commit dated **2026-06-23**; the COPR NVR's
  `20260910_063354` is the BUILD timestamp, not the commit date. The installed tree was
  **109 commits behind** and contained NONE of `f8417452`/`23e33084`/`d453bdb8`/`728307ff`.
  Only `85ef43b1` "scissor out painted areas only" (Jun 20) was in it — checked per commit
  with `compare/e8475d0...<sha>`, never by author date. Local proof that survives the
  stripped binary: `strings -el` finds `BlurStrength`…`ForceContrastParams` (10/10 controls)
  but NOT `BlurCacheIgnore`/`BlurCacheRateLimit`. Plain `strings` finds nothing for ANY key
  — KConfigXT emits them as `QStringLiteral`, i.e. UTF-16. `nm` is useless (COPR strips).
- ✅ **Rebuilt from source at `728307f`** (2026-08-06; all four scissor/cache commits + the
  `BlurCacheIgnore` killswitch). **Artifacts still appear.** The missing fixes were never it.
- ✅ Cache REUSE ruled out: `BlurCacheIgnore=true` (pipeline engaged, never reused) → still
  appears. Caveat: `m_ignoreCache` has ONE use site (`blur_cache.cpp:325`, a forced flush),
  so this exonerates stale cached CONTENT only, not the cache path.
- ✅ Expansion SIZE ruled out: `BlurStrength=1` (expandSize 10) vs 15 (up to 150) behave
  identically.
- ✅✅ **DECISIVE — display scale 200% (integer): NO artifacts. 160%: they return.**
- Measured from a cropped screenshot (1 image row = 1 device row): a SINGLE row, y=35,
  L 79-84 vs 73-75 neighbours; window's current top edge y=66 → offset 31 device px =
  **19.375 logical px** at 1.6×. One *device* row with no partial blend either side ⇒ the
  shortfall happens AFTER the scale multiply, at the device-space boundary. The stale row
  carries the effect's own transform (steeper gradient, `Saturation=150`), so the effect
  painted it and the repaint region for the vacated area was one row short — NOT "nobody
  requested a repaint".

**Mechanism, named**: on KWin ≥ 6.6.90 the whole damage/paint-region block of
`BlurEffect::prePaintWindow` is `#else`'d out to a bare `effects->prePaintWindow(view, w,
data)`. The pre-6.6.90 branch computed its blur area as
`view->mapToDeviceCoordinatesAligned(QRectF(blurRegion(w).boundingRect()).translated(w->pos()))`
— explicit device space, explicit alignment. The 6.7 path hands KWin only
`setPixelsToExpandRepaintsBelowOpaqueRegions(m_expandSize)` (an int, no scale multiply) and
`setEffectBoundingRect(blurRegion(w).boundingRect())` (logical, unmapped). No device-coordinate
alignment survives anywhere in the compiled path; KWin keeps geometry in logical scene coords,
so at 1.6× the old rect's device edge is fractional and the repaint truncates at it.

**S3 — NEW TO THE RECORD, AND BY DESIGN (do not file)**: blur is absent in the Overview and
during minimize/maximize. Owner-observed for months, never written down by any prior pass.
`window.cpp:156` returns false while minimized ("While minimized there's no reason to blur")
and `setIsTransformed(false)` clears the flag commenting "Needed e.g. for Magic Lamp effect
to not draw blur." Zero upstream issues mention overview. #65 "[REQ] change/disable blur on
window state" is the thread if the behaviour should be configurable.

**Tooling built (reusable)**: `~/src/bbdx-ab.sh {show|A|B|C|D|set|unset}` flips effect config
and reloads WITHOUT unloading, via `org.kde.kwin.Effects.reconfigureEffect` — the August
methodology's unload killed blur SYSTEM-WIDE for no reason. Reload genuinely re-reads
kwinrc: writing `BlitMode=7` logged `better_blur_dx: Invalid BlitMode value: 7`, and 0
warnings after restoring. Traps: `kwriteconfig6` wants `--key K --delete`, never
`--delete K` (exits 0, does nothing); the KCM's restore-defaults deletes keys an agent wrote;
`Effects.debug()`/`listEffects` are useless here.

**Action**:
- ⬜ Report upstream (xarblu/kwin-effects-better-blur-dx; repo active, pushed 2026-09-10).
  The suspect line is theirs, the API that dropped the alignment step is KWin's
  `BackgroundEffectItem` — file with Xarblu first, cross-link KWin if the conversion belongs
  compositor-side. Issue creation was restricted in Aug; commenting on #94 (ungated re-blur)
  or #108 (stale cached content on a **fractional-scaled** output) is the fallback.
- ⬜ Optional strengthening, both cheap, NEITHER DONE: 150% scale → proves fractional-ness
  rather than 1.6 specifically; a different translucent window (Konsole ~90% opacity +
  `BlurNonMatching=true`) → proves it isn't Songstress/Tauri/WebKitGTK.
- ⬜ Ask infinality to bump the COPR snapshot: the NVR stamps a Sept build date on a June
  commit and reads as "already contains the fixes". No packaged path forward exists —
  `dnf repoquery` latest == the stale 2.5.1 build, so "re-test after dnf update" is moot.
- ⬜ Box state: running a source build of `728307f` (sha256-identical to
  `build/src/better_blur_dx.so`); the RPM is **intentionally removed**, so `dnf update`
  won't restore or clobber it. A kwin/Plasma upgrade WILL break blur until rebuilt
  (README: works only for the exact KWin version built against; a rebuilt `.so` needs
  **logout+login** — DBus reload cannot swap the library, and Wayland has no `--replace`).
- Cosmetic meanwhile: artifact clears on any redraw over that area; integer scale avoids it.
- App-side fix impossible: the stale pixel is outside the moved window.
- Superseded upstream refs kept for the record: #92 (closed 2026-07-09, the day
  `ce052e39` ADDED the `BlurCacheIgnore` toggle — the August note "closed by adding fixes,
  not a toggle" is out of date), #14 (closed 2025-12-08, off-screen trigger, different bug),
  taj-ny #143 (artifacts at smallest blur strength, open — we did NOT reproduce that
  strength dependence).

### Up next (user-confirmed 2026-08-30, after the motion-audit fixes)

- **Skeleton loaders for the grid + artist list** — ✅ shipped 2026-09-02, and the
  EmptyState progress bar did NOT stay: the stage is now textless while loading
  (see the implementation log). Two loose ends left on purpose, both needing the
  owner's live library rather than the sandbox:
  1. ~~`BOOT_GRACE_MS` is a guess~~ — **measured 2026-09-02** on the live library:
     `get_library` over 251 albums / 4464 tracks is **46–50ms**, so the 150ms grace
     stays (≈3x, never flashes on a warm launch). The note's grace was measured too:
     a watcher rescan of the same library ran 83ms and `.scan-note` never appeared
     across 357 sampled frames. STILL UNVERIFIED: the note *appearing* on a genuinely
     long scan (Library → Full Rescan, ~25s, proven safe — the track upsert does not
     touch the `staged` column) — it needs the owner's eyes, not another agent run.
  2. `.sk-cover` carries no cover-mass shadow. The real tile does. Call: whether a
     placeholder should already weigh 18px of shadow, or whether weight arrives
     with the artwork. One line either way.
     ✅ CLOSED 2026-09-10 (owner call): weight arrives WITH the placeholder —
     `.sk-cover` carries the tile's `0 4px 18px rgba(0,0,0,0.35)` so the artwork
     fades in without the tile gaining mass. Scan note verified live by owner
     the same night.
- **Modal work (later, user-owned)**: TagEditor's entrance. The audit noted
  it mounts with zero entrance motion (TagEditor.svelte:251 `{#if open}`) —
  a ~180ms scale(0.97)+fade from center is the planned direction
  (transform+opacity only; center origin is correct for a centered modal).
  The first real modal of this round — the imported-music manager — shipped
  without entrance motion too, matching MusicFolders/About on purpose.
  ✅ SUPERSEDED 2026-09-10 (found on audit, no work needed): the whole modal
  family already animates — app.css carries the tier (scrim-in 200ms +
  surface-in 240ms, 12px rise + 0.99 lean-in for every modal) with symmetric
  outros (scrim-out 190ms / surface-out 150ms, each modal stays mounted
  until animationend, Escape backstop included). Tag editor, import manager,
  About, Music folders — all four, both directions.

### Explicitly deferred

- Artwork editing in the tag editor (Step 1 decision). — specced 2026-09-03,
  see **Tag Editor Redesign — working spec** at the end of this file.
- Crossfade option — the eventual escape hatch for the Phase 3 audible seam
  at Epicus Furor→Emerald Sword (source-file MP3 encoder-padding estimation;
  not fixable app-side).
- MPRIS-initiated volume changes do NOT write through to the settings DB
  (the frontend owns persistence and doesn't hear about them) — acceptable
  for now; revisit if it annoys.
- Visual pass on the import flow on the real library (user) — ✅ 2026-09-04 — full flow run for real (see log entry below 0.9.5).
- Menu label ambiguity: "Add music folder…" (import staging, Step 2a) vs
  "Music folders…" (library roots, Step 7c) sit in the same Library menu
  and are easily confused — **resolved 2026-08-31 (pass A)**: the staging pair
  is now "Import music files…" / "Import music folder…" (ids unchanged),
  "Music folders…" (roots) keeps its name, and "Full Rescan (rebuild)" became
  "Full rescan (all files)". EmptyState's "Add music folder…" stays — it opens
  the ROOTS flow, where that name is the correct one.
- ~~**Global Menu: theme is 2-state there while the app has 3 modes**~~ —
  **resolved 2026-09-03** by the full Global Menu pass (see "The broadcast
  mirrors the pane" at the end of this log): `MenuState.theme` is now the MODE,
  the Appearance menu carries three radio items, and every `pushMenuState()`
  field was re-checked for bool-vs-mode flattening (`stagedCount` carries the
  sidebar door's count; `playbarGradient` joined the state; shuffle/repeat/eq
  were already honest strings or true bools).

