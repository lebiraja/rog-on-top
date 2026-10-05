import { z } from "zod";

const hex = z.string().regex(/^#[0-9a-fA-F]{6}$/);

export const effectSchema = z.discriminatedUnion("effect", [
  z.object({ effect: z.literal("static"), colour: hex }),
  z.object({ effect: z.literal("breathe"), colour: hex, period_ms: z.number() }),
  z.object({ effect: z.literal("pulse"), colour: hex, period_ms: z.number() }),
  z.object({ effect: z.literal("rainbow"), period_ms: z.number(), spread: z.number() }),
  z.object({
    effect: z.literal("comet"),
    colour: hex,
    period_ms: z.number(),
    tail: z.number(),
    floor: z.number(),
    tail_colour: hex.nullable(),
  }),
]);

export type Effect = z.infer<typeof effectSchema>;
export type EffectName = Effect["effect"];

export const EFFECT_NAMES: readonly EffectName[] = ["static", "breathe", "pulse", "rainbow", "comet"];

export interface Params {
  colour: string;
  periodMs: number;
  spread: number;
}

export const DEFAULT_PARAMS: Params = { colour: "#ffffff", periodMs: 4000, spread: 1 };

export const PERIOD_RANGE = { min: 500, max: 12000, step: 100 } as const;

export function fieldsFor(name: EffectName): { colour: boolean; period: boolean; spread: boolean } {
  return { colour: name !== "rainbow", period: name !== "static", spread: name === "rainbow" };
}

export function buildEffect(name: EffectName, p: Params): Effect {
  switch (name) {
    case "static":
      return { effect: "static", colour: p.colour };
    case "breathe":
      return { effect: "breathe", colour: p.colour, period_ms: p.periodMs };
    case "pulse":
      return { effect: "pulse", colour: p.colour, period_ms: p.periodMs };
    case "rainbow":
      return { effect: "rainbow", period_ms: p.periodMs, spread: p.spread };
    case "comet":
      return { effect: "comet", colour: p.colour, period_ms: p.periodMs, tail: 0.45, floor: 0.05, tail_colour: null };
  }
}

export function paramsFromEffect(e: Effect, fallback: Params = DEFAULT_PARAMS): Params {
  switch (e.effect) {
    case "static":
      return { ...fallback, colour: e.colour };
    case "breathe":
    case "pulse":
    case "comet":
      return { ...fallback, colour: e.colour, periodMs: e.period_ms };
    case "rainbow":
      return { ...fallback, periodMs: e.period_ms, spread: e.spread };
  }
}
