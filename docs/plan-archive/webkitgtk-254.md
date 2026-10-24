# WebKitGTK 2.54: the artwork outage, the stutter, and the apple-design audit (2026-09-26) (plan archive)

Moved out of `PLAN.md` on 2026-10-24 so the plan file stays readable in one pass.
This is the record, not the state: current state lives in `PLAN.md`, and this
file is the evidence behind it. Sections are kept verbatim, in their original
order. The index of every section is `README.md` next to this file.

## WebKitGTK 2.54: the artwork outage, the stutter, and the apple-design audit (2026-09-26)

Two unrelated regressions shipped in the same Fedora update (`webkit2gtk41/-6.0
2.52.x → 2.54.0-2`, **installed Fri 2026-09-25 12:11** — the RPM instance that
"behaved normally" had been running since Sep 24 and therefore on 2.52; every
Songstress process started after the update sees both bugs).

**1. Custom-scheme subresources are blocked from web origins.** A page loaded
from `http://` (the vite dev server) can no longer load `thumb://…` into an
`<img>`: the registered `register_uri_scheme_protocol` handler is NEVER called
(instrumented with eprintln — zero hits; fetch() from the page dies with
"Load failed"). Top-level navigation to a custom scheme still fires the
handler, and from a custom-scheme page (production's `tauri://localhost`)
cross-scheme subresources still work — so **the RPM's artwork never broke**;
only dev did. Proven with a standalone WebKit2GTK-4.1 Python probe
(`/tmp/thumb_probe*.py`, disposable): base `http://localhost:1420/` → 0 hits
for any flag combo of `secure/display_isolated/local/cors_enabled`; base
`otherprobe://self/page` → handler fires. This is the request-path/security
layer, not the compositor. **Fix:** `src/lib/artSrc.ts` rewrites
`thumb://… → /thumb-http/…` in DEV only, served by the vite middleware
`vite.thumb-dev.js` from the same cache dir with the same size-fallback rules
as the Rust handler (hashed `512-<h>.webp` parsed, `art-` strict, album chain
512→256→96). Wired at all 7 `<img>`/warm-Image sites (AlbumGrid, ExpandedPanel,
PlayBar, ArtSelector). Permanent insurance: legal on 2.52 AND 2.54.

**2. The new Skia compositor stutters on this GPU/desktop** (radeonsi Phoenix1
780M, GNOME/Mutter, Wayland). Measured with `WEBKIT_DRAW_FPS=1`: 60 at rest,
**3–47 during tooltip hover, typing, grid populate** — in the dev instance AND
in the freshly-restarted RPM (owner confirmed the release app regressed too,
and is 100% sure it wasn't happening before the update). The main thread is
clean (rAF sampler during hover: 520/520 frames, 0 gaps >25ms) — the drops are
compositor/presentation-side. Every env lever into the old paths MISPAINTS
worse: `WEBKIT_DISABLE_DMABUF_RENDERER=1` and `WEBKIT_DISABLE_COMPOSITING_MODE=1`
paint only the damage rect around the pointer (black everywhere else);
`WEBKIT_USE_SKIA_FOR_COMPOSITION=0` reproduces the band-clip black bands;
`WEBKIT_DISABLE_DMABUF_ATLAS=1` doesn't help. The 2.54 release notes describe
exactly this subsystem (Skia compositor replacing TextureMapper, damage-aware
compositing on by default, `DamageRectangleThreshold` preference that is NOT
exposed as a WebKitSettings property in the 4.1/6.0 typelibs — nothing to tune
from our side). **Decision (owner, 2026-09-26): do NOT downgrade** (2.52.5 is
off the mirrors; 2.52.1 would work but pins security fixes behind it). Ride
2.54, keep the dev instance's `WEBKIT_DRAW_FPS` drop-in OUT (removed), and
revisit on 2.54.1. Upstream report is welcome-but-not-yet-filed; the minimal
repro is any 250-image grid + hover tooltips on this box.

**Update, later the same day (2026-09-26) — the downgrade was run, and it fixes
it.** The owner ran `sudo dnf downgrade webkit2gtk4.1 webkit2gtk4.1-devel
webkitgtk6.0` → **2.52.1-1.fc44** (that build is in the base `fedora` repo;
2.52.5 is gone from the mirrors). A cold-started dev instance on 2.52.1 (a fresh
process, not a fossil of the 2.54 session) showed **no stutter at all** on the
exact surfaces that had measured 3–47 FPS — owner-verified, so the regression is
attributed to 2.54's compositor by A/B on this one box. Decision (owner, same
day): **stay on 2.52.1 and check for updates at a later date** — this replaces
the "ride 2.54" call above. Cost, stated honestly: 2.52.1 keeps the 2.54
security fixes off **system-wide**, not just for us — epiphany-runtime (GNOME
Web), yelp, gnome-initial-setup, evolution-data-server and
NetworkManager-openconnect-gnome all moved back with it
(`rpm -q --whatrequires 'libwebkitgtk-6.0.so.4()(64bit)'` and
`'libwebkit2gtk-4.1.so.0()(64bit)'`; gnome-shell only *declares* the package, it
does not link the library). Nothing in the RPM changed — this is system state,
not a code change, so **no version bump**. The pin is manual: a plain
`dnf upgrade` puts 2.54.0 back, and the stutter with it. Revert deliberately
with `sudo dnf upgrade webkit2gtk4.1 webkit2gtk4.1-devel webkitgtk6.0`.

**Instrument trap (cost one blind measurement).** The FPS counter env var was
**renamed**: 2.52.1 reads `WEBKIT_SHOW_FPS`, 2.54 reads `WEBKIT_DRAW_FPS`.
Setting the 2.54 name on 2.52 printed **zero lines for every input** — the
counter was blind, not the compositor quiet. Positive control:
`strings /usr/lib64/libwebkit2gtk-4.1.so.0 | grep -i fps` names the variable the
installed library actually reads (run it before trusting an empty trace).

Diagnostics worth keeping: `rpm -q --last` is the smoking-gun command (the
"nothing changed but it broke" answer usually changed in the system, not the
app); a long-running GUI process is a fossil of the library set at its start
time; and two visually-identical instances (RPM + dev) were both in play
mid-debug — check `pgrep -af songstress` before trusting which window a
screenshot shows.

**Same session, before the regression hunt:** apple-design audit of the UI
(record in the session log): interruptibility/spatial-consistency/motion
discipline graded strong; two fixes shipped — (a) `prefers-reduced-transparency`
+ `prefers-contrast` blocks in app.css reusing the GNOME solid tokens, (b)
`:active` press washes on playbar transport/mode/volume buttons (house
`--active`, no scale). DESIGN.md body token corrected 14→16px (code was
canonical, docs drifted). `docs/performance-cheatsheet.md` saved from
emilkowalski/skills (7-row table, dev reference).

### GNOME matte fields: grain as the fallback material (2026-09-26, owner ask, shipped **0.13.3**, no RPM yet)

Owner's brief: the GTK/GNOME fallback pins every alpha to 1, and glassmorphism
died with it — "add texture to the UI, not just revamp it". Scope set by him:
the three BACKGROUND fields (grid backdrop, sidebar, playbar), tiles/artwork/
text/glyphs stay untouched on top; grain + the low-frequency unevenness; start
at the suggested strength and dial from there.

**What shipped** (`src/app.css`): one `--tex` background layer per field
(`html[data-de="gnome"] .app .stage::before`, `.sidebar`, `.playbar`), plus
`background-blend-mode` per theme (`--tex-blend`: soft-light dark, multiply
light). The tile is a 320px SVG holding TWO noise scales — unevenness
(`baseFrequency .012`, `stitchTiles='stitch'`) under fine grain (`.9`) at half
weight. GNOME-only; KDE is byte-identical, and its 2% `.glass::after` banding
garnish is switched off on GNOME (no blur → no banding, and it sat over text).

**Four things measured, three of them wrong first** (all now in the code comment
and DESIGN.md → The Matte Rule):

1. `.glass::after` could NOT be reused. It is a positioned last child, so it
   paints above `.glass > *` — raising its opacity textures the text. Grain had
   to be a background layer on the field itself. The grid field also can't take
   an overlay pseudo at all: `.stage::before` is the field color, and a
   `.stage::after` at z-index auto paints above AlbumGrid (later sibling).
2. **SVG filters default to linearRGB.** A 0.5-pivot noise there is sRGB 0.76 —
   the whole field came out ~10 RGB units lighter (measured: mean 0.762 before
   the fix, 0.500 after adding `color-interpolation-filters='sRGB'`).
3. The intercept must be `0.5 − slope/2`: Skia's fractalNoise is centred on raw
   0.5, so `values='s 0 0 0 .5 …'` yields mean 0.5+s/2 (measured 0.56 at s=.12).
4. **Amplitude had to be 3× the first guess.** On a near-black field, soft-light's
   modulation is `2·Δb·base·(1−base)` = 0.0786 per unit of blend deviation, so
   slopes .08/.12 (±8.5/255 in blend space) bought only ±1.3 RGB — invisible.
   First cut was slopes .5/.9 on a 320px tile with `.012` unevenness: ±4.1 RGB,
   which the owner looked at and rejected — "renders like a grid of individual
   grain pictures, should be uniform; lower the alpha". At `.012` a blotch is
   ~80px across and the eye locks onto the 320px repeat, with the strongest blobs
   echoing tile to tile even though `stitchTiles` keeps the edges continuous.

   **v2:** unevenness `.025` (~40px blobs), tile 480px (a pane shows 2–3 repeats
   at most), amplitude ~44% of v1 (slopes .22/.40) → dark ±1.8 RGB at 5–95%, mean
   unchanged, extremes ±4.5. Lesson: the "tiled texture" artifact is four dials at
   once — frequency, tile size, amplitude AND the grain:unevenness ratio — not
   "just less grain".

   **v3 (owner: "remove the unevenness, it reads dirty instead of aesthetic"):**
   grain only. One fine-grain layer (`baseFrequency .9`, 480px tile, no
   `stitchTiles` — nothing low-frequency left to stitch), slope .32 in dark /
   .09 in light (the slope is now the whole signal: the unevenness had been
   carrying half the level). Measured: dark mean −0.02 RGB (unchanged), ±2.5 RGB
   at 5–95%, extremes ±5.3; light mean −2.8 RGB, tooth 0…+6.5 RGB. Two noise
   scales were tried and are now documented as rejected in both app.css and
   DESIGN.md — large-scale mottling under a clean grid reads as dirt, and the
   first coarse version read as visible tile repeats.

   **v5 (owner: "add it to the modal background throughout the app"):** one more
   selector, `html[data-de="gnome"] .scrim > .glass` — About, Music folders, Tag
   editor, Imported music are all `.glass` panels that are the direct child of
   their scrim, and each declares `background: var(--panel-bg-strong)` with no
   image layer of its own, so the rule at (0,3,1) beats the component shorthand at
   (0,1,0) and no component file changed. Verified with a probe node declaring the
   same shorthand: `background-image: url(data:…)` + `soft-light` + the opaque
   strong color. NOT covered on purpose (separate ask): the popover tier (context
   menu, EQ, queue, art picker) and the expanded album panel.

   **v6 (owner: "yeah, popovers as well. let's try it on the expanded view too"):**
   the rule collapsed to `html[data-de="gnome"] .glass` — `.glass` IS the
   chrome/popover tier (EQ + queue popovers, context menu, the four scrim
   panels), so the explicit per-surface list was redundant. The expanded panel
   needed its own line of work: it sets `background` INLINE (album gradient +
   `--panel-bg` base, since 2026-09-05) and an inline shorthand beats any
   stylesheet, so the grain was added as the top layer of that stack in
   ExpandedPanel.svelte — `var(--tex) padding-box, <gradient> padding-box
   padding-box, var(--panel-bg) border-box` with `background-blend-mode:
   var(--tex-blend), normal, normal`, so the album gradient keeps rendering
   unblended beneath it. `--tex`/`--tex-blend` now default to `none`/`normal` at
   `:root`, which is what makes the component-side use safe on KDE (the shorthand
   stays valid, the layer paints nothing). Verified live: panel computes
   `background-image: url(data:…), linear-gradient(…), none` with blend
   `soft-light, normal`, base opaque `rgb(32,32,40)`; a `.glass` probe node gets
   the same texture. Not covered, on purpose: `.app-tip`/`.vol-tip` (a 20px grain
   stencil on a 24px pill reads as noise, not material).

   **v4 (owner: "reduce the alpha more"):** the same mechanism at half the slope —
   .16 dark / .045 light, and the light mean target raised to 0.995 so its drift
   halves too. Measured: dark mean −0.0 RGB, ±1.25 RGB at 5–95%, extremes ±2.6;
   light mean −1.3 RGB, tooth 0…+3.1 RGB. No structure changed; the level is a
   setting the owner dials by eye, so the recorded number is the current
   position, not a target to defend.

The dial is the two `values` slopes in the tile SVG; nothing else needs to change.

Gates: `npm run check` 0/0, `npm test` 99 passed, `cargo test --lib` 116 passed
(untouched, run for completeness), `npm run build` ok. CSS-only → hot reload, no
restart, no window move. Owner to judge the strength on screen (dev unit is live
on GNOME; DOM probes confirmed the three layers, the opaque colors, and the
garnish at opacity 0).

**Shipped as 0.13.3** (commit e842f8c, pushed to GitHub; no RPM on purpose —
the owner wants to wait until the WebKitGTK 2.54 compositor stutter is fixed
upstream before building one). Version bumped in all three files together
(`package.json`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`).

Also corrected here: AGENTS.md's 2026-09-23 "no `songstress-dev` unit on GNOME"
note is stale — the unit is loaded and running, and `tools/devctl.mjs` bridges
into the page on GNOME. `spectacle`, `qdbus-qt6` and `gnome-screenshot` are all
absent on this box (measured), vs the previously documented gnome-screenshot path.

