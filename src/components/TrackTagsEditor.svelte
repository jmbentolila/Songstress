<script lang="ts">
  /**
   * The TRACK tag modal (Tag Editor Redesign, Phase C). The surgical one:
   * it says WHICH file it is editing (name, folder, reveal), walks the
   * album with a header stepper so you can fix five files without the
   * window ever closing, offers existing album titles to retag into
   * (datalist, same-album-artist first), and warns honestly when a retag
   * leaves a library file behind in a folder that no longer matches.
   * Artwork here is per-file: the album's candidates, applied to THIS file.
   */
  import { invoke } from "@tauri-apps/api/core";
  import { slide } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import { prefersReducedMotion } from "svelte/motion";
  import { library, LIVE_LIBRARY } from "../lib/stores/library.svelte";
  import { sortKey } from "../lib/sort";
  import { ui } from "../lib/stores/ui.svelte";
  import { rescan } from "../lib/stores/scanner.svelte";
  import { announcer } from "../lib/stores/announcer.svelte";
  import TagSurface from "./TagSurface.svelte";
  import FieldGrid from "./FieldGrid.svelte";
  import ArtSelector from "./ArtSelector.svelte";
  import type { ArtChange, ArtInventory } from "../lib/artChange";
  import {
    NUM_LABELS,
    badFieldKeys,
    toEditable,
    strNum,
    type Editable,
    type TrackFile,
    type TrackTags,
  } from "../lib/tagFields";

  let { trackId }: { trackId: string } = $props();

  let edit = $state<Editable>(toEditable({}));
  let base = $state<Editable | null>(null);
  let snapshotJson = $state("");
  let meta = $state<TrackFile | null>(null);
  let loading = $state(true);
  let saving = $state(false);
  let error = $state("");
  /** Set once a write lands; shown until the next edit or step. Track mode
   *  has no receipt — one file, you watched it happen. */
  let written = $state(false);
  let inv = $state<ArtInventory | null>(null);
  let art = $state<ArtChange>("keep");
  let artLb = $state(false);

  /** "More tag fields": the rarely-touched four live behind this fold
     (owner ruling — the surgical modal leads with the everyday fields).
     Collapsed by default; the header accounts for what the fold hides,
     so the window never feels like it swallowed content. */
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
    written = false;
    art = "keep";
    if (!LIVE_LIBRARY) {
      // Fake-library dev mode has no files behind the rows.
      edit = toEditable({});
      snapshotJson = JSON.stringify(edit);
      base = { ...edit };
      return;
    }
    const [t, f] = await Promise.all([
      invoke<TrackTags>("get_track_tags", { trackId }),
      invoke<TrackFile>("get_track_file", { trackId }),
    ]);
    meta = f;
    edit = toEditable(t);
    snapshotJson = JSON.stringify(edit);
    base = { ...edit };
  }

  $effect(() => {
    if (!ui.tagEditor.open) return;
    const id = trackId; // the stepper re-points this: same window, next file
    loading = true;
    void load()
      .catch((e) => (error = String(e)))
      .finally(() => (loading = false));
    void id;
  });

  /** "Add to existing album…" — the pending-only door (Phase E). A staged
   *  file may JOIN an album that already exists: this is a RETAG, not a
   *  mover (scan.rs truth: same artist + normalized title ⇒ the next scan
   *  APPENDS the row even across folders), and the later Import moves the
   *  file into that album's own folder via album_destination. Library
   *  tracks never see this door — merging their folders is a bigger verb
   *  than a tag edit can promise, which is why the datalist warns instead. */
  let pickerOpen = $state(false);
  let pickerFilter = $state("");
  let picked = $state<{ album: string; n: number; total: number } | null>(null);

  /** Targets with the file's own artist first, then the rest alphabetically
   *  (article-stripping sortKey, same order the grid uses); the album the
   *  file already points at is not a target. `n` is the next free number
   *  (max+1, gaps respected — a 12-track album with a hole at 7 joins at
   *  12, not 13); `total` is what "of" will read. */
  let pickerOptions = $derived.by(() => {
    const q = pickerFilter.trim().toLowerCase();
    const who = (edit.albumArtist || edit.artist).trim().toLowerCase();
    const here = edit.album.trim().toLowerCase();
    const out: {
      id: string;
      title: string;
      year: number | null;
      artist: string;
      n: number;
      total: number;
      mine: boolean;
    }[] = [];
    for (const a of library.albums) {
      if (a.title.trim().toLowerCase() === here) continue;
      const artist = library.artistOf(a)?.name ?? "";
      if (
        q &&
        !(a.title.toLowerCase().includes(q) || artist.toLowerCase().includes(q))
      )
        continue;
      const ts = library.tracksOf(a.id);
      const max = ts.reduce((m, t) => Math.max(m, t.track ?? 0), 0);
      out.push({
        id: a.id,
        title: a.title,
        year: a.year,
        artist,
        n: max + 1,
        total: Math.max(ts.length + 1, max + 1),
        mine: artist.trim().toLowerCase() === who,
      });
    }
    out.sort(
      (a, b) =>
        Number(b.mine) - Number(a.mine) ||
        sortKey(a.artist).localeCompare(sortKey(b.artist)) ||
        sortKey(a.title).localeCompare(sortKey(b.title)),
    );
    return out.slice(0, 40);
  });

  function applyTarget(o: (typeof pickerOptions)[number]) {
    edit.album = o.title;
    edit.albumArtist = o.artist;
    // Cosmetic hygiene (spec): year cannot split a row — the adopt pass
    // matches on title alone — but agreeing with the target keeps the
    // album line reading as one record.
    if (o.year !== null) edit.year = String(o.year);
    edit.trackNo = String(o.n);
    edit.trackTotal = String(o.total);
    picked = { album: o.title, n: o.n, total: o.total };
    pickerOpen = false;
    announcer.say(
      `Will join ${o.title} as track ${o.n} of ${o.total}. Save writes it; Import moves the file.`,
    );
  }

  let dirty = $derived(
    (JSON.stringify(edit) !== snapshotJson && snapshotJson !== "") || art !== "keep",
  );
  const bad = $derived(badFieldKeys(edit));

  /** The picker's field-actions: open lands focused (search is the point
   *  of the popup), Enter commits the first candidate — the keyboard path
   *  the rest of the modal already honors. */
  function autofocus(el: HTMLInputElement) {
    el.focus();
    // The fold may open partly below the body's scroll line; the search
    // box — the point of the fold — earns its own scroll-into-view once
    // the slide has a frame of geometry to measure.
    requestAnimationFrame(() => el.scrollIntoView({ block: "nearest" }));
  }
  function enterFirst(el: HTMLInputElement) {
    const onKey = (e: KeyboardEvent) => {
      if (e.key !== "Enter") return;
      e.preventDefault();
      const first = pickerOptions[0];
      if (first) applyTarget(first);
    };
    el.addEventListener("keydown", onKey);
    return {
      destroy: () => el.removeEventListener("keydown", onKey),
    };
  }

  /* Phase D copy: one sentence per failure set, grammatical for both
     sizes ("Year must be a number" / "Track #, Year must be numbers"). */
  let badMsg = $derived(
    bad.size
      ? [...bad]
          .map((k) => NUM_LABELS[k] ?? k)
          .join(", ") + (bad.size === 1 ? " must be a number" : " must be numbers")
      : "",
  );

  // --- the stepper: the listless way to walk an album ----------------------
  let siblings = $derived(meta ? library.tracksOf(meta.albumId) : []);
  let idx = $derived(siblings.findIndex((t) => t.id === trackId));

  function step(d: number) {
    const next = siblings[idx + d];
    if (!next || saving) return;
    // Unsaved edits gate the stepper (the buttons say so): the promise is
    // "fields you did not touch are never written", and silently dropping
    // fields you DID touch would break it in the other direction.
    if (dirty) return;
    // A track change dismisses the lightbox: it is a layer about the file
    // you WERE looking at, and its captions would go stale one step behind
    // (probe 0.9.0: the stepper left it standing over a moved context).
    artLb = false;
    ui.tagEditor = { ...ui.tagEditor, trackId: next.id };
  }

  // --- retag door: existing titles, same-album-artist first ----------------
  let albumOptions = $derived.by(() => {
    const who = (edit.albumArtist || edit.artist).trim().toLowerCase();
    const seen = new Set<string>();
    const mine: string[] = [];
    const rest: string[] = [];
    for (const a of library.albums) {
      if (seen.has(a.title)) continue;
      seen.add(a.title);
      const artist = (library.artistOf(a)?.name ?? "").toLowerCase();
      (who && artist === who ? mine : rest).push(a.title);
    }
    return [...mine.sort(), ...rest.sort()];
  });

  /** The stays-behind caution: retagging a LIBRARY file into another album
   *  moves the row, not the file — the folder will disagree with the tags
   *  until the file is imported (the Phase E door covers pending tracks). */
  let staysBehind = $derived.by(() => {
    if (!meta || meta.staged || !base || !snapshotJson) return false;
    return edit.album !== base.album || edit.albumArtist !== base.albumArtist;
  });

  const label = $derived(
    edit.title ? `Edit tags — ${edit.title}` : meta ? `Edit tags — ${meta.file}` : "Edit tags",
  );

  function reveal() {
    void invoke("reveal_container", { trackId }).catch((e) => console.error(e));
  }

  /** Accelerators (critique 0.9.0 — the Flexibility gap): Enter writes the
     *  file you are looking at, ←/→ walk the album. Typing rules decide the
     *  split: Enter inside a text field means "submit" (form convention),
     *  but arrows inside a field belong to the caret; on a button, Enter is
     *  the button's own native activation. step() itself gates on dirty. */
  function quickKey(e: KeyboardEvent) {
    const t = e.target as HTMLElement | null;
    if (e.key === "Enter") {
      if (t && t.tagName === "BUTTON") return;
      if (!loading && dirty && !saving && !bad.size) {
        e.preventDefault();
        void save();
      }
      return;
    }
    if (e.key === "ArrowLeft" || e.key === "ArrowRight") {
      if (
        t &&
        (t.tagName === "INPUT" || t.tagName === "TEXTAREA" || t.isContentEditable)
      )
        return;
      e.preventDefault();
      step(e.key === "ArrowLeft" ? -1 : 1);
    }
  }

  async function save() {
    if (!dirty || saving || bad.size) return;
    saving = true;
    error = "";
    try {
      await invoke("save_track_tags", {
        trackId,
        art,
        tags: {
          title: edit.title,
          artist: edit.artist,
          albumArtist: edit.albumArtist,
          album: edit.album,
          year: strNum(edit.year),
          trackNo: strNum(edit.trackNo),
          trackTotal: strNum(edit.trackTotal),
          discNo: strNum(edit.discNo),
          discTotal: strNum(edit.discTotal),
          genre: edit.genre,
          composer: edit.composer,
          label: edit.label,
          comment: edit.comment,
          grouping: edit.grouping,
        },
      });
      // The window STAYS (this is what the stepper is for): re-read the file
      // so the fields show disk truth again and the next step starts clean.
      await load();
      written = true;
      // "Written ✓" in the footer, heard (WCAG 4.1.3).
      announcer.say(`Written to ${meta?.file ?? "the file"}`);
    } catch (e) {
      error = String(e);
      announcer.say(`Writing failed: ${error}`);
    } finally {
      saving = false;
    }
    // The re-tag may regroup the row; scan-finished refreshes the frontend
    // under the open window. Fire-and-forget, same as the album modal.
    void rescan();
  }
