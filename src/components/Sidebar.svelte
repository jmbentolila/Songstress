<script lang="ts">
  import { ui, resolvedTheme, openAbout, resetAppearance } from "../lib/stores/ui.svelte";
  import { marquee } from "../lib/marquee";
  import { library, LIVE_LIBRARY } from "../lib/stores/library.svelte";
  import { openContextMenu } from "../lib/stores/contextMenu.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { ACCENT_PRESETS, accentVariants, hexToHsl } from "../lib/accent";
  import { menu, activateMenuItem } from "../lib/stores/menu.svelte";
  import { scanner } from "../lib/stores/scanner.svelte";
  import { tooltip } from "../lib/tooltip";
  import { openImportManager } from "../lib/stores/imports.svelte";
  import { surfaceOpen } from "../lib/stores/surfaces.svelte";
  import { fold } from "../lib/search";
  import { libraryLoading, sidebarRows, NAME_W, NOTE_GRACE_MS } from "../lib/loadingState";
  import {
    windowClose,
    windowMinimize,
    windowToggleMaximize,
  } from "../lib/window";
  import { decoState, decoVars, loadDecoration } from "../lib/stores/decoration.svelte";
  import Toggle from "./Toggle.svelte";
  import ProgressRing from "./ProgressRing.svelte";
  import {
    playback,
    applyEqPreset,
    setEqPreamp,
    setEqBand,
    setEqEnabled,
    setShuffleStage,
    setShuffleOn,
    setRepeatStage,
    setRepeatOn,
    resetPlayback,
  } from "../lib/stores/playback.svelte";
  import {
    EQ_BANDS,
    EQ_PRESETS,
    EQ_MAX_DB,
    fmtDb,
    fmtHz,
  } from "../lib/eq";

  loadDecoration();

  // --- menu stack (Step 8b, iOS Settings-style) ----------------------------
  // Three layers in .stack, sliding on transform only (compositor):
  //   home (search + artists) → root ("Settings") → detail (pane).
  // The pushed root recedes -30% (iOS parallax) instead of leaving.
  // Real panes are the Rust-owned model (menu.svelte); "appearance" is
  // frontend-only (the old gear-popover settings).
  const APPEARANCE = "appearance";
  const PLAYBACK = "playback";
  const LIBRARY = "library";

  // Stage labels for the Playback pane's reveals. "off" is deliberately NOT a
  // segment: the group's checkbox owns Off, so the segmented control only ever
  // shows the modes you can actually be in (2-up repeat, 3-up shuffle) and the
  // two controls can never disagree about the state.
  const REPEAT_MODES: { id: "album" | "track"; label: string }[] = [
    { id: "album", label: "Album" },
    { id: "track", label: "Track" },
  ];
  const SHUFFLE_MODES: { id: "album" | "artist" | "all"; label: string }[] = [
    { id: "album", label: "Album" },
    { id: "artist", label: "Artist" },
    { id: "all", label: "All" },
  ];
  const THEME_IDS = ["system", "light", "dark"];
  const REPEAT_IDS = REPEAT_MODES.map((m) => m.id);
  const SHUFFLE_IDS = SHUFFLE_MODES.map((m) => m.id);

  let inMenu = $derived(ui.menuOpen);
  let atDetail = $derived(ui.menuDetail !== null);
  // 4th layer (quieter pass): the accent picker is a once-a-year decision,
  // so it no longer shares the Appearance pane's first screen — this row
  // pushes it one level deeper, same drawer language.
  let atSub = $derived(ui.menuSub !== null);

  // A layer keeps showing ITS OWN content while it animates out. Both ids flip to
  // null at t=0 of a back, and a layer re-rendered for the null state is either a
  // blank panel sweeping across (detail) or, worse, the WRONG pane: the sub layer's
  // `{:else}` branch is the accent picker, so backing out of the EQ preset list
  // printed colour swatches over the drawer on the way out (found by the owner,
  // 2026-09-01). The held id is only ever read by a layer that is visible or
  // leaving, so it never needs clearing — and every gate that decides what the
  // user can ACT on (inert, aria-hidden, class:active, `atDetail`/`atSub`) still
  // comes from the real state, not from this.
  let heldDetail = $state("");
  let heldSub = $state("");
  $effect(() => {
    if (ui.menuDetail) heldDetail = ui.menuDetail;
    if (ui.menuSub) heldSub = ui.menuSub;
  });
  const shownDetail = $derived(ui.menuDetail ?? heldDetail);
  const shownSub = $derived(ui.menuSub ?? heldSub);

  let detailMenu = $derived(menu.menus.find((m) => m.id === shownDetail) ?? null);

  // --- Sidebar-specific tree (impeccable critique 2026-08-30, option C) --
  // The Rust model (menu.rs) feeds the Plasma Global Menu — a menu-BAR
  // shape. The sidebar does not mirror it:
  //   • View's one item (theme) already lives in Appearance → pane dropped
  //   • Help's one item (About) was a no-op; it is now the root's footer
  //     row opening a real in-glass dialog
  //   • Playback loses its 6 transport rows — the PlayBar owns those
  const PLAYBACK_TRANSPORT = new Set([
    "playback.play-pause",
    "playback.stop",
    "playback.previous",
    "playback.next",
    "playback.album-prev",
    "playback.album-next",
  ]);

  let rootPanes = $derived([
    { id: APPEARANCE, label: "Appearance" },
    // "appearance" now EXISTS in the Rust model (renamed from "view" in the
    // 2026-09-03 Global Menu pass) — it feeds the panel, not this root; the
    // bespoke pane above is the sidebar's Appearance.
    ...menu.menus
      .filter((m) => m.id !== "appearance" && m.id !== "view" && m.id !== "help")
      .map((m) => ({ id: m.id, label: m.label })),
  ]);

  let detailItems = $derived.by(() => {
    if (
      shownDetail === APPEARANCE ||
      shownDetail === PLAYBACK ||
      shownDetail === LIBRARY
    )
      return [];
    return (detailMenu?.items ?? []).filter(
      (i) =>
        !PLAYBACK_TRANSPORT.has(i.id) &&
        // Section breaks belong to the panel's menu-bar shape; a drawer
        // renders its groups as spaced sections, so separators don't pass.
        !i.separator &&
        // "Manage imported music" exists only to act on staged imports —
        // in the sidebar it hides instead of standing as a dead row.
        !(i.id === "library.manage-imports" && !i.enabled),
    );
  });
  let detailTitle = $derived(
    shownDetail === APPEARANCE ? "Appearance" : (detailMenu?.label ?? "Settings"),
  );

  // Focus handoff (audit pass B): pushing a layer used to leave focus on the
  // row that triggered it — inside a layer that is now aria-hidden/inert, so
  // the ring vanished and Tab walked the covered rows before reaching one
  // visible control. Every layer change moves focus to the incoming layer's
  // back affordance (or its first row), which is where the a11y tree now starts.
  function focusFirst(sel: string) {
    // preventScroll is load-bearing, not politeness: `.stack` is a scroll
    // port (overflow hidden still scrolls programmatically), and the incoming
    // layer sits at translateX(+100%) when we focus into it — its back button
    // is therefore OFF to the right, and focusing it SCROLLED the stack to
    // scrollLeft ~202. That scroll is permanent (nothing scrolls it back) and
    // shifts every layer left at once: titles printed over each other,
    // chevrons clipped, the sub layer's swatches showing through the root
    // pane. The layer is animating into place anyway, so it needs no scroll.
    requestAnimationFrame(() =>
      document.querySelector<HTMLElement>(sel)?.focus({ preventScroll: true }),
    );
  }
  function openDetail(id: string) {
    ui.menuSub = null;
    ui.menuDetail = id;
    focusFirst(".layer.detail .backchev");
  }
  function openSub(id: string) {
    ui.menuSub = id;
    focusFirst(".layer.sub .backchev");
  }
  function back() {
    if (ui.menuSub) {
      ui.menuSub = null;
      focusFirst(".layer.detail .backchev");
    } else {
      ui.menuDetail = null;
      focusFirst(".layer.root .scroll .mrow");
    }
  }
  function closeSettings() {
    if (ui.menuSub) teleportDetail();
    ui.menuSub = null;
    ui.menuDetail = null;
    ui.menuOpen = false;
    focusFirst(".gear");
  }
  // A receded detail layer sits fully off-screen LEFT; on a full pop it would
  // otherwise sweep the whole sidebar width on its way back to its +100%
  // entry slot (it paints over the root layer). Same one-frame transition
  // suppression the root layer uses — both ends are off-screen, so the
  // teleport is invisible.
  let detailNoAnim = $state(false);
  function teleportDetail() {
    detailNoAnim = true;
    requestAnimationFrame(() =>
      requestAnimationFrame(() => (detailNoAnim = false)),
    );
  }
  // Full-pop close choreography: root exits LEFT, parallel to home
  // arriving at the same speed (no crossing, no 2× sweep, no centered
  // main-menu intermediate). While off-screen at -200% it then teleports
  // back to its +100% entry slot (transition suppressed one frame) so the
  // next open re-enters from the right.
  let rootNoAnim = $state(false);
  let closeTimer = 0;
  function toggleSettings() {
    if (!ui.menuOpen) {
      ui.menuOpen = true;
      ui.menuDetail = null;
      ui.menuSub = null;
      focusFirst(".layer.root .scroll .mrow");
    } else {
      if (ui.menuSub) teleportDetail();
      ui.menuSub = null;
      ui.menuOpen = false;
      if (ui.menuDetail) {
        clearTimeout(closeTimer);
        closeTimer = setTimeout(() => {
          if (ui.menuOpen) return; // re-opened mid-close: keep the state
          ui.menuDetail = null;
          rootNoAnim = true;
          requestAnimationFrame(() =>
            requestAnimationFrame(() => (rootNoAnim = false)),
          );
        }, 340);
      }
    }
  }

  // The native color input needs a concrete value; presets leave it alone.
  let customHex = $state(ui.accentColor ?? "#ff6ec7");

  // --- pane footers ----------------------------------------------------------
  // Each detail layer gets one dim footer row: an action where a real reset
  // exists, a status line where it doesn't. The resets themselves moved to
  // the stores (2026-09-03) so the Global Menu's footer rows call the SAME
  // code — the two surfaces can't drift on what "reset" means.
  let libStats = $derived(
    `${library.albums.length.toLocaleString()} albums · ${library.trackCount.toLocaleString()} tracks`,
  );
  // "never scanned" would be a lie on a library built before this readout
  // existed — the time clause only appears once a scan happened in anger. The
  // live scan progress is NOT here: it belongs next to the rows it disables
  // (see scanLine), and the footer stays the truth about what the library IS.
  let scanTime = $derived(
    ui.lastScan === null
      ? ""
      : new Date(ui.lastScan).toLocaleTimeString([], {
          hour: "2-digit",
          minute: "2-digit",
        }),
  );

  // --- Library pane (store-driven, like Appearance and Playback) ---------
  // The generic row list could not say the two things that matter here: that
  // N albums are sitting staged, and where a running scan has got to. Actions
  // still go through activateMenuItem, so a click here and a click in the
  // Global Menu are the same door into the same Rust command.
  let stagedAlbums = $derived(
    library.albums.reduce((n, a) => n + (a.staged ? 1 : 0), 0),
  );
  let stagedLabel = $derived(
    `${stagedAlbums} ${stagedAlbums === 1 ? "album" : "albums"}`,
  );
  let folderLabel = $derived(
    `${ui.musicFolders.length} ${ui.musicFolders.length === 1 ? "folder" : "folders"}`,
  );
  const PHASE_LABELS: Record<string, string> = {
    scan: "Scanning",
    artwork: "Reading artwork",
    import: "Importing",
  };
  // Operations that emit no progress events (discard, a folder added or removed)
  // still owe assistive tech a sentence, and "Scanning…" would misreport what
  // the disabled rows are waiting for.
  const KIND_LABELS: Record<string, string> = {
    scan: "Scanning",
    full: "Re-reading all files",
    import: "Indexing",
    save: "Saving imported music",
    discard: "Discarding imported music",
    folder: "Updating music folders",
  };
  let scanWords = $derived(
    PHASE_LABELS[scanner.phase] ?? KIND_LABELS[scanner.kind] ?? "Working",
  );
  let scanLine = $derived(
    !scanner.running
      ? ""
      : scanner.total > 0
        ? `${scanWords} ${scanner.done.toLocaleString()} of ${scanner.total.toLocaleString()}`
        : `${scanWords}\u2026`,
  );
  // Determinate: the ring fills, it does not spin. No events yet (total 0) is an
  // empty ring, not a fake halfway — "started, no news" is the honest state.
  // Finished runs report 1: the hold frame exists precisely to show a complete
  // arc, so an operation that ended at its last coarse event still lands full.
  let scanProgress = $derived(
    scanner.running ? (scanner.total > 0 ? scanner.done / scanner.total : 0) : 1,
  );
  let ringKind = $derived(scanner.running ? scanner.kind : scanner.heldKind);
  // The ring goes on the row whose own verb is running, so a disabled pane is
  // never disabled without saying which row is the reason.
  const ringOn = (kind: string) => ringKind === kind;

  /** The artist list's placeholder — the SAME rule the album grid applies, so the
   * two surfaces cannot disagree about whether a scan is happening. The facts are
   * spelled out at each call site on purpose: the type makes both of them answer
   * for every fact rather than let one surface quietly stop checking one. */
  const loading = $derived(
    library.live &&
      libraryLoading({
        ready: library.ready,
        albums: library.albums.length,
        running: scanner.running,
        scanning: library.scanning,
        devLoading: library.devLoading,
      }),
  );
  let navHeight = $state(0);
  const skRows = $derived(sidebarRows(navHeight, ui.sidebarRowSize));
  // The grid's pill-width pattern, shared so the two placeholders read as one
  // material (see loadingState: fixed array, never Math.random).
  const skName = (i: number) =>
    `${Math.round(NAME_W[i % NAME_W.length] * 100)}%`;

  /** "Updating library…" — the scan note for a library that already HAS content,
   * where there is no placeholder to show and a word is all that's left.
   *
   * It used to read `scanner.running` alone, which means "the frontend asked"; a
   * watcher-triggered rescan (Rust calls `run_library_scan` directly, no IPC) was
   * invisible here too — the same blind spot `libraryLoading` closes. The grace is
   * what makes that union safe: an incremental pass over unchanged files is ~56ms
   * end to end, so a line that mounts for one frame would be a flicker, not news. */
  let scanNote = $state(false);
  $effect(() => {
    const busy = library.albums.length > 0 && (scanner.running || library.scanning);
    if (!busy) {
      scanNote = false;
      return;
    }
    const t = setTimeout(() => (scanNote = true), NOTE_GRACE_MS);
    return () => clearTimeout(t);
  });

  // Theme modes, in the order the segmented control shows them. "system" is
  // the shipped default and was unreachable once touched: the old button
  // cycled dark|light, which resolves system to a fixed value.
  // System first: reading left to right, the leftmost segment reads as the
  // default/entry option, and following the OS is exactly the default for a
  // music app on a desktop with a day/night schedule. Light | Dark then run in
  // the same order the `theme` setting has always been listed in (user choice,
  // 2026-08-31). Index order is also the radiogroup's arrow/Home/End order.
  const THEMES: { id: "light" | "dark" | "system"; label: string }[] = [
    { id: "system", label: "System" },
    { id: "light", label: "Light" },
    { id: "dark", label: "Dark" },
  ];

  // Summary chip on the Accent row: the picker itself lives one level deeper.
  let accentName = $derived(
    ui.accentColor === null
      ? "Stock purple"
      : (ACCENT_PRESETS.find((p) => p.hex === ui.accentColor)?.name ??
        ui.accentColor),
  );

  // --- polish: say what you actually get ----------------------------------
  // accentVariants() clamps accent lightness per theme (accent.ts CLAMP), so
  // the Black preset renders LIGHT GREY in the dark theme — a black dot that
  // lies about its result. Swatches keep the raw color (Black and White must
  // stay tellable apart) and gain a ring in the color they actually produce,
  // but only where the two differ enough to matter.
  let themeNow = $derived(resolvedTheme());
  function clampColor(hex: string | null): string | undefined {
    if (!hex) return undefined;
    const derived = accentVariants(hex, themeNow).accent;
    const l0 = hexToHsl(hex)[2];
    const l1 = hexToHsl(derived)[2];
    return Math.abs(l1 - l0) > 0.12 ? derived : undefined;
  }
  function swatchRing(hex: string | null, selected: boolean): string | undefined {
    const c = clampColor(hex);
    // A selected dot already wears a 2px ring (its outline). Adding the clamp
    // ring outside it drew two near-identical concentric rings — read as a
    // rendering glitch, and it made the SELECTED dot look disabled. The
    // tooltip carries the promise while a clamped preset is selected.
    if (!c || selected) return undefined;
    return `0 0 0 2px ${c}`;
  }
  function clampHint(hex: string | null, name: string): string {
    const ring = clampColor(hex);
    if (!ring) return name;
    return `${name} — renders ${ring} in ${themeNow} theme`;
  }
  // The custom swatch shows the picked color (the rainbow is a permanent
  // placeholder for a choice already made); a conic corner keeps it readable
  // as the picker rather than as a twelfth preset.
  let hasCustom = $derived(
    ui.accentColor !== null && !ACCENT_PRESETS.some((p) => p.hex === ui.accentColor),
  );
  let chipColor = $derived(
    ui.accentColor === null
      ? undefined
      : accentVariants(ui.accentColor, themeNow).accent,
  );

  // Slider filled track: the native range paints one uniform track, so the
  // accent fill is a background gradient sized to the current value.
  function fill(v: number, min: number, max: number) {
    const pct = ((v - min) / (max - min)) * 100;
    return `linear-gradient(to right, var(--accent) ${pct}%, var(--hover) ${pct}%)`;
  }

  // Radiogroup keys: Left/Right (and Home/End) move within the group — the
  // ARIA contract for role="radio", which stackKeydown's Up/Down walk does
  // not cover. Selection follows focus (radios select on focus). Shared by
  // every segmented control in the stack (Theme, Repeat, Shuffle).
  function segKeys(
    ids: readonly string[],
    current: () => string,
    apply: (id: string) => void,
  ) {
    return (e: KeyboardEvent) => {
      const n = ids.length;
      const cur = ids.indexOf(current());
      let next: number | null = null;
      if (e.key === "ArrowRight" || e.key === "ArrowDown") next = (cur + 1) % n;
      else if (e.key === "ArrowLeft" || e.key === "ArrowUp") next = (cur + n - 1) % n;
      else if (e.key === "Home") next = 0;
      else if (e.key === "End") next = n - 1;
      if (next === null) return;
      e.preventDefault();
      e.stopPropagation();
      // Grab the buttons NOW: `currentTarget` is null once the dispatch is over,
      // and a deferred lookup silently leaves focus on the old segment (ring on
      // System, aria-checked on Light).
      const btns = (e.currentTarget as HTMLElement).querySelectorAll<HTMLButtonElement>(".segbtn");
      apply(ids[next]);
      btns[next]?.focus({ preventScroll: true });
    };
  }

  // Arrow keys walk the focused layer's rows (menu convention). Tab still
  // works; ranges keep their native arrow behavior (this only fires when a
  // BUTTON in a layer is focused). Home layer included: arrow-walk artists.
  function stackKeydown(e: KeyboardEvent) {
    if (e.key !== "ArrowDown" && e.key !== "ArrowUp") return;
    const layer = (document.activeElement as HTMLElement | null)?.closest(".layer");
    if (!layer) return;
    const btns = [...layer.querySelectorAll("button")].filter(
      (b) => !(b as HTMLButtonElement).disabled,
    ) as HTMLButtonElement[];
    const i = btns.indexOf(document.activeElement as HTMLButtonElement);
    if (i === -1) return;
    e.preventDefault();
    const next =
      e.key === "ArrowDown" ? Math.min(i + 1, btns.length - 1) : Math.max(i - 1, 0);
    btns[next].focus();
  }

  const albumCounts = $derived.by(() => {
    const m = new Map<string, number>();
    for (const a of library.albums) {
      m.set(a.artistId, (m.get(a.artistId) ?? 0) + 1);
    }
    return m;
  });

  // artistId → albums awaiting import. Drives the accent dot on the row's
  // count: the staged pile is a decision pending, and the artist list is
  // where you learn an artist EXISTS — the indicator belongs there, not
  // only on the album's own tile.
  const stagedCounts = $derived.by(() => {
    const m = new Map<string, number>();
    for (const a of library.albums) {
      if (a.staged) m.set(a.artistId, (m.get(a.artistId) ?? 0) + 1);
    }
    return m;
  });

  // One field drives both surfaces: the artist list here and the
  // Songs/Albums sections in the grid. Diacritic-insensitive via fold()
  // — same matching rule as the grid ("bjork" finds Björk in both).
  const filtered = $derived.by(() => {
    const q = ui.search.trim();
    if (q === "") return library.artists;
    const fq = fold(q);
    return library.artists.filter((a) => fold(a.name).includes(fq));
  });

  function select(id: string) {
    const switched = ui.activeArtistId !== id;
    ui.activeArtistId = id;
    ui.expandedAlbum.songs = null;
    ui.expandedAlbum.albums = null;
    ui.search = "";
    // Back to the top, every time — including re-clicking the artist you're
    // already on. Collapsing the panel shrinks the content under the
    // scroller, so a kept scrollTop strands the view mid-grid ("middle of
    // nowhere"). Instant, not smooth: the rows are being replaced anyway,
    // and a glide would chase a layout that's still collapsing. On a REAL
    // switch the grid owns the reset: it holds the old list through a 160 ms
    // exit, so resetting here would jump the fading rows — the reset lands
    // at the swap (AlbumGrid's switch bridge), when the new rows mount.
    // Re-clicking the same artist still pops straight to the top here.
    if (switched) return;
    const scroller = document.querySelector<HTMLElement>("main.content");
    if (scroller) scroller.scrollTop = 0;
  }

  // Right-click an artist → the same context menu as tiles/rows (shared
  // component ⇒ identical scroll/blur/focus dismissal). One item only: an
  // artist has no tags of its own to edit and no single thing to play. The
  // folder Rust derives is the DEEPEST one holding all of the artist's files
  // (lib.rs container_target), so a one-album artist lands on the album
  // folder and a scattered one on the shared parent. Fake-library dev mode
  // has no DB rows, so the menu only appears where it can act.
  function artistMenu(e: MouseEvent, artistId: string) {
    if (!LIVE_LIBRARY || !library.live) return;
    e.preventDefault();
    e.stopPropagation();
    openContextMenu(e.clientX, e.clientY, [
      {
        label: "Open containing folder",
        action: () =>
          void invoke("reveal_container", { artistId }).catch((err) =>
            console.error(err),
          ),
      },
    ]);
  }

  type Kind = "X" | "I" | "A";

  const ACTION: Record<Kind, () => void> = {
    X: windowClose,
    I: windowMinimize,
    A: windowToggleMaximize,
  };
  const LABEL: Record<Kind, string> = { X: "Close", I: "Minimize", A: "Maximize" };
  const CLASS: Record<Kind, string> = { X: "tb-close", I: "tb-min", A: "tb-max" };

  // Honor the user's KWin button order; letters we can't implement as a CSD
  // app (Shade, Keep Above, ...) are skipped rather than rendered dead.
  const layout = $derived(
    (decoState.value?.buttonsLeft ?? "XIA")
      .split("")
      .filter((c): c is Kind => c === "X" || c === "I" || c === "A"),
  );

  const chromeVars = $derived(
    decoVars()
      .map(([k, v]) => `${k}: ${v}`)
      .join("; "),
  );
  // Keyboard accelerators (critique P3): `/` focuses the library search,
  // `s` toggles the Settings stack — both inert while typing in any field.
  let searchInput = $state<HTMLInputElement | null>(null);

  function onGlobalKey(e: KeyboardEvent) {
    // One press, one verb. While a floating surface is up, THAT surface owns the
    // keyboard: this router does nothing at all — not Escape, not `s`, not `/`. It
    // used to answer Escape after the surface had (closing a modal opened from the
    // Library pane popped the pane off the stack too, and `s` behind the About
    // scrim pushed the stack in where nothing could see it). See surfaces.svelte.ts
    // for why the precedence is a predicate rather than stopPropagation.
    if (surfaceOpen()) return;
    // Escape pops one menu level: sub → detail → root → home.
    if (e.key === "Escape" && ui.menuOpen) {
      if (ui.menuSub) ui.menuSub = null;
      else if (ui.menuDetail) ui.menuDetail = null;
      else ui.menuOpen = false;
      return;
    }
    const t = e.target as HTMLElement | null;
    const typing =
      !!t &&
      (t.tagName === "INPUT" ||
        t.tagName === "TEXTAREA" ||
        t.tagName === "SELECT" ||
        t.isContentEditable);
    if (typing || e.ctrlKey || e.metaKey || e.altKey) return;
    if (e.key === "/" && !ui.menuOpen) {
      e.preventDefault();
      searchInput?.focus();
    } else if (e.key === "s") {
      toggleSettings();
    }
  }
