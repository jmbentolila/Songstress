# Playbar progress style: straight line or waveform (2026-09-28, owner-driven) (plan archive)

Moved out of `PLAN.md` on 2026-10-24 so the plan file stays readable in one pass.
This is the record, not the state: current state lives in `PLAN.md`, and this
file is the evidence behind it. Sections are kept verbatim, in their original
order. The index of every section is `README.md` next to this file.

## Playbar progress style: straight line or waveform (2026-09-28, owner-driven)

An Appearance toggle — "Waveform progress bar" — chooses the playbar's seek
control: the shipped **straight line** (native range, gradient fill, accent
thumb) or a **waveform** of 80 mirrored pill bars, played portion in accent and
the rest a faint wash. Mirrored in the Global Menu (`appearance.playbar-waveform`),
persisted as `playbarWaveform`, reset by "Reset appearance". Default = the line.

- **Height follows the style.** Line mode keeps the original 84px shelf; the
  waveform needs the taller 101.4px (84 × 1.15 × 1.05, the owner's two bumps).
  `html[data-playbar]` carries the choice; every consumer of `--playbar-h` follows.
- **Quiet state = the waveform at minimum height**, not a separate dotted line:
  `WAVE_FLOOR` (0.05) is a small NON-zero amplitude, so the bars rest as a beaded
  line and expand straight out of it. (First attempt was a separate dotted line
  built from 80 degenerate round-cap segments — their caps merged into a
  bead-line; replaced by an HTML dotted rule, then dropped entirely for this
  unified approach. One mechanism, one animation.)
- **Bars animate.** A track change MORPHS from the heights on screen into the new
  silhouette; an entrance rises from the floor and an exit collapses back at the
  SAME 460ms / 0.35 stagger (the exit ran at 260ms and read as a snap). The tween
  reads a PLAIN mirror of the animated array, never the `$state`, or the effect
  re-enters every frame.
- **Both variants stay mounted and stacked** (absolute), cross-faded by an `.on`
  class, so the toggle never squeezes the lane; the inactive one is `disabled`
  AND `pointer-events: none` (a transparent absolute layer still hit-tests —
  without that the line could not be dragged).
- **Bars are stroked lines with round caps + `vector-effect: non-scaling-stroke`**,
  so the pill ends stay circular under the non-uniform viewBox scale.

### Peaks are REAL: symphonia decode + a SQLite cache (migration v7)

`track_peaks(trackId)` looks the track up, returns the cached `track_peaks.data`
when its `mtime_ns` still matches, else decodes with **symphonia** (pure Rust — the
RPM stays `depends = ["mpv"]`), reduces to 256 mono values blending each
bucket's peak with its RMS (`sqrt(peak · rms)`), normalizes them so the loudest
blend is 255, and caches them. **The decode runs OUTSIDE the DB
lock** (it is ~200ms of CPU and must not stall settings reads). Measured on a real
library track: **203ms decode, 1ms cached**. The frontend (`lib/peaks.ts`) caches
per session and resamples 256 → 80 bars taking the MAX per group, so a transient
survives. Formats: mp3/flac/m4a(AAC·ALAC)/ogg(Vorbis)/wav AND **aiff** (the
`symphonia` `aiff` feature — it maps to the already-compiled
`symphonia-format-riff`, and `sample/untitled-song.aiff` pins it in a test).
The ONE gap is **Opus** (`.opus`, and Opus-in-`.ogg`): symphonia 0.5 ships no
Opus decoder at all, so those degrade to the
quiet floor line rather than fabricating data.

### Prefetch + full-rescan backfill (2026-09-28)

The first play of an uncached track waited ~200ms on the decode, so the playbar
looked slow to update. Three additions:
- **`track_peaks` is now `async` + `spawn_blocking`.** As a SYNC command it ran on
  the MAIN thread, so each decode stalled EVERY other command — the playbar's own
  updates included. Off-thread now.
- **Album prefetch** (`lib/peaks.ts`): the PlayBar warms the rest of the current
  album's peaks, next tracks first, ONE AT A TIME (so a burst never competes with
  playback), so an album is instant after its first track.
- **Full-rescan backfill** (`peaks::backfill`): a FULL rescan precomputes peaks
  for the WHOLE library; an IMPORT precomputes just the files it indexed; a
  watcher/incremental scan does NEITHER (a trivial change must not launch a
  whole-library decode — those fill in lazily on play). Parallel with headroom
  for playback/UI: **`cores−1` above 10 cores, else HALF** (16 → 15, 8 → 4, 4 →
  2). Chunked at 64 so it persists incrementally and resumes if interrupted.
  Progress rides the scan phase as "Precomputing waveforms".
- **It does NOT hold `SCAN_RUNNING`.** The scan lock is released before the peaks
  phase (guarded instead by its own `PEAKS_RUNNING`), so a watcher rescan — which
  WAITS OUT `SCAN_RUNNING` — is not deferred for minutes; it runs its fast
  incremental pass concurrently. Connections now carry `PRAGMA busy_timeout` so
  the concurrent writers queue rather than colliding with SQLITE_BUSY.
- **Gated on the `playbarWaveform` setting** — a line-mode user never pays a
  library re-read for a feature they do not use (`[scan] peaks skipped (waveform
  style off)`). The frontend peaks cache is capped at 256 tracks; the SQLite
  table is the real store, so a re-fetch is ~1ms. Display aggregation changed
  from a per-group MAXIMUM to a group RMS without recomputing the backend
  cache: the same bytes now show more of the quiet structure instead of letting
  every loud sub-bucket fill the cell.

### Scrub-affordance trial — reverted at owner request

A neutral waveform preview cursor plus provisional elapsed-time readout was
implemented, evaluated, and removed. The shipped behavior keeps the native
range control and uses the accent boundary only for committed playback.

The cache is the SQLite `track_peaks` table, so it is **persistent** across
sessions — the table IS the precache; nothing is recomputed unless a file's mtime
changes. Stored rows also carry a `peaksAlgo` decode-metric revision: app startup
deletes only the derived rows on a metric mismatch, then backfills/prefetches
them, so an old peak-only row can never pose as the current peak–RMS blend.

### Light-mode accents: darker + no white (2026-09-28, owner-driven)

Light-mode contrast on the waveform traced to two accent problems: the
unplayed wash compounds to ~11% ink, and the played side inherits whatever
accent is chosen — a stored White clamped to mid gray (~1.3:1 on the
playbar). Two changes, both light-mode-only:
- **Every light accent darkens 10 HSL lightness points** after the theme
  clamp (`accentVariants` + the stock `--accent` tokens, GNOME fallback
  included). Stock light purple moves `#7c58f0` → `#4715ea`; `--active`
  follows the darkened color. Dark theme untouched.
- **White is disabled in light mode.** Its swatch renders dimmed with an
  "unavailable in light theme" label, and a stored white selection entering
  light mode falls back to Black (store-level, so it persists).

### Tile-size slider snaps to effective layouts (2026-09-28, owner-driven)

The slider used to walk nominal pixels 120–320 while the grid only reflowed
at column-count thresholds. It now offers one detent per layout that the
measured grid width actually produces (`tileStops`/`tileStopIndex` in
`buildRows.ts`, width published by AlbumGrid through a session-only layout
store). The readout names the rendered result (`212px · 5 across`); moving
the thumb always moves tiles. Keyboard arrows step one layout at a time.

### Grain +5%, tile readout simplified (2026-09-28, owner-driven)

