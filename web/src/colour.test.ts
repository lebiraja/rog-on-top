import { describe, expect, it } from "vitest";
import { gradient, hexToHsv, hsvToHex, kelvinToHex, mixHex, rgbToHex } from "./colour";

describe("colour", () => {
  it("converts HSV primaries to hex", () => {
    expect(hsvToHex({ h: 0, s: 1, v: 1 })).toBe("#ff0000");
    expect(hsvToHex({ h: 120, s: 1, v: 1 })).toBe("#00ff00");
    expect(hsvToHex({ h: 240, s: 1, v: 1 })).toBe("#0000ff");
    expect(hsvToHex({ h: 0, s: 0, v: 0.5 })).toBe("#808080");
  });

  it("round-trips hex through HSV", () => {
    for (const hex of ["#ff7a00", "#00ffd5", "#9b3dff", "#808080", "#000000"]) {
      expect(hsvToHex(hexToHsv(hex))).toBe(hex);
    }
  });

  it("clamps channels when building hex", () => {
    expect(rgbToHex(300, -4, 15.6)).toBe("#ff0010");
  });

  it("maps kelvin from warm orange to cool blue-white", () => {
    expect(kelvinToHex(6600)).toBe("#ffffff");
    expect(kelvinToHex(2000) < kelvinToHex(6600)).toBe(true);
    const warm = kelvinToHex(2700);
    const cool = kelvinToHex(9000);
    expect(warm.slice(1, 3)).toBe("ff");
    expect(cool.slice(5, 7)).toBe("ff");
  });

  it("mixes two colours", () => {
    expect(mixHex("#000000", "#ffffff", 0.5)).toBe("#808080");
  });

  it("spreads a gradient across the requested count with exact end stops", () => {
    const g = gradient(["#ff0000", "#0000ff"], 5);

    expect(g).toHaveLength(5);
    expect(g[0]).toBe("#ff0000");
    expect(g[4]).toBe("#0000ff");
    expect(g[2]).toBe("#800080");
  });

  it("handles one stop and zero count", () => {
    expect(gradient(["#123456"], 3)).toEqual(["#123456", "#123456", "#123456"]);
    expect(gradient([], 3)).toEqual([]);
    expect(gradient(["#ff0000", "#00ff00"], 0)).toEqual([]);
  });

  it("walks through three stops", () => {
    expect(gradient(["#ff0000", "#00ff00", "#0000ff"], 3)).toEqual(["#ff0000", "#00ff00", "#0000ff"]);
  });
});
