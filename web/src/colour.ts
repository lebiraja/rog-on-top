export interface Hsv {
  h: number; // 0..360
  s: number; // 0..1
  v: number; // 0..1
}

const HEX = /^#[0-9a-fA-F]{6}$/;

export const isHex = (v: string): boolean => HEX.test(v);

const byte = (n: number): number => Math.min(255, Math.max(0, Math.round(n)));

export function hexToRgb(hex: string): [number, number, number] {
  const n = Number.parseInt(hex.slice(1), 16);
  return [(n >> 16) & 255, (n >> 8) & 255, n & 255];
}

export function rgbToHex(r: number, g: number, b: number): string {
  return `#${[r, g, b].map((c) => byte(c).toString(16).padStart(2, "0")).join("")}`;
}

export function hsvToHex({ h, s, v }: Hsv): string {
  const f = (n: number): number => {
    const k = (n + h / 60) % 6;
    return v - v * s * Math.max(0, Math.min(k, 4 - k, 1));
  };
  return rgbToHex(f(5) * 255, f(3) * 255, f(1) * 255);
}

export function hexToHsv(hex: string): Hsv {
  const [r, g, b] = hexToRgb(hex).map((c) => c / 255) as [number, number, number];
  const max = Math.max(r, g, b);
  const d = max - Math.min(r, g, b);
  let h = 0;
  if (d !== 0) {
    if (max === r) h = ((g - b) / d) % 6;
    else if (max === g) h = (b - r) / d + 2;
    else h = (r - g) / d + 4;
  }
  return { h: (h * 60 + 360) % 360, s: max === 0 ? 0 : d / max, v: max };
}

/** Blackbody approximation (Tanner Helland), good enough for warm to cool white. */
export function kelvinToHex(kelvin: number): string {
  const t = Math.min(10000, Math.max(1000, kelvin)) / 100;
  const r = t <= 66 ? 255 : 329.698727446 * (t - 60) ** -0.1332047592;
  const g = t <= 66 ? 99.4708025861 * Math.log(t) - 161.1195681661 : 288.1221695283 * (t - 60) ** -0.0755148492;
  const b = t >= 66 ? 255 : t <= 19 ? 0 : 138.5177312231 * Math.log(t - 10) - 305.0447927307;
  return rgbToHex(r, g, b);
}

export function mixHex(a: string, b: string, k: number): string {
  const [ar, ag, ab] = hexToRgb(a);
  const [br, bg, bb] = hexToRgb(b);
  return rgbToHex(ar + (br - ar) * k, ag + (bg - ag) * k, ab + (bb - ab) * k);
}

/** `count` colours spread evenly along the stops (at least one stop). */
export function gradient(stops: readonly string[], count: number): string[] {
  const first = stops[0];
  if (first === undefined) return [];
  if (stops.length === 1 || count <= 1) return Array.from({ length: count }, () => first);
  return Array.from({ length: count }, (_, i) => {
    const pos = (i / (count - 1)) * (stops.length - 1);
    const lo = Math.min(Math.floor(pos), stops.length - 2);
    return mixHex(stops[lo] ?? first, stops[lo + 1] ?? first, pos - lo);
  });
}
