import { EFFECT_NAMES, type EffectName, fieldsFor, type Params, PERIOD_RANGE } from "./effects";

interface Props {
  name: EffectName;
  params: Params;
  onChange: (name: EffectName, params: Params) => void;
}

export function EffectPanel({ name, params, onChange }: Props) {
  const fields = fieldsFor(name);
  return (
    <div className="flex flex-col gap-5">
      <ul className="border-y border-line" aria-label="Effects">
        {EFFECT_NAMES.map((n) => (
          <li key={n}>
            <button
              type="button"
              aria-pressed={n === name}
              onClick={() => onChange(n, params)}
              className={`flex w-full items-center justify-between border-b border-line px-3 py-2 text-left text-sm uppercase tracking-wide last:border-b-0 ${
                n === name ? "bg-rog text-white" : "hover:bg-raised"
              }`}
            >
              <span>{n}</span>
              {n === name && <span className="text-[10px] tracking-widest">ACTIVE</span>}
            </button>
          </li>
        ))}
      </ul>

      {fields.period && (
        <label className="block text-xs uppercase tracking-widest text-mute">
          Cycle {(params.periodMs / 1000).toFixed(1)}s
          <input
            type="range"
            aria-label="Cycle length"
            {...PERIOD_RANGE}
            value={params.periodMs}
            onChange={(e) => onChange(name, { ...params, periodMs: Number(e.target.value) })}
          />
        </label>
      )}
      {fields.spread && (
        <label className="block text-xs uppercase tracking-widest text-mute">
          Spread {params.spread.toFixed(1)}
          <input
            type="range"
            aria-label="Hue spread"
            min={0}
            max={3}
            step={0.1}
            value={params.spread}
            onChange={(e) => onChange(name, { ...params, spread: Number(e.target.value) })}
          />
        </label>
      )}
    </div>
  );
}
