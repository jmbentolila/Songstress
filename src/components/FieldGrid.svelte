<script lang="ts">
  /**
   * The tag editors' shared field grid (Phase C). One renderer for the
   * label+input rhythm both modals wear; the mode-specific pieces ride in
   * as data, not as branches the file fights over:
   *
   *  - `census` (album modal) puts dispute chips under a disagreed field;
   *  - track numbers carry their inline "of N" totals wherever they render;
   *  - `albumOptions` gives the Album field its datalist (the track modal's
   *    retag door — same-album-artist suggestions first, computed outside).
   *
   * The grid is TWO tracks — label | field. A row that holds two fields
   * (Genre+Year, Track#+Disc# — owner ruling: number fields are small, give
   * them the row they fit on) carries its pair INSIDE the field cell, so
   * each one takes exactly the space its content earns: a stretching genre
   * pushes its Year to the edge; the fixed-number rows pack left instead of
   * spraying across reserved tracks no other row uses.
   */
  import {
    FIELDS,
    type Editable,
    type FieldCensus,
    type FieldDef,
    type FieldKey,
  } from "../lib/tagFields";
  import { tooltip } from "../lib/tooltip";

  let {
    edit,
    layout,
    bad,
    census = {},
    disputed = {},
    albumOptions = null,
  }: {
    /** The editors' own $state object — mutated in place across the prop
     *  boundary, which is exactly the reactivity Svelte 5 proxies give you. */
    edit: Editable;
    /** Which ROWS (and pairs) this modal shows — the label rhythm is one
     *  for both: the track modal deliberately mirrors the album modal.
     *  "track-core" / "track-more" serve the track modal's accordion: the
     *  everyday fields, and the rarely-touched ones behind the fold. */
    layout: "album" | "album-core" | "album-more" | "track" | "track-core" | "track-more";
    bad: Set<string>;
    census?: Record<string, FieldCensus>;
    disputed?: Record<string, boolean>;
    /** Existing album titles, ordered (matches first); renders a datalist. */
    albumOptions?: string[] | null;
  } = $props();

  // Which fields this modal shows, and which share a row. Album mode never
  // touches per-file fields (title/artist/numbering belong to the files —
  // it says so in its hint).
  const PAIRS: Record<string, [FieldKey, FieldKey][]> = {
    album: [["genre", "year"]],
    "album-core": [["genre", "year"]],
    "album-more": [],
    track: [
      ["genre", "year"],
      ["trackNo", "discNo"],
    ],
    "track-core": [
      ["genre", "year"],
      ["trackNo", "discNo"],
    ],
    "track-more": [],
  };

  const MORE: FieldKey[] = ["composer", "label", "grouping", "comment"];
  const NOT_ALBUM: FieldKey[] = ["title", "artist", "trackNo", "discNo"];

  /* Year wears the SAME box as the Disc # field one row below (owner
     ruling) — but "same" can't be a guessed constant: the number row's
     box width is live flex math. So the grid measures its own disc field
     and hands the answer to Year via --num-w; a ResizeObserver keeps the
     pair married through any window change. (Album mode has no number row:
     the fallback 56px stands.) */
  let gridEl = $state<HTMLDivElement>();
  $effect(() => {
    const el = gridEl;
    if (!el) return;
    const d = el.querySelector<HTMLInputElement>("#te-discNo");
    const dl = el.querySelector<HTMLElement>("#te-lab-discNo");
    if (!d || !dl) {
      el.style.removeProperty("--cluster");
      // No number row to mirror (album mode): the YEAR label is then the
      // thing worth measuring — the symmetric Genre|Year row computes the
      // shared box width from its exact measure.
      const yl = el.querySelector<HTMLElement>("#te-lab-year");
      if (yl) {
        const roY = new ResizeObserver((entries) => {
          for (const e of entries) {
            const box = e.borderBoxSize?.[0];
            el.style.setProperty(
              "--lab-w",
              `${box ? box.inlineSize : e.contentRect.width}px`,
            );
          }
        });
        roY.observe(yl);
        return () => roY.disconnect();
      }
      el.style.removeProperty("--lab-w");
      return;
    }
    // The RO's borderBoxSize is the LAYOUT box — immune to the modal's
    // scale-in transform, which made a getBoundingClientRect snapshot read
    // a mid-animation pixel and pin Year short of the box it mirrors.
    // Observing fires once with the current size, then on every reflow of
    // those very fields, so the mirrored column can never drift.
    //
    // --cluster is the whole tail the disc pair occupies, from its LABEL
    // leftward to the row edge: label + (6 gap + 4 margin) + box + gap +
    // "of" + gap + box. The year row parks a group THAT wide at the same
    // edge, so "Year" starts exactly where "Disc #" starts — label under
    // label, box under box, the empty "of __" room simply left blank.
    // The whole disc tail — label, box, "of" text, total box — is observed
    // as a set. Every term of --cluster is then an exact layout box:
    // offsetWidth's integer rounding on any unobserved sibling left the year
    // label tenths of a pixel adrift of its disc twin, and "approximately
    // flush" is what got us here twice.
    const tail = [...(d.parentElement?.children ?? [])].filter(
      (n) => getComputedStyle(n).display !== "none",
    );
    const sizes = new Map<Element, number>();
    const ro = new ResizeObserver((entries) => {
      for (const e of entries) {
        const box = e.borderBoxSize?.[0];
        sizes.set(e.target, box ? box.inlineSize : e.contentRect.width);
      }
      const labw = sizes.get(dl);
      if (labw == null) return;
      let claimed = -6; // no row gap trails the last sibling
      for (const n of tail) {
        const w = sizes.get(n);
        if (w == null) return; // not every term measured yet
        claimed += w;
        if (n === dl) claimed += 4; // .te-grp > label margin-right
        claimed += 6; // the row gap before every following sibling
      }
      el.style.setProperty("--lab-w", `${labw}px`);
      el.style.setProperty("--cluster", `${claimed}px`);
    });
    for (const n of tail) ro.observe(n);
    return () => ro.disconnect();
  });

  const rows = $derived.by(() => {
    let visible: FieldDef[];
    if (layout === "album") {
      visible = FIELDS.filter((f) => !NOT_ALBUM.includes(f.key));
    } else if (layout === "album-core") {
      // The owner ruling of 2026-09-03: the rarely-touched four hide behind
      // the same accordion here too — the editor auto-opens it when the
      // census says a hidden field is disputed, so no disagreement sleeps
      // behind a fold.
      visible = FIELDS.filter(
        (f) => !NOT_ALBUM.includes(f.key) && !MORE.includes(f.key),
      );
    } else if (layout === "album-more") {
      visible = FIELDS.filter((f) => MORE.includes(f.key));
    } else if (layout === "track-core") {
      visible = FIELDS.filter((f) => !MORE.includes(f.key));
    } else if (layout === "track-more") {
      visible = FIELDS.filter((f) => MORE.includes(f.key));
    } else {
      visible = FIELDS;
    }
    const out: FieldDef[][] = [];
    const skip = new Set<FieldKey>();
    for (const f of visible) {
      if (skip.has(f.key)) continue;
      const pair = PAIRS[layout].find((p) => p[0] === f.key);
      if (pair && visible.some((v) => v.key === pair[1])) {
        skip.add(pair[1]);
        out.push([f, visible.find((v) => v.key === pair[1])!]);
      } else {
        out.push([f]);
      }
    }
    return out;
  });