GNOME matte grain bumped 5% on both themes (dark slope .184 → .193,
light .052 → .055, intercepts re-derived by each theme's rule). Tile-size
readout dropped the pixel count — `5 across` only.

### Light grid: softer cast shadows, deeper field (2026-09-28, owner-driven)

Light-mode edge shadows eased 0.22/0.30 → 0.14/0.18; the light grid field
deepened `246,246,249` → `236,236,241` (tokens plus `gradient.ts` SURFACE,
so artwork mixing follows the same surface).

### Current-track marker: mini equalizer (2026-09-28, owner-driven)

The ▶/❚❚ glyph in the tracklist's number slot became a 4-bar animated
equalizer: transform-only staggered loops while playing, frozen bars while
paused, static under reduced motion — both layers stay mounted and cross-fade
(560ms, owner-tuned) so the play/pause flip dissolves instead of teleporting.
Screen-reader label follows state (Now playing / Paused).

### Code audit (2026-09-28, owner request)

  target/ at 17G is normal cargo weight: 7.8G incremental, 6.4G deps
  (~1G of that in seven stale full songstress binaries from rebuilds),
  2G release. Gitignored, regenerable; reclaim with `cargo clean`
  (nuclear) or `rm -rf target/debug/incremental` (~8G, slower next build).
  Clippy pass done (clippy + rustfmt newly installed): ~37 warnings →
  zero. Auto-fix took the clones, casts, lifetimes and sort_by_keys;
  hand-fixed the 10-arg test helper (named-field Stage struct), two
  sort_by_keys, a let-else, two complex-type aliases and a NaN-sloppy
  float comparison in peaks.rs. Edition 2021 → 2024 surfaced 25 more
  collapsible-ifs (auto-fixed) plus one unsafe set_var in tests (now in
  an explicit unsafe block). Production unwraps audited: mutex locks +
  startup fail-fasts are fine, queue_jump's pops are range-guarded; one
  fresh unwrap in screen_pick.rs replaced with if-let. `cargo fmt` also
  normalized the tree, so this diff carries reflow churn throughout
  src-tauri/.

Gates: cargo 142, vitest 140, svelte-check 0, build ok.

### Fused scan ring + live stage line (2026-09-29, owner-driven)

A full re-read runs files → artwork → peaks, each phase with its OWN
done/total — so the ring's raw fraction restarted at every boundary and the
arc snapped back to empty mid-run, reading as a second scan popping in.
Two changes, both frontend-only (the backend already emits the phases):
- **One monotonic arc.** `scanProgress` (Sidebar) fuses the phases when peaks
  will actually run (waveform style on + full re-read or import): files
  0→0.35, artwork 0.35→0.5, peaks 0.5→1 (peaks owns the back half — it is
  the long decode). Every other operation keeps plain done/total. A phase
  reporting total 0 counts as its band complete, except "no events yet"
  (phase ""), which stays an honest empty ring. `ProgressRing` dropped its
  `{#key phase}` re-mount (the snap's mechanism) and fades in/out (160ms,
  off under reduced motion, same gate as the grid skeleton).
- **Live stage line.** The Scan group now renders `scanLine` under Re-read
  all files while a scan/full run is in flight — same quiet voice as the
  `.scan-note` under search, `aria-live` polite. The two live regions never
  show together (home layer vs. Library pane), so no double-announce.

Gates: svelte-check 0, vitest 140, build ok (no Rust touched).

SUPERSEDED same day (see next entry): the peaks tail left the scan for its
own background channel, so the ring fused two phases instead of three
(files 0→0.5, artwork 0.5→1) and the stage line reads the background peaks
progress once the scan reports done — one node, the text swaps, no pop.

### Waveform backfill decoupled from the rescan (2026-09-29, owner-driven)

The backfill ran INSIDE `scan_inner` before `scan-finished`: measured on the
fresh-wipe library, files 5.2s + artwork 5.1s, then peaks held the ring for
**4,588 tracks in 821s (~14 min)** — a 10-second tool taking 14 minutes and
pinning the progress UI to a phase nobody asked for. Now `scan_inner` ends
at the sidecar sync and `scan-finished` fires in seconds; `scan_library`
(full re-reads) and `import_music` (the batch just indexed) hand a
`PeaksScope` to `spawn_peaks_backfill`, which converges in the background:
first plays stay covered by the lazy decode + album prefetch, plain
incremental/watcher scans still trigger nothing. Progress rides a separate
`peaks-progress` channel with a terminal `peaks-finished` on EVERY path
(same discipline as `scan-finished`; the guard releases in the task on
success, in the wrapper on failure, so a panicking decode can never wedge
`PEAKS_RUNNING` true). The old mid-scan `SCAN_RUNNING` release hack is
gone — the lock is simply never held for the decode. Frontend: the scanner
store carries `peaksRunning/peaksDone/peaksTotal`, and the Scan-group stage
line shows them with no ring (no verb is running to hang one on).
Addendum same day (owner suggestion): `rescan_written` — the tag-save /
artwork-delete follow-up — takes `db_path` and fires the same background
backfill scoped to exactly the files it rewrote, so a retag's stale peaks
(a rewrite bumps mtime, which voids the cached row) refresh within seconds
instead of waiting for first play. One or two files of decode, silent when
the style is off or everything is cached.

Gates: svelte-check 0, cargo 142, clippy 0, vitest 140, build ok.
Follow-up, not this change: resume-at-startup when pending peaks exist
(retags leave stale rows until the next full re-read; lazy play covers
them meanwhile).

### GNOME window frame v1 (2026-10-02, owner-approved visual tweak → 0.16.3)

Our window is decorations(false) + transparent(true) (lib.rs), so Mutter draws
no frame; `.app` is fixed inset 0 with radius but no border/shadow, so on
GNOME the edge melted into the wallpaper. Ground truth measured from installed
libadwaita 1.9.4: radius 15px, 1px outer dark ring, 1px inner light hairline
(offset -1px) plus drop shadow. Smallest change: GNOME-gated rules on `.app`
in app.css — `border: 1px solid` paints the outer ring, `box-shadow: inset
0 0 0 1px` paints the inner hairline (inset so nothing clips at inset 0; an
outer ring would clip). Dark: border rgba(0,0,0,0.55), hairline
rgba(255,255,255,0.07). Light: border rgba(0,0,0,0.18), hairline
rgba(255,255,255,0.6). Radius, alphas, grain, KDE styling untouched.
Retune same day: dark outer ring softened to rgba(0,0,0,0.3) + hairline to
rgba(255,255,255,0.09) — owner's Nautilus corner clip reads light gray, and
libadwaita tokens confirm (borders are currentColor at 15%, window.csd outer
ring black at only 5%); the 0.55 ring was too heavy. Light rule untouched.
Second retune same day: outer ring to rgba(0,0,0,0.12), hairline to
rgba(255,255,255,0.1) — owner's ruling is the LIGHT line leads (it matches the
internal seams: sidebar border-right and playbar border-top both read
var(--border) = 0.1 in dark), so the hairline now equals --border exactly and
the dark ring drops to near-invisible like Adwaita's ~5%.
Root-cause fix same day: hairline moved from box-shadow inset to outline -
1px in BOTH themes (same alphas) — the inset shadow painted below the opaque
sidebar/grid/playbar and only peeked through on straights, dying at the bend;
the negative-offset outline paints above them, follows the radius, unclipped.

Gates: svelte-check 0, vitest, cargo test --lib, build ok.

### Picker hides uppercase extensions — case-insensitive audio/image filters (2026-10-03, owner report → 0.16.4)

Owner's `.WAV` rips were invisible under the import picker's "Audio Files"
filter (workaround: "All files" + navigate away and back to refresh). Root
cause: portal filter globs are case-sensitive on both backends (GTK pattern
match, Qt wildcard), so `*.wav` never matched `*.WAV` — while the scanner
lowercases before comparing and imported the same files fine. Same class of
bug in the symphonia probe hint (passed through raw) and the cover picker
(`.JPG` from phones/cameras). Second find in the same pass: the picker's
audio set had drifted from the scanner — `.oga` was indexed but unpickable
(hand-spelled `AUDIO_GLOBS` vs `scan::EXTENSIONS`, plus two more hand-spelled
kdialog copies).

Fix: `ci_glob` builds one bracket-class glob per extension (`wav` →
`*.[wW][aA][vV]`, digits untouched); `audio_globs()` derives from the
scanner's EXTENSIONS so the two can never drift again; all three pickers
(import, relink, cover-art) and both backends (portal + kdialog strings)
build from the same helpers. `image_globs()` gets the same treatment.
`peaks::compute` lowercases the probe hint. Tests: glob-shape +
scanner-coverage pin in lib.rs, uppercase-`.WAV` end-to-end decode in
peaks.rs. kdialog filter runtime strings byte-identical in shape to before
(newline separator preserved via `'\n'`).

Gates (2026-10-03, all green): svelte-check 0, vitest 140/140, vite build
ok, `cargo test --lib` 144/144 (incl. the two new tests) — Rust ran in the
gateway container after installing the toolchain live (rustup minimal +
Tauri -devel set, baked into both Dockerfiles for the next rebuild).
Owner live-confirmed the bug fix: `.WAV` rips import first try on the dev
instance under "Audio Files". Cargo.lock's package version refreshes on
the next host cargo run. Uncommitted.

### Security audit fixes — read_image confinement, CSP, set_setting allowlist, reveal guard (2026-10-03, audit v0.16.4 baseline → uncommitted)

Implements all four task findings from wiki/songstress-security-audit, nothing else. No version bump, no commit (owner review).

