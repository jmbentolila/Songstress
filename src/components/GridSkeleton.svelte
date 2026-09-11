<script lang="ts">
  /**
   * The album grid's placeholder: the real tile geometry with the real content
   * taken out. Nothing here is invented — column count, the 20px grid gap, the
   * 12px row gap, the square cover at --radius-cover, the 13px cover→caption
   * gap and the caption's two line heights all mirror AlbumGrid's own rules, so
   * when the library lands it FILLS the shapes rather than reflowing them.
   *
   * It is a preview of the grid, not of the albums: pill widths vary so the
   * column reads as content, but they come from a fixed pattern (see
   * loadingState) because this node re-renders on every progress event.
   */
  import { COUNT_W, NAME_W } from "../lib/loadingState";

  let { cols, rows }: { cols: number; rows: number } = $props();

  const cap = (arr: number[], i: number) => `${Math.round(arr[i % arr.length] * 100)}%`;
</script>

<div class="sk-grid" aria-hidden="true">
  {#each Array(rows) as _, r (r)}
    <div class="sk-row" style:--cols={cols}>
      {#each Array(cols) as _, c (c)}
        {@const i = r * cols + c}
        <div class="sk-tile">
          <div class="sk sk-cover" style:--i={i}></div>
          <div class="sk-caption">
            <div class="sk sk-line" style:--i={i + 1} style:width={cap(NAME_W, c)}></div>
            <div class="sk sk-line sub" style:--i={i + 2} style:width={cap(COUNT_W, c + r)}></div>
          </div>
        </div>
      {/each}
    </div>
  {/each}
</div>

<style>
  /* Same skeleton as the real grid's mount: an instant swap when the library
     arrives (the switch contract), a fade only for the placeholder coming in.
     CSS rather than svelte/transition so the global reduced-motion kill switch
     can reach it. */
  .sk-grid {
    display: flex;
    flex-direction: column;
    gap: var(--gap);
    animation: sk-in 160ms var(--ease-out);
  }

  @keyframes sk-in {
    from {
      opacity: 0;
    }
  }

  .sk-row {
    display: grid;
    grid-template-columns: repeat(var(--cols), minmax(0, 1fr));
    gap: 12px var(--gap);
  }

  .sk-tile {
    display: flex;
    flex-direction: column;
    gap: 13px;
    padding: 0 0 8px;
  }

  .sk-cover {
    width: 100%;
    aspect-ratio: 1;
    border-radius: var(--radius-cover);
    /* Weight arrives WITH the placeholder (owner call 2026-09-10): the real
       tile carries this shadow, so the skeleton holds the same mass and the
       artwork fades in without the tile gaining weight. */
    box-shadow: 0 4px 18px rgba(0, 0, 0, 0.35);
  }

  .sk-caption {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  /* 14px under a 13px/600 line, 12px under an 11.5px one: the box sits on the
     text's box, so the row height matches the real caption's. */
  .sk-line {
    height: 14px;
    border-radius: 999px;
  }

  .sk-line.sub {
    height: 12px;
  }
</style>
