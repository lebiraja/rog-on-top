import { describe, expect, it } from "vitest";
import { layout, select, zoneOf } from "./zones";

describe("zones", () => {
  it("assigns the far-left key to zone 1 and the far-right key to zone 4", () => {
    const left = layout.keys.reduce((a, b) => (a.x < b.x ? a : b));
    const right = layout.keys.reduce((a, b) => (a.x + a.w > b.x + b.w ? a : b));

    expect(zoneOf(left)).toBe("key1");
    expect(zoneOf(right)).toBe("key4");
  });

  it("splits the width into four equal bands", () => {
    expect(zoneOf({ x: 4.9, w: 0.1 }, 20)).toBe("key1");
    expect(zoneOf({ x: 5, w: 0 }, 20)).toBe("key2");
    expect(zoneOf({ x: 10, w: 0 }, 20)).toBe("key3");
    expect(zoneOf({ x: 15, w: 0 }, 20)).toBe("key4");
  });

  it("uses every zone in the real layout", () => {
    const zones = new Set(layout.keys.map((k) => zoneOf(k)));

    expect(zones.size).toBe(4);
  });

  it("replaces the selection unless additive", () => {
    expect([...select(new Set(["key1"]), ["key2"], false)]).toEqual(["key2"]);
  });

  it("additive select adds, then removes when all are already in", () => {
    const added = select(new Set(["key1"]), ["key2"], true);
    const removed = select(added, ["key2"], true);

    expect([...added].sort()).toEqual(["key1", "key2"]);
    expect([...removed]).toEqual(["key1"]);
  });
});
