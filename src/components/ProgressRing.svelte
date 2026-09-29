<script lang="ts">
  import { fade } from "svelte/transition";

  /**
   * Determinate progress: an arc that sweeps clockwise as the work gets done,
   * and stops. Deliberately NOT a spinner — a scan that has been running for
   * forty seconds looks identical to one that has been running for two when the
   * only signal is rotation, which reads as a hang. This fills, so the row it
   * sits on says how far its own verb has got.
   *
   * The viewBox is fixed at 16 units and `size` scales the box, so the stroke
   * stays proportional to the dot it shares a row tail with.
   *
   * Mount/unmount fades (160ms, off under reduced motion): the ring used to
   * pop in and out around multi-phase runs. The value itself is fused into one
   * monotonic 0..1 by the caller, so no phase re-key is needed — the arc never
   * runs backwards, it just keeps filling.
   */
  const FADE = {
    duration: matchMedia("(prefers-reduced-motion: reduce)").matches ? 0 : 160,
  };
  let {
    value,
    label,
    size = 16,
  }: {
    /** 0..1. Out-of-range and absent totals clamp to empty, never to a guess. */
    value: number;
    /** Full sentence for assistive tech, e.g. "Scanning 137 of 4,434". */
    label: string;
    /** The run's current phase. Accepted and ignored: the value arrives
     * pre-fused and monotonic, so nothing re-keys the node (the old {#key}
     * snapped the arc back to empty at every phase boundary, which read as
     * a second scan). Kept so callers keep passing scanner.phase. */
    phase?: string;
    size?: number;
  } = $props();

  const R = 6.5;
  const C = 2 * Math.PI * R;
  const clamped = $derived(Math.min(1, Math.max(0, value || 0)));
  const pct = $derived(Math.round(clamped * 100));
</script>

<span
  class="ring"
  role="progressbar"
  aria-valuenow={pct}
  aria-valuemin={0}
  aria-valuemax={100}
  aria-valuetext={label}
  style:width={`${size}px`}
  style:height={`${size}px`}
  transition:fade={FADE}
>
  <svg viewBox="0 0 16 16" aria-hidden="true">
    <circle class="track" cx="8" cy="8" r={R} />
    <circle
      class="fill"
      cx="8"
      cy="8"
      r={R}
      style:stroke-dasharray={C}
      style:stroke-dashoffset={C * (1 - clamped)}
    />
  </svg>
</span>

<style>
  .ring {
    position: relative;
    flex: none;
  }

  /* -90deg so the sweep starts at 12 o'clock: a ring that starts filling at
     3 o'clock reads as a gauge that was left mid-measurement. */
  .ring svg {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    /* transform, not standalone `rotate` — this WebKitGTK misapplies the
       individual transform properties on svg (2026-09-05). */
    transform: rotate(-90deg);
    overflow: visible;
  }

  .track {
    fill: none;
    stroke: color-mix(in srgb, var(--text) 20%, transparent);
    stroke-width: 2;
  }

  .fill {
    fill: none;
    stroke: var(--accent);
    stroke-width: 2;
    /* Butt caps: a round cap draws a dot at 0%, which is the one state that
       must look like nothing yet. */
    stroke-linecap: butt;
    /* Linear, and short: the value is data arriving from the backend, and an
       ease on it would lag the truth it is reporting. */
    transition: stroke-dashoffset 120ms linear;
  }

  @media (prefers-reduced-motion: reduce) {
    .fill {
      transition: none;
    }
  }
</style>
