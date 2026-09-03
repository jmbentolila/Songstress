<script lang="ts">
  /**
   * The tag editor's mode switch (Tag Editor Redesign, Phase C split).
   *
   * The mode branches used to fight inside one 800-line file: album mode is a
   * BULK surface (census, chips, blast radius, receipt) and track mode is a
   * SURGICAL one (which file am I in, stepper, datalist, stays-behind
   * caution) — sharing markup only where the two genuinely agree. Each editor
   * is a complete modal built on TagSurface; this file only picks the door.
   */
  import { ui } from "../lib/stores/ui.svelte";
  import AlbumTagsEditor from "./AlbumTagsEditor.svelte";
  import TrackTagsEditor from "./TrackTagsEditor.svelte";
</script>

{#if ui.tagEditor.open}
  {#if ui.tagEditor.trackId}
    <TrackTagsEditor trackId={ui.tagEditor.trackId} />
  {:else if ui.tagEditor.albumId}
    <AlbumTagsEditor albumId={ui.tagEditor.albumId} />
  {/if}
{/if}