</script>

<TagSurface
  {label}
  loading={loading && !meta}
  busy={saving}
  escapeFirst={() => {
    if (!artLb) return false;
    artLb = false;
    return true;
  }}
  {quickKey}
>
  {#snippet headerExtra()}
    {#if siblings.length > 1}
      <div class="te-step" role="group" aria-label="Track in album">
        <button
          class="te-step-btn"
          disabled={idx <= 0 || dirty}
          title={dirty ? "Save or cancel first" : "Previous track (←)"}
          aria-label="Previous track"
          onclick={() => step(-1)}
        >◂</button>
        <span class="te-step-pos">{idx + 1} / {siblings.length}</span>
        <button
          class="te-step-btn"
          disabled={idx < 0 || idx >= siblings.length - 1 || dirty}
          title={dirty ? "Save or cancel first" : "Next track (→)"}
          aria-label="Next track"
          onclick={() => step(1)}
        >▸</button>
      </div>
    {/if}
  {/snippet}

  {#snippet footer(close)}
    <footer class="te-foot">
      {#if meta}
        {#if LIVE_LIBRARY}
          <button class="te-file-reveal" title="Open containing folder" onclick={reveal}>
            <svg viewBox="0 0 16 16" aria-hidden="true">
              <path
                d="M1.5 4.5A1.5 1.5 0 0 1 3 3h3l1.4 1.5H13a1.5 1.5 0 0 1 1.5 1.5v5.5A1.5 1.5 0 0 1 13 13H3a1.5 1.5 0 0 1-1.5-1.5z"
                fill="none"
                stroke="currentColor"
                stroke-width="1.3"
                stroke-linejoin="round"
              />
            </svg>
          </button>
        {/if}
        <span class="te-file-name" title={meta.path}>{meta.file}</span>
      {/if}
      <!-- The path owns the slack: it clips its own head (rtl trick) and
           hides the excess before the buttons, which never move. -->
      <span class="te-file-dir" title={meta?.path}>
        {#if error}
          <span class="te-dir-text te-dir-msg">{error}</span>
        {:else if bad.size}
          <span class="te-dir-text te-dir-msg">{badMsg}</span>
        {:else if written && !dirty}
          <span class="te-dir-text te-dir-ok">Written ✓</span>
        {:else}
          <span class="te-dir-text">{meta?.folder}</span>
        {/if}
      </span>
      <button class="te-btn" onclick={close} title="Close without writing (Esc)">Cancel</button>
      <button
        class="te-btn primary"
        disabled={!dirty || saving || loading || bad.size > 0}
        title={saving ? "Writing…" : dirty ? "Write this file (Enter)" : "Nothing to write"}
        onclick={save}
      >
        {saving ? "Saving…" : "Save"}
      </button>
    </footer>
  {/snippet}

  <p class="te-topline">Fields you do not edit are not written.</p>
  <div class="te-cols">
    {#if LIVE_LIBRARY && meta}
      <!-- Same artwork COLUMN as the album modal — stacked candidates, real
           previews — only the APPLY target differs: this file, not the album. -->
      <ArtSelector
        albumId={meta.albumId}
        {trackId}
        stack
        bind:open={artLb}
        bind:change={art}
        bind:inventory={inv}
      />
    {:else}
      <div></div>
    {/if}
    <div class="te-right">
      <div class="te-col-title">Song tags</div>
      <FieldGrid {edit} layout="track-core" bad={bad} {albumOptions} />
      {#if meta?.staged}
        <!-- The pending-only door (Phase E). A staged file has no folder to
             betray yet, so JOINING an existing album is a promise the retag
             can actually keep. -->
        <div class="te-join">
          <button
            class="te-more"
            aria-expanded={pickerOpen}
            onclick={() => {
              pickerFilter = "";
              pickerOpen = !pickerOpen;
            }}
          >
            <svg class="te-more-chev" class:open={pickerOpen} viewBox="0 0 10 10" aria-hidden="true">
              <path
                d="M3 1.5 6.5 5 3 8.5"
                fill="none"
                stroke="currentColor"
                stroke-width="1.4"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
            </svg>
            Add to existing album…
          </button>
          {#if pickerOpen}
            <!-- It wears the fold's face, so it folds like the fold: inline
                 slide, same wrap, same curve — not a floating layer the
                 user must chase (owner ruling). -->
            <div
              class="te-more-wrap te-pick"
              transition:slide={{
                duration: prefersReducedMotion.current ? 0 : 220,
                easing: cubicOut,
              }}
            >
              <input
                class="te-pop-q"
                placeholder="Search albums…"
                aria-label="Search albums"
                bind:value={pickerFilter}
                use:autofocus
                use:enterFirst
              />
              <div class="te-pop-list">
                {#each pickerOptions as o (o.id)}
                  <button class="te-pop-row" onclick={() => applyTarget(o)}>
                    <span class="te-pop-t">{o.title}</span>
                    <span class="te-pop-m">{o.artist}{o.year ? " · " + o.year : ""}</span>
                    <span class="te-pop-n">track {o.n} of {o.total}</span>
                  </button>
                {:else}
                  <p class="te-pop-none">No album matches “{pickerFilter}”.</p>
                {/each}
              </div>
            </div>
          {/if}
          {#if picked && edit.album === picked.album}
            <span class="te-join-promise">
              will join <b>{picked.album}</b> as track {picked.n} of {picked.total} —
              Save writes it; Import moves the file
            </span>
          {/if}
        </div>
      {/if}
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
        <!-- Slide = the disclosure animates its own height, enter and exit
             along the same path, interruptible mid-flight (apple-design);
             reduced motion collapses it to instant, not to a different
             animation. Margin lives INSIDE the wrapper: slide clips what
             it animates, and an outside margin would survive the outro
             as a phantom gap for the transition's whole tail. -->
        <div
          class="te-more-wrap"
          transition:slide={{
            // Svelte 5.9's is a reactive MediaQuery value (docs use it in
            // transition params verbatim); transitions read params when
            // they START, so this is "live" for every open and close.
            duration: prefersReducedMotion.current ? 0 : 220,
            easing: cubicOut,
          }}
        >
          <FieldGrid {edit} layout="track-more" bad={bad} />
        </div>
      {/if}
    </div>
  </div>

  {#if staysBehind}
    <p class="te-stay">
      Album changed — this file is already in your library, so it stays in
      <b>{meta?.folder}</b> until you import it; the folder will read differently from the
      tags until then.
    </p>
  {/if}
</TagSurface>

<style>
  /* Header stepper: the listless way to walk the album. Small, quiet, and
     DISABLED while unsaved — it will not eat your edits silently. */
  .te-step {
    display: flex;
    align-items: center;
    gap: 4px;
    flex: none;
  }

  .te-step-btn {
    border: 1px solid var(--border);
    border-radius: 7px;
    background: transparent;
    color: var(--text-dim);
    font-size: 11px;
    line-height: 1;
    padding: 6px 8px;
    cursor: pointer;
  }

  .te-step-btn:hover:not(:disabled) {
    background: var(--hover);
    color: var(--text);
  }

  .te-step-btn:active:not(:disabled) {
    transform: scale(0.94);
  }

  .te-step-btn:disabled {
    opacity: 0.4;
    cursor: default;
  }

  .te-step-pos {
    font-size: 11px;
    color: var(--text-dim);
    min-width: 40px;
    text-align: center;
    font-variant-numeric: tabular-nums;
  }

  /* The footer IS the identity bar: reveal · name · path, then the
     buttons — never moved, never moved by the text. The path carries every
     message (errors, the written note) in its own slot: the buttons are
     furniture, the path is the news. */
  .te-file-name {
    flex: none;
    max-width: 45%;
    font-size: 11.5px;
    color: var(--text-dim);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .te-file-dir {
    flex: 1;
    min-width: 0;
    font-size: 11.5px; /* the SAME measure as the name beside it (owner
                         ruling): one caption, not a caption and a heading */
    color: var(--text-dim);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    direction: rtl; /* keep the TAIL of the path when it has to clip */
    text-align: left;
  }

  /* …but the PATH stays LTR (see the bidi note from the screenshot that
     read "Music/Music Files/Aimer/DAWN/~"): isolate + its OWN direction. */
  .te-dir-text {
    direction: ltr;
    unicode-bidi: isolate;
  }

  .te-dir-msg {
    color: var(--caution);
  }

  .te-dir-ok {
    color: var(--accent);
    font-weight: 600;
  }

  .te-file-reveal {
    flex: none;
    display: inline-flex;
    padding: 4px 6px;
    border: 1px solid var(--border);
    border-radius: 7px;
    background: transparent;
    color: var(--text-dim);
    cursor: pointer;
  }

  .te-file-reveal:hover {
    background: var(--hover);
    color: var(--text);
  }

  .te-file-reveal svg {
    width: 14px;
    height: 14px;
  }

  /* The right column of the body: title, core fields, then the fold.
     The 11px top measure matches the artwork column's own (its `.as`
     padding + dashed border) — "Artwork" and "Song tags" start on the
     same line (owner ruling). */
  .te-right {
    min-width: 0;
    padding-top: 11px;
  }

  /* A caption for the column, the exact sibling of ArtSelector's
     "Artwork": same size, same hue, same 6px breath below. */
  .te-col-title {
    font-size: 12px;
    color: var(--text-dim);
    margin: 0 0 6px;
  }

  /* The disclosure row's styles moved to TagSurface (both editors share
     the control). */

  /* The honest warning — the system's one caution hue, same as the album
     modal's errors. A retag that splits a folder must not be a surprise. */
  .te-stay {
    margin: 10px 0 0;
    font-size: 11.5px;
    color: var(--caution);
  }

  .te-stay b {
    font-weight: 600;
  }
  /* ── The pending-only join door (Phase E) ─────────────────────────── */
  .te-join {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: 8px;
  }

  /* The promise the picker made, in the sentence the spec promised: not
     "album changed" but "you will be track 12 of 14". Shown while the
     album field still says what the picker wrote (any later typing that
     diverges retires it honestly). */
  .te-join-promise {
    flex: 1 1 100%;
    font-size: 11.5px;
    color: var(--text-dim);
  }

  .te-join-promise b {
    color: var(--text);
    font-weight: 600;
  }

  .te-pick {
    display: flex;
    flex-direction: column;
    border: 1px solid var(--border);
    border-radius: var(--radius-control);
    padding: 8px;
  }

  .te-pop-q {
    flex: none;
    padding: 7px 9px;
    border: 1px solid var(--border);
    border-radius: 7px;
    background: transparent;
    color: var(--text);
    font-size: 13px;
  }

  .te-pop-q:focus {
    outline: none;
    border-color: var(--accent);
  }

  .te-pop-list {
    overflow-y: auto;
    margin-top: 6px;
    min-height: 0;
    /* Inline means the fold owns its size: the list scrolls in place, it
       does not stretch the modal across the screen. Sized to the modal's
       real estate — open this and the first rows plus the promise are in
       view without a chase. */
    max-height: 160px;
  }

  /* One row per target: title, provenance, and the number it will wear —
     the consequence is on the option, not after choosing it. */
  .te-pop-row {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    grid-template-rows: auto auto;
    gap: 0 10px;
    width: 100%;
    padding: 7px 8px;
    border: 0;
    border-radius: 7px;
    background: none;
    text-align: left;
    cursor: pointer;
  }

  .te-pop-row:hover {
    background: var(--hover);
  }

  .te-pop-t {
    grid-row: 1;
    grid-column: 1;
    font-size: 13px;
    color: var(--text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .te-pop-n {
    grid-row: 1 / span 2;
    grid-column: 2;
    align-self: center;
    font-size: 11px;
    color: var(--text-dim);
    white-space: nowrap;
  }

  .te-pop-m {
    grid-row: 2;
    grid-column: 1;
    font-size: 11px;
    color: var(--text-dim);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .te-pop-none {
    margin: 4px 8px;
    font-size: 12px;
    color: var(--text-dim);
  }
</style>
