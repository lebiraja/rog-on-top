import layoutData from "./layout.json";

export interface KeyShape {
  id: string;
  label: string;
  x: number;
  y: number;
  w: number;
  h: number;
}

export interface Layout {
  width: number;
  height: number;
  keys: KeyShape[];
}

export const layout: Layout = layoutData;

export const KEY_LEDS = ["key1", "key2", "key3", "key4"] as const;
export const BAR_LEDS = ["bar1", "bar2", "bar3", "bar4", "bar5", "bar6"] as const;
export const ALL_LEDS: readonly string[] = [...KEY_LEDS, ...BAR_LEDS];

/** The keyboard has four hardware zones, left to right; a key belongs to the zone under its centre. */
export function zoneOf(key: Pick<KeyShape, "x" | "w">, width: number = layout.width): (typeof KEY_LEDS)[number] {
  const index = Math.min(3, Math.max(0, Math.floor(((key.x + key.w / 2) / width) * 4)));
  return KEY_LEDS[index] ?? "key1";
}

/** Add or remove LED names from a selection (shift/ctrl click), or replace it. */
export function select(current: ReadonlySet<string>, leds: readonly string[], additive: boolean): Set<string> {
  if (!additive) return new Set(leds);
  const next = new Set(current);
  const allIn = leds.every((l) => next.has(l));
  for (const l of leds) {
    if (allIn) next.delete(l);
    else next.add(l);
  }
  return next;
}
