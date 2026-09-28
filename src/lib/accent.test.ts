import { describe, expect, it } from "vitest";
import {
  accentVariants,
  fallbackAccentForTheme,
  hexToHsl,
  hslToHex,
  isAccentAvailable,
  relativeLuminance,
  ACCENT_PRESETS,
  BLACK_PRESET_HEX,
  WHITE_PRESET_HEX,
} from "./accent";

describe("hexToHsl / hslToHex", () => {
  it("converts stock dark accent", () => {
    const [h, s, l] = hexToHsl("#a78bfa");
    expect(Math.round(h)).toBe(255);
    expect(s).toBeCloseTo(0.92, 1);
    expect(l).toBeCloseTo(0.76, 2);
  });

  it("round-trips", () => {
    for (const hex of ["#d6409f", "#e5484d", "#46a758", "#3b82f6", "#000000", "#ffffff"]) {
      expect(hslToHex(...hexToHsl(hex))).toBe(hex.toLowerCase());
    }
  });

  it("expands 3-digit hex and rejects garbage", () => {
    expect(hexToHsl("#fa0")).toEqual(hexToHsl("#ffaa00"));
    expect(() => hexToHsl("nope")).toThrow();
  });
});

describe("relativeLuminance", () => {
  it("black is 0, white is 1", () => {
    expect(relativeLuminance("#000000")).toBe(0);
    expect(relativeLuminance("#ffffff")).toBeCloseTo(1);
  });

  it("amber is light, blue is dark", () => {
    expect(relativeLuminance("#ffb224")).toBeGreaterThan(0.5);
    expect(relativeLuminance("#3b82f6")).toBeLessThan(0.5);
  });
});

describe("accentVariants", () => {
  it("lightens too-dark colors for the dark theme", () => {
    const v = accentVariants("#46a758", "dark"); // L≈0.47
    const [, , l] = hexToHsl(v.accent);
    expect(l).toBeGreaterThanOrEqual(0.7);
    expect(l).toBeLessThanOrEqual(0.8);
  });

  it("darkens too-light colors for the light theme", () => {
    const v = accentVariants("#a78bfa", "light"); // L≈0.76
    const [, , l] = hexToHsl(v.accent);
    expect(l).toBeLessThanOrEqual(0.6);
    expect(l).toBeGreaterThanOrEqual(0.45);
  });

  it("darkens light-theme accents by ten lightness points", () => {
    const [h, s, l] = hexToHsl("#f76b15"); // L≈0.53
    const v = accentVariants("#f76b15", "light");
    expect(v.accent).toBe(hslToHex(h, s, l - 0.1));
    const [, , darkL] = hexToHsl(accentVariants("#f76b15", "dark").accent);
    expect(darkL).toBeCloseTo(0.7, 2);
  });

  it("active wash uses the darkened light-theme accent", () => {
    expect(accentVariants("#a78bfa", "dark").active).toBe("rgba(167, 139, 250, 0.18)");
    expect(accentVariants("#f76b15", "light").active).toBe("rgba(210, 84, 7, 0.14)");
  });

  it("accent text follows the darkened light-theme luminance", () => {
    expect(accentVariants("#ffb224", "light").accentText).toBe("#ffffff");
    expect(accentVariants("#3b82f6", "light").accentText).toBe("#ffffff");
    expect(accentVariants("#ffb224", "dark").accentText).toBe("#1b1b1f");
  });

  it("presets carry one hex each, stock is null", () => {
    expect(ACCENT_PRESETS[0]).toEqual({ name: "Stock purple", hex: null });
    for (const p of ACCENT_PRESETS.slice(1)) expect(p.hex).toMatch(/^#[0-9a-f]{6}$/);
  });

  it("black/white presets become theme-appropriate grays", () => {
    const black = accentVariants("#000000", "dark");
    const [, , bl] = hexToHsl(black.accent);
    expect(bl).toBeGreaterThanOrEqual(0.7); // lifted to readable gray
    const lightBlack = accentVariants("#000000", "light");
    const [, , lbl] = hexToHsl(lightBlack.accent);
    expect(lbl).toBeCloseTo(0.35, 2); // clamped, then darkened
    const white = accentVariants("#ffffff", "light");
    const [, , wl] = hexToHsl(white.accent);
    expect(wl).toBeCloseTo(0.5, 2); // sunk, then darkened
  });

  it("white is unavailable in light mode and falls back to black", () => {
    expect(isAccentAvailable(WHITE_PRESET_HEX, "light")).toBe(false);
    expect(isAccentAvailable(WHITE_PRESET_HEX, "dark")).toBe(true);
    expect(isAccentAvailable(BLACK_PRESET_HEX, "light")).toBe(true);
    expect(isAccentAvailable(null, "light")).toBe(true);
    expect(fallbackAccentForTheme(WHITE_PRESET_HEX, "light")).toBe(BLACK_PRESET_HEX);
    expect(fallbackAccentForTheme(WHITE_PRESET_HEX, "dark")).toBe(WHITE_PRESET_HEX);
    expect(fallbackAccentForTheme(BLACK_PRESET_HEX, "light")).toBe(BLACK_PRESET_HEX);
  });
});
