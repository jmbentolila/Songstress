import { describe, expect, it } from "vitest";
import {
  peaksPath,
  resample,
  shapeAmplitude,
  WAVE_BARS,
  WAVE_FLOOR,
  WAVE_HEADROOM,
} from "./waveform";

describe("waveform", () => {
  it("resample maps to n bars using group energy, not group maximum", () => {
    expect(resample([], 4)).toEqual([0, 0, 0, 0]);
    expect(resample([0.1, 0.2, 0.3, 0.4], 4)).toEqual([0.1, 0.2, 0.3, 0.4]);
    // Narrowing keeps a sparse transient visible while letting valleys fall.
    const narrowed = resample([0.1, 0.9, 0.3, 0.4], 2);
    expect(narrowed[0]).toBeCloseTo(Math.sqrt((0.1 ** 2 + 0.9 ** 2) / 2), 6);
    expect(narrowed[1]).toBeCloseTo(Math.sqrt((0.3 ** 2 + 0.4 ** 2) / 2), 6);
    expect(narrowed[1]).toBeLessThan(narrowed[0]);
    // Widening repeats each source value across its group.
    expect(resample([0.5, 1], 4)).toEqual([0.5, 0.5, 1, 1]);
    expect(resample([Number.NaN, 0.5], 1)).toEqual([0.5]);
  });

  it("shapeAmplitude leaves headroom and expands loud differences", () => {
    expect(shapeAmplitude(0)).toBeCloseTo(WAVE_FLOOR, 6);
    expect(shapeAmplitude(1)).toBeCloseTo(WAVE_HEADROOM, 6);
    // Gamma > 1 pulls mid/loud values DOWN, so a compressed block spreads out.
    expect(shapeAmplitude(0.75)).toBeLessThan(0.6);
    expect(shapeAmplitude(0.75)).toBeGreaterThan(WAVE_FLOOR);
  });

  it("peaksPath emits one vertical segment per non-zero bar", () => {
    const d = peaksPath(new Array(WAVE_BARS).fill(0.5));
    expect(d.match(/M/g)).toHaveLength(WAVE_BARS);
    expect(d.match(/V/g)).toHaveLength(WAVE_BARS);
    expect(d).not.toMatch(/NaN|Infinity/);
  });

  it("peaksPath clamps out-of-range amplitudes and omits the floor", () => {
    const d = peaksPath([-5, 2, 0]);
    expect(d).not.toMatch(/NaN|Infinity/);
    // -5 clamps to the floor (omitted); 2 clamps to a full bar; 0 is omitted.
    expect(d).toBe("M1.5 0.00V100.00");
  });
});