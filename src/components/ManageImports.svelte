<script lang="ts">
  /**
   * The door that replaces "Save imported music" / "Discard imported music".
   *
   * Those two verbs asked the question in the wrong order: they acted on the
   * whole pile before saying where the files would go, and the pile's shape —
   * which album joins which folder, which one starts a new artist — is what the
   * decision is actually about. So the destination is on screen per album
   * (resolved in Rust, by the same cascade the save itself will follow), the
   * decision is marked per album, and one Apply commits every mark.
   */
  import { tick } from "svelte";
  import { ui } from "../lib/stores/ui.svelte";
  import { scanner } from "../lib/stores/scanner.svelte";
  import {
    applyImportDecisions,
    closeImportManager,
    imports,
    markAllImports,
    markImport,
    refreshImportPlan,
    toggleImportTracks,
  } from "../lib/stores/imports.svelte";
  import {
    destinationLine,
    fmtDuration,
    groupByArtist,
    plural,
    summarize,
    trackNumbers,
  } from "../lib/importPlan";
  import SurfaceClose from "./SurfaceClose.svelte";
  import ProgressRing from "./ProgressRing.svelte";
  import { trapTab } from "../lib/focusTrap";

  const groups = $derived(groupByArtist(imports.plan));
  const held = $derived(imports.report?.already ?? []);
  // The report band sits OUTSIDE the scroller, so it has to stay short: three names,
  // then the count. The full list rides in the `title`.
  const heldShown = $derived(held.slice(0, 3));
  const heldMore = $derived(held.length - heldShown.length);
  const heldAll = $derived(held.map((a) => `${a.artist} — ${a.title}`).join(", "));
  const was = (n: number) => (n === 1 ? "was" : "were");
  // What Apply did, in the order the verbs matter in. The duplicate line is the
  // one that earns its space: it reports the only deletion of a file the user
  // owns that this app performs, so it names which file went and which stayed.
  const applyLines = $derived.by(() => {
    const a = imports.applied;
    if (!a) return [] as string[];
    const out: string[] = [];
    if (a.moved) out.push(`${plural(a.moved, "file")} moved into your library`);
    if (a.duplicates) {
      // Which file went, said in the same number as the count above it: this line
      // reports a deletion of the user's own file, so it would be rude to be
      // ungrammatical in it.
      const one = a.duplicates === 1;
      out.push(
        `${plural(a.duplicates, "file")} ${was(a.duplicates)} identical to music you already own — your ${
          one ? "copy was" : "copies were"
        } deleted, the ${one ? "one" : "ones"} in your library stayed`,
      );
    }
    if (a.discarded)
      out.push(
        `${plural(a.discarded, "file")} ${was(a.discarded)} dropped from the library; your files stay where they are`,
      );
    if (a.vanished)
      out.push(
        `${plural(a.vanished, "file")} ${was(a.vanished)} already gone from disk and forgotten`,
      );
    return out;
  });
  const sum = $derived(summarize(imports.plan, imports.decisions));
  const busy = $derived(imports.applying || scanner.running);

  // The arc is the outer count — albums applied of albums marked — plus the
  // fine-grained fraction the backend reports while files are copying, so a
  // 36-track album moves smoothly instead of jumping at the end. Only the copy
  // phase counts: the scan that follows a save has its own totals.
  const inner = $derived(
    scanner.running && scanner.phase === "import" && scanner.total > 0
      ? Math.min(1, scanner.done / scanner.total)
      : 0,
  );
  const progress = $derived(
    imports.total > 0 ? Math.min(1, (imports.done + inner) / imports.total) : inner,
  );
  // A file imported while this window is open — from the pane behind it, the
  // Global Menu, anywhere — has no reason to be invisible here, and a mark made
  // before it landed would otherwise apply to an album nobody has seen. So the
  // plan is re-read whenever a scan finishes with this window up.
  let wasScanning = false;
  $effect(() => {
    const running = scanner.running;
    if (wasScanning && !running && imports.open) void refreshImportPlan();
    wasScanning = running;
  });

  // What Apply is about to do, said in the order the buttons above it are
  // marked in. "2 decisions marked" would describe the UI; this describes the
  // files.
  const footLine = $derived(
    imports.waiting
      ? "Waiting for the scan to finish…"
      : imports.applying
      ? `${imports.active}…`
        : sum.decided === 0
        ? "Mark an album to apply"
        : sum.save === 0
          ? `${plural(sum.discard, "album")} to discard`
          : sum.discard === 0
            ? `${plural(sum.save, "album")} to save`
            : `${plural(sum.save, "album")} to save · ${plural(sum.discard, "album")} to discard`,
  );

  function onKeydown(e: KeyboardEvent) {
    if (!imports.open) return;
    if (e.key === "Tab") {
      // `aria-modal` promises the rest of the window is gone; the ring keeps focus
      // inside the panel that made the promise (App.svelte inertes the background).
      trapTab(e, panel);
      return;
    }
    if (e.key === "Escape") {
      requestClose();
      return;
    }
    if (e.key !== "ArrowDown" && e.key !== "ArrowUp") return;
    // Arrow-walk the controls, as the menu stack does. Restricted to the panel
    // so a focused element outside cannot be dragged in.
    const btns = [...(panel?.querySelectorAll<HTMLButtonElement>("button:not(:disabled)") ?? [])];
    const i = btns.indexOf(document.activeElement as HTMLButtonElement);
    if (i === -1) return;
    e.preventDefault();
    const next =
      e.key === "ArrowDown" ? Math.min(i + 1, btns.length - 1) : Math.max(i - 1, 0);
    btns[next]?.focus({ preventScroll: true });
  }

  // --- the file list's disclosure ---------------------------------------------
  //
  // The height has to be MEASURED. This webview reports
  // `CSS.supports("interpolate-size", "allow-keywords") === false`, so `height: 0 → auto`
  // cannot be transitioned in CSS and there is no clever-CSS version of this motion —
  // which is the same wall the album panel hit and for the same reason (see
  // ExpandedPanel: "Height is driven in px on .inner"). This borrows its two rules and
  // none of its machinery: px driven, `auto` at rest, and the exit ends on
  // `transitionend`, never on a timer.
  //
  // Deliberately local and deliberately not a primitive: one list, one motion, one file.
  const OPEN_MS = 200;
  const SHUT_MS = 150; // the same path, the shorter beat — leaving is the system response

  function disc(node: HTMLElement, arg: { open: boolean }) {
    let open = arg.open;
    let settle: (() => void) | undefined;
    let guard: ReturnType<typeof setTimeout> | undefined;
    // The authored display value, read before this function ever touches the property.
    const restDisplay = getComputedStyle(node).display;

    function end() {
      if (guard) {
        clearTimeout(guard);
        guard = undefined;
      }
      const fn = settle;
      settle = undefined;
      fn?.();
    }

    function onTransitionEnd(e: TransitionEvent) {
      if (e.target === node && e.propertyName === "height") end();
    }

    /** Animate the box from its CURRENT height to `to` px. `auto` cannot interpolate,
     *  so the current value is pinned first; mid-flight that pin is the animated height,
     *  which is what makes a second click reverse instead of restarting. */
    function moveTo(from: number, to: number, ms: number, done: () => void) {
      if (guard) clearTimeout(guard);
      settle = done;
      node.style.transition = "none";
      node.style.height = `${from}px`;
      void node.offsetHeight; // commit the pin before aiming anywhere
      node.style.transition = `height ${ms}ms var(--ease-out), opacity ${ms}ms var(--ease-out)`;
      node.style.height = `${to}px`;
      // opacity is written HERE rather than from a CSS rule on purpose: an `opacity`
      // that changes via an attribute selector in the same batch as the `transition`
      // string gets no transition at all on this engine (measured with
      // `getAnimations()`), so the fade would silently snap while the box moved.
      node.style.opacity = to > 0 ? "1" : "0";
      guard = setTimeout(end, ms + 150);
    }

    const openHeight = () => {
      const prev = node.style.height;
      node.style.transition = "none";
      node.style.height = "auto";
      const px = node.offsetHeight;
      node.style.height = prev;
      return px;
    };

    const restOpen = () => {
      node.style.height = "auto";
      node.style.opacity = "1";
      // Unclip at rest: the rows are text, and a clipped box would cut an ellipsized
      // title's descender at the bottom edge. Clipping is only needed while moving.
      node.style.overflow = "visible";
      node.style.transition = "";
    };

    const restClosed = () => {
      // `display: none`, not `height: 0`: the card is a grid, and a zero-height item
      // still occupies row 2 — so the album would keep paying its 6px row gap and sit
      // that much away from its own collapsed summary forever. It also takes the rows
      // out of the accessibility tree, which is what `aria-expanded="false"` on the
      // summary claims, and out of the scroller's content box.
      node.style.display = "none";
      node.style.height = "0px";
      node.style.opacity = "0";
      node.style.overflow = "hidden";
      node.style.transition = "";
    };

    node.addEventListener("transitionend", onTransitionEnd);
    // Mount is a rest state, never a performance: the window opens with one album
    // unfolded, and that list must not replay its unfold under the window's own
    // entrance.
    if (open) restOpen();
    else restClosed();

    return {
      update(next: { open: boolean }) {
        if (next.open === open) return;
        open = next.open;
        if (open) {
          const wasHidden = node.style.display === "none";
          node.style.overflow = "hidden";
          if (wasHidden) {
            node.style.display = restDisplay;
            node.style.height = "0px";
            void node.offsetHeight;
          }
          const from = wasHidden ? 0 : node.offsetHeight;
          const to = openHeight();
          moveTo(from, to, OPEN_MS, restOpen);
        } else {
          node.style.overflow = "hidden";
          moveTo(node.offsetHeight, 0, SHUT_MS, restClosed);
        }
      },
      destroy() {
        node.removeEventListener("transitionend", onTransitionEnd);
        if (guard) clearTimeout(guard);
        settle = undefined;
      },
    };
  }

  // --- the outro ------------------------------------------------------------
  // Same shape as About.svelte: `out` starts the exit and `imports.open` stays true
  // until the animation has ended, so `modalOpen()` (which drives `inert`) and the
  // Escape ownership below keep telling the truth while the window is still on screen.
  //
  // Deferring `closeImportManager()` to the end is also what fixes its content: that
  // function clears `report`/`applied`/`failed` the moment it is called, so closing it
  // at the START of the exit would have faded out a window that had just been emptied
  // — reading "Nothing is waiting." in its last 190ms. Same call, one beat later.
  let out = $state(false);

  function requestClose() {
    if (imports.applying) return;
    if (out) {
      out = false;
      closeImportManager();
      return;
    }
    out = true;
  }

  function onOutroEnd(e: AnimationEvent) {
    // The scrim's animation is the longer of the pair (190ms against the panel's
    // 150ms), so the surface is removed once the dim has finished dissolving. The
    // keyframe names live in app.css, so they are not component-scoped and cannot
    // drift out from under this string.
    if (e.animationName !== "scrim-out") return;
    out = false;
    closeImportManager();
  }

  let panel = $state<HTMLElement | null>(null);

  // A door that opened this window AT an album (the panel's Imported badge,
  // an album menu): expand happens in the store; here the row is scrolled
  // into view and wears a short accent ring so the eye finds it in a tall
  // pile. The flag is consumed on read, so a later reopen never re-teleports
  // for a click the user already honoured. The ring itself is STATE
  // (`attn`), not a classList string — Svelte can only style what it can
  // see in the markup, and reactive state is the better mechanism anyway.
  let attn = $state<string | null>(null);
  $effect(() => {
    if (!imports.open || !imports.focus) return;
    const id = imports.focus;
    imports.focus = null;
    void tick().then(() => {
      const row = panel?.querySelector<HTMLElement>(
        `[data-album-id="${CSS.escape(id)}"]`,
      );
      if (!row) return;
      row.scrollIntoView({ block: "nearest" });
      attn = id;
      setTimeout(() => {
        if (attn === id) attn = null;
      }, 1600);
    });
  });
  // Closing hands focus back to the door it came from, unless the door itself is
  // gone (an emptied pile removes the row) — a keyboard user should not be dropped
  // at the top of the document for having finished a task.
  let wasOpen = false;
  $effect(() => {
    const open = imports.open;
    if (wasOpen && !open) {
      document.querySelector<HTMLElement>("[data-imports-door]")?.focus({ preventScroll: true });
    }
    wasOpen = open;
  });

  // The window fetches for itself when it is up with nothing in hand: the store
  // fills the plan on open, and a module reload that cleared the plan without
  // closing the window would otherwise leave it claiming nothing is waiting.
  let fetched = false;
  $effect(() => {
    if (imports.open && !fetched) {
      fetched = true;
      void refreshImportPlan();
    }
  });

  // Focus the first decision when the window opens, so the keyboard does not
  // have to cross the panel to get to work. preventScroll: the body is a scroll
  // port and would jump the list to the focused row.
  $effect(() => {
    if (!imports.open) return;
    requestAnimationFrame(() => {
      panel?.querySelector<HTMLButtonElement>("button.mi-disc")?.focus({ preventScroll: true });
    });
  });
