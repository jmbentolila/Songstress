import { describe, expect, it } from "vitest";
import {
  TEXT,
  SURFACE,
  artGradientContrast,
  contrastRatio,
  ensureContrast,
  hexToRgb,
  mixedColor,
  normalizeHex,
} from "./gradient";
import type { RGB } from "./artColors";

function composite(fg: RGB, alpha: number, bg: RGB): RGB {
  return [
    Math.round(fg[0] * alpha + bg[0] * (1 - alpha)),
    Math.round(fg[1] * alpha + bg[1] * (1 - alpha)),
    Math.round(fg[2] * alpha + bg[2] * (1 - alpha)),
  ];
}

function parseRgbaStops(grad: string): RGB[] {
  const m = grad.match(/rgba\((\d+), (\d+), (\d+),/g);
  expect(m).not.toBeNull();
  return m!.map((s) => s.match(/\d+/g)!.map(Number) as RGB);
}

describe("gradient contrast guarantee", () => {
  const cases: Array<[string, string]> = [
    ["ffffff", "eeeeee"],
    ["0a0a0a", "050505"],
    ["a78bfa", "7c58f0"],
    ["ffd700", "ff4500"],
    ["123456", "654321"],
  ];

  for (const theme of ["dark", "light"] as const) {
    for (const [c1, c2] of cases) {
      it(`keeps ${theme} text readable over ${c1}/${c2}`, () => {
        const grad = artGradientContrast(c1, c2, theme, 0.7);
        expect(grad).not.toBeNull();
        const text = TEXT[theme];
        const surface = SURFACE[theme];
        for (const stop of parseRgbaStops(grad!)) {
          const comp = composite(stop, 0.7, surface);
          expect(contrastRatio(text, comp)).toBeGreaterThanOrEqual(4.5);
          const dimOnStop = composite(text, 0.64, comp);
          expect(contrastRatio(dimOnStop, comp)).toBeGreaterThanOrEqual(3);
        }
      });
    }
  }

  it("already-passing stops are returned unchanged", () => {
    const mixed = mixedColor(hexToRgb("123456")!, "dark");
    expect(ensureContrast(mixed, "dark", 0.7)).toEqual(mixed);
  });

  it("returns null on invalid hex", () => {
    expect(artGradientContrast("zzzzzz", "123456", "dark", 0.7)).toBeNull();
  });
});

describe("normalizeHex", () => {
  it("accepts bare, hashed and shorthand forms", () => {
    expect(normalizeHex("ff6ec7")).toBe("ff6ec7");
    expect(normalizeHex("#FF6EC7")).toBe("ff6ec7");
    expect(normalizeHex("#abc")).toBe("aabbcc");
    expect(normalizeHex("abc")).toBe("aabbcc");
    expect(normalizeHex("  #123456  ")).toBe("123456");
  });

  it("rejects non-hex", () => {
    expect(normalizeHex("")).toBeNull();
    expect(normalizeHex("zzzzzz")).toBeNull();
    expect(normalizeHex("#12345")).toBeNull();
    expect(normalizeHex("red")).toBeNull();
  });
});
