import { describe, expect, it } from "vitest";
import {
  clampDb,
  defaultEq,
  EQ_BANDS,
  EQ_MAX_DB,
  EQ_PRESETS,
  fmtDb,
  fmtHz,
  matchingPreset,
} from "./eq";

describe("eq", () => {
  it("bands are the standard 10-band set, presets align with them", () => {
    expect(EQ_BANDS).toHaveLength(10);
    for (const p of EQ_PRESETS) {
      expect(p.gains, p.name).toHaveLength(10);
      for (const g of p.gains) {
        expect(Math.abs(g)).toBeLessThanOrEqual(EQ_MAX_DB);
      }
    }
    expect(matchingPreset(new Array(10).fill(0))).toBe("Flat");
  });

  it("clampDb keeps slider values in ±12", () => {
    expect(clampDb(15)).toBe(12);
    expect(clampDb(-15)).toBe(-12);
    expect(clampDb(3.3)).toBe(3.3);
  });

  it("matchingPreset detects divergence as Custom (null)", () => {
    const rock = EQ_PRESETS.find((p) => p.name === "Rock")!.gains.slice();
    expect(matchingPreset(rock)).toBe("Rock");
    rock[4] += 0.5;
    expect(matchingPreset(rock)).toBeNull();
    expect(matchingPreset([])).toBeNull();
  });

  it("labels read like an EQ faceplate", () => {
    expect(fmtHz(31)).toBe("31");
    expect(fmtHz(1000)).toBe("1k");
    expect(fmtHz(16000)).toBe("16k");
    expect(fmtDb(4)).toBe("+4");
    expect(fmtDb(-6.5)).toBe("-6.5");
    expect(fmtDb(0.04)).toBe("0");
  });

  it("default state is disabled flat", () => {
    const eq = defaultEq();
    expect(eq.enabled).toBe(false);
    expect(eq.preset).toBe("Flat");
    expect(eq.preampDb).toBe(0);
    expect(eq.gains.every((g) => g === 0)).toBe(true);
  });
});
