/**
 * Playbar waveform geometry.
 *
 * The peaks themselves come from the backend: `track_peaks` decodes a track
 * with symphonia and caches 256 amplitudes in SQLite (see
 * `src-tauri/src/library/peaks.rs`). This module holds the pure geometry that
 * turns those amplitudes into the lane's SVG, plus the resample that maps the
 * stored bucket count onto the lane's bar count — nothing here touches audio
 * or the DB.
 */

/** The quiet baseline an idle lane (or a track that just cleared) rests at.
 *  NON-zero: the quiet state IS the waveform at its minimum height — the bars
 *  expand straight out of it — rather than a separate dotted line. The round
 *  cap (~3px) dominates the rendered height either way. */
export const WAVE_FLOOR = 0.05;

/** Bars across the lane. Fewer bars = wider cells, so each bar reads with
 *  more definition; 80 keeps the silhouette yet gives the bars body. */
export const WAVE_BARS = 80;

/** Display headroom: the loudest bar reaches this fraction of the lane height,
 *  so the waveform is never a wall-to-wall block. Owner-tuned down twice from
 *  0.82 — a loud master fills it otherwise (median bar ~56% of the lane). */
export const WAVE_HEADROOM = 0.5;

/** Display gamma (>1). EXPANDS the differences between loud passages — which
 *  is what turns a heavily-compressed master (peaks all near full, median
 *  bucket ~200/255) from a solid rectangle into a readable waveform. Applied
 *  at DISPLAY time, so the stored peaks stay a faithful linear amplitude. */
export const WAVE_GAMMA = 1.7;

/** Shape a normalized peak (0..1) for the lane: expand contrast, leave
 *  headroom, and never fall below the quiet floor. */
export function shapeAmplitude(v: number): number {
  const a = clamp01(v) ** WAVE_GAMMA;
  return WAVE_FLOOR + a * (WAVE_HEADROOM - WAVE_FLOOR);
}

/**
 * Reduce a peak array of any length to `n` display bars using the group's root
 * mean square. MAX preserves transients, but it also lets one loud sub-bucket
 * dominate an entire lane cell and flattens quiet valleys into the surrounding
 * loud section. RMS instead tracks the section's sustained energy while still
 * rendering a sparse transient as a nonzero bar. Maps the backend's stored
 * bucket count onto the lane's bar count.
 */
export function resample(peaks: number[], n: number): number[] {
  if (peaks.length === 0) return new Array<number>(n).fill(0);
  if (peaks.length === n) return peaks.slice();
  const out = new Array<number>(n);
  for (let i = 0; i < n; i++) {
    const lo = Math.floor((i * peaks.length) / n);
    const hi = Math.max(lo + 1, Math.floor(((i + 1) * peaks.length) / n));
    let sumSquares = 0;
    let count = 0;
    for (let j = lo; j < hi && j < peaks.length; j++) {
      const value = peaks[j];
      if (!Number.isFinite(value)) continue;
      sumSquares += value * value;
      count += 1;
    }
    out[i] = count > 0 ? Math.sqrt(sumSquares / count) : 0;
  }
  return out;
}

/**
 * SVG path for a MIRRORED bar waveform, in a `viewBox="0 0 n 100"` space and
 * drawn with `preserveAspectRatio="none"` so the lane can be any size. Bars
 * are centred on y=50; amplitude 1 fills the lane.
 *
 * Every bar is a vertical LINE, not a filled rect: the lane strokes it with
 * `stroke-linecap: round` + `vector-effect: non-scaling-stroke`, so each bar
 * keeps a true circular pill end under the non-uniform viewBox scale.
 * A zero-amplitude bar is omitted (a lone zero-length segment would be a
 * single round dot); the quiet floor is a small NON-zero amplitude instead,
 * so the rest line is simply the waveform at minimum height.
 */
export function peaksPath(peaks: number[]): string {
  const half = 50;
  let d = "";
  for (let i = 0; i < peaks.length; i++) {
    const h = clamp01(peaks[i]) * half;
    if (h < 0.5) continue;
    const x = i + 0.5;
    d += `M${x} ${(half - h).toFixed(2)}V${(half + h).toFixed(2)}`;
  }
  return d;
}

function clamp01(v: number): number {
  return v < 0 ? 0 : v > 1 ? 1 : v;
}