</script>

<svelte:window onkeydown={onKeydown} />

{#if imports.open}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div
    class="mi-backdrop scrim"
    class:out
    role="presentation"
    onanimationend={onOutroEnd}
    onclick={(e) => {
      if (e.target === e.currentTarget) requestClose();
    }}
  >
    <!-- svelte-ignore a11y_no_noninteractive_element_to_interactive_role -->
    <section class="mi glass" bind:this={panel} role="dialog" aria-modal="true" aria-label="Imported music">
      <!-- The surface template: title left, dismissal right, one seam under them.
           h2 is flex:1, so the ✕ lands flush with the content's right edge. -->
      <header class="mi-head">
        <h2>Imported music</h2>
        <SurfaceClose label="Close" onclick={requestClose} />
      </header>

      {#if held.length || applyLines.length}
        <!-- The two statements this window exists to make, and the one place it must
             NOT be: below the fold. They used to render after the album cards, inside
             the scroller — measured at 389px shown against 864px of content, so an
             8-album pile opened with the report off-screen and the Apply receipt
             unreachable without a scroll. A band under the seam is outside the scroll
             region, so it is read or it is not there. Sentence case, no tracking: the
             app is speaking, which is not what an artist label looks like. -->
        <div class="mi-report">
          {#if applyLines.length}
            <div class="mi-said" role="status">
              <span class="mi-said-label">Just applied</span>
              {#each applyLines as line (line)}
                <p class="mi-said-line">{line}</p>
              {/each}
            </div>
          {/if}
          {#if held.length}
            <div class="mi-said" role="group" aria-label="Already in your library">
              <span class="mi-said-label">Already in your library</span>
              {#each heldShown as a (`${a.artist}|${a.title}`)}
                <p class="mi-said-line">
                  {a.artist} — {a.title} · {plural(a.tracks, "track")}
                </p>
              {/each}
              {#if heldMore > 0}
                <p class="mi-said-line" title={heldAll}>
                  and {heldMore} more
                </p>
              {/if}
              <p class="mi-said-note">
                Nothing was added and nothing was moved: you already own these.
              </p>
            </div>
          {/if}
        </div>
      {/if}

      <div class="mi-body">
        {#if groups.length === 0}
          <p class="mi-empty">
            {applyLines.length
              ? "Nothing is waiting."
              : "Nothing is waiting to be imported. Files you import will show up here with the folder they are about to move to."}
          </p>
        {:else}
          {#each groups as g (g.artist)}
            <div class="mi-group" role="group" aria-label={g.artist}>
              <div class="mi-glabel">{g.artist}</div>
              {#each g.albums as a (a.albumId)}
                {@const open = !!imports.expanded[a.albumId]}
                {@const decision = imports.decisions[a.albumId]}
                {@const dest = destinationLine(a, ui.musicFolders)}
                {@const numbers = trackNumbers(a)}
                <div class="mi-album" class:mi-attn={attn === a.albumId} data-album-id={a.albumId}>
                  <!-- ONE disclosure control per card: the summary is the target and the
                       chevron is an indicator inside it. It used to be TWO buttons (the
                       caret and the title) carrying the same `aria-expanded` — one state
                       announced twice, the arrow-walk stopping twice per album, and a
                       destination line that was dead to a click landing 6px below a live
                       title. Whole-card is safe here specifically because expanding is
                       READING: the verbs that commit are in the other grid column and
                       outside the expander, so there is no mis-click to guard against.
                       The file list is a SIBLING of the button, never its child — a
                       button containing the list it discloses swallows every track title
                       into its accessible name. -->
                  <button
                    class="mi-disc"
                    aria-expanded={open}
                    disabled={imports.applying}
                    onclick={() => toggleImportTracks(a.albumId)}
                  >
                    <span class="mi-row">
                      <span class="mi-caret" aria-hidden="true">{open ? "▾" : "▸"}</span>
                      <span class="mi-name">
                        <span class="mi-title">{a.title}</span>
                        <span class="mi-meta">
                          {a.year ?? "—"} · {plural(a.tracks.length, "track")}
                        </span>
                      </span>
                    </span>

                    <span class="mi-dest" title={a.destination.folder}>
                      <span class="mi-rule">{dest.lead}</span>
                      <!-- The two claims are different kinds of statement (what will
                           happen / where), and 6px of gap did not say that. A middle
                           dot, the same separator the meta line and the footer
                           summary already use. Hidden from assistive tech: it is
                           punctuation for the eye, and a screen reader has the two
                           spans in their own order. -->
                      <span class="mi-sep" aria-hidden="true">·</span>
                      <span class="mi-path">{dest.path}</span>
                    </span>
                  </button>

                  <ul class="mi-tracks" use:disc={{ open }}>
                    {#each a.tracks as t, i (t.id)}
                      <li class="mi-track">
                        <span class="mi-num">{numbers[i]}</span>
                        <span class="mi-tname">{t.title}</span>
                        <span class="mi-tdur">{fmtDuration(t.durationSec)}</span>
                      </li>
                    {/each}
                  </ul>

                  <!-- Row 1, column 2 — see .mi-album: the buttons belong to the
                       summary, not to the card's height, so expanding a long album cannot
                       move them out from under a cursor that is on one. -->
                  <div class="mi-decide" role="group" aria-label="Decision for {a.title}">
                    <button
                      class="mi-mark mi-save"
                      aria-pressed={decision === "save"}
                      disabled={busy}
                      onclick={() => markImport(a.albumId, "save")}
                    >
                      Save
                    </button>
                    <button
                      class="mi-mark mi-drop"
                      aria-pressed={decision === "discard"}
                      disabled={busy}
                      onclick={() => markImport(a.albumId, "discard")}
                    >
                      Discard
                    </button>
                  </div>
                </div>
              {/each}
            </div>
          {/each}
        {/if}
      </div>

      <footer class="mi-foot">
        <div class="mi-sum">
          {#if sum.albums > 0}
            <span class="mi-counts"
              >{plural(sum.albums, "album")} · {plural(sum.tracks, "track")}</span
            >
          {/if}
          <span class="mi-line" aria-live="polite">
            {#if imports.applying}
              <ProgressRing value={progress} label={footLine} phase="apply" />
            {/if}
            <span>{footLine}</span>
          </span>
          {#if imports.failed.length}
            <span class="mi-failed" role="alert">
              Didn’t move: {imports.failed.join(", ")}
            </span>
          {/if}
        </div>

        <div class="mi-actions">
          <button
            class="mi-quiet"
            disabled={busy || groups.length === 0}
            onclick={() => markAllImports("save")}
          >
            Save all
          </button>
          <button
            class="mi-quiet"
            disabled={busy || groups.length === 0}
            onclick={() => markAllImports("discard")}
          >
            Discard all
          </button>
          <button
            class="mi-apply"
            disabled={busy || sum.decided === 0}
            aria-label={sum.decided > 0 ? `Apply ${sum.decided} decisions` : "Apply"}
            title={sum.decided === 0 ? "Mark an album first" : undefined}
            onclick={() => void applyImportDecisions()}
          >
            Apply
          </button>
        </div>
      </footer>
    </section>
  </div>
{/if}

<style>
  /* Same glass tier, radius and structure as the Music folders modal: two modal
     shapes in one app is one too many. */
  .mi {
    width: min(620px, calc(100vw - 80px));
    /* ALWAYS this box — height is not content-driven any more.
       It used to be `max-height`, which made the window hug its pile: two albums gave
       a short card, thirty gave a tall one, and the report band appearing or a row
       expanding moved the floor under the buttons the cursor was sitting on. A fixed
       frame also means the dismissal, the seam and the footer sit at the same
       coordinates on every open, and the only thing that flexes is the scroller. */
    height: calc(100vh - 140px);
    display: flex;
    flex-direction: column;
    padding: 16px;
    border-radius: var(--radius-panel);
    border: 1px solid var(--border);
    background: var(--panel-bg-strong);
    box-shadow: var(--shadow);
    color: var(--text);
  }

  .mi-head {
    display: flex;
    align-items: center;
    gap: 12px;
    padding-bottom: 10px;
    border-bottom: 1px solid var(--border);
  }

  .mi-head h2 {
    flex: 1;
    margin: 0;
    font-size: 15px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .mi-body {
    overflow-y: auto;
    /* The scroll port, now that the frame is fixed. `min-height: 0` is load-bearing:
       a flex item's automatic minimum is its content size, so without it a long pile
       pushes the frame open instead of scrolling inside it. */
    flex: 1 1 auto;
    min-height: 0;
    padding: 12px 0 0;
    display: flex;
    flex-direction: column;
    /* Artists are neighbours. With one card each, that is the only distance this
       window measures between groups. */
    gap: 12px;
  }

  /* One card PER ARTIST, with the albums as touching rows inside it — the
     grouped-list idiom the panes use for `.rows`, and the reason the spacing
     question in this tree went away. When each album was its own card, the 6px
     that bound an album to its artist label had to compete with the 12px that
     separates neighbours, and the reader had to decide which rung was which.
     Now the grouping is structural: the card IS the artist, the albums are its
     rows, and every gap in the window means one thing. */
  .mi-group {
    display: flex;
    flex-direction: column;
    background: var(--hover);
    border-radius: 8px;
    /* clip, not hidden: `hidden` makes the card a scroll port, and focusing a
       row inside it would scroll the card underneath the reader. */
    overflow: clip;
  }

  /* The artist is the card's header, not a caption floating above it: 10px of
     inset, then the 6px hug to the first row, with no divider — a divider would
     separate the artist from the albums it owns. */
  .mi-glabel {
    font-size: 11.5px;
    font-weight: 600;
    letter-spacing: 0.09em;
    text-transform: uppercase;
    color: var(--text-dim);
    padding: 10px 10px 6px;
  }

  /* The arrival ring: a row the window was opened AT announces itself once.
     box-shadow (not outline) because the row owns its own rounding and the
     ring must sit outside it without re-clipping the expander. */
  .mi-attn {
    border-radius: 8px;
    animation: mi-attn 1.6s var(--ease-out);
  }
  @keyframes mi-attn {
    0%,
    55% {
      box-shadow: 0 0 0 2px var(--accent);
    }
    100% {
      box-shadow: 0 0 0 2px transparent;
    }
  }

  .mi-album {
    display: grid;

    grid-template-columns: minmax(0, 1fr) auto;
    /* The line heights the decision column measures its band with: 13.5px of title in
       18, 11.5px of path in 15 — the list tier's leading, written down rather than
       inherited so the arithmetic below is arithmetic and not a guess. */
    --mi-lh-title: 18px;
    --mi-lh-dest: 15px;
    --mi-hug: 6px;
    /* The album is TWO rows now — the summary, then the file list — and the decision
       column lives in row 1. That is what lets Save/Discard be centred against the
       ALBUM (title line plus destination line, which is the thing they are decisions
       about) rather than dangling at the top of a card whose height changes when the
       list opens. It used to be one row with the list inside the left column, which
       got the same non-movement guarantee but only by pinning the buttons to the top
       edge of a two-line block. */
    align-items: start;
    column-gap: 10px;
    /* The ladder's hug, now measured between the rows: the file list belongs to its
       album exactly the way the destination line does. */
    row-gap: 6px;
    padding: 8px 10px;
  }

  /* Rows are divided, not spaced — the pane's `.rows` touch, made legible at
     this content height. */
  .mi-album + .mi-album {
    border-top: 1px solid var(--border);
  }

  /* The summary button — see the markup note for why the whole thing is the target.
     The negative margins give the wash and the focus ring breathing room while the TEXT
     stays on the card's own edge: a stretched grid item's used width is its column minus
     its margins, so -6px each side widens the box without moving what is in it. */
  .mi-disc {
    grid-area: 1 / 1;
    justify-self: stretch;
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-width: 0;
    margin: -2px -6px;
    padding: 2px 6px;
    border: none;
    border-radius: 7px;
    background: transparent;
    color: inherit;
    text-align: left;
    cursor: pointer;
  }

  .mi-disc:disabled {
    cursor: default;
  }

  /* A wash and an underline, no lift (the No-Lift rule) — and gated for pointer
     devices, because a tap would otherwise leave this row highlighted. The wash is
     `--hover`, the same rung every other control answers on; the region is large
     enough that anything stronger would out-shout the decision buttons beside it. */
  @media (hover: hover) and (pointer: fine) {
    .mi-disc:not(:disabled):hover {
      background: var(--hover);
    }

    .mi-disc:not(:disabled):hover .mi-title {
      text-decoration: underline;
      text-underline-offset: 2px;
    }
  }

  .mi-disc:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 3px;
    /* 8, the controls rung — 4 was off the shapes ladder. */
    border-radius: 8px;
  }

  .mi-row {
    display: flex;
    align-items: baseline;
    gap: 6px;
    min-width: 0;
    /* Declared, not inherited: the decision column measures its band from this line,
     * so the line has to have a height the stylesheet knows. */
    line-height: var(--mi-lh-title);
  }

  /* An indicator now, not a target: `aria-expanded` on the summary carries the state,
     so the glyph is hidden from assistive tech rather than announced a second time. */
  .mi-caret {
    flex: none;
    width: 14px;
    color: var(--text-dim);
    font-size: 10px;
    line-height: 1;
  }

  .mi-name {
    display: flex;
    align-items: baseline;
    gap: 8px;
    min-width: 0;
    color: var(--text);
  }

  .mi-title {
    min-width: 0;
    font-size: 13.5px;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .mi-meta {
    flex: none;
    font-size: 11.5px;
    color: var(--text-dim);
    font-variant-numeric: tabular-nums;
  }

  /* What will happen, then where. The rule is the sentence, the path is the
     evidence — and the EVIDENCE is the part that must survive: this row exists to
     answer "where will this end up", and a long album title used to cost the path
     everything but one character ("· M…"), with the full text only in a hover
     tooltip. So the gloss yields first and the path wraps instead of truncating. */
  .mi-dest {
    display: flex;
    flex-wrap: wrap;
    gap: 0 6px;
    min-width: 0;
    /* Lines up with the album title, not with the caret. */
    padding-left: 20px;
    font-size: 11.5px;
    /* Declared for the same reason as the title row's: the decision column's band is
       measured in these two line heights, so they cannot be left to the engine. */
    line-height: 15px;
    color: var(--text-dim);
  }

  .mi-rule {
    flex: 0 1 auto;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-style: italic;
  }

  /* Quieter than both sides: a joint, not a third statement. */
  .mi-sep {
    flex: none;
    opacity: 0.5;
  }

  .mi-path {
    flex: 1 1 auto;
    min-width: 0;
    /* Inter, not a mono face: DESIGN.md's type system is one voice with no pairing,
       and "technical" is not a reason to borrow a second family. A path that must
       break, breaks — `anywhere` only fires when one token exceeds the line. */
    overflow-wrap: anywhere;
    opacity: 0.85;
  }

  /* The file list. Its motion belongs to `disc()` above: the box's height and opacity
     are written inline by that action (px → px, `auto` at rest), so nothing animates
     here in CSS — and the `reveal-in` keyframe that used to live on this rule is gone,
     because a fade over a box that snaps to its new height is exactly the motion that
     reads as quick. */
  .mi-tracks {
    grid-area: 2 / 1;
    list-style: none;
    /* No margin: the card's 6px row gap IS the ladder's hug, and the list belongs to
       its album at that distance — the same 6px the destination line sits at. */
    margin: 0;
    padding: 0 0 0 20px;
    display: flex;
    flex-direction: column;
  }

  .mi-track {
    display: flex;
    align-items: baseline;
    gap: 8px;
    padding: 3px 0 3px 8px;
    font-size: 12px;
    color: var(--text-dim);
  }

  .mi-num {
    flex: none;
    min-width: 2ch;
    text-align: right;
    font-variant-numeric: tabular-nums;
    opacity: 0.7;
  }

  .mi-tname {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--text);
  }

  .mi-tdur {
    flex: none;
    font-variant-numeric: tabular-nums;
    opacity: 0.7;
  }

  .mi-decide {
    /* Row 1, column 2. The anchor is the CENTRE OF THE TITLE LINE AND THE FIRST LINE OF
       THE DESTINATION — not the centre of the summary block. The path wraps instead of
       truncating (the 0.6.0 fix), so a block-centred column would sit lower on every
       album whose path needs two lines, and the decision column would arrive ragged.
       So: a band of exactly title (18) + the summary's own gap (6) + one destination
       line (15), pinned to the top of the row, with the buttons centred inside it.
       Expansion is safe for the same reason as before — the list is row 2, below. */
    grid-area: 1 / 2;
    align-self: start;
    /* Title line + the summary's own hug + ONE destination line, measured from the top
     * of the row. Written as three named values so the band is visibly the same three
     * distances the layout uses above it. */
    height: calc(var(--mi-lh-title) + var(--mi-hug) + var(--mi-lh-dest));
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .mi-mark {
    padding: 5px 11px;
    border-radius: 7px;
    border: 1px solid var(--border);
    background: transparent;
    color: var(--text-dim);
    font-size: 12px;
    cursor: pointer;
    transition: background 140ms var(--ease-out), color 140ms var(--ease-out),
      border-color 140ms var(--ease-out);
  }

  .mi-mark:hover:not(:disabled) {
    background: var(--hover);
    color: var(--text);
  }

  .mi-mark:disabled {
    cursor: default;
    opacity: 0.6;
  }

  .mi-mark:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  /* A mark is the affirmative choice, so it takes the accent — the same word
     the footer's primary verb will act on. */
  .mi-save[aria-pressed="true"] {
    background: var(--accent);
    border-color: var(--accent);
    color: var(--accent-text, #fff);
    font-weight: 600;
  }

  /* The destructive mark takes the system's one caution hue (the same token the
     missing-file glyphs wear) — not the undocumented salmon red this window and the
     Music folders modal were each improvising. Hue is never the only cue: border,
     fill and weight change with it. */
  .mi-drop[aria-pressed="true"] {
    background: var(--caution-wash);
    border-color: var(--caution-line);
    color: var(--caution);
    font-weight: 600;
  }

  /* The receipt: quieter than the pile, because it reports a decision that was
     already obvious to the app and to nobody else. */
  .mi-report {
    flex: none;
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 10px 0 0;
  }

  .mi-said {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  /* Deliberately NOT `.mi-glabel`: uppercase + tracking is what an artist name
     looks like here, and a heading in that voice under the last artist card reads
     as one more artist called "Already in your library". */
  .mi-said-label {
    font-size: 12.5px;
    font-weight: 600;
    color: var(--text);
  }

  .mi-said-line {
    margin: 0;
    font-size: 12.5px;
    color: var(--text-dim);
  }

  .mi-said-note {
    margin: 2px 0 0;
    font-size: 11.5px;
    color: var(--text-dim);
    opacity: 0.85;
  }

  .mi-foot {
    /* `flex: none` for the same reason the scroller got `min-height: 0`: inside a fixed
       frame something has to give, and it must be the list, not the buttons. */
    flex: none;
    margin-top: 12px;
    padding-top: 12px;
    border-top: 1px solid var(--border);
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .mi-sum {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 3px;
    font-size: 12px;
    color: var(--text-dim);
  }

  .mi-counts {
    font-variant-numeric: tabular-nums;
  }

  .mi-line {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .mi-failed {
    color: var(--caution);
  }

  .mi-actions {
    display: flex;
    gap: 6px;
  }

  .mi-quiet {
    padding: 6px 10px;
    border-radius: 7px;
    border: 1px solid transparent;
    background: transparent;
    color: var(--text-dim);
    font-size: 12px;
    cursor: pointer;
  }

  .mi-quiet:hover:not(:disabled) {
    background: var(--hover);
    color: var(--text);
  }

  .mi-quiet:disabled {
    cursor: default;
    opacity: 0.6;
  }

  .mi-quiet:focus-visible,
  .mi-apply:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  .mi-apply {
    padding: 7px 14px;
    border: 1px solid var(--accent);
    border-radius: 8px;
    background: var(--accent);
    color: var(--accent-text, #fff);
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
  }

  .mi-apply:disabled {
    opacity: 0.55;
    cursor: default;
  }
</style>
