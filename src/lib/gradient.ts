import type { Theme } from "./types";
import type { RGB } from "./artColors";

export const SURFACE: Record<string, [number, number, number]> = {
  dark: [22, 22, 28],
  light: [246, 246, 249],
};

export function rgbToHsl(c: RGB): RGB {
  const r = c[0] / 255;
  const g = c[1] / 255;
  const b = c[2] / 255;
  const max = Math.max(r, g, b);
  const min = Math.min(r, g, b);
  const l = (max + min) / 2;
  if (max === min) return [0, 0, l];
  const d = max - min;
  const s = d > 0.5 ? d / (2 - max - min) : d / (max + min);
  let h: number;
  if (max === r) h = ((g - b) / d + (g < b ? 6 : 0)) / 6;
  else if (max === g) h = ((b - r) / d + 2) / 6;
  else h = ((r - g) / d + 4) / 6;
  return [h, s, l];
}

export function hslToRgb(h: number, s: number, l: number): RGB {
  if (s === 0) {
    const v = Math.round(l * 255);
    return [v, v, v];
  }
  const q = l < 0.5 ? l * (1 + s) : l + s - l * s;
  const p = 2 * l - q;
  const channel = (t: number) => {
    if (t < 0) t += 1;
    if (t > 1) t -= 1;
    if (t < 1 / 6) return p + (q - p) * 6 * t;
    if (t < 1 / 2) return q;
    if (t < 2 / 3) return p + (q - p) * (2 / 3 - t) * 6;
    return p;
  };
  return [
    Math.round(channel(h + 1 / 3) * 255),
    Math.round(channel(h) * 255),
    Math.round(channel(h - 1 / 3) * 255),
  ];
}

export function mixedColor(c: RGB, theme: string): RGB {
  const s = SURFACE[theme] ?? SURFACE.dark;
  const t = 0.3;
  const [, , surfaceL] = rgbToHsl(s);
  const [h, sat, light] = rgbToHsl(c);
  // Blend and lift LIGHTNESS only — per-RGB-channel mixing stirs in gray
  // and washes the artwork hue away.
  let l = light * (1 - t) + surfaceL * t;
  l = l * 0.85 + 26 / 255;
  const s2 = Math.min(1, sat * 1.25);
  return hslToRgb(h, s2, l);
}

export function hexToRgb(hex: string): [number, number, number] | null {
  if (!/^[0-9a-fA-F]{6}$/.test(hex)) return null;
  return [
    parseInt(hex.slice(0, 2), 16),
    parseInt(hex.slice(2, 4), 16),
    parseInt(hex.slice(4, 6), 16),
  ];
}

export function gradientFromColors(c1: RGB, c2: RGB, theme: Theme, alpha: number): string {
  return `linear-gradient(135deg, rgba(${mixedColor(c1, theme).join(", ")}, ${alpha}), rgba(${mixedColor(c2, theme).join(", ")}, ${alpha}))`;
}

/** Same, from scan-computed hex colors; null when either hex is invalid. */
export function artGradient(
  c1hex: string,
  c2hex: string,
  theme: Theme,
  alpha: number,
): string | null {
  const c1 = hexToRgb(c1hex);
  const c2 = c1 ? hexToRgb(c2hex) : null;
  if (!c1 || !c2) return null;
  return gradientFromColors(c1, c2, theme, alpha);
}

// --- contrast guarantee (playbar backdrop, Step 2b) -------------------------

/** Theme --text colors from app.css. */
export const TEXT: Record<string, RGB> = {
  dark: [244, 244, 246],
  light: [35, 35, 41],
};

/** --text-dim is --text at alpha 0.64. */
const DIM_ALPHA = 0.64;
const MIN_TEXT_CONTRAST = 4.5;
const MIN_DIM_CONTRAST = 3;

function relativeLuminance(c: RGB): number {
  const f = (v: number) => {
    const s = v / 255;
    return s <= 0.04045 ? s / 12.92 : Math.pow((s + 0.055) / 1.055, 2.4);
  };
  return 0.2126 * f(c[0]) + 0.7152 * f(c[1]) + 0.0722 * f(c[2]);
}

export function contrastRatio(a: RGB, b: RGB): number {
  const la = relativeLuminance(a);
  const lb = relativeLuminance(b);
  const [hi, lo] = la >= lb ? [la, lb] : [lb, la];
  return (hi + 0.05) / (lo + 0.05);
}

function composite(fg: RGB, alpha: number, bg: RGB): RGB {
  return [
    Math.round(fg[0] * alpha + bg[0] * (1 - alpha)),
    Math.round(fg[1] * alpha + bg[1] * (1 - alpha)),
    Math.round(fg[2] * alpha + bg[2] * (1 - alpha)),
  ];
}

function stopPasses(mixed: RGB, text: RGB, alpha: number, surface: RGB): boolean {
  const comp = composite(mixed, alpha, surface);
  if (contrastRatio(text, comp) < MIN_TEXT_CONTRAST) return false;
  const dimOnStop = composite(text, DIM_ALPHA, comp);
  return contrastRatio(dimOnStop, comp) >= MIN_DIM_CONTRAST;
}

/**
 * Nudge a mixed stop's lightness (hue/saturation untouched) until the stop
 * composited at `alpha` over the theme surface clears WCAG contrast for both
 * the text color and the dimmed variant — moving AWAY from the text pole.
 */
export function ensureContrast(mixed: RGB, theme: Theme, alpha: number): RGB {
  const text = TEXT[theme] ?? TEXT.dark;
  const surface = SURFACE[theme] ?? SURFACE.dark;
  if (stopPasses(mixed, text, alpha, surface)) return mixed;
  const textLight = relativeLuminance(text) > 0.5;
  const [h, s, l0] = rgbToHsl(mixed);
  const step = textLight ? -0.04 : 0.04;
  let l = l0;
  for (let i = 0; i < 25; i++) {
    l = Math.min(1, Math.max(0, l + step));
    const candidate = hslToRgb(h, s, l);
    if (stopPasses(candidate, text, alpha, surface)) return candidate;
    if (l === 0 || l === 1) return candidate;
  }
  return hslToRgb(h, s, l);
}

/** Like artGradient, but every stop is lightness-clamped to keep the playbar
 *  text/icons readable (Step 2b contrast guarantee). */
export function artGradientContrast(
  c1hex: string,
  c2hex: string,
  theme: Theme,
  alpha: number,
): string | null {
  const c1 = hexToRgb(c1hex);
  const c2 = c1 ? hexToRgb(c2hex) : null;
  if (!c1 || !c2) return null;
  const s1 = ensureContrast(mixedColor(c1, theme), theme, alpha);
  const s2 = ensureContrast(mixedColor(c2, theme), theme, alpha);
  return `linear-gradient(135deg, rgba(${s1.join(", ")}, ${alpha}), rgba(${s2.join(", ")}, ${alpha}))`;
}
