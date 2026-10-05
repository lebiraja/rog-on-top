import { z } from "zod";
import { type Effect, effectSchema } from "./effects";

const hex = z.string().regex(/^#[0-9a-fA-F]{6}$/);
export const paintSchema = z.record(z.string(), hex);
export type Paint = z.infer<typeof paintSchema>;

const stateSchema = z.object({
  status: z.literal("state"),
  effect: effectSchema,
  brightness: z.number(),
  paint: paintSchema,
});

const frameSchema = z.object({
  device: z.string(),
  leds: z.array(z.object({ name: z.string(), group: z.string(), colour: hex })),
});

const presetSchema = z.object({
  id: z.string(),
  name: z.string(),
  favourite: z.boolean(),
  look: z.object({ effect: effectSchema, brightness: z.number(), paint: paintSchema }),
});

const presetsSchema = z.object({
  status: z.literal("presets"),
  presets: z.array(presetSchema),
  default: z.string().nullable(),
});

const savedSchema = z.object({ status: z.literal("preset"), preset: presetSchema });

export type Frame = z.infer<typeof frameSchema>;
export type State = z.infer<typeof stateSchema>;
export type Preset = z.infer<typeof presetSchema>;
export type Presets = z.infer<typeof presetsSchema>;

async function request(path: string, init?: { method?: string; body?: unknown }): Promise<unknown> {
  const res = await fetch(`/api/v1${path}`, {
    method: init?.method,
    body: init?.body === undefined ? undefined : JSON.stringify(init.body),
    headers: init?.body === undefined ? undefined : { "content-type": "application/json" },
  });
  if (!res.ok) throw new Error(`${path}: ${res.status}`);
  return res.json();
}

const send = async (path: string, method: string, body?: unknown): Promise<void> => {
  await request(path, { method, body });
};

export const getState = async (): Promise<State> => stateSchema.parse(await request("/state"));
export const getFrame = async (): Promise<Frame> => frameSchema.parse(await request("/frame"));
export const putEffect = (effect: Effect): Promise<void> => send("/effect", "PUT", effect);
export const putBrightness = (level: number): Promise<void> => send("/brightness", "PUT", { level });
export const putPaint = (paint: Paint): Promise<void> => send("/paint", "PUT", paint);

export const getPresets = async (): Promise<Presets> => presetsSchema.parse(await request("/presets"));
export const savePreset = async (name: string): Promise<Preset> =>
  savedSchema.parse(await request("/presets", { method: "POST", body: { name } })).preset;
export const applyPreset = (id: string): Promise<void> => send(`/presets/${encodeURIComponent(id)}/apply`, "POST");
export const deletePreset = (id: string): Promise<void> => send(`/presets/${encodeURIComponent(id)}`, "DELETE");
export const updatePreset = (id: string, patch: { name?: string; favourite?: boolean }): Promise<void> =>
  send(`/presets/${encodeURIComponent(id)}`, "PATCH", patch);
export const setDefaultPreset = (id: string | null): Promise<void> => send("/default-preset", "PUT", { id });