</script>

<div
  class="te-grid"
  class:album-grid={layout === "album-core" || layout === "album-more"}
  bind:this={gridEl}
>
  {#each rows as row (row[0].key)}
    <label class="te-label" for={`te-${row[0].key}`}>
      {row[0].label}{disputed[row[0].key] ? " •" : ""}
    </label>
    <span
      class="te-field"
      class:nums={row.length === 2 && row[1].key === "discNo"}
      class:yrs={row.length === 2 && row[1].key === "year"}
    >
      {@render unit(row[0])}
      {#if row.length === 2}
        <span
          class="te-grp"
          class:year-grp={row[1].key === "year"}
        >
          <label
            class="te-label"
            for={`te-${row[1].key}`}
            id={`te-lab-${row[1].key}`}
          >
            {row[1].label}{disputed[row[1].key] ? " •" : ""}
          </label>
          {@render unit(row[1])}
        </span>
      {/if}
      <!-- Dispute chips come AFTER the row's inputs: a width-100% chip
           would otherwise wrap the paired field onto its own line. -->
      {#each row as f (f.key)}
        {#if (census[f.key]?.values.length ?? 0) >= 2}
          <span class="te-chips">
            {#each census[f.key].values as v (v.value)}
              <button
                class="te-chip"
                class:te-chip-on={String(edit[f.key]) === v.value}
                onclick={() => (edit[f.key] = v.value)}
                use:tooltip={`${v.value} — in ${v.count} ${v.count === 1 ? "file" : "files"}`}
              >{v.value}<i>{v.count}</i></button>
            {/each}
          </span>
        {/if}
      {/each}
    </span>
  {/each}
</div>

{#snippet unit(field: FieldDef)}
  <input
    id={`te-${field.key}`}
    class={field.half ? "half" : ""}
    class:is-year={field.key === "year"}
    class:te-bad={bad.has(field.key)}
    bind:value={edit[field.key]}
    list={field.key === "album" && albumOptions ? "te-albums" : undefined}
    inputmode={["year", "trackNo", "trackTotal", "discNo", "discTotal"].includes(
      field.key,
    )
      ? "numeric"
      : undefined}
  />
  {#if field.key === "trackNo"}<i>of</i>
    <input class="of" aria-label="Track total" bind:value={edit.trackTotal} inputmode="numeric" />
  {:else if field.key === "discNo"}<i>of</i>
    <input class="of" aria-label="Disc total" bind:value={edit.discTotal} inputmode="numeric" />
  {/if}
{/snippet}

{#if albumOptions}
  <datalist id="te-albums">
    {#each albumOptions as name (name)}
      <option value={name}></option>
    {/each}
  </datalist>
{/if}

<style>
  .te-grid {
    display: grid;
    grid-template-columns: max-content minmax(0, 1fr);
    gap: 8px 10px;
    align-items: center;
    min-width: 0;
  }

  .te-label {
    font-size: 14px;
    color: var(--text-dim);
    white-space: nowrap;
  }

  /* The second label of a pair: part of the row's unit, not the label
     column — it names the field beside it, right where the eye arrives. */
  /* The second field of a pair travels with its label as ONE unit: if a
     narrow window ever forces the row to wrap, the pair moves whole — it
     must never split mid-unit the way bare flex children did. */
  .te-grp {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    flex: none;
    margin-left: 6px;
  }

  /* A paired label keeps its field at the SAME distance a column label
     keeps its field (the grid's 10px column gap): Track #→input and
     Disc #→input, and Year→input, are one measure (owner ruling). The
     genre field is flex:1, so it pays for the extra pixel itself. */
  .te-grp > label {
    margin-right: 4px;
  }

  /* The disc cell is the row's tail: it claims the remaining column, split
     evenly between the two boxes, so the row ends at the same edge every
     other field ends at (owner ruling). MUST live below `.te-field
     input.half` — same specificity, and the loser silently kept the boxes
     fixed and left the slack at the row's tail (the screenshot that
     proved it). */
  /* The disc cell is the row's tail (owner ruling): the row must end at the
     edge every other field ends at, AND the four number boxes must stay
     EQUAL (an earlier ruling). Both hold only if all four grow from ONE
     flex context — so the numbering row's group dissolves (`display:
     contents`): the disc label joins the row as a plain item, and the four
     boxes share the remainder as siblings. Position IS the override again:
     this block lives after `.te-field input.half` (same specificity). */
  .te-field.nums .te-grp {
    display: contents;
  }

  .te-field.nums input.half,
  .te-field.nums input.of {
    flex: 1 1 0;
    width: auto;
    min-width: 56px;
  }

  .te-field {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 6px;
    min-width: 0;
  }

  .te-field input {
    flex: 1;
    min-width: 0;
    padding: 6px 9px;
    border: 1px solid var(--border);
    border-radius: 7px;
    background: transparent;
    color: var(--text);
    font-size: 15px;
  }

  .te-field input:focus {
    outline: none;
    border-color: var(--accent);
  }

  /* the fields small by nature (year, the numbers — all five the same
     measure, pair to pair and row to row): fixed width, and the content
     centered — short values float in a wide left-aligned box, and these
     boxes are sized to the value, so the value owns the box (owner
     ruling). */
  .te-field input.half,
  .te-field input.of {
    flex: none;
    width: 56px;
    text-align: center;
  }

  .te-field input.te-bad {
    border-color: var(--caution);
  }

  /* Consumes --num-w (see the grid script): the year box IS the disc box.
     Genre is flex:1, so it absorbs the width difference silently. THIRD
     time this file learned it: same specificity as `.te-field input.half`
     means POSITION is the override — or outrank it outright, which is what
     the doubled class does, so no future reshuffle can silently lose. */
  /* The year box FILLS whatever the label leaves of the mirrored cluster
     (owner ruling) — so it covers the room the "of __" pair would claim on
     the row below, and the two rows end flush on one shared right edge. */
  .te-field input.half.is-year {
    flex: 1 1 0;
    width: auto;
    min-width: 56px;
    text-align: center;
  }

  /* Owner ruling: the year LABEL stands under the Disc # label. The group
     hangs left-aligned where Genre's flex ends, the label wears the disc
     label's measure (--lab-w), and the box (--num-w) then falls exactly
     under the disc box — label under label, box under box. Whatever room
     the "of 1" pair needs further right simply stays empty. */
  .te-grp.year-grp {
    flex: none;
    width: var(--cluster, auto);
    margin-left: 0; /* one plain 6px gap from Genre, like the nums row's */
    justify-content: flex-start;
    /* The label's 4px margin-right would otherwise ADD to the group's set
       width (content-box), breaking every equality built on it. */
    box-sizing: border-box;
  }

  .te-grp.year-grp > label {
    width: var(--lab-w, auto);
  }

  /* Album mode's Genre|Year row (owner ruling): the two boxes are ONE
     width. The label sits between them, so each box is the column minus
     the label minus the two gaps, halved — with --lab-w measured from the
     real label, the equality is exact, not approximate. */
  .album-grid .te-field.yrs {
    /* budget: genre(b) + gap6 + label + gap6 + margin4 + year(b) = 100%
       → the two label/gap separations cost lab-w + 16 together. */
    --b: calc((100% - var(--lab-w, 38px) - 16px) / 2);
  }

  .album-grid .te-field.yrs > input:first-child {
    flex: none;
    width: var(--b);
  }

  .album-grid .te-field.yrs .te-grp.year-grp {
    width: calc(100% - var(--b) - 6px);
  }

  .te-field i {
    font-size: 13px;
    color: var(--text-dim);
    font-style: normal;
  }

  /* Dispute chips: the competing values themselves, census included.
     Click adopts — an immediate, on-press answer (border+bg, no lift). */
  .te-chips {
    display: inline-flex;
    flex-wrap: wrap;
    gap: 4px;
    width: 100%; /* its own line under the field — never cramped beside it */
  }

  .te-chip {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    max-width: 160px;
    padding: 2px 7px;
    border: 1px solid var(--border);
    border-radius: 999px;
    background: none;
    color: var(--text-dim);
    font-size: 13px;
    cursor: pointer;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .te-chip i {
    font-style: normal;
    font-size: 11.5px;
    opacity: 0.7;
  }

  .te-chip:hover {
    border-color: color-mix(in srgb, var(--accent) 55%, var(--border));
    color: var(--text);
  }

  .te-chip:active {
    transform: scale(0.96);
  }

  .te-chip-on {
    border-color: var(--accent);
    color: var(--accent);
    background: color-mix(in srgb, var(--accent) 12%, transparent);
  }
</style>
