<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  // Screen color picker overlay (fullscreen window at #screen-pick): renders
  // the portal screenshot edge to edge under a standard crosshair cursor,
  // with a magnifier loupe riding the pointer. Click lifts the pixel under
  // the cursor; Escape/right-click cancels. Either way ONE finish call
  // reports back and tears this window down — never leave it parked.
  let dataUrl = $state<string | null>(null);
  let failed = $state(false);
  let finished = false;

  let shot = $state<HTMLImageElement | null>(null);
  let sample = $state<HTMLCanvasElement | null>(null);
  let loupe = $state<HTMLCanvasElement | null>(null);

  // Loupe geometry: an 11px source patch shown at 12x, so every screen
  // pixel reads as a tile and the exact target stays countable.
  const SRC = 11;
  const SCALE = 12;
  const SIZE = SRC * SCALE;

  let loupeOn = $state(false);
  let loupeLeft = $state(0);
  let loupeTop = $state(0);
  let loupeHex = $state("#000000");

  function finish(hex: string | null) {
    if (finished) return;
    finished = true;
    void invoke("finish_screen_pick", { hex });
  }

  function rgbToHex(r: number, g: number, b: number): string {
    const h = (v: number) => Math.round(v).toString(16).padStart(2, "0");
    return `${h(r)}${h(g)}${h(b)}`;
  }

  function shotPoint(e: PointerEvent): { x: number; y: number } | null {
    if (!shot || !sample) return null;
    const r = shot.getBoundingClientRect();
    if (r.width <= 0 || r.height <= 0) return null;
    const x = Math.min(
      sample.width - 1,
      Math.max(0, Math.round(((e.clientX - r.left) / r.width) * sample.width)),
    );
    const y = Math.min(
      sample.height - 1,
      Math.max(0, Math.round(((e.clientY - r.top) / r.height) * sample.height)),
    );
    return { x, y };
  }

  function pixelAt(x: number, y: number): string | null {
    const ctx = sample?.getContext("2d", { willReadFrequently: true });
    if (!ctx) return null;
    try {
      const [r, g, b] = ctx.getImageData(x, y, 1, 1).data;
      return rgbToHex(r, g, b);
    } catch {
      return null;
    }
  }

  function onMove(e: PointerEvent) {
    const p = shotPoint(e);
    if (!p) {
      loupeOn = false;
      return;
    }
    const hex = pixelAt(p.x, p.y);
    if (hex === null) {
      loupeOn = false;
      return;
    }
    loupeHex = `#${hex}`;
    // Paint the loupe: source patch centred on the target, clamped into
    // the shot, smoothing off so pixels stay tiles. The centre tile gets
    // the reticle — that tile IS the pixel a click would lift.
    const lctx = loupe?.getContext("2d");
    const sctx = sample?.getContext("2d", { willReadFrequently: true });
    if (lctx && sctx && sample) {
      const half = Math.floor(SRC / 2);
      const sx = Math.min(sample.width - SRC, Math.max(0, p.x - half));
      const sy = Math.min(sample.height - SRC, Math.max(0, p.y - half));
      lctx.imageSmoothingEnabled = false;
      lctx.clearRect(0, 0, SIZE, SIZE);
      lctx.drawImage(sample, sx, sy, SRC, SRC, 0, 0, SIZE, SIZE);
      lctx.strokeStyle = "rgba(255, 255, 255, 0.9)";
      lctx.lineWidth = 2;
      const c = half * SCALE;
      lctx.strokeRect(c, c, SCALE, SCALE);
    }
    // Park the loupe down-right of the pointer, flipping inside the
    // viewport near the edges — it must never cover its own target.
    const pad = 18;
    loupeLeft =
      e.clientX + pad + SIZE + 16 > window.innerWidth
        ? e.clientX - pad - SIZE
        : e.clientX + pad;
    loupeTop =
      e.clientY + pad + SIZE + 40 > window.innerHeight
        ? e.clientY - pad - SIZE - 36
        : e.clientY + pad;
    loupeOn = true;
  }

  function onPick(e: PointerEvent) {
    if (e.button !== 0) return;
    const p = shotPoint(e);
    const hex = p ? pixelAt(p.x, p.y) : null;
    finish(hex);
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === "Escape") finish(null);
  }

  function onMenu(e: Event) {
    e.preventDefault();
    finish(null);
  }

  // Pointer listeners live on the window, not the div: a fullscreen overlay
  // is mouse-driven by nature, and element-level pointer handlers would trip
  // the static-element-interactions lint with nothing to gain.
  $effect(() => {
    void invoke<string>("take_pick_image")
      .then((url) => {
        dataUrl = url;
      })
      .catch(() => {
        failed = true;
      });
    window.addEventListener("keydown", onKey);
    window.addEventListener("contextmenu", onMenu);
    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerdown", onPick);
    document.addEventListener("mouseleave", onLeave);
    return () => {
      window.removeEventListener("keydown", onKey);
      window.removeEventListener("contextmenu", onMenu);
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerdown", onPick);
      document.removeEventListener("mouseleave", onLeave);
    };
  });

  function onLeave() {
    loupeOn = false;
  }

  function onShotLoad() {
    // Sampling canvas at the shot's native size: getImageData reads are
    // exact pixels, never interpolated display pixels.
    if (shot && sample) {
      sample.width = shot.naturalWidth;
      sample.height = shot.naturalHeight;
      sample.getContext("2d")?.drawImage(shot, 0, 0);
    }
  }
</script>

<div class="pick">
  {#if dataUrl}
    <img
      class="shot"
      src={dataUrl}
      alt=""
      draggable="false"
      bind:this={shot}
      onload={onShotLoad}
    />
  {:else if failed}
    <p class="err">The screen grab failed — press Escape to go back.</p>
  {:else}
    <p class="wait">Grabbing the screen…</p>
  {/if}
  {#if loupeOn}
    <div
      class="loupe glass"
      style:left={`${Math.round(loupeLeft)}px`}
      style:top={`${Math.round(loupeTop)}px`}
    >
      <canvas bind:this={loupe} width={SIZE} height={SIZE}></canvas>
      <div class="hex">{loupeHex}</div>
    </div>
  {/if}
  <canvas class="sample" bind:this={sample} hidden></canvas>
</div>

<style>
  .pick {
    position: fixed;
    inset: 0;
    background: #000;
    cursor: crosshair;
    overflow: hidden;
    /* No user-select quirk here: WebKitGTK needs the prefix (gotcha). */
    -webkit-user-select: none;
    user-select: none;
  }

  .shot {
    display: block;
    width: 100%;
    height: 100%;
    /* Fill, not contain: the click math maps client pixels to shot pixels
       by the same stretch, so any letterbox would lie about the target. */
    object-fit: fill;
    pointer-events: none;
  }

  .wait,
  .err {
    position: absolute;
    inset: 0;
    display: grid;
    place-items: center;
    color: #f4f4f6;
    font-size: 15px;
  }

  .loupe {
    position: fixed;
    z-index: 2;
    padding: 8px;
    border-radius: 10px;
    pointer-events: none;
  }

  .loupe canvas {
    display: block;
    border-radius: var(--radius-icon);
  }

  .hex {
    margin-top: 6px;
    text-align: center;
    font-size: 13px;
    font-variant-numeric: tabular-nums;
    color: var(--text);
  }

  .sample {
    position: absolute;
    visibility: hidden;
  }
</style>
