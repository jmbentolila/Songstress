<script lang="ts">
  /**
   * The tag editors' suggestion list (owner ask, 2026-09-26): the values the
   * library already holds, filtered to what is typed, wearing the SAME surface
   * as our context menu (`.menu-surface` in app.css, `menuPop` in
   * lib/menuPop.ts) instead of the browser's datalist popup.
   *
   * Positioned `absolute` INSIDE the field's cell by its owner (FieldGrid),
   * with offsets measured from the input: a `fixed` child of a `.glass` panel
   * would anchor to the panel — backdrop-filter is a containing block — and
   * the modal body both scrolls and clips. The owner also decides `flip`
   * (above the field) when the list would run past the panel's visible bottom.
   *
   * Picking is on POINTER-DOWN, with the default prevented so the input never
   * blurs: press-to-choose is the house rule, and a blur would close this list
   * out from under the press.
   */
  import { menuPop } from "../lib/menuPop";

  let {
    id,
    items,
    hi,
    left,
    top,
    bottom,
    flip,
    width,
    onpick,
    onhover,
  }: {
    id: string;
    items: string[];
    /** Highlighted row index, -1 for none (the owner's contract: a suggestion
     *  that merely repeats the typed value is not a completion). */
    hi: number;
    /** Offsets in px from the positioning cell's top-left. */
    left: number;
    top: number;
    bottom: number;
    flip: boolean;
    /** The input's width, so the list is never narrower than its field. */
    width: number;
    onpick: (value: string) => void;
    onhover: (index: number) => void;
  } = $props();
</script>

<div
  class="sug menu-surface glass"
  id={id}
  role="listbox"
  aria-label="Suggestions"
  style:left={`${left}px`}
  style:top={flip ? "auto" : `${top}px`}
  style:bottom={flip ? `${bottom}px` : "auto"}
  style:min-width={`${width}px`}
  transition:menuPop
>
  {#each items as item, i (item)}
    <div
      class="sug-item menu-item"
      class:is-active={i === hi}
      id={`${id}-${i}`}
      role="option"
      tabindex="-1"
      aria-selected={i === hi}
      onpointerdown={(e) => {
        e.preventDefault(); // keep focus in the input: no blur, no close first
        e.stopPropagation();
        onpick(item);
      }}
      onmousedown={(e) => e.preventDefault()}
      onpointerenter={() => onhover(i)}
    >{item}</div>
  {/each}
</div>

<style>
  .sug {
    position: absolute;
    /* The list is a suggestion, not a document: cap it and scroll rather than
     * growing past the field it belongs to. 320px clears the 8-item cap at
     * 15px type with the surface's padding and gaps. */
    max-height: 320px;
    overflow-y: auto;
    overflow-x: hidden;
    max-width: min(420px, 60vw);
  }

  /* Long album titles ellipsize instead of widening the popover past the
   * grid — the full value is already in the input once it is picked. */
  .sug-item {
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>