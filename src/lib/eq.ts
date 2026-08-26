/**
 * Equalizer shared bits (PLAN.md Step 6): band table, presets and pure
 * helpers for the playbar popover + Playback menu. Rust clamps again on the
 * wire (eq.rs) — these are the UI-side mirrors.
 */

export const EQ_BANDS = [31, 62, 125, 250, 500, 1000, 2000, 4000, 8000, 16000];

export const EQ_MAX_DB = 12;

export type EqState = {
  enabled: boolean;
  preampDb: number;
  /** 10 gains, dB, index-aligned with EQ_BANDS. */
  gains: number[];
  /** Preset name the gains came from; null = Custom (diverged). */
  preset: string | null;
};

export function defaultEq(): EqState {
  return { enabled: false, preampDb: 0, gains: new Array(10).fill(0), preset: "Flat" };
}

export const EQ_PRESETS: ReadonlyArray<{ name: string; gains: number[] }> = [
  { name: "Flat", gains: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0] },
  // Rock/Pop/Hip-Hop follow the gentler modern curves (Spotify/Poweramp-style);
  // Jazz/Classical/Electronic are Winamp's well-known shapes, still standard.
  { name: "Rock", gains: [5, 4, 2.5, 0, -2, -1, 1.5, 3.5, 5, 5.5] },
  { name: "Pop", gains: [2, 1.5, 0.5, -1, -1.5, -0.5, 1, 2, 3, 2.5] },
  { name: "Jazz", gains: [4, 3, 1, 2, -2, -2, 0, 1, 3, 4] },
  { name: "Classical", gains: [4, 3, 2, 0, -1, -1, 0, 2, 3, 4] },
  { name: "Electronic", gains: [7, 6, 2, 0, -2, 2, 1, 2, 6, 7] },
  { name: "Hip-Hop", gains: [6, 5, 2, 0, -1, 0, 1, 2, 3, 3] },
  { name: "Bass Boost", gains: [8, 8, 6, 4, 2, 0, 0, 0, 0, 0] },
  { name: "Treble Boost", gains: [0, 0, 0, 0, 0, 2, 4, 6, 8, 8] },
  { name: "Vocal Boost", gains: [-3, -2, 0, 3, 5, 5, 3, 1, 0, -1] },
];

export function clampDb(v: number): number {
  return Math.min(EQ_MAX_DB, Math.max(-EQ_MAX_DB, v));
}

/** Band label under a slider: kHz above 1k, plain Hz below. */
export function fmtHz(hz: number): string {
  return hz >= 1000 ? `${hz / 1000}k` : `${hz}`;
}

/** Signed dB readout ("+4", "0", "-6.5"). */
export function fmtDb(v: number): string {
  const r = Math.round(v * 10) / 10;
  return r > 0 ? `+${r}` : `${r}`;
}

/** Which preset these gains match exactly; null means Custom. */
export function matchingPreset(gains: number[]): string | null {
  const hit = EQ_PRESETS.find((p) =>
    p.gains.every((g, i) => Math.abs(g - (gains[i] ?? NaN)) < 0.01),
  );
  return hit?.name ?? null;
}
