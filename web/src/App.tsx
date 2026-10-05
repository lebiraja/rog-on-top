import { useCallback, useEffect, useRef, useState } from "react";
import {
  applyPreset,
  deletePreset,
  getFrame,
  getPresets,
  getState,
  type Paint,
  type Preset,
  putBrightness,
  putEffect,
  putPaint,
  savePreset,
  setDefaultPreset,
  updatePreset,
} from "./api";
import { ColourTools, loadRecents, pushRecent } from "./ColourTools";
import { ColourWheel } from "./ColourWheel";
import { EffectPanel } from "./EffectPanel";
import { buildEffect, DEFAULT_PARAMS, type EffectName, fieldsFor, type Params, paramsFromEffect } from "./effects";
import { GradientBuilder } from "./GradientBuilder";
import { Keyboard } from "./Keyboard";
import { PresetPanel } from "./PresetPanel";
import { ALL_LEDS, BAR_LEDS, KEY_LEDS, select } from "./zones";

const PUSH_DELAY_MS = 120;
const POLL_MS = 100;

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <section className="border border-line bg-panel p-4">
      <h2 className="mb-3 text-xs font-bold uppercase tracking-[0.2em] text-rog">{title}</h2>
      {children}
    </section>
  );
}

function Chip({ onClick, children, disabled }: { onClick: () => void; children: React.ReactNode; disabled?: boolean }) {
  return (
    <button
      type="button"
      onClick={onClick}
      disabled={disabled}
      className="border border-line px-3 py-1.5 text-[11px] uppercase tracking-widest hover:border-rog hover:text-rog disabled:opacity-30 disabled:hover:border-line disabled:hover:text-inherit"
    >
      {children}
    </button>
  );
}