1. MED — `read_image` confined (lib.rs). The command took `state` now and serves only: canonicalized paths staged server-side from proven user intent (`pick_image` result + OS drop payloads staged in the new `on_window_event` DragDrop arm — drop events arrive via the backend, not renderer-suppliable) or files under configured music roots (`image_read_allowed`, component-wise `starts_with`). Symlinks resolve before the check, so a `cover.jpg -> ~/.ssh/id_rsa` stages its target and dies at the new extension gate (image list or `scan::EXTENSIONS`; previously ANY extension was served as raw bytes). 25 MB caps and the audio-embedded-art path preserved. Frontend callers unchanged (same signature — ArtSelector's picker/drop flows cannot be id-keyed: the files live outside the library).
2. LOW — CSP set in tauri.conf.json (was `null`): `default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'` (Svelte scoped styles are inline `<style>`); `img-src 'self' data: thumb: http://localhost:1420` (data: uploads/paste, thumb: prod covers, /thumb-http dev rewrite); `connect-src 'self' ipc: http://ipc.localhost ws://localhost:1420 http://localhost:1420` (invoke transport + Vite HMR); `media-src 'self'; font-src 'self' data:` (bundled Inter woff2); `object-src 'none'; base-uri 'self'; frame-ancestors 'none'`. Repo grep: no external URLs, no webfonts-by-URL, no iframes/objects, no prod eval.
3. LOW — `set_setting` allowlisted (`SETTABLE_KEYS` + `albumGradient:` namespace; `musicDirs`/`musicDir` rejected with a redirect to the music-folders commands). Folder changes still flow through `add/remove_music_folder` → `validate_new_root` → `persist_roots`, untouched.
4. LOW — `reveal_container` inserts `--` before the path for dolphin/xdg-open when (and only when) it starts with `-` (`reveal_needs_dashdash`); absolute library paths never do, so normal reveals are byte-identical.

Tests: +3 lib.rs unit tests (allowlist incl. `albumGradient:<id>` / `peaksAlgo` refusal, dashdash, gate incl. `/music-bc` vs `/music/b` and `~/.ssh/id_rsa` refusal).

Gates (2026-10-03, all green, gateway container): svelte-check 0 errors, vitest 140/140, `cargo test --lib` 147/147, vite build ok, `cargo check` parses the new tauri.conf (generate_context). NOT done: live dev-instance smoke check — no user session/display/systemd in this container (`systemctl --user` unreachable, nothing on :1420), so no CSP-meta probe, no picker/drop/reveal run. Owner to confirm on his box: covers render (thumb:/data:), tag-editor artwork upload from ~/Pictures pick + drag-drop, music-folder add/remove, reveal-in-folder.

### Dark-mode surface swap — grid darkest, both darker (2026-10-04, uncommitted)

Owner ask: the darkest dark-mode bg sat on the playbar/sidebar (`--bg-chrome`) with the lighter one on the grid (`--bg-grid`) — swap so the grid is darkest, and darken both. CSS-token-only change in `src/app.css`, three dark blocks (glass `rgba`, GNOME solid `rgb`, reduced-transparency solid `rgb`); light theme untouched; `--panel-bg`/`--panel-bg-strong` (popover/modal tier) deliberately left so panels stay lifted above the chrome.

- `--bg-grid`: (32,32,40) → (10,10,13), alpha 0.8 kept — now the darkest surface.
- `--bg-chrome` (sidebar/playbar via `.glass`): (22,22,28) → (17,17,22), alpha 0.7 kept — darker than before, lighter than the grid.
- Blue-channel ratio (~1.3x) preserved so the hue family doesn't shift.

Gates (2026-10-04): svelte-check 0 errors, 0 warnings. No restart needed (CSS hot-reloads); owner to eyeball on the dev instance. Uncommitted.

### Dark-mode surface swap follow-up — gradient-active playbar base pinned (2026-10-04, uncommitted)

Owner: don't darken the playbar gradient when active. Cause: the `.pb-bg` layers render at 0.7 alpha OVER the chrome fill (stops themselves are mixed against the hardcoded SURFACE constant in `lib/gradient.ts`, so only the 30% show-through changed). Fix in `PlayBar.svelte`: footer takes `has-gradient` while any gradient layer exists (covers the fade-out tail), and scoped rules pin the base to the pre-swap chrome — `rgba(22,22,28,0.7)` glass, `rgb(22,22,28)` on GNOME + reduced-transparency — via `background-color` only so the GNOME grain layer survives. Dark theme only. Gradient-off playbar wears the new darker chrome.

Gates (2026-10-04): svelte-check 0 errors, 0 warnings. Uncommitted.

### Window drop shadow v2 — blurred frame shadow (2026-10-04, Aina spec → 0.16.6)

Follows the 2026-10-02 GNOME-frame incident: the v1 ring (border + outline at -1px) stays untouched underneath; the shadow is a new tier, no DESIGN.md override. `.app` moves from `inset: 0` to `inset: var(--frame-margin, 20px)` (App.svelte scoped rule) and paints the two-layer outset `--shadow-window` seat from app.css into the margin — outset lives outside the border box, so the incident failure mode (inset buried under opaque children) cannot recur. KDE dark `0 6px 12px rgba(0,0,0,0.50), 0 2px 8px rgba(0,0,0,0.35)`, KDE light `0.22 / 0.12`; GNOME dark `0.55 / 0.40`, light `0.25 / 0.15` (hotter, no frost behind the matte slab). Rescale same day: first cut was `0 14px 40px` for the main layer, and a 40px blur cannot fade inside a 20px margin — it read as a hard-edged band cut at the window bounds (owner eyeball; DOM probe showed no element paints in the margin, compositor-extension border ruled out, extension disabled). Margin kept at 20px per owner decision; main layer rescaled to `0 6px 12px` (falloff 6 + 12 = 18px completes inside the margin on all sides), contact layer and all alphas unchanged. Radius untouched — the shadow follows `--radius-window` automatically. Owner-tuned alphas (0.8/0.7/0.36/0.30) untouched.

Gate: maximized/tiled drops margin + shadow via `html[data-window="maximized"] .app { inset: 0; box-shadow: none; }` (specificity (0,2,1) beats the scoped `.app` rule (0,2,0)), published by `syncWindowState()` in `lib/window.ts` at startup + on every `onResized`. No Rust change (`window_state.rs` saves outer size; the margin is inside). Known gap: tiled-but-not-maximized is not detectable through the Tauri API, so a Mutter-tiled window keeps its 20px margin — recorded, not fixed. Fullscreen handling deliberately left out per spec.

Risks per spec: Mutter edge-resize over the transparent margin should still grab (hit-test is on window bounds, not pixels) — if an edge goes dead that is a compositor finding, not CSS-fixable, fallback is a smaller margin or revert. KWin force-blur sits behind the margin, shadow paints over it (correct order; if muddy, drop KDE alphas by 0.05). At rest the shadow is a cached raster, zero per-frame cost; resize-lag dial on weak iGPUs is blur 12 → 8 with margin held at 20.

Patch bump 0.16.5 → 0.16.6 (user-visible frame change) in package.json + tauri.conf.json + Cargo.toml, same commit. Committed locally, NOT pushed.

### Playbar drag, attempt 1 — footer-root drag region (2026-10-05, owner ask → 0.16.7)

Owner wanted the bigger surface tried first (Aina had specced region on the track-info `.now` block only). Attempt 1 puts `data-tauri-drag-region` on the PlayBar `footer` root, so inter-column gaps and slack drag too. No fallback needed: Tauri excludes interactive descendants (`input`/`button`/`a`) from region dragging, and the owner eyeballed it live with resounding success — gaps drag, seek seeks in both styles, buttons click. Frontend-only (PlayBar.svelte markup + comment), no test changes, no restart needed (hot reload).

Gates (2026-10-05): svelte-check 0 errors, 0 warnings; vitest 140/140; `cargo test --lib` 147/147; vite build ok.

Patch bump 0.16.6 → 0.16.7 in package.json + tauri.conf.json + Cargo.toml, same commit. Committed locally, NOT pushed.

### Transport row gap 14px → 20px (2026-10-05, owner ask → 0.16.8)

One-value CSS change in PlayBar.svelte `.transport`: `gap: 14px` → literal `20px` with a comment naming the DESIGN.md 20px gap token — deliberately NOT `var(--gap)`, so the row keeps its spacing if the token ever moves. Button sizes, washes, tokens, seek, themes untouched.

Probed live (devctl, computed + rects): `gap` computes `20px`; five 30px buttons at x 525/575/625/675/725 → four inner gaps of exactly 20px, flanks 0/0 — identical across wave/line × dark/light (style flipped via the Appearance toggle, theme via System→Light→System; both restored, menu closed, app left as found).

Gates (2026-10-05): svelte-check 0 errors, 0 warnings; vitest 140/140; `cargo test --lib` 147/147; vite build ok.

Patch bump 0.16.7 → 0.16.8 in package.json + tauri.conf.json + Cargo.toml, same commit. Committed locally, NOT pushed.

#### Outer album-skip clearance (2026-10-05, Aina spec → 0.16.12)

MIS-TARGETED, corrected same commit: the first cut put the 8px on the outer album skips (`skip-outer` + margin rules, since fully reverted) — owner: the crowded ones are the prev/next TRACK steps (second and fourth). Final: `nth-child(2)` margin-right 8px, `nth-child(4)` margin-left 8px, dated comment; no classes, handlers/aria/svg untouched, gap byte-identical. Net: play-adjacent intervals 28px equal, all others 20px, flanks 0 — probed identical across wave/line × dark/light (both restored, menu closed).

### Edge-shadows softened + GNOME ring strengthened (2026-10-05, Aina spec → 0.16.9)

GNOME-only per owner decision (no KDE ring added). App.svelte `.edge-shadows`: dark top 0.22→0.14, shelf 0.30→0.20; light top 0.14→0.09, shelf 0.18→0.12; 26px/34px lengths unchanged. app.css GNOME ring: dark border 0.12→0.38, outline 0.10→0.24; light border 0.18→0.32, outline 0.60→0.70; offset -1px unchanged. Playbar seam untouched. DESIGN.md: Cast-light line re-tuned with date, app.css frame comment carries the new values, plus a dated window-ring exception note (the 2026-10-02 "hairline equals --border" ruling no longer holds for the window edge — wallpaper, not sibling chrome, is what it reads against).

Probed live (devctl, capped pass): dark `backgroundImage` 0.14/0.20, ring 0.38/0.24, seam dark `--border`; light flip (attribute-only, store untouched — these four properties are pure functions of `html[data-theme]`) 0.09/0.12, ring 0.32/0.70, seam light `--border`; restored to dark.

Gates (2026-10-05): svelte-check 0 errors, 0 warnings; vitest 140/140; `cargo test --lib` 147/147; vite build ok.

Patch bump 0.16.8 → 0.16.9 in package.json + tauri.conf.json + Cargo.toml, same commit (restarts the dev unit; owner moves the window himself). Committed locally, NOT pushed.

### Corner resize handles (2026-10-05, owner finding → 0.16.10)

Owner: the four window corners moved the window instead of resizing it. Mechanism (traced, not guessed): all three `data-tauri-drag-region`s are bare attributes, so drag.js fires `start_dragging` (move) only on direct clicks; the 20px margin belongs to no element (elementFromPoint = BODY at corners); nothing anywhere requests a resize — Mutter draws no grabs on decorations(false) windows and Tauri wires no resize gesture (tao's DragResizeWindow sat unused). So move always won by default. Fix is frontend-only and contained: four transparent 20px `.resize-corner` squares in the margin (siblings of `.app`, outside `contain:paint` clipping), mousedown → `startResizeDrag` → tao → gtk `begin_resize_drag`; hidden maximized via the shared `html[data-window]` gate. Edges untouched. Owner to try each corner live (a real resize gesture is not assertable from devctl).

Gates (2026-10-05): svelte-check 0 errors, 0 warnings (also proves `startResizeDragging` exists on the installed API); vitest 140/140; `cargo test --lib` 147/147; vite build ok.

Patch bump 0.16.9 → 0.16.10 in package.json + tauri.conf.json + Cargo.toml, same commit. Committed locally, NOT pushed.

### Corner resize diagnosis probe (2026-10-05, uncommitted diagnostic on 0.16.10 → amended)

Owner on fb4362b: resize acts on the border of the margin, not the corner squares. Verified live: all four 20×20 handles render at the viewport corners, topmost in hit-test, correct cursors; input region is full-surface (nothing in tao/wry restricts it); `plugin:window|start_resize_dragging` is registered and direction mapping is correct — so the chain is airtight on paper and the `try/catch` was hiding the runtime truth. Added TEMPORARY `dataset.resizeProbe` stamps (mousedown/resolved/rejected) so one real owner corner-press discriminates delivery failure vs lost request vs Mutter refusal; remove after diagnosis. No versioned behavior changed, no bump.

### Corner resize handles REMOVED — client-side corner resize unwinnable (2026-10-05 closer)

Owner pressed the live BR handle: `resizeProbe` ABSENT, with the stamps proven live in the served code — the press never reaches the webview. Motion/hover IS delivered (cursors), so the margin is inside the input region; the mousedown is consumed pre-client. Only Mutter can do that (synchronous edge-zone grab on the decorationless window beats the client's async IPC path every time), and whatever Mutter does with corner grabs yields no corner resize. Full chain was verified airtight (registered command, correct direction mapping, handles topmost in hit-test) — there is no DOM/JS lever left: no element, however positioned, can receive a press the compositor keeps. Handles + helper + stamps removed (window.ts back to move/min/max only; maximized gate untouched). The remaining lever is making Mutter not-grab the margin — `_GTK_FRAME_EXTENTS` / `gdk_window_set_shadow_width` so the margin reads as shadow — which needs direct gtk/gdk deps plus raw-handle plumbing in Rust (tao exposes no flag; verified by grep). Beyond a flag: NOT implemented. Options if corners ever matter enough: (1) Rust shadow-width plumbing, (2) accept Mutter edges + glass drag as the window's geometry (current state), (3) re-test on future Mutter (grab behavior is compositor-version policy, not API). No version bump (dead code removal, no observable behavior change — the squares never fired).

### Bottom-corner resize zones + missing resize capability (2026-10-05, owner-verified → 0.16.11)

Bottom-only overlay zones (Aina spec) beat the ancestor move via capture intercept; presses arrived but `startResizeDragging` threw until `core:window:allow-start-resize-dragging` was allowlisted (move permission existed, resize did not). Owner pressed BR live: SouthEast:rejected named the denial; after the one-line capability fix plus dev-unit restart, both bottom corners resize. Gates green. Patch bump 0.16.10 → 0.16.11, same commit. Committed locally, NOT pushed.

### Track-step triangles optically centered (2026-10-05, owner ask → 0.16.13)

Prev-track path center sat 9.5, next-track 6.5 (16-unit box, center 8) — backwards from optical. Pure ±2 x-translates (shape/size untouched): prev `M12.5 3 L6.5 8 l6 5 Z` → `M10.5 3 L4.5 8 l6 5 Z` (center 7.5), next `M3.5 3 L9.5 8 l-6 5 Z` → `M5.5 3 L11.5 8 l-6 5 Z` (center 8.5). Probed getBBox 7.5/8.5, width 6 both, buttons unmoved. Patch bump 0.16.12 → 0.16.13, same commit. Committed locally, NOT pushed.

### Expanded-panel hybrid choreography (2026-10-09, Aina spec → 0.16.14)

Slot keeps the px-height animation (growDur/closeDur unchanged, settle-to-auto at rest); the eye follows compositor-only motion on ONE `.glide` wrapper inside the `.inner` clip mask — top edge pinned, content starting 12px up in the growing box. Open: opacity 0→1 200ms ease-out + translateY(-12px)→0 300ms ease-drawer, same frame as the height move, runs shorter than tall grows. Close (incl. ghosts): opacity →0 + →+8px, both 160ms. Same-row swap: 160ms fade + translateY(-6px)→0 on the incoming block, hidden committed under a suppressed transition + reflow before the content swaps, no height motion unless size mismatch. Two-beat stagger, two static delays only: header t=0, tracklist + footer +50ms, same durations/curves. Transition strings static in CSS per data-cstate; values inline (attribute-driven values set in the same batch as their transition string lose the transition on this engine). The ONE spacing fix: slot margin-top (via closest, own slot only) + expander margin-bottom take the actual move duration inline a frame before the phase flip (static CSS stays as pre-JS fallback). Ghosts mount at measured px with content final, close the same frame the host grows; generation guards extend to all content frames; visibleTrackIds count changes re-settle height from current px with content untouched (armed on count change only, never on album swap). Reduced motion mounts final values, zero travel (global kill switch owns the CSS half, untouched). No DESIGN.md change (existing tokens only; alphas/padding/radius/shadow untouched). Probed on Helloween Pumpkins United (2 tracks, al-aecd…) × Survivors Anison no Kokoro (223 tracks, longest of 252, al-15534…): fresh opens hit opacity 1 in ~130–205ms post-start, margins pre-seeded 0, ≤2 zero-opacity frames post-mount; both cross-rows start host+ghost within one frame; ghost travel +8px confirmed; mid-grow retarget ghosts from current px (2700) with a single close and the host ending 593/1/none; forced-RM run shows final-values first paint + zero travel (height freeze left to the real OS switch, declared). Gates green. Patch bump 0.16.13 → 0.16.14, same commit. Committed locally, NOT pushed.

### Expanded-panel retune (2026-10-09, Aina spec → 0.16.15)

Spread plus floor for the 0.16.14 reads-instant short opens: fade 200→320ms, travel -12px/300ms→-16px/380ms, beats 0/+50ms→0/+80ms (reversed on close: beat-b 0ms, beat-h +80ms), growDur floor 200→320ms (slope 0.6, cap 500 untouched; tall character unchanged). closeDur retired — exit is the exact mirror on the same clock (upward -16px travel, never eyeballed before; probed, numbers below). Same-row swap: hide path unchanged (-6px suppressed commit), reveal runs the open strings. Shell shadow moved off `.expander` onto a `::before` (position relative, inset 0, inherit radius, pointer-events none) whose opacity rides the beat-b value with per-cstate delays (+80 open/swap, 0 close); geometry still follows the box, --shadow alphas untouched. Five dated standing-law overrides written (AGENTS switch-160ms + grid-template-rows staleness, DESIGN shorter-exit exemption + Cards ::before amendment, PLAN <300ms budget extension). No new curves/tokens. Probed Helloween Pumpkins United (2 tracks) × Survivors Anison no Kokoro (223, longest of 252): short open hits 1.00 at ~292ms with travel landing ~457ms (≈380 clock) and ::before==beat-b to 6 decimals; short close mirrors (fade ~283ms, travel to -16.0, margin -20 with height); cross-row onto tall starts host+ghost within one frame, ghost exits upward to -16.0 with a single close, host hits 1.00 at ~348ms; tall close runs the 500ms cap with reversed beats proven (beat-h still 1.000 while glide reads 0.382); forced-RM run shows final-values first paint and zero travel on host and ghost (height freeze left to the real OS switch, declared). Filter re-settles reuse growDur (320 floor) — deliberateness left to the owner eyeball. Gates green. Patch bump 0.16.14 → 0.16.15, same commit. Committed locally, NOT pushed.

### Close-hitch fix (2026-10-09, owner eyeball → 0.16.16)

Owner: on close the content shifts up first, then starts its upward slide (open approved). Per-frame probes found no value snap: all seven close transitions share one startTime, values ride smooth easings. The defect was a shape sum — the slot/expander margins ran ease-out (front-load ~7.4px of the 20px gap in the first frame-pair, measured) while content travels the drawer, same direction; tracked-content first step was ~11.5px vs ~3.8px on the margin-static open. Ruled out with data: rest-vs-start mismatch (rest is none/1, transitions run from it), content arming late (shared startTime), beat staging as motion cause (beats are fade-only on the single glide layer; +80ms reversal verified in values), shadow desync (::before==beat-b to 6 decimals both directions), viewport travel (scrollTop rock-steady 6331 through the close). Fix: margins run var(--ease-drawer) — one velocity story for top edge + content; durations still match the height exactly, heights stay ease-out mirrored, content strings untouched, RM intact. After: margin first step -4.20 (drawer prediction), tracked sum ~-8.1px, transform monotonic none→-16 with no jumps, beat-h still holds through the +80ms delay. Gates green. Patch bump 0.16.15 → 0.16.16, same commit. Committed locally, NOT pushed.

### Artist-switch shadow linger (2026-10-09, owner report → BLOCKED, suspect falsified, no code change)

Owner: with an album expanded, switching artist tabs leaves the expanded album's shadow hanging a beat after its content is gone. Suspect was the ::before shadow riding beat-b with the host collapsing to 0px sans close. FALSIFIED with data: Sidebar.select() clears both expandedAlbum slots synchronously with the tab change, so the host ALWAYS plays the real mirrored close during the 160ms exit (short: h 303→29, op/sh 1.00→0.08 lockstep; tall: h 3846→1176, op/sh lockstep) and unmounts mid-close at the flip — beat-b fades every time, and post-flip DOM holds zero .panel-slot / zero wide-shadow elements for 540ms+ (width>500px sweep: exactly one ::before pre-switch fading in lockstep, none after). All seven close transitions share one startTime (prior probe), so no arming stagger either. Variants all clean: settled-short, settled-tall, mid-grow switch, ghost-in-flight switch, switch-to-containing-tab (key reuse continues the close to 0/0/0 in the new tab — correct). Class check: no search-filter or retarget path yields 0-height-with-shadow-1 (mounts are hidden-at-0 or revealed-with-close-playing; reuses continue continuously). No fix written — nothing to fix in reach of a probe; needs the owner's exact repro recipe (which album/tab, search active?, reduced motion?, how long is the beat, on demand?). No version bump (record-only).

### Jupiter→Trillium shadow remnant (2026-10-09, exact recipe → BLOCKED, engine-side, no code change)

Exact recipe (Temperance/Of Jupiter and Moons al-5e766a… expanded h374, switch to Trillium/Alloy tab): host plays the real close during the exit (h 374→64, op/sh 1.00→0.14 lockstep), unmounts mid-close at the flip; post-flip DOM holds zero .panel-slot/.expander/.glide for 800ms+ and a document-wide big-element inventory names no painter (only legit window ring + tile art). Five prior variants (settled-short/tall, mid-grow, ghost-flight, switch-to-containing-tab with key-reuse close completing to 0/0/0) all identical. Yet the owner screenshot shows a soft panel-geometry band in the settled Trillium grid — a painter with no node. Named node that painted it: the Jupiter panel's `.expander::before` (1129px, 0 10px 32px, last sampled sh 0.14 pre-unmount; the only 32px-blur shadow source in the grid). Working theory, stated as theory: WebKitGTK retains the unmounted mid-transition layers a beat (7 CSSTransitions cancelled in the same flush as a 40-row DOM surgery; the grid's 0→1 entrance restore can re-brighten the retained quad — faint text reads gone, wide faint band reads hanging, then it clears). Pin intact (2.52.1), so not the 2.54 regression. Confounds on record: /usr/bin/songstress (0.16.13 RPM) has been running since 17:56 alongside dev — confirm the eyeball was on the dev window; HMR staleness ruled out (fading ::before measured live this session). No app fix written: every DOM-side state is already correct, and layer/compositor behavior is unverifiable from DOM probes. Owner discriminators: on-demand repro?, cold unit restart?, does it follow tab size/heavy unmounts? No version bump (record-only).

### Gated-flip experiment (2026-10-10, owner-approved → REMOVED after measurement, no code ships)

Built exactly as proposed: window-global `__songstressGateFlip` arm (one boolean read when off — normal paths byte-identical), holding the old grid past 160ms until open hosts settle closed (phase poll every 50ms, +800ms cap, nested waits ride switchTimers so re-clicks cancel). Baseline determinism (gate off, 2/2): flip ~160-173ms, unmount mid-close (h35-38/op0.09-0.10, transitions running). Gate hit rate 5/5: flips at 520/534/532/524/525ms, every unmount at rest (h0/op0.00, no running transitions) — engagement deterministic, mechanism delta DOM-proven. Side notes: flip lands ~120-170ms after nominal close end (timer slop under the exit commit); a benign pre-existing `derived_inert` svelte warning fires on switch teardown with and without the gate. Verdict is CONDITIONAL by construction: the retention trigger (mid-transition unmount) is eliminated, which predicts the remnant gone — but pixels are the owner's (no screenshots/readback exist on this box; canvas-foreignObject readback is SecurityError-blocked). If the remnant persists with the gate on, the hypothesis is killed (retention is not transition-dependent) and attention moves to paint-invalidation paths. Rig removed in-run (global deleted, code reverted, normal 160ms flip + mid-close unmount re-verified post-removal: flip 173, h28/op0.07). If pixels confirm, the follow-up ships a designed gate (real state, not a window global) with owner-approved slower switches — this experiment is not that. No version bump (record-only, 0.16.17 stands).

### Angus→Arjen outline (2026-10-10, exact pair + controls → engine-side with mechanism, no code change)

Exact pair (Angus McSix 1 album expanded h374 → Arjen/Alloy tab): close plays during exit, flip unmounts mid-close, post-flip DOM holds one clean tile row through 727ms (full per-child census with rects/borders/backgrounds). Mid-grow variant identical. Controls: Angus→Helloween (multi-album dest) clean, 2 rows, 0 slots; Angus→All Artists yields the CORRECT collapsed memory slot (h0/op0/shadow0/closed — the feared 0-height-with-shadow-1 state does not occur). Prior coverage stands (5 Trillium/Jupiter variants, ghost-flight, containing-tab reuse, cold boot, inventories, border-box hunt, wide-shadow sweep). Ruled out with data this round: focus path (no :focus-visible rules on artist rows; devctl click verified DOM-level in devtools.ts, same toggleExpand/select handlers — no OS pointer exists), drainGlide scroll race (harmless: detached-tile writes clamp to 0 by sign), track-row outros delaying unmount (transition:fade|local blocks ancestor-driven outros by design — slots:0 one frame post-flip proves it), stageHeight/gridWidth paint paths (skeleton-only/store-only), tile/instance reuse (same-album continuity or destroy). Reproducible mechanism, stated precisely: EVERY artist switch unmounts the open panel MID-CLOSE (160ms flip < 320–500ms close) with 7 CSSTransitions running, in the same flush as row surgery + grid 0→1 restore + scroll reset; the retained quad is the panel's border layer at unmount opacity (~0.1) — crisp 1px survives perception where the opaque fill (== background) and blur shadow (invisible) do not — clearing on a later repaint. DOM-side deterministic, paint-side engine-nondeterministic (jank/load lottery), which fits intermittent triggering; current design already minimizes it (close-during-exit dims the unmount; deferring unmount till close end would change beloved switch timing — not done blind). Confirming experiment if ever wanted: gate the flip on close completion and watch the remnant vanish (owner call first). No version bump (record-only).

### Shadow arrival delay removed (2026-10-10, owner call, still 0.16.17, no bump)

Owner: the retune shadow must fade in AS the content moves, no delay. One-line change: .expander::before open+swap transition-delay 80ms → 0ms (same 320ms ease-out fade, same beat-b value source). Close untouched (already 0ms); content beats unchanged (header 0, tracklist +80 — still visible: beat-b holds 0.000 while glide/shadow lift). Probed open: shadow==glide opacity every frame from motion start (0.472/0.472 at first moving frame, 0.654/0.654 next — previously shadow sat 0.000 through ~130ms). Probed close: shadow==glide==beat-b throughout (byte-identical pattern to pre-change). RM intact by construction (untouched mount-final branches, static strings still kill-covered). Gates green.

### Stale-cover at flip (2026-10-10, lifecycle-closed mechanism → SHIPPED unversioned)

Owner lifecycle fact closes it: the remnant clears iff real new pixels paint over it — retained damage, not retained layers (an orphaned layer would survive repaints; the zero-pixel flush failed precisely because transparent damage elides). All probes sampled settled DOM post-populate, hence always clean. Fix: `.stale-cover` permanent-hidden overlay div (last .grid child, abspos, grid-bg paint, pointer-events none, no z-index/tokens/motion) shown for exactly one frame post-flip over the measured host-slot rect + shadow skirt (grid-relative, clamped to grid), then released — two guaranteed re-rasters from the live panel-less tree. RM-safe (instant, sub-perceptual); idempotent under re-clicks (plain rAF fields, hide-first); ghost/close paths untouched; multi-row dests may see one bg-colored frame over entering tiles (subliminal, noted). Execution proven by computed readback on the exact pair: flip ~t160, cover block 1129x109 at t188 only, none before/after; control pair normal with cover released. Falsification built in: a promoted-layer hold would paint ABOVE the cover (z-auto tree order) — outline persisting over it kills damage theory with data. Gates green. No version change per standing owner order.

### Flush failed → layer retention; blue unresolved (2026-10-10, owner pixels, flush REMOVED)

Verdict on damage theory in one line: owner pixels with cold restart show marker + rect unchanged, so repaint is not the release path — damage-staleness dead, flush removed as passenger (AlbumGrid back to pre-flush code, verified clean tree). Layer findings with numbers: at-flip promotion inventory = .glide (will-change permanent layer holding panel@op~0.1: border+fill+faint content — border-only read plausible, crisp survives low opacity), beats/::before (transient opacity layers, faint), grid (opacity layer during exit/entrance); repaint cannot release retained layers (only re-raster live ones) — consistent with the flush failing. No layer fix built: collapsing will-change pre-unmount is unverifiable from here and risks praised motion for an engine-side hold. Blue: live sources exhausted — post-flip accent sweep names nothing at the spot (only sidebar row + chrome), programmatic focus paints no ring anywhere, eval clicks focus identically to real ones, and the old tile's expanded ring fades transparent ~10ms before flip (outline-color 150ms < 160ms flip) — retained-ring would be faint, yet Ciel reads saturated #1659ab. Marker-tracking verdict: UNDETERMINED from here (both marker and outline are owner-visible only); need paired observations (blue without outline? vice versa?). Decisive owner-side tests proposed (one repro, no code): elementFromPoint at the remnant coords (null/body = stale paint proven; a node = named instantly) and a scroll nudge (stale screen-space quad stays, DOM-attached moves). No version bump.

### Stale-quad flush at flip (2026-10-10, blue-marker evidence → SHIPPED unversioned, 0.16.17 stands)

Ciel's light-mode read redefined the remnant again: saturated accent-blue (22,89,171 == GNOME-light --accent #1659ab, verified live) fragment at the old tile zone + shadow sides/bottom with no top = the exiting ROW's tile ring + panel shadow retained across the flip (focus ruled out: programmatic focus paints no ring anywhere, and eval clicks focus identically to real ones). Fix: one-shot paint invalidation at every artist-switch flip — transparent 1px outline on .grid for exactly one frame (double rAF around the flip flush), then released. Zero pixels ever change; both toggles invalidate so retained quads re-raster from the live (panel-less) tree. Inline (a scoped class trips unused-selector); inset inside layout; no z-index/tokens/motion; RM-safe (instant, sub-perceptual); idempotent under re-clicks (plain rAF fields, never stranded ON); ghost/close paths untouched. Execution proven by computed readback on the exact pair: flip ~t160, outline solid/1px for one frame at t184, none before/after; control pair (Angus→Helloween) normal with outline released. Marker/outcome verdict is the owner's (no pixel readback exists here): if the blue fragment + outline stop appearing across runs, the flush stands as the fix; if they persist, damage-staleness is killed and only layer-retention remains. Gates green. No version change per standing owner order.

### Outline verdict: stale paint via small-damage commits (2026-10-10, gate killed it, mechanism stated)

Pixel verdict IN: with the gate (unmount at rest, zero transitions running) the outline persists LONGER, scaling with the held duration — retention is NOT transition-dependent, gate hypothesis killed, no gate ships. Redirected hunt (in-exit outline): per-frame tail on Angus→Arjen shows the panel rect NEVER shrinks (content-sized 374 throughout; the clip does the shrinking), content fades 1.00→0.07 with beat-h holding 1.0 through its +80ms delay, flip unmounts at h28 — the redirect's "content 0 + full border" state never coexists above perception (grid exit multiplies everything to ~0 by t~120). Reproducible mechanism, stated precisely: EVERY switch unmounts the open panel, but small commits (1→1 rows) don't invalidate the panel's compositor tiles while big commits (40 rows) repaint everything — the retained quad is the border layer at unmount opacity (~0.1), re-brightened by the grid's 0→1 entrance restore, lingering exactly where no new content ever paints (below the single destination tile) until incidental damage clears it. This fits every fact at once: artist→artist only (tiny damage), never involving All (full repaint), below-destination geometry, crisp-border-no-fill read (fill==background, shadow sub-perceptual at 0.1), intermittence (retention lottery), gate stretching (real fade + invisible hold + stale reappearance). App DOM is correct in all variants; the only app-side lever (forced post-flip invalidation) is unverifiable without pixels and unbuilt. Owner discriminators: does a jiggle (scroll/hover) clear it instantly? does it ever survive on a multi-row destination? No version bump (record-only).

### Linger outline, resolved + shadow-band dither veil (2026-10-09, Ciel pixels → 0.16.17)

FINDING 1 (outline): Ciel's zoomed read redefined the remnant — a full-grid-width EMPTY crisp 1px rounded outline (radius 8-12, stroke ~#1B1B1E on ~#101013, no blur), i.e. a border with no fill: the .panel border, not the blurred ::before shadow (stale-shadow theory retired). Combined with zero leftover nodes across two methods (slot/expander/glide counts + document-wide big-element inventory, post-flip 540-800ms, plus a cold unit restart with healthy tokens) the verdict is engine-side stale paint of the unmounted mid-close panel layer: at ~0.1 opacity its opaque fill reads identical to background ("empty"), its blur shadow reads invisible, and only the crisp border survives perception — then it clears on a later repaint. Ruled out with data: HMR/stale unit (cold boot), focus rings (no such rules on artist rows + color mismatch), devctl-vs-real-click path (identical handler; no OS pointer exists), drainGlide scroll race (scroll 12832→0 at flip, stable — detached-tile writes clamp to 0 by sign), tile/instance reuse (same-album continuity or destroy; alleged ghost paths unmount clean). No code change for Finding 1; needs no owner repro beyond what Ciel read.

FINDING 2 (shadow banding, FIXED): Ciel's strip read confirmed the ::before blur ramp stepping (~4 levels over ~100px below the panel; panel fill ~(92,66,54), border row 296, near-black ramp (7,7,9)→(10-11)→(9,9,12)). Fix: `.veil` div as the expander's first child laying the standing --tex tooth over the band via mix-blend-mode: var(--tex-blend) (mean-safe; soft-light dark, multiply light), radial-masked to the falloff (blind-estimated, owner judges seams), opacity riding --shadow-op with mirrored delays, below panel content by tree order + one position:relative lift (no z-index anywhere). Geometry sides -56px / bottom -104px / top 0; KDE resolves --tex to none (zero change). Caught-in-probe save: the first cut deleted the ::before shadow rule outright (shadowBox:none) plus tried overflow:clip+clip-margin (clip-margin unsupported here; bare clip cuts shadows) and padding+negative-margins (do NOT unscoll in-flow overflow) — all reverted, all recorded in code comments. Verified by computed readback: grain URI active, soft-light, mask present, inset exact, veil rect 245/1241x478 covering the band, shadow geometry/alpha intact, veil==shadow opacity every close frame (1.00→0.00 lockstep), expander overflow visible. Known price: ~79px transient scroll room on open last-row panels (measured Theater: scrollH 15397 vs content 15318) — which is also what lets the shadow fully show at list end. Amplitude is tokens-only; tooth-fineness verdict and band breakup go to the owner eyeball (before/after pixels his loop). Gates green. Patch bump 0.16.16 → 0.16.17, same commit. Committed locally, NOT pushed.

### Shadow-band veil REVERTED (2026-10-10, owner pixels → still 0.16.17, no bump)

Owner screenshot: the veil renders as a flat light-gray slab (~111 across full width, ramping 89→131) instead of invisible tooth. Root cause: mix-blend-mode: soft-light does not blend on this WebKitGTK — same tile renders fine via background-blend on .glass fields, but through the mix-blend compositor path the layer paints ~its own mean gray with no variance (Ciel samples ±3 where raw noise would read ±18). Canvas-foreignObject readback ruled out as an independent witness (SecurityError — blocked on this engine). Fix: full revert of the veil (markup div, veil CSS + delay selectors, .inner lift, DESIGN Matte note) back to the 0.16.16 component; the ::before shadow rule it briefly deleted is restored byte-identical (verified: boxShadow rgba(0,0,0,0.45) 0 10px 32px, opacity riding). Standing behavior returns to the accepted static ramp + band (wiki precedent: dither attempts lost, owner grain-averse). Still-open, untried, needs pixel verification by nature: pre-dithered baked glow asset. No version change per standing owner order (0.16.14–17 consolidate at ship). Gates re-run green on the reverted tree.

### GNOME-dark shadow retune (2026-10-10, Aina spec, owner-approved → panel-motion entry, no bump)

On the near-black GNOME ground the 0.45-peak shadow kept only its core visible, reading as a hard band: one token value inside the existing html[data-de="gnome"][data-theme="dark"] block — `--shadow: 0 16px 56px rgba(0, 0, 0, 0.30)` (peak 0.45→0.30, blur 32→56, offset 10→16) — with dated notes in app.css and the DESIGN.md Shadow Vocabulary bullet. Light stays 0 10px 32px rgba(0,0,0,0.16), KDE dark stays 0 10px 32px rgba(0,0,0,0.45); --border, --shadow-window, cover/popover literals, curves, beats, --shadow-op plumbing, margins, ghost/close paths, RM all untouched, no JS change. Gates green. No version change per standing owner order.

### GNOME-dark ground lift (2026-10-10, aina-ground first step 22, owner-approved → panel-motion, no bump)

GNOME-dark --bg-grid rgb(10,10,13) → rgb(22,22,28) (one token, gnome-dark block only) so the 0.30 shadow shows real falloff; grid one breath above chrome 17,17,22 by design. Contrast pairs MEASURED live in the running build (tokens resolved + tile title/sub render match): primary text 16.4:1, dim counts composite (164.1,164.1,167.5) 7.26:1, badges same dim token 7.26:1, panel-strong gap kept at 4 channels ((26,26,33)-(22,22,28)=(4,4,5)); grid-vs-chrome gap (5,5,6). Dated notes in app.css + DESIGN Cards bullet. Iteration protocol: further steps land as uncommitted tweaks, eyeballed, committed only at the approved final (notes amended if value differs). Gates green.

### GNOME-dark shadow off (2026-10-10, Aina verdict, owner-approved → panel-motion, no bump)

Expanded panel casts nothing on GNOME-dark: `:global(html[data-de="gnome"][data-theme="dark"]) .expander::before { box-shadow: none }` (path intact — inset, radius, --shadow-op plumbing, per-cstate delays — one-line return). The 0.30 token reverted with it (falls back to base dark 0 10px 32px 0.45, still used by remaining surfaces); ground 22 stays; hairline as-is on all four sides; edge-shadows softened GNOME-dark only (top 0.14→0.10, shelf 0.20→0.14, lengths 26/34px unchanged; light/KDE untouched). Cover-mass/popover literals, --shadow-window, motion strings, ghost/close/RM all untouched, no JS behavior change. Live-probed on GNOME-dark: ::before none, root --shadow base fallback, edge 0.1/0.14, hairline 1px. Gates green. No version change per standing owner order.

### GNOME-dark ground REVERTED (2026-10-10, owner call, interruption-resumed → panel-motion, no bump)

Ground lift rgb(10,10,13) → rgb(22,22,28) reverted same session per owner call: with the panel shadow gone the lift's shadow job is moot and the owner wants the dark room back. One token back to rgb(10,10,13) in the gnome-dark block; app.css comment + DESIGN Cards bullet amended to reverted status with date. Mare's edits were complete in-tree at interruption; Ciel ran the gates green and committed. No version change per standing owner order.

### Checkbox → switch conversion (2026-10-10, Aina spec, owner-approved → panel-motion, no bump)

Toggle.svelte rewritten verbatim to spec: row tier 38x22 track / 16px thumb / 3px inset / 16px travel, compact tier 32x18 / 12 / 14px, track flex-none; OFF hover-wash + border with dim thumb (hover to text), ON accent track + accent-text thumb; motion transform + background-color 160ms var(--ease-out); focus inset ring via input:focus-visible + .track; RM transition:none; native input keeps role="switch" hidden 1px. All 7 sites flipped with zero usage changes (6 row + 1 small); EQ preset ellipsizes under narrow headers; Global Menu untouched. Probed live (dark/GNOME): row 38x22 + matrix(...,16,0) travel, small 32x18 + matrix(...,14,0), header single 28px line with ellipsis armed, inset-ring rule present (live Tab proof needs owner's key — programmatic focus doesn't match :focus-visible on this engine), RM instant by static rule + kill switch (stylesheet @media unstubbable from eval — OS-level proof is owner's). Static grep: no beziers, no new tokens, sole hex is the spec-dictated #fff fallback. Probe side effects: none (travel measured via .checked property pokes, no onchange fired; all pokes restored and state re-verified; UI left at rest). Gates green.

### Toggle thumb centering (2026-10-10, owner-found 1px-high thumbs → panel-motion, no bump)

Root cause in code, no probe needed: global border-box makes the 22px track's padding box 20px, so top:3px left a 3-over-1 gap (compact tier same arithmetic). Fix: top:2px (2+16+2=20, 2+12+2=16 — both tiers share the rule), comment states the arithmetic so it isn't "fixed" back. Travel/colors/motion/focus/RM/semantics intact. Probed: thumb center deviation 0 in both tiers (22×38/16px, 18×32/12px). Gates green.

### GNOME-dark shelf clip-ease (2026-10-10 round-2, Aina spec → panel-motion, no bump)

Owner strip ruling (1 playbar flat stays, 2 hairline stays, 3 shelf GOES, 4 grid stays): the shelf read as a band beside the bright 2px hairline (Ciel: 1 level of shelf vs 31-39 hairline), so the bottom layer is now a 12px clip-ease at 0.05 — half the alpha, one-third the length, sub-band-threshold, anti-alias of the hairline rather than a shelf. Named edge stays the playbar's border-top hairline (untouched); top layer kept (no hairline at the drag strip, hard cut would return). GNOME-dark block + comment only; base/light/KDE, geometry, motion, RM untouched, no new tokens. Live-probed: top 0.07/26px, bottom 0.05/12px. Gates green. No version change per standing owner order.

### GNOME opaque frame (2026-10-10, Aina spec, owner-approved scope EXPANDED to whole frame → panel-motion, no bump)

PHASE 1 AUDIT (live dev build, GNOME-dark, windowed, viewport 1471x922): the 20px transparent margin is symmetric — .app rect offset 20px on ALL four sides (top/left/right/bottom all 20) — while the shadow pools downward (offsets 6px/2px, zero upward), which is why Ciel pixel-guiltied the bottom (17-27px blend vs ~10px sides) while the other three sides leak quietly. elementFromPoint at all four corners (1,1 etc), all four mid-edges, and 10px-inset points: every one hits BODY — transparent window, wallpaper straight through — so top/left/right/corners are all LEAK, not just the bottom. Ring live-verified present (1px border rgba(0,0,0,0.38) + 1px -1px outline rgba(255,255,255,0.24), radius 14px); seat live-verified two-layer GNOME-dark (0.55/0.40). Token at fault on every side: --frame-margin (the margin) + --shadow-window (the bottom bias). Light theme by construction (margin rule is theme-independent; light seat tokens 0.25/0.15) — no state flip performed on the live app to prove it.

PHASE 2 IMPLEMENT: `html[data-de="gnome"] .app { inset: 0; box-shadow: none; }` in app.css after the maximized gate (specificity 0,2,1 beats App.svelte's scoped inset; ties the gate with identical values). Windowed GNOME converges to maximized geometry; v1 ring + extension 1px border carry every edge; maximized/tiled gate untouched; KDE margin + seat exactly as-is; both themes, both windowed states; no new token (literals 0/none), RM/light/KDE/curves untouched. Dated override notes in app.css GNOME-frame rule + DESIGN.md Shadow Vocabulary seat bullet. Frontend-only, no Rust gate met. Verification, stated exactly: live-DOM proof of the new rule was NOT obtainable — songstress-dev serves the main checkout (/home/yossi/Projects/Songstress), not this worktree, so HMR/reload only ever rendered base CSS (rule absent from all 25 live sheets post-reload); the unit was left untouched. Proof instead: production build carries `data-de=gnome] .app{inset:0;box-shadow:none}` (dist grep) + specificity arithmetic (0,2,1 beats the scoped 0,2,0 inset, ties the gate with identical values). Owner eyeball on his GNOME box is the pixel verdict. Gates green. No version change per standing owner order.

### Bottom-ring verdict: keep as-is (2026-10-10, Aina measurements → fallback taken, no code churn)

Owner: bottom transition reads different from the other three sides (Ciel 1:1 pixels: 4-5px ring + AA, no separate layer). Aina measured all four: inner outline pixel bottom ~(74,74,78) vs others ~(70,70,72) — delta 4-6 RGB, sub-perceptual, not the cause; no value, geometry, or clipping difference between sides. Cause is wallpaper contrast, 100%: outer border pixel (black 0.38 over wallpaper) over dark ground gives ~3.4:1 crisp step, over bright orange gives ring-to-ring ~1.1:1 — the defining step is erased by the wallpaper, not by any value. Compensation math recorded and rejected (0.38→0.55 lifts outer-vs-orange only ~1.6:1→2.2:1, still short of 3.4; needs per-side literals, violates tokens-first, misfires on any other wallpaper). Risks: bottom-only compensation would read as a dark smudge over dark wallpaper — as-is is the only position-independent state. Owner close-out test: drag window over dark wallpaper, bottom must read identical (predicts PASS, confirms cause). No code changed; check sanity only.

### Settings toggles right + shipped defaults 5-across / 46px rows (2026-10-12, owner + Aina verdict → uncommitted, no bump)

TOGGLES: Toggle.svelte only, no new files, no token changes. Visual order caption-left / track-right via `order` scoped to `.toggle:not(.small)` (caption order:0, track order:1); DOM order input/track/caption untouched for AT/focus; display:flex/gap:8px/min-height token unchanged; thumb travel/insets/tiers unchanged; no margin/padding/gap/min-height change (pitch still --sidebar-row-size). EQ popover header (.small) keeps control-left by construction. States unchanged (OFF wash+dim thumb, ON accent+accent-text full travel, focus-visible inset ring, RM none, role=switch 1px input, whole-row label target).

DEFAULTS: ui.svelte.ts DEFAULT_TILE_SIZE 180→147 (= the 5-column stop of tileStops at the 980 reference width — locked by the existing buildRows test stops [120,123,147,181,231,314]/columns [7,6,5,4,3,2]; rendered tile at 980 ≈180px, so tiles look the same size as before there), DEFAULT_SIDEBAR_ROWS 36→46; resetAppearance invariant kept (restores exactly them). app.css --sidebar-row-size 36px→46px, --tile-size first-paint initial 180px→147px (App.svelte overwrites both from the store at boot). Both stay user-editable (tile detent slider, 28–52 step-2 row slider); stored prefs untouched — existing profiles keep their values until Reset appearance. DESIGN.md: Switch-section override pasted verbatim with old control-left line struck through; row numbers updated with dates (overview default 180→147, artist-row default 36→46, menu-row token-dump 40→46 with note that "fixed 40px" predates the 2026-08-30 one-row language, row-of-options 40→46 with old arithmetic retired to a dated parenthetical). Historical 40px mentions (checkbox-row history, 50-vs-40 pitch history, shadow/frame literals) left intact.

5-ACROSS ANALYSIS: with t=147/gap 20, floor((w+20)/167)=5 holds for grid widths 815–981px. Owner's 1200×660 window (grid ≈924) renders 5 across; the 1280 default window (grid ≈1004) renders 6 — no single px holds 5 across at every common width (the band is one 167px stop wide), 147 centers the shipped experience on the reference width and the owner's window. No store-model change, per brief.

VERIFICATION: songstress-dev was inactive and the owner's prod songstress+mpv (13:45 UTC) was left alone — NO live-DOM probe, no restart. Proof instead: svelte-check 0 errors; vitest 140/140; cargo test --lib 147/147; npm run build succeeds; built CSS carries `.toggle…:not(.small) .caption{order:0}` / `.track{order:1}`, `--sidebar-row-size:46px`, `--tile-size:147px`; DOM order verified in source (Svelte emits template order as-written). Owner eyeball is the pixel verdict. No version change per 2026-10-09 order; uncommitted, unpushed.

### Correction 2026-10-12 (same day, owner eyeball): DEFAULT_TILE_SIZE 147→180 (ui.svelte.ts + app.css first-paint reverted; DESIGN.md overview back to 180 with round-trip note — no single px reads 5 across on every window). 46px rows and toggle override stand. No version change per 2026-10-09 order; uncommitted, unpushed.

### Save-to-library lag, killed in three cuts (2026-10-13, owner symptom → Mare systematic-debugging, no bump)

SYMPTOM: single-song save stalled the library update. MEASURED: save ran a full-roots rescan + 937KB full dump inside one awaited window, then the watcher echoed our own move into a second full cycle (~1.1s total). Falsified: hashing (zero bytes without collision), artwork (~0 rows), peaks (never on save path), staging prune (tiny).

CUT 1 echo suppression (watcher.rs): RECENT_SAVES map, 10s window, folder-granular, registered in a read-only pre-pass BEFORE the first write (post-hoc loses the race: the 2s debounce delays the rescan, never the evaluation). Echo absent over 20s, ~600ms second cycle deleted, external-change control proven twice. Planning errors now abort before any move (safer). Cross-instance echo remains (separate processes, out of scope).

CUT 2 targeted refresh (import.rs/lib.rs/savePatch.ts/library.svelte.ts): save reports moves, scan-finished carries optional savePatch, frontend applies in place (rename staged-to-false, removals, prune, reindex, dumpVersion++); non-save payloads byte-identical, unknowns fall back to full load. UI refresh +450ms → +0ms, DOM churn 7 mutations, merge control validated. Chose patch over coalescing (echo dead, bursts gone, the dump itself had to go). saveRenames map added on the patch (re-anchor follows).

CUT 3 scoped rescan (lib.rs save tail): follow-up scan calls scan_inner directly with destination parents as roots + landed set as only — 58.9ms → 13.5ms, artwork/sidecar unchanged. Missing-sweep skipped out-of-scope by construction (same as every only-scan).

VERIFICATION: cargo 148/148, svelte-check 0, vitest 145/145 (5 savePatch tests), build green, both DBs 4610/252/0 zero fixture traces, /tmp clean, prod untouched (dev restarted repeatedly — rebuild churn, window needs owner re-place). No version change per standing owner order. Committed as the save-lag batch.

### Tab-follow after tag-driven move + rename-aware re-anchor (2026-10-13, owner behavior → Mare, no bump)

BEHAVIOR: files moved by tags into a different existing album left the view on the emptied source artist tab; the view now follows the files to the destination album tab. Frontend-only: editors arm library.pendingFollow pre-invoke, scan-finished consumes it after load via pure resolveRetagFollow (retagFollow.ts, first track wins, same-album/unknown/missing stays put, shared ui.revealAlbum). Probed: cross-album lands on new tab + panel, genre-only no yank, failed save untouched. Re-anchor taken: saveRenames consulted on miss (kills the Nothing-playing ghost on saves); discard removed-only patch skipped for later. Tests: vitest 150/150 (5 retagFollow tests), full gates green. Committed as the navigation batch with the re-anchor. No version change per standing owner order.

### Picker ingress canonicalized (2026-10-14, Erde v1.1.0 trial finding → Mare constraints-trial proposal, no bump)

FINDING: Erde's lens trial re-audited picker-only roots (9a86b1b) and found the picker return was never canonicalized before validate/persist/scan/watch — a symlink dir passed is_dir and got scanned + inotify-watched (walkdir follow_root_links defaults true), and ..-spellings bypassed the lexical nesting check. The image flow already canonicalized; the new sole ingress did not.
FIX: resolve_new_root at the add_music_folder ingress (one spot, fallback to raw path so is_dir still reports missing), killing both the symlink-follow and the nesting-bypass. Regression test proves raw alias passes validation while resolved is rejected as duplicate. KNOWN GAP (owner decision): pre-fix stored roots are not migrated; later symlink swaps under a stored real path stay out of scope (same TOCTOU posture as read_image).
VERIFICATION: cargo check clean, cargo test --lib 149/149, proposal built in a disposable worktree under trial CONSTRAINTS.md (all Floor held) and applied to main after owner approval. No version change per standing owner order.

### Swallowed-event silence fixed (2026-10-14, Erde trial B1+B2 → full spec chain, no bump)

FINDING: Erde's v1.1.0 trial on the save-lag batch found the RECENT_SAVES pre-pass silenced predicted folders for files that never landed (duplicates, vanished) and left silence armed when saves errored — real sibling changes swallowed up to 10s with no error (both ALIVE through a blind verifier).
FIX (full gated chain SPEC→PLAN→TASKS→IMPLEMENT, every gate owner-approved): B1 plan-then-execute restructure — decide all outcomes read-only first, suppress only folders with a real Place dest, no write ahead of suppress_paths; B2 exact-set clearing primitive with suppress-then-closure on post-registration error paths. Six probes (AC1-AC4 incl. red-green-pinned chmod fault), folder-cover rule and timing constants untouched.
VERIFICATION: cargo check clean, cargo test --lib 154/154 (6 new), vitest 150/150 + svelte-check 0 (run in main checkout, zero frontend files differ), proposal built in a disposable worktree, applied to main after final owner gate. No version change per standing owner order. Accepted: concurrent same-folder saves fail open into one extra scan; pre-fix behavior needs no migration (suppression is runtime-only).

### Stored-roots migration (2026-10-15, owner decision, no bump)

CLOSE-OUT of the canonicalize gap: startup migration reads stored roots, canonicalizes each with ingress semantics, persists only changed rows, keeps unresolvable roots raw and reported, idempotent reruns. Legacy musicDir key covered too. Deliberately no dedup of two aliases sharing one target (dropping rows contradicts the never-drop rule; ingress validation blocks new ones). Out-of-band callers holding stale alias strings get "not a music folder" — frontend reads canonical via get_music_folders, stays consistent.
VERIFICATION: cargo check clean, cargo test --lib 158/158 (1 new migration test), proposal in a disposable worktree under the standing CONSTRAINTS.md, applied to main after owner gate. No version change per standing owner order.

### GLOSSARY.md + contradiction fixes (2026-10-15, domain-modeling trial, no bump)

TRIAL: drafted the repo's first GLOSSARY.md (30 terms, terms-only) in a disposable worktree; the survey surfaced 4 comment-vs-code contradictions, fixed first per owner gate.
FIXES: album-id comment corrected to tag-derived (scan.rs stable_id proves it; pure folder moves never strand entries), staged comment qualified for unhealed copy-era rows, copy/rename wording split (filesystem move vs row-identity rename), duplicate entry names both already-checks (size-only locate flow vs name+size+hash save flow).
VERIFICATION: cargo check clean; glossary committed as GLOSSARY.md at repo root. No version change per standing owner order.

### Root-alias row reconciliation (2026-10-22, owner option A, no bump)

GAP left by the stored-roots migration: it canonicalized `musicDirs` but track rows are keyed on the path the scan wrote (`id = stable_id("tr", path)`, `UNIQUE(path)`), so pre-fix rows kept the alias spelling and `delete_tracks_under_root(canonical)` matched none of them — removing a root purged nothing. The owner's DB was already migrated by the last startup, so a fix that only fires on a fresh migration would not heal it.
FIX: `settings::reconcile_root_paths` resolves each row's path once (one `canonicalize` per row), rewrites alias rows to the canonical path with its re-derived id, drops an alias row only when the canonical path is already a row (survivor keeps `staged` if the alias row had it), and never invents a path for a missing file. Runs from `migrate_music_roots` after the settings write — same startup step, and identical on every later startup (idempotent by construction, no flag). One `unchecked_transaction`; a failure leaves no half-reconciled library. This supersedes the 2026-10-15 "deliberately no dedup" note above: dropping the alias row when the canonical row exists deletes a duplicate of the same file, not a user's track.
VERIFICATION: cargo test --lib 164/164 (6 new: the previously ignored red test turned green + collision/staged/outside-roots/idempotent/unresolvable), vitest 150/150, svelte-check 0, prod build clean. Cost measured on a synthetic 5000-row temp DB (debug): 37 ms no-op startup, 125 ms when all 5000 rows rewrite. No version change per standing owner order.
