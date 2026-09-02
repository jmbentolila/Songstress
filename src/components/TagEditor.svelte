<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { library } from "../lib/stores/library.svelte";
  import { ui } from "../lib/stores/ui.svelte";
  import SurfaceClose from "./SurfaceClose.svelte";
  import { trapTab } from "../lib/focusTrap";
  import { rescan } from "../lib/stores/scanner.svelte";

  /** File-level tag surface mirrored from the Rust TrackTags (serde names). */
  interface TrackTags {
    title: string;
    artist: string;
    albumArtist: string;
    album: string;
    year: number | null;
    trackNo: number | null;
    trackTotal: number | null;
    discNo: number | null;
    discTotal: number | null;
    genre: string;
    composer: string;
    label: string;
    comment: string;
    grouping: string;
  }

  interface AlbumTagsApi extends Omit<TrackTags, "title" | "artist"> {
    [k: string]: string | number | null | boolean;
    albumArtistDisputed: boolean;
    albumDisputed: boolean;
    yearDisputed: boolean;
    genreDisputed: boolean;
    composerDisputed: boolean;
    labelDisputed: boolean;
    groupingDisputed: boolean;
    commentDisputed: boolean;
    trackTotalDisputed: boolean;
    discTotalDisputed: boolean;
  }

  /** Everything editable lives in strings; "" ⇔ absent (empty clears keys). */
  interface Editable {
    title: string;
    artist: string;
    albumArtist: string;
    album: string;
    year: string;
    trackNo: string;
    trackTotal: string;
    discNo: string;
    discTotal: string;
    genre: string;
    composer: string;
    label: string;
    comment: string;
    grouping: string;
  }

  const FIELDS: {
    key: keyof Editable;
    label: string;
    half?: boolean;
    wide?: boolean;
  }[] = [
    { key: "title", label: "Title" },
    { key: "artist", label: "Artist" },
    { key: "albumArtist", label: "Album artist" },
    { key: "album", label: "Album" },
    { key: "genre", label: "Genre" },
    { key: "year", label: "Year" },
    { key: "composer", label: "Composer" },
    { key: "trackNo", label: "Track #", half: true },
    { key: "label", label: "Label" },
    { key: "discNo", label: "Disc #", half: true },
    { key: "grouping", label: "Grouping" },
    { key: "comment", label: "Comment", wide: true },
  ];

  const numStr = (n: number | null): string => (n === null ? "" : String(n));
  const strNum = (s: string): number | null => (s.trim() === "" ? null : Number(s));

  function toEditable(t: Partial<TrackTags>): Editable {
    return {
      title: t.title ?? "",
      artist: t.artist ?? "",
      albumArtist: t.albumArtist ?? "",
      album: t.album ?? "",
      year: numStr(t.year ?? null),
      trackNo: numStr(t.trackNo ?? null),
      trackTotal: numStr(t.trackTotal ?? null),
      discNo: numStr(t.discNo ?? null),
      discTotal: numStr(t.discTotal ?? null),
      genre: t.genre ?? "",
      composer: t.composer ?? "",
      label: t.label ?? "",
      comment: t.comment ?? "",
      grouping: t.grouping ?? "",
    };
  }

  let mode = $derived(ui.tagEditor.trackId !== null ? "track" : "album");
  let edit = $state<Editable>(toEditable({}));
  let disputed = $state<Record<string, boolean>>({});
  let loading = $state(false);
  let saving = $state(false);
  let error = $state("");
  let panelEl = $state<HTMLElement>();

  let snapshot = "";

  async function load() {
    const { trackId, albumId } = ui.tagEditor;
    error = "";
    snapshot = "";
    if (trackId !== null) {
      const t = await invoke<TrackTags>("get_track_tags", { trackId });
      edit = toEditable(t);
    } else if (albumId !== null) {
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
      edit = toEditable({
        albumArtist: a.albumArtist as string,
        album: a.album as string,
        year: a.year as number | null,
        trackTotal: a.trackTotal as number | null,
        discTotal: a.discTotal as number | null,
        genre: a.genre as string,
        composer: a.composer as string,
        label: a.label as string,
        comment: a.comment as string,
        grouping: a.grouping as string,
      });
    }
    snapshot = JSON.stringify(edit);
  }

  $effect(() => {
    if (!ui.tagEditor.open) return;
    loading = true;
    void load()
      .catch((e) => (error = String(e)))
      .finally(() => (loading = false));
    // First field takes focus once the inputs exist.
    queueMicrotask(() =>
      requestAnimationFrame(() => panelEl?.querySelector("input")?.focus()),
    );
  });

  // Regroup edge: a save that merges/splits albums can make the edited album
  // vanish on reload. Close gracefully instead of editing a ghost.
  $effect(() => {
    if (!ui.tagEditor.open || !library.ready) return;
    const id = ui.tagEditor.albumId;
    if (id !== null && !library.albums.some((a) => a.id === id)) {
      if (ui.expandedAlbum.songs === id) ui.expandedAlbum.songs = null;
      if (ui.expandedAlbum.albums === id) ui.expandedAlbum.albums = null;
      ui.tagEditor.open = false;
    }
  });

  let dirty = $derived(JSON.stringify(edit) !== snapshot && snapshot !== "");

  let heading = $derived.by(() => {
    if (mode === "track") return "Edit tags";
    const album = library.albums.find((a) => a.id === ui.tagEditor.albumId);
    return album ? `Edit album tags — ${album.title}` : "Edit album tags";
  });

  function cancel() {
    if (saving) return;
    ui.tagEditor.open = false;
  }

  async function save() {
    if (!dirty || saving) return;
    saving = true;
    error = "";
    try {
      const { trackId, albumId } = ui.tagEditor;
      if (trackId !== null) {
        await invoke("save_track_tags", {
          trackId,
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
      } else {
        // Shared struct carries album-level fields only; per-track fields
        // (title/artist/numbers) are never touched by album saves.
        await invoke("save_album_tags", {
          albumId,
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
        });
      }
      ui.tagEditor.open = false;
      // Edited mtimes re-parse through the real grouping path; scan-finished
      // refreshes the frontend. The vanish-guard above closes us if the
      // album merged away.
      await rescan();
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }
</script>

<svelte:window
  onkeydown={(e) => {
    if (!ui.tagEditor.open) return;
    if (e.key === "Tab") {
      trapTab(e, panelEl);
      return;
    }
    if (e.key === "Escape") cancel();
  }}
/>

{#if ui.tagEditor.open}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="te-backdrop scrim" role="presentation" onclick={(e) => e.target === e.currentTarget && cancel()}>
    <!-- svelte-ignore a11y_no_noninteractive_element_to_interactive_role -->
    <section class="te glass" bind:this={panelEl} role="dialog" aria-modal="true" aria-label={heading}>
      <header class="te-head">
        <h2>{heading}</h2>
        <SurfaceClose label="Close" onclick={cancel} />
      </header>

      {#if loading}
        <p class="te-note">Reading file tags…</p>
      {:else}
        <div class="te-body">
          <div class="te-grid">
            {#each FIELDS.filter((f) => mode === "track" || !(f.key === "title" || f.key === "artist")) as field (field.key)}
              <label class="te-label" for={`te-${field.key}`}>
                {field.label}{disputed[field.key] ? " •" : ""}
              </label>
              <span class="te-field" class:wide={field.wide}>
                <input
                  id={`te-${field.key}`}
                  class={field.half ? "half" : ""}
                  bind:value={edit[field.key]}
                  inputmode={["year", "trackNo", "trackTotal", "discNo", "discTotal"].includes(field.key)
                    ? "numeric"
                    : undefined}
                />
                {#if field.key === "trackNo"}<i>of</i>
                  <input aria-label="Track total" bind:value={edit.trackTotal} inputmode="numeric" />
                {:else if field.key === "discNo"}<i>of</i>
                  <input aria-label="Disc total" bind:value={edit.discTotal} inputmode="numeric" />
                {/if}
              </span>
            {/each}
          </div>

          {#if mode === "album"}
            <p class="te-hint">
              • marks fields where tracks disagree (first non-empty wins on save).
              Per-track fields are edited via right-click → Edit tags… on a track.
            </p>
          {/if}
        </div>
      {/if}

      <footer class="te-foot">
        {#if error}<span class="te-error">{error}</span>{/if}
        <button class="te-btn" onclick={cancel}>Cancel</button>
        <button class="te-btn primary" disabled={!dirty || saving || loading} onclick={save}>
          {saving ? "Saving…" : "Save"}
        </button>
      </footer>
    </section>
  </div>
{/if}

<style>
  .te {
    width: min(620px, calc(100vw - 80px));
    max-height: calc(100vh - 120px);
    display: flex;
    flex-direction: column;
    padding: 16px;
    border-radius: var(--radius-panel);
    border: 1px solid var(--border);
    background: var(--panel-bg-strong);
    box-shadow: var(--shadow);
  }

  .te-head {
    display: flex;
    align-items: center;
    gap: 12px;
    padding-bottom: 10px;
    border-bottom: 1px solid var(--border);
  }

  .te-head h2 {
    flex: 1;
    min-width: 0;
    margin: 0;
    font-size: 15px;
    color: var(--text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

      .te-body {
    overflow-y: auto;
    padding: 12px 2px;
  }

  .te-grid {
    display: grid;
    grid-template-columns: max-content 1fr max-content 1fr;
    gap: 8px 10px;
    align-items: center;
  }

  /* Wide fields (comment) span both field columns. */
  .te-grid .wide {
    grid-column: 2 / -1;
  }

  .te-label {
    font-size: 12px;
    color: var(--text-dim);
    white-space: nowrap;
  }

  .te-field {
    display: flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
  }

  .te-field.wide {
    grid-column: 2 / -1;
  }

  .te-field input {
    flex: 1;
    min-width: 0;
  }

  .te-field input.half {
    flex: none;
    width: 64px;
  }

  .te-field i {
    font-size: 11px;
    color: var(--text-dim);
    font-style: normal;
  }

  .te-body input {
    padding: 6px 9px;
    border: 1px solid var(--border);
    border-radius: 7px;
    background: transparent;
    color: var(--text);
    font-size: 13px;
  }

  .te-body input:focus {
    outline: none;
    border-color: var(--accent);
  }

  .te-hint {
    margin: 10px 0 0;
    font-size: 11px;
    color: var(--text-dim);
  }

  .te-note {
    margin: 0;
    padding: 20px 2px;
    font-size: 13px;
    color: var(--text-dim);
  }

  .te-foot {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 8px;
    padding-top: 12px;
    border-top: 1px solid var(--border);
  }

  .te-error {
    flex: 1;
    min-width: 0;
    font-size: 12px;
    /* The system's one caution hue — the same token a missing-file glyph and a
       discard mark wear. A second, undocumented red was what this family used to
       look like; one hue, named, is what it now looks like. */
    color: var(--caution);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .te-btn {
    padding: 7px 14px;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: transparent;
    color: var(--text);
    font-size: 13px;
    cursor: pointer;
  }

  .te-btn:hover:not(:disabled) {
    background: var(--hover);
  }

  .te-btn.primary {
    background: var(--active);
    color: var(--accent);
    border-color: transparent;
    font-weight: 600;
  }

  .te-btn.primary:hover:not(:disabled) {
    background: var(--accent);
    color: var(--accent-text, #fff);
  }

  .te-btn:disabled {
    opacity: 0.45;
    cursor: default;
  }
</style>