export function App() {
  const [name, setName] = useState<EffectName>("static");
  const [params, setParams] = useState<Params>(DEFAULT_PARAMS);
  const [brightness, setBrightness] = useState(100);
  const [paint, setPaint] = useState<Paint>({});
  const [selected, setSelected] = useState<ReadonlySet<string>>(new Set());
  const [colour, setColour] = useState("#ffffff");
  const [recents, setRecents] = useState<string[]>(loadRecents);
  const [presets, setPresets] = useState<Preset[]>([]);
  const [defaultId, setDefaultId] = useState<string | null>(null);
  const [live, setLive] = useState<Record<string, string>>({});
  const [device, setDevice] = useState("—");
  const [error, setError] = useState<string | null>(null);
  const timers = useRef(new Map<string, ReturnType<typeof setTimeout>>());

  const push = useCallback((key: string, work: () => Promise<void>) => {
    clearTimeout(timers.current.get(key));
    timers.current.set(
      key,
      setTimeout(() => {
        work().then(
          () => setError(null),
          () => setError("rogd rejected the change"),
        );
      }, PUSH_DELAY_MS),
    );
  }, []);

  const loadState = useCallback(async () => {
    const s = await getState();
    setName(s.effect.effect);
    setParams((p) => paramsFromEffect(s.effect, p));
    setBrightness(s.brightness);
    setPaint(s.paint);
  }, []);

  const loadPresets = useCallback(async () => {
    const p = await getPresets();
    setPresets(p.presets);
    setDefaultId(p.default);
  }, []);

  useEffect(() => {
    Promise.all([loadState(), loadPresets()]).catch(() => setError("cannot reach rogd"));
  }, [loadState, loadPresets]);

  useEffect(() => {
    let alive = true;
    const tick = () =>
      getFrame().then(
        (f) => {
          if (!alive) return;
          setLive(Object.fromEntries(f.leds.map((l) => [l.name, l.colour])));
          setDevice(f.device);
          setError((e) => (e === "rogd offline" ? null : e));
        },
        () => alive && setError("rogd offline"),
      );
    void tick();
    const id = setInterval(tick, POLL_MS);
    return () => {
      alive = false;
      clearInterval(id);
    };
  }, []);

  const setEffect = (nextName: EffectName, nextParams: Params) => {
    setName(nextName);
    setParams(nextParams);
    push("effect", () => putEffect(buildEffect(nextName, nextParams)));
  };

  const commitPaint = (next: Paint) => {
    setPaint(next);
    push("paint", () => putPaint(next));
  };

  const remember = (hex: string) => push("recent", async () => setRecents((r) => pushRecent(r, hex)));

  const pickColour = (hex: string) => {
    setColour(hex);
    remember(hex);
    if (selected.size > 0) {
      commitPaint({ ...paint, ...Object.fromEntries([...selected].map((led) => [led, hex])) });
    } else if (fieldsFor(name).colour) {
      setEffect(name, { ...params, colour: hex });
    } else {
      setEffect("static", { ...params, colour: hex });
    }
  };

  const changeSelection = (leds: readonly string[], additive: boolean) => {
    const next = select(selected, leds, additive);
    setSelected(next);
    const first = ALL_LEDS.find((l) => next.has(l));
    if (first) setColour(paint[first] ?? params.colour);
  };

  const clearPaint = (all: boolean) => {
    const keep = Object.entries(paint).filter(([led]) => !all && !selected.has(led));
    commitPaint(Object.fromEntries(keep));
  };

  const paintGradient = (colours: string[]) => {
    const order = ALL_LEDS.filter((l) => selected.has(l));
    commitPaint({ ...paint, ...Object.fromEntries(order.map((led, i) => [led, colours[i] ?? colour])) });
  };

  const run = (work: Promise<unknown>) => work.then(() => setError(null), () => setError("rogd rejected the change"));

  const apply = (id: string) =>
    run(applyPreset(id).then(loadState)).then(() => setSelected(new Set()));

  const favourites = presets.filter((p) => p.favourite);

  return (
    <div className="mx-auto flex max-w-[1500px] flex-col gap-4 p-4">
      <header className="flex flex-wrap items-end justify-between gap-3 border-b-2 border-rog pb-3">
        <div>
          <h1 className="text-3xl font-black uppercase italic tracking-tight">
            ROG<span className="text-rog">/</span>ON TOP
          </h1>
          <p className="text-[11px] uppercase tracking-[0.25em] text-mute">{device}</p>
        </div>
        <nav aria-label="Favourite presets" className="flex flex-wrap items-center gap-2">
          {favourites.map((p) => (
            <Chip key={p.id} onClick={() => void apply(p.id)}>
              ★ {p.name}
            </Chip>
          ))}
        </nav>
      </header>

      <div className="grid gap-4 lg:grid-cols-[280px_minmax(0,1fr)_320px]">
        <div className="order-2 flex flex-col gap-4 lg:order-1">
          <Section title="Effect">
            <EffectPanel name={name} params={params} onChange={setEffect} />
          </Section>
          <Section title="Brightness">
            <label className="block text-xs uppercase tracking-widest text-mute">
              {brightness}%
              <input
                type="range"
                aria-label="Brightness"
                min={0}
                max={100}
                value={brightness}
                onChange={(e) => {
                  const level = Number(e.target.value);
                  setBrightness(level);
                  push("brightness", () => putBrightness(level));
                }}
              />
            </label>
          </Section>
        </div>

        <div className="order-1 flex flex-col gap-4 lg:order-2">
          <Section title="Keyboard & lightbar">
            <Keyboard colours={live} selected={selected} onSelect={changeSelection} />
            <div className="mt-3 flex flex-wrap items-center gap-2">
              <Chip onClick={() => changeSelection(KEY_LEDS, false)}>All keys</Chip>
              <Chip onClick={() => changeSelection(BAR_LEDS, false)}>Lightbar</Chip>
              <Chip onClick={() => changeSelection(ALL_LEDS, false)}>Everything</Chip>
              <Chip onClick={() => setSelected(new Set())} disabled={selected.size === 0}>
                Deselect
              </Chip>
              <span className="mx-1 h-5 w-px bg-line" />
              <Chip onClick={() => clearPaint(false)} disabled={selected.size === 0}>
                Clear paint on selection
              </Chip>
              <Chip onClick={() => clearPaint(true)} disabled={Object.keys(paint).length === 0}>
                Clear all paint
              </Chip>
            </div>
            <p className="mt-3 text-xs text-mute">
              {selected.size === 0
                ? "Nothing selected: colour changes edit the effect. Click a key to pick its zone, shift-click to add more."
                : `${selected.size} selected: colour changes paint ${[...selected].join(", ")}.`}{" "}
              This keyboard has 4 hardware zones, so every key in a zone shares one colour. Paint replaces the effect on those LEDs.
            </p>
          </Section>

          <Section title="Presets">
            <PresetPanel
              presets={presets}
              defaultId={defaultId}
              onSave={(n) => void run(savePreset(n).then(loadPresets))}
              onApply={(id) => void apply(id)}
              onToggleFavourite={(p) => void run(updatePreset(p.id, { favourite: !p.favourite }).then(loadPresets))}
              onRename={(id, n) => void run(updatePreset(id, { name: n }).then(loadPresets))}
              onDelete={(id) => void run(deletePreset(id).then(loadPresets))}
              onSetDefault={(id) => void run(setDefaultPreset(id).then(loadPresets))}
            />
          </Section>
        </div>

        <div className="order-3 flex flex-col gap-4">
          <Section title="Colour">
            <ColourWheel value={colour} onChange={pickColour} />
          </Section>
          <Section title="Tools">
            <ColourTools recents={recents} onPick={pickColour} />
          </Section>
          <Section title="Gradient">
            <GradientBuilder current={colour} selectionCount={selected.size} onApply={paintGradient} />
          </Section>
        </div>
      </div>

      <footer className="min-h-5 text-xs uppercase tracking-widest text-rog" role="status">
        {error}
      </footer>
    </div>
  );
}
