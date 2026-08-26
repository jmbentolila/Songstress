/** Accent color derivation (Step 4): ONE stored hex → per-theme CSS
 *  variable overrides. Unset (null) = stock purple from app.css. */

export interface AccentVariants {
  /** Main accent (lightened for dark theme, darkened for light). */
  accent: string;
  /** Selection/active wash — accent at the user-tuned theme alpha. */
  active: string;
  /** Text color on accent surfaces: white or near-black by luminance. */
  accentText: string;
}

export const ACCENT_PRESETS: { name: string; hex: string | null }[] = [
  { name: "Stock purple", hex: null },
  { name: "Magenta", hex: "#d6409f" },
  { name: "Red", hex: "#e5484d" },
  { name: "Orange", hex: "#f76b15" },
  { name: "Amber", hex: "#ffb224" },
  { name: "Green", hex: "#46a758" },
  { name: "Teal", hex: "#12a594" },
  { name: "Cyan", hex: "#00a2c7" },
  { name: "Blue", hex: "#3b82f6" },
  { name: "Black", hex: "#000000" },
  { name: "White", hex: "#ffffff" },
];

/** #rgb/#rrggbb → [h° 0-360, s 0-1, l 0-1]. Throws on garbage (fail loud in
 *  dev; the picker only ever produces valid hexes). */
export function hexToHsl(hex: string): [number, number, number] {
  let h = hex.replace("#", "");
  if (h.length === 3) {
    h = h
      .split("")
      .map((c) => c + c)
      .join("");
  }
  if (!/^[0-9a-fA-F]{6}$/.test(h)) throw new Error(`bad hex: ${hex}`);
  const r = parseInt(h.slice(0, 2), 16) / 255;
  const g = parseInt(h.slice(2, 4), 16) / 255;
  const b = parseInt(h.slice(4, 6), 16) / 255;
  const max = Math.max(r, g, b);
  const min = Math.min(r, g, b);
  const l = (max + min) / 2;
  if (max === min) return [0, 0, l];
  const d = max - min;
  const s = d / (1 - Math.abs(2 * l - 1));
  let hh: number;
  if (max === r) hh = ((g - b) / d) % 6;
  else if (max === g) hh = (b - r) / d + 2;
  else hh = (r - g) / d + 4;
  hh *= 60;
  if (hh < 0) hh += 360;
  return [hh, s, l];
}

export function hslToHex(h: number, s: number, l: number): string {
  const hn = ((h % 360) + 360) % 360;
  const sn = Math.min(1, Math.max(0, s));
  const ln = Math.min(1, Math.max(0, l));
  const c = (1 - Math.abs(2 * ln - 1)) * sn;
  const x = c * (1 - Math.abs(((hn / 60) % 2) - 1));
  const m = ln - c / 2;
  let rgb: [number, number, number];
  if (hn < 60) rgb = [c, x, 0];
  else if (hn < 120) rgb = [x, c, 0];
  else if (hn < 180) rgb = [0, c, x];
  else if (hn < 240) rgb = [0, x, c];
  else if (hn < 300) rgb = [x, 0, c];
  else rgb = [c, 0, x];
  const to = (v: number) =>
    Math.round((v + m) * 255)
      .toString(16)
      .padStart(2, "0");
  return `#${to(rgb[0])}${to(rgb[1])}${to(rgb[2])}`;
}

/** WCAG relative luminance 0-1. */
export function relativeLuminance(hex: string): number {
  const h = hex.replace("#", "");
  const ch = (i: number) => {
    const v = parseInt(h.slice(i, i + 2), 16) / 255;
    return v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4;
  };
  return 0.2126 * ch(0) + 0.7152 * ch(2) + 0.0722 * ch(4);
}

const CLAMP = {
  dark: [0.7, 0.8], // light theme reads better slightly deep
  light: [0.45, 0.6],
} as const;

const ACTIVE_ALPHA = { dark: 0.18, light: 0.14 } as const; // user-tuned

/** Derive the theme's accent variables from one stored hex. */
export function accentVariants(hex: string, theme: "dark" | "light"): AccentVariants {
  const [lo, hi] = CLAMP[theme];
  const [h, s, l0] = hexToHsl(hex);
  const l = Math.min(hi, Math.max(lo, l0));
  const accent = hslToHex(h, s, l);
  const r = parseInt(accent.slice(1, 3), 16);
  const g = parseInt(accent.slice(3, 5), 16);
  const b = parseInt(accent.slice(5, 7), 16);
  return {
    accent,
    active: `rgba(${r}, ${g}, ${b}, ${ACTIVE_ALPHA[theme]})`,
    accentText: relativeLuminance(accent) > 0.5 ? "#1b1b1f" : "#ffffff",
  };
}
