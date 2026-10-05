import { describe, expect, it } from "vitest";
import { buildEffect, DEFAULT_PARAMS, effectSchema, fieldsFor, paramsFromEffect } from "./effects";

describe("effects", () => {
  it("builds a payload the daemon's schema accepts for every effect", () => {
    for (const name of ["static", "breathe", "pulse", "rainbow", "comet"] as const) {
      const effect = buildEffect(name, DEFAULT_PARAMS);

      expect(effectSchema.parse(effect)).toEqual(effect);
    }
  });

  it("round-trips params through an effect", () => {
    const params = { colour: "#ff0000", periodMs: 2500, spread: 1 };

    const back = paramsFromEffect(buildEffect("breathe", params));

    expect(back).toEqual(params);
  });

  it("rainbow ignores colour and static ignores period", () => {
    expect(fieldsFor("rainbow")).toEqual({ colour: false, period: true, spread: true });
    expect(fieldsFor("static")).toEqual({ colour: true, period: false, spread: false });
  });

  it("rejects an unknown effect name", () => {
    expect(effectSchema.safeParse({ effect: "disco" }).success).toBe(false);
  });
});