</script>

<svelte:window onkeydown={onGlobalKey} />

<aside class="sidebar glass">
  <header class="head" data-tauri-drag-region style={chromeVars}>
    <div class="tb-traffic" role="group" aria-label="Window controls">
      {#each layout as kind (kind)}
        <button
          class={`tb-light ${CLASS[kind]}`}
          aria-label={`${LABEL[kind]} window`}
          onclick={ACTION[kind]}
        >
          <svg viewBox="0 0 10 10">
            {#if kind === "X"}
              <path d="M2.2 2.2 L7.8 7.8 M7.8 2.2 L2.2 7.8" />
            {:else if kind === "I"}
              <path d="M1.8 5 L8.2 5" />
            {:else}
              <path d="M2 6 L2 8 L4 8 M8 4 L8 2 L6 2 M2 8 L4.2 5.8 M8 2 L5.8 4.2" />
            {/if}
          </svg>
        </button>
      {/each}
    </div>
    <button
      class="gear"
      aria-label={ui.menuOpen ? "Close settings" : "Open settings"}
      use:tooltip={"Settings — press s to toggle"}
      aria-expanded={ui.menuOpen}
      onclick={toggleSettings}
    >
      <!-- Hamburger: the settings toggle. The ✕ morph below is
           shape-agnostic. -->
      <svg
        class="icon icon-menu"
        class:show={!ui.menuOpen}
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <path d="M3 6h18 M3 12h18 M3 18h18" />
      </svg>
      <svg class="icon icon-x" class:show={ui.menuOpen} viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round">
        <path d="M4 4 L12 12 M12 4 L4 12" />
      </svg>
    </button>
  </header>

  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <!-- Key router only: arrow keys move focus among the focused layer's
       buttons; the layers themselves are the semantic containers. -->
  <div class="stack" onkeydown={stackKeydown}>
    <!-- home: search + artists (the sidebar's normal job) -->
    <div class="layer" class:off={inMenu} inert={inMenu}>
      <div class="search">
        <svg viewBox="0 0 16 16" aria-hidden="true">
          <circle cx="7" cy="7" r="4.4" fill="none" stroke="currentColor" stroke-width="1.4" />
          <path d="M10.4 10.4 L13.6 13.6" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
        </svg>
        <input
          type="text"
          placeholder="Search library..."
          spellcheck="false"
          aria-label="Search library"
          use:tooltip={loading
            ? "Search once your library is built"
            : "Tip: press / anywhere to focus search — Escape clears"}
          disabled={loading}
          bind:this={searchInput}
          bind:value={ui.search}
          onkeydown={(e) => {
            // Escape in the field: clear if there's a query, else blur —
            // the old titlebar search's behavior, kept through the merge.
            if (e.key === "Escape") {
              if (ui.search) ui.search = "";
              else searchInput?.blur();
            }
          }}
        />
        {#if ui.search !== ""}
          <button class="search-clear" aria-label="Clear search" use:tooltip={"Clear search"} onclick={() => (ui.search = "")}>
            <svg viewBox="0 0 10 10"><path d="M2.2 2.2 L7.8 7.8 M7.8 2.2 L2.2 7.8" /></svg>
          </button>
        {/if}
      </div>

      {#if scanNote}
        <p class="scan-note" aria-live="polite">Updating library…</p>
      {/if}

      <nav
        class="scroll"
        class:enter={library.entering}
        bind:clientHeight={navHeight}
        aria-busy={loading}
      >
        <!-- Always visible: it's a control (clear selection + total count),
             not a search result — filtering it out made a zero-match query
             a dead end (critique P1). -->
        <button
          class="row"
          class:active={ui.activeArtistId === "all"}
          style:height="var(--sidebar-row-size)"
          onclick={() => select("all")}
        >
          <span class="name">All Artists</span>
          <!-- The row is a control, so it stays real. Its COUNT is data the scan
               has not produced yet: "0" there is a number, and a number is a
               claim. The pill says "unknown" in the same voice as the list. -->
          {#if loading}
            <span class="sk sk-count" style:--i={0}></span>
          {:else}
            <span class="count">{library.albums.length}</span>
          {/if}
        </button>
        {#each filtered as artist, i (artist.id)}
          <button
            class="row"
            style:--i={i + 1}
            class:active={ui.activeArtistId === artist.id}
            style:height="var(--sidebar-row-size)"
            onclick={() => select(artist.id)}
            oncontextmenu={(e) => artistMenu(e, artist.id)}
          >
            <span class="name mq" use:marquee={{ key: artist.id, hover: true }}><span class="mq-in">{artist.name}</span></span>
            <span
              class="count"
              class:pending={stagedCounts.has(artist.id)}
              use:tooltip={stagedCounts.has(artist.id)
                ? `${albumCounts.get(artist.id) ?? 0} albums — ${stagedCounts.get(artist.id)} awaiting import`
                : undefined}
              >{albumCounts.get(artist.id) ?? 0}</span
            >
          </button>
        {/each}
        {#if loading}
          <div class="sk-rows" aria-hidden="true">
            {#each Array(skRows) as _, i (i)}
              <div class="sk-row" style:height="var(--sidebar-row-size)">
                <span class="sk sk-name" style:--i={i + 1} style:width={skName(i)}></span>
                <span class="sk sk-count" style:--i={i + 2}></span>
              </div>
            {/each}
          </div>
        {/if}
        {#if !loading && filtered.length === 0 && ui.search.trim() !== ""}
          <!-- Action, not caption: a zero-match state that offers no remedy
               is a dead end (critique P3). -->
          <button class="empty" onclick={() => (ui.search = "")}>
            No artists match — clear search
          </button>
        {/if}
      </nav>
    </div>

    <!-- menu root: top-level panes -->
    <div
      class="layer root"
      inert={!(inMenu && !atDetail)}
      class:active={inMenu && !atDetail}
      class:receded={inMenu && atDetail}
      class:exiting={!inMenu && atDetail}
      class:no-anim={rootNoAnim}
      aria-hidden={!(inMenu && !atDetail)}
    >
      <div class="navrow">
        <button class="backchev" aria-label="Back to library" onclick={closeSettings}>
          <svg viewBox="0 0 10 14" aria-hidden="true"><path d="M7 2 L3 7 L7 12" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" /></svg>
        </button>
        <span class="navtitle">Settings</span>
      </div>
      <nav class="scroll">
        {#each rootPanes as pane (pane.id)}
          <button class="mrow" onclick={() => openDetail(pane.id)}>
            <span class="name">{pane.label}</span>
            <svg class="drill" viewBox="0 0 10 14" aria-hidden="true">
              <path d="M3 2 L7 7 L3 12" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
          </button>
        {/each}
      </nav>
      <!-- About: a footer row, not a nav pane. Help held exactly one item
           and it did nothing — now it opens the real dialog. -->
      <div class="rootfoot">
        <button class="mrow" onclick={(e) => openAbout(e.currentTarget)}>
          <span class="name">About Songstress</span>
        </button>
      </div>
    </div>

    <!-- detail: one pane at a time (a pushed sub-layer recedes it fully — the
         same full push the root layer uses, no parallax peek in a flat glass
         column) -->
    <div
      class="layer detail"
      inert={!(inMenu && atDetail && !atSub)}
      class:active={inMenu && atDetail && !atSub}
      class:receded={inMenu && atDetail && atSub}
      class:no-anim={detailNoAnim}
      aria-hidden={!(inMenu && atDetail && !atSub)}
    >
      <div class="navrow">
        <button class="backchev" aria-label="Back to settings" onclick={back}>
          <svg viewBox="0 0 10 14" aria-hidden="true"><path d="M7 2 L3 7 L7 12" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" /></svg>
        </button>
        <span class="navtitle">{detailTitle}</span>
      </div>
      {#if shownDetail === APPEARANCE}
        <div class="panebody">
          <section class="group">
            <div class="glabel">Sizes</div>
            <label class="ctl">
              <span class="ctlhead"
                ><span>Tile size</span><span class="val">{ui.tileSize}px</span></span
              >
              <input
                type="range" min="120" max="320" step="4"
                value={ui.tileSize}
                aria-valuetext={`${ui.tileSize} pixels`}
                style:background={fill(ui.tileSize, 120, 320)}
                oninput={(e) => (ui.tileSize = +e.currentTarget.value)}
              />
            </label>
            <label class="ctl">
              <span class="ctlhead"
                ><span>Sidebar rows</span><span class="val">{ui.sidebarRowSize}px</span></span
              >
              <input
                type="range" min="28" max="52" step="2"
                value={ui.sidebarRowSize}
                aria-valuetext={`${ui.sidebarRowSize} pixels`}
                style:background={fill(ui.sidebarRowSize, 28, 52)}
                oninput={(e) => (ui.sidebarRowSize = +e.currentTarget.value)}
              />
            </label>
          </section>
          <section class="group">
            <div class="glabel">Theme</div>
            <!-- the group is not a tab stop (tabindex -1); the checked radio
                 inside carries tabindex=0, arrows move within it -->
            <div
              class="seg"
              role="radiogroup"
              aria-label="Theme"
              tabindex="-1"
              onkeydown={segKeys(THEME_IDS, () => ui.theme, (id) => (ui.theme = id as typeof ui.theme))}
            >
              <!-- roving tabindex: the group is one Tab stop, arrows move inside it -->
              {#each THEMES as t, i (t.id)}
                <button
                  class="segbtn"
                  class:on={ui.theme === t.id}
                  role="radio"
                  aria-checked={ui.theme === t.id}
                  tabindex={ui.theme === t.id ? 0 : -1}
                  data-i={i}
                  onclick={() => (ui.theme = t.id)}
                >{t.label}</button>
              {/each}
            </div>
            <!-- Both are rows, so they touch: the run of rows starts here and
                 the group's 12px peer gap applies above it, to the segment. -->
            <div class="rows">
              <Toggle
                checked={ui.albumGradient}
                label="Album artwork gradient"
                onchange={(on) => (ui.albumGradient = on)}
              />
              <Toggle
                checked={ui.playbarGradient}
                label="Playbar artwork gradient"
                onchange={(on) => (ui.playbarGradient = on)}
              />
              <!-- Accent folded one level deep: the 12-dot grid was the highest
                   element count in the sidebar for the lowest-frequency
                   decision. Summary row here, grid in the sub layer. -->
              <button class="mrow subrow" onclick={() => openSub("accent")}>
                <span class="name">Accent</span>
                <span class="tail">
                  <span
                    class="cur"
                    class:stock={ui.accentColor === null}
                    style:background={ui.accentColor ?? undefined}
                    aria-hidden="true"
                  ></span>
                  <span class="curname">{accentName}</span>
                  <svg class="drill" viewBox="0 0 10 14" aria-hidden="true">
                    <path d="M3 2 L7 7 L3 12" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
                  </svg>
                </span>
              </button>
            </div>
          </section>
        </div>
        <div class="panefoot">
          <button class="frow" onclick={resetAppearance}>Reset appearance</button>
        </div>
      {:else if shownDetail === PLAYBACK}
        <!-- Bespoke pane, driven by the stores rather than the flattened menu
             model (Appearance does the same). Three subjects, each with an
             enable checkbox; the fields it owns appear underneath it, indented,
             and disappear when it is off — which is also what killed the dead
             `EQ Preset: …` row (it could not be reachable and meaningful while
             the equalizer was off, and now it simply isn't rendered).
             The Global Menu keeps its flat cycling rows and writes the same
             state through the same setters, so the two surfaces cannot drift. -->
        <div class="panebody">
          <section class="group">
            <div class="subject">
              <Toggle
                checked={playback.repeat !== "off"}
                label="Repeat"
                onchange={setRepeatOn}
              />
              {#if playback.repeat !== "off"}
                <div class="reveal" style="--seg-n: {REPEAT_MODES.length}">
                  <div
                    class="seg"
                    role="radiogroup"
                    aria-label="Repeat mode"
                    tabindex="-1"
                    onkeydown={segKeys(REPEAT_IDS, () => playback.repeat, (id) => setRepeatStage(id as "album" | "track"))}
                  >
                    {#each REPEAT_MODES as m (m.id)}
                      <button
                        class="segbtn"
                        class:on={playback.repeat === m.id}
                        role="radio"
                        aria-checked={playback.repeat === m.id}
                        tabindex={playback.repeat === m.id ? 0 : -1}
                        onclick={() => setRepeatStage(m.id)}
                      >{m.label}</button>
                    {/each}
                  </div>
                </div>
              {/if}
            </div>

            <div class="subject">
              <Toggle
                checked={playback.shuffle !== "off"}
                label="Shuffle"
                onchange={setShuffleOn}
              />
              {#if playback.shuffle !== "off"}
                <div class="reveal" style="--seg-n: {SHUFFLE_MODES.length}">
                  <div
                    class="seg"
                    role="radiogroup"
                    aria-label="Shuffle scope"
                    tabindex="-1"
                    onkeydown={segKeys(SHUFFLE_IDS, () => playback.shuffle, (id) => setShuffleStage(id as "album" | "artist" | "all"))}
                  >
                    {#each SHUFFLE_MODES as m (m.id)}
                      <button
                        class="segbtn"
                        class:on={playback.shuffle === m.id}
                        role="radio"
                        aria-checked={playback.shuffle === m.id}
                        tabindex={playback.shuffle === m.id ? 0 : -1}
                        onclick={() => setShuffleStage(m.id)}
                      >{m.label}</button>
                    {/each}
                  </div>
                </div>
              {/if}
            </div>

            <div class="subject">
              <Toggle
                checked={playback.eq.enabled}
                label="Equalizer"
                onchange={setEqEnabled}
              />
              {#if playback.eq.enabled}
                <div class="reveal">
                  <!-- 10 named presets is too many for a segment: same drill
                       language as Accent, value shown in the count column. -->
                  <button class="mrow subrow" onclick={() => openSub("eq-preset")}>
                    <span class="name">Preset</span>
                    <span class="tail">
                      <span class="curname">{playback.eq.preset ?? "Custom"}</span>
                      <svg class="drill" viewBox="0 0 10 14" aria-hidden="true">
                        <path d="M3 2 L7 7 L3 12" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
                      </svg>
                    </span>
                  </button>
                  <div class="glabel">Custom</div>
                  <label class="ctl">
                    <span class="ctlhead"
                      ><span>Preamp</span><span class="val">{fmtDb(playback.eq.preampDb)} dB</span></span
                    >
                    <input
                      type="range" min={-EQ_MAX_DB} max={EQ_MAX_DB} step="0.5"
                      value={playback.eq.preampDb}
                      aria-valuetext={`${fmtDb(playback.eq.preampDb)} decibels`}
                      style:background={fill(playback.eq.preampDb, -EQ_MAX_DB, EQ_MAX_DB)}
                      oninput={(e) => setEqPreamp(+e.currentTarget.value)}
                    />
                  </label>
                  {#each EQ_BANDS as hz, i (hz)}
                    <label class="ctl">
                      <span class="ctlhead"
                        ><span>{fmtHz(hz)}</span><span class="val">{fmtDb(playback.eq.gains[i])}</span></span
                      >
                      <input
                        type="range" min={-EQ_MAX_DB} max={EQ_MAX_DB} step="0.5"
                        value={playback.eq.gains[i]}
                        aria-label={`${fmtHz(hz)} hertz`}
                        aria-valuetext={`${fmtDb(playback.eq.gains[i])} decibels`}
                        use:tooltip={"Double-click to zero"}
                        style:background={fill(playback.eq.gains[i], -EQ_MAX_DB, EQ_MAX_DB)}
                        ondblclick={() => setEqBand(i, 0)}
                        oninput={(e) => setEqBand(i, +e.currentTarget.value)}
                      />
                    </label>
                  {/each}
                </div>
              {/if}
            </div>
          </section>
        </div>
        <div class="panefoot">
          <button class="frow" onclick={resetPlayback}>Reset playback</button>
        </div>
      {:else if shownDetail === LIBRARY}
        <!-- Bespoke for the same reason the other two are: six undifferentiated
             rows hid which decisions were about getting files in, which about
             re-reading them, and that anything was staged at all. Three groups,
             one door for the staged pile, and the live scan line where it
             explains the disabled rows. -->
        <div class="panebody">
          <section class="group">
            <div class="glabel">Import</div>
            <div class="rows">
              <button
                class="mrow"
                disabled={scanner.running}
                onclick={() => activateMenuItem("library.add-files")}
              >
                <span class="name">Import music files…</span>
                {#if ringOn("import")}
                  <span class="tail"><ProgressRing value={scanProgress} label={scanLine || "Finished"} phase={scanner.phase} /></span>
                {/if}
              </button>
              <button
                class="mrow"
                disabled={scanner.running}
                onclick={() => activateMenuItem("library.add-folder")}
              >
                <span class="name">Import music folder…</span>
                {#if ringOn("import")}
                  <span class="tail"><ProgressRing value={scanProgress} label={scanLine || "Finished"} phase={scanner.phase} /></span>
                {/if}
              </button>
              {#if stagedAlbums > 0}
                <!-- One door for the pile, and the amount as its tail: the two
                     verbs this replaced acted on the whole pile before saying
                     where it would go, and the destination is what the decision
                     is about. The modal states it per album. -->
                <button
                  class="mrow"
                  data-imports-door
                  disabled={scanner.running}
                  onclick={() => void openImportManager()}
                >
                  <span class="name">Manage imported music…</span>
                  <span class="tail">
                    {#if ringOn("save") || ringOn("discard")}
                      <ProgressRing value={scanProgress} label={scanLine || "Finished"} phase={scanner.phase} />
                    {:else}
                      <span class="curname">{stagedLabel}</span>
                    {/if}
                  </span>
                </button>
              {/if}
            </div>
          </section>

          <section class="group">
            <div class="glabel">Scan</div>
            <div class="rows">
              <button
                class="mrow"
                disabled={scanner.running}
                onclick={() => activateMenuItem("library.rescan")}
              >
                <span class="name">Scan for changes</span>
                {#if ringOn("scan")}
                  <span class="tail"><ProgressRing value={scanProgress} label={scanLine || "Finished"} phase={scanner.phase} /></span>
                {/if}
              </button>
              <button
                class="mrow"
                disabled={scanner.running}
                onclick={() => activateMenuItem("library.rescan-full")}
              >
                <span class="name">Re-read all files</span>
                {#if ringOn("full")}
                  <span class="tail"><ProgressRing value={scanProgress} label={scanLine || "Finished"} phase={scanner.phase} /></span>
                {/if}
              </button>
            </div>
          </section>

          <section class="group">
            <div class="glabel">Storage</div>
            <button class="mrow" onclick={() => activateMenuItem("library.choose-folder")}>
              <span class="name">Music folders…</span>
              <span class="tail">
                {#if ringOn("folder")}
                  <ProgressRing value={scanProgress} label={scanLine || "Finished"} phase={scanner.phase} />
                {:else}
                  <span class="curname">{folderLabel}</span>
                {/if}
              </span>
            </button>
          </section>
        </div>
        <div class="panefoot">
          <!-- status, not padding: this pane had no idea when the library was
               last built (heuristic 1 gap). Two lines by design — one long
               joined line wrapped mid-clause as the counts grew. -->
          <div class="fstat">
            <span>{libStats}</span>
            {#if ui.lastScan !== null}
              <span class="fsub">last scan {scanTime}</span>
            {/if}
          </div>
        </div>
      {:else}
        <div class="scroll items">
          {#each detailItems as item (item.id)}
            <button
              class="irow"
              disabled={!item.enabled}
              onclick={() => activateMenuItem(item.id)}
            >
              <span class="name">{item.label}</span>
              {#if item.checked === true}
                <svg class="ok" viewBox="0 0 14 14" aria-hidden="true">
                  <path d="M2 7.5 L5.5 11 L12 3.5" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" />
                </svg>
              {/if}
            </button>
          {/each}
        </div>
      {/if}
    </div>

    <!-- sub layer: one level deeper than a pane (currently the accent
         picker). Same drawer language: enters from the right, pushes the
         detail layer fully left, `‹` returns the way it came. -->
    <div
      class="layer sub"
      inert={!(inMenu && atDetail && atSub)}
      class:active={inMenu && atDetail && atSub}
      aria-hidden={!(inMenu && atDetail && atSub)}
    >
      <div class="navrow">
        <button class="backchev" aria-label={`Back to ${detailTitle}`} onclick={back}>
          <svg viewBox="0 0 10 14" aria-hidden="true"><path d="M7 2 L3 7 L7 12" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" /></svg>
        </button>
        <span class="navtitle">{shownSub === "eq-preset" ? "EQ Preset" : "Accent"}</span>
      </div>
      {#if shownSub === "eq-preset"}
        <!-- 10 named presets: a list, not a segment. Same row language as the
             generic panes' checked items, so "this is the one that is on" is
             one grammar across the stack. -->
        <div class="scroll items">
          {#each EQ_PRESETS as p (p.name)}
            <button
              class="irow"
              aria-current={playback.eq.preset === p.name || undefined}
              onclick={() => applyEqPreset(p.name)}
            >
              <span class="name">{p.name}</span>
              {#if playback.eq.preset === p.name}
                <svg class="ok" viewBox="0 0 14 14" aria-hidden="true">
                  <path d="M2 7.5 L5.5 11 L12 3.5" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" />
                </svg>
              {/if}
            </button>
          {/each}
        </div>
        {#if playback.eq.preset === null}
          <!-- a hand-edited curve is a real state; saying so beats showing
             nothing (setEqBand clears the preset name) -->
          <div class="panefoot">
            <div class="fstat">Custom curve — adjust the sliders in Playback</div>
          </div>
        {/if}
      {:else}
      <div class="panebody">
        <section class="group">
          <div class="glabel">Presets</div>
          <div class="swatches">
            {#each ACCENT_PRESETS as p (p.name)}
              <button
                class="swatch"
                class:selected={ui.accentColor === p.hex}
                style:background={p.hex ?? "linear-gradient(135deg, #a78bfa 50%, #7c58f0 50%)"}
                style:box-shadow={swatchRing(p.hex, ui.accentColor === p.hex)}
                use:tooltip={clampHint(p.hex, p.name)}
                aria-label={clampHint(p.hex, p.name)}
                onclick={() => (ui.accentColor = p.hex)}
              ></button>
            {/each}
          </div>
        </section>

        <!-- Its own group: the picker is an EDITOR, not a twelfth choice in a
             list of named ones, and a rainbow dot among named colors promised
             something the presets beside it didn't need — plus it inherited a
             preset's color the moment one was picked. Readout row language
             (label left, Micro value right, control at the end of the line),
             so it says the same thing as the slider rows above it. -->
        <section class="group">
          <div class="glabel">Custom</div>
          <label class="colorrow">
            <span class="ctlhead">
              <span>Pick any color</span>
              <span class="val">{hasCustom ? ui.accentColor : "not set"}</span>
            </span>
            <span
              class="swatch custom"
              class:picked={hasCustom}
              style:background={hasCustom ? chipColor : undefined}
              aria-hidden="true"
            >
              <input
                type="color"
                bind:value={customHex}
                aria-label="Custom accent color"
                oninput={(e) => (ui.accentColor = e.currentTarget.value)}
              />
            </span>
          </label>
        </section>
      </div>
      {/if}
    </div>
  </div>
</aside>

<style>
  .sidebar {
    position: absolute;
    top: 0;
    left: 0;
    bottom: var(--playbar-h);
    z-index: 10;
    display: flex;
    flex-direction: column;
    width: var(--sidebar-width);
    /* border-bottom was doubling the seam: the playbar's own border-top
       already runs full width underneath, so sidebar-bottom + playbar-top
       read as a 2px line along the sidebar only. */
    border-right: 1px solid var(--border);
    user-select: none;
  }

  .head {
    position: relative;
    flex: none;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 16px 10px 10px var(--tb-margin, 15px);
  }

  .tb-traffic {
    display: flex;
    /* Mirror, not imitation: the spacing Klassy puts between its button rects
       (10) plus the 1px a small-circle button loses to its rect, all read from
       klassyrc by `kde_window_decoration` and published as --tb-gap. */
    gap: var(--tb-gap, 11px);
  }

  .tb-light {
    position: relative;
    width: var(--tb-dot, 15px);
    height: var(--tb-dot, 15px);
    border-radius: 50%;
    border: 1px solid rgba(0, 0, 0, 0.18);
    padding: 0;
    cursor: pointer;
  }

  /* Hit area: the dot alone is a 15px target, under this app's own ~28px
     minimum, so the invisible box grows by half the gap on each side — which
     makes it exactly the cluster's pitch, by construction, for whatever the
     decoration asks for. Any more and neighbouring targets overlap and the later
     button steals the shared strip, putting the close dot's east edge inside
     minimize. Same trick the surface dots don't need (their button is 26px
     around the same dot). */
  .tb-light::before {
    content: "";
    position: absolute;
    inset: calc(var(--tb-gap, 11px) / -2);
  }

  /* Absolute centering — place-items:center on a native <button> drifts
     ~1px down in WebKitGTK (shadow-DOM layout), which read as off-glyphs. */
  .tb-light svg {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: 10px;
    height: 10px;
    stroke: rgba(0, 0, 0, 0.62);
    stroke-width: 1.4;
    stroke-linecap: round;
    fill: none;
    opacity: 0;
  }

  /* System traffic-light spec, BOTH desktops (owner call 2026-09-23): the GTK
     theme at ~/.config/gtk-{3.0,4.0}/gtk.css + windows-assets/*.svg — NOT the
     Klassy mirror (--tb-* vars now serve only the SurfaceClose family).
     Hover never darkens the dot: glyph appearance IS the feedback. Pressed
     has its own fill per button. Glyphs are dark, matching the system. */
  .tb-close { background: #FF5F57; }
  .tb-close:hover { background: #FF5F57; }
  .tb-close:active { background: #E04C44; }
  .tb-min   { background: #FEBD2E; }
  .tb-min:hover { background: #FEBD2E; }
  .tb-min:active { background: #DC9E22; }
  .tb-max   { background: #28C840; }
  .tb-max:hover { background: #28C840; }
  .tb-max:active { background: #1FAA33; }

  /* Glyph appears only on the button actually hovered (Klassy behavior). */
  .tb-light:hover svg {
    opacity: 1;
  }

  .gear {
    position: relative;
    flex: none;
    border: none;
    background: transparent;
    color: var(--text);
    width: 32px;
    height: 32px;
    border-radius: 8px;
    display: grid;
    place-items: center;
    cursor: pointer;
    padding: 0;
  }

  .gear:hover,
  .gear[aria-expanded="true"] {
    background: var(--hover);
    color: var(--text);
  }

  .gear:active {
    background: var(--active);
  }

  /* Icon morph: menu and ✕ crossfade while rotating — a state change,
     not a swap. 160ms ease-out (fast, purposeful). */
  .gear .icon {
    position: absolute;
    top: 50%;
    left: 50%;
    width: 20px;
    height: 20px;
    transform: translate(-50%, -50%);
    transition:
      opacity 160ms ease-out,
      transform 160ms ease-out;
  }

  .gear .icon:not(.show) {
    opacity: 0;
  }

  .gear .icon-menu:not(.show) {
    transform: translate(-50%, -50%) rotate(90deg);
  }

  .gear .icon-x:not(.show) {
    transform: translate(-50%, -50%) rotate(-90deg);
  }

  /* --- menu stack (iOS Settings-style) -----------------------------------
     Layers are absolute and slide on transform ONLY (compositor-friendly,
     interruptible by state changes without layout). Apple's standard curve.
     Off-screen layers keep pointer-events off; the receded root is a
     visual-only parallax, not interactive. */
  .stack {
    position: relative;
    flex: 1;
    min-height: 0;
    /* `clip`, not `hidden`: hidden still creates a scroll port that a
       programmatic focus() inside a translated layer can scroll (see
       focusFirst). clip cannot be scrolled at all, so a mis-timed focus can
       never shift the layer stack sideways. hidden stays first as the
       fallback for engines without `overflow: clip`. */
    overflow: hidden;
    overflow: clip;
  }

  .layer {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    pointer-events: none;
    transition: transform 320ms var(--ease-drawer);
  }

  /* On-screen layers are interactive; off-screen ones aren't. (The home
     layer once had no rule at all and the artist list went click-dead —
     layers default to pointer-events: none up there.) Off-screen
     siblings are fully outside .stack's clip, so auto never over-captures. */
  .layer:not(.off) {
    pointer-events: auto;
  }

  /* home: in at 0, out to the LEFT (it was pushed) */
  .layer.off {
    transform: translateX(-100%);
  }

  /* root: in from the RIGHT, at 0 when active, and a FULL push left when a
     detail is open (the -30% iOS parallax recede was tried first — on a flat
     glass sidebar the peeking root items read as clutter, user rejected it). */
  .layer.root {
    transform: translateX(100%);
    z-index: 1;
  }

  .layer.root.active {
    transform: translateX(0);
  }

  .layer.root.receded {
    transform: translateX(-100%);
  }

  /* full-pop close: root exits further LEFT, in parallel with home sliding
     in — same speed, never crossing the home layer */
  .layer.root.exiting {
    transform: translateX(-200%);
  }

  /* one-frame transition suppression while an off-screen layer teleports
     back to its +100% entry slot (root from -200%, detail from -100%) */
  .layer.root.no-anim,
  .layer.detail.no-anim {
    transition: none;
  }

  /* detail: in from the RIGHT, on top */
  .layer.detail {
    transform: translateX(100%);
    z-index: 2;
  }

  .layer.detail.active {
    transform: translateX(0);
  }

  /* A pushed sub layer takes the detail pane fully off-screen LEFT (same
     full push as root — a flat glass column has no room for a parallax peek,
     and the sub layer covers the column edge to edge anyway). */
  .layer.detail.receded {
    transform: translateX(-100%);
  }

  /* sub: one level deeper than a pane (the accent picker). Same drawer
     language, one z-step up. */
  .layer.sub {
    transform: translateX(100%);
    z-index: 3;
  }

  .layer.sub.active {
    transform: translateX(0);
  }

  /* --- nav row (per-layer title bar; wayfinding per the iOS pattern) ---
     No divider under the title (2026-08-27, user decision): the root pane
     dropped its line first and read correctly bare — keeping it on detail
     panes only made the identical navrow render differently per depth. */
  .navrow {
    flex: none;
    display: flex;
    align-items: center;
    gap: 4px;
    height: 44px;
    padding: 0 10px 0 12px;
  }

  .navtitle {
    font-size: 17px;
    font-weight: 700;
    letter-spacing: 0.01em;
    color: var(--text);
  }

  .backchev {
    border: none;
    background: transparent;
    width: 28px;
    height: 28px;
    border-radius: 7px;
    display: grid;
    place-items: center;
    cursor: pointer;
    padding: 0;
    color: var(--accent);
  }

  .backchev:hover,
  .backchev:active {
    background: var(--active);
  }

  .backchev svg {
    width: 10px;
    height: 14px;
  }

  /* --- scrollable list body ---------------------------------------------- */
  .scroll {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 0 8px;
  }

  /* home: search */
  .search {
    position: relative;
    margin: 10px 12px 12px;
    flex: none;
  }

  .search svg {
    position: absolute;
    left: 9px;
    top: 50%;
    width: 14px;
    height: 14px;
    transform: translateY(-50%);
    color: var(--text-dim);
    pointer-events: none;
  }

  .search input {
    width: 100%;
    height: 30px;
    padding: 0 26px 0 30px;
    border-radius: 8px;
    border: 1px solid var(--border);
    background: var(--hover);
    color: var(--text);
    font-size: 15px;
    outline: none;
  }

  /* Focus state = accent stroke only (the system focus language). The old
     inset outline ring was invisible against the near-opaque field fill. */
  .search input:focus {
    border-color: var(--accent);
  }

  .search input::placeholder {
    color: var(--text-dim);
  }

  /* Out while the library is being built: the grid and this list are both
     placeholders, so a query typed now can only match nothing — and a field that
     takes the typing and answers with silence is the worse lie. The glyph dims
     with it (`:has`, since the icon precedes the input). */
  .search input:disabled {
    opacity: 0.55;
    cursor: default;
  }

  .search:has(input:disabled) > svg {
    opacity: 0.55;
  }

  .search-clear {
    position: absolute;
    right: 5px;
    top: 50%;
    /* Centering lives in `transform` everywhere on this engine: this
       WebKitGTK misapplies the STANDALONE `translate`/`rotate`/`scale`
       properties on some elements (measured: the × sat 4px high — the
       svg's percentage translate resolved wrong on the block axis), the
       same class of silence as unprefixed `user-select`. transform is the
       dialect this engine provably honors (the search magnifier above).
       (owner report 2026-09-05) */
    transform: translateY(-50%);
    display: grid;
    place-items: center;
    width: 18px;
    height: 18px;
    border: none;
    border-radius: 50%;
    background: transparent;
    color: var(--text-dim);
    cursor: pointer;
  }

  .search-clear:hover {
    background: var(--hover);
    color: var(--text);
  }

  .search-clear svg {
    /* Absolute centering — place-items:center on a native <button> drifts
       ~1px down in WebKitGTK (shadow-DOM layout); same fix as the traffic
       lights (critique: "× not centered in its hover circle"). transform,
       not the standalone `translate` property — see .search-clear. */
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: 8px;
    height: 8px;
    stroke: currentColor;
    stroke-width: 1.4;
    stroke-linecap: round;
    fill: none;
  }

  /* rows (artists / menu top-level / items share the base) */
  .row,
  .mrow,
  .irow {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    width: 100%;
    padding: 0 10px;
    border: none;
    border-radius: 8px;
    background: transparent;
    color: var(--text);
    font-size: 15px;
    cursor: default;
    text-align: left;
  }

  /* ONE row language (impeccable critique 2026-08-30, option A): menu
     rows share the artist-row rhythm — same height token (user-tunable),
     same 13px type, same label column. The old fixed 40px/13.5px cadence
     made the same container speak three row dialects. */
  .mrow,
  .irow {
    height: var(--sidebar-row-size);
    font-size: 15px;
  }

  .row:hover,
  .mrow:hover,
  .irow:hover:not(:disabled) {
    background: var(--hover);
  }

  /* Press feedback: instant and distinct from hover */
  .mrow:active:not(:disabled),
  .irow:active:not(:disabled) {
    background: var(--active);
  }

  /* Keyboard focus: explicit accent ring (inset) instead of the browser's
     inconsistent default */
  .mrow:focus-visible,
  .irow:focus-visible,
  .backchev:focus-visible,
  .gear:focus-visible,
  .search-clear:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
  }

  .row.active {
    background: var(--active);
    /* Translucent wash → normal text color (see PlayBar .playpause note). */
    color: var(--text);
    font-weight: 600;
  }

  .irow:disabled,
  .mrow:disabled {
    color: var(--text-dim);
    cursor: default;
  }

  .name {
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
  }

  /* Hover marquee for overlong artist names (same action + strip as the
     playbar title/artist): the row rests at plain ellipsis; hovering an
     overflowing name slides it. No animation delay — hover already states
     intent. The :global(.is-over) escape hatch is the same one the playbar
     uses: the class is JS-owned, the compiler never sees it. */
  .row .mq:global(.is-over) .mq-in {
    animation: mq-scroll var(--mq-dur, 8s) linear infinite;
  }

  .count {
    font-size: 13px;
    color: var(--text-dim);
    flex: none;
  }

  /* The pending-import dot: the accent dot is this app's "a decision lives
     here" mark (same tier as the mode-button badge). Trailing so the COUNT
     keeps its column alignment; the dot grows to the right where nothing
     is measured. */
  .count.pending::after {
    content: "";
    display: inline-block;
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--accent);
    margin-left: 5px;
    vertical-align: middle;
  }

  /* The artist list's placeholder. `.sk-row` copies `.row`'s box exactly — same
     height token, same 10px inset, same 8px radius — because the real rows land
     INTO this rhythm; a skeleton 2px taller per row moves the whole list at the
     moment the library arrives. */
  .sk-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 0 10px;
    border-radius: 8px;
  }

  .sk-name {
    height: 13px;
    min-width: 0;
    border-radius: 999px;
  }

  /* 22px — the width of a two-digit album count, so the column reads as "a
     number belongs here" instead of as another bar. */
  .sk-count {
    width: 22px;
    height: 11px;
    border-radius: 999px;
    flex: none;
  }

  /* This list keeps app.css's defaults unchanged — a row of text is the thing with
     a position, so the shared ladder is exactly right here. What it does carry is
     the index: each artist row sets `--i` inline ("All Artists" is child 0 and
     excluded below, so the first ARRIVING row is 1). Note that dials must be
     declared on the ANIMATING element: a `--enter-step` on `.scroll.enter` would be
     overridden for each row by the child's own declaration in the global rule. */
  /* The entrance utility is global (`.enter` in app.css), but this list's first
     child is a CONTROL that has been on screen the whole time — running it through
     the entrance would blink "All Artists" out and back while the user may be
     reaching for it. The arriving content starts at the second child. */
  .scroll.enter > .row:first-child {
    animation: none;
  }

  /* Trailing glyphs sit in the count column: ✓ state (checked items) and
     › drill (root rows). No left check column — labels align with the
     pane title and the artist names. */
  .ok {
    width: 14px;
    height: 14px;
    flex: none;
    color: var(--accent);
  }

  .irow:disabled .ok {
    color: var(--text-dim);
  }

  /* The drill is a meaningful graphic (it says "this row navigates"), so it
     needs 3:1 — the old extra opacity: .6 on top of --text-dim measured
     ≈2.3:1 over a bright wallpaper under the 0.7 chrome tier. At --text-dim
     alone it is ≈3.5:1: still quiet, but it survives the wallpaper. */
  .drill {
    width: 10px;
    height: 14px;
    flex: none;
    color: var(--text-dim);
    transition: color 120ms var(--ease-out);
  }

  /* Hover AND keyboard focus both resolve it — the ring alone left keyboard
     users with the weakest version of the one glyph that explains the drawer. */
  .mrow:hover .drill,
  .mrow:focus-visible .drill {
    color: var(--text);
  }

  /* About footer: anchored to the root's bottom (the layer is a flex
     column; .scroll's flex:1 pushes it down). Dimmed — it's a quiet exit,
     not a pane. Padding/flex live in the shared .panefoot rule. */
  .rootfoot .mrow {
    color: var(--text-dim);
  }

  .rootfoot .mrow:hover {
    color: var(--text);
  }

  .empty {
    display: block;
    width: 100%;
    margin: 14px 10px;
    padding: 4px 0;
    border: none;
    background: transparent;
    font-size: 14px;
    color: var(--text-dim);
    text-align: left;
    cursor: pointer;
  }

  .empty:hover {
    color: var(--text);
  }

  .empty:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
  }

  /* Transient scanning state for a populated library — an empty one has nothing
     to announce but itself, so it shows the skeleton instead (loadingState). */
  .scan-note {
    flex: none;
    margin: -8px 12px 4px;
    font-size: 13px;
    color: var(--text-dim);
  }

    /* --- pane bodies: Appearance + Accent ---------------------------------
     Rhythm is deliberately UNEVEN (layout critique): ~10px inside a group,
     30px between groups. The old uniform 16px cadence across five
     heterogeneous controls gave the eye no group seams, so it read as one
     block ("crammed") even though 40% of the column below it is empty.
     Group labels use DESIGN.md's Label tier — the tier this pane never
     used, and the one reserved for exactly these seams. */
  .panebody {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 14px 14px 8px;
    display: flex;
    flex-direction: column;
    gap: 30px;
  }

  /* Spacing ladder inside a pane — three rungs, each one a relationship:
       6px  a thing and the thing it owns or labels
       12px peers within one subject
       30px between subjects (.panebody's own gap)
     One undifferentiated gap made a toggle's own reveal sit exactly as far from
     it as an unrelated subject did, so the spacing described no hierarchy. */
  /* Two distances inside a subject: 12px between anything and its neighbour —
     a group label included, because a title crammed 6px onto the first control
     reads as a caption glued to it, not as a heading — and 6px from a row to the
     fields it owns, the only hug in the pane. The hug is a negative margin on a
     parent-template element, because that is the only way to go BELOW the
     container gap. Mechanism matters:
     a parent's scoped selector does not reach a child component's root element,
     so `* + *` margins silently skipped every <Toggle> in the pane and its
     neighbours collapsed to zero separation. gap works regardless of who
     compiled the child. */
  .group {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  /* One setting and the fields it owns. The 6px lives here, as a container gap —
     not as a negative margin on the reveal — so it also applies when the owner is
     a component root (<Toggle>), which a parent's scoped selector cannot reach.
     The three Playback subjects are ONE group of three rows, not three groups:
     30px between them said "unrelated subjects" and made the pane look like a
     list with the pages falling out of it. */
  .subject {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  /* A run of action rows is ONE block: rows touch at the app's row rhythm
     (--sidebar-row-size pitch, no gap), exactly as they do in the root layer
     and in the generic model-driven list. The group's 10px gap is air for
     mixed content — a label, a slider, a reveal — and letting it fall between
     rows too is what made this pane's rows float apart from every other list
     in the sidebar. */
  .rows {
    display: flex;
    flex-direction: column;
  }

  .glabel {
    font-size: 13.5px;
    font-weight: 600;
    letter-spacing: 0.09em;
    text-transform: uppercase;
    color: var(--text-dim);
  }

  /* Fields that belong to an enable checkbox, revealed underneath it. Indented
     12px so the dependency is visible without a card (inset translucent panels
     on the 0.7 glass are banned app-wide), and it MOUNTS with the checkbox
     rather than animating its own height — a height transition on a block that
     can hold 11 sliders needs a magic max-height, and 200ms of fade+rise reads
     the same. app.css's reduced-motion rule covers the animation. */
  .reveal {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding-left: 12px;
    animation: reveal-in 200ms var(--ease-out);
  }

  @keyframes reveal-in {
    from {
      opacity: 0;
      transform: translateY(-4px);
    }
  }

  /* A control's label is its row's primary content: list tier at full
     strength (the old 12px/400 dim was a tier DESIGN.md doesn't declare). */
  .ctl {
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 15px;
    color: var(--text);
  }

  /* Glass sliders: the native uniform track is replaced by the inline
     fill gradient (accent → hover wash); the thumb is an accent dot. */
  .panebody input[type="range"] {
    -webkit-appearance: none;
    appearance: none;
    height: 4px;
    border-radius: 999px;
    background: var(--hover);
    cursor: pointer;
  }

  .panebody input[type="range"]::-webkit-slider-thumb {
    -webkit-appearance: none;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: var(--accent);
    box-shadow: 0 1px 4px rgba(0, 0, 0, 0.4);
  }

  .panebody input[type="range"]:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 4px;
  }

  /* Value readout: the pane's whole job is picking a number and the sliders
     said nothing. Right-aligned in the count column like the artist rows'
     counts, tabular so the digits don't jitter while dragging. */
  .ctlhead {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 8px;
  }

  .val {
    font-size: 13px;
    font-variant-numeric: tabular-nums;
    color: var(--text-dim);
  }

  /* Glass segmented control: the state space shown at full width — Light /
     Dark / System, the active one on the accent wash. Hover stays NEUTRAL
     (row language: neutral wash on hover, accent wash for state/press). */
  .seg {
    display: grid;
    /* 2..4 segments; the group sets --seg-n (Theme 3, Repeat 2, Shuffle 3). */
    grid-template-columns: repeat(var(--seg-n, 3), 1fr);
    gap: 2px;
    padding: 2px;
    height: 32px;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--hover);
  }

  .segbtn {
    border: none;
    /* 7px = the icon tier. A 2px inset inside an 8px trough would compute to
       6px, but 6 is not on the radius ladder and 7 is indistinguishable here
       — no documented exception needed. */
    border-radius: 7px;
    background: transparent;
    color: var(--text-dim);
    font-size: 14px;
    cursor: pointer;
    transition: background 140ms var(--ease-out), color 140ms var(--ease-out);
  }

  .segbtn:hover:not(.on) {
    background: var(--hover);
    color: var(--text);
  }

  .segbtn.on {
    background: var(--active);
    color: var(--text);
    font-weight: 600;
  }

  .segbtn:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
  }

  /* Glass checkbox (impeccable critique 2026-08-30): the OS-default square
     was the only non-glass control in the app. The native input stays
     focusable and drives everything; .box is its visual. The !important is
     gone now that the label rules are split per control. */
  /* The checkbox itself moved to components/Toggle.svelte (the playbar's
     equalizer popover needs the same control; copying the CSS was worse). */

  /* Accent summary row: name · current color · drill. The 12-dot grid lives
     one level deeper now — it was the densest thing in the sidebar for the
     rarest decision in the pane. */
  /* Readout column of a pane row ("3 albums", "1 folder", "Rock"). Selector is
     .panebody, not .subrow: a row that carries a number is not necessarily a
     drill, and the number is the same language either way. */
  .panebody .tail {
    display: flex;
    align-items: center;
    gap: 8px;
    flex: none;
    margin-left: auto;
  }

  .panebody .cur {
    width: 16px;
    height: 16px;
    flex: none;
    border: 1px solid var(--border);
    border-radius: 50%;
  }

  .panebody .cur.stock {
    background: linear-gradient(135deg, #a78bfa 50%, #7c58f0 50%);
  }

  .panebody .curname {
    font-size: 13px;
    color: var(--text-dim);
  }

  /* --- pane footers ------------------------------------------------------
     One dim row per detail layer: an action where a real reset exists, a
     status line where it doesn't. .frow is an ACTION, so it sits at 0.82 of
     --text rather than --text-dim — that measures ≈4.6:1 over the worst case
     (bright wallpaper under the 0.7 chrome tier), where --text-dim does not
     clear AA. The status line is caption-grade and may stay dim. */
  .panefoot,
  .rootfoot {
    flex: none;
    padding: 4px 8px 10px;
  }

  .frow {
    display: flex;
    align-items: center;
    width: 100%;
    height: var(--sidebar-row-size);
    padding: 0 10px;
    border: none;
    border-radius: 8px;
    background: transparent;
    color: var(--text);
    opacity: 0.82;
    font-size: 15px;
    text-align: left;
    cursor: pointer;
  }

  .frow:hover {
    background: var(--hover);
    opacity: 1;
  }

  .frow:active {
    background: var(--active);
  }

  .frow:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
    opacity: 1;
  }

  .fstat {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 6px 10px 2px;
    font-size: 13px;
    font-variant-numeric: tabular-nums;
    color: var(--text-dim);
  }

  /* The second clause is subordinate: same tier, quieter. */
  .fstat .fsub {
    opacity: 0.75;
  }

  /* Swatches (sub layer): 6 per row — 11 presets + the custom picker is 12,
     so the grid is two even rows instead of the old ragged 7 + 5. */
  .swatches {
    display: grid;
    grid-template-columns: repeat(6, 20px);
    justify-content: space-between;
    /* Row gap 14px: the selection/hover outline reaches 4px past a dot, and
       with no row gap the outlined dot's ring touched the dot above it.
       Column gap MUST stay 0: the tracks are fixed 20px and `space-between`
       is what centers the row, so any minimum column gap makes the tracks
       overflow their own box (6*20 + 5*20 = 220 in a 207px pane) and the last
       dot runs into the sidebar border — measured after the first attempt at
       this rule. Horizontal spacing is then fluid ((207-120)/5 = 17px, still
       clearing the 4px outline twice over) and both edges sit on the pane's
       14px padding, aligned with the seam above and the picker row below. */
    gap: 14px 0;
  }

  /* Custom picker row (its own group, outside the preset grid). */
  .colorrow {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 15px;
    color: var(--text);
    cursor: pointer;
  }

  .colorrow .ctlhead {
    flex: 1;
  }

  .swatch {
    width: 20px;
    height: 20px;
    padding: 0;
    border: 1px solid var(--border);
    border-radius: 50%;
    cursor: pointer;
    outline: 2px solid transparent;
    outline-offset: 2px;
    transition: outline-color 120ms ease;
  }

  .swatch:hover {
    outline-color: var(--text-dim);
  }

  /* The outline channel is shared with hover/selected, so focus needs its own
     color or keyboard users get nothing. The custom picker's input is
     opacity: 0 inside the label — :focus-within is its only ring. */
  .swatch:focus-visible,
  .swatch.custom:focus-within {
    outline-color: var(--accent);
  }

  .swatch.selected {
    outline-color: var(--text);
  }

  .swatch.custom {
    position: relative;
    overflow: hidden;
    background: conic-gradient(#e5484d, #ffb224, #46a758, #00a2c7, #7c58f0, #e5484d);
  }

  .swatch.custom input {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    /* appearance:none: WebKit otherwise draws its own ~14px color swatch
       inside the box, which read as a shrunken, mis-centered dot. */
    -webkit-appearance: none;
    appearance: none;
    border: none;
    padding: 0;
    opacity: 0;
    cursor: pointer;
  }

  /* Once a custom color is picked, the dot IS that color and a conic corner
     marks it as the picker (a twelfth identical rainbow taught nothing). */
  .swatch.custom.picked::after {
    content: "";
    position: absolute;
    right: -1px;
    bottom: -1px;
    width: 9px;
    height: 9px;
    border-radius: 50%;
    background: conic-gradient(#e5484d, #ffb224, #46a758, #00a2c7, #7c58f0, #e5484d);
  }
</style>
