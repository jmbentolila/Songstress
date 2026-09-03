<script lang="ts">
  /**
   * The ALBUM tag modal (Tag Editor Redesign, Phase B split into its own
   * file by Phase C). A bulk surface: the census drives everything, chips
   * adopt the values the files disagree with, the footer states the blast
   * radius BEFORE the press and the receipt prints the exact truth after.
   * The promise: fields you did not touch are never written.
   */
  import { invoke } from "@tauri-apps/api/core";
  import { slide } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import { prefersReducedMotion } from "svelte/motion";
  import { library } from "../lib/stores/library.svelte";
  import { ui } from "../lib/stores/ui.svelte";
  import { rescan } from "../lib/stores/scanner.svelte";
  import TagSurface from "./TagSurface.svelte";
  import FieldGrid from "./FieldGrid.svelte";
  import ArtSelector from "./ArtSelector.svelte";
  import type { ArtChange, ArtInventory } from "../lib/artChange";
  import {
    NUM_LABELS,
    SHARED,
    badFieldKeys,
    toEditable,
    strNum,
    type AlbumTagsApi,
    type Editable,
    type FieldCensus,
  } from "../lib/tagFields";

  let { albumId }: { albumId: string } = $props();

  let edit = $state<Editable>(toEditable({}));
  let disputed = $state<Record<string, boolean>>({});
  let loading = $state(true);
  let saving = $state(false);
  let error = $state("");

  // snapshotJson is $state FOR A REASON (blast): a plain let there was not
  // tracked, the first pass early-returned on it, and the derived therefore
  // never registered `touched` as a dependency — so it stayed null forever.
  let snapshotJson = $state("");
  let base = $state<Editable | null>(null);
  let census = $state<Record<string, FieldCensus>>({});
  let fileCount = $state(0);
  let inv = $state<ArtInventory | null>(null);
  let art = $state<ArtChange>("keep");
  let artLb = $state(false);
  /** Set after a successful save: the footer becomes the receipt. */
  let receipt = $state<{ written: number; total: number; lines: string[] } | null>(null);

  /** The same fold as the track modal (owner ruling 2026-09-03): Composer,
     Label, Grouping, Comment hide behind it — but the census promise holds:
     load() AUTO-OPENS the fold when any hidden field is disputed, so a
     disagreement can never sleep behind it unnoticed. */
  let moreOpen = $state(false);
  const MORE_KEYS = ["composer", "label", "grouping", "comment"] as const;
  let moreFilled = $derived.by(() => {
    const b = base;
    return b ? MORE_KEYS.filter((k) => b[k].trim() !== "").length : 0;
  });
  let moreEdited = $derived(
    !!base && snapshotJson !== "" && MORE_KEYS.some((k) => edit[k] !== base![k]),
  );

  async function load() {
    error = "";
    snapshotJson = "";
    receipt = null;
    art = "keep";
    moreOpen = false;
    const a = await invoke<AlbumTagsApi>("get_album_tags", { albumId });
    disputed = {
      albumArtist: a.albumArtistDisputed,
      album: a.albumDisputed,
      year: a.yearDisputed,
      genre: a.genreDisputed,
      composer: a.composerDisputed,
      label: a.labelDisputed,
      grouping: a.groupingDisputed,
      comment: a.commentDisputed,
      trackTotal: a.trackTotalDisputed,
      discTotal: a.discTotalDisputed,
    };
    census = Object.fromEntries(a.conflicts.map((c) => [c.field, c]));
    fileCount = a.fileCount;
    edit = toEditable({
      albumArtist: a.albumArtist,
      album: a.album,
      year: a.year,
      trackTotal: a.trackTotal,
      discTotal: a.discTotal,
      genre: a.genre,
      composer: a.composer,
      label: a.label,
      comment: a.comment,
      grouping: a.grouping,
    });
    snapshotJson = JSON.stringify(edit);
    base = { ...edit };
    // The census promise, on the census's OWN criterion: `conflicts` is the
    // full tally (every field carries an entry — presence counts included),
    // so "disputed" is the disputed flag, exactly what lights a chip. An
    // earlier draft opened the fold on mere membership and sprang it open
    // for every album in the library.
    if (MORE_KEYS.some((k) => disputed[k])) moreOpen = true;
  }

  $effect(() => {
    if (!ui.tagEditor.open) return;
    const id = albumId; // tracked: a re-point (rare while open) reloads
    loading = true;
    void load()
      .catch((e) => (error = String(e)))
      .finally(() => (loading = false));
    void id;
  });

  // Regroup edge: a save that merges/splits albums can make the edited album
  // vanish on reload. Close gracefully instead of editing a ghost.
  $effect(() => {
    if (!ui.tagEditor.open || !library.ready) return;
    if (albumId && !library.albums.some((a) => a.id === albumId)) {
      if (ui.expandedAlbum.songs === albumId) ui.expandedAlbum.songs = null;
      if (ui.expandedAlbum.albums === albumId) ui.expandedAlbum.albums = null;
      ui.tagEditor.open = false;
    }
  });

  let dirty = $derived(
    (JSON.stringify(edit) !== snapshotJson && snapshotJson !== "") || art !== "keep",
  );

  /** Fields the user actually edited. THE PROMISE: anything not in here is
   *  never written to any file. */
  let touched = $derived.by(() => {
    const t: Record<string, boolean> = {};
    if (base && snapshotJson)
      for (const k of SHARED) t[k] = String(edit[k]) !== String(base[k]);
    return t;
  });
  const artChanged = $derived(art !== "keep");

  const bad = $derived(badFieldKeys(edit));

  /** The blast radius, stated BEFORE the press: which fields disagree with
   *  the edit, summed against the census. A union across several fields
   *  cannot be exact without per-file rows, so it says "up to" — and the
   *  receipt afterward prints the exact truth. */
  let blast = $derived.by(() => {
    if (!snapshotJson || receipt) return null;
    const parts: number[] = [];
    for (const k of SHARED) {
      if (!touched[k]) continue;
      const c = census[k];
      if (!c) continue;
      const v = String(edit[k]).trim();
      const same =
        v === ""
          ? fileCount - c.present
          : (c.values.find((x) => x.value === v)?.count ?? 0);
      parts.push(fileCount - same);
    }
    let cover = 0;
    if (art === "clear") cover = inv?.filesWithArt ?? 0;
    else if (artChanged) {
      const h = typeof art === "object" && "hash" in art ? art.hash : null;
      const cand = h ? inv?.candidates.find((c) => c.hash === h) : null;
      cover = cand ? fileCount - cand.count : fileCount;
    }
    const n = Math.min(fileCount, parts.reduce((a, b) => a + b, 0) + cover);
    const contributors = parts.filter((p) => p > 0).length + (cover > 0 ? 1 : 0);
    return { n, upTo: contributors > 1 && n > 0, cover: artChanged };
  });

  const discs = $derived.by(() => {
    if (!albumId) return 1;
    return new Set(library.tracksOf(albumId).map((t) => t.disc)).size;
  });

  const label = $derived.by(() => {
    const album = library.albums.find((a) => a.id === albumId);
    return album ? `Edit album tags — ${album.title}` : "Edit album tags";
  });

  /** The receipt's Done button takes focus on mount (the attribute form
   *  mounts focus before layout; the action does it one frame later, which
   *  is also where the modal's own focus rules live). */
  function focusNow(el: HTMLElement) {
    const id = requestAnimationFrame(() => el.focus());
    return { destroy: () => cancelAnimationFrame(id) };
  }

  /** Enter writes the album (the track modal's accelerator, extended):
     *  from any text field it reads as "submit", which is exactly the
     *  intent here; on a button it stays the button's own activation.
     *  After a save the footer is the receipt — Enter then presses Done. */
  function quickKey(e: KeyboardEvent) {
    if (e.key !== "Enter") return;
    const t = e.target as HTMLElement | null;
    if (t && t.tagName === "BUTTON") return;
    if (receipt) {
      e.preventDefault();
      ui.tagEditor.open = false;
      return;
    }
    if (!loading && dirty && !saving && !bad.size) {
      e.preventDefault();
      void save();
    }
  }

  async function save() {
    if (!dirty || saving || bad.size) return;
    saving = true;
    error = "";
    try {
      // Shared struct carries album-level fields only; per-track fields
      // (title/artist/numbers) are never touched by album saves. Only what
      // the user edited rides along — fields they did not touch are never
      // written, so files that already agree keep their mtimes and the next
      // scan skips them.
      const rep = await invoke<{ written: number; total: number; receipt: string[] }>(
        "save_album_tags",
        {
          albumId,
          touched,
          art,
          shared: {
            title: "",
            artist: "",
            albumArtist: edit.albumArtist,
            album: edit.album,
            year: strNum(edit.year),
            trackNo: null,
            trackTotal: strNum(edit.trackTotal),
            discNo: null,
            discTotal: strNum(edit.discTotal),
            genre: edit.genre,
            composer: edit.composer,
            label: edit.label,
            comment: edit.comment,
            grouping: edit.grouping,
          },
        },
      );
      receipt = { written: rep.written, total: rep.total, lines: rep.receipt };
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
    // Edited mtimes re-parse through the real grouping path; scan-finished
    // refreshes the frontend. The vanish-guard closes us if the album merged
    // away. Fire-and-forget so the receipt lands while the library refreshes.
    void rescan();
  }
</script>

<TagSurface
  {label}
  {loading}
  busy={saving}
  escapeFirst={() => {
    if (!artLb) return false;
    artLb = false;
    return true;
  }}
  {quickKey}
>
  {#snippet footer(close)}
    {#if receipt}
      <footer class="te-foot te-receipt">
        <span class="te-ok">
          Wrote {receipt.written} of {receipt.total} {receipt.total === 1 ? "file" : "files"}
        </span>
        {#if receipt.lines.length}
          <span class="te-receipt-lines">{receipt.lines.join(" · ")}</span>
        {/if}
        <button class="te-btn primary" onclick={close} use:focusNow>Done</button>
      </footer>
    {:else}
      <footer class="te-foot">
        {#if error}
          <span class="te-error">{error}</span>
        {:else if bad.size}
          <span class="te-error">{[...bad].map((k) => NUM_LABELS[k] ?? k).join(", ")} must be a number</span>
        {:else if blast && dirty}
          <span class="te-blast">
            writes {blast.upTo ? "up to " : ""}{blast.n} of {fileCount} files{blast.cover ? " · cover" : ""}
          </span>
        {:else}
          <!-- The album's identity caption, in the slot the track modal
               spends on file name + path (owner ruling: the footer is
               where "what am I looking at" lives, in both modals). -->
          <span class="te-foot-files">
            {fileCount} {fileCount === 1 ? "file" : "files"} associated to this
            album{#if discs > 1} · across {discs} discs{/if}
          </span>
        {/if}
        <button class="te-btn" onclick={close}>Cancel</button>
        <button
          class="te-btn primary"
          disabled={!dirty || saving || loading || bad.size > 0}
          onclick={save}
        >
          {saving ? "Saving…" : "Save"}
        </button>
      </footer>
    {/if}
  {/snippet}

  <p class="te-topline">
    Fields you do not edit are not written. Chips show the values your files
    disagree with — click one to adopt it. Track and disc numbering belongs to
    the files: edit it on a track.
  </p>

  <div class="te-cols">
    <ArtSelector
      {albumId}
      stack
      bind:open={artLb}
      bind:change={art}
      bind:inventory={inv}
    />
    <div class="te-fields">
      <div class="te-col-title">Album tags</div>
      <FieldGrid {edit} layout="album-core" bad={bad} {census} {disputed} />
      <button class="te-more" aria-expanded={moreOpen} onclick={() => (moreOpen = !moreOpen)}>
        <svg class="te-more-chev" class:open={moreOpen} viewBox="0 0 10 10" aria-hidden="true">
          <path
            d="M3 1.5 6.5 5 3 8.5"
            fill="none"
            stroke="currentColor"
            stroke-width="1.4"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
        More tag fields
        {#if moreEdited}
          <span class="te-more-tag te-more-edited">edited</span>
        {:else if moreFilled > 0}
          <span class="te-more-tag">
            {moreFilled} {moreFilled === 1 ? "field" : "fields"} filled
          </span>
        {/if}
      </button>
      {#if moreOpen}
        <div
          class="te-more-wrap"
          transition:slide={{
            duration: prefersReducedMotion.current ? 0 : 220,
            easing: cubicOut,
          }}
        >
          <FieldGrid {edit} layout="album-more" bad={bad} {census} {disputed} />
        </div>
      {/if}
    </div>
  </div>
</TagSurface>

<style>
  /* Mirrors the track modal: the two column titles start on one line
     (the artwork column's own 11px offset is matched, not ignored). */
  .te-fields {
    min-width: 0;
    padding-top: 11px;
  }

  .te-col-title {
    font-size: 12px;
    color: var(--text-dim);
    margin: 0 0 6px;
  }

  .te-foot-files {
    flex: 1;
    min-width: 0;
    font-size: 11.5px; /* the track modal's footer caption measure */
    color: var(--text-dim);
    letter-spacing: 0.02em;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .te-blast {
    flex: 1;
    min-width: 0;
    font-size: 11.5px;
    color: var(--text-dim);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .te-receipt {
    gap: 10px;
  }

  .te-receipt-lines {
    flex: 1;
    min-width: 0;
    font-size: 11px;
    color: var(--text-dim);
    text-align: right;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
