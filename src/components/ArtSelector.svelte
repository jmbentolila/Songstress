<script lang="ts">
  /**
   * The artwork chooser (Tag Editor Redesign spec): every image the album
   * actually owns is a tile — distinct embedded pictures with their census,
   * folder-art files by name — plus your own via kdialog, clipboard (Ctrl+V)
   * or drag-drop. Choosing is an instruction, and the ring shows which one
   * the save will carry out. Tiles answer on pointer-down; the ring is the
   * same accent language as the sliders and focus rings.
   */
  import { invoke } from "@tauri-apps/api/core";
  import { fade } from "svelte/transition";
  import { MediaQuery } from "svelte/reactivity";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import type { ArtChange, ArtInventory } from "../lib/artChange";
  import { sniffMime, toBase64 } from "../lib/artChange";
  import { tooltip } from "../lib/tooltip";

  let {
    albumId,
    trackId = undefined,
    refreshSeq = 0,
    stack = false,
    open = $bindable(false),
    change = $bindable("keep" as ArtChange),
    inventory = $bindable(null as ArtInventory | null),
  }: {
    albumId: string;
    trackId?: string;
    /** The host bumps this after a write that can change the census
     *  (saving this album's artwork): the picker refreshes SILENTLY —
     *  tiles never trade places with the skeleton over content the user
     *  is looking at. */
    refreshSeq?: number;
    /** Vertical column (two-column modal layout) instead of a strip. */
    stack?: boolean;
    /** Lightbox visibility, shared with the host modal so Escape closes the
     *  lightbox before it closes the dialog. */
    open?: boolean;
    change?: ArtChange;
    inventory?: ArtInventory | null;
  } = $props();

  let loading = $state(true);
  let error = $state("");
  /** Images brought in this session (not yet in any file): one tile each. */
  let arrivals = $state<{ image: string; mime: string; url: string }[]>([]);
  let dropOver = $state(false);
  let busy = $state(false);

  async function load(quiet = false) {
    // A refresh keeps the tiles standing (no skeleton over content that is
    // still true); the skeleton is ONLY for the first load of an album.
    if (!quiet) loading = true;
    error = "";
    try {
      const next = await invoke<ArtInventory>("get_art_candidates", {
        albumId,
        trackId: trackId ?? null,
      });
      inventory = next;
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }
  // The inventory is the ALBUM'S picture census; trackId is request
  // context only. Read the signals OUTSIDE the effect so stepping the
  // stepper does not register trackId as a dependency: it used to, and
  // every step / save re-fetched and swapped the candidate tiles for
  // skeletons — the "modal glitches its dimensions and returns" the
  // owner reports (2026-09-05), one shimmer per track.
  let loadAlbum = $state(""); // albumId this inventory was fetched for
  let reloadSeq = $state(0); // last refreshSeq consumed
  $effect(() => {
    const a = albumId;
    if (!a) return;
    if (a !== loadAlbum) {
      loadAlbum = a;
      void load();
    }
  });
  $effect(() => {
    const s = refreshSeq;
    if (!s || s === reloadSeq) return;
    reloadSeq = s;
    void load(true);
  });

  const selectedHash = $derived(
    typeof change === "string" ? null : "hash" in change ? change.hash : null,
  );
  /** The picker's reading order: the cover that IS (or will be) worn goes
     first, then the rest in the server's order (most-worn first). The
     corner-dot idiom alone left "which one is the cover" a scavenger hunt
     on a 6-encoding compilation — position is the loudest ordering cue
     (owner request 2026-09-05). Stable: exactly one promotion, no ties.
     While the clear decision is up, nothing is worn, so no promotion. */
  const orderedCandidates = $derived.by(() => {
    const list = inventory?.candidates ?? [];
    if (cleared) return list;
    const winner = selectedHash ?? (uploaded ? null : inventory?.current ?? null);
    if (!winner) return list;
    const i = list.findIndex((c) => c.hash === winner);
    if (i <= 0) return list;
    return [list[i], ...list.slice(0, i), ...list.slice(i + 1)];
  });
  /** Reveal-on-load for tile images. A candidate preview is a real file
     read (and for a 218-file album, several), so tiles would otherwise
     pop in one by one after the inventory arrives — the SECOND late
     phase the owner saw on Anison. The tile box is always there (its bg
     is the dedicated placeholder token); the image fades in when it's
     actually painted. Cache-safe: `complete` is checked at connect, so a
     cached image is revealed on the same frame it mounts. */
  function reveal(node: HTMLImageElement) {
    const done = () => node.classList.add("as-load");
    if (node.complete && node.naturalWidth > 0) done();
    else {
      node.addEventListener("load", done);
      node.addEventListener("error", done); // a broken src must not ghost forever
    }
  }
  const cleared = $derived(change === "clear");
  const uploaded = $derived(change !== "keep" && change !== "clear" && "upload" in change);

  /** Lightbox subject: { src, label, hash? } — hash absent for arrivals. */
  let view = $state<{ src: string; label: string; hash?: string } | null>(null);
  // The lightbox entered on a 140 ms ease-out fade; the mirror exit is
  // now the SAME transition played backwards (owner rule: entrances owe a
  // mirror), which a keyframe could never do. 140 ms, reduced-motion to
  // zero — JS-driven, so the stylesheet switch can't reach it.
  const prefersReducedMotion = new MediaQuery("(prefers-reduced-motion: reduce)");
  const LB_FADE = $derived({ duration: prefersReducedMotion.current ? 0 : 140 });
  let viewHash = $state<string | null>(null);
  function expand(v: { src: string; label: string; hash?: string }) {
    view = v;
    viewHash = v.hash ?? null;
    open = true;
  }
  function closeLb() {
    view = null;
    open = false;
  }
  // The host may close the lightbox itself (first Escape belongs to it).
  $effect(() => {
    if (!open) view = null;
  });

  function pick(hash: string) {
    arrivals = [];
    change = hash === (inventory?.current ?? "") || hash === inventory?.current ? "keep" : { hash };
  }
  function adopt(arrival: { image: string; mime: string; url: string }) {
    arrivals = [arrival, ...arrivals.filter((a) => a.image !== arrival.image)];
    change = { upload: { image: arrival.image, mime: arrival.mime } };
  }

  async function fromPaths(paths: string[]) {
    // Audio extensions join the accepted set (owner ask 2026-09-05): the
    // command reads an AUDIO pick's largest EMBEDDED picture, so "use the
    // cover this mp3 already carries" is one click — no detour through a
    // tag editor. Audio without embedded art says so plainly (the error
    // line is the surface for it).
    const img = paths.find((p) => /\.(png|jpe?g|webp|gif|tiff?|mp3|flac|m4a|aiff?|ogg|oga|opus|wav)$/i.test(p));
    if (!img) return;
    busy = true;
    error = "";
    try {
      const b64 = await invoke<string>("read_image", { path: img });
      adopt({ image: b64, mime: guess(b64), url: dataUrl(b64, guess(b64)) });
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }
  function guess(b64: string): string {
    const bin = atob(b64.slice(0, 24));
    const bytes = Uint8Array.from(bin, (c) => c.charCodeAt(0));
    return sniffMime(bytes) || "image/jpeg";
  }
  const dataUrl = (b64: string, mime: string) => `data:${mime};base64,${b64}`;

  async function browse() {
    busy = true;
    try {
      const path = await invoke<string | null>("pick_image");
      if (path) await fromPaths([path]);
    } finally {
      busy = false;
    }
  }

  function onPaste(e: ClipboardEvent) {
    const file = Array.from(e.clipboardData?.files ?? []).find((f) => f.type.startsWith("image/"));
    if (!file) return;
    busy = true;
    void file.arrayBuffer().then((buf) => {
      const bytes = new Uint8Array(buf);
      const mime = sniffMime(bytes) || file.type;
      adopt({ image: toBase64(bytes), mime, url: dataUrl(toBase64(bytes), mime) });
      busy = false;
    });
  }

  $effect(() => {
    const un = getCurrentWebview()
      .onDragDropEvent((e) => {
        if (e.payload.type === "enter" || e.payload.type === "over") dropOver = true;
        else if (e.payload.type === "leave") dropOver = false;
        else if (e.payload.type === "drop") {
          dropOver = false;
          void fromPaths(e.payload.paths);
        }
      })
      .catch(() => null);
    return () => void un.then((f) => f?.());
  });
</script>

<svelte:window onpaste={onPaste} />

<div class="as" class:as-drop={dropOver}>
  <div class="as-head">
    <span class="as-title">Artwork</span>
    {#if cleared}
      <button class="as-link" onclick={() => (change = "keep")}>Keep artwork</button>
    {/if}
  </div>

  {#if loading}
    <div class="as-strip" class:as-stack={stack}>
      <!-- Stack: ONE placeholder, sized exactly like a real stacked tile —
           three of them made the loading column TALLER than the finished
           picker (measured 749 → 525 shrink on open), which is the
           inflate-then-shrink jump the owner ruled against (2026-09-05):
           the loader sits in the standard box and content may only
           EXPAND it. Strip: the row scrolls sideways, skeletons there
           cost no height. -->
      {#each stack ? [0] : [0, 1, 2] as i (i)}
        <div class="as-tile as-sk"></div>
      {/each}
    </div>
  {:else if error}
    <p class="as-error">{error}</p>
  {:else}
    <div class="as-strip" class:as-stack={stack}>
      {#each arrivals as a (a.image)}
        <button
          class="as-tile as-sel"
          onclick={() => expand({ src: a.url, label: "New image" })}
          use:tooltip={"Click to see the full image"}
        >
          <img src={a.url} alt="" decoding="async" use:reveal />
          <span
            class="as-badge as-sel-badge"
            role="checkbox"
            aria-checked="true"
            aria-label="This is the cover"
            tabindex="0"
            use:tooltip={"This is the cover"}
            onclick={(e) => e.stopPropagation()}
            onkeydown={(e) => e.key === "Enter" && e.stopPropagation()}
            >✓</span
          >
          <span class="as-cap">New image</span>
        </button>
      {/each}
      {#each orderedCandidates as c (c.hash)}
        {@const isSel =
          !cleared &&
          (selectedHash
            ? selectedHash === c.hash
            : c.hash === inventory?.current && !uploaded)}
        <!-- The image this decision removes. GHOSTING USED TO FALL ON
             EVERY TILE, which read as "the whole picker broke" when the
             truth is "THE cover goes away, these others are still here"
             — and they are: clicking any of them replaces the decision.
             The ghost is now the single victim; the others stay full
             contrast so the recovery path is the loudest thing left
             (owner confusion 2026-09-05). -->
        {@const willRm = cleared && c.hash === inventory?.current}
        <button
          class="as-tile"
          class:as-sel={isSel}
          class:as-ghost={willRm}
          onclick={() =>
            expand({
              src: c.full,
              // The lightbox caption is a sentence, not a tile label.
              label:
                c.count > 0
                  ? `Found in ${c.count} ${c.count === 1 ? "file" : "files"}`
                  : c.label,
              hash: c.hash,
            })}
          use:tooltip={"Click to see the full image"}
        >
          <img src={c.preview} alt="" decoding="async" use:reveal />
          <!-- selection: the corner dot (iOS photo-picker language) -->
          <span
            class="as-badge"
            class:as-badge-on={isSel}
            role="checkbox"
            aria-checked={isSel}
            aria-label={isSel ? "This is the cover" : "Use as cover"}
            tabindex="0"
            use:tooltip={isSel ? "This is the cover" : "Use as cover"}
            onclick={(e) => {
              e.stopPropagation();
              pick(c.hash);
            }}
            onkeydown={(e) => {
              if (e.key === "Enter" || e.key === " ") {
                e.stopPropagation();
                pick(c.hash);
              }
            }}
            >{isSel ? "✓" : ""}</span
          >
          <!-- removal lives ON the image it dismisses, not beside the strip -->
          {#if isSel && !cleared}
            <span
              class="as-rm"
              role="button"
              tabindex="0"
              use:tooltip={"Remove artwork from every file"}
              onclick={(e) => {
                e.stopPropagation();
                change = "clear";
              }}
              onkeydown={(e) => {
                if (e.key === "Enter" || e.key === " ") {
                  e.stopPropagation();
                  change = "clear";
                }
              }}
              >✕</span
            >
          {/if}
          <span class="as-cap" class:as-cap-rm={willRm}>{willRm ? "removing on Save" : c.count > 0 ? `in ${c.count} ${c.count === 1 ? "file" : "files"}` : c.label}</span>
        </button>
      {/each}
      <button class="as-tile as-add" onclick={() => void browse()} disabled={busy} use:tooltip={"Choose an image — or an audio file whose embedded cover to take"}>
        <svg viewBox="0 0 16 16" aria-hidden="true">
          <path d="M8 3v10M3 8h10" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
        </svg>
        <span class="as-cap">From disk…</span>
      </button>
    </div>
    {#if cleared}
      <p class="as-note">The artwork will be removed from every file{inventory?.folderArt ? ` and ${inventory.folderArt} deleted` : ""}.</p>
    {:else if inventory && inventory.candidates.length === 0}
      <p class="as-note">This album has no artwork yet — add one, or paste from the clipboard.</p>
    {:else}
      <p class="as-note">Click to enlarge · the dot marks the cover.</p>
    {/if}
  {/if}

  {#if view}
    <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
    <div class="as-lb" role="presentation" onclick={closeLb} transition:fade|local={LB_FADE}>
      <!-- svelte-ignore a11y_no_noninteractive_element_to_interactive_role -->
      <figure class="as-lbfig" role="dialog" aria-modal="true" aria-label={view.label} onclick={(e) => e.stopPropagation()}>
        <img src={view.src} alt={view.label} />
        <figcaption>
          <span>{view.label}</span>
          {#if cleared}
            <button class="as-btn" onclick={() => (change = "keep")}>Keep artwork</button>
          {:else if viewHash}
            {@const vh = viewHash}
            <button class="as-btn" onclick={() => (change = "clear")}>Remove artwork</button>
            {#if !(selectedHash ? selectedHash === vh : vh === inventory?.current && !uploaded)}
              <button class="as-btn as-btn-accent" onclick={() => pick(vh)}>Use as cover</button>
            {/if}
          {/if}
          <!-- Close is primary: more often than not, the image stays. -->
          <button class="as-btn primary" onclick={closeLb}>Close</button>
        </figcaption>
      </figure>
    </div>
  {/if}
</div>

<style>
  .as {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 10px;
    border: 1px dashed transparent;
    border-radius: 10px;
  }
  .as-drop {
    border-color: var(--accent);
    background: color-mix(in srgb, var(--accent) 8%, transparent);
  }
  .as-head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 12px;
  }
  .as-title {
    font-size: 14px;
    color: var(--text-dim);
  }
  .as-link {
    border: 0;
    background: none;
    padding: 0;
    font: inherit;
    font-size: 13px;
    color: var(--accent);
    cursor: pointer;
  }
  .as-link:hover {
    text-decoration: underline;
  }
  .as-strip {
    display: flex;
    gap: 8px;
    overflow-x: auto;
    padding-bottom: 2px;
  }
  /* Two-column modal: candidates stack, tiles grow into real previews —
     one or two images deserve to be seen, not thumbnail-queued. */
  .as-stack {
    flex-direction: column;
    overflow-x: visible;
  }
  .as-stack .as-tile,
  .as-stack .as-sk {
    width: 100%;
    height: auto;
  }
  /* The stack skeleton had height:auto via the rule above — i.e. ZERO
     height, an invisible loader in exactly the modal (album, stacked
     column) whose slow inventory fetch needs it most (Anison: 218 files
     parsed per open). Match the real stack tile: img room + caption. */
  .as-stack .as-sk {
    height: 172px;
  }
  .as-stack .as-tile img {
    width: 100%;
    height: 148px;
  }
  .as-stack .as-add {
    height: 96px;
  }
  .as-tile {
    position: relative;
    flex: none;
    width: 76px;
    padding: 0;
    border: 1px solid var(--border);
    border-radius: 8px;
    /* The placeholder material, not --hover: app.css spells out that a
       placeholder is a fourth thing, tuned for sitting quiet-but-present,
       and the sk tokens were made for exactly that job. */
    background: var(--sk-base);
    color: inherit;
    cursor: pointer;
    overflow: hidden;
    transition: box-shadow 120ms var(--ease-out, ease-out), border-color 120ms ease-out;
  }
  .as-tile img {
    display: block;
    width: 76px;
    height: 76px;
    object-fit: cover;
    border-radius: 7px 7px 0 0;
    /* Late images arrive INTO the placeholder instead of popping: the
       reveal-on-load action adds as-load when the bytes are painted.
       No motion preference is honored here on purpose — an opacity
       fade is the calmest possible event; the jarring one is a pop. */
    opacity: 0;
    transition: opacity 220ms ease-out;
  }
  /* :global() because the class is added by the reveal ACTION, invisible
     to the scoped-CSS scanner. */
  .as-tile :global(img.as-load) {
    opacity: 1;
  }
  .as-tile:hover:not(:disabled) {
    border-color: color-mix(in srgb, var(--accent) 55%, var(--border));
  }
  .as-tile:active {
    transform: scale(0.97);
  }
  .as-sel {
    border-color: var(--accent);
    box-shadow: 0 0 0 2px var(--accent);
  }
  .as-ghost img {
    opacity: 0.45;
  }
  /* The victim says so itself, in the caution hue — the strip-wide ghost
     used to do the shouting with no subject. */
  .as-cap-rm {
    color: var(--caution);
  }
  .as-cap {
    display: block;
    padding: 3px 4px 4px;
    font-size: 11.5px;
    line-height: 1.2;
    color: var(--text-dim);
    text-align: center;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .as-add {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 2px;
    height: 95px;
    color: var(--text-dim);
    border-style: dashed;
  }
  .as-add svg {
    width: 18px;
    height: 18px;
  }
  .as-add .as-cap {
    padding: 0 4px 6px;
  }
  .as-add:disabled {
    opacity: 0.5;
    cursor: default;
  }
  .as-sk {
    height: 95px;
    background: var(--sk-base);
    animation: as-pulse 1.1s ease-in-out infinite;
  }
  @keyframes as-pulse {
    50% {
      opacity: 0.45;
    }
  }
  .as-note {
    margin: 0;
    font-size: 13px;
    color: var(--text-dim);
  }
  .as-error {
    margin: 0;
    font-size: 14px;
    color: var(--caution);
  }

  /* Corner dot: the chooser. iOS photo-picker language — the image itself
     opens the full view, the dot decides the cover. */
  .as-badge {
    position: absolute;
    top: 6px;
    left: 6px;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    border: 1.5px solid rgba(255, 255, 255, 0.85);
    background: rgba(0, 0, 0, 0.35);
    color: var(--on-cover);
    font-size: 14px;
    line-height: 17px;
    text-align: center;
    cursor: pointer;
    transition:
      background 120ms ease-out,
      transform 100ms ease-out;
  }
  .as-badge:hover {
    background: rgba(0, 0, 0, 0.55);
  }
  .as-badge:active {
    transform: scale(0.88);
  }
  .as-badge-on,
  .as-sel-badge {
    /* The window-controls GREEN (--tb-a = the KWin decoration palette's
       maximize button, read live by decoration.svelte.ts), not the accent:
       "chosen" is a state, not a brand, and green is the color the eye
       already owns for "this one" (owner ruling). */
    background: var(--tb-a, var(--accent));
    border-color: var(--tb-a, var(--accent));
  }

  /* Removal, ON the image it dismisses (only the chosen one carries it). */
  .as-rm {
    position: absolute;
    top: 6px;
    right: 6px;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: rgba(0, 0, 0, 0.45);
    color: var(--on-cover);
    font-size: 13px;
    line-height: 19px;
    text-align: center;
    cursor: pointer;
    opacity: 0;
    transition: opacity 120ms ease-out;
  }
  .as-tile:hover .as-rm,
  .as-rm:focus-visible {
    opacity: 1;
  }
  .as-rm:hover {
    background: var(--caution);
  }

  /* Lightbox: the whole image, on dim, inside this dialog (not a second
     scrim — it rides the modal's). */
  .as-lb {
    position: fixed;
    inset: 0;
    z-index: 1200;
    display: grid;
    place-items: center;
    background: rgba(0, 0, 0, 0.7);
  }
  /* The lightbox is a SIBLING of the dialog it belongs to: same radius
     token, same border, same panel material — a card, not a floating
     rectangle. The clip lives on the card (overflow), because a replaced
     img may paint its content over its own border-radius on WebKitGTK. */
  .as-lbfig {
    margin: 0;
    /* The card is a mat, not a cage: the IMAGE carries the definite size,
       the container grows to fit it plus the padding — equal on all four
       sides, since a frame padded only sideways reads as a bug. */
    width: fit-content;
    padding: 12px;
    display: flex;
    flex-direction: column;
    /* The row sits inside the mat at the mat's own measure — one rhythm,
       the image above as far from the row as the row is from the edge.
       That measure is 13, not 12: content-to-border reads as padding PLUS
       the border's own pixel (probed: row→outer bottom = 13), so a 12px
       gap sat a pixel tight against every other side's spacing. */
    gap: 13px;
    border: 1px solid var(--border);
    border-radius: var(--radius-panel);
    background: var(--panel-bg-strong);
    box-shadow: var(--shadow);
    overflow: hidden;
  }
  .as-lbfig img {
    display: block;
    /* The clamp subtracts the card's REAL overhead (padding 24, caption ~46,
       a little outer breathing) — not an invented margin, which would shrink
       the image in short windows for no reason. */
    width: min(384px, calc(100vw - 164px), calc(100vh - 140px));
    height: auto;
    max-height: calc(100vh - 140px);
    object-fit: contain;
  }
  .as-lbfig figcaption {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 8px;
    font-size: 13px;
    color: var(--text-dim);
  }
  .as-lbfig figcaption span {
    flex: 1;
    min-width: 0;
    text-align: left;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .as-btn {
    padding: 7px 14px;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: transparent;
    color: var(--text);
    font-size: 15px;
    cursor: pointer;
  }
  .as-btn:hover {
    background: var(--hover);
  }
  .as-btn.primary {
    /* Solid accent, not the panel's translucent wash: over the scrim a
       see-through primary reads as disabled, and Close is the opposite. */
    background: var(--accent);
    color: var(--accent-text, #fff);
    border-color: transparent;
    font-weight: 600;
  }
  .as-btn.primary:hover {
    background: color-mix(in srgb, var(--accent) 88%, white);
    color: var(--accent-text, #fff);
  }
  /* The verdict button when it is not the expected path: present, accented,
     but one weight below Close. */
  .as-btn-accent {
    color: var(--accent);
    border-color: color-mix(in srgb, var(--accent) 45%, var(--border));
  }
  .as-btn-accent:hover {
    background: color-mix(in srgb, var(--accent) 12%, var(--hover));
  }
</style>
