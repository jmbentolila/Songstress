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
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import type { ArtChange, ArtInventory } from "../lib/artChange";
  import { sniffMime, toBase64 } from "../lib/artChange";

  let {
    albumId,
    trackId = undefined,
    stack = false,
    open = $bindable(false),
    change = $bindable("keep" as ArtChange),
    inventory = $bindable(null as ArtInventory | null),
  }: {
    albumId: string;
    trackId?: string;
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

  async function load() {
    loading = true;
    error = "";
    try {
      inventory = await invoke<ArtInventory>("get_art_candidates", {
        albumId,
        trackId: trackId ?? null,
      });
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }
  $effect(() => {
    if (albumId) void load();
  });

  const selectedHash = $derived(
    typeof change === "string" ? null : "hash" in change ? change.hash : null,
  );
  const cleared = $derived(change === "clear");
  const uploaded = $derived(change !== "keep" && change !== "clear" && "upload" in change);

  /** Lightbox subject: { src, label, hash? } — hash absent for arrivals. */
  let view = $state<{ src: string; label: string; hash?: string } | null>(null);
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
    const img = paths.find((p) => /\.(png|jpe?g|webp|gif|tiff?)$/i.test(p));
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
      {#each [0, 1, 2] as i (i)}
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
          title="Click to see the full image"
        >
          <img src={a.url} alt="" decoding="async" />
          <span
            class="as-badge as-sel-badge"
            role="checkbox"
            aria-checked="true"
            tabindex="0"
            title="This is the cover"
            onclick={(e) => e.stopPropagation()}
            onkeydown={(e) => e.key === "Enter" && e.stopPropagation()}
            >✓</span
          >
          <span class="as-cap">New image</span>
        </button>
      {/each}
      {#each inventory?.candidates ?? [] as c (c.hash)}
        {@const isSel =
          !cleared &&
          (selectedHash
            ? selectedHash === c.hash
            : c.hash === inventory?.current && !uploaded)}
        <button
          class="as-tile"
          class:as-sel={isSel}
          class:as-ghost={cleared}
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
          title="Click to see the full image"
        >
          <img src={c.preview} alt="" decoding="async" />
          <!-- selection: the corner dot (iOS photo-picker language) -->
          <span
            class="as-badge"
            class:as-badge-on={isSel}
            role="checkbox"
            aria-checked={isSel}
            tabindex="0"
            title={isSel ? "This is the cover" : "Use as cover"}
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
              title="Remove artwork from every file"
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
          <span class="as-cap">{c.count > 0 ? `in ${c.count} ${c.count === 1 ? "file" : "files"}` : c.label}</span>
        </button>
      {/each}
      <button class="as-tile as-add" onclick={() => void browse()} disabled={busy} title="Choose an image from disk">
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
    <div class="as-lb" role="presentation" onclick={closeLb}>
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
    font-size: 12px;
    color: var(--text-dim);
  }
  .as-link {
    border: 0;
    background: none;
    padding: 0;
    font: inherit;
    font-size: 11px;
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
    background: var(--hover);
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
  .as-cap {
    display: block;
    padding: 3px 4px 4px;
    font-size: 9.5px;
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
    animation: as-pulse 1.1s ease-in-out infinite;
  }
  @keyframes as-pulse {
    50% {
      opacity: 0.45;
    }
  }
  .as-note {
    margin: 0;
    font-size: 11px;
    color: var(--text-dim);
  }
  .as-error {
    margin: 0;
    font-size: 12px;
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
    font-size: 12px;
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
    font-size: 11px;
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
    animation: as-lb-in 140ms var(--ease-out, ease-out);
  }
  @keyframes as-lb-in {
    from {
      opacity: 0;
    }
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
    font-size: 11px;
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
    font-size: 13px;
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